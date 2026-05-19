//! Cross-language end-to-end tests — verify each detector works on real code snippets
//! for all 11 supported languages: Rust, Python, JavaScript, TypeScript, Go, Java,
//! Ruby, Swift, Zig, C, C++.
//!
//! Each language section has:
//!   - A "garbage" code snippet with multiple signal violations
//!   - Individual detector tests against the garbage snippet
//!   - A "clean" code snippet that should produce 0 violations
//!   - Language-specific signal tests (goroutines, wildcard imports, etc.)

use garbage_code_hunter::detectors::{
    CodeSmellsDetector, DuplicationDetector, HotfixCultureDetector, LegacyCodeDetector,
    LineCountSmellDetector, NamingChaosDetector, NestedHellDetector, OverEngineeringDetector,
    PanicAddictionDetector, TodoMountainDetector,
};
use garbage_code_hunter::signals::SignalDetector;
use garbage_code_hunter::treesitter::{ParsedFile, TreeSitterEngine};
use std::path::Path;

// ── Helpers ─────────────────────────────────────────────────────────────────

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

fn assert_clean(detector: &dyn SignalDetector, code: &str, filename: &str) {
    let count = count_violations(detector, code, filename);
    assert_eq!(
        count,
        0,
        "{} should be clean for {}\n  got {}",
        detector.signal().display_name(),
        filename,
        count
    );
}

// ============================================================================
// Rust
// ============================================================================

mod rust_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
fn process(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    let x = std::env::var("HOME").unwrap();
    let y = some_result.expect("must work");
    let data = 42;

    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            unsafe {
                                let ptr = 100 as *const i32;
                                let _ = *ptr;
                            }
                            panic!("unexpected state");
                        }
                    }
                }
            }
        }
    }

    println!("data = {}", data);
    dbg!(y);
    todo!("implement error handling");

    data + x.len() as i32
}

fn helper() -> i32 {
    let data = 1;
    let manager = 2;
    let strName = "hello";
    let g_count = 0;
    let mgr = "boss";
    data + manager
}
"#
    }

    fn clean_code() -> &'static str {
        r#"
fn add_user_score(user_name: &str, base_score: i32) -> Result<i32, String> {
    if user_name.is_empty() {
        return Err("user name cannot be empty".to_string());
    }
    let bonus = if user_name.len() > 5 { 10 } else { 5 };
    Ok(base_score + bonus)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let score = add_user_score("Alice", 100)?;
    Ok(())
}
"#
    }

    #[test]
    fn test_rust_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "test.rs");
        assert!(
            count >= 3,
            "Rust panic: unwrap + expect + panic! = >=3, got {count}"
        );
    }

    #[test]
    fn test_rust_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "test.rs");
    }

    #[test]
    fn test_rust_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.rs");
        assert!(
            count >= 4,
            "Rust naming: data, manager, strName, g_count, mgr = >=4, got {count}"
        );
    }

    #[test]
    fn test_rust_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.rs");
    }

    #[test]
    fn test_rust_nested_hell() {
        let count = count_violations(&NestedHellDetector::new(), garbage_code(), "test.rs");
        assert!(
            count >= 1,
            "Rust nested: 6-level if nesting >= 1, got {count}"
        );
    }

    #[test]
    fn test_rust_nested_clean() {
        assert_clean(&NestedHellDetector::new(), clean_code(), "test.rs");
    }

    #[test]
    fn test_rust_hotfix_culture() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.rs");
        assert!(
            count >= 3,
            "Rust hotfix: println + dbg + todo! = >=3, got {count}"
        );
    }

    #[test]
    fn test_rust_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.rs");
    }

    #[test]
    fn test_rust_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.rs");
        assert!(
            count >= 1,
            "Rust over-eng: process with 6 params >= 1, got {count}"
        );
    }

    #[test]
    fn test_rust_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.rs");
    }

    #[test]
    fn test_rust_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.rs");
        assert!(
            count >= 2,
            "Rust code smells: at least 2 (unsafe block + magic numbers), got {count}"
        );
    }

    #[test]
    fn test_rust_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.rs");
    }
}

// ============================================================================
// Python
// ============================================================================

mod python_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
try:
    do_something()
except:
    pass

try:
    risky()
except BaseException:
    pass

def getData():
    a = 1
    b = 2
    manager = "boss"
    data = 42
    print(f"data={data}")
    print("debug")

def process(a, b, c, d, e, f):
    foo(42)
    bar(100)
    return a + b + c + d + e + f

from os import *
"#
    }

    fn clean_code() -> &'static str {
        r#"
def greet_user(user_name):
    if not user_name:
        return ""
    return "Hello, " + user_name
"#
    }

    #[test]
    fn test_python_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "test.py");
        assert!(
            count >= 2,
            "Python panic: bare except + BaseException = >=2, got {count}"
        );
    }

    #[test]
    fn test_python_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "test.py");
    }

    #[test]
    fn test_python_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.py");
        assert!(
            count >= 4,
            "Python naming: a+b+manager+data+camelCase = >=4, got {count}"
        );
    }

    #[test]
    fn test_python_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.py");
    }

    #[test]
    fn test_python_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.py");
        assert!(count >= 2, "Python hotfix: 2x print = >=2, got {count}");
    }

    #[test]
    fn test_python_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.py");
    }

    #[test]
    fn test_python_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.py");
        assert!(
            count >= 1,
            "Python over-eng: process with 6 params >= 1, got {count}"
        );
    }

    #[test]
    fn test_python_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.py");
    }

    #[test]
    fn test_python_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.py");
        assert!(
            count >= 3,
            "Python code smells: 42 + 100 + wildcard import = >=3, got {count}"
        );
    }

    #[test]
    fn test_python_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.py");
    }

    #[test]
    fn test_python_nested_hell() {
        let count = count_violations(&NestedHellDetector::new(), garbage_code(), "test.py");
        assert_eq!(count, 0, "flat Python code = 0");
    }
}

// ============================================================================
// JavaScript
// ============================================================================

mod js_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
function main() {
    throw new Error("boom");
    let a = 1;
    let b = 2;
    let data = 42;
    if (true) {
        if (true) {
            if (true) {
                if (true) {
                    if (true) {
                        if (true) {
                            foo(100);
                        }
                    }
                }
            }
        }
    }
    console.log("data =", data);
    console.error("error");
    debugger;
}

function process(a, b, c, d, e, f) {
    foo(41);
    bar(100);
    return a + b + c + d + e + f;
}
"#
    }

    fn clean_code() -> &'static str {
        r#"
function greetUser(userName) {
    if (!userName) {
        return "";
    }
    return "Hello, " + userName;
}
"#
    }

    #[test]
    fn test_js_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "test.js");
        assert!(count >= 1, "JS panic: throw = >=1, got {count}");
    }

    #[test]
    fn test_js_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "test.js");
    }

    #[test]
    fn test_js_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.js");
        assert!(count >= 2, "JS naming: a + b + data = >=2, got {count}");
    }

    #[test]
    fn test_js_naming_clean() {
        let count = count_violations(&NamingChaosDetector::new(), clean_code(), "test.js");
        assert_eq!(count, 0, "JS clean naming = 0, got {count}");
    }

    #[test]
    fn test_js_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.js");
        assert!(
            count >= 3,
            "JS hotfix: console.log + console.error + debugger = >=3, got {count}"
        );
    }

    #[test]
    fn test_js_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.js");
    }

    #[test]
    fn test_js_nested_hell() {
        let count = count_violations(&NestedHellDetector::new(), garbage_code(), "test.js");
        assert_eq!(
            count, 0,
            "JS nesting: not supported via StyleIR, got {count}"
        );
    }

    #[test]
    fn test_js_nested_clean() {
        assert_clean(&NestedHellDetector::new(), clean_code(), "test.js");
    }

    #[test]
    fn test_js_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.js");
        assert!(
            count >= 1,
            "JS over-eng: process with 6 params >= 1, got {count}"
        );
    }

    #[test]
    fn test_js_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.js");
    }

    #[test]
    fn test_js_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.js");
        assert!(
            count >= 3,
            "JS code smells: 42 + 100 + 41 = >=3, got {count}"
        );
    }

    #[test]
    fn test_js_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.js");
    }
}

// ============================================================================
// TypeScript
// ============================================================================

mod ts_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
function main(): void {
    throw new Error("boom");
    throw "bang";
    let a = 1;
    let b = 2;
    let data = 42;
    console.log("data =", data);
    console.error("error");
    debugger;
    foo(100);
}

function process(a: number, b: number, c: number, d: number, e: number, f: number): void {
    foo(41);
    return;
}

let x: any = 42;
enum Color { Red, Green, Blue }
"#
    }

    fn clean_code() -> &'static str {
        r#"
function greetUser(userName: string): string {
    if (!userName) {
        return "";
    }
    return "Hello, " + userName;
}

interface User {
    name: string;
}
"#
    }

    #[test]
    fn test_ts_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "test.ts");
        assert!(count >= 2, "TS panic: 2x throw = >=2, got {count}");
    }

    #[test]
    fn test_ts_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "test.ts");
    }

    #[test]
    fn test_ts_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.ts");
        assert!(count >= 2, "TS naming: a + b + data = >=2, got {count}");
    }

    #[test]
    fn test_ts_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.ts");
    }

    #[test]
    fn test_ts_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.ts");
        assert!(
            count >= 3,
            "TS hotfix: console.log + console.error + debugger = >=3, got {count}"
        );
    }

    #[test]
    fn test_ts_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.ts");
    }

    #[test]
    fn test_ts_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.ts");
        assert!(
            count >= 3,
            "TS code smells: magic 42+100+41 + any-type + enum = >=3, got {count}"
        );
    }

    #[test]
    fn test_ts_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.ts");
    }

    #[test]
    fn test_ts_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.ts");
        assert!(
            count >= 1,
            "TS over-eng: process with 6 params >= 1, got {count}"
        );
    }

    #[test]
    fn test_ts_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.ts");
    }
}

// ============================================================================
// Go
// ============================================================================

mod go_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
package main

import "fmt"

func process(a int, b int, c int, d int, e int, f int) {
    panic("unexpected")
    x := 1
    y := 2
    data := 42
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            foo(100)
                        }
                    }
                }
            }
        }
    }
    fmt.Println("data =", data)
    fmt.Printf("x=%d", x)

    go doWork()
    go logResult()

    for i := 0; i < 10; i++ {
        defer cleanup(i)
    }
}

func helper() {
    data := 1
    manager := 2
}
"#
    }

    fn clean_code() -> &'static str {
        r#"
package main

// clean: no magic numbers, no debug prints, no panics
func greetUser(userName string) string {
    if userName == "" {
        return ""
    }
    return "Hello, " + userName
}
"#
    }

    #[test]
    fn test_go_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "test.go");
        assert!(count >= 1, "Go panic: panic() = >=1, got {count}");
    }

    #[test]
    fn test_go_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "test.go");
    }

    #[test]
    fn test_go_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.go");
        assert!(
            count >= 3,
            "Go naming: x + y + data + manager = >=3, got {count}"
        );
    }

    #[test]
    fn test_go_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.go");
    }

    #[test]
    fn test_go_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.go");
        assert!(
            count >= 2,
            "Go hotfix: 2x fmt.Println/Printf = >=2, got {count}"
        );
    }

    #[test]
    fn test_go_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.go");
    }

    #[test]
    fn test_go_nested_hell() {
        let count = count_violations(&NestedHellDetector::new(), garbage_code(), "test.go");
        assert!(count >= 1, "Go nested: 6-level nesting >= 1, got {count}");
    }

    #[test]
    fn test_go_nested_clean() {
        assert_clean(&NestedHellDetector::new(), clean_code(), "test.go");
    }

    #[test]
    fn test_go_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.go");
        assert!(count >= 1, "Go over-eng: 6 params >= 1, got {count}");
    }

    #[test]
    fn test_go_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.go");
    }

    #[test]
    fn test_go_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.go");
        assert!(
            count >= 4,
            "Go code smells: magic 42+100 + goroutine*2 + defer*1 = >=4, got {count}"
        );
    }

    #[test]
    fn test_go_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.go");
    }
}

// ============================================================================
// Java
// ============================================================================

mod java_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
class Main {
    void main() {
        throw new RuntimeException("boom");
        throw new Exception("bang");
        int a = 1;
        int b = 2;
        int data = 42;
        if (true) {
            if (true) {
                if (true) {
                    if (true) {
                        if (true) {
                            if (true) {
                                foo(100);
                            }
                        }
                    }
                }
            }
        }
        System.out.println("data = " + data);
        System.err.println("error");
        e.printStackTrace();
    }

    void process(int a, int b, int c, int d, int e, int f) {
        foo(41);
        bar(100);
    }
}
"#
    }

    fn clean_code() -> &'static str {
        r#"
class Calculator {
    // clean: no magic numbers, no debug prints, no panics
    int add(int x, int y) {
        return x + y;
    }
}
"#
    }

    #[test]
    fn test_java_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "Test.java");
        assert!(count >= 2, "Java panic: 2x throw = >=2, got {count}");
    }

    #[test]
    fn test_java_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "Test.java");
    }

    #[test]
    fn test_java_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "Test.java");
        assert!(count >= 2, "Java naming: a + b + data = >=2, got {count}");
    }

    #[test]
    fn test_java_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "Test.java");
    }

    #[test]
    fn test_java_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "Test.java");
        assert!(count >= 3, "Java hotfix: System.out.println + System.err.println + printStackTrace = >=3, got {count}");
    }

    #[test]
    fn test_java_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "Test.java");
    }

    #[test]
    fn test_java_nested_hell() {
        let count = count_violations(&NestedHellDetector::new(), garbage_code(), "Test.java");
        assert!(count >= 1, "Java nested: 6-level nesting >= 1, got {count}");
    }

    #[test]
    fn test_java_nested_clean() {
        assert_clean(&NestedHellDetector::new(), clean_code(), "Test.java");
    }

    #[test]
    fn test_java_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "Test.java");
        assert!(count >= 1, "Java over-eng: 6 params >= 1, got {count}");
    }

    #[test]
    fn test_java_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "Test.java");
    }

    #[test]
    fn test_java_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "Test.java");
        assert!(
            count >= 3,
            "Java code smells: magic 42 + 100 + 41 = >=3, got {count}"
        );
    }

    #[test]
    fn test_java_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "Test.java");
    }
}

// ============================================================================
// Ruby
// ============================================================================

mod ruby_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
def process(a, b, c, d, e, f)
  raise "error"
  a = 1
  b = 2
  data = 42
  manager = "boss"
  puts "data=#{data}"
  binding.pry
  byebug
  foo(100)
  bar(41)
end

$global_var = "bad"
"#
    }

    fn clean_code() -> &'static str {
        r#"# frozen_string_literal: true

def foo
end
"#
    }

    #[test]
    fn test_ruby_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "test.rb");
        assert!(count >= 1, "Ruby panic: raise = >=1, got {count}");
    }

    #[test]
    fn test_ruby_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "test.rb");
    }

    #[test]
    fn test_ruby_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.rb");
        assert!(
            count >= 3,
            "Ruby naming: a + b + data + manager = >=3, got {count}"
        );
    }

    #[test]
    fn test_ruby_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.rb");
    }

    #[test]
    fn test_ruby_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.rb");
        assert!(
            count >= 2,
            "Ruby hotfix: puts + binding.pry + byebug = >=2, got {count}"
        );
    }

    #[test]
    fn test_ruby_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.rb");
    }

    #[test]
    fn test_ruby_nested_hell() {
        let count = count_violations(&NestedHellDetector::new(), garbage_code(), "test.rb");
        assert_eq!(count, 0, "Ruby nested: not supported, got {count}");
    }

    #[test]
    fn test_ruby_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.rb");
        assert!(count >= 1, "Ruby over-eng: 6 params >= 1, got {count}");
    }

    #[test]
    fn test_ruby_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.rb");
    }

    #[test]
    fn test_ruby_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.rb");
        assert!(
            count >= 3,
            "Ruby code smells: 100 + 41 + global_var = >=3, got {count}"
        );
    }

    #[test]
    fn test_ruby_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.rb");
    }
}

// ============================================================================
// Swift
// ============================================================================

mod swift_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
func main() {
    fatalError("boom")
    preconditionFailure("bad")
    let a = 1
    let b = 2
    let data = 42
    print("data = \(data)")
    debugPrint("debug")
    dump(obj)
    foo(100)
    bar(41)
}

func process(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int) -> Int {
    return a + b + c + d + e + f
}
"#
    }

    fn clean_code() -> &'static str {
        r#"
func greetUser(userName: String) -> String {
    guard !userName.isEmpty else { return "" }
    return "Hello, " + userName
}
"#
    }

    #[test]
    fn test_swift_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "test.swift");
        assert!(
            count >= 2,
            "Swift panic: fatalError + preconditionFailure = >=2, got {count}"
        );
    }

    #[test]
    fn test_swift_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "test.swift");
    }

    #[test]
    fn test_swift_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.swift");
        assert!(count >= 2, "Swift naming: a + b + data = >=2, got {count}");
    }

    #[test]
    fn test_swift_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.swift");
    }

    #[test]
    fn test_swift_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.swift");
        assert!(
            count >= 3,
            "Swift hotfix: print + debugPrint + dump = >=3, got {count}"
        );
    }

    #[test]
    fn test_swift_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.swift");
    }

    #[test]
    fn test_swift_over_engineering() {
        let count = count_violations(
            &OverEngineeringDetector::new(),
            garbage_code(),
            "test.swift",
        );
        assert!(count >= 1, "Swift over-eng: 6 params >= 1, got {count}");
    }

    #[test]
    fn test_swift_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.swift");
    }

    #[test]
    fn test_swift_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.swift");
        assert!(
            count >= 3,
            "Swift code smells: 100 + 41 + 42 = >=3, got {count}"
        );
    }

    #[test]
    fn test_swift_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.swift");
    }
}

// ============================================================================
// Zig
// ============================================================================

mod zig_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
const std = @import("std");

fn main() void {
    @panic("boom");
    @panic("bang");
    const a: i32 = 1;
    const b: i32 = 2;
    const data: i32 = 42;
    std.debug.print("data = {}\n", .{data});
    foo(100);
    bar(41);
}

fn process(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) void {
    _ = a + b + c + d + e + f;
}
"#
    }

    fn clean_code() -> &'static str {
        r#"
const std = @import("std");

fn calculateScore(userName: []const u8, baseScore: i32) i32 {
    if (userName.len == 0) return 0;
    const bonus: i32 = if (userName.len > 5) 10 else 5;
    return baseScore + bonus;
}
"#
    }

    #[test]
    fn test_zig_panic_detection() {
        let count = count_violations(&PanicAddictionDetector::new(), garbage_code(), "test.zig");
        assert!(count >= 2, "Zig panic: 2x @panic = >=2, got {count}");
    }

    #[test]
    fn test_zig_panic_clean() {
        assert_clean(&PanicAddictionDetector::new(), clean_code(), "test.zig");
    }

    #[test]
    fn test_zig_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.zig");
        assert!(count >= 2, "Zig naming: a + b + data = >=2, got {count}");
    }

    #[test]
    fn test_zig_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.zig");
    }

    #[test]
    fn test_zig_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.zig");
        assert!(count >= 1, "Zig hotfix: std.debug.print = >=1, got {count}");
    }

    #[test]
    fn test_zig_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.zig");
    }

    #[test]
    fn test_zig_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.zig");
        assert!(count >= 1, "Zig over-eng: 6 params >= 1, got {count}");
    }

    #[test]
    fn test_zig_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.zig");
    }

    #[test]
    fn test_zig_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.zig");
        assert!(
            count >= 2,
            "Zig code smells: 100 + 41 + 42 = >=2, got {count}"
        );
    }

    #[test]
    fn test_zig_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.zig");
    }
}

// ============================================================================
// C
// ============================================================================

mod c_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
int process(int a, int b, int c, int d, int e, int f) {
    int x = 1;
    int y = 2;
    int data = 42;
    if (1) {
        if (1) {
            if (1) {
                if (1) {
                    if (1) {
                        if (1) {
                            foo(100);
                        }
                    }
                }
            }
        }
    }
    printf("data = %d", data);
    fprintf(stderr, "error");
    goto cleanup;
    foo(41);
cleanup:
    return a + b + c + d + e + f;
}
"#
    }

    fn clean_code() -> &'static str {
        r#"
// clean: no magic numbers, no debug prints, no panics
void greetUser(const char* userName) {
    if (userName == (void*)0) {
        return;
    }
}
"#
    }

    #[test]
    fn test_c_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.c");
        assert!(count >= 2, "C naming: x + y + data = >=2, got {count}");
    }

    #[test]
    fn test_c_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.c");
    }

    #[test]
    fn test_c_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.c");
        assert!(count >= 2, "C hotfix: printf + fprintf = >=2, got {count}");
    }

    #[test]
    fn test_c_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.c");
    }

    #[test]
    fn test_c_nested_hell() {
        let count = count_violations(&NestedHellDetector::new(), garbage_code(), "test.c");
        assert!(count >= 1, "C nested: 6-level nesting >= 1, got {count}");
    }

    #[test]
    fn test_c_nested_clean() {
        assert_clean(&NestedHellDetector::new(), clean_code(), "test.c");
    }

    #[test]
    fn test_c_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.c");
        assert!(count >= 1, "C over-eng: 6 params >= 1, got {count}");
    }

    #[test]
    fn test_c_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.c");
    }

    #[test]
    fn test_c_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.c");
        assert!(
            count >= 3,
            "C code smells: 100 + 42 + 41 + goto = >=3, got {count}"
        );
    }

    #[test]
    fn test_c_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.c");
    }
}

// ============================================================================
// C++
// ============================================================================

mod cpp_tests {
    use super::*;

    fn garbage_code() -> &'static str {
        r#"
int process(int a, int b, int c, int d, int e, int f) {
    int x = 1;
    int y = 2;
    int data = 42;
    auto* obj = new int(100);
    if (1) {
        if (1) {
            if (1) {
                if (1) {
                    if (1) {
                        if (1) {
                            foo(41);
                        }
                    }
                }
            }
        }
    }
    printf("data = %d", data);
    fprintf(stderr, "error");
    goto cleanup;
cleanup:
    return a + b + c + d + e + f;
}
"#
    }

    fn clean_code() -> &'static str {
        r#"
// clean: no magic numbers, no debug prints, no panics
void greetUser(const std::string& userName) {
    if (userName.empty()) {
        return;
    }
}
"#
    }

    #[test]
    fn test_cpp_naming_detection() {
        let count = count_violations(&NamingChaosDetector::new(), garbage_code(), "test.cpp");
        assert!(count >= 2, "C++ naming: x + y + data = >=2, got {count}");
    }

    #[test]
    fn test_cpp_naming_clean() {
        assert_clean(&NamingChaosDetector::new(), clean_code(), "test.cpp");
    }

    #[test]
    fn test_cpp_hotfix_detection() {
        let count = count_violations(&HotfixCultureDetector::new(), garbage_code(), "test.cpp");
        assert!(
            count >= 2,
            "C++ hotfix: printf + fprintf = >=2, got {count}"
        );
    }

    #[test]
    fn test_cpp_hotfix_clean() {
        assert_clean(&HotfixCultureDetector::new(), clean_code(), "test.cpp");
    }

    #[test]
    fn test_cpp_nested_hell() {
        let count = count_violations(&NestedHellDetector::new(), garbage_code(), "test.cpp");
        assert!(count >= 1, "C++ nested: 6-level nesting >= 1, got {count}");
    }

    #[test]
    fn test_cpp_nested_clean() {
        assert_clean(&NestedHellDetector::new(), clean_code(), "test.cpp");
    }

    #[test]
    fn test_cpp_over_engineering() {
        let count = count_violations(&OverEngineeringDetector::new(), garbage_code(), "test.cpp");
        assert!(count >= 1, "C++ over-eng: 6 params >= 1, got {count}");
    }

    #[test]
    fn test_cpp_over_engineering_clean() {
        assert_clean(&OverEngineeringDetector::new(), clean_code(), "test.cpp");
    }

    #[test]
    fn test_cpp_code_smells() {
        let count = count_violations(&CodeSmellsDetector::new(), garbage_code(), "test.cpp");
        assert!(
            count >= 4,
            "C++ code smells: 42 + 100 + 41 + new + goto = >=4, got {count}"
        );
    }

    #[test]
    fn test_cpp_code_smells_clean() {
        assert_clean(&CodeSmellsDetector::new(), clean_code(), "test.cpp");
    }
}

// ============================================================================
// LegacyCode / TodoMountain / LineCountSmell — Language-agnostic tests
// ============================================================================

mod universal_signals_tests {
    use super::*;

    // ── LegacyCodeDetector ──────────────────────────────────────

    #[test]
    fn test_legacy_code_commented_out_rust() {
        let code = r#"
fn main() {
    // let x = 1;
    // let y = 2;
    // let z = 3;
    // let w = x + y + z;
    // println!("{}", w);
    println!("hello");
}
"#;
        let count = count_violations(&LegacyCodeDetector::new(), code, "test.rs");
        assert!(
            count >= 5,
            "5 consecutive commented-out Rust lines should be detected, got {count}"
        );
    }

    #[test]
    fn test_legacy_code_no_false_positive_on_doc_comments() {
        let code = r#"
/// A documented function
fn documented() -> i32 {
    // normal comment
    let x = 1;
    x
}
"#;
        let count = count_violations(&LegacyCodeDetector::new(), code, "test.rs");
        assert_eq!(count, 0, "doc comments + normal comment = 0, got {count}");
    }

    #[test]
    fn test_legacy_code_commented_out_python() {
        let code = r#"
# def old_func():
#     print("hello")
#     return 42
# x = old_func()
def new_func():
    print("world")
"#;
        let count = count_violations(&LegacyCodeDetector::new(), code, "test.py");
        assert!(
            count >= 3,
            "4 commented-out Python lines should be >=3, got {count}"
        );
    }

    #[test]
    fn test_legacy_code_no_false_positive_short_comments() {
        let code = r#"
// This is a regular comment
// Another regular comment
fn main() {}
"#;
        let count = count_violations(&LegacyCodeDetector::new(), code, "test.rs");
        assert_eq!(count, 0, "short regular comments = 0, got {count}");
    }

    #[test]
    fn test_legacy_code_js_commented_code() {
        let code = r#"
// function oldFunc() {
//     return 42;
// }
// var x = oldFunc();
function newFunc() {
    return 10;
}
"#;
        let count = count_violations(&LegacyCodeDetector::new(), code, "test.js");
        assert!(
            count >= 3,
            "commented-out JS code lines should be >=3, got {count}"
        );
    }

    // ── TodoMountainDetector ────────────────────────────────────

    #[test]
    fn test_todo_mountain_basic() {
        let code = r#"
// TODO: refactor this
// FIXME: this is broken
// BUG: critical issue
// HACK: workaround for now
fn main() {}
"#;
        let count = count_violations(&TodoMountainDetector::new(), code, "test.rs");
        assert_eq!(count, 4, "TODO + FIXME + BUG + HACK = 4, got {count}");
    }

    #[test]
    fn test_todo_mountain_inline() {
        let code = r#"
fn main() {
    let x = 1; // TODO: use constant
    let y = 2; // FIXME: off-by-one
}
"#;
        let count = count_violations(&TodoMountainDetector::new(), code, "test.rs");
        assert_eq!(count, 2, "inline TODO + FIXME = 2, got {count}");
    }

    #[test]
    fn test_todo_mountain_python() {
        let code = r#"
# TODO: implement caching
# FIXME: race condition
def process():
    pass
"#;
        let count = count_violations(&TodoMountainDetector::new(), code, "test.py");
        assert_eq!(count, 2, "TODO + FIXME = 2, got {count}");
    }

    #[test]
    fn test_todo_mountain_clean() {
        let code = r#"
/// Completes registration for the given user.
fn finish(user: &str) -> String {
    format!("done: {}", user)
}
"#;
        let count = count_violations(&TodoMountainDetector::new(), code, "test.rs");
        assert_eq!(count, 0, "no TODO markers = 0, got {count}");
    }

    #[test]
    fn test_todo_mountain_case_insensitive() {
        let code = r#"
// todo: lowercase
// FIXME: uppercase
fn main() {}
"#;
        let count = count_violations(&TodoMountainDetector::new(), code, "test.rs");
        assert!(
            count >= 2,
            "case-insensitive: both todo + FIXME should be counted, got {count}"
        );
    }

    // ── LineCountSmellDetector ──────────────────────────────────

    fn long_code(num_lines: usize) -> String {
        let mut s = String::from("fn main() {\n");
        for i in 0..num_lines {
            s.push_str(&format!("    let line_{} = {};\n", i, i));
        }
        s.push_str("}\n");
        s
    }

    #[test]
    fn test_line_count_smell_normal_file() {
        let code = long_code(50);
        let count = count_violations(&LineCountSmellDetector::new(), &code, "test.rs");
        assert_eq!(
            count, 0,
            "50-line file should not trigger line-count smell, got {count}"
        );
    }

    #[test]
    fn test_line_count_smell_large_file() {
        let code = long_code(2000);
        let count = count_violations(&LineCountSmellDetector::new(), &code, "test.rs");
        assert!(
            count > 0,
            "2000-line file should trigger line-count smell, got {count}"
        );
    }

    #[test]
    fn test_line_count_smell_test_file_has_higher_threshold() {
        let code = long_code(1500);
        let count = count_violations(&LineCountSmellDetector::new(), &code, "test_suite.rs");
        // Test file threshold is 2000, so 1500 should not trigger
        assert_eq!(
            count, 0,
            "1500-line test file should not trigger line-count smell, got {count}"
        );
    }

    // ── DuplicationDetector ─────────────────────────────────────

    #[test]
    fn test_duplication_no_false_positive_for_unique_code() {
        let code = r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn sub(a: i32, b: i32) -> i32 { a - b }
fn mul(a: i32, b: i32) -> i32 { a * b }
"#;
        let count = count_violations(&DuplicationDetector::new(), code, "test.rs");
        assert_eq!(count, 0, "unique functions = 0, got {count}");
    }

    #[test]
    fn test_duplication_short_file_skipped() {
        let code = "fn main() { let x = 1; }";
        let count = count_violations(&DuplicationDetector::new(), code, "test.rs");
        assert_eq!(count, 0, "short file = 0, got {count}");
    }

    // ── NamingChaos: additional edge cases ──────────────────────

    #[test]
    fn test_naming_hungarian_notation_rust() {
        let code = r#"
fn main() {
    let strName = "hello";
    let intCount = 42;
    let arrList = vec![1, 2, 3];
}
"#;
        let count = count_violations(&NamingChaosDetector::new(), code, "test.rs");
        assert!(
            count >= 3,
            "strName + intCount + arrList = >=3, got {count}"
        );
    }

    #[test]
    fn test_naming_domain_prefixes_exempt() {
        let code = r#"
fn main() {
    let ctxUser = "admin";
    let dbQuery = "SELECT *";
    let kvStore = "redis";
    let apiKey = "abc123";
}
"#;
        let count = count_violations(&NamingChaosDetector::new(), code, "test.rs");
        assert_eq!(
            count, 0,
            "domain-prefixed vars should be exempt, got {count}"
        );
    }

    #[test]
    fn test_naming_abbreviation_abuse() {
        let code = r#"
fn main() {
    let mgr = create_manager();
    let btn = Button::new();
    let ctrl = Controller::new();
}
"#;
        let count = count_violations(&NamingChaosDetector::new(), code, "test.rs");
        assert!(count >= 3, "mgr + btn + ctrl = >=3, got {count}");
    }
}
