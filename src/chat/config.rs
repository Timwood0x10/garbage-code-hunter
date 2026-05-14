use serde::Deserialize;
use std::path::Path;

use crate::llm::client::LlmConfig;

/// AI provider configuration for the chat room.
#[derive(Debug, Clone, Deserialize)]
pub struct AiProviderConfig {
    pub provider: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    /// Custom auth header name (e.g. "api-key"). Default: "Authorization" with Bearer prefix.
    pub auth_header: Option<String>,
    pub timeout_secs: Option<u64>,
}

impl AiProviderConfig {
    pub fn to_llm_config(&self, default_model: &str) -> LlmConfig {
        let ep = self.endpoint.as_deref();
        let model = self.model.as_deref().unwrap_or(default_model);
        let mut cfg = LlmConfig::from_args(
            &self.provider,
            ep,
            Some(model),
            self.api_key.as_deref(),
            self.timeout_secs.unwrap_or(120),
        );
        cfg.auth_header = self.auth_header.clone();
        cfg
    }
}

/// Top-level chat configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatConfig {
    /// Unified AI config (all AIs use this provider).
    pub ai: Option<AiProviderConfig>,
    /// Individual AI role configs with explicit role binding.
    pub ai_roles: Option<Vec<RoleBinding>>,
    /// List of providers to randomly assign to roles (no manual binding needed).
    pub providers: Option<Vec<AiProviderConfig>>,
    /// Ollama fallback (default).
    pub ollama: Option<AiProviderConfig>,
}

/// Bind a role name to a specific AI provider.
#[derive(Debug, Clone, Deserialize)]
pub struct RoleBinding {
    pub name: String,
    pub provider: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub auth_header: Option<String>,
    pub timeout_secs: Option<u64>,
}

impl RoleBinding {
    pub fn to_llm_config(&self) -> LlmConfig {
        let ep = self.endpoint.as_deref();
        let model = self.model.as_deref().unwrap_or("llama3.2:3b");
        let mut cfg = LlmConfig::from_args(
            &self.provider,
            ep,
            Some(model),
            self.api_key.as_deref(),
            self.timeout_secs.unwrap_or(120),
        );
        cfg.auth_header = self.auth_header.clone();
        cfg
    }
}

impl ChatConfig {
    /// Load config from standard locations.
    pub fn load() -> Self {
        let paths = [
            "./chat.yml",
            "./chat.yaml",
            "./.chat.yml",
            &dirs::config_dir()
                .map(|p| {
                    p.join("garbage-code-hunter/chat.yml")
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_default(),
            &dirs::config_dir()
                .map(|p| {
                    p.join("garbage-code-hunter/chat.yaml")
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_default(),
            ".env",
        ];

        for path in &paths {
            if path.is_empty() {
                continue;
            }
            let p = Path::new(path);
            if !p.exists() {
                continue;
            }
            if path.ends_with(".env") {
                return Self::from_dotenv(p);
            }
            if let Ok(content) = std::fs::read_to_string(p) {
                if let Ok(cfg) = serde_yaml::from_str::<ChatConfig>(&content) {
                    return cfg;
                }
            }
        }
        Self::default_ollama()
    }

    fn from_dotenv(path: &Path) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let mut provider = None;
        let mut endpoint = None;
        let mut model = None;
        let mut api_key = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_uppercase();
                let val = val.trim().to_string();
                match key.as_str() {
                    "AI_PROVIDER" => provider = Some(val),
                    "AI_ENDPOINT" => endpoint = Some(val),
                    "AI_MODEL" => model = Some(val),
                    "AI_API_KEY" => api_key = Some(val),
                    _ => {}
                }
            }
        }

        Self {
            ai: Some(AiProviderConfig {
                provider: provider.unwrap_or_else(|| "ollama".to_string()),
                endpoint,
                model,
                api_key,
                auth_header: None,
                timeout_secs: Some(120),
            }),
            ai_roles: None,
            providers: None,
            ollama: None,
        }
    }

    fn default_ollama() -> Self {
        Self {
            ai: Some(AiProviderConfig {
                provider: "ollama".to_string(),
                endpoint: Some("http://localhost:11434".to_string()),
                model: Some("llama3.2:3b".to_string()),
                api_key: None,
                auth_header: None,
                timeout_secs: Some(120),
            }),
            ai_roles: None,
            providers: None,
            ollama: None,
        }
    }

    /// Get config for a specific role index.
    /// Priority: ai_roles > providers[index % len] > ai (unified) > ollama fallback
    pub fn config_for_role(&self, index: usize, _total_roles: usize) -> LlmConfig {
        // 1) Explicit role bindings
        if let Some(roles) = &self.ai_roles {
            if index < roles.len() {
                return roles[index].to_llm_config();
            }
        }

        // 2) Providers list — randomly assigned, just cycle through them
        if let Some(providers) = &self.providers {
            if !providers.is_empty() {
                let idx = index % providers.len();
                let default_model = providers[idx].model.as_deref().unwrap_or("gpt-4o-mini");
                return providers[idx].to_llm_config(default_model);
            }
        }

        // 3) Unified mode
        if let Some(ai) = &self.ai {
            let model = ai.model.as_deref().unwrap_or("llama3.2:3b");
            return ai.to_llm_config(model);
        }

        // 4) Ollama fallback
        let model = self
            .ollama
            .as_ref()
            .and_then(|o| o.model.as_deref())
            .unwrap_or("llama3.2:3b");
        LlmConfig::from_args(
            "ollama",
            Some("http://localhost:11434"),
            Some(model),
            None,
            120,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::client::LlmProviderType;

    #[test]
    fn test_default_config() {
        let cfg = ChatConfig::default_ollama();
        assert_eq!(cfg.ai.unwrap().provider, "ollama".to_string());
    }

    #[test]
    fn test_dotenv_parsing() {
        let dir = tempfile::TempDir::new().unwrap();
        let env_path = dir.path().join(".env");
        std::fs::write(
            &env_path,
            "AI_PROVIDER=openai\nAI_API_KEY=sk-test123\nAI_MODEL=gpt-4o\n",
        )
        .unwrap();
        let cfg = ChatConfig::from_dotenv(&env_path);
        let ai = cfg.ai.unwrap();
        assert_eq!(ai.provider, "openai");
        assert_eq!(ai.api_key, Some("sk-test123".to_string()));
        assert_eq!(ai.model, Some("gpt-4o".to_string()));
    }

    #[test]
    fn test_role_config_fallback() {
        let cfg = ChatConfig::default_ollama();
        let llm = cfg.config_for_role(0, 4);
        assert!(matches!(llm.provider, LlmProviderType::Ollama));
        assert_eq!(llm.endpoint, "http://localhost:11434");
    }
}
