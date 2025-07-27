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

    // Empty file should have no issues
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
/// Documentation comment
//! Inner documentation comment
"#;

    fs::write(&file_path, content).expect("Failed to write comments file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Comments-only file should have no issues
    assert!(
        issues.is_empty(),
        "Comments-only file should have no issues"
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

    // Whitespace-only file should have no issues
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

    // Minimal valid Rust should have no issues
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
    let データ = "test";  // This should be detected as terrible naming
    let température = 25.0;
    let 🚀 = "rocket";
    
    println!("{} {} {} {}", 用户名, データ, température, 🚀);
}
"#;

    fs::write(&file_path, content).expect("Failed to write unicode file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let _issues = analyzer.analyze_file(&file_path);

    // Should handle Unicode variable names gracefully
    // May or may not detect issues depending on implementation
}

#[test]
fn test_very_long_variable_name() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("long_names.rs");

    let very_long_name = "a".repeat(1000);
    let content = format!(
        r#"
fn main() {{
    let {very_long_name} = "very long variable name";
    let data = "should be detected";
    println!("{{}}", {very_long_name});
}}
"#
    );

    fs::write(&file_path, content).expect("Failed to write long names file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should handle very long variable names without crashing
    // Should still detect "data" as terrible naming
    let naming_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "terrible-naming")
        .collect();

    assert!(
        !naming_issues.is_empty(),
        "Should detect 'data' as terrible naming"
    );
}

#[test]
fn test_nested_modules() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("nested_modules.rs");

    let content = r#"
mod outer {
    mod inner {
        fn function_with_issues() {
            let data = "test";
            let temp = 42;
            let result = Some(123).unwrap();
        }
    }
    
    fn another_function() {
        let info = vec![1, 2, 3];
        let obj = String::new();
    }
}

fn main() {
    let manager = "test";
}
"#;

    fs::write(&file_path, content).expect("Failed to write nested modules file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should detect issues in nested modules
    assert!(!issues.is_empty(), "Should detect issues in nested modules");

    let naming_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "terrible-naming")
        .collect();

    assert!(
        naming_issues.len() >= 3,
        "Should detect multiple naming issues across modules"
    );
}

#[test]
fn test_generic_functions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("generics.rs");

    let content = r#"
fn generic_function<T, U>(data: T, temp: U) -> T 
where 
    T: Clone,
    U: std::fmt::Debug,
{
    let info = data.clone();
    let obj = format!("{:?}", temp);
    println!("{}", obj);
    info
}

fn main() {
    let result = generic_function("hello", 42);
    println!("{}", result);
}
"#;

    fs::write(&file_path, content).expect("Failed to write generics file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should detect terrible naming in generic functions
    let naming_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "terrible-naming")
        .collect();

    assert!(
        !naming_issues.is_empty(),
        "Should detect naming issues in generic functions"
    );
}

#[test]
fn test_macro_definitions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("macros.rs");

    let content = r#"
macro_rules! bad_macro {
    ($data:expr, $temp:expr) => {
        {
            let info = $data;
            let obj = $temp;
            let result = Some(info).unwrap();
            println!("{} {}", obj, result);
        }
    };
}

fn main() {
    bad_macro!("hello", 42);
    let manager = "test";
}
"#;

    fs::write(&file_path, content).expect("Failed to write macros file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let _issues = analyzer.analyze_file(&file_path);

    // Should handle macros gracefully and detect issues where possible
    // The exact behavior may vary depending on how syn handles macros
}

#[test]
fn test_async_functions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("async_fn.rs");

    let content = r#"
async fn async_function() {
    let data = "async test";
    let temp = 42;
    let result = Some("value").unwrap();
    
    // Simulate some async work
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    
    println!("{} {} {}", data, temp, result);
}

#[tokio::main]
async fn main() {
    async_function().await;
    let info = "main function";
}
"#;

    fs::write(&file_path, content).expect("Failed to write async file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should detect issues in async functions
    let naming_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "terrible-naming")
        .collect();

    assert!(
        !naming_issues.is_empty(),
        "Should detect naming issues in async functions"
    );
}

#[test]
fn test_trait_implementations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("traits.rs");

    let content = r#"
trait BadTrait {
    fn bad_method(&self) {
        let data = "trait method";
        let temp = 42;
    }
}

struct BadStruct {
    data: String,
    temp: i32,
}

impl BadTrait for BadStruct {
    fn bad_method(&self) {
        let info = &self.data;
        let obj = self.temp;
        let result = Some(obj).unwrap();
        println!("{} {}", info, result);
    }
}

fn main() {
    let instance = BadStruct {
        data: "test".to_string(),
        temp: 42,
    };
    instance.bad_method();
}
"#;

    fs::write(&file_path, content).expect("Failed to write traits file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should detect issues in trait implementations
    assert!(
        !issues.is_empty(),
        "Should detect issues in trait implementations"
    );
}

#[test]
fn test_closure_with_issues() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("closures.rs");

    let content = r#"
fn main() {
    let data = vec![1, 2, 3, 4, 5];
    
    let result: Vec<_> = data
        .iter()
        .map(|x| {
            let temp = x * 2;
            let info = temp + 1;
            let obj = Some(info).unwrap();
            obj
        })
        .collect();
    
    println!("{:?}", result);
}
"#;

    fs::write(&file_path, content).expect("Failed to write closures file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should detect issues inside closures
    let naming_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "terrible-naming")
        .collect();

    assert!(
        !naming_issues.is_empty(),
        "Should detect naming issues in closures"
    );
}
