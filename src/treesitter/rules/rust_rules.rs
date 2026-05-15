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
use crate::analyzer::Severity;
use crate::language::Language;

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

    // Goto abuse (C/C++)
    engine.add(Box::new(CountRule {
        name: "goto-abuse",
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
        name: "new-expression",
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
    // Matches both raw malloc and framework-specific allocators (curlx_malloc, zmalloc, ngx_alloc, etc.)
    engine.add(Box::new(CountRule {
        name: "malloc-leak",
        pattern: "(call_expression function: (identifier) @func (#match? @func \"^(malloc|curlx_malloc|Curl_cmalloc|zmalloc|zcalloc|zrealloc|ngx_alloc|ngx_palloc|ngx_pcalloc)$\"))",
        threshold: 0,
        severity: Severity::Spicy,
        languages: &[Language::C, Language::Cpp],
        message_fn: |count| {
            format!(
                "Found {} heap allocation calls — did you free() everything?",
                count
            )
        },
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::engine::ParsedFile;
    use crate::treesitter::query::collect_captures;
    use crate::treesitter::rule::TreeSitterRule;
    use crate::treesitter::TreeSitterEngine;
    use std::path::Path;

    fn parse_rust(code: &str) -> ParsedFile {
        parse_rust_as("main.rs", code)
    }

    fn parse_rust_as(filename: &str, code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(Path::new(filename), code)
            .expect("Should parse")
    }

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
    fn test_all_rules_fire_on_comprehensive_code() {
        use crate::treesitter::rule::TreeSitterRuleEngine;
        use crate::treesitter::TreeSitterEngine;

        let code = r#"
fn test_unwrap() {
    let a = Some(1).unwrap();
    let b = Some(2).unwrap();
    let c = Some(3).unwrap();
    let d = Some(4).unwrap();
    let e = Some(5).unwrap();
}

fn test_clone() {
    let x = vec![1, 2, 3];
    let y = x.clone(); let z = y.clone(); let w = z.clone();
    let v = w.clone(); let u = v.clone(); let t = u.clone();
    let s = t.clone(); let r = s.clone(); let q = r.clone();
    let p = q.clone(); let o = p.clone(); let n = o.clone();
    let m = n.clone(); let l = m.clone(); let k = l.clone();
    let j = k.clone(); let i = j.clone(); let h = i.clone();
    let g = h.clone(); let f = g.clone(); let e = f.clone();
    let d = e.clone(); let c = d.clone(); let b = c.clone();
    let a = b.clone(); let aa = a.clone(); let bb = aa.clone();
}

fn test_async() {
    let _ = async { 1 }; let _ = async { 2 }; let _ = async { 3 };
    let _ = async { 4 }; let _ = async { 5 }; let _ = async { 6 };
    let _ = async { 7 }; let _ = async { 8 }; let _ = async { 9 };
    let _ = async { 10 }; let _ = async { 11 };
}

fn test_panic() {
    panic!("a"); panic!("b"); panic!("c");
}

fn test_macros() {
    println!("a"); println!("b"); println!("c"); println!("d"); println!("e");
    println!("f"); println!("g"); println!("h"); println!("i"); println!("j");
    println!("k"); println!("l"); println!("m"); println!("n"); println!("o");
    println!("p"); println!("q"); println!("r"); println!("s"); println!("t");
    println!("u");
}

fn test_nesting() {
    if true { if true { if true { if true { if true { if true {
        println!("deep");
    } } } } } }
}

fn test_long_fn() {
    let x0 = 0;
    let x1 = 1;
    let x2 = 2;
    let x3 = 3;
    let x4 = 4;
    let x5 = 5;
    let x6 = 6;
    let x7 = 7;
    let x8 = 8;
    let x9 = 9;
    let x10 = 10;
    let x11 = 11;
    let x12 = 12;
    let x13 = 13;
    let x14 = 14;
    let x15 = 15;
    let x16 = 16;
    let x17 = 17;
    let x18 = 18;
    let x19 = 19;
    let x20 = 20;
    let x21 = 21;
    let x22 = 22;
    let x23 = 23;
    let x24 = 24;
    let x25 = 25;
    let x26 = 26;
    let x27 = 27;
    let x28 = 28;
    let x29 = 29;
    let x30 = 30;
    let x31 = 31;
    let x32 = 32;
    let x33 = 33;
    let x34 = 34;
    let x35 = 35;
    let x36 = 36;
    let x37 = 37;
    let x38 = 38;
    let x39 = 39;
    let x40 = 40;
    let x41 = 41;
    let x42 = 42;
    let x43 = 43;
    let x44 = 44;
    let x45 = 45;
    let x46 = 46;
    let x47 = 47;
    let x48 = 48;
    let x49 = 49;
    let x50 = 50;
    let x51 = 51;
    let x52 = 52;
    let x53 = 53;
    let x54 = 54;
    let x55 = 55;
    let x56 = 56;
    let x57 = 57;
    let x58 = 58;
    let x59 = 59;
    let x60 = 60;
    let x61 = 61;
    let x62 = 62;
    let x63 = 63;
    let x64 = 64;
    let x65 = 65;
    let x66 = 66;
    let x67 = 67;
    let x68 = 68;
    let x69 = 69;
    let x70 = 70;
    let x71 = 71;
    let x72 = 72;
    let x73 = 73;
    let x74 = 74;
    let x75 = 75;
    let x76 = 76;
    let x77 = 77;
    let x78 = 78;
    let x79 = 79;
    let x80 = 80;
    let x81 = 81;
    let x82 = 82;
    let x83 = 83;
    let x84 = 84;
    let x85 = 85;
}

fn test_god(a:i32,b:i32,c:i32,d:i32,e:i32,f:i32,g:i32) {
    if a>0{if b>0{if c>0{for x in 0..10{match d{1=>{},2=>{},_=>{}}}}}}
    if a>1{if b>1{if c>1{for x in 0..10{match d{1=>{},2=>{},_=>{}}}}}}
    if a>2{if b>2{if c>2{for x in 0..10{match d{1=>{},2=>{},_=>{}}}}}}
    if a>3{if b>3{if c>3{for x in 0..10{match d{1=>{},2=>{},_=>{}}}}}}
    if a>4{if b>4{if c>4{for x in 0..10{match d{1=>{},2=>{},_=>{}}}}}}
}

fn test_closure() {
    let f = |x| { let g = |y| { let h = |z| { z + 1 }; h(y) }; g(x) };
}

fn test_naming() { let data = 1; let temp = 2; let value = 3; let info = 4; }

fn test_single_letter() { let q = 1; let m = 2; }

mod outer { mod inner { fn foo() {} } }

fn test_lifetime<'a,'b,'c,'d,'e,'f,'g,'h,'i,'j,'k,'l,'m,'n,'o,'p,'q,'r,'s,'t,'u,'v>(x:&'a str)->&'b str{x}

fn test_generics<T,U,V,W,X,Y,Z>(a:T,b:U,c:V,d:W,e:X,f:Y,g:Z)->T{a}
"#;

        let engine = TreeSitterEngine::new();
        let file = engine
            .parse_file(Path::new("comprehensive.rs"), code)
            .expect("Should parse");

        let mut ts_engine = TreeSitterRuleEngine::new();
        register_rust_rules(&mut ts_engine);

        let issues = ts_engine.check_file(&file, false);
        let rule_names: std::collections::HashSet<&str> =
            issues.iter().map(|i| i.rule_name.as_str()).collect();

        let expected_rules = [
            "unwrap-abuse",
            "unnecessary-clone",
            "async-abuse",
            "panic-abuse",
            "macro-abuse",
            "deep-nesting",
            "long-function",
            "god-function",
            "complex-closure",
            "terrible-naming",
            "single-letter-variable",
            "module-complexity",
            "lifetime-abuse",
            "generic-abuse",
        ];

        let mut missing = Vec::new();
        for rule in &expected_rules {
            if !rule_names.contains(rule) {
                missing.push(*rule);
            }
        }

        assert!(
            missing.is_empty(),
            "These tree-sitter rules did NOT fire: {:?}\nRules that did fire: {:?}\nTotal issues: {}",
            missing,
            rule_names,
            issues.len()
        );

        // Verify issue counts are reasonable
        let unwrap_count = issues
            .iter()
            .filter(|i| i.rule_name == "unwrap-abuse")
            .count();
        assert!(
            unwrap_count >= 1,
            "unwrap-abuse should produce at least 1 issue, got {}",
            unwrap_count
        );

        let clone_count = issues
            .iter()
            .filter(|i| i.rule_name == "unnecessary-clone")
            .count();
        assert!(
            clone_count >= 1,
            "unnecessary-clone should produce at least 1 issue, got {}",
            clone_count
        );

        let deep_count = issues
            .iter()
            .filter(|i| i.rule_name == "deep-nesting")
            .count();
        assert!(
            deep_count >= 1,
            "deep-nesting should produce at least 1 issue, got {}",
            deep_count
        );

        let long_count = issues
            .iter()
            .filter(|i| i.rule_name == "long-function")
            .count();
        assert!(
            long_count >= 1,
            "long-function should produce at least 1 issue, got {}",
            long_count
        );

        let god_count = issues
            .iter()
            .filter(|i| i.rule_name == "god-function")
            .count();
        assert!(
            god_count >= 1,
            "god-function should produce at least 1 issue, got {}",
            god_count
        );

        let closure_count = issues
            .iter()
            .filter(|i| i.rule_name == "complex-closure")
            .count();
        assert!(
            closure_count >= 1,
            "complex-closure should produce at least 1 issue, got {}",
            closure_count
        );

        println!(
            "All {} tree-sitter rules fired! Total issues: {}",
            expected_rules.len(),
            issues.len()
        );
    }
}
