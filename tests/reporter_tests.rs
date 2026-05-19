use garbage_code_hunter::{
    CodeAnalyzer, CodeIssue, CodeScorer, LocalRoastProvider, Reporter, Severity,
};
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
        },
        CodeIssue {
            file_path: PathBuf::from("test1.rs"),
            line: 15,
            column: 8,
            rule_name: "unwrap-abuse".to_string(),
            message: "Test unwrap abuse message".to_string(),
            severity: Severity::Nuclear,
        },
        CodeIssue {
            file_path: PathBuf::from("test2.rs"),
            line: 5,
            column: 1,
            rule_name: "single-letter-variable".to_string(),
            message: "Test single letter message".to_string(),
            severity: Severity::Mild,
        },
    ]
}

#[test]
fn test_reporter_creation() {
    let reporter = Reporter::new(
        false,
        false,
        5,
        false,
        false,
        false,
        "en-US",
        Box::new(LocalRoastProvider),
    );
    let issues = create_test_issues();
    // Smoke test: report_with_metrics should not panic
    reporter.report_with_metrics(issues, 1, 100);
}

#[test]
fn test_reporter_harsh_mode() {
    // Harsh mode filters to Nuclear + Spicy only
    let issues = create_test_issues();
    let spicy_and_nuclear: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Nuclear | Severity::Spicy))
        .collect();
    assert_eq!(
        spicy_and_nuclear.len(),
        2,
        "Test data should have 2 spicy+nuclear issues"
    );

    let reporter = Reporter::new(
        true,
        false,
        5,
        false,
        false,
        false,
        "en-US",
        Box::new(LocalRoastProvider),
    );
    reporter.report_with_metrics(issues, 1, 100);
}

#[test]
fn test_reporter_summary_only() {
    let reporter = Reporter::new(
        false,
        false,
        5,
        true,
        false,
        false,
        "zh-CN",
        Box::new(LocalRoastProvider),
    );
    let issues = create_test_issues();
    // Smoke test: summary-only mode should not panic
    reporter.report_with_metrics(issues, 1, 100);
}

#[test]
fn test_reporter_markdown_output() {
    let reporter = Reporter::new(
        false,
        true,
        3,
        false,
        false,
        true,
        "en-US",
        Box::new(LocalRoastProvider),
    );
    let issues = create_test_issues();
    // Smoke test: markdown mode should not panic
    reporter.report_with_metrics(issues, 1, 100);
}

#[test]
fn test_reporter_chinese_output() {
    let reporter = Reporter::new(
        false,
        true,
        5,
        false,
        false,
        false,
        "zh-CN",
        Box::new(LocalRoastProvider),
    );
    let issues = create_test_issues();
    // Smoke test: Chinese locale should not panic
    reporter.report_with_metrics(issues, 1, 100);
}

#[test]
fn test_reporter_empty_issues() {
    let reporter = Reporter::new(
        false,
        false,
        5,
        false,
        false,
        false,
        "en-US",
        Box::new(LocalRoastProvider),
    );
    let empty_issues = vec![];
    // Smoke test: empty issues path should show clean-code message without panicking
    reporter.report_with_metrics(empty_issues, 1, 100);
}

#[test]
fn test_reporter_limited_issues_per_file() {
    let reporter = Reporter::new(
        false,
        false,
        1,
        false,
        false,
        false,
        "en-US",
        Box::new(LocalRoastProvider),
    );
    let mut issues = vec![];
    for i in 0..5 {
        issues.push(CodeIssue {
            file_path: PathBuf::from("same_file.rs"),
            line: i * 10,
            column: 1,
            rule_name: "terrible-naming".to_string(),
            message: format!("Issue {i}"),
            severity: Severity::Spicy,
        });
    }
    // Smoke test: max_issues_per_file=1 should not panic (reporter handles truncation)
    reporter.report_with_metrics(issues, 1, 100);
}

#[test]
fn test_scorer_produces_reasonable_scores() {
    let scorer = CodeScorer::new();

    // Empty issues → score 0
    let score = scorer.calculate_score(&[], 1, 100);
    assert_eq!(score.total_score, 0.0);
    assert_eq!(score.n_score, 0.0);
    assert_eq!(score.d_score, 0.0);

    // Issues in a large codebase → lower density → lower score
    let issues = create_test_issues();
    let small_codebase = scorer.calculate_score(&issues, 1, 100);
    let large_codebase = scorer.calculate_score(&issues, 1, 100000);
    assert!(
        small_codebase.total_score >= large_codebase.total_score,
        "Same issues in smaller codebase should score >= larger codebase"
    );
}

#[test]
fn test_scorer_severity_distribution() {
    let scorer = CodeScorer::new();
    let issues = create_test_issues();
    let score = scorer.calculate_score(&issues, 1, 500);

    assert_eq!(
        score.severity_distribution.nuclear, 1,
        "Should have 1 nuclear issue"
    );
    assert_eq!(
        score.severity_distribution.spicy, 1,
        "Should have 1 spicy issue"
    );
    assert_eq!(
        score.severity_distribution.mild, 1,
        "Should have 1 mild issue"
    );
    assert_eq!(score.file_count, 1);
    assert_eq!(score.total_lines, 500);
}

#[test]
fn test_scorer_category_assignment() {
    let scorer = CodeScorer::new();
    let issues = create_test_issues();
    let score = scorer.calculate_score(&issues, 1, 500);

    // terrible-naming and single-letter-variable should be in "naming" category
    let naming_score = score.category_scores.get("naming");
    assert!(naming_score.is_some(), "Should have 'naming' category");
    assert!(
        naming_score.unwrap() > &0.0,
        "Naming category score should be > 0"
    );

    // unwrap-abuse now maps to PanicAddiction → "student-code"
    let student_score = score.category_scores.get("student-code");
    assert!(
        student_score.is_some(),
        "Should have 'student-code' category"
    );
    assert!(
        student_score.unwrap() > &0.0,
        "Student-code score should be > 0 for unwrap-abuse"
    );
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

    assert!(
        !issues.is_empty(),
        "Real analysis should find issues in test code"
    );

    // Verify scorer works with real issues
    let scorer = CodeScorer::new();
    let score = scorer.calculate_score(&issues, 1, code.lines().count());
    assert!(
        score.total_score > 0.0,
        "Score should be > 0 for code with issues"
    );

    // Test all reporter configurations with real issues (smoke test)
    let configurations = vec![
        (false, false, 5, false, false, "en-US"),
        (true, false, 3, false, false, "en-US"),
        (false, true, 5, false, false, "zh-CN"),
        (false, false, 5, true, false, "en-US"),
        (false, false, 5, false, true, "en-US"),
    ];

    for (harsh, verbose, max_issues, summary, markdown, lang) in configurations {
        let reporter = Reporter::new(
            harsh,
            verbose,
            max_issues,
            summary,
            false,
            markdown,
            lang,
            Box::new(LocalRoastProvider),
        );
        reporter.report_with_metrics(issues.clone(), 1, 100);
    }
}

#[test]
fn test_reporter_with_different_severities() {
    let issues = vec![
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            rule_name: "terrible-naming".to_string(),
            message: "Nuclear issue".to_string(),
            severity: Severity::Nuclear,
        },
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 2,
            column: 1,
            rule_name: "unwrap-abuse".to_string(),
            message: "Spicy issue".to_string(),
            severity: Severity::Spicy,
        },
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 3,
            column: 1,
            rule_name: "single-letter-variable".to_string(),
            message: "Mild issue".to_string(),
            severity: Severity::Mild,
        },
    ];

    // Verify all severity levels produce a non-zero score
    let scorer = CodeScorer::new();
    let score = scorer.calculate_score(&issues, 1, 200);
    assert!(score.total_score > 0.0);
    assert!(score.severity_distribution.nuclear > 0);
    assert!(score.severity_distribution.spicy > 0);
    assert!(score.severity_distribution.mild > 0);

    // Smoke test with verbose mode
    let reporter = Reporter::new(
        false,
        true,
        5,
        false,
        false,
        false,
        "en-US",
        Box::new(LocalRoastProvider),
    );
    reporter.report_with_metrics(issues, 1, 200);
}
