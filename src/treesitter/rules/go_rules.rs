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

    // go-receiver-name: method receiver should be 1-2 chars
    engine.add(Box::new(GoReceiverNameRule));

    // go-error-string: error strings should not start with capital letter
    engine.add(Box::new(GoErrorStringRule));

    // go-context-first: context.Context must be the first parameter
    engine.add(Box::new(GoContextFirstRule));

    // go-else-return: if-else with return in if should use early return
    engine.add(Box::new(GoElseReturnRule));

    // go-mixed-caps: Go convention is camelCase, not snake_case or ALL_CAPS for locals
    engine.add(Box::new(GoMixedCapsRule));
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
                if has_defer_descendant(file, child_cursor.node()) {
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

#[expect(clippy::only_used_in_recursion)]
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

// ─── Go: receiver name should be 1-2 chars ──────────────────────────

struct GoReceiverNameRule;

impl TreeSitterRule for GoReceiverNameRule {
    fn name(&self) -> &'static str {
        "go-receiver-name"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Go]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let pattern = "(method_declaration receiver: (parameter_list (parameter_declaration name: (identifier) @rec)))";
        let captures = match collect_captures(file, pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut issues = Vec::new();
        for group in &captures {
            if let Some(cap) = group.first() {
                let name = cap.text;
                if name.len() > 2 {
                    let pos = cap.node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "go-receiver-name".to_string(),
                        message: format!(
                            "Receiver '{}' is {} chars; Go convention is 1-2 chars",
                            name,
                            name.len()
                        ),
                        severity: Severity::Mild,
                    });
                }
            }
        }
        issues
    }
}

// ─── Go: error strings should not start with capital letter ─────────

struct GoErrorStringRule;

impl TreeSitterRule for GoErrorStringRule {
    fn name(&self) -> &'static str {
        "go-error-string"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Go]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let pattern = r#"(call_expression function: (selector_expression operand: (identifier) @pkg field: (field_identifier) @method) (#eq? @pkg "fmt") (#match? @method "^(Errorf|New)$"))"#;
        let captures = match collect_captures(file, pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut issues = Vec::new();
        for group in &captures {
            if let Some(cap) = group.first() {
                // Get parent call_expression (selector_expression → call_expression)
                let call = cap.node.parent().and_then(|p| p.parent());
                if let Some(call_node) = call {
                    let mut cursor = call_node.walk();
                    for child in call_node.children(&mut cursor) {
                        if child.kind() == "argument_list" {
                            let child_text = file.node_text(child);
                            let trimmed = child_text.trim();
                            // Extract first string argument content
                            let start = trimmed.find('"');
                            let content = if let Some(s) = start {
                                let from_quote = &trimmed[s + 1..];
                                from_quote.find('"').map(|e| &from_quote[..e]).unwrap_or("")
                            } else {
                                ""
                            };
                            if !content.is_empty() {
                                let first = content.chars().next().unwrap_or(' ');
                                if first.is_uppercase() {
                                    let pos = call_node.start_position();
                                    issues.push(CodeIssue {
                                        file_path: file.path.clone(),
                                        line: pos.row + 1,
                                        column: pos.column + 1,
                                        rule_name: "go-error-string".to_string(),
                                        message: format!(
                                            "Error string starts with uppercase '{}' — use lowercase per Go convention",
                                            first
                                        ),
                                        severity: Severity::Mild,
                                    });
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        issues
    }
}

// ─── Go: context.Context must be first parameter ───────────────────

struct GoContextFirstRule;

impl TreeSitterRule for GoContextFirstRule {
    fn name(&self) -> &'static str {
        "go-context-first"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Go]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip non-function lines (no "func" keyword starting the line)
            if !trimmed.starts_with("func ") {
                continue;
            }
            // Extract parameter list between '(' and ')'
            let params_start = trimmed.find('(');
            let params_end = trimmed.rfind(')');
            if let (Some(ps), Some(pe)) = (params_start, params_end) {
                let params_str = &trimmed[ps + 1..pe];
                // Check if context.Context appears anywhere in params
                if params_str.contains("context.Context") {
                    // Check if it's NOT the first parameter
                    let first_param = params_str.split(',').next().unwrap_or("").trim();
                    if !first_param.contains("context.Context") {
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: line_num + 1,
                            column: trimmed.find("context.Context").unwrap_or(0) + 1,
                            rule_name: "go-context-first".to_string(),
                            message: "context.Context should be the first parameter".to_string(),
                            severity: Severity::Mild,
                        });
                    }
                }
            }
        }
        issues
    }
}

// ─── Go: if-else with return should be early return ────────────────

struct GoElseReturnRule;

impl TreeSitterRule for GoElseReturnRule {
    fn name(&self) -> &'static str {
        "go-else-return"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Go]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let root = file.root_node();
        find_if_else_return(file, root, &mut issues);
        issues
    }
}

fn find_if_else_return(file: &ParsedFile, node: tree_sitter::Node, issues: &mut Vec<CodeIssue>) {
    if node.kind() == "if_statement" {
        let has_else = node.children(&mut node.walk()).any(|c| c.kind() == "else");
        if has_else {
            // Check if the if-body (consequence) contains a return_statement
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if (child.kind() == "block" || child.kind() == "compound_statement")
                    && contains_return(file, child)
                {
                    let pos = node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "go-else-return".to_string(),
                        message: "Use early return instead of if-else with return in if-block"
                            .to_string(),
                        severity: Severity::Mild,
                    });
                    break;
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_if_else_return(file, child, issues);
    }
}

#[expect(clippy::only_used_in_recursion)]
fn contains_return(file: &ParsedFile, node: tree_sitter::Node) -> bool {
    if node.kind() == "return_statement" {
        return true;
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if contains_return(file, cursor.node()) {
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

// ─── Go: mixed-caps — Go convention is camelCase, not snake_case ─────

struct GoMixedCapsRule;

impl TreeSitterRule for GoMixedCapsRule {
    fn name(&self) -> &'static str {
        "go-mixed-caps"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Go]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            // Look for `var name` or `name :=` patterns
            let name = if let Some(rest) = trimmed.strip_prefix("var ") {
                rest.split_whitespace().next().unwrap_or("")
            } else if let Some(idx) = trimmed.find(":=") {
                trimmed[..idx].split_whitespace().last().unwrap_or("")
            } else {
                ""
            };
            if name.is_empty() || name.len() < 2 {
                continue;
            }
            // Skip Go-idiomatic names: err, ok, ctx, mu, wg, ch, db, id, ip, etc.
            let go_idioms = [
                "err", "ok", "ctx", "mu", "wg", "ch", "db", "id", "ip", "tx", "rx", "fd", "fs",
                "ns", "fn", "hp", "os", "rc",
            ];
            if go_idioms.contains(&name) || name == "_" {
                continue;
            }
            // Skip struct field access (e.g., s.field) and type names
            if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                continue;
            }
            // Detect snake_case (has underscore, not just _)
            let has_underscore = name.contains('_') && name != "_";
            // Detect ALL_CAPS (all uppercase letters with possible underscores)
            let is_all_caps = name
                .chars()
                .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
                && name.chars().any(|c| c.is_uppercase());
            if has_underscore || is_all_caps {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: 1,
                    rule_name: "go-mixed-caps".to_string(),
                    message: format!(
                        "'{}' should use camelCase per Go convention (Effective Go)",
                        name
                    ),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::parse_go;
    use super::*;

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

    #[test]
    fn test_go_receiver_name_short_ok() {
        let file = parse_go(
            r#"
package main
type T struct{}
func (s *T) Good() {}   // 1 char — OK
func (srv *T) Bad() {}  // 3 chars — should flag
"#,
        );
        let rule = GoReceiverNameRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "Only srv should be flagged");
        assert!(issues[0].message.contains("srv"), "srv should be flagged");
    }

    #[test]
    fn test_go_context_first_detected() {
        let file = parse_go(
            r#"
package main
import "context"
func Bad(ctx context.Context, x int) {}    // ctx is first — OK
func AlsoBad(x int, ctx context.Context) {} // ctx is NOT first — flag
func Fine(ctx context.Context, x int, y string) {} // ctx is first — OK
"#,
        );
        let rule = GoContextFirstRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "Only AlsoBad should be flagged");
        assert!(
            issues.iter().any(|i| i.rule_name == "go-context-first"),
            "Rule name mismatch"
        );
    }

    #[test]
    fn test_go_else_return_detected() {
        let file = parse_go(
            r#"
package main
func foo() error {
    if err != nil {
        return err
    } else {
        return nil
    }
}
"#,
        );
        let rule = GoElseReturnRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect else-return pattern");
        assert_eq!(issues[0].rule_name, "go-else-return");
    }

    #[test]
    fn test_go_else_return_no_issue() {
        let file = parse_go(
            r#"
package main
func foo() error {
    if err != nil {
        return err
    }
    return nil
}
"#,
        );
        let rule = GoElseReturnRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Early return without else should not trigger"
        );
    }

    #[test]
    fn test_go_error_string_uppercase() {
        let file = parse_go(
            r#"
package main
import "fmt"
func ok() error  { return fmt.Errorf("lowercase ok") }    // lowercase — OK
func bad() error { return fmt.Errorf("Uppercase bad") }   // uppercase — flag
func also_bad() error { return fmt.Errorf("Invalid input") }  // uppercase — flag
"#,
        );
        let rule = GoErrorStringRule;
        let issues = rule.check(&file);
        // The lowercase error should be OK, two uppercase should be flagged
        assert_eq!(
            issues.len(),
            2,
            "Should flag 2 uppercase errors, got {}",
            issues.len()
        );
        assert!(
            issues.iter().all(|i| i.rule_name == "go-error-string"),
            "Rule name mismatch"
        );
    }

    #[test]
    fn test_go_mixed_caps_snake_case() {
        let file = parse_go(
            r#"
package main
func main() {
    my_var := 42
    goodName := "ok"
}
"#,
        );
        let rule = GoMixedCapsRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "Only my_var should be flagged");
        assert!(issues[0].message.contains("my_var"));
    }

    #[test]
    fn test_go_mixed_caps_camel_case_ok() {
        let file = parse_go(
            r#"
package main
func main() {
    goodName := "ok"
    x := 1
}
"#,
        );
        let rule = GoMixedCapsRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "camelCase should not be flagged");
    }

    #[test]
    fn test_go_mixed_caps_idiom_ok() {
        let file = parse_go(
            r#"
package main
func main() {
    err := doSomething()
    ok := check()
    ctx := getContext()
}
"#,
        );
        let rule = GoMixedCapsRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Go idioms (err, ok, ctx) should not be flagged"
        );
    }
}
