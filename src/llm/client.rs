//! HTTP client for communicating with LLM endpoints.
//!
//! Supports two provider types:
//! - Ollama: Local LLM inference via `/api/generate`
//! - OpenAI-compatible: Any endpoint implementing the OpenAI chat completions API

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Supported LLM provider types.
#[derive(Debug, Clone)]
pub enum LlmProviderType {
    /// Local Ollama server (default endpoint: http://localhost:11434)
    Ollama,
    /// OpenAI-compatible API (default endpoint: http://localhost:1234 for LM Studio)
    OpenAICompatible,
}

/// Configuration for connecting to an LLM endpoint.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProviderType,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

impl LlmConfig {
    /// Build configuration from CLI arguments with sensible defaults.
    ///
    /// - Ollama default endpoint: `http://localhost:11434`, model: `llama3.2`
    /// - OpenAI-compatible default endpoint: `http://localhost:1234`, model: `default`
    pub fn from_args(
        provider: &str,
        endpoint: Option<&str>,
        model: Option<&str>,
        api_key: Option<&str>,
        timeout: u64,
    ) -> Self {
        let provider_type = match provider.to_lowercase().as_str() {
            "ollama" => LlmProviderType::Ollama,
            _ => LlmProviderType::OpenAICompatible,
        };

        let default_endpoint = match provider_type {
            LlmProviderType::Ollama => "http://localhost:11434",
            LlmProviderType::OpenAICompatible => "http://localhost:1234",
        };

        let default_model = match provider_type {
            LlmProviderType::Ollama => "llama3.2",
            LlmProviderType::OpenAICompatible => "gpt-3.5-turbo",
        };

        Self {
            provider: provider_type,
            endpoint: endpoint.unwrap_or(default_endpoint).to_string(),
            model: model.unwrap_or(default_model).to_string(),
            api_key: api_key.map(String::from),
            timeout_secs: timeout,
        }
    }
}

// --- Ollama request/response types ---

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: Option<String>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

// --- OpenAI-compatible request/response types ---

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f64,
    response_format: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

// --- Client ---

/// HTTP client that communicates with LLM endpoints.
///
/// Creates a minimal tokio runtime for blocking HTTP calls.
/// Each call is a single request-response cycle.
pub struct LlmClient {
    config: LlmConfig,
}

impl LlmClient {
    /// Create a new client with the given configuration.
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    /// Send a prompt to the LLM and return the response text.
    ///
    /// This is a blocking call that creates a temporary tokio runtime.
    pub fn call_blocking(&self, prompt: &str) -> Result<String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("Failed to create tokio runtime")?;

        rt.block_on(self.call_async(prompt))
    }

    async fn call_async(&self, prompt: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                self.config.timeout_secs.max(120),
            ))
            .build()
            .context("Failed to build HTTP client")?;

        match self.config.provider {
            LlmProviderType::Ollama => self.call_ollama(&client, prompt).await,
            LlmProviderType::OpenAICompatible => self.call_openai_compatible(&client, prompt).await,
        }
    }

    async fn call_ollama(&self, client: &reqwest::Client, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.config.endpoint);

        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            // Don't force JSON format — some models (gemma, etc.) return empty
            // responses when the json format flag is set. Instead, we instruct
            // JSON output in the prompt and parse it from the free-form response.
            format: None,
        };

        tracing::debug!(
            "Ollama request: model={}, endpoint={}",
            self.config.model,
            self.config.endpoint
        );

        let resp = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        let body: OllamaResponse = resp
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        tracing::debug!(
            "Ollama raw response ({} chars): {}",
            body.response.len(),
            &body.response[..body.response.len().min(500)]
        );

        Ok(body.response)
    }

    async fn call_openai_compatible(
        &self,
        client: &reqwest::Client,
        prompt: &str,
    ) -> Result<String> {
        let url = format!("{}/v1/chat/completions", self.config.endpoint);

        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "You are a sarcastic code reviewer. Always respond with valid JSON."
                    .to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages,
            temperature: 0.8,
            response_format: Some(serde_json::json!({"type": "json_object"})),
        };

        let mut req_builder = client.post(&url).json(&request);

        if let Some(ref api_key) = self.config.api_key {
            req_builder = req_builder.bearer_auth(api_key);
        }

        let resp = req_builder
            .send()
            .await
            .context("Failed to send request to OpenAI-compatible endpoint")?;

        let body: OpenAIResponse = resp
            .json()
            .await
            .context("Failed to parse OpenAI-compatible response")?;

        body.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("No choices in LLM response"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults_for_ollama() {
        // Objective: Verify Ollama config has correct default endpoint and model.
        // Invariants: Default endpoint must be localhost:11434, model must be llama3.2.
        let config = LlmConfig::from_args("ollama", None, None, None, 30);

        assert!(
            matches!(config.provider, LlmProviderType::Ollama),
            "Provider type must be Ollama"
        );
        assert_eq!(
            config.endpoint, "http://localhost:11434",
            "Default Ollama endpoint must be localhost:11434"
        );
        assert_eq!(
            config.model, "llama3.2",
            "Default Ollama model must be llama3.2"
        );
        assert!(
            config.api_key.is_none(),
            "Ollama should not require an API key"
        );
    }

    #[test]
    fn test_config_defaults_for_openai_compatible() {
        // Objective: Verify OpenAI-compatible config has correct defaults.
        // Invariants: Default endpoint must be localhost:1234.
        let config = LlmConfig::from_args("openai-compatible", None, None, None, 30);

        assert!(
            matches!(config.provider, LlmProviderType::OpenAICompatible),
            "Provider type must be OpenAICompatible"
        );
        assert_eq!(
            config.endpoint, "http://localhost:1234",
            "Default OpenAI-compatible endpoint must be localhost:1234"
        );
    }

    #[test]
    fn test_config_overrides_defaults() {
        // Objective: Verify custom values override defaults.
        // Invariants: All custom values must be preserved exactly.
        let config = LlmConfig::from_args(
            "ollama",
            Some("http://custom:9999"),
            Some("mistral"),
            Some("sk-test"),
            60,
        );

        assert_eq!(config.endpoint, "http://custom:9999");
        assert_eq!(config.model, "mistral");
        assert_eq!(config.api_key.as_deref(), Some("sk-test"));
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_config_unknown_provider_defaults_to_openai_compatible() {
        // Objective: Verify unknown provider strings default to OpenAI-compatible.
        // Invariants: Any non-"ollama" string must produce OpenAICompatible variant.
        let config = LlmConfig::from_args("lmstudio", None, None, None, 30);
        assert!(
            matches!(config.provider, LlmProviderType::OpenAICompatible),
            "Unknown provider '{}' should default to OpenAICompatible",
            "lmstudio"
        );
    }
}
