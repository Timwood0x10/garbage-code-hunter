// Additional tests to improve code coverage

use garbage_code_hunter::{CodeAnalyzer, I18n, LocalRoastProvider, Reporter};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_analyzer_with_multiple_exclusions() {
    let exclusions = vec![
        "target/*".to_string(),
        "test_*".to_string(),
        "tmp_*".to_string(),
        "*.tmp".to_string(),
        "build/*".to_string(),
        "node_modules/*".to_string(),
    ];

    let analyzer = CodeAnalyzer::new(&exclusions, "en-US");

    // Test with a directory that should be excluded
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let excluded_file = temp_dir.path().join("test_should_be_excluded.rs");
    fs::write(&excluded_file, "mod foo { mod bar {}\n}\nfn main() {}\n")
        .expect("Failed to write file");

    let issues = analyzer.analyze_path(temp_dir.path());
    // Should have no issues because file is excluded
    assert!(issues.is_empty(), "Excluded files should not be analyzed");
}

#[test]
fn test_analyzer_with_empty_exclusions() {
    let analyzer = CodeAnalyzer::new(&[], "en-US");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");
    // Use repeated code blocks to trigger intra-file duplication detection
    let code = r#"
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
    fs::write(&file_path, code).expect("Failed to write file");

    let issues = analyzer.analyze_file(&file_path);
    assert!(
        !issues.is_empty(),
        "Should detect issues when no exclusions"
    );
}

#[test]
fn test_analyzer_with_invalid_exclusion_patterns() {
    // Test with invalid regex patterns
    let exclusions = vec![
        "[invalid".to_string(), // Invalid regex
        "valid_pattern".to_string(),
    ];

    let analyzer = CodeAnalyzer::new(&exclusions, "en-US");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");
    // Use repeated code blocks to trigger intra-file duplication detection
    let code = r#"
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
    fs::write(&file_path, code).expect("Failed to write file");

    // Should still work even with invalid patterns
    let issues = analyzer.analyze_file(&file_path);
    assert!(
        !issues.is_empty(),
        "Should still work with invalid exclusion patterns"
    );
}

#[test]
fn test_reporter_all_combinations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    let code = r#"
mod foo { mod bar {} }
fn main() {
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
"#;

    fs::write(&file_path, code).expect("Failed to write test file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Test all possible reporter configurations
    let configurations = vec![
        (true, true, 1, false, false, false, "zh-CN"),
        (true, true, 1, false, false, true, "zh-CN"),
        (true, true, 1, true, false, false, "zh-CN"),
        (true, true, 1, true, false, true, "zh-CN"),
        (false, false, 10, false, false, false, "en-US"),
        (false, false, 10, false, false, true, "en-US"),
        (false, false, 10, true, false, false, "en-US"),
        (false, false, 10, true, false, true, "en-US"),
        (true, true, 5, false, false, false, "zh-CN"),
        (false, false, 3, false, false, false, "en-US"),
    ];

    for (harsh, verbose, max_issues, summary, brief, markdown, lang) in configurations {
        let reporter = Reporter::new(
            harsh,
            verbose,
            max_issues,
            summary,
            brief,
            markdown,
            lang,
            Box::new(LocalRoastProvider),
        );
        reporter.report_with_metrics(issues.clone(), 1, 100);
    }
}

#[test]
fn test_i18n_all_rule_types() {
    let i18n_zh = I18n::new("zh-CN");
    let i18n_en = I18n::new("en-US");
    let i18n_invalid = I18n::new("invalid-lang");

    let rule_types = vec![
        "terrible-naming",
        "single-letter-variable",
        "deep-nesting",
        "long-function",
        "unwrap-abuse",
        "unnecessary-clone",
        "unknown-rule",
    ];

    for rule_type in rule_types {
        // Test Chinese messages
        let zh_messages = i18n_zh.get_roast_messages(rule_type);
        assert!(
            !zh_messages.is_empty(),
            "Should have messages for {rule_type}"
        );

        // Test English messages
        let en_messages = i18n_en.get_roast_messages(rule_type);
        assert!(
            !en_messages.is_empty(),
            "Should have messages for {rule_type}"
        );

        // Test invalid language (should fallback)
        let invalid_messages = i18n_invalid.get_roast_messages(rule_type);
        assert!(
            !invalid_messages.is_empty(),
            "Should have fallback messages for {rule_type}"
        );
    }
}

#[test]
fn test_i18n_missing_keys() {
    let i18n = I18n::new("en-US");

    let missing_keys = vec![
        "nonexistent_key_1",
        "missing_translation",
        "invalid_key",
        "",
    ];

    for key in missing_keys {
        let result = i18n.get(key);
        assert!(
            result.contains("Missing translation"),
            "Should handle missing key: {key}"
        );
    }
}

#[test]
fn test_analyzer_with_non_rust_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create non-Rust files
    let txt_file = temp_dir.path().join("readme.txt");
    let js_file = temp_dir.path().join("script.js");
    let py_file = temp_dir.path().join("script.py");

    fs::write(&txt_file, "This is a text file").expect("Failed to write txt file");
    fs::write(&js_file, "console.log('hello');").expect("Failed to write js file");
    fs::write(&py_file, "print('hello')").expect("Failed to write py file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_path(temp_dir.path());

    // JS/Python rules have been migrated to LanguageAdapter pipeline.
    // analyze_path runs tree-sitter rules only, so non-Rust files may
    // produce 0 issues here (they are checked by the detector pipeline).
    // .txt files should still be ignored.
    assert!(
        issues
            .iter()
            .all(|i| i.file_path.extension().unwrap() != "txt"),
        "TXT files should not produce issues"
    );
}

#[test]
fn test_analyzer_with_mixed_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create mixed files — use a Rust file that triggers intra-file duplication
    let rust_file = temp_dir.path().join("code.rs");
    let txt_file = temp_dir.path().join("readme.txt");

    // Repeated code block triggers intra-file duplication detection
    let code = r#"
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
    fs::write(&rust_file, code).expect("Failed to write rust file");
    fs::write(&txt_file, "This is a text file").expect("Failed to write txt file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_path(temp_dir.path());

    // Should only analyze Rust files
    assert!(!issues.is_empty(), "Should analyze Rust files");
    assert!(
        issues
            .iter()
            .all(|issue| issue.file_path.extension().unwrap() == "rs"),
        "All issues should be from Rust files"
    );
}

#[test]
fn test_severity_coverage() {
    use garbage_code_hunter::Severity;

    // Test all severity levels
    let severities = vec![Severity::Mild, Severity::Spicy, Severity::Nuclear];
    for severity in severities {
        // Test Debug trait
        let debug_str = format!("{severity:?}");
        assert!(!debug_str.is_empty());

        // Test Clone trait
        let _cloned = severity.clone();

        // Test PartialEq trait
        assert_eq!(severity, severity);
    }
}

#[test]
fn test_code_issue_debug_and_clone() {
    use garbage_code_hunter::{CodeIssue, Severity};
    use std::path::PathBuf;

    let issue = CodeIssue {
        file_path: PathBuf::from("test.rs"),
        line: 10,
        column: 5,
        rule_name: "test-rule".to_string(),
        message: "Test message".to_string(),
        severity: Severity::Spicy,
    };

    // Test Debug trait
    let debug_str = format!("{issue:?}");
    assert!(debug_str.contains("test.rs"));
    assert!(debug_str.contains("Test message"));

    // Test Clone trait
    let cloned_issue = issue.clone();
    assert_eq!(issue.file_path, cloned_issue.file_path);
    assert_eq!(issue.message, cloned_issue.message);
}
