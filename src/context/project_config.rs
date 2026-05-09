use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// 项目级配置 - 可通过 .garbage-code-hunter.toml 自定义
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    /// 项目类型提示（可选，辅助上下文推断）
    #[serde(default)]
    pub project_type: Option<ProjectType>,

    /// 全局白名单配置
    #[serde(default)]
    pub whitelists: Whitelists,

    /// 各个规则的独立配置
    #[serde(default)]
    pub rules: RulesConfig,

    /// 文件和目录级别的覆盖配置
    #[serde(default)]
    pub overrides: Vec<OverrideConfig>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_type: None,
            whitelists: Whitelists::default(),
            rules: RulesConfig::default(),
            overrides: Vec::new(),
        }
    }
}

impl ProjectConfig {
    /// 从文件加载配置，如果文件不存在返回默认配置
    pub fn load_from_file(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    /// 在当前目录或父级目录中查找配置文件
    pub fn discover(start_dir: &Path) -> Self {
        let config_name = ".garbage-code-hunter.toml";

        // 从当前目录向上查找
        let mut current = Some(start_dir);
        while let Some(dir) = current {
            let config_path = dir.join(config_name);
            if let Some(config) = Self::load_from_file(&config_path) {
                return config;
            }
            current = dir.parent();
        }

        // 未找到，使用默认配置
        Self::default()
    }

    /// 获取指定路径的有效覆盖配置（合并所有匹配的 override）
    pub fn get_override_for_path(&self, path: &Path) -> Option<&OverrideConfig> {
        let path_str = path.to_string_lossy();

        self.overrides.iter().find(|override_config| {
            // 简单的字符串匹配（未来可改为 glob 匹配）
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
    /// 允许的魔法数字列表（不会报告这些数字）
    #[serde(default)]
    pub magic_numbers: Vec<i64>,

    /// 允许的变量名列表（不会报告这些名称）
    #[serde(default)]
    pub variable_names: Vec<String>,

    /// 降低敏感度的目录模式（glob 模式）
    #[serde(default)]
    pub directories: Vec<String>,

    /// 完全排除的目录或文件模式
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct RulesConfig {
    /// 命名规则配置
    #[serde(default)]
    pub naming: NamingRuleConfig,

    /// Unwrap 规则配置
    #[serde(default)]
    pub unwrap: UnwrapRuleConfig,

    /// Magic Number 规则配置
    #[serde(default)]
    pub magic_number: MagicNumberRuleConfig,

    /// println 规则配置
    #[serde(default)]
    pub println: PrintlnRuleConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NamingRuleConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default = "default_severity_mild")]
    pub severity: SeverityOverride,

    /// 额外的允许变量名
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

    /// 触发报告的最小 unwrap 数量（默认 1）
    #[serde(default = "default_unwrap_threshold")]
    pub threshold: usize,

    /// Nuclear 级别的阈值（默认 15）
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

    /// 额外允许的魔法数字
    #[serde(default)]
    pub allowed_numbers: Vec<i64>,

    /// UI 布局常见值（自动添加到白名单）
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

    /// 是否在 main.rs/lib.rs 中允许 println
    #[serde(default = "default_true")]
    pub allow_in_main_files: bool,

    /// 触发报告的最小数量
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct OverrideConfig {
    /// 匹配模式（glob 或路径前缀）
    pub pattern: String,

    /// 强制使用的上下文类型
    #[serde(default)]
    pub context: Option<FileContextType>,

    /// 规则权重的全局乘数
    #[serde(default = "default_one")]
    pub weight_multiplier: f64,

    /// 要禁用的规则列表
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

// === 默认值函数 ===

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
