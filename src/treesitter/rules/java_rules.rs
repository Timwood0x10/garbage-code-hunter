use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::rule::TreeSitterRule;

pub fn register_java_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    engine.add(Box::new(EmptyCatchRule));
}

/// Empty catch block: `catch (Exception e) {}`
/// Swallowing exceptions silently is the #1 source of "it works on my machine"
/// bugs that take 3 days to debug in production.
struct EmptyCatchRule;

impl TreeSitterRule for EmptyCatchRule {
    fn name(&self) -> &'static str {
        "empty-catch"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Java]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let root = file.root_node();
        find_empty_catch(file, root, &mut issues);
        issues
    }
}

fn find_empty_catch(file: &ParsedFile, node: tree_sitter::Node, issues: &mut Vec<CodeIssue>) {
    if node.kind() == "catch_clause" {
        // Check if the body block is empty
        if let Some(body) = node.child_by_field_name("body") {
            let named_count = body.named_child_count();
            if named_count == 0 {
                let pos = node.start_position();
                let msgs = [
                    "Empty catch block — exceptions don't fix themselves by being ignored",
                    "Swallowing exceptions silently? Bold strategy, let's see how it plays out.",
                    "Empty catch block. Future you will spend 3 days debugging this.",
                    "Catch block is empty. At least log it, you monster.",
                    "Empty catch block detected — the debugging equivalent of 'la la la I can't hear you'",
                ];
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    rule_name: "empty-catch".to_string(),
                    message: msgs[issues.len() % msgs.len()].to_string(),
                    severity: Severity::Spicy,
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_empty_catch(file, child, issues);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::TreeSitterEngine;
    use std::path::Path;

    fn parse_java(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(Path::new("Test.java"), code)
            .expect("Should parse Java")
    }

    #[test]
    fn test_empty_catch_detected() {
        let file = parse_java(
            r#"
class Test {
    void foo() {
        try {
            risky();
        } catch (Exception e) {
        }
    }
}
"#,
        );
        let rule = EmptyCatchRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect empty catch");
    }

    #[test]
    fn test_non_empty_catch_not_detected() {
        let file = parse_java(
            r#"
class Test {
    void foo() {
        try {
            risky();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
"#,
        );
        let rule = EmptyCatchRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Non-empty catch should not trigger");
    }
}
