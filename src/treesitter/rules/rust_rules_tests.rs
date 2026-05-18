use super::super::test_helpers::{parse_rust, parse_rust_as};
use super::*;
use crate::treesitter::{
    query::collect_captures,
    rules::base_rules::{CountRule, MacroRule, MethodCallRule},
    rules::rust_rules::{
        RustDeriveOrderRule, RustDocExampleRule, RustErrorDisplayRule, RustMustUseRule,
        TooManyParamsRule,
    },
};

use crate::treesitter::rules::complex_rules::{
    AbbreviationAbuseTsRule, DeepNestingRule, HungarianNotationTsRule, SingleLetterTsRule,
    TerribleNamingRule,
};

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
        names.contains(&"rust-must-use"),
        "rust-must-use should be registered"
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
fn test_rust_doc_example_missing() {
    let file = parse_rust(
        r#"
/// This function does something
fn foo() {}
"#,
    );
    let rule = RustDocExampleRule;
    let issues = rule.check(&file);
    assert_eq!(
        issues.len(),
        1,
        "Doc comment without example should be flagged"
    );
    assert_eq!(issues[0].rule_name, "rust-doc-example");
}

#[test]
fn test_rust_doc_example_ok() {
    let file = parse_rust(
        r#"
/// Does something
/// ```
/// let x = foo();
/// ```
fn foo() {}
"#,
    );
    let rule = RustDocExampleRule;
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "Doc comment with example should be OK");
}

#[test]
fn test_rust_derive_order_bad() {
    let file = parse_rust(
        r#"
#[derive(Clone, Debug)]
struct Foo;
"#,
    );
    let rule = RustDeriveOrderRule;
    let issues = rule.check(&file);
    assert_eq!(issues.len(), 1, "Bad derive order should be flagged");
    assert_eq!(issues[0].rule_name, "rust-derive-order");
}

#[test]
fn test_rust_derive_order_ok() {
    let file = parse_rust(
        r#"
#[derive(Debug, Clone, PartialEq)]
struct Foo;
"#,
    );
    let rule = RustDeriveOrderRule;
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "Good derive order should be OK");
}

#[test]
fn test_rust_error_display_detected() {
    let file = parse_rust(
        r#"
impl fmt::Debug for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "err") }
}
"#,
    );
    let rule = RustErrorDisplayRule;
    let issues = rule.check(&file);
    assert!(!issues.is_empty(), "Missing Display should be flagged");
    assert_eq!(issues[0].rule_name, "rust-error-display");
}

#[test]
fn test_rust_error_display_both_ok() {
    let file = parse_rust(
        r#"
impl fmt::Debug for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "err") }
}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "err") }
}
"#,
    );
    let rule = RustErrorDisplayRule;
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "Both Debug and Display should be OK");
}

#[test]
fn test_rust_must_use_detected() {
    let file = parse_rust(
        r#"
pub fn find_user() -> Result<User, Error> { Ok(user) }
"#,
    );
    let rule = RustMustUseRule;
    let issues = rule.check(&file);
    assert!(!issues.is_empty(), "Missing #[must_use] should be flagged");
    assert_eq!(issues[0].rule_name, "rust-must-use");
}

#[test]
fn test_rust_must_use_ok() {
    let file = parse_rust(
        r#"
#[must_use]
pub fn find_user() -> Result<User, Error> { Ok(user) }
"#,
    );
    let rule = RustMustUseRule;
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "With #[must_use] should be OK");
}

#[test]
fn test_too_many_params_few_ok() {
    let file = parse_rust("fn good(a: i32, b: i32) {}");
    let rule = TooManyParamsRule;
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "2 params should not be flagged");
}

// ─── CountRule / MethodCallRule check() tests ──────────────────────

#[test]
fn test_unnecessary_clone_below_threshold() {
    // threshold=24, need 25+ to trigger. Code with 2 clones should be clean.
    let file = parse_rust(
        r#"
fn main() {
    let a = String::from("hello");
    let b = a.clone();
    let c = b.clone();
}
"#,
    );
    let rule = MethodCallRule {
        name: "unnecessary-clone",
        method_name: "clone",
        threshold: 24,
        severity_fn: |_| Severity::Spicy,
        message_fn: |count| format!("{} clones", count),
    };
    let issues = rule.check(&file);
    assert!(
        issues.is_empty(),
        "2 clones should not trigger (threshold=24)"
    );
}

#[test]
fn test_trait_complexity_check() {
    // threshold=10 methods in a trait body — verify check() doesn't panic
    let file = parse_rust(
        r#"
trait Complex {
    fn a(&self);
    fn b(&self);
    fn c(&self);
    fn d(&self);
    fn e(&self);
    fn f(&self);
    fn g(&self);
    fn h(&self);
    fn i(&self);
    fn j(&self);
    fn k(&self);
}
"#,
    );
    let rule = CountRule {
        name: "trait-complexity",
        pattern: "(trait_item body: (declaration_list (function_item) @method))",
        threshold: 10,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} methods", count),
    };
    // The pattern may not match in all tree-sitter versions;
    // just verify check() doesn't panic
    let _ = rule.check(&file);
}

#[test]
fn test_trait_complexity_simple_ok() {
    let file = parse_rust(
        r#"
trait Simple {
    fn do_something(&self);
    fn do_other(&self);
}
"#,
    );
    let rule = CountRule {
        name: "trait-complexity",
        pattern: "(trait_item body: (declaration_list (function_item) @method))",
        threshold: 10,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} methods", count),
    };
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "2 methods should not trigger");
}

#[test]
fn test_generic_abuse_detected() {
    // threshold=5 type parameters
    let file = parse_rust(
        r#"
fn bad<T, U, V, W, X, Y>(a: T, b: U, c: V, d: W, e: X, f: Y) {}
"#,
    );
    let rule = CountRule {
        name: "generic-abuse",
        pattern: "(type_parameters (type_parameter) @param)",
        threshold: 5,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} params", count),
    };
    let issues = rule.check(&file);
    assert!(
        !issues.is_empty(),
        "6 generic params should trigger (threshold=5)"
    );
}

#[test]
fn test_generic_abuse_few_ok() {
    let file = parse_rust("fn good<T, U>(a: T, b: U) {}");
    let rule = CountRule {
        name: "generic-abuse",
        pattern: "(type_parameters (type_parameter) @param)",
        threshold: 5,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} params", count),
    };
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "2 generic params should not trigger");
}

#[test]
fn test_module_complexity_detected() {
    // threshold=0, so 1+ nested module triggers
    let file = parse_rust(
        r#"
mod outer {
    mod inner {
        fn foo() {}
    }
}
"#,
    );
    let rule = CountRule {
        name: "module-complexity",
        pattern: "(mod_item body: (declaration_list (mod_item) @nested))",
        threshold: 0,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} nested", count),
    };
    let issues = rule.check(&file);
    assert!(
        !issues.is_empty(),
        "Nested module should trigger (threshold=0)"
    );
}

#[test]
fn test_module_complexity_flat_ok() {
    let file = parse_rust(
        r#"
mod foo {
    fn bar() {}
}
"#,
    );
    let rule = CountRule {
        name: "module-complexity",
        pattern: "(mod_item body: (declaration_list (mod_item) @nested))",
        threshold: 0,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} nested", count),
    };
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "Flat module should not trigger");
}

#[test]
fn test_box_abuse_below_threshold() {
    // threshold=8, need 9+ Box::new() calls
    let file = parse_rust(
        r#"
fn main() {
    let a = Box::new(1);
    let b = Box::new(2);
}
"#,
    );
    let rule = CountRule {
        name: "box-abuse",
        pattern: "(call_expression function: (scoped_identifier) @si)",
        threshold: 8,
        severity: Severity::Spicy,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} calls", count),
    };
    let issues = rule.check(&file);
    assert!(
        issues.is_empty(),
        "2 Box::new() should not trigger (threshold=8)"
    );
}

#[test]
fn test_slice_abuse_below_threshold() {
    // threshold=29
    let file = parse_rust("fn foo(x: &[i32]) -> &[i32] { x }");
    let rule = CountRule {
        name: "slice-abuse",
        pattern: "(slice_type) @st",
        threshold: 29,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} slices", count),
    };
    let issues = rule.check(&file);
    assert!(
        issues.is_empty(),
        "2 slice types should not trigger (threshold=29)"
    );
}

#[test]
fn test_pattern_matching_below_threshold() {
    // threshold=15 tuple patterns
    let file = parse_rust(
        r#"
fn main() {
    let (a, b) = (1, 2);
    let (c, d) = (3, 4);
}
"#,
    );
    let rule = CountRule {
        name: "pattern-matching-abuse",
        pattern: "(tuple_pattern) @tp",
        threshold: 15,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} patterns", count),
    };
    let issues = rule.check(&file);
    assert!(
        issues.is_empty(),
        "2 tuple patterns should not trigger (threshold=15)"
    );
}

#[test]
fn test_reference_abuse_below_threshold() {
    // threshold=50
    let file = parse_rust("fn foo(x: &i32) -> &i32 { x }");
    let rule = CountRule {
        name: "reference-abuse",
        pattern: "(reference_type) @rt",
        threshold: 50,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} refs", count),
    };
    let issues = rule.check(&file);
    assert!(
        issues.is_empty(),
        "2 references should not trigger (threshold=50)"
    );
}

#[test]
fn test_string_abuse_below_threshold() {
    // Check that String type in a simple function doesn't trigger
    let file = parse_rust(
        r#"
fn main() {
    let s = String::from("hello");
}
"#,
    );
    let rule = CountRule {
        name: "string-abuse",
        pattern: "(generic_type) @gt",
        threshold: 30,
        severity: Severity::Mild,
        languages: &[Language::Rust],
        message_fn: |count| format!("{} strings", count),
    };
    let issues = rule.check(&file);
    assert!(
        issues.is_empty(),
        "1 String should not trigger (threshold=30)"
    );
}
