use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Project-level configuration - customizable via .garbage-code-hunter.toml
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProjectConfig {
    /// Project type hint (optional, helps context inference)
    #[serde(default)]
    pub project_type: Option<ProjectType>,

    /// Global whitelist configuration
    #[serde(default)]
    pub whitelists: Whitelists,

    /// Individual rule configurations
    #[serde(default)]
    pub rules: RulesConfig,

    /// Signal detector configuration
    #[serde(default)]
    pub signals: SignalsConfig,

    /// File and directory-level override configuration
    #[serde(default)]
    pub overrides: Vec<OverrideConfig>,
}

impl ProjectConfig {
    /// Load configuration from file, returns default if file doesn't exist
    pub fn load_from_file(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Discover configuration in current directory or parent directories
    pub fn discover(start_dir: &Path) -> Self {
        let config_name = ".garbage-code-hunter.toml";

        // Search upward from current directory
        let mut current = Some(start_dir);
        while let Some(dir) = current {
            let config_path = dir.join(config_name);
            if let Some(config) = Self::load_from_file(&config_path) {
                return config;
            }
            current = dir.parent();
        }

        // Not found, use default configuration
        Self::default()
    }

    /// Get effective override config for the given path (merge all matching overrides)
    pub fn get_override_for_path(&self, path: &Path) -> Option<&OverrideConfig> {
        let path_str = path.to_string_lossy();

        self.overrides.iter().find(|override_config| {
            // Simple string matching (could be upgraded to glob matching in the future)
            path_str.contains(&override_config.pattern)
                || path_str.starts_with(&override_config.pattern)
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectType {
    CliTool,
    Library,
    WebService,
    Game,
    Embedded,
    Wasm,
    Other(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Whitelists {
    /// Allowed magic numbers (won't report these)
    #[serde(default)]
    pub magic_numbers: Vec<i64>,

    /// Allowed variable names (won't report these)
    #[serde(default)]
    pub variable_names: Vec<String>,

    /// Directory patterns for sensitivity reduction (glob patterns)
    #[serde(default)]
    pub directories: Vec<String>,

    /// Completely excluded directory or file patterns
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct RulesConfig {
    /// Naming rule configuration
    #[serde(default)]
    pub naming: NamingRuleConfig,

    /// Unwrap rule configuration
    #[serde(default)]
    pub unwrap: UnwrapRuleConfig,

    /// Magic Number rule configuration
    #[serde(default)]
    pub magic_number: MagicNumberRuleConfig,

    /// Println rule configuration
    #[serde(default)]
    pub println: PrintlnRuleConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NamingRuleConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default = "default_severity_mild")]
    pub severity: SeverityOverride,

    /// Additional allowed variable names
    #[serde(default)]
    pub allowed_names: Vec<String>,
}

impl Default for NamingRuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: SeverityOverride::Mild,
            allowed_names: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnwrapRuleConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Minimum unwrap count to trigger report (default: 1)
    #[serde(default = "default_unwrap_threshold")]
    pub threshold: usize,

    /// Nuclear level threshold (default: 15)
    #[serde(default = "default_nuclear_threshold")]
    pub nuclear_threshold: usize,
}

impl Default for UnwrapRuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 1,
            nuclear_threshold: 15,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct MagicNumberRuleConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Additional allowed magic numbers
    #[serde(default)]
    pub allowed_numbers: Vec<i64>,

    /// Common UI layout values (automatically added to whitelist)
    #[serde(default = "default_ui_numbers")]
    pub ui_layout_numbers: Vec<i64>,
}

impl Default for MagicNumberRuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_numbers: Vec::new(),
            ui_layout_numbers: default_ui_numbers(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PrintlnRuleConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Whether to allow println in main.rs/lib.rs
    #[serde(default = "default_true")]
    pub allow_in_main_files: bool,

    /// Minimum count to trigger report
    #[serde(default = "default_println_threshold")]
    pub threshold: usize,
}

impl Default for PrintlnRuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_in_main_files: true,
            threshold: 3,
        }
    }
}

/// Signal detector configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SignalsConfig {
    /// Whether to skip signal detection in test files (default: true).
    /// Set to `false` to report signals in test files.
    #[serde(default = "default_true")]
    pub skip_tests: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct OverrideConfig {
    /// Matching pattern (glob or path prefix)
    pub pattern: String,

    /// Forced context type
    #[serde(default)]
    pub context: Option<FileContextType>,

    /// Global multiplier for rule weight
    #[serde(default = "default_one")]
    pub weight_multiplier: f64,

    /// List of rules to disable
    #[serde(default)]
    pub disabled_rules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileContextType {
    Business,
    Example,
    Test,
    Benchmark,
    Documentation,
    Config,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SeverityOverride {
    Mild,
    Spicy,
    Nuclear,
}

// === Default value functions ===

fn default_enabled() -> bool {
    true
}
fn default_severity_mild() -> SeverityOverride {
    SeverityOverride::Mild
}
fn default_unwrap_threshold() -> usize {
    1
}
fn default_nuclear_threshold() -> usize {
    15
}
fn default_ui_numbers() -> Vec<i64> {
    vec![20, 25, 30, 33, 40, 50, 60, 66, 70, 75, 80, 90, 100]
}
fn default_true() -> bool {
    true
}
fn default_println_threshold() -> usize {
    3
}
fn default_one() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default();
        assert!(config.rules.naming.enabled);
        assert_eq!(config.rules.unwrap.threshold, 1);
        assert!(config.rules.println.allow_in_main_files);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let config = ProjectConfig::load_from_file(Path::new("/nonexistent/config.toml"));
        assert!(config.is_none());
    }

    #[test]
    fn test_discover_without_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = ProjectConfig::discover(temp_dir.path());

        assert!(config.project_type.is_none());
        assert!(config.whitelists.magic_numbers.is_empty());
    }

    #[test]
    fn test_load_valid_config() {
        let toml_content = r#"
[whitelists]
magic-numbers = [800, 1000, 2000]
variable-names = ["data", "info", "ctx"]

[rules.naming]
enabled = true
severity = "mild"
allowed-names = ["e", "ctx"]

[rules.unwrap]
threshold = 3
nuclear-threshold = 20

[rules.println]
allow-in-main-files = true
threshold = 5
"#;

        let config: ProjectConfig = toml::from_str(toml_content).expect("Failed to parse config");

        assert_eq!(config.whitelists.magic_numbers.len(), 3);
        assert_eq!(config.whitelists.variable_names.len(), 3);
        assert_eq!(config.rules.unwrap.threshold, 3);
        assert_eq!(config.rules.unwrap.nuclear_threshold, 20);
        assert_eq!(config.rules.println.threshold, 5);
        // Note: overrides parsing tested separately to isolate potential issues
    }
}
