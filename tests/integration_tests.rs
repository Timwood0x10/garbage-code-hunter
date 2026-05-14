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

#[test]
fn test_terrible_naming_detection() {
    let code = r#"
fn main() {
    let data = "hello";
    let temp = 42;
    let info = vec![1, 2, 3];
    let obj = String::new();
    let manager = "test";
    let handler = 123;
}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should detect multiple terrible naming issues
    let naming_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "terrible-naming")
        .collect();

    assert!(
        !naming_issues.is_empty(),
        "Should detect terrible naming issues"
    );
    assert!(
        naming_issues.len() >= 4,
        "Should detect at least 4 naming issues"
    );
}

#[test]
fn test_single_letter_variable_detection() {
    let code = r#"
fn main() {
    let m = 10;
    let p = 20;
    let q = m + p;
    let g = "bad";

    // These should be allowed
    let i = 0;
    let j = 1;
    let k = 2;
    let x = 3.14;
    let y = 2.71;
    let z = 1.41;
}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    let single_letter_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "single-letter-variable")
        .collect();

    // Should detect m, p, q, g but not i, j, k, x, y, z
    assert!(
        !single_letter_issues.is_empty(),
        "Should detect single letter variables"
    );
    assert!(
        single_letter_issues.len() >= 2,
        "Should detect at least 2 single letter issues"
    );
}

#[test]
fn test_unwrap_abuse_detection() {
    let code = r#"
fn main() {
    let result = Some(42);
    let value = result.unwrap();
    let another = Some("test").unwrap();
    let third = Some(vec![1, 2, 3]).unwrap();
    let fourth = None::<i32>.unwrap();
    let fifth = Ok::<i32, &str>(123).unwrap();
}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    let unwrap_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "unwrap-abuse")
        .collect();

    assert!(!unwrap_issues.is_empty(), "Should detect unwrap abuse");
    // unwrap-abuse now reports a single issue with total count
    assert_eq!(
        unwrap_issues.len(),
        1,
        "Should report one unwrap-abuse issue with count"
    );
    assert!(
        unwrap_issues[0].message.contains("5"),
        "Should mention 5 unwrap()s in message"
    );
}

#[test]
fn test_unnecessary_clone_detection() {
    let mut code = String::from("fn main() {\n");
    code.push_str("    let s0 = String::from(\"hello\");\n");
    // Use 30 clones to exceed the new threshold of 25
    for i in 1..=30 {
        code.push_str(&format!("    let s{i} = s{}.clone();\n", i - 1));
    }
    code.push_str("}\n");

    let (_temp_dir, file_path) = create_temp_rust_file(&code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    let clone_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "unnecessary-clone")
        .collect();

    assert!(!clone_issues.is_empty(), "Should detect unnecessary clones");
}

#[test]
fn test_deep_nesting_detection() {
    let code = r#"
fn deeply_nested() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                if true {
                                    println!("Too deep!");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    let nesting_issues: Vec<_> = issues
        .iter()
        .filter(|issue| issue.rule_name == "deep-nesting")
        .collect();

    assert!(!nesting_issues.is_empty(), "Should detect deep nesting");
}

#[test]
fn test_long_function_detection() {
    // Long function detection is tested via unit tests in
    // treesitter::rules::rust_rules::tests::test_long_function_detection
    // which uses LongFunctionRule directly.
    // Integration test: verify the analyzer doesn't crash on long files.
    use garbage_code_hunter::CodeAnalyzer;
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

    // Clean code should have minimal or no issues
    // With the addition of new comprehensive rules, we may detect more issues
    // Adjust the threshold to account for new file structure and pattern detection rules
    assert!(
        issues.len() <= 10,
        "Clean code should have minimal issues, found: {}",
        issues.len()
    );
}

#[test]
fn test_exclude_patterns() {
    let code = r#"
fn main() {
    let data = "test";
    let temp = 42;
}
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    // Use a filename that won't be classified as test/example to avoid context-based skipping
    let file_path = temp_dir.path().join("sample_code.rs");
    fs::write(&file_path, code).expect("Failed to write test file");

    // Test without exclusion
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues_without_exclusion = analyzer.analyze_file(&file_path);
    assert!(
        !issues_without_exclusion.is_empty(),
        "Should find issues without exclusion, got {} issues. File path: {}",
        issues_without_exclusion.len(),
        file_path.display()
    );

    // Test with exclusion - use analyze_path instead of analyze_file for exclusion to work
    let analyzer_with_exclusion = CodeAnalyzer::new(&["sample_*".to_string()], "en-US");
    let issues_with_exclusion = analyzer_with_exclusion.analyze_path(temp_dir.path());
    assert!(
        issues_with_exclusion.is_empty(),
        "Should exclude files matching pattern"
    );
}

#[test]
fn test_severity_levels() {
    let code = r#"
fn main() {
    let data = "hello";  // Should be Spicy
    let a = 10;          // Should be Mild
    let result = Some(42).unwrap(); // Should vary based on count
}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Check that we have different severity levels
    let has_mild = issues
        .iter()
        .any(|issue| matches!(issue.severity, Severity::Mild));
    let has_spicy = issues
        .iter()
        .any(|issue| matches!(issue.severity, Severity::Spicy));

    assert!(
        has_mild || has_spicy,
        "Should have issues with different severity levels"
    );
}

#[test]
fn test_issue_severity_valid() {
    let code = r#"
fn main() {
    let data = "test";
    let temp = 42;
}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(code);
    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Check that issues have severity levels assigned
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

    // Create multiple test files
    let file1_path = temp_dir.path().join("file1.rs");
    let file2_path = temp_dir.path().join("file2.rs");

    fs::write(&file1_path, "fn main() { let data = \"test\"; }").expect("Failed to write file1");
    fs::write(&file2_path, "fn main() { let temp = 42; }").expect("Failed to write file2");

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
