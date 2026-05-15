use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

pub fn register_ruby_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    engine.add(Box::new(GlobalVariableRule));
    engine.add(Box::new(BareRescueRule));
}

/// Global variables ($prefix) in Ruby.
/// Global variables are the lazy person's dependency injection.
/// They make code untestable and create invisible coupling.
struct GlobalVariableRule;

impl TreeSitterRule for GlobalVariableRule {
    fn name(&self) -> &'static str {
        "global-variable"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Ruby]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let captures = match collect_captures(file, "(global_variable) @gv") {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut issues = Vec::new();
        // Ruby has some built-in globals ($stdout, $stderr, $stdin, $VERBOSE, etc.)
        // that are acceptable
        let acceptable: &[&str] = &[
            "$stdout",
            "$stderr",
            "$stdin",
            "$VERBOSE",
            "$DEBUG",
            "$SAFE",
            "$LOAD_PATH",
            "$LOADED_FEATURES",
            "$PROGRAM_NAME",
            "$FILENAME",
            "$.",
            "$,",
            "$;",
            "$/",
            "$\\",
            "$&",
            "$`",
            "$'",
            "$+",
            "$~",
            "$=",
            "$<",
            "$>",
            "$!",
            "$?",
            "$0",
            "$*",
            "$_",
            "$-d",
            "$-v",
            "$-w",
            "$-W",
        ];
        for group in &captures {
            if let Some(cap) = group.first() {
                let name = cap.text.trim();
                if acceptable.contains(&name) {
                    continue;
                }
                let pos = cap.node.start_position();
                let msgs = [
                    format!(
                        "Global variable `{}` — because who needs testable code?",
                        name
                    ),
                    format!(
                        "`{}` is a global variable. Your future self will not thank you.",
                        name
                    ),
                    format!(
                        "Global `{}` detected. Invisible coupling speedrun any%.",
                        name
                    ),
                ];
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    rule_name: "global-variable".to_string(),
                    message: msgs[issues.len() % msgs.len()].clone(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

/// bare rescue: `rescue => e` without specifying exception class
/// In Ruby, bare `rescue` catches StandardError by default, which is
/// broader than most people realize.
struct BareRescueRule;

impl TreeSitterRule for BareRescueRule {
    fn name(&self) -> &'static str {
        "bare-rescue"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Ruby]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let root = file.root_node();
        find_bare_rescue(file, root, &mut issues);
        issues
    }
}

fn find_bare_rescue(file: &ParsedFile, node: tree_sitter::Node, issues: &mut Vec<CodeIssue>) {
    // Only check named rescue nodes, skip the anonymous 'rescue' keyword token
    if node.kind() == "rescue" && node.is_named() {
        // bare rescue has no 'exceptions' child node
        // typed rescue: `rescue StandardError => e` has an `exceptions` child
        let has_exceptions = node
            .children(&mut node.walk())
            .any(|c| c.kind() == "exceptions");
        if !has_exceptions {
            let pos = node.start_position();
            let msgs = [
                "Bare `rescue` — catching all StandardErrors? At least be specific.",
                "Bare `rescue` detected. What exactly are you rescuing us from?",
                "Bare `rescue` — the Ruby equivalent of 'catch (Exception e) {}'",
            ];
            issues.push(CodeIssue {
                file_path: file.path.clone(),
                line: pos.row + 1,
                column: pos.column + 1,
                rule_name: "bare-rescue".to_string(),
                message: msgs[issues.len() % msgs.len()].to_string(),
                severity: Severity::Mild,
            });
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_bare_rescue(file, child, issues);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::TreeSitterEngine;
    use std::path::Path;

    fn parse_ruby(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(Path::new("test.rb"), code)
            .expect("Should parse Ruby")
    }

    #[test]
    fn test_global_variable_detected() {
        let file = parse_ruby(
            r#"
$my_global = 42
puts $my_global
"#,
        );
        let rule = GlobalVariableRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect global variable");
    }

    #[test]
    fn test_builtin_global_not_detected() {
        let file = parse_ruby(
            r#"
$stdout.puts "hello"
$stderr.puts "error"
"#,
        );
        let rule = GlobalVariableRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Built-in globals ($stdout, $stderr) should not trigger"
        );
    }

    #[test]
    fn test_bare_rescue_detected() {
        let file = parse_ruby(
            r#"
begin
  risky
rescue
  puts "oops"
end
"#,
        );
        let rule = BareRescueRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect bare rescue");
    }

    #[test]
    fn test_typed_rescue_not_detected() {
        let file = parse_ruby(
            r#"
begin
  risky
rescue StandardError => e
  puts e.message
end
"#,
        );
        let rule = BareRescueRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Typed rescue should not trigger");
    }
}
