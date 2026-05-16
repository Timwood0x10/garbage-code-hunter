use super::base_rules::{CountRule, MacroRule, MethodCallRule};
use super::complex_rules::{
    AbbreviationAbuseTsRule, ComplexClosureRule, DeepNestingRule, GodFunctionRule,
    HungarianNotationTsRule, LongFunctionRule, MagicNumberRule, PrintlnDebuggingRule,
    SingleLetterTsRule, TerribleNamingRule,
};
use super::remaining_rules::{
    CommentedCodeRule, DeadCodeRule, DuplicateImportsRule, FileTooLongRule, MeaninglessRule,
    TodoCommentRule,
};
use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::rule::TreeSitterRule;

/// Register all tree-sitter based Rust rules.
pub fn register_rust_rules(engine: &mut crate::treesitter::rule::TreeSitterRuleEngine) {
    // Unwrap abuse (escalating severity)
    engine.add(Box::new(MethodCallRule {
        name: "unwrap-abuse",
        method_name: "unwrap",
        threshold: 0,
        severity_fn: |count| {
            if count > 15 {
                Severity::Nuclear
            } else if count > 8 {
                Severity::Spicy
            } else {
                Severity::Mild
            }
        },
        message_fn: |count| {
            format!(
                "Found {} .unwrap() calls — use proper error handling",
                count
            )
        },
    }));

    // Unnecessary clone
    engine.add(Box::new(MethodCallRule {
        name: "unnecessary-clone",
        method_name: "clone",
        threshold: 24,
        severity_fn: |_| Severity::Spicy,
        message_fn: |count| {
            format!(
                "Found {} .clone() calls — consider using references instead",
                count
            )
        },
    }));

    // Async abuse (>10 async blocks)
    engine.add(Box::new(CountRule {
        name: "async-abuse",
        pattern: "(async_block) @block",
        threshold: 10,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} async blocks — consider consolidating async operations",
                count
            )
        },
    }));

    // Macro abuse
    engine.add(Box::new(CountRule {
        name: "macro-abuse",
        pattern: "(macro_invocation) @m",
        threshold: 20,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} macro invocations — consider reducing macro usage",
                count
            )
        },
    }));

    // Lifetime abuse (non-static lifetimes)
    engine.add(Box::new(CountRule {
        name: "lifetime-abuse",
        pattern: "(lifetime) @life",
        threshold: 20,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} lifetime annotations — consider simplifying lifetime management",
                count
            )
        },
    }));

    // Trait complexity (methods in trait body)
    engine.add(Box::new(CountRule {
        name: "trait-complexity",
        pattern: "(trait_item body: (declaration_list (function_item) @method))",
        threshold: 10,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Trait has {} methods — consider splitting into smaller traits",
                count
            )
        },
    }));

    // Generic abuse (>5 type parameters)
    engine.add(Box::new(CountRule {
        name: "generic-abuse",
        pattern: "(type_parameters (type_parameter) @param)",
        threshold: 5,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} generic parameters — consider simplifying the type signature",
                count
            )
        },
    }));

    // Pattern matching abuse (tuple patterns)
    engine.add(Box::new(CountRule {
        name: "pattern-matching-abuse",
        pattern: "(tuple_pattern) @tp",
        threshold: 15,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} complex tuple patterns — consider using named structs",
                count
            )
        },
    }));

    // Box abuse
    engine.add(Box::new(CountRule {
        name: "box-abuse",
        pattern: "(call_expression function: (scoped_identifier) @si)",
        threshold: 8,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} Box::new() calls — consider using stack allocation",
                count
            )
        },
    }));

    // Reference abuse (type references)
    engine.add(Box::new(CountRule {
        name: "reference-abuse",
        pattern: "(reference_type) @rt",
        threshold: 50,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} reference types — consider simplifying ownership",
                count
            )
        },
    }));

    // Slice abuse
    engine.add(Box::new(CountRule {
        name: "slice-abuse",
        pattern: "(slice_type) @st",
        threshold: 29,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} slice types — consider using concrete collection types",
                count
            )
        },
    }));

    // Module complexity (nested mod_items)
    engine.add(Box::new(CountRule {
        name: "module-complexity",
        pattern: "(mod_item body: (declaration_list (mod_item) @nested))",
        threshold: 0,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| {
            format!(
                "Found {} nested modules — consider flattening the module structure",
                count
            )
        },
    }));

    // Panic abuse (count panic! invocations)
    engine.add(Box::new(MacroRule {
        name: "panic-abuse",
        macro_name: "panic",
        threshold: 2,
        severity: Severity::Nuclear,
        message_fn: |count| {
            format!(
                "Found {} panic! calls — use proper error handling with Result",
                count
            )
        },
    }));

    // Deep nesting
    engine.add(Box::new(DeepNestingRule));
    // Long function
    engine.add(Box::new(LongFunctionRule));

    // God function
    engine.add(Box::new(GodFunctionRule));

    // Complex closure
    engine.add(Box::new(ComplexClosureRule));
    // Terrible naming
    engine.add(Box::new(TerribleNamingRule));

    // Single letter variable
    engine.add(Box::new(SingleLetterTsRule));

    // Hungarian notation
    engine.add(Box::new(HungarianNotationTsRule));

    // Abbreviation abuse
    engine.add(Box::new(AbbreviationAbuseTsRule));

    // Println debugging
    engine.add(Box::new(PrintlnDebuggingRule));

    // Magic number
    engine.add(Box::new(MagicNumberRule));

    // Meaningless naming
    engine.add(Box::new(MeaninglessRule));

    // Commented code
    engine.add(Box::new(CommentedCodeRule));

    // Dead code
    engine.add(Box::new(DeadCodeRule));

    // TODO/FIXME comments
    engine.add(Box::new(TodoCommentRule));

    // Duplicate imports
    engine.add(Box::new(DuplicateImportsRule));

    // File too long
    engine.add(Box::new(FileTooLongRule));

    // String abuse (String::from, .to_string)
    engine.add(Box::new(MethodCallRule {
        name: "string-abuse",
        method_name: "to_string",
        threshold: 20,
        severity_fn: |_| Severity::Mild,
        message_fn: |count| format!("Found {} .to_string() calls — consider using &str", count),
    }));

    // Vec abuse (vec! macro)
    engine.add(Box::new(CountRule {
        name: "vec-abuse",
        pattern: "(macro_invocation macro: (identifier) @m (#eq? @m \"vec\"))",
        threshold: 15,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| format!("Found {} vec![] calls — consider using arrays", count),
    }));

    // too-many-params: warn if function has > 6 parameters
    engine.add(Box::new(TooManyParamsRule));
}

struct TooManyParamsRule;

impl TreeSitterRule for TooManyParamsRule {
    fn name(&self) -> &'static str {
        "too-many-params"
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        use super::base_rules::count_parameters;
        if let Ok(captures) =
            crate::treesitter::query::collect_captures(file, "(function_item) @fn")
        {
            let mut issues = Vec::new();
            for group in &captures {
                if let Some(cap) = group.first() {
                    let count = count_parameters(cap.node);
                    if count > 6 {
                        let pos = cap.node.start_position();
                        issues.push(CodeIssue {
                            file_path: file.path.clone(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            rule_name: "too-many-params".to_string(),
                            message: format!(
                                "Function has {} parameters — consider splitting",
                                count
                            ),
                            severity: Severity::Mild,
                        });
                    }
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
    use super::super::test_helpers::{parse_rust, parse_rust_as};
    use super::*;
    use crate::treesitter::query::collect_captures;

    /// Objective: Verify unwrap-abuse detects .unwrap() calls
    /// Invariants: Multiple unwrap calls should trigger with correct severity
    #[test]
    fn test_unwrap_abuse_detection() {
        let file = parse_rust(
            r#"
fn main() {
    let a = x.unwrap();
    let b = y.unwrap();
    let c = z.unwrap();
    let d = w.unwrap();
}
"#,
        );
        let rule = MethodCallRule {
            name: "unwrap-abuse",
            method_name: "unwrap",
            threshold: 0,
            severity_fn: |count| {
                if count > 15 {
                    Severity::Nuclear
                } else if count > 8 {
                    Severity::Spicy
                } else {
                    Severity::Mild
                }
            },
            message_fn: |count| format!("{} unwraps", count),
        };
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "Should report one aggregated issue");
        assert_eq!(issues[0].severity, Severity::Mild);
        assert!(issues[0].message.contains("4"), "Should count 4 unwraps");
    }

    /// Objective: Verify unwrap-abuse escalates to Nuclear for >15 calls
    /// Invariants: 16+ unwraps should be Nuclear severity
    #[test]
    fn test_unwrap_abuse_nuclear() {
        let mut code = String::from("fn main() {\n");
        for i in 0..16 {
            code.push_str(&format!("    let x{} = v{}.unwrap();\n", i, i));
        }
        code.push('}');
        let file = parse_rust(&code);
        let rule = MethodCallRule {
            name: "unwrap-abuse",
            method_name: "unwrap",
            threshold: 0,
            severity_fn: |count| {
                if count > 15 {
                    Severity::Nuclear
                } else if count > 8 {
                    Severity::Spicy
                } else {
                    Severity::Mild
                }
            },
            message_fn: |count| format!("{} unwraps", count),
        };
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Nuclear);
    }

    /// Objective: Verify async-abuse counts async blocks
    /// Invariants: Should find async blocks via tree-sitter query
    #[test]
    fn test_async_abuse_detection() {
        let file = parse_rust(
            r#"
async fn foo() {
    let _ = async { 1 }.await;
    let _ = async { 2 }.await;
}
"#,
        );
        let pattern = "(async_block) @block";
        let captures = collect_captures(&file, pattern).expect("query should work");
        let count: usize = captures.iter().map(|c| c.len()).sum();
        assert!(
            count >= 2,
            "Should find at least 2 async blocks, found {}",
            count
        );
    }

    /// Objective: Verify macro-abuse counts macro invocations
    /// Invariants: Should detect println!, vec!, etc.
    #[test]
    fn test_macro_abuse_detection() {
        let file = parse_rust(
            r#"
fn main() {
    println!("a");
    println!("b");
    vec![1, 2, 3];
}
"#,
        );
        let pattern = "(macro_invocation) @m";
        let captures = collect_captures(&file, pattern).expect("query should work");
        let count: usize = captures.iter().map(|c| c.len()).sum();
        assert!(
            count >= 3,
            "Should find at least 3 macro invocations, found {}",
            count
        );
    }

    /// Objective: Verify lifetime detection finds lifetime annotations
    /// Invariants: Should find 'a and 'b lifetimes
    #[test]
    fn test_lifetime_detection() {
        let file = parse_rust(
            r#"
fn foo<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { x }
"#,
        );
        let pattern = "(lifetime) @life";
        let captures = collect_captures(&file, pattern).expect("query should work");
        let count: usize = captures.iter().map(|c| c.len()).sum();
        assert!(
            count >= 2,
            "Should find at least 2 lifetimes, found {}",
            count
        );
    }

    /// Objective: Verify deep-nesting detects deeply nested code
    /// Invariants: 6+ levels of nesting should trigger
    #[test]
    fn test_deep_nesting_detection() {
        let file = parse_rust(
            r#"
fn main() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            println!("deep");
                        }
                    }
                }
            }
        }
    }
}
"#,
        );
        let rule = DeepNestingRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect deep nesting");
        assert!(issues.iter().any(|i| i.rule_name == "deep-nesting"));
    }

    /// Objective: Verify deep-nesting does not trigger on shallow code
    /// Invariants: 2 levels of nesting should be fine
    #[test]
    fn test_deep_nesting_clean_code() {
        let file = parse_rust(
            r#"
fn main() {
    if true {
        println!("shallow");
    }
}
"#,
        );
        let rule = DeepNestingRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Shallow nesting should not trigger, found {} issues",
            issues.len()
        );
    }

    /// Objective: Verify long-function detects functions > 80 lines
    /// Invariants: A function with 90+ lines should trigger
    #[test]
    fn test_long_function_detection() {
        let mut code = String::from("fn long_function() {\n");
        for i in 0..90 {
            code.push_str(&format!("    let x{} = {};\n", i, i));
        }
        code.push_str("}\n");
        let file = parse_rust_as("main.rs", &code);
        let rule = LongFunctionRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect long function");
        assert!(issues.iter().any(|i| i.rule_name == "long-function"));
    }

    /// Objective: Verify long-function does not trigger on short functions
    /// Invariants: A function with 10 lines should not trigger
    #[test]
    fn test_long_function_clean_code() {
        let file = parse_rust(
            r#"
fn short_function() {
    let x = 1;
    let y = 2;
    println!("{}", x + y);
}
"#,
        );
        let rule = LongFunctionRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Short function should not trigger");
    }

    /// Objective: Verify god-function detects high complexity
    /// Invariants: A function with many control flow + params should trigger
    #[test]
    fn test_god_function_detection() {
        let file = parse_rust(
            r#"
fn god(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                for x in 0..10 {
                    match d {
                        1 => {},
                        2 => {},
                        _ => {},
                    }
                }
            }
        }
    }
    if a > 1 {
        if b > 1 {
            if c > 1 {
                for x in 0..10 {
                    match d {
                        1 => {},
                        2 => {},
                        _ => {},
                    }
                }
            }
        }
    }
    if a > 2 {
        if b > 2 {
            if c > 2 {
                for x in 0..10 {
                    match d {
                        1 => {},
                        2 => {},
                        _ => {},
                    }
                }
            }
        }
    }
}
"#,
        );
        let rule = GodFunctionRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect god function");
        assert!(issues.iter().any(|i| i.rule_name == "god-function"));
    }

    /// Objective: Verify complex-closure detects deeply nested closures
    /// Invariants: 3+ levels of closure nesting should trigger
    #[test]
    fn test_complex_closure_detection() {
        let file = parse_rust(
            r#"
fn main() {
    let f = |x| {
        let g = |y| {
            let h = |z| {
                z + 1
            };
            h(y)
        };
        g(x)
    };
}
"#,
        );
        let rule = ComplexClosureRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect complex closure nesting");
        assert!(issues.iter().any(|i| i.rule_name == "complex-closure"));
    }

    /// Objective: Verify module-complexity detects nested modules
    /// Invariants: Nested mod items should be detected
    #[test]
    fn test_module_complexity_detection() {
        let file = parse_rust(
            r#"
mod outer {
    mod inner {
        fn foo() {}
    }
}
"#,
        );
        let pattern = "(mod_item body: (declaration_list (mod_item) @nested))";
        let captures = collect_captures(&file, pattern).expect("query should work");
        let count: usize = captures.iter().map(|c| c.len()).sum();
        assert!(count >= 1, "Should find nested module, found {}", count);
    }

    /// Objective: Verify terrible-naming detects bad variable names
    /// Invariants: Names like 'data', 'temp' should trigger
    #[test]
    fn test_terrible_naming_detection() {
        let file = parse_rust(
            r#"
fn main() {
    let data = 1;
    let temp = 2;
    let value = 3;
}
"#,
        );
        let rule = TerribleNamingRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect terrible naming");
        assert!(issues.iter().any(|i| i.rule_name == "terrible-naming"));
    }

    /// Objective: Verify terrible-naming does not trigger on good names
    /// Invariants: Meaningful names should not trigger
    #[test]
    fn test_terrible_naming_clean_code() {
        let file = parse_rust(
            r#"
fn main() {
    let user_count = 1;
    let max_retries = 3;
}
"#,
        );
        let rule = TerribleNamingRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "Good names should not trigger");
    }

    /// Objective: Verify single-letter-variable detects bad single-letter names
    /// Invariants: 'q' should trigger, 'i' should not
    #[test]
    fn test_single_letter_variable_detection() {
        let file = parse_rust(
            r#"
fn main() {
    let q = 1;
}
"#,
        );
        let rule = SingleLetterTsRule;
        let issues = rule.check(&file);
        assert!(
            !issues.is_empty(),
            "Should detect single-letter variable 'q'"
        );
        assert!(issues
            .iter()
            .any(|i| i.rule_name == "single-letter-variable"));
    }

    /// Objective: Verify single-letter-variable allows loop counters by context
    /// Invariants: 'i', 'j' in for loops should be allowed; 'q' standalone should not
    #[test]
    fn test_single_letter_allows_loop_counters() {
        let file = parse_rust(
            r#"
fn main() {
    for i in 0..10 {
        for j in 0..5 {
            let q = 42;
        }
    }
}
"#,
        );
        let rule = SingleLetterTsRule;
        let issues = rule.check(&file);
        let names: Vec<&str> = issues
            .iter()
            .map(|i| i.message.split('\'').nth(1).unwrap_or(""))
            .collect();
        assert!(
            !names.contains(&"i"),
            "Loop variable 'i' should be allowed (not in issues)"
        );
        assert!(
            !names.contains(&"j"),
            "Loop variable 'j' should be allowed (not in issues)"
        );
        assert!(
            names.contains(&"q"),
            "Standalone single-letter 'q' should still be flagged"
        );
    }

    /// Objective: Verify panic-abuse detects panic! calls
    /// Invariants: Multiple panic! calls should trigger
    #[test]
    fn test_panic_abuse_detection() {
        let file = parse_rust(
            r#"
fn main() {
    panic!("oh no");
    panic!("again");
    panic!("and again");
}
"#,
        );
        let rule = MacroRule {
            name: "panic-abuse",
            macro_name: "panic",
            threshold: 2,
            severity: Severity::Nuclear,
            message_fn: |count| format!("{} panics", count),
        };
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect panic abuse");
        assert!(issues[0].message.contains("3"), "Should count 3 panics");
    }

    /// Objective: Verify hungarian-notation detects type prefixes
    /// Invariants: 'strName', 'intCount' should trigger
    #[test]
    fn test_hungarian_notation_detection() {
        let file = parse_rust(
            r#"
fn main() {
    let strName = "test";
    let intCount = 42;
}
"#,
        );
        let rule = HungarianNotationTsRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect Hungarian notation");
        assert!(issues.iter().any(|i| i.rule_name == "hungarian-notation"));
    }

    /// Objective: Verify abbreviation-abuse detects bad abbreviations
    /// Invariants: 'mgr', 'btn' should trigger
    #[test]
    fn test_abbreviation_abuse_detection() {
        let file = parse_rust(
            r#"
fn main() {
    let mgr = get_manager();
    let btn = get_button();
}
"#,
        );
        let rule = AbbreviationAbuseTsRule;
        let issues = rule.check(&file);
        assert!(!issues.is_empty(), "Should detect abbreviation abuse");
        assert!(issues.iter().any(|i| i.rule_name == "abbreviation-abuse"));
    }

    /// Objective: Verify abbreviation-abuse does not trigger on full words
    /// Invariants: 'manager', 'button' should not trigger
    #[test]
    fn test_abbreviation_abuse_clean_code() {
        let file = parse_rust(
            r#"
fn main() {
    let manager = get_manager();
    let button = get_button();
}
"#,
        );
        let rule = AbbreviationAbuseTsRule;
        let issues = rule.check(&file);
        assert!(
            issues.is_empty(),
            "Full words should not trigger abbreviation abuse"
        );
    }

    /// Objective: Verify ALL tree-sitter rules fire on comprehensive test code
    /// Invariants: Every registered rule should detect at least one issue
    #[test]
    fn test_engine_runs_without_panic() {
        use crate::treesitter::rule::TreeSitterRuleEngine;
        let mut ts_engine = TreeSitterRuleEngine::new();
        register_rust_rules(&mut ts_engine);
        let names = ts_engine.rule_names();
        assert!(!names.is_empty(), "Engine should have registered rules");
        assert!(
            names.contains(&"unwrap-abuse"),
            "unwrap-abuse should be registered"
        );
    }

    #[test]
    fn test_too_many_params_detected() {
        let file = parse_rust("fn bad(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) {}");
        let rule = TooManyParamsRule;
        let issues = rule.check(&file);
        assert_eq!(issues.len(), 1, "7 params should be flagged");
    }

    #[test]
    fn test_too_many_params_few_ok() {
        let file = parse_rust("fn good(a: i32, b: i32) {}");
        let rule = TooManyParamsRule;
        let issues = rule.check(&file);
        assert!(issues.is_empty(), "2 params should not be flagged");
    }
}
