use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Top-level application configuration.
/// Merged from config.toml (defaults) + CLI overrides.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mode: AppMode,
}

/// Analysis mode: local (no LLM) or LLM-powered.
#[derive(Debug, Clone)]
pub enum AppMode {
    /// Local mode: use hardcoded i18n roast messages.
    Local,
    /// LLM mode: generate roasts via an external LLM API.
    Llm(LlmModeConfig),
}

/// Configuration for LLM mode.
#[derive(Debug, Clone)]
pub struct LlmModeConfig {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

// --- TOML deserialization types ---

#[derive(Debug, Deserialize, Default)]
struct TomlConfig {
    mode: Option<TomlMode>,
    llm: Option<TomlLlm>,
}

#[derive(Debug, Deserialize, Default)]
struct TomlMode {
    active: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct TomlLlm {
    provider: Option<String>,
    endpoint: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    timeout_secs: Option<u64>,
}

impl AppConfig {
    /// Load configuration from a TOML file. Returns defaults if no file is found.
    pub fn from_file(path: Option<&Path>) -> Result<Self> {
        let toml_config = match path {
            Some(p) => load_toml_from_path(p)?,
            None => find_and_load_config()?,
        };
        Ok(Self::from_toml(toml_config))
    }

    /// Build AppConfig from parsed TOML config.
    fn from_toml(toml: TomlConfig) -> Self {
        let active_mode = toml
            .mode
            .as_ref()
            .and_then(|m| m.active.as_deref())
            .unwrap_or("local");

        let mode = match active_mode {
            "llm" => {
                let llm = toml.llm.unwrap_or_default();
                let provider = llm.provider.unwrap_or_else(|| "ollama".to_string());
                let (default_endpoint, default_model) = match provider.as_str() {
                    "ollama" => ("http://localhost:11434", "llama3.2"),
                    _ => ("http://localhost:1234", "gpt-3.5-turbo"),
                };
                AppMode::Llm(LlmModeConfig {
                    endpoint: llm
                        .endpoint
                        .filter(|e| !e.is_empty())
                        .unwrap_or_else(|| default_endpoint.to_string()),
                    model: llm
                        .model
                        .filter(|m| !m.is_empty())
                        .unwrap_or_else(|| default_model.to_string()),
                    api_key: llm.api_key.filter(|k| !k.is_empty()),
                    timeout_secs: llm.timeout_secs.unwrap_or(30),
                    provider,
                })
            }
            _ => AppMode::Local,
        };

        Self { mode }
    }

    /// Merge CLI arguments into the config. CLI flags override config file values.
    pub fn merge_cli(
        &mut self,
        llm_flag: bool,
        llm_provider: &str,
        llm_endpoint: Option<&str>,
        llm_model: Option<&str>,
        llm_api_key: Option<&str>,
        llm_timeout: Option<u64>,  // Use Option to distinguish "not set" from "explicitly set to 30"
    ) {
        // --llm flag overrides config file mode
        if llm_flag {
            let provider = llm_provider.to_string();
            let (default_endpoint, default_model) = match provider.as_str() {
                "ollama" => ("http://localhost:11434", "llama3.2"),
                _ => ("http://localhost:1234", "gpt-3.5-turbo"),
            };
            self.mode = AppMode::Llm(LlmModeConfig {
                endpoint: llm_endpoint.unwrap_or(default_endpoint).to_string(),
                model: llm_model.unwrap_or(default_model).to_string(),
                api_key: llm_api_key.map(String::from),
                timeout_secs: llm_timeout.unwrap_or(30),  // Default to 30 if not set
                provider,
            });
        }

        // Individual --llm-* flags can override config file LLM settings
        if let AppMode::Llm(ref mut llm_cfg) = self.mode {
            if let Some(ep) = llm_endpoint {
                llm_cfg.endpoint = ep.to_string();
            }
            if let Some(m) = llm_model {
                llm_cfg.model = m.to_string();
            }
            if let Some(k) = llm_api_key {
                llm_cfg.api_key = Some(k.to_string());
            }
            // Override timeout only if explicitly set via CLI
            if let Some(timeout) = llm_timeout {
                llm_cfg.timeout_secs = timeout;
            }
        }
    }
}

/// Load and parse a TOML file at the given path.
fn load_toml_from_path(path: &Path) -> Result<TomlConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))
}

/// Search for config file in standard locations.
fn find_and_load_config() -> Result<TomlConfig> {
    // Search order: ./config.toml, ~/.config/garbage-code-hunter/config.toml
    let local_config = PathBuf::from("config.toml");
    if local_config.exists() {
        return load_toml_from_path(&local_config);
    }

    if let Ok(home) = std::env::var("HOME") {
        let user_config = PathBuf::from(home)
            .join(".config")
            .join("garbage-code-hunter")
            .join("config.toml");
        if user_config.exists() {
            return load_toml_from_path(&user_config);
        }
    }

    Ok(TomlConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults_when_no_file() {
        let config = AppConfig::from_toml(TomlConfig::default());
        assert!(matches!(config.mode, AppMode::Local));
    }

    #[test]
    fn test_config_parse_local_mode() {
        let toml_str = r#"
[mode]
active = "local"
"#;
        let toml_config: TomlConfig = toml::from_str(toml_str).unwrap();
        let config = AppConfig::from_toml(toml_config);
        assert!(matches!(config.mode, AppMode::Local));
    }

    #[test]
    fn test_config_parse_llm_mode() {
        let toml_str = r#"
[mode]
active = "llm"

[llm]
provider = "ollama"
endpoint = "http://custom:11434"
model = "llama3.1"
timeout_secs = 60
"#;
        let toml_config: TomlConfig = toml::from_str(toml_str).unwrap();
        let config = AppConfig::from_toml(toml_config);
        match config.mode {
            AppMode::Llm(llm) => {
                assert_eq!(llm.provider, "ollama");
                assert_eq!(llm.endpoint, "http://custom:11434");
                assert_eq!(llm.model, "llama3.1");
                assert_eq!(llm.timeout_secs, 60);
            }
            _ => panic!("Expected LLM mode"),
        }
    }

    #[test]
    fn test_config_llm_defaults() {
        let toml_str = r#"
[mode]
active = "llm"
"#;
        let toml_config: TomlConfig = toml::from_str(toml_str).unwrap();
        let config = AppConfig::from_toml(toml_config);
        match config.mode {
            AppMode::Llm(llm) => {
                assert_eq!(llm.provider, "ollama");
                assert_eq!(llm.endpoint, "http://localhost:11434");
                assert_eq!(llm.model, "llama3.2");
                assert_eq!(llm.timeout_secs, 30);
                assert!(llm.api_key.is_none());
            }
            _ => panic!("Expected LLM mode"),
        }
    }

    #[test]
    fn test_config_llm_openai_compatible() {
        let toml_str = r#"
[mode]
active = "llm"

[llm]
provider = "openai-compatible"
api_key = "sk-test123"
"#;
        let toml_config: TomlConfig = toml::from_str(toml_str).unwrap();
        let config = AppConfig::from_toml(toml_config);
        match config.mode {
            AppMode::Llm(llm) => {
                assert_eq!(llm.provider, "openai-compatible");
                assert_eq!(llm.endpoint, "http://localhost:1234");
                assert_eq!(llm.model, "gpt-3.5-turbo");
                assert_eq!(llm.api_key, Some("sk-test123".to_string()));
            }
            _ => panic!("Expected LLM mode"),
        }
    }

    #[test]
    fn test_config_invalid_toml() {
        let result = toml::from_str::<TomlConfig>("this is not valid toml [[[");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_cli_overrides_file() {
        let toml_str = r#"
[mode]
active = "local"

[llm]
provider = "ollama"
"#;
        let toml_config: TomlConfig = toml::from_str(toml_str).unwrap();
        let mut config = AppConfig::from_toml(toml_config);
        assert!(matches!(config.mode, AppMode::Local));

        // CLI --llm flag should override local mode
        config.merge_cli(true, "openai-compatible", None, None, Some("sk-key"), Some(60));
        match config.mode {
            AppMode::Llm(llm) => {
                assert_eq!(llm.provider, "openai-compatible");
                assert_eq!(llm.api_key, Some("sk-key".to_string()));
                assert_eq!(llm.timeout_secs, 60);
            }
            _ => panic!("Expected LLM mode after CLI override"),
        }
    }
}
