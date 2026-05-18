use crate::analyzer::{CodeIssue, Severity};
use crate::context::project_config::ProjectConfig;
use crate::context::FileContext;
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

use super::complex_rules::{apply_naming_config, variable_name_query};

/// Meaningless naming: detects placeholder names like foo, bar, aaa, data, temp.
pub(crate) struct MeaninglessRule;

impl TreeSitterRule for MeaninglessRule {
    fn name(&self) -> &'static str {
        "meaningless-naming"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let query = variable_name_query(file.language);
        let captures = match collect_captures(file, query) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let meaningless: &[&str] = &[
            "foo", "bar", "baz", "qux", "quux", "quuz", "aaa", "bbb", "ccc", "ddd", "eee", "xxx",
            "yyy", "zzz", "test1", "test2", "test3",
        ];

        let mut issues = Vec::new();

        for group in &captures {
            if let Some(cap) = group.first() {
                let name = cap.text.to_lowercase();
                let chars: Vec<char> = name.chars().collect();
                let is_repeating = chars.len() >= 3 && chars.iter().all(|c| *c == chars[0]);
                let is_meaningless = meaningless.contains(&name.as_str()) || is_repeating;
                if is_meaningless {
                    let msgs = [
                        format!(
                            "Variable '{}'? Did you fall asleep on the keyboard?",
                            cap.text
                        ),
                        format!("'{}'? Naming is hard, but this is just sad", cap.text),
                        format!(
                            "A variable named '{}'? I've seen better names in random tests",
                            cap.text
                        ),
                        format!(
                            "'{}' is not a real variable name, it's a cry for help",
                            cap.text
                        ),
                        format!(
                            "Congratulations on naming a variable '{}' — truly innovative",
                            cap.text
                        ),
                    ];
                    let pos = cap.node.start_position();
                    let severity = if matches!(name.as_str(), "foo" | "bar" | "baz") {
                        Severity::Spicy
                    } else {
                        Severity::Mild
                    };
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "meaningless-naming".to_string(),
                        message: msgs[issues.len() % msgs.len()].clone(),
                        severity,
                    });
                }
            }
        }
        issues
    }

    fn check_with_context(
        &self,
        file: &ParsedFile,
        _is_test_file: bool,
        _context: &FileContext,
        config: &ProjectConfig,
    ) -> Vec<CodeIssue> {
        let issues = self.check(file);
        apply_naming_config(issues, &config.rules.naming)
    }
}

/// Dead code: detects unreachable code after return/break/continue/panic.
#[cfg_attr(not(test), expect(dead_code))]
pub(crate) struct DeadCodeRule;

impl TreeSitterRule for DeadCodeRule {
    fn name(&self) -> &'static str {
        "dead-code"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust, Language::Go]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let mut dead_start: Option<usize> = None;
        let mut reported = false;
        let is_go = file.language == Language::Go;

        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            let terminator = if is_go {
                is_go_terminator(trimmed)
            } else {
                is_rust_terminator(trimmed)
            };

            if terminator {
                dead_start = Some(line_num + 2);
                reported = false;
                continue;
            }

            if let Some(start) = dead_start {
                let comment_start = "//";
                if trimmed.is_empty()
                    || trimmed.starts_with(comment_start)
                    || trimmed.starts_with("/*")
                {
                    continue;
                }
                if trimmed == "}" || trimmed.starts_with("} else") || trimmed.starts_with("} else")
                {
                    dead_start = None;
                    continue;
                }
                if !reported && line_num + 1 >= start {
                    let msgs = [
                        "Dead code detected — this code never executes",
                        "Unreachable code! Return already happened, this is just decoration",
                        "Dead code walking... nothing after 'return' ever runs",
                    ];
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: line_num + 1,
                        column: 1,
                        rule_name: "dead-code".to_string(),
                        message: msgs[issues.len() % msgs.len()].to_string(),
                        severity: Severity::Mild,
                    });
                    reported = true;
                }
            }
        }
        issues
    }
}

#[cfg_attr(not(test), expect(dead_code))]
fn is_rust_terminator(line: &str) -> bool {
    let trimmed = line.trim();
    matches!(
        trimmed,
        "return;" | "break;" | "continue;" | "unreachable!()" | "unreachable!();"
    ) || (trimmed.starts_with("return ") && trimmed.ends_with(';'))
        || (trimmed.starts_with("panic!(") && trimmed.ends_with(';'))
        || (trimmed.starts_with("unreachable!(") && trimmed.ends_with(')'))
}

#[cfg_attr(not(test), expect(dead_code))]
fn is_go_terminator(line: &str) -> bool {
    let trimmed = line.trim();
    // Go: return, return x, break, continue, panic(...), or with optional semicolon
    trimmed == "return"
        || trimmed == "return;"
        || trimmed == "break"
        || trimmed == "break;"
        || trimmed == "continue"
        || trimmed == "continue;"
        || (trimmed.starts_with("return ") && (trimmed.ends_with(';') || !trimmed.ends_with('}')))
        || trimmed.starts_with("panic(")
        || trimmed.starts_with("goto ")
}

/// Duplicate imports detection.
pub(crate) struct DuplicateImportsRule;

impl TreeSitterRule for DuplicateImportsRule {
    fn name(&self) -> &'static str {
        "duplicate-imports"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut seen = std::collections::HashSet::new();
        let mut issues = Vec::new();
        let mut first_use_line = None;

        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") {
                if first_use_line.is_none() {
                    first_use_line = Some(line_num + 1);
                }
                if !seen.insert(trimmed.to_string()) {
                    let msgs = [
                        format!(
                            "Duplicate import '{}' — reading comprehension matters",
                            trimmed
                        ),
                        format!(
                            "Importing '{}' twice doesn't make it more imported",
                            trimmed
                        ),
                        format!("You already imported '{}' once. That was enough", trimmed),
                    ];
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: line_num + 1,
                        column: 1,
                        rule_name: "duplicate-imports".to_string(),
                        message: msgs[issues.len() % msgs.len()].clone(),
                        severity: Severity::Mild,
                    });
                }
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::parse_rust;
    use super::*;

    #[test]
    fn test_meaningless_names_detected() {
        let file = parse_rust("fn main() { let foo = 1; let aaa = 2; let xxx = 3; }");
        let rule = MeaninglessRule;
        let issues = rule.check(&file);
        assert!(issues.len() >= 3, "Should detect foo, aaa, xxx");
    }

    #[test]
    fn test_meaningful_names_clean() {
        let file = parse_rust("fn main() { let user_count = 1; let max_retries = 3; }");
        let rule = MeaninglessRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Good names should not trigger");
    }

    #[test]
    fn test_dead_code_detected() {
        let file = parse_rust(
            r#"
fn main() {
    return;
    let x = 1;
}
"#,
        );
        let rule = DeadCodeRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect dead code after return");
    }

    #[test]
    fn test_duplicate_imports_detected() {
        let file = parse_rust(
            r#"
use std::collections::HashMap;
use std::collections::HashMap;
fn main() {}
"#,
        );
        let rule = DuplicateImportsRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect duplicate import");
    }
}
