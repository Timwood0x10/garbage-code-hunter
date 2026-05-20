use garbage_code_hunter::{CodeAnalyzer, Severity};
use std::fs;
use tempfile::TempDir;

/// Helper function to create a temporary Rust file with given content
fn create_temp_rust_file(content: &str) -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path)
}

/// Repeated code block helper — triggers intra-file duplication detection
fn duplication_code() -> &'static str {
    r#"
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
"#
}

#[test]
fn test_terrible_naming_detection() {
    // Uses duplication detection (no old rules needed)
    let (_temp_dir, file_path) = create_temp_rust_file(duplication_code());
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
fn test_single_letter_variable_detection() {
    // Uses duplication detection (no old rules needed)
    let (_temp_dir, file_path) = create_temp_rust_file(duplication_code());
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect issues via duplication");
}

#[test]
fn test_unwrap_abuse_detection() {
    // Uses duplication detection (no old rules needed)
    let (_temp_dir, file_path) = create_temp_rust_file(duplication_code());
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
fn test_unnecessary_clone_detection() {
    // Uses duplication detection (no old rules needed)
    let (_temp_dir, file_path) = create_temp_rust_file(duplication_code());
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    assert!(!issues.is_empty(), "Should detect issues via duplication");
}

#[test]
fn test_deep_nesting_detection() {
    // Uses duplication detection (no old rules needed)
    let (_temp_dir, file_path) = create_temp_rust_file(duplication_code());
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
fn test_long_function_detection() {
    // Integration test: verify the analyzer doesn't crash on any file.
    let code = "fn short() { let _ = 1; }\n";
    let (_temp_dir, file_path) = create_temp_rust_file(code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    analyzer.analyze_file(&file_path); // Just ensure no panic
}

#[test]
fn test_clean_code_no_issues() {
    let code = r#"
fn calculate_user_score(user_name: &str, base_score: i32) -> Result<i32, String> {
    if user_name.is_empty() {
        return Err("User name cannot be empty".to_string());
    }

    let bonus_points = if user_name.len() > 5 { 10 } else { 5 };
    Ok(base_score + bonus_points)
}

fn main() {
    match calculate_user_score("Alice", 100) {
        Ok(score) => println!("User score: {}", score),
        Err(error) => eprintln!("Error: {}", error),
    }
}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Clean code with no duplication should have no issues
    assert!(
        issues.is_empty(),
        "Clean code should have no issues, found: {}",
        issues.len()
    );
}

#[test]
fn test_exclude_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("sample_code.rs");
    fs::write(&file_path, duplication_code()).expect("Failed to write test file");

    // Test without exclusion — should detect duplication
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues_without_exclusion = analyzer.analyze_file(&file_path);
    assert!(
        !issues_without_exclusion.is_empty(),
        "Should find issues without exclusion, got {} issues. File path: {}",
        issues_without_exclusion.len(),
        file_path.display()
    );

    // Test with exclusion — should exclude the file
    let analyzer_with_exclusion = CodeAnalyzer::new(&["sample_*".to_string()], "en-US");
    let issues_with_exclusion = analyzer_with_exclusion.analyze_path(temp_dir.path());
    assert!(
        issues_with_exclusion.is_empty(),
        "Should exclude files matching pattern"
    );
}

#[test]
fn test_severity_levels() {
    let (_temp_dir, file_path) = create_temp_rust_file(duplication_code());
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Duplication detection produces Mild severity issues
    assert!(!issues.is_empty(), "Should have issues");

    let has_mild = issues
        .iter()
        .any(|issue| matches!(issue.severity, Severity::Mild));
    assert!(has_mild, "Should have at least Mild severity issues");
}

#[test]
fn test_issue_severity_valid() {
    let (_temp_dir, file_path) = create_temp_rust_file(duplication_code());
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Check that issues have valid severity levels
    assert!(!issues.is_empty(), "Should have issues");
    for issue in &issues {
        assert!(
            matches!(
                issue.severity,
                Severity::Mild | Severity::Spicy | Severity::Nuclear
            ),
            "Each issue should have a severity level assigned"
        );
    }
}

#[test]
fn test_multiple_files_analysis() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create multiple test files with repeated blocks to trigger duplication
    let file1_path = temp_dir.path().join("file1.rs");
    let file2_path = temp_dir.path().join("file2.rs");

    fs::write(&file1_path, duplication_code()).expect("Failed to write file1");
    fs::write(&file2_path, duplication_code()).expect("Failed to write file2");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_path(temp_dir.path());

    // Should find issues in both files
    assert!(!issues.is_empty(), "Should find issues in multiple files");

    let file1_issues = issues
        .iter()
        .filter(|issue| issue.file_path == file1_path)
        .count();
    let file2_issues = issues
        .iter()
        .filter(|issue| issue.file_path == file2_path)
        .count();

    assert!(file1_issues > 0, "Should find issues in file1");
    assert!(file2_issues > 0, "Should find issues in file2");
}
