use super::super::test_helpers::parse_rust;
use super::*;

#[test]
fn test_magic_number_detected() {
    // Magic number rule skips literals inside let/const assignments
    // but flags them in expressions like function args and comparisons
    let file = parse_rust(
        r#"
fn main() {
    foo(42 + 99);
    bar(256);
}
"#,
    );
    let rule = MagicNumberRule;
    let issues = rule.check(&file);
    assert!(
        !issues.is_empty(),
        "Should detect magic numbers in expressions (99, 256)"
    );
}

#[test]
fn test_magic_number_const_ok() {
    let file = parse_rust(
        r#"
const MAX: i32 = 42;
fn main() {
    let x = MAX;
}
"#,
    );
    let rule = MagicNumberRule;
    let issues = rule.check(&file);
    assert!(
        issues.is_empty(),
        "Named constants should not be flagged as magic numbers"
    );
}

#[test]
fn test_println_debugging_detected() {
    let file = parse_rust(
        r#"
fn main() {
    let x = 42;
    println!("{}", x);
    println!("debug");
}
"#,
    );
    let rule = PrintlnDebuggingRule;
    let issues = rule.check(&file);
    assert!(!issues.is_empty(), "Should detect println! debugging");
}

#[test]
fn test_println_debugging_empty() {
    let file = parse_rust(
        r#"
fn main() {
    let x = 42;
}
"#,
    );
    let rule = PrintlnDebuggingRule;
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "No println! should not trigger");
}

#[test]
fn test_deep_nesting_detected() {
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
    assert!(!issues.is_empty(), "Should detect deep nesting (6 levels)");
}

#[test]
fn test_deep_nesting_shallow_ok() {
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
    assert!(issues.is_empty(), "2-level nesting should not trigger");
}

#[test]
fn test_terrible_naming_detected() {
    let file = parse_rust(
        r#"
fn main() {
    let data = 1;
    let temp = 2;
    let info = 3;
    let obj = 4;
    let result = 5;
    let value = 6;
    let item = 7;
}
"#,
    );
    let rule = TerribleNamingRule;
    let issues = rule.check(&file);
    assert!(
        issues.len() >= 5,
        "Should detect terrible names (data, temp, info, obj, result, value, item)"
    );
}

#[test]
fn test_terrible_naming_good_names_ok() {
    let file = parse_rust(
        r#"
fn main() {
    let user_count = 1;
    let max_retries = 3;
    let connection_timeout = 30;
}
"#,
    );
    let rule = TerribleNamingRule;
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "Good names should not trigger");
}

#[test]
fn test_single_letter_detected() {
    let file = parse_rust(
        r#"
fn main() {
    let q = 1;
    let m = 2;
    let z = 3;
}
"#,
    );
    let rule = SingleLetterTsRule;
    let issues = rule.check(&file);
    assert!(
        !issues.is_empty(),
        "Should detect single-letter variables (q, m, z)"
    );
}

#[test]
fn test_single_letter_loop_counter_ok() {
    let file = parse_rust(
        r#"
fn main() {
    for i in 0..10 {
        println!("{}", i);
    }
    let _ = 42;
}
"#,
    );
    let rule = SingleLetterTsRule;
    let issues = rule.check(&file);
    assert!(
        issues.is_empty(),
        "Loop counters (i) and throwaway (_) should be OK"
    );
}

#[test]
fn test_abbreviation_abuse_detected() {
    let file = parse_rust(
        r#"
fn main() {
    let cfg = 1;
    let mgr = 2;
    let srv = 3;
    let ctx = 4;
}
"#,
    );
    let rule = AbbreviationAbuseTsRule;
    let issues = rule.check(&file);
    assert!(!issues.is_empty(), "Should detect abbreviation abuse");
}

#[test]
fn test_abbreviation_abuse_normal_ok() {
    let file = parse_rust(
        r#"
fn main() {
    let config = 1;
    let manager = 2;
    let server = 3;
}
"#,
    );
    let rule = AbbreviationAbuseTsRule;
    let issues = rule.check(&file);
    assert!(issues.is_empty(), "Full words should not trigger");
}

#[test]
fn test_complex_closure_detected() {
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
    assert!(!issues.is_empty(), "Should detect deeply nested closures");
}

#[test]
fn test_god_function_detected() {
    // God function score = line_count + params + control_flow
    // 11+ control flow nodes (if/for/while) → score > 10
    let file = parse_rust(
        r#"
fn god(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) {
    if a > 0 { println!("a"); }
    if b > 0 { println!("b"); }
    if c > 0 { println!("c"); }
    if d > 0 { println!("d"); }
    if e > 0 { println!("e"); }
    if f > 0 { println!("f"); }
    if g > 0 { println!("g"); }
    for i in 0..10 { println!("{}", i); }
    while a > 0 { break; }
}
"#,
    );
    let rule = GodFunctionRule;
    let issues = rule.check(&file);
    assert!(
        !issues.is_empty(),
        "Should detect god function (7 params + 9 control flow nodes)"
    );
}
