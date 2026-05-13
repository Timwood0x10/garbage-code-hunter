use std::path::Path;
use syn::{visit::Visit, ExprMethodCall, File};

use crate::analyzer::{CodeIssue, Severity};
use crate::context::FileContext;
use crate::rules::Rule;
use crate::utils::get_position;

/// Detects excessive use of `.unwrap()` instead of proper error handling.
pub struct UnwrapAbuseRule;

impl Rule for UnwrapAbuseRule {
    fn name(&self) -> &'static str {
        "unwrap-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = UnwrapVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }

    fn check_with_context(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
        context: &FileContext,
        config: &crate::context::ProjectConfig,
    ) -> Vec<CodeIssue> {
        // Test/Benchmark/Documentation: skip entirely
        let weight = context.rule_weight_multiplier();
        if weight < 0.5 {
            return Vec::new();
        }

        let unwrap_cfg = &config.rules.unwrap;
        let threshold = unwrap_cfg.threshold;
        let nuclear_threshold = unwrap_cfg.nuclear_threshold;

        // Example/Demo: use a higher threshold
        let effective_threshold = if weight < 0.7 {
            threshold.max(10)
        } else {
            threshold
        };

        let mut visitor = UnwrapVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);

        // Apply nuclear threshold — upgrade severity for excessive unwraps
        let mut issues = visitor.issues;
        if issues.len() >= nuclear_threshold {
            for issue in &mut issues {
                issue.severity = Severity::Nuclear;
            }
        }

        // Filter by threshold
        if issues.len() >= effective_threshold {
            issues
        } else {
            Vec::new()
        }
    }
}

/// Detects excessive use of `.clone()` that may indicate ownership issues.
pub struct UnnecessaryCloneRule;

impl Rule for UnnecessaryCloneRule {
    fn name(&self) -> &'static str {
        "unnecessary-clone"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = CloneVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct UnwrapVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    unwrap_count: usize,
    first_position: (usize, usize),
    lang: String,
}

impl UnwrapVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            unwrap_count: 0,
            first_position: (1, 1),
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for UnwrapVisitor {
    fn visit_expr_method_call(&mut self, method_call: &'ast ExprMethodCall) {
        if method_call.method == "unwrap" {
            if self.unwrap_count == 0 {
                self.first_position = get_position(method_call);
            }
            self.unwrap_count += 1;
        }

        syn::visit::visit_expr_method_call(self, method_call);
    }

    fn visit_file(&mut self, file: &'ast syn::File) {
        syn::visit::visit_file(self, file);
        // After visiting, report a single issue with total count
        if self.unwrap_count > 0 {
            let (line, column) = self.first_position;

            // Determine severity and message based on count
            let (severity, message) = if self.unwrap_count > 15 {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!(
                            "🚨 {} 个 unwrap()！这是在生产环境玩俄罗斯轮盘",
                            self.unwrap_count
                        ),
                        format!(
                            "{} 个 unwrap()，你的程序比我的感情生活还不稳定",
                            self.unwrap_count
                        ),
                        format!("unwrap() 滥用警报：{} 处潜在崩溃点", self.unwrap_count),
                        format!(
                            "{} 个 unwrap() = {} 次潜在的 panic！立即修复！",
                            self.unwrap_count, self.unwrap_count
                        ),
                    ]
                } else {
                    vec![
                        format!(
                            "🚨 {} unwrap()s! Playing Russian roulette in production?",
                            self.unwrap_count
                        ),
                        format!(
                            "{} unwrap()s - less stable than my love life",
                            self.unwrap_count
                        ),
                        format!(
                            "unwrap() abuse alert: {} potential crash points",
                            self.unwrap_count
                        ),
                        format!(
                            "{} unwrap()s = {} potential panics! Fix immediately!",
                            self.unwrap_count, self.unwrap_count
                        ),
                    ]
                };
                (
                    Severity::Nuclear,
                    messages[self.issues.len() % messages.len()].clone(),
                )
            } else if self.unwrap_count > 8 {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!(
                            "⚠️ {} 个 unwrap()，建议用 ? 运算符或 match 替代",
                            self.unwrap_count
                        ),
                        format!("发现 {} 个 unwrap()，错误处理在哪里？", self.unwrap_count),
                        format!("{} 个 unwrap()，程序健壮性堪忧", self.unwrap_count),
                        format!("这么多 unwrap()，Rust 的类型系统在哭泣",),
                    ]
                } else {
                    vec![
                        format!(
                            "⚠️ {} unwrap()s - consider using ? or match",
                            self.unwrap_count
                        ),
                        format!(
                            "Found {} unwrap()s - where's error handling?",
                            self.unwrap_count
                        ),
                        format!(
                            "{} unwrap()s - program robustness questionable",
                            self.unwrap_count
                        ),
                        format!("So many unwrap()s, Rust's type system is crying",),
                    ]
                };
                (
                    Severity::Spicy,
                    messages[self.issues.len() % messages.len()].clone(),
                )
            } else if self.unwrap_count > 3 {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!(
                            "{} 个 unwrap()，建议使用 ? 运算符或 match 处理错误",
                            self.unwrap_count
                        ),
                        format!(
                            "发现 {} 个 unwrap()，考虑用 expect() 至少给个错误信息",
                            self.unwrap_count
                        ),
                        format!(
                            "{} 个 unwrap() 调用，可以用 .ok()? 或 .map_err()? 替代",
                            self.unwrap_count
                        ),
                    ]
                } else {
                    vec![
                        format!(
                            "{} unwrap()s - consider using ? operator or match",
                            self.unwrap_count
                        ),
                        format!(
                            "Found {} unwrap()s - at least use expect() for error messages",
                            self.unwrap_count
                        ),
                        format!(
                            "{} unwrap() calls - can use .ok()? or .map_err()?",
                            self.unwrap_count
                        ),
                    ]
                };
                (
                    Severity::Mild,
                    messages[self.issues.len() % messages.len()].clone(),
                )
            } else {
                // 1-3 unwraps: just a gentle reminder
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!(
                            "{} 个 unwrap()，小提醒：生产代码建议用 ? 运算符",
                            self.unwrap_count
                        ),
                        format!(
                            "检测到 {} 个 unwrap()，记得处理可能的 None/Err",
                            self.unwrap_count
                        ),
                    ]
                } else {
                    vec![
                        format!(
                            "{} unwrap()s - reminder: use ? in production code",
                            self.unwrap_count
                        ),
                        format!(
                            "Detected {} unwrap()s - remember to handle None/Err",
                            self.unwrap_count
                        ),
                    ]
                };
                (
                    Severity::Mild,
                    messages[self.issues.len() % messages.len()].clone(),
                )
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "unwrap-abuse".to_string(),
                message,
                severity,
            });
        }
    }
}

struct CloneVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    clone_count: usize,
    lang: String,
}

impl CloneVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            clone_count: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for CloneVisitor {
    fn visit_expr_method_call(&mut self, method_call: &'ast ExprMethodCall) {
        if method_call.method == "clone" {
            self.clone_count += 1;

            // Report once when clone count reaches threshold (>=25)
            // Increased from 15 to avoid false positives in rule definitions
            // (each CodeIssue creation uses .clone() for file_path)
            if self.clone_count >= 25 && self.issues.is_empty() {
                let messages = if self.lang == "zh-CN" {
                    [
                        "clone() 狂魔！你是想把内存用完吗？",
                        "这么多 clone()，你确定不是在写 Java？",
                        "clone() 使用过度！Rust 的借用检查器在哭泣",
                        "又见 clone()！也许你需要重新学习一下 Rust 的所有权系统",
                        "这些 clone() 让我想起了复印机店的老板",
                    ]
                } else {
                    [
                        "clone() fanatic! Trying to use up all the memory?",
                        "This many clone()s - are you sure you're not writing Java?",
                        "clone() abuse! Rust's borrow checker is crying",
                        "clone() again! Maybe you need to relearn Rust's ownership system",
                        "These clone()s remind me of a copy shop owner",
                    ]
                };

                let (line, column) = get_position(method_call);

                self.issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line,
                    column,
                    rule_name: "unnecessary-clone".to_string(),
                    message: messages[self.issues.len() % messages.len()].to_string(),
                    severity: Severity::Spicy,
                });
            }
        }

        syn::visit::visit_expr_method_call(self, method_call);
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
    fn test_unwrap_abuse_detects_unwrap() {
        let code = r#"
            fn main() {
                let x = Some(1).unwrap();
                let y = Some(2).unwrap();
                let z = Some(3).unwrap();
                let w = Some(4).unwrap();
            }
        "#;
        let issues = check_rule(&UnwrapAbuseRule, code);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.rule_name == "unwrap-abuse"));
    }

    #[test]
    fn test_unwrap_abuse_clean_code() {
        let code = r#"
            fn main() -> Result<(), Box<dyn std::error::Error>> {
                let x = Some(1).ok_or("missing")?;
                Ok(())
            }
        "#;
        let issues = check_rule(&UnwrapAbuseRule, code);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_rule_names() {
        assert_eq!(UnwrapAbuseRule.name(), "unwrap-abuse");
        assert_eq!(UnnecessaryCloneRule.name(), "unnecessary-clone");
    }
}
