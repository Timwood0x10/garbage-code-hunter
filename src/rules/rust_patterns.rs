use std::path::Path;
use syn::File;

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;
use crate::utils::{count_non_import_matches, find_line_of_str_non_import};

/// Detect using String everywhere instead of &str
pub struct StringAbuseRule;

impl Rule for StringAbuseRule {
    fn name(&self) -> &'static str {
        "string-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        _syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // Check String usage patterns in content (excluding comments and imports)
        let string_new_count = count_non_import_matches(content, "String::new()");
        let string_from_count = count_non_import_matches(content, "String::from(");
        let to_string_count = count_non_import_matches(content, ".to_string()");
        let total = string_new_count + string_from_count + to_string_count;

        if total > 15 {
            let messages = if lang == "zh-CN" {
                vec![
                    format!("{} 次 String 转换？你是在开字符串工厂吗？", total),
                    format!("这么多 String 分配，内存都要哭了"),
                    format!("{} 个 String 转换，考虑用 &str 吧", total),
                    format!("String 用得比我换衣服还频繁"),
                    format!("到处都是 String::from()，性能在哭泣"),
                ]
            } else {
                vec![
                    format!(
                        "{} String conversions? Are you running a string factory?",
                        total
                    ),
                    format!("So many String allocations, memory is crying"),
                    format!("{} String conversions - consider using &str", total),
                    format!("You use String more than I change clothes"),
                    format!("String::from() everywhere - performance is weeping"),
                ]
            };

            let candidates = [
                find_line_of_str_non_import(content, "String::from("),
                find_line_of_str_non_import(content, "String::new()"),
                find_line_of_str_non_import(content, ".to_string()"),
            ];
            let line = candidates
                .iter()
                .copied()
                .filter(|&l| l > 1)
                .min()
                .unwrap_or(1);

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line,
                column: 1,
                rule_name: "string-abuse".to_string(),
                message: messages[total % messages.len()].clone(),
                severity: Severity::Spicy,
            });
        }

        issues
    }
}

/// Detect unnecessary Vec allocations
pub struct VecAbuseRule;

impl Rule for VecAbuseRule {
    fn name(&self) -> &'static str {
        "vec-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        _syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // Check Vec usage patterns in content (excluding comments and imports)
        let vec_new_count = count_non_import_matches(content, "Vec::new()");

        if vec_new_count > 15 {
            let messages = if lang == "zh-CN" {
                vec![
                    format!("{} 个 Vec::new()？你在收集什么？", vec_new_count),
                    format!("这么多 Vec 分配，考虑用数组或切片"),
                    format!("{} 次 Vec 分配，内存分配器很忙", vec_new_count),
                    format!("Vec 用得这么多，确定都需要动态数组吗？"),
                ]
            } else {
                vec![
                    format!("{} Vec::new()s? What are you collecting?", vec_new_count),
                    format!("So many Vec allocations - consider arrays or slices"),
                    format!(
                        "{} Vec allocations - memory allocator is busy",
                        vec_new_count
                    ),
                    format!("So many Vecs - sure you need all these dynamic arrays?"),
                ]
            };

            let line = find_line_of_str_non_import(content, "Vec::new()");

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line,
                column: 1,
                rule_name: "vec-abuse".to_string(),
                message: messages[vec_new_count % messages.len()].clone(),
                severity: Severity::Mild,
            });
        }

        issues
    }
}
