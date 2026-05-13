use std::path::Path;
use syn::{visit::Visit, File, Ident};

use crate::analyzer::{CodeIssue, Severity};
use crate::context::FileContext;
use crate::rules::Rule;
use crate::utils::get_position;

/// Detects meaningless placeholder names: foo, bar, baz, qux, test, temp, etc.
pub struct MeaninglessNamingRule;

impl Rule for MeaninglessNamingRule {
    fn name(&self) -> &'static str {
        "meaningless-naming"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = MeaninglessNamingVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }

    fn check_with_context(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        _is_test_file: bool,
        context: &FileContext,
        _config: &crate::context::ProjectConfig,
    ) -> Vec<CodeIssue> {
        use crate::context::FileContext::*;

        match context {
            // Test/Documentation/Benchmark: 完全跳过
            Test | Documentation | Benchmark => return Vec::new(),

            // Example/Demo: 仅报告 Nuclear 级别问题（极少）
            Example => {
                let issues = self.check(file_path, syntax_tree, content, lang, _is_test_file);
                return issues
                    .into_iter()
                    .filter(|issue| issue.severity == Severity::Nuclear)
                    .collect();
            }

            // UI/TUI code: Skip UI-specific common names (x, y, w, h, r, g, b, etc.)
            UI => {
                let all_issues = self.check(file_path, syntax_tree, content, lang, _is_test_file);
                let ui_whitelist = [
                    // Coordinates and dimensions
                    "x", "y", "w", "h", // Colors
                    "r", "g", "b", "a", // Deltas
                    "dx", "dy", // Geometry
                    "rect", "area", "size", // Positions/vectors
                    "pos", "vec", // UI references
                    "ui", "tui", // Common TUI data names (very common in UI callbacks)
                    "data", "info", "value", "state", "config", "event", "input", "output",
                    "result", // Layout-related
                    "chunk", "layout", "frame", "block",
                ];
                return all_issues
                    .into_iter()
                    .filter(|issue| {
                        !ui_whitelist.iter().any(|&name| {
                            issue
                                .message
                                .to_lowercase()
                                .contains(&format!("'{}'", name))
                        })
                    })
                    .collect();
            }

            // GPU code: Skip GPU-common names (i, j, k, idx, src, dst, etc.)
            GPU => {
                let all_issues = self.check(file_path, syntax_tree, content, lang, _is_test_file);
                let gpu_whitelist = [
                    "i", "j", "k",   // loop indices
                    "idx", // index abbreviation
                    "src", "dst",  // source/destination
                    "buf",  // buffer
                    "ptr",  // pointer
                    "data", // data buffer (common in GPU)
                ];
                return all_issues
                    .into_iter()
                    .filter(|issue| {
                        !gpu_whitelist.iter().any(|&name| {
                            issue
                                .message
                                .to_lowercase()
                                .contains(&format!("'{}'", name))
                        })
                    })
                    .collect();
            }

            // Web code: Slightly relaxed - allow common web naming
            Web => {
                let all_issues = self.check(file_path, syntax_tree, content, lang, _is_test_file);
                let web_whitelist = [
                    "data", // request/response data
                    "info", // metadata
                    "req", "res",    // request/response abbreviations
                    "body",   // response body
                    "config", // configuration objects
                ];
                return all_issues
                    .into_iter()
                    .filter(|issue| {
                        !web_whitelist.iter().any(|&name| {
                            issue
                                .message
                                .to_lowercase()
                                .contains(&format!("'{}'", name))
                        })
                    })
                    .collect();
            }

            // Business 上下文：正常检测（保持原有行为）
            Business => {}

            // Config files: skip completely (handled by should_skip_rule)
            Config => return Vec::new(),
        }

        self.check(file_path, syntax_tree, content, lang, _is_test_file)
    }
}

/// Detect outdated Hungarian notation: strName, intCount, bIsValid, etc.
pub struct HungarianNotationRule;

impl Rule for HungarianNotationRule {
    fn name(&self) -> &'static str {
        "hungarian-notation"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = HungarianNotationVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// Detect excessive abbreviations: mgr, ctrl, btn, usr, pwd, etc.
pub struct AbbreviationAbuseRule;

impl Rule for AbbreviationAbuseRule {
    fn name(&self) -> &'static str {
        "abbreviation-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = AbbreviationAbuseVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

// ============================================================================
// Visitor implementations
// ============================================================================

struct MeaninglessNamingVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl MeaninglessNamingVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn is_meaningless_name(&self, name: &str) -> bool {
        let meaningless_names = [
            // Classic placeholders - truly meaningless
            "foo",
            "bar",
            "baz",
            "qux",
            "quux",
            "quuz",
            // Generic words with no context (note: 'item' excluded as it's a common Rust iterator variable)
            "data",
            "info",
            "obj",
            "thing",
            "stuff",
            "value",
            "temp",
            "tmp",
            "example",
            "sample",
            // Manager suffix abuse
            "manager",
            "handler",
            "processor",
            "controller",
            // Chinese pinyin (common ones)
            "yonghu",
            "mima",
            "denglu",
            "zhuce",
            "shuju",
        ];

        let name_lower = name.to_lowercase();
        meaningless_names
            .iter()
            .any(|&bad_name| name_lower == bad_name)
    }

    fn create_issue(&self, name: &str, line: usize, column: usize) -> CodeIssue {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("变量名 '{}' 比我的网名还随意", name),
                format!("'{}' 这个名字，是从字典里随机选的吗？", name),
                format!("用 '{}' 做变量名？你是想让下一个维护代码的人猜谜吗？", name),
                format!("'{}' 这个名字毫无意义，就像我的人生一样", name),
                format!("看到 '{}' 这个变量名，我的智商受到了侮辱", name),
            ]
        } else {
            vec![
                format!("Variable name '{}' is more meaningless than my existence", name),
                format!("'{}' - did you pick this name with your eyes closed?", name),
                format!("Using '{}' as a variable name? Are you playing charades with future developers?", name),
                format!("'{}' tells me nothing about what this variable does", name),
                format!("The name '{}' is as helpful as a chocolate teapot", name),
            ]
        };

        let severity = if ["foo", "bar", "baz", "data", "temp"].contains(&name) {
            Severity::Spicy
        } else {
            Severity::Mild
        };

        CodeIssue {
            file_path: self.file_path.clone(),
            line,
            column,
            rule_name: "meaningless-naming".to_string(),
            message: messages[self.issues.len() % messages.len()].clone(),
            severity,
        }
    }
}

impl<'ast> Visit<'ast> for MeaninglessNamingVisitor {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        let name = ident.to_string();
        if self.is_meaningless_name(&name) {
            let (line, column) = get_position(ident);
            self.issues.push(self.create_issue(&name, line, column));
        }
        syn::visit::visit_ident(self, ident);
    }
}

// ============================================================================
// Hungarian notation detection
// ============================================================================

struct HungarianNotationVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl HungarianNotationVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn is_hungarian_notation(&self, name: &str) -> bool {
        let hungarian_prefixes = [
            // Type prefixes (only check with camelCase, not underscore)
            "str", "int", "bool", "float", "double", "char", "arr", "vec", "list", "map", "set",
            // Scope prefixes
            "g_", "m_", "s_", "p_",
        ];

        // Check if starts with a Hungarian prefix
        for prefix in hungarian_prefixes {
            if name.starts_with(prefix) && name.len() > prefix.len() {
                // Check if prefix is followed by uppercase letter (camelCase)
                if let Some(next_char) = name.chars().nth(prefix.len()) {
                    if next_char.is_uppercase() {
                        // Avoid false positives on words like "stringify", "internal", "boolean"
                        // by checking that the prefix is not part of a longer word
                        let rest = &name[prefix.len()..];
                        // If the rest starts with a common word continuation, skip
                        if rest.starts_with("ify")
                            || rest.starts_with("nal")
                            || rest.starts_with("ean")
                        {
                            continue;
                        }
                        return true;
                    }
                }
            }
        }

        // Check underscore-separated case separately
        for prefix in &["g_", "m_", "s_", "p_"] {
            if name.starts_with(prefix) {
                return true;
            }
        }

        false
    }

    fn create_issue(&self, name: &str, line: usize, column: usize) -> CodeIssue {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("'{}' 使用了匈牙利命名法？这不是1990年代了", name),
                format!("看到 '{}' 我仿佛回到了 C++ 的石器时代", name),
                format!("'{}' 这种命名方式已经过时了，就像我的发型一样", name),
                format!("匈牙利命名法 '{}'？Rust 编译器已经帮你检查类型了", name),
                format!("'{}' 让我想起了那些痛苦的 C++ 岁月", name),
            ]
        } else {
            vec![
                format!(
                    "'{}' uses Hungarian notation? This isn't the 1990s anymore",
                    name
                ),
                format!(
                    "Seeing '{}' makes me nostalgic for the dark ages of C++",
                    name
                ),
                format!(
                    "'{}' - Hungarian notation is as outdated as my haircut",
                    name
                ),
                format!(
                    "Hungarian notation '{}'? Rust's type system has got you covered",
                    name
                ),
                format!("'{}' reminds me of painful C++ memories", name),
            ]
        };

        CodeIssue {
            file_path: self.file_path.clone(),
            line,
            column,
            rule_name: "hungarian-notation".to_string(),
            message: messages[self.issues.len() % messages.len()].clone(),
            severity: Severity::Mild,
        }
    }
}

impl<'ast> Visit<'ast> for HungarianNotationVisitor {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        let name = ident.to_string();
        if self.is_hungarian_notation(&name) {
            let (line, column) = get_position(ident);
            self.issues.push(self.create_issue(&name, line, column));
        }
        syn::visit::visit_ident(self, ident);
    }
}

// ============================================================================
// Excessive abbreviation detection
// ============================================================================

struct AbbreviationAbuseVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl AbbreviationAbuseVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn is_bad_abbreviation(&self, name: &str) -> Option<&'static str> {
        let bad_abbreviations = [
            // Management related - these are truly unclear
            ("mgr", "manager"),
            ("mngr", "manager"),
            ("ctrl", "controller"),
            ("hdlr", "handler"),
            // User related
            ("usr", "user"),
            ("pwd", "password"),
            ("prefs", "preferences"),
            // UI related
            ("btn", "button"),
            ("lbl", "label"),
            ("pic", "picture"),
            // Data related
            ("tbl", "table"),
            ("col", "column"),
            ("cnt", "count"),
        ];

        let name_lower = name.to_lowercase();
        for (abbrev, full) in bad_abbreviations {
            if name_lower == abbrev || name_lower.starts_with(&format!("{abbrev}_")) {
                return Some(full);
            }
        }
        None
    }

    fn create_issue(&self, name: &str, suggestion: &str, line: usize, column: usize) -> CodeIssue {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("'{}' 缩写得太狠了，建议用 '{}'", name, suggestion),
                format!("看到 '{}' 我需要解密，不如直接用 '{}'", name, suggestion),
                format!(
                    "'{}' 这个缩写让我想起了发电报的年代，用 '{}' 吧",
                    name, suggestion
                ),
                format!(
                    "'{}' 省了几个字母，却让代码可读性大打折扣，试试 '{}'",
                    name, suggestion
                ),
                format!("缩写 '{}' 就像密码一样难懂，'{}'不香吗？", name, suggestion),
            ]
        } else {
            vec![
                format!("'{}' is too abbreviated, consider '{}'", name, suggestion),
                format!(
                    "Seeing '{}' makes me feel like I'm decoding, just use '{}'",
                    name, suggestion
                ),
                format!(
                    "'{}' reminds me of telegraph era, try '{}'",
                    name, suggestion
                ),
                format!(
                    "'{}' saves a few letters but kills readability, use '{}'",
                    name, suggestion
                ),
                format!(
                    "Abbreviation '{}' is cryptic, isn't '{}' better?",
                    name, suggestion
                ),
            ]
        };

        CodeIssue {
            file_path: self.file_path.clone(),
            line,
            column,
            rule_name: "abbreviation-abuse".to_string(),
            message: messages[self.issues.len() % messages.len()].clone(),
            severity: Severity::Mild,
        }
    }
}

impl<'ast> Visit<'ast> for AbbreviationAbuseVisitor {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        let name = ident.to_string();
        if let Some(suggestion) = self.is_bad_abbreviation(&name) {
            let (line, column) = get_position(ident);
            self.issues
                .push(self.create_issue(&name, suggestion, line, column));
        }
        syn::visit::visit_ident(self, ident);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_rule(rule: &impl Rule, code: &str) -> Vec<CodeIssue> {
        let path = std::path::Path::new("test.rs");
        let ast = syn::parse_file(code).unwrap();
        rule.check(path, &ast, code, "en-US", false)
    }

    #[test]
    fn test_meaningless_naming_detects_foo() {
        let code = r#"
            fn main() {
                let foo = 1;
                let bar = 2;
            }
        "#;
        let issues = check_rule(&MeaninglessNamingRule, code);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.rule_name == "meaningless-naming"));
    }

    #[test]
    fn test_meaningless_naming_clean_code() {
        let code = r#"
            fn main() {
                let user_count = 1;
                let max_retries = 3;
            }
        "#;
        let issues = check_rule(&MeaninglessNamingRule, code);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_rule_names() {
        assert_eq!(MeaninglessNamingRule.name(), "meaningless-naming");
        assert_eq!(HungarianNotationRule.name(), "hungarian-notation");
        assert_eq!(AbbreviationAbuseRule.name(), "abbreviation-abuse");
    }
}
