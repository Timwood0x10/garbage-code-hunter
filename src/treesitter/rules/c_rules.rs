use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::rule::TreeSitterRule;

use super::base_rules::CountRule;

/// Register all C/C++ specific rules.
pub fn register_c_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    // Goto abuse (C/C++)
    engine.add(Box::new(CountRule {
        name: "c-goto-abuse",
        pattern: "(goto_statement) @goto",
        threshold: 0,
        severity: Severity::Spicy,
        languages: &[Language::C, Language::Cpp],
        message_fn: |count| {
            format!(
                "Found {} goto statements — Dijkstra is turning in his grave",
                count
            )
        },
    }));

    // C++ new expression detection
    engine.add(Box::new(CountRule {
        name: "c-new-expression",
        pattern: "(new_expression) @new",
        threshold: 0,
        severity: Severity::Spicy,
        languages: &[Language::Cpp],
        message_fn: |count| {
            format!(
                "Found {} new expressions — did you delete() everything?",
                count
            )
        },
    }));

    // c-malloc-check: malloc return value not checked for NULL
    engine.add(Box::new(CMallocCheckRule));

    // c-sizeof-type: using sizeof(type) instead of sizeof(expr)
    engine.add(Box::new(CSizeofTypeRule));

    // Malloc leak detection (C/C++): count heap allocation calls
    engine.add(Box::new(CountRule {
        name: "c-malloc-leak",
        pattern: r#"(call_expression function: (identifier) @func (#match? @func "^(malloc|curlx_malloc|Curl_cmalloc|zmalloc|zcalloc|zrealloc|ngx_alloc|ngx_palloc|ngx_pcalloc)$"))"#,
        threshold: 0,
        severity: Severity::Spicy,
        languages: &[Language::C, Language::Cpp],
        message_fn: |count| {
            format!("Found {} heap allocation calls — did you free() everything?", count)
        },
    }));
}

// ─── C: malloc return value not checked for NULL ─────────────────

struct CMallocCheckRule;

impl TreeSitterRule for CMallocCheckRule {
    fn name(&self) -> &'static str {
        "c-malloc-check"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::C, Language::Cpp]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("malloc(")
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("*")
            {
                // Check if there's a NULL check on the next few lines
                let lines: Vec<&str> = file.content.lines().collect();
                let mut has_null_check = false;
                for k in 1..=3 {
                    if let Some(next) = lines.get(line_num + k) {
                        let next_trimmed = next.trim();
                        if next_trimmed.contains("== NULL")
                            || next_trimmed.contains("!= NULL")
                            || next_trimmed.contains("== 0")
                            || next_trimmed.contains("!= 0")
                            || next_trimmed.contains("if (!")
                            || next_trimmed.contains("if (NULL")
                        {
                            has_null_check = true;
                            break;
                        }
                    }
                }
                if !has_null_check {
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: line_num + 1,
                        column: trimmed.find("malloc(").unwrap_or(0) + 1,
                        rule_name: "c-malloc-check".to_string(),
                        message: "malloc return value not checked for NULL".to_string(),
                        severity: Severity::Spicy,
                    });
                }
            }
        }
        issues
    }
}

// ─── C: sizeof(type) instead of sizeof(expr) ──────────────────────

struct CSizeofTypeRule;

impl TreeSitterRule for CSizeofTypeRule {
    fn name(&self) -> &'static str {
        "c-sizeof-type"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::C, Language::Cpp]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let type_keywords: &[&str] = &[
            "int", "char", "float", "double", "long", "short", "unsigned", "signed", "void",
            "size_t", "bool", "struct", "union", "enum", "uint8_t", "uint16_t", "uint32_t",
            "uint64_t", "int8_t", "int16_t", "int32_t", "int64_t",
        ];
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            if let Some(pos) = trimmed.find("sizeof(") {
                let after = &trimmed[pos + 7..];
                let inner = after.split(')').next().unwrap_or("").trim();
                if !inner.is_empty() {
                    let first_word = inner.split_whitespace().next().unwrap_or("");
                    if type_keywords.contains(&first_word) {
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: line_num + 1,
                            column: pos + 1,
                            rule_name: "c-sizeof-type".to_string(),
                            message: "Use sizeof(expression) instead of sizeof(type)".to_string(),
                            severity: Severity::Mild,
                        });
                    }
                }
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::{parse_c, parse_cpp};
    use super::*;
    use crate::treesitter::query::collect_captures;

    #[test]
    fn test_goto_detected() {
        let file = parse_c(
            r#"
void foo() {
    goto cleanup;
cleanup:
    return;
}
"#,
        );
        let pattern = "(goto_statement) @goto";
        if let Ok(captures) = collect_captures(&file, pattern) {
            let count: usize = captures.iter().map(|c| c.len()).sum();
            assert!(count > 0, "Should detect goto statement");
        } else {
            panic!("Query failed");
        }
    }

    #[test]
    fn test_new_expression_detected() {
        let file = parse_cpp(
            r#"
void foo() {
    int* p = new int(42);
}
"#,
        );
        let pattern = "(new_expression) @new";
        if let Ok(captures) = collect_captures(&file, pattern) {
            let count: usize = captures.iter().map(|c| c.len()).sum();
            assert!(count > 0, "Should detect new expression");
        } else {
            panic!("Query failed");
        }
    }

    #[test]
    fn test_malloc_detected() {
        let file = parse_c(
            r#"
#include <stdlib.h>
void foo() {
    int* p = (int*)malloc(sizeof(int) * 10);
}
"#,
        );
        let pattern = r#"(call_expression function: (identifier) @func (#eq? @func "malloc"))"#;
        if let Ok(captures) = collect_captures(&file, pattern) {
            let count: usize = captures.iter().map(|c| c.len()).sum();
            assert!(count > 0, "Should detect malloc call");
        } else {
            panic!("Query failed");
        }
    }

    #[test]
    fn test_malloc_check_detected() {
        let file = parse_c(
            r#"
void foo() {
    int* p = (int*)malloc(10 * sizeof(int));
    *p = 42;
}
"#,
        );
        let rule = CMallocCheckRule;
        let issues = rule.check(&file);
        assert!(
            !issues.is_empty(),
            "Missing NULL check after malloc should be flagged"
        );
        assert_eq!(issues[0].rule_name, "c-malloc-check");
    }

    #[test]
    fn test_malloc_check_with_null_ok() {
        let file = parse_c(
            r#"
void foo() {
    int* p = (int*)malloc(10 * sizeof(int));
    if (p == NULL) {
        return;
    }
    *p = 42;
}
"#,
        );
        let rule = CMallocCheckRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "With NULL check after malloc should be OK"
        );
    }

    #[test]
    fn test_sizeof_type_detected() {
        let file = parse_c(
            r#"
void foo() {
    int* p = malloc(sizeof(int));
}
"#,
        );
        let rule = CSizeofTypeRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "sizeof(type) should be flagged");
        assert_eq!(issues[0].rule_name, "c-sizeof-type");
    }

    #[test]
    fn test_sizeof_expr_ok() {
        let file = parse_c(
            r#"
void foo() {
    int* p = malloc(sizeof(*p));
}
"#,
        );
        let rule = CSizeofTypeRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "sizeof(expr) should not be flagged");
    }

    #[test]
    fn test_no_goto_no_issue() {
        let file = parse_c(
            r#"
void foo() {
    int x = 42;
    return x;
}
"#,
        );
        let pattern = "(goto_statement) @goto";
        if let Ok(captures) = collect_captures(&file, pattern) {
            let count: usize = captures.iter().map(|c| c.len()).sum();
            assert_eq!(count, 0, "No goto should not trigger");
        } else {
            panic!("Query failed");
        }
    }

    #[test]
    fn test_malloc_leak_check() {
        let file = parse_c(
            r#"
#include <stdlib.h>
void foo() {
    int* p = (int*)malloc(10 * sizeof(int));
}
"#,
        );
        let rule = CountRule {
            name: "c-malloc-leak",
            pattern: r#"(call_expression function: (identifier) @func (#match? @func "^(malloc|curlx_malloc|Curl_cmalloc|zmalloc|zcalloc|zrealloc|ngx_alloc|ngx_palloc|ngx_pcalloc)$"))"#,
            threshold: 0,
            severity: Severity::Spicy,
            languages: &[Language::C, Language::Cpp],
            message_fn: |count| format!("{} mallocs", count),
        };
        let issues = rule.check(&file);
        assert!(
            !issues.is_empty(),
            "malloc call should trigger c-malloc-leak"
        );
    }
}
