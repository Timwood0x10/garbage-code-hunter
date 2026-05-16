use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

pub fn register_ruby_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    engine.add(Box::new(GlobalVariableRule));
    engine.add(Box::new(BareRescueRule));
    engine.add(Box::new(FrozenStringRule));
    engine.add(Box::new(NegatedIfRule));
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

// ─── Ruby frozen_string_literal: file should start with magic comment ──

struct FrozenStringRule;

impl TreeSitterRule for FrozenStringRule {
    fn name(&self) -> &'static str {
        "frozen-string"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Ruby]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let first_line = file.content.lines().next().unwrap_or("");
        if !first_line.contains("frozen_string_literal: true") {
            return vec![CodeIssue {
                file_path: file.path.clone(),
                line: 1,
                column: 1,
                rule_name: "frozen-string".to_string(),
                message: "Missing '# frozen_string_literal: true' at top of file".to_string(),
                severity: Severity::Mild,
            }];
        }
        vec![]
    }
}

// ─── Ruby negated if: `if !x` → `unless x` ─────────────────────────

struct NegatedIfRule;

impl TreeSitterRule for NegatedIfRule {
    fn name(&self) -> &'static str {
        "negated-if"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Ruby]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if (trimmed.starts_with("if !") || trimmed.starts_with("if("))
                && trimmed.contains('!')
                && !trimmed.contains("!= ")
            {
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: line_num + 1,
                    column: 1,
                    rule_name: "negated-if".to_string(),
                    message: "Use 'unless' instead of 'if !'".to_string(),
                    severity: Severity::Mild,
                });
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::parse_ruby;
    use super::*;

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

    #[test]
    fn test_frozen_string_missing() {
        let file = parse_ruby("def foo; end\n");
        let rule = FrozenStringRule;
        let issues = rule.check(&file);
        assert_eq!(
            issues.len(),
            1,
            "Missing frozen_string_literal should be flagged"
        );
    }

    #[test]
    fn test_frozen_string_present_ok() {
        let file = parse_ruby("# frozen_string_literal: true\n\ndef foo; end\n");
        let rule = FrozenStringRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Present frozen_string_literal should be OK"
        );
    }

    #[test]
    fn test_negated_if_detected() {
        let file = parse_ruby("def foo\n  if !x\n    puts 'not'\n  end\nend\n");
        let rule = NegatedIfRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "if !x should be flagged");
        assert_eq!(issues[0].rule_name, "negated-if");
    }

    #[test]
    fn test_negated_if_unless_not_flagged() {
        let file = parse_ruby("def foo\n  unless x\n    puts 'not'\n  end\nend\n");
        let rule = NegatedIfRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "unless should not be flagged");
    }
}
