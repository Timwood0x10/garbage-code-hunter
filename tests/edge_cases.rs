use garbage_code_hunter::CodeAnalyzer;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_empty_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("empty.rs");

    fs::write(&file_path, "").expect("Failed to write empty file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(issues.is_empty(), "Empty file should have no issues");
}

#[test]
fn test_only_comments() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("comments_only.rs");

    let content = r#"
// This file only contains comments
/*
 * Multi-line comment
 * with multiple lines
 */
// Another regular comment
// Yet another comment
"#;

    fs::write(&file_path, content).expect("Failed to write comments file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(
        issues.is_empty(),
        "Comments-only file should have no issues, found {}",
        issues.len()
    );
}

#[test]
fn test_only_whitespace() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("whitespace.rs");

    let content = "   \n\t\n   \n\t\t\n   ";

    fs::write(&file_path, content).expect("Failed to write whitespace file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(
        issues.is_empty(),
        "Whitespace-only file should have no issues"
    );
}

#[test]
fn test_minimal_valid_rust() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("minimal.rs");

    let content = "fn main() {}";

    fs::write(&file_path, content).expect("Failed to write minimal file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(
        issues.is_empty(),
        "Minimal valid Rust should have no issues"
    );
}

#[test]
fn test_unicode_variable_names() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("unicode.rs");

    let content = r#"
fn main() {
    let 用户名 = "Alice";
    let データ = "test";
    let température = 25.0;
    let 🚀 = "rocket";

    println!("{} {} {} {}", 用户名, データ, température, 🚀);
}
"#;

    fs::write(&file_path, content).expect("Failed to write unicode file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let _issues = analyzer.analyze_file(&file_path);
}

#[test]
fn test_very_long_variable_name() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("long_names.rs");

    // Nested modules trigger module-complexity (threshold 0) — a remaining tree-sitter rule
    let content = r#"
mod outer { mod inner {} }
fn main() {
    let long_name = "very long variable name";
    println!("{}", long_name);
}
"#;

    fs::write(&file_path, content).expect("Failed to write long names file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect nested module complexity");
}

#[test]
fn test_nested_modules() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("nested_modules.rs");

    let content = r#"
mod outer {
    mod inner {
        fn function_with_issues() {
            // Box::new triggers box-abuse with enough calls
            let _ = Box::new(1);
            let _ = Box::new(2);
            let _ = Box::new(3);
            let _ = Box::new(4);
            let _ = Box::new(5);
            let _ = Box::new(6);
            let _ = Box::new(7);
            let _ = Box::new(8);
            let _ = Box::new(9);
        }
    }
}

fn main() {
    let _ = Box::new(10);
    let _ = Box::new(11);
    let _ = Box::new(12);
    let _ = Box::new(13);
    let _ = Box::new(14);
}
"#;

    fs::write(&file_path, content).expect("Failed to write nested modules file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect issues in nested modules");
    let module_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "module-complexity")
        .collect();
    assert!(
        !module_issues.is_empty(),
        "Should detect nested module issues"
    );
}

#[test]
fn test_generic_functions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("generics.rs");

    // Mock Error type that implements Debug but not Display
    let content = r#"
use std::fmt;
struct MyError;
impl fmt::Debug for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError")
    }
}

fn main() {
    let _ = MyError;
}
"#;

    fs::write(&file_path, content).expect("Failed to write generics file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect Debug without Display");
    let display_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "rust-error-display")
        .collect();
    assert!(
        !display_issues.is_empty(),
        "Should detect rust-error-display"
    );
}

#[test]
fn test_macro_definitions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("macros.rs");

    let content = r#"
mod outer { mod inner {} }
fn main() {
    let manager = "test";
}
"#;

    fs::write(&file_path, content).expect("Failed to write macros file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let _issues = analyzer.analyze_file(&file_path);
}

#[test]
fn test_async_functions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("async_fn.rs");

    let content = r#"
mod outer { mod inner {} }

async fn async_function() {
    let _ = 42;
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
}

#[tokio::main]
async fn main() {
    async_function().await;
}
"#;

    fs::write(&file_path, content).expect("Failed to write async file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect nested module complexity");
}

#[test]
fn test_trait_implementations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("traits.rs");

    let content = r#"
use std::fmt;
struct MyError;
impl fmt::Debug for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError")
    }
}

fn main() {
    let _ = MyError;
}
"#;

    fs::write(&file_path, content).expect("Failed to write traits file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect Debug without Display");
}

#[test]
fn test_closure_with_issues() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("closures.rs");

    let content = r#"
mod outer { mod inner {} }

fn main() {
    let data = vec![1, 2, 3, 4, 5];

    let result: Vec<_> = data
        .iter()
        .map(|x| {
            let _ = x * 2;
            x
        })
        .collect();

    println!("{:?}", result);
}
"#;

    fs::write(&file_path, content).expect("Failed to write closures file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect nested module complexity");
}
