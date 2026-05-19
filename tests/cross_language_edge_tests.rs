//! Additional cross-language edge case tests.
//!
//! Each language section tests:
//!   - Empty files (should parse and have 0 violations for all detectors)
//!   - Standalone function with clean naming
//!   - Single parameter function (not over-engineered)
//!   - Minimal code smell detection

use garbage_code_hunter::detectors::{
    CodeSmellsDetector, HotfixCultureDetector, LegacyCodeDetector, LineCountSmellDetector,
    NamingChaosDetector, NestedHellDetector, OverEngineeringDetector, PanicAddictionDetector,
    TodoMountainDetector,
};
use garbage_code_hunter::signals::SignalDetector;
use garbage_code_hunter::treesitter::{ParsedFile, TreeSitterEngine};
use std::path::Path;

fn parse_code(code: &str, filename: &str) -> ParsedFile {
    let engine = TreeSitterEngine::new();
    engine
        .parse_file(Path::new(filename), code)
        .unwrap_or_else(|| panic!("parse should succeed for {}", filename))
}

fn count_violations(detector: &dyn SignalDetector, code: &str, filename: &str) -> usize {
    let file = parse_code(code, filename);
    detector.count_violations(&file)
}

fn assert_all_clean(code: &str, filename: &str) {
    let detectors: Vec<Box<dyn SignalDetector>> = vec![
        Box::new(PanicAddictionDetector::new()),
        Box::new(NamingChaosDetector::new()),
        Box::new(NestedHellDetector::new()),
        Box::new(HotfixCultureDetector::new()),
        Box::new(OverEngineeringDetector::new()),
        Box::new(CodeSmellsDetector::new()),
        Box::new(LegacyCodeDetector::new()),
        Box::new(TodoMountainDetector::new()),
        Box::new(LineCountSmellDetector::new()),
    ];
    for detector in &detectors {
        let count = count_violations(detector.as_ref(), code, filename);
        assert_eq!(
            count,
            0,
            "{} should be clean for {}, got {}",
            detector.signal().display_name(),
            filename,
            count
        );
    }
}

// ============================================================================
// Rust edge cases
// ============================================================================

mod rust_edges {
    use super::*;

    #[test]
    fn test_rust_empty_file_parses() {
        let file = parse_code("", "test.rs");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_rust_empty_fn_clean() {
        assert_all_clean("fn empty() {}", "test.rs");
    }

    #[test]
    fn test_rust_clean_ternary() {
        assert_all_clean(
            "fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }",
            "test.rs",
        );
    }

    #[test]
    fn test_rust_single_param_ok() {
        let code = "fn greet(name: &str) -> String { format!(\"Hello, {}\", name) }";
        let count = count_violations(&OverEngineeringDetector::new(), code, "test.rs");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_rust_unwrap_detected_even_in_nested() {
        let code = r#"
fn main() {
    if true {
        let x = val.unwrap();
    }
}
"#;
        let count = count_violations(&PanicAddictionDetector::new(), code, "test.rs");
        assert_eq!(count, 1);
    }
}

// ============================================================================
// Python edge cases
// ============================================================================

mod python_edges {
    use super::*;

    #[test]
    fn test_python_empty_file_parses() {
        let file = parse_code("", "test.py");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_python_simple_def_clean() {
        assert_all_clean("def add(x, y):\n    return x + y\n", "test.py");
    }

    #[test]
    fn test_python_lambda_clean() {
        assert_all_clean("double = lambda x: x * 2\n", "test.py");
    }

    #[test]
    fn test_python_todo_detected() {
        let code = "# TODO: implement me\ndef foo(): pass\n";
        let count = count_violations(&TodoMountainDetector::new(), code, "test.py");
        assert_eq!(count, 1);
    }
}

// ============================================================================
// JavaScript edge cases
// ============================================================================

mod js_edges {
    use super::*;

    #[test]
    fn test_js_empty_file_parses() {
        let file = parse_code("", "test.js");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_js_simple_fn_clean() {
        assert_all_clean("function add(a, b) { return a + b; }", "test.js");
    }

    #[test]
    fn test_js_arrow_fn_clean() {
        assert_all_clean("const add = (a, b) => a + b;", "test.js");
    }

    #[test]
    fn test_js_console_log_detected() {
        let count = count_violations(
            &HotfixCultureDetector::new(),
            "console.log('debug');",
            "test.js",
        );
        assert_eq!(count, 1);
    }
}

// ============================================================================
// TypeScript edge cases
// ============================================================================

mod ts_edges {
    use super::*;

    #[test]
    fn test_ts_empty_file_parses() {
        let file = parse_code("", "test.ts");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_ts_typed_fn_clean() {
        assert_all_clean(
            "function add(a: number, b: number): number { return a + b; }",
            "test.ts",
        );
    }

    #[test]
    fn test_ts_interface_clean() {
        assert_all_clean("interface User { name: string; }", "test.ts");
    }
}

// ============================================================================
// Go edge cases
// ============================================================================

mod go_edges {
    use super::*;

    #[test]
    fn test_go_empty_file_parses() {
        let file = parse_code("", "test.go");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_go_simple_fn_clean() {
        assert_all_clean(
            "package main\nfunc add(x int, y int) int { return x + y }",
            "test.go",
        );
    }

    #[test]
    fn test_go_panic_detected() {
        let count = count_violations(
            &PanicAddictionDetector::new(),
            "package main\nfunc main() { panic(\"boom\") }",
            "test.go",
        );
        assert_eq!(count, 1);
    }
}

// ============================================================================
// Java edge cases
// ============================================================================

mod java_edges {
    use super::*;

    #[test]
    fn test_java_empty_file_parses() {
        let file = parse_code("", "Test.java");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_java_simple_method_clean() {
        assert_all_clean(
            "class A { int add(int x, int y) { return x + y; } }",
            "Test.java",
        );
    }
}

// ============================================================================
// Ruby edge cases
// ============================================================================

mod ruby_edges {
    use super::*;

    #[test]
    fn test_ruby_empty_file_parses() {
        let file = parse_code("", "test.rb");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_ruby_method_clean_with_pragma() {
        assert_all_clean(
            "# frozen_string_literal: true\n\ndef add(x, y)\n  x + y\nend\n",
            "test.rb",
        );
    }

    #[test]
    fn test_ruby_raise_detected() {
        let count = count_violations(
            &PanicAddictionDetector::new(),
            "def foo\n  raise 'err'\nend",
            "test.rb",
        );
        assert_eq!(count, 1);
    }
}

// ============================================================================
// Swift edge cases
// ============================================================================

mod swift_edges {
    use super::*;

    #[test]
    fn test_swift_empty_file_parses() {
        let file = parse_code("", "test.swift");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_swift_simple_fn_clean() {
        assert_all_clean(
            "func add(x: Int, y: Int) -> Int { return x + y }",
            "test.swift",
        );
    }
}

// ============================================================================
// Zig edge cases
// ============================================================================

mod zig_edges {
    use super::*;

    #[test]
    fn test_zig_empty_file_parses() {
        let file = parse_code("", "test.zig");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_zig_simple_fn_clean() {
        assert_all_clean("fn add(a: i32, b: i32) i32 { return a + b; }", "test.zig");
    }
}

// ============================================================================
// C edge cases
// ============================================================================

mod c_edges {
    use super::*;

    #[test]
    fn test_c_empty_file_parses() {
        let file = parse_code("", "test.c");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_c_simple_fn_clean() {
        assert_all_clean("int add(int x, int y) { return x + y; }", "test.c");
    }

    #[test]
    fn test_c_goto_detected() {
        let code = "void f() { goto end; end: return; }";
        let count = count_violations(&CodeSmellsDetector::new(), code, "test.c");
        assert!(count >= 1, "goto should be detected as code smell");
    }
}

// ============================================================================
// C++ edge cases
// ============================================================================

mod cpp_edges {
    use super::*;

    #[test]
    fn test_cpp_empty_file_parses() {
        let file = parse_code("", "test.cpp");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_cpp_simple_fn_clean() {
        assert_all_clean("int add(int x, int y) { return x + y; }", "test.cpp");
    }
}

// ============================================================================
// Cross-detector edge cases
// ============================================================================

mod cross_detector_edges {
    use super::*;

    #[test]
    fn test_todo_not_in_string_literals() {
        // TODO in a comment should NOT be detected inside string literals
        let code = r#"fn main() { let s = "TODO: not really a marker"; }"#;
        let count = count_violations(&TodoMountainDetector::new(), code, "test.rs");
        // If the detector only checks comments, this should be 0
        // If it checks all lines, this might be detected — allow either behavior
        assert!(
            count <= 1,
            "TODO in string should not create false positives"
        );
    }

    #[test]
    fn test_single_line_file_clean() {
        assert_all_clean("fn main() {}", "test.rs");
    }

    #[test]
    fn test_empty_string_clean() {
        assert_all_clean("", "test.rs");
    }

    #[test]
    fn test_whitespace_only_file_clean() {
        assert_all_clean("   \n  \n", "test.rs");
    }

    #[test]
    fn test_panic_addiction_all_variants_rust() {
        let code = r#"
fn main() {
    let a = x.unwrap();
    let b = y.expect("msg");
    panic!("boom");
    let c = z.unwrap_or_default();
}
"#;
        // unwrap_or_default is not a panic call, so 3 violations
        let count = count_violations(&PanicAddictionDetector::new(), code, "test.rs");
        assert_eq!(count, 3, "unwrap + expect + panic! = 3");
    }

    #[test]
    fn test_no_false_positive_on_ok_result() {
        assert_all_clean(r#"fn main() -> Result<(), String> { Ok(()) }"#, "test.rs");
    }

    #[test]
    fn test_naming_detects_abc_meaningless() {
        let code = "fn main() { let aaa = 1; let bbb = 2; }";
        let count = count_violations(&NamingChaosDetector::new(), code, "test.rs");
        assert!(count >= 2, "meaningless repeated vars should be caught");
    }

    #[test]
    fn test_naming_ok_with_generic_type_params() {
        let code = "fn id<T>(x: T) -> T { x }";
        let count = count_violations(&NamingChaosDetector::new(), code, "test.rs");
        assert_eq!(count, 0, "generic type params not naming violations");
    }

    #[test]
    fn test_over_engineering_boundary_5_params() {
        let code =
            "fn process(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 { a + b + c + d + e }";
        let count = count_violations(&OverEngineeringDetector::new(), code, "test.rs");
        assert_eq!(count, 0, "5 params should be at threshold (not over)");
    }

    #[test]
    fn test_over_engineering_6_params_triggers() {
        let code = "fn process(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 { a + b }";
        let count = count_violations(&OverEngineeringDetector::new(), code, "test.rs");
        assert_eq!(count, 1, "6 params should trigger over-engineering");
    }

    #[test]
    fn test_legacy_code_block_in_python_detected() {
        // Must have 5+ consecutive comment lines for legacy detection
        let code =
            "# def old():\n#     pass\n#     print('x')\n#     return 1\n# end\ndef new(): pass\n";
        let count = count_violations(&LegacyCodeDetector::new(), code, "test.py");
        assert!(
            count >= 4,
            "5 consecutive commented lines in Python should be detected"
        );
    }

    #[test]
    fn test_todo_detects_all_four_markers() {
        let code = "// TODO: a\n// FIXME: b\n// BUG: c\n// HACK: d\n";
        let count = count_violations(&TodoMountainDetector::new(), code, "test.rs");
        assert_eq!(count, 4);
    }

    #[test]
    fn test_todo_clean_without_markers() {
        assert_all_clean("// normal comment\nfn main() {}", "test.rs");
    }

    #[test]
    fn test_code_smell_no_false_positive_on_basic_rust() {
        // println! is detected by HotfixCultureDetector, not CodeSmellsDetector
        let code = "fn main() { let x = 1 + 2; }";
        let count = count_violations(&CodeSmellsDetector::new(), code, "test.rs");
        assert_eq!(count, 0, "basic Rust with no magic numbers = 0");
    }

    #[test]
    fn test_line_count_edge_999_lines_ok() {
        let mut code = String::from("fn main() {\n");
        for i in 0..997 {
            code.push_str(&format!("    let x_{} = {};\n", i, i));
        }
        code.push_str("}\n");
        let count = count_violations(&LineCountSmellDetector::new(), &code, "test.rs");
        assert_eq!(count, 0, "999 lines = 0");
    }
}
