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

/// Commented code: detects blocks of commented-out code.
#[cfg_attr(not(test), expect(dead_code))]
pub(crate) struct CommentedCodeRule;

impl TreeSitterRule for CommentedCodeRule {
    fn name(&self) -> &'static str {
        "commented-code"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let mut block_start = 0;
        let mut block_size = 0;

        let line_comment = file.language.line_comment();

        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip doc comments (///) and tool directives, only flag actual commented-out code
            if trimmed.starts_with("///") || trimmed.starts_with("/**") {
                continue;
            }
            if trimmed.starts_with(line_comment) {
                let comment_text = trimmed.strip_prefix(line_comment).unwrap_or("").trim();
                if is_likely_code(comment_text) {
                    if block_size == 0 {
                        block_start = line_num + 1;
                    }
                    block_size += 1;
                } else if block_size > 0 {
                    emit_comment_block(&mut issues, file, block_start, block_size);
                    block_size = 0;
                }
            } else if !trimmed.is_empty() && block_size > 0 {
                emit_comment_block(&mut issues, file, block_start, block_size);
                block_size = 0;
            }
        }
        if block_size > 0 {
            emit_comment_block(&mut issues, file, block_start, block_size);
        }
        issues
    }
}

#[cfg_attr(not(test), expect(dead_code))]
fn is_likely_code(text: &str) -> bool {
    let code_patterns = [
        "fn ", "if ", "else", "for ", "while ", "match ", "struct ", "enum ", "impl ", "let ",
        "return ", "use ", "mod ", "break", "continue", "{", "}", "(", ")", "[", "]", ";", "=",
        "==", "!=", "&&", "||", "->", "::",
    ];
    let keywords = [
        "pub", "const", "static", "mut", "ref", "move", "async", "await", "unsafe", "extern",
        "crate", "def", "class", "import", "from", "lambda", "function", "var", "let", "const",
        "if", "else",
    ];
    let pattern_count = code_patterns.iter().filter(|p| text.contains(*p)).count();
    let keyword_count = keywords.iter().filter(|k| text.contains(*k)).count();
    pattern_count >= 2 || keyword_count >= 1
}

#[cfg_attr(not(test), expect(dead_code))]
fn emit_comment_block(
    issues: &mut Vec<CodeIssue>,
    file: &ParsedFile,
    start_line: usize,
    size: usize,
) {
    if size < 3 {
        return;
    }
    let msgs = [
        format!(
            "{} lines of commented-out code — commit or delete, don't hoard",
            size
        ),
        format!(
            "Found {} lines of dead comment code. Git exists for a reason",
            size
        ),
        format!(
            "Commenting out code is like keeping an ex's photos. Let it go ({} lines)",
            size
        ),
    ];
    let severity = if size > 10 {
        Severity::Spicy
    } else {
        Severity::Mild
    };
    issues.push(CodeIssue {
        file_path: file.path.clone(),
        line: start_line,
        column: 1,
        rule_name: "commented-code".to_string(),
        message: msgs[issues.len() % msgs.len()].clone(),
        severity,
    });
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

/// TODO/FIXME comment detection.
pub(crate) struct TodoCommentRule;

impl TreeSitterRule for TodoCommentRule {
    fn name(&self) -> &'static str {
        "todo-comment"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let mut total_todos = 0;

        // Count todo!(), unimplemented!(), unreachable!() calls via query
        for pattern in &[
            "(macro_invocation macro: (identifier) @m (#eq? @m \"todo\"))",
            "(macro_invocation macro: (identifier) @m (#eq? @m \"unimplemented\"))",
            "(macro_invocation macro: (identifier) @m (#eq? @m \"unreachable\"))",
        ] {
            if let Ok(caps) = collect_captures(file, pattern) {
                total_todos += caps.iter().map(|c| c.len()).sum::<usize>();
            }
        }

        let line_comment = file.language.line_comment();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(pos) = trimmed.find(line_comment) {
                let comment = trimmed[pos + line_comment.len()..].trim();
                let upper = comment.to_uppercase();

                let has_todo = upper.starts_with("TODO") || upper.contains(" TODO ");
                let has_fixme = upper.starts_with("FIXME") || upper.contains(" FIXME ");
                let has_bug = upper.starts_with("BUG") || upper.contains(" BUG ");
                let has_hack = upper.starts_with("HACK") || upper.contains(" HACK ");

                if has_todo {
                    total_todos += 1;
                }
                if has_fixme {
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: line_num + 1,
                        column: pos + 1,
                        rule_name: "todo-fixme".to_string(),
                        message: format!("FIXME: {}", comment.trim()),
                        severity: Severity::Mild,
                    });
                }
                if has_bug {
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: line_num + 1,
                        column: pos + 1,
                        rule_name: "todo-bug".to_string(),
                        message: format!("BUG: {}", comment.trim()),
                        severity: Severity::Spicy,
                    });
                }
                if has_hack {
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: line_num + 1,
                        column: pos + 1,
                        rule_name: "todo-hack".to_string(),
                        message: format!("HACK: {}", comment.trim()),
                        severity: Severity::Mild,
                    });
                }
            }
        }

        if total_todos > 3 {
            let sev = if total_todos > 10 {
                Severity::Spicy
            } else {
                Severity::Mild
            };
            let msgs = [
                format!(
                    "Found {} TODO markers — your backlog must be terrifying",
                    total_todos
                ),
                format!(
                    "{} TODOs left in code. Future you will hate present you",
                    total_todos
                ),
                format!("{} unfinished tasks. Ship now, fix later?", total_todos),
            ];
            issues.push(CodeIssue {
                file_path: file.path.clone(),
                line: 1,
                column: 1,
                rule_name: "todo-comment".to_string(),
                message: msgs[issues.len() % msgs.len()].clone(),
                severity: sev,
            });
        }

        issues
    }
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

/// File too long detection.
pub(crate) struct FileTooLongRule;

impl TreeSitterRule for FileTooLongRule {
    fn name(&self) -> &'static str {
        "file-too-long"
    }

    fn supported_languages(&self) -> &'static [Language] {
        crate::language::LANGUAGES_WITH_GRAMMAR
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let line_count = file.content.lines().count();
        let is_test = file.path.to_string_lossy().contains("test");
        let threshold = if is_test { 2000 } else { 1000 };

        if line_count > threshold {
            let msgs = [
                format!(
                    "{} lines! Is this a file or a novel? Split it up",
                    line_count
                ),
                format!(
                    "This file has {} lines. Your editor is judging you",
                    line_count
                ),
                format!(
                    "{} lines in one file — that's not 'modular', that's a disaster",
                    line_count
                ),
            ];
            let severity = if line_count > 2000 {
                Severity::Nuclear
            } else if line_count > 1500 {
                Severity::Spicy
            } else {
                Severity::Mild
            };
            vec![CodeIssue {
                file_path: file.path.clone(),
                line: 1,
                column: 1,
                rule_name: "file-too-long".to_string(),
                message: msgs[0].clone(),
                severity,
            }]
        } else {
            vec![]
        }
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
    fn test_commented_code_detected() {
        let file = parse_rust(
            r#"
fn main() {
    // let x = foo();
    // let y = bar();
    // let z = baz();
    println!("real");
}
"#,
        );
        let rule = CommentedCodeRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect commented-out code");
    }

    #[test]
    fn test_todo_comment_detected() {
        let file = parse_rust(
            r#"
fn main() {
    // TODO: implement this
    // FIXME: this is broken
    // TODO: also this
    // TODO: and one more
    // XXX: cleanup required
    todo!();
}
"#,
        );
        let rule = TodoCommentRule;
        let issues = rule.check(&file);
        let rule_names: Vec<_> = issues.iter().map(|i| i.rule_name.as_str()).collect();
        assert!(rule_names.contains(&"todo-fixme"), "Should detect FIXME");
        assert!(rule_names.contains(&"todo-comment"), "Should detect TODO");
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

    #[test]
    fn test_file_too_long_detected() {
        // Generate a file with >1000 lines
        let mut code = String::from("fn main() {\n");
        for i in 0..1100 {
            code.push_str(&format!("    let x{} = {};\n", i, i));
        }
        code.push_str("}\n");
        let file = parse_rust(&code);
        let rule = FileTooLongRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect file >1000 lines");
    }

    #[test]
    fn test_file_too_long_short_ok() {
        let file = parse_rust("fn main() { println!(\"hello\"); }\n");
        let rule = FileTooLongRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Short file should not trigger");
    }
}
