// Additional tests to improve code coverage

use garbage_code_hunter::{CodeAnalyzer, I18n, Reporter};
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
    fs::write(&excluded_file, "fn main() { let data = \"test\"; }").expect("Failed to write file");

    let issues = analyzer.analyze_path(temp_dir.path());
    // Should have no issues because file is excluded
    assert!(issues.is_empty(), "Excluded files should not be analyzed");
}

#[test]
fn test_analyzer_with_empty_exclusions() {
    let analyzer = CodeAnalyzer::new(&[], "en-US");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, "fn main() { let data = \"test\"; }").expect("Failed to write file");

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
    fs::write(&file_path, "fn main() { let data = \"test\"; }").expect("Failed to write file");

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
fn main() {
    let data = "test";
    let temp = 42;
    let a = 10;
    let result = Some(42).unwrap();
    let s1 = String::from("test");
    let s2 = s1.clone();
    let s3 = s2.clone();
    let s4 = s3.clone();
    let s5 = s4.clone();
}
"#;

    fs::write(&file_path, code).expect("Failed to write test file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Test all possible reporter configurations
    let configurations = vec![
        // (harsh, savage, verbose, top, max_issues, summary, markdown, lang)
        (true, true, true, 1, 1, false, false, "zh-CN"),
        (true, true, true, 1, 1, false, true, "zh-CN"),
        (true, true, true, 1, 1, true, false, "zh-CN"),
        (true, true, true, 1, 1, true, true, "zh-CN"),
        (false, false, false, 10, 10, false, false, "en-US"),
        (false, false, false, 10, 10, false, true, "en-US"),
        (false, false, false, 10, 10, true, false, "en-US"),
        (false, false, false, 10, 10, true, true, "en-US"),
        (true, false, true, 5, 5, false, false, "zh-CN"),
        (false, true, false, 3, 3, false, false, "en-US"),
    ];

    for (harsh, savage, verbose, top, max_issues, summary, markdown, lang) in configurations {
        let reporter = Reporter::new(
            harsh, savage, verbose, top, max_issues, summary, markdown, lang,
        );
        reporter.report(issues.clone());
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
            "Should have messages for {}",
            rule_type
        );

        // Test English messages
        let en_messages = i18n_en.get_roast_messages(rule_type);
        assert!(
            !en_messages.is_empty(),
            "Should have messages for {}",
            rule_type
        );

        // Test invalid language (should fallback)
        let invalid_messages = i18n_invalid.get_roast_messages(rule_type);
        assert!(
            !invalid_messages.is_empty(),
            "Should have fallback messages for {}",
            rule_type
        );
    }
}

#[test]
fn test_i18n_all_suggestion_combinations() {
    let i18n_zh = I18n::new("zh-CN");
    let i18n_en = I18n::new("en-US");

    let rule_combinations = vec![
        vec![],
        vec!["terrible-naming".to_string()],
        vec!["deep-nesting".to_string()],
        vec!["long-function".to_string()],
        vec!["unwrap-abuse".to_string()],
        vec!["unnecessary-clone".to_string()],
        vec!["terrible-naming".to_string(), "deep-nesting".to_string()],
        vec!["unwrap-abuse".to_string(), "unnecessary-clone".to_string()],
        vec![
            "terrible-naming".to_string(),
            "deep-nesting".to_string(),
            "long-function".to_string(),
        ],
        vec![
            "terrible-naming".to_string(),
            "single-letter-variable".to_string(),
            "deep-nesting".to_string(),
            "long-function".to_string(),
            "unwrap-abuse".to_string(),
            "unnecessary-clone".to_string(),
        ],
    ];

    for rules in rule_combinations {
        let zh_suggestions = i18n_zh.get_suggestions(&rules);
        assert!(
            !zh_suggestions.is_empty(),
            "Should have Chinese suggestions for {:?}",
            rules
        );

        let en_suggestions = i18n_en.get_suggestions(&rules);
        assert!(
            !en_suggestions.is_empty(),
            "Should have English suggestions for {:?}",
            rules
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
            "Should handle missing key: {}",
            key
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

    // Should not analyze non-Rust files
    assert!(issues.is_empty(), "Should not analyze non-Rust files");
}

#[test]
fn test_analyzer_with_mixed_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create mixed files
    let rust_file = temp_dir.path().join("code.rs");
    let txt_file = temp_dir.path().join("readme.txt");

    fs::write(&rust_file, "fn main() { let data = \"test\"; }").expect("Failed to write rust file");
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
fn test_severity_and_roast_level_coverage() {
    use garbage_code_hunter::{RoastLevel, Severity};

    // Test all severity levels
    let severities = vec![Severity::Mild, Severity::Spicy, Severity::Nuclear];
    for severity in severities {
        // Test Debug trait
        let debug_str = format!("{:?}", severity);
        assert!(!debug_str.is_empty());

        // Test Clone trait
        let _cloned = severity.clone();

        // Test PartialEq trait
        assert_eq!(severity, severity);
    }

    // Test all roast levels
    let roast_levels = vec![
        RoastLevel::Gentle,
        RoastLevel::Sarcastic,
        RoastLevel::Savage,
    ];
    for roast_level in roast_levels {
        // Test Debug trait
        let debug_str = format!("{:?}", roast_level);
        assert!(!debug_str.is_empty());

        // Test Clone trait
        let _cloned = roast_level.clone();

        // Test PartialEq trait
        assert_eq!(roast_level, roast_level);
    }
}

#[test]
fn test_code_issue_debug_and_clone() {
    use garbage_code_hunter::{CodeIssue, RoastLevel, Severity};
    use std::path::PathBuf;

    let issue = CodeIssue {
        file_path: PathBuf::from("test.rs"),
        line: 10,
        column: 5,
        rule_name: "test-rule".to_string(),
        message: "Test message".to_string(),
        severity: Severity::Spicy,
        roast_level: RoastLevel::Sarcastic,
    };

    // Test Debug trait
    let debug_str = format!("{:?}", issue);
    assert!(debug_str.contains("test.rs"));
    assert!(debug_str.contains("Test message"));

    // Test Clone trait
    let cloned_issue = issue.clone();
    assert_eq!(issue.file_path, cloned_issue.file_path);
    assert_eq!(issue.message, cloned_issue.message);
}
