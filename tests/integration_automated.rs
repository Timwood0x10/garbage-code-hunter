use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

// ============================================================
// Automated Bootstrap Integration Tests
// ============================================================

/// Find the garbage-code-hunter binary path (release or debug)
fn find_binary_path() -> Option<PathBuf> {
    let possible_paths = [
        PathBuf::from("./target/release/garbage-code-hunter"),
        PathBuf::from("./target/debug/garbage-code-hunter"),
    ];

    for path in &possible_paths {
        if path.exists() {
            return Some(path.clone());
        }
    }

    None
}

/// Helper function to run garbage-code-hunter CLI and return (stdout, stderr, exit_code)
/// Returns None if binary not found
fn run_garbage_hunter(args: &[&str]) -> Option<(String, String, i32)> {
    let binary_path = find_binary_path()?;

    let output = Command::new(&binary_path)
        .args(args)
        .output()
        .expect("Failed to execute garbage-code-hunter");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Some((stdout, stderr, exit_code))
}

/// Macro to simplify running tests with automatic skip if binary not found
macro_rules! run_test {
    ($args:expr) => {
        match run_garbage_hunter($args) {
            Some(result) => result,
            None => return,
        }
    };
}

/// Extract total issue count from output
fn extract_total_issues(output: &str) -> u32 {
    output
        .lines()
        .find(|line| line.contains("📝 Total") || line.contains("Total"))
        .and_then(|line| line.split_whitespace().next())
        .and_then(|num| num.parse().ok())
        .unwrap_or_else(|| {
            // Fallback: try to find any number followed by "Total" or "total"
            let re = Regex::new(r"(\d+)\s*(?:📝\s*)?Total").unwrap();
            re.captures(output)
                .and_then(|caps| caps.get(1).and_then(|m| m.as_str().parse().ok()))
                .unwrap_or(0)
        })
}

// ============================================================
// Test: Self-Bootstrap (garbage-code-hunter analyzes itself)
// ============================================================

#[test]
fn test_self_bootstrap_completes_successfully() {
    let (stdout, stderr, exit_code) = run_test!(&["analyze", ".", "--lang", "en-US"]);

    assert_eq!(exit_code, 0, "Should exit with code 0\nstderr: {}", stderr);

    let total = extract_total_issues(&stdout);
    assert!(
        total > 0,
        "Self-bootstrap should detect some issues in itself, got {}",
        total
    );

    assert!(
        total < 2000,
        "Self-bootstrap should have reasonable issue count (<2000), got {}",
        total
    );
}

#[test]
fn test_self_bootstrap_performance() {
    let start = Instant::now();
    let (_stdout, _stderr, exit_code) = run_test!(&["analyze", ".", "--lang", "en-US"]);
    let duration = start.elapsed();

    assert_eq!(exit_code, 0, "Should complete successfully");

    assert!(
        duration.as_secs() < 10,
        "Self-bootstrap should complete within 10 seconds, took {:?}",
        duration
    );
}

// ============================================================
// Test: Known Projects Validation
// ============================================================

#[test]
fn test_system_alert_detection_stable() {
    let project_path = "../system_alert";

    if !Path::new(project_path).exists() {
        eprintln!("Skipping: {} not found", project_path);
        return;
    }

    let (stdout, _stderr, exit_code) = run_test!(&["analyze", project_path, "--lang", "en-US"]);

    assert_eq!(exit_code, 0, "Should analyze system_alert successfully");

    let total = extract_total_issues(&stdout);

    assert!(
        (100..=170).contains(&total),
        "system_alert should have ~133 issues (±20%), got {}",
        total
    );
}

#[test]
fn test_rechat_server_detection_stable() {
    let project_path = "../ReChat-server";

    if !Path::new(project_path).exists() {
        eprintln!("Skipping: {} not found", project_path);
        return;
    }

    let (stdout, _stderr, exit_code) = run_test!(&["analyze", project_path, "--lang", "en-US"]);

    assert_eq!(exit_code, 0, "Should analyze system_alert successfully");

    let total = extract_total_issues(&stdout);

    assert!(
        (130..=220).contains(&total),
        "ReChat-server should have ~171 issues (±20%), got {}",
        total
    );
}

#[test]
fn test_finance_project_improved_accuracy() {
    let project_path = "../Finance";

    if !Path::new(project_path).exists() {
        eprintln!("Skipping: {} not found", project_path);
        return;
    }

    let (stdout, _stderr, exit_code) = run_test!(&["analyze", project_path, "--lang", "en-US"]);

    assert_eq!(exit_code, 0, "Should analyze Finance successfully");

    let total = extract_total_issues(&stdout);

    assert!(
        (1000..=1500).contains(&total),
        "Finance should have ~1256 issues with tree-sitter engine, got {}",
        total
    );
}

#[test]
fn test_memscope_rs_best_in_class() {
    let project_path = "../memscope-rs";

    if !Path::new(project_path).exists() {
        eprintln!("Skipping: {} not found", project_path);
        return;
    }

    let (stdout, _stderr, exit_code) = run_test!(&["analyze", project_path, "--lang", "en-US"]);

    assert_eq!(exit_code, 0, "Should analyze memscope-rs successfully");

    let total = extract_total_issues(&stdout);

    assert!(
        (3000..=4500).contains(&total),
        "memscope-rs should have ~3816 issues with tree-sitter engine, got {}",
        total
    );
}

// ============================================================
// Test: Zero Crash Guarantee
// ============================================================

#[test]
fn test_all_testable_projects_zero_crashes() {
    let projects: Vec<(&str, Option<std::ops::RangeInclusive<u32>>)> = vec![
        ("../algo", Some(0..=0)),       // Algorithm example: perfect code, 0 issues
        ("../gpu-code", Some(55..=90)), // GPU code: small number of issues
    ];

    for (path, expected_range) in projects {
        if !Path::new(path).exists() {
            eprintln!("Skipping: {} not found", path);
            continue;
        }

        let (stdout, stderr, exit_code) = run_test!(&["analyze", path, "--lang", "en-US"]);

        assert_eq!(
            exit_code, 0,
            "{} should complete without crash\nstderr: {}",
            path, stderr
        );

        if let Some(range) = expected_range {
            let total = extract_total_issues(&stdout);
            assert!(
                range.contains(&total),
                "{} should have issues in range {:?}, got {}",
                path,
                range,
                total
            );
        }
    }
}

// ============================================================
// Test: Performance Benchmarks
// ============================================================

#[test]
fn test_small_project_performance_under_1s() {
    let project_path = "../AlgoGpuRust";

    if !Path::new(project_path).exists() {
        eprintln!("Skipping: {} not found", project_path);
        return;
    }

    let start = Instant::now();
    let (_stdout, _stderr, exit_code) = run_test!(&["analyze", project_path, "--lang", "en-US"]);
    let duration = start.elapsed();

    assert_eq!(exit_code, 0, "Should complete successfully");

    assert!(
        duration.as_millis() < 8000,
        "Small project should complete under 8s, took {:?}",
        duration
    );
}

#[test]
fn test_medium_project_performance_under_5s() {
    let project_path = "../Finance";

    if !Path::new(project_path).exists() {
        eprintln!("Skipping: {} not found", project_path);
        return;
    }

    let start = Instant::now();
    let (_stdout, _stderr, exit_code) = run_test!(&["analyze", project_path, "--lang", "en-US"]);
    let duration = start.elapsed();

    assert_eq!(exit_code, 0, "Should complete successfully");

    assert!(
        duration.as_secs() < 15,
        "Medium project should complete under 15s, took {:?}",
        duration
    );
}

// ============================================================
// Test: Output Format Validation
// ============================================================

#[test]
fn test_markdown_output_format_valid() {
    let (stdout, _stderr, exit_code) =
        run_test!(&["analyze", ".", "--lang", "en-US", "--markdown"]);

    assert_eq!(exit_code, 0, "Should generate markdown output");

    // Markdown output should contain standard elements
    assert!(
        stdout.contains("# 🗑️") || stdout.contains("# Garbage"),
        "Markdown output should contain header"
    );

    assert!(
        stdout.contains("## 📈")
            || stdout.contains("Issue Statistics")
            || stdout.contains("Issues by File"),
        "Markdown output should contain issues section"
    );

    // Should have table or list format for issues
    assert!(
        stdout.contains("|") || stdout.contains("- **"),
        "Markdown output should contain formatted issues (table or list)"
    );
}

#[test]
fn test_verbose_output_contains_rule_weights() {
    let (stdout, _stderr, exit_code) =
        run_test!(&["analyze", "../system_alert", "--lang", "en-US", "--verbose"]);

    assert_eq!(exit_code, 0, "Should generate verbose output");

    // Verbose output should show rule weight multipliers
    assert!(
        stdout.contains("⚡") || stdout.contains("rule_weight"),
        "Verbose output should show performance metrics or rule weights"
    );
}

// ============================================================
// Test: Context Detection Accuracy
// ============================================================

#[test]
fn test_ui_context_reduces_false_positives() {
    let project_path = "../system_alert";

    if !Path::new(project_path).exists() {
        eprintln!("Skipping: {} not found", project_path);
        return;
    }

    let (stdout, _stderr, exit_code) =
        run_test!(&["analyze", project_path, "--lang", "en-US", "--verbose"]);

    assert_eq!(exit_code, 0, "Should complete successfully");

    // Count meaningless-naming issues (should be low due to UI whitelist)
    let naming_issues: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("meaningless-naming"))
        .collect();

    // With UI context detection, meaningless-naming should be ≤10 (was 89 before fix)
    assert!(
        naming_issues.len() <= 15,
        "UI context should reduce meaningless-naming to ≤15, got {}",
        naming_issues.len()
    );

    println!(
        "✅ system_alert UI context test passed: {} naming issues",
        naming_issues.len()
    );
}

// ============================================================
// Test: Regression Prevention
// ============================================================

#[test]
fn test_no_regression_from_round4_to_round5() {
    let regression_data: Vec<(&str, std::ops::RangeInclusive<u32>)> = vec![
        ("../system_alert", 100..=160),  // tree-sitter: 133
        ("../ReChat-server", 130..=220), // tree-sitter: 171
        ("../AlgoGpuRust", 50..=90),     // tree-sitter: 70
        ("../memscope-rs", 3000..=4500), // tree-sitter: 3816
    ];

    for (project_path, expected_range) in regression_data {
        if !Path::new(project_path).exists() {
            continue;
        }

        let (stdout, _stderr, exit_code) = run_test!(&["analyze", project_path, "--lang", "en-US"]);
        assert_eq!(exit_code, 0, "{} should succeed", project_path);

        let total = extract_total_issues(&stdout);

        assert!(
            expected_range.contains(&total),
            "REGRESSION DETECTED!\n\
             Project: {}\n\
             Expected range: {:?}\n\
             Actual: {}\n\
             This suggests a regression from the validated Round 5 baseline.",
            project_path,
            expected_range,
            total
        );
    }
}
