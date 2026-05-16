use crate::analyzer::Severity;
use crate::language::Language;

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

#[cfg(test)]
mod tests {
    use crate::treesitter::engine::ParsedFile;
    use crate::treesitter::query::collect_captures;
    use crate::treesitter::TreeSitterEngine;
    use std::path::Path;

    fn parse_c(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(Path::new("test.c"), code)
            .expect("Should parse C")
    }

    fn parse_cpp(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(Path::new("test.cpp"), code)
            .expect("Should parse C++")
    }

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
}
