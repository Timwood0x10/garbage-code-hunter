use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

pub fn register_ts_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    engine.add(Box::new(AnyTypeRule));
    engine.add(Box::new(PreferInterfaceRule));
}

/// `any` type usage in TypeScript.
/// Using `any` defeats the entire purpose of TypeScript's type system.
/// It's the "I'll fix it later" that never gets fixed.
struct AnyTypeRule;

impl TreeSitterRule for AnyTypeRule {
    fn name(&self) -> &'static str {
        "any-type"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::TypeScript]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        // Skip declaration files (.d.ts) where `any` is often necessary
        if file.path.to_string_lossy().ends_with(".d.ts") {
            return vec![];
        }
        // predefined_type captures built-in types; filter for "any"
        let captures = match collect_captures(file, "(predefined_type) @t") {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut issues = Vec::new();
        for group in &captures {
            if let Some(cap) = group.first() {
                if cap.text.trim() != "any" {
                    continue;
                }
                // Skip if inside a type assertion — sometimes unavoidable
                let pos = cap.node.start_position();
                let msgs = [
                    "`any` type detected — TypeScript's type system is crying",
                    "Using `any`? Just write JavaScript at that point.",
                    "`any` — the TypeScript equivalent of ¯\\_(ツ)_/¯",
                    "Found `any` type. You had one job: use types.",
                    "`any` type? Congratulations, you've invented dynamically typed TypeScript.",
                ];
                issues.push(CodeIssue {
                    file_path: file.path.clone(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    rule_name: "any-type".to_string(),
                    message: msgs[issues.len() % msgs.len()].to_string(),
                    severity: Severity::Spicy,
                });
            }
        }
        issues
    }
}

// ─── prefer-interface: type Foo = { ... } should be interface Foo { ... } ──

struct PreferInterfaceRule;

impl TreeSitterRule for PreferInterfaceRule {
    fn name(&self) -> &'static str {
        "prefer-interface"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::TypeScript]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let pattern =
            "(type_alias_declaration name: (type_identifier) @name value: (object_type) @obj)";
        if let Ok(captures) = collect_captures(file, pattern) {
            let mut issues = Vec::new();
            for group in &captures {
                if let Some(cap) = group.first() {
                    let pos = cap.node.start_position();
                    issues.push(CodeIssue {
                        file_path: file.path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        rule_name: "prefer-interface".to_string(),
                        message: "Use 'interface' instead of 'type' for object types".to_string(),
                        severity: Severity::Mild,
                    });
                }
            }
            issues
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::parse_ts;
    use super::*;

    #[test]
    fn test_any_type_detected() {
        let file = parse_ts(
            r#"
function foo(x: any): any {
    return x;
}
"#,
        );
        let rule = AnyTypeRule;
        let issues = rule.check(&file);
        assert!(
            issues.len() >= 2,
            "Should detect both `any` types (param + return)"
        );
    }

    #[test]
    fn test_typed_params_not_detected() {
        let file = parse_ts(
            r#"
function foo(x: string): number {
    return 42;
}
"#,
        );
        let rule = AnyTypeRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Typed params should not trigger");
    }

    #[test]
    fn test_prefer_interface_detected() {
        let file = parse_ts("type Person = { name: string; };\n");
        let rule = PreferInterfaceRule;
        let issues = rule.check(&file);
        assert_eq!(
            issues.len(),
            1,
            "type alias with object type should be flagged"
        );
    }

    #[test]
    fn test_prefer_interface_interface_ok() {
        let file = parse_ts("interface Person { name: string; }\n");
        let rule = PreferInterfaceRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "interface should not be flagged");
    }
}
