use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

pub fn register_go_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    // panic abuse: detect panic() calls
    engine.add(Box::new(PanicRule {
        name: "panic-abuse",
        threshold: 0,
        severity_fn: |count| {
            if count > 5 {
                Severity::Nuclear
            } else if count > 2 {
                Severity::Spicy
            } else {
                Severity::Mild
            }
        },
        message_fn: |count| {
            format!(
                "Found {} panic() calls — use proper error handling with Result",
                count
            )
        },
    }));

    // goroutine abuse (>8 go statements)
    engine.add(Box::new(CountGoRule {
        name: "goroutine-abuse",
        pattern: "(go_statement) @go",
        threshold: 8,
        severity: Severity::Spicy,
        message_fn: |count| {
            format!(
                "Found {} goroutine spawns — potential goroutine leaks",
                count
            )
        },
    }));

    // defer in loop: detect defer inside for loop body
    engine.add(Box::new(DeferInLoopRule));
}

struct PanicRule {
    name: &'static str,
    threshold: usize,
    severity_fn: fn(usize) -> Severity,
    message_fn: fn(usize) -> String,
}

impl TreeSitterRule for PanicRule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Go]
    }

    fn skips_test_files(&self) -> bool {
        true
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let pattern = r#"(call_expression function: (identifier) @f (#eq? @f "panic"))"#;
        let captures = match collect_captures(file, pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let count: usize = captures.iter().map(|c| c.len()).sum();
        if count > self.threshold {
            vec![CodeIssue {
                file_path: file.path.clone(),
                line: 1,
                column: 1,
                rule_name: self.name.to_string(),
                message: (self.message_fn)(count),
                severity: (self.severity_fn)(count),
            }]
        } else {
            vec![]
        }
    }
}

struct CountGoRule {
    name: &'static str,
    pattern: &'static str,
    threshold: usize,
    severity: Severity,
    message_fn: fn(usize) -> String,
}

impl TreeSitterRule for CountGoRule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Go]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let captures = match collect_captures(file, self.pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let count: usize = captures.iter().map(|c| c.len()).sum();
        if count > self.threshold {
            vec![CodeIssue {
                file_path: file.path.clone(),
                line: 1,
                column: 1,
                rule_name: self.name.to_string(),
                message: (self.message_fn)(count),
                severity: self.severity.clone(),
            }]
        } else {
            vec![]
        }
    }
}

struct DeferInLoopRule;

impl TreeSitterRule for DeferInLoopRule {
    fn name(&self) -> &'static str {
        "defer-in-loop"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Go]
    }

    fn skips_test_files(&self) -> bool {
        true
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let mut cursor = file.root_node().walk();
        check_defer_in_for(file, file.root_node(), &mut cursor, &mut issues);
        issues
    }
}

fn check_defer_in_for(
    file: &ParsedFile,
    node: tree_sitter::Node,
    cursor: &mut tree_sitter::TreeCursor,
    issues: &mut Vec<CodeIssue>,
) {
    if node.kind() == "for_statement" {
        // Check if any descendant is a defer_statement
        let mut child_cursor = node.walk();
        if child_cursor.goto_first_child() {
            loop {
                if has_defer_descendant(&file, child_cursor.node()) {
                    let pos = child_cursor.node().start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "defer-in-loop".to_string(),
                        message: "defer inside a for loop — deferred calls accumulate until the function returns"
                            .to_string(),
                        severity: Severity::Spicy,
                    });
                    break; // One issue per for_statement
                }
                if !child_cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    } else {
        if node.child_count() > 0 && cursor.goto_first_child() {
            loop {
                check_defer_in_for(file, cursor.node(), cursor, issues);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
}

fn has_defer_descendant(file: &ParsedFile, node: tree_sitter::Node) -> bool {
    if node.kind() == "defer_statement" {
        return true;
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if has_defer_descendant(file, cursor.node()) {
                return true;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::TreeSitterEngine;
    use std::path::Path;

    fn parse_go(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(Path::new("test.go"), code)
            .expect("Should parse Go")
    }

    #[test]
    fn test_panic_abuse_detection() {
        let file = parse_go(
            r#"
package main
func main() {
    panic("oh no")
    panic("again")
    panic("and again")
}
"#,
        );
        let rule = PanicRule {
            name: "panic-abuse",
            threshold: 0,
            severity_fn: |_| Severity::Nuclear,
            message_fn: |count| format!("{} panics", count),
        };
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect panic abuse");
        assert!(issues[0].message.contains("3"), "Should count 3 panics");
    }

    #[test]
    fn test_goroutine_abuse_detection() {
        let file = parse_go(
            r#"
package main
func main() {
    go work()
    go work()
    go work()
    go work()
    go work()
    go work()
    go work()
    go work()
    go work()
}
"#,
        );
        let rule = CountGoRule {
            name: "goroutine-abuse",
            pattern: "(go_statement) @go",
            threshold: 8,
            severity: Severity::Spicy,
            message_fn: |count| format!("{} goroutines", count),
        };
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect goroutine abuse");
        assert!(issues[0].message.contains("9"), "Should count 9 goroutines");
    }

    #[test]
    fn test_defer_in_loop_detection() {
        let file = parse_go(
            r#"
package main
func main() {
    for i := 0; i < 10; i++ {
        defer cleanup(i)
    }
}
"#,
        );
        let rule = DeferInLoopRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect defer in loop");
        assert!(issues.iter().any(|i| i.rule_name == "defer-in-loop"));
    }

    #[test]
    fn test_defer_outside_loop_not_detected() {
        let file = parse_go(
            r#"
package main
func main() {
    defer cleanup()
    for i := 0; i < 10; i++ {
        work(i)
    }
}
"#,
        );
        let rule = DeferInLoopRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Defer outside loop should not trigger");
    }
}
