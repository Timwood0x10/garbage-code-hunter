use garbage_code_hunter::{CodeAnalyzer, CodeIssue, Reporter, RoastLevel, Severity};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_issues() -> Vec<CodeIssue> {
    vec![
        CodeIssue {
            file_path: PathBuf::from("test1.rs"),
            line: 10,
            column: 5,
            rule_name: "terrible-naming".to_string(),
            message: "Test terrible naming message".to_string(),
            severity: Severity::Spicy,
            roast_level: RoastLevel::Sarcastic,
        },
        CodeIssue {
            file_path: PathBuf::from("test1.rs"),
            line: 15,
            column: 8,
            rule_name: "unwrap-abuse".to_string(),
            message: "Test unwrap abuse message".to_string(),
            severity: Severity::Nuclear,
            roast_level: RoastLevel::Savage,
        },
        CodeIssue {
            file_path: PathBuf::from("test2.rs"),
            line: 5,
            column: 1,
            rule_name: "single-letter-variable".to_string(),
            message: "Test single letter message".to_string(),
            severity: Severity::Mild,
            roast_level: RoastLevel::Gentle,
        },
    ]
}

#[test]
fn test_reporter_creation() {
    let reporter = Reporter::new(
        false,   // harsh_mode
        false,   // savage_mode
        false,   // verbose
        5,       // top_files
        5,       // max_issues_per_file
        false,   // summary_only
        false,   // markdown
        "en-US", // lang
    );

    // Just test that creation doesn't panic
    let issues = create_test_issues();
    reporter.report(issues);
}

#[test]
fn test_reporter_harsh_mode() {
    let reporter = Reporter::new(
        true,    // harsh_mode - should filter to only Nuclear and Spicy
        false,   // savage_mode
        false,   // verbose
        5,       // top_files
        5,       // max_issues_per_file
        false,   // summary_only
        false,   // markdown
        "en-US", // lang
    );

    let issues = create_test_issues();
    reporter.report(issues);

    // In harsh mode, only Nuclear and Spicy issues should be shown
    // This is more of a visual test - we'd need to capture output to test properly
}

#[test]
fn test_reporter_summary_only() {
    let reporter = Reporter::new(
        false,   // harsh_mode
        false,   // savage_mode
        false,   // verbose
        5,       // top_files
        5,       // max_issues_per_file
        true,    // summary_only - should skip detailed output
        false,   // markdown
        "zh-CN", // lang
    );

    let issues = create_test_issues();
    reporter.report(issues);
}

#[test]
fn test_reporter_markdown_output() {
    let reporter = Reporter::new(
        false,   // harsh_mode
        false,   // savage_mode
        true,    // verbose
        3,       // top_files
        3,       // max_issues_per_file
        false,   // summary_only
        true,    // markdown - should output markdown format
        "en-US", // lang
    );

    let issues = create_test_issues();
    reporter.report(issues);
}

#[test]
fn test_reporter_chinese_output() {
    let reporter = Reporter::new(
        false,   // harsh_mode
        true,    // savage_mode - should make messages more aggressive
        true,    // verbose
        5,       // top_files
        5,       // max_issues_per_file
        false,   // summary_only
        false,   // markdown
        "zh-CN", // lang
    );

    let issues = create_test_issues();
    reporter.report(issues);
}

#[test]
fn test_reporter_empty_issues() {
    let reporter = Reporter::new(
        false,   // harsh_mode
        false,   // savage_mode
        false,   // verbose
        5,       // top_files
        5,       // max_issues_per_file
        false,   // summary_only
        false,   // markdown
        "en-US", // lang
    );

    let empty_issues = vec![];
    reporter.report(empty_issues);

    // Should show clean code message
}

#[test]
fn test_reporter_limited_issues_per_file() {
    let reporter = Reporter::new(
        false,   // harsh_mode
        false,   // savage_mode
        false,   // verbose
        5,       // top_files
        1,       // max_issues_per_file - should limit to 1 issue per file
        false,   // summary_only
        false,   // markdown
        "en-US", // lang
    );

    // Create multiple issues for the same file
    let mut issues = vec![];
    for i in 0..5 {
        issues.push(CodeIssue {
            file_path: PathBuf::from("same_file.rs"),
            line: i * 10,
            column: 1,
            rule_name: "terrible-naming".to_string(),
            message: format!("Issue {}", i),
            severity: Severity::Spicy,
            roast_level: RoastLevel::Sarcastic,
        });
    }

    reporter.report(issues);
    // Should only show 1 issue per file
}

#[test]
fn test_integration_with_real_analysis() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_integration.rs");

    let code = r#"
fn main() {
    let data = "hello";
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

    // Test different reporter configurations with real issues
    let configurations = vec![
        (false, false, false, 5, 5, false, false, "en-US"),
        (true, false, false, 3, 3, false, false, "en-US"),
        (false, true, true, 5, 5, false, false, "zh-CN"),
        (false, false, false, 5, 5, true, false, "en-US"),
        (false, false, false, 5, 5, false, true, "en-US"),
    ];

    for (harsh, savage, verbose, top, max_issues, summary, markdown, lang) in configurations {
        let reporter = Reporter::new(
            harsh, savage, verbose, top, max_issues, summary, markdown, lang,
        );
        reporter.report(issues.clone());
        println!("--- Configuration tested ---");
    }
}

#[test]
fn test_reporter_with_different_severities() {
    let mut issues = vec![];

    // Create issues with all severity levels
    issues.push(CodeIssue {
        file_path: PathBuf::from("test.rs"),
        line: 1,
        column: 1,
        rule_name: "terrible-naming".to_string(),
        message: "Nuclear issue".to_string(),
        severity: Severity::Nuclear,
        roast_level: RoastLevel::Savage,
    });

    issues.push(CodeIssue {
        file_path: PathBuf::from("test.rs"),
        line: 2,
        column: 1,
        rule_name: "unwrap-abuse".to_string(),
        message: "Spicy issue".to_string(),
        severity: Severity::Spicy,
        roast_level: RoastLevel::Sarcastic,
    });

    issues.push(CodeIssue {
        file_path: PathBuf::from("test.rs"),
        line: 3,
        column: 1,
        rule_name: "single-letter-variable".to_string(),
        message: "Mild issue".to_string(),
        severity: Severity::Mild,
        roast_level: RoastLevel::Gentle,
    });

    let reporter = Reporter::new(false, false, true, 5, 5, false, false, "en-US");
    reporter.report(issues);
}
