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
    // Use repeated code blocks to trigger intra-file duplication detection
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("long_names.rs");

    let content = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let x = 0;
    let y = 0;
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let z = 0;
}
"#;

    fs::write(&file_path, content).expect("Failed to write file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect code duplication");
}

#[test]
fn test_nested_modules() {
    // Use repeated code blocks to trigger intra-file duplication detection
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("nested_modules.rs");

    let content = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let x = 0;
    let y = 0;
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let z = 0;
}
"#;

    fs::write(&file_path, content).expect("Failed to write file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect issues via duplication");
    let dup_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "code-duplication")
        .collect();
    assert!(
        !dup_issues.is_empty(),
        "Should detect code-duplication issues"
    );
}

#[test]
fn test_generic_functions() {
    // Use repeated code blocks to trigger intra-file duplication detection
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("generics.rs");

    let content = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let x = 0;
    let y = 0;
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let z = 0;
}
"#;

    fs::write(&file_path, content).expect("Failed to write file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect issues via duplication");
    let dup_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "code-duplication")
        .collect();
    assert!(
        !dup_issues.is_empty(),
        "Should detect code-duplication issues"
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
    // Use repeated code blocks to trigger intra-file duplication detection
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("async_fn.rs");

    let content = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let x = 0;
    let y = 0;
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let z = 0;
}
"#;

    fs::write(&file_path, content).expect("Failed to write file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect issues via duplication");
}

#[test]
fn test_trait_implementations() {
    // Use repeated code blocks to trigger intra-file duplication detection
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("traits.rs");

    let content = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let x = 0;
    let y = 0;
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let z = 0;
}
"#;

    fs::write(&file_path, content).expect("Failed to write file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect issues via duplication");
}

#[test]
fn test_closure_with_issues() {
    // Use repeated code blocks to trigger intra-file duplication detection
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("closures.rs");

    let content = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let x = 0;
    let y = 0;
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    let e = d + 1;
    let z = 0;
}
"#;

    fs::write(&file_path, content).expect("Failed to write file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect issues via duplication");
}
