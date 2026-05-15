use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

pub fn register_python_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    engine.add(Box::new(BareExceptRule));
    engine.add(Box::new(WildcardImportRule));
}

/// bare except: `except:` without specifying exception type.
/// Every language has `except SomeError:`, but bare `except:` catches
/// KeyboardInterrupt, SystemExit, GeneratorExit — things you almost
/// never want to swallow silently.
struct BareExceptRule;

impl TreeSitterRule for BareExceptRule {
    fn name(&self) -> &'static str {
        "bare-except"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        // Walk AST: except_clause nodes without a 'value' child are bare except
        let mut issues = Vec::new();
        let root = file.root_node();
        find_bare_except(file, root, &mut issues);
        issues
    }
}

fn find_bare_except(file: &ParsedFile, node: tree_sitter::Node, issues: &mut Vec<CodeIssue>) {
    if node.kind() == "except_clause" {
        // bare except has no 'value' field child
        if node.child_by_field_name("value").is_none() {
            let pos = node.start_position();
            let msgs = [
                "Bare `except:` — catching KeyboardInterrupt and SystemExit too? Bold move.",
                "Bare `except:` is the programming equivalent of '¯\\_(ツ)_/¯'",
                "Bare `except:` — when you don't care what breaks, as long as it stops complaining",
                "Using bare `except:`? Even `except Exception:` would be less reckless.",
            ];
            issues.push(CodeIssue {
                file_path: file.path.clone(),
                line: pos.row + 1,
                column: pos.column + 1,
                rule_name: "bare-except".to_string(),
                message: msgs[issues.len() % msgs.len()].to_string(),
                severity: Severity::Spicy,
            });
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_bare_except(file, child, issues);
    }
}

/// wildcard import: `from module import *`
/// Pollutes the namespace, makes it impossible to track where names come from.
struct WildcardImportRule;

impl TreeSitterRule for WildcardImportRule {
    fn name(&self) -> &'static str {
        "wildcard-import"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Python]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let captures = match collect_captures(file, "(wildcard_import) @wi") {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        // Libraries where `import *` is idiomatic
        let acceptable_modules = [
            "manim",
            "numpy",
            "matplotlib",
            "pytest",
            "tensorflow",
            "torch",
            "tkinter",
            "PyQt5",
            "PySide6",
            "gi.repository",
        ];
        let mut issues = Vec::new();
        for group in &captures {
            if let Some(cap) = group.first() {
                let line = cap.node.start_position().row;
                // Check the source line for acceptable module imports
                if let Some(source_line) = file.content.lines().nth(line) {
                    let is_acceptable = acceptable_modules
                        .iter()
                        .any(|m| source_line.contains(&format!("from {} import *", m)));
                    if is_acceptable {
                        continue;
                    }
                }
                let pos = cap.node.start_position();
                let msgs = [
                    "`import *` — namespace pollution speedrun any%",
                    "Wildcard import? Good luck finding where that name came from.",
                    "`from x import *` — the 'I give up on explicit imports' special",
                    "Wildcard import detected. Your IDE is crying.",
                ];
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    rule_name: "wildcard-import".to_string(),
                    message: msgs[issues.len() % msgs.len()].to_string(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::TreeSitterEngine;
    use std::path::Path;

    fn parse_python(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(Path::new("test.py"), code)
            .expect("Should parse Python")
    }

    #[test]
    fn test_bare_except_detected() {
        let file = parse_python(
            r#"
try:
    risky()
except:
    pass
"#,
        );
        let rule = BareExceptRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect bare except");
    }

    #[test]
    fn test_typed_except_not_detected() {
        let file = parse_python(
            r#"
try:
    risky()
except ValueError:
    pass
"#,
        );
        let rule = BareExceptRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Typed except should not trigger");
    }

    #[test]
    fn test_wildcard_import_detected() {
        let file = parse_python(
            r#"
from os import *
from sys import *
"#,
        );
        let rule = WildcardImportRule;
        let issues = rule.check(&file);
        assert!(issues.len() >= 2, "Should detect both wildcard imports");
    }

    #[test]
    fn test_normal_import_not_detected() {
        let file = parse_python(
            r#"
from os import path
import sys
"#,
        );
        let rule = WildcardImportRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Normal imports should not trigger");
    }
}
