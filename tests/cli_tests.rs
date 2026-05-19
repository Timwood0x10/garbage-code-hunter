use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test the CLI interface by running the actual binary
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "analyze", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("garbage-code-hunter") || stdout.contains("Analyze"));
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("--lang"));
    assert!(stdout.contains("--markdown"));
}

#[test]
fn test_cli_version_info() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("garbage-code-hunter")
            && (stdout.contains("0.2.1") || stdout.contains("0.2.2"))
    );
}

#[test]
fn test_cli_with_garbage_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("garbage.rs");

    let garbage_code = r#"
fn main() {
    let data = "hello";
    let temp = 42;
    let a = 10;
    let result = Some(42).unwrap();
}
"#;

    fs::write(&file_path, garbage_code).expect("Failed to write test file");

    let output = Command::new("cargo")
        .args(["run", "--", "analyze", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain some analysis output
    assert!(stdout.contains("垃圾代码") || stdout.contains("Garbage Code"));
}

#[test]
fn test_cli_english_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    fs::write(&file_path, "fn main() { let thing = \"test\"; }")
        .expect("Failed to write test file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--lang",
            "en-US",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain English output
    assert!(stdout.contains("Garbage Code Hunter") || stdout.contains("variable"));
}

#[test]
fn test_cli_chinese_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    fs::write(&file_path, "fn main() { let thing = \"test\"; }")
        .expect("Failed to write test file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--lang",
            "zh-CN",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain Chinese output
    assert!(stdout.contains("垃圾代码") || stdout.contains("变量"));
}

/// Objective: Verify analyze JSON exposes issues and Style IR summary.
/// Invariants: JSON output must remain parseable and include stable summary fields.
#[test]
fn test_cli_json_includes_style_ir_summary() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    fs::write(
        &file_path,
        "fn main() { let result = Some(42).unwrap(); println!(\"debug\"); }",
    )
    .expect("Failed to write test file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--format",
            "json",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "analyze JSON command should exit successfully"
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("analyze JSON output should be valid JSON");

    assert!(
        parsed
            .get("issues")
            .and_then(|value| value.as_array())
            .is_some(),
        "analyze JSON should include an issues array"
    );
    assert_eq!(
        parsed
            .get("schema_version")
            .and_then(|value| value.as_str()),
        Some("1.0")
    );
    assert!(
        parsed
            .get("files")
            .and_then(|value| value.as_array())
            .is_some(),
        "analyze JSON should include a files array"
    );
    assert_eq!(
        parsed
            .get("summary")
            .and_then(|value| value.get("issue_count"))
            .and_then(|value| value.as_u64()),
        parsed
            .get("issues")
            .and_then(|value| value.as_array())
            .map(|issues| issues.len() as u64)
    );
    let summary = parsed
        .get("style_ir_summary")
        .expect("analyze JSON should include style_ir_summary");
    assert_eq!(
        summary.get("language").and_then(|value| value.as_str()),
        Some("Rust")
    );
    assert!(
        summary.get("thresholds").is_some(),
        "style_ir_summary should include thresholds"
    );
    assert!(
        summary
            .get("panic_call_count")
            .and_then(|value| value.as_u64())
            .unwrap_or_default()
            >= 1,
        "style_ir_summary should expose panic call count"
    );
}

#[test]
fn test_cli_markdown_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    fs::write(&file_path, "fn main() { let data = \"test\"; }").expect("Failed to write test file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--markdown",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain markdown formatting
    assert!(stdout.contains("#") && (stdout.contains("|") || stdout.contains("**")));
}

#[test]
fn test_cli_summary_only() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    fs::write(&file_path, "fn main() { let data = \"test\"; }").expect("Failed to write test file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--summary",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Summary mode should be shorter
    assert!(!stdout.is_empty());
}

#[test]
fn test_cli_verbose_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    fs::write(
        &file_path,
        "fn main() { let data = \"test\"; let temp = 42; }",
    )
    .expect("Failed to write test file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--verbose",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verbose mode should contain detailed analysis
    assert!(stdout.contains("MUTATION ANALYSIS") || stdout.contains("Personality"));
}

#[test]
fn test_cli_issues_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    let code_with_many_issues = r#"
fn main() {
    let data = "test";
    let temp = 42;
    let info = vec![1, 2, 3];
    let obj = String::new();
    let a = 1;
    let b = 2;
    let c = 3;
}
"#;

    fs::write(&file_path, code_with_many_issues).expect("Failed to write test file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--issues",
            "2",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_cli_exclude_patterns() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create files that should be excluded
    let excluded_file = temp_dir.path().join("test_excluded.rs");
    let included_file = temp_dir.path().join("included.rs");

    fs::write(&excluded_file, "fn main() { let data = \"test\"; }")
        .expect("Failed to write excluded file");
    fs::write(&included_file, "fn main() { let temp = 42; }")
        .expect("Failed to write included file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--exclude",
            "test_*",
            temp_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // The excluded file should not appear in the analysis
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("test_excluded.rs"));
}

#[test]
fn test_cli_harsh_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");

    fs::write(&file_path, "fn main() { let data = \"test\"; }").expect("Failed to write test file");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--harsh",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_cli_nonexistent_file() {
    let output = Command::new("cargo")
        .args(["run", "--", "analyze", "nonexistent_file.rs"])
        .output()
        .expect("Failed to execute command");

    // Should handle nonexistent files gracefully
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show clean code message or handle gracefully
    assert!(
        stdout.contains("垃圾")
            || stdout.contains("Garbage")
            || stdout.contains("clean")
            || stdout.contains("Wow")
    );
}

#[test]
fn test_cli_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let output = Command::new("cargo")
        .args(["run", "--", "analyze", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should handle empty directories gracefully
    assert!(
        stdout.contains("垃圾")
            || stdout.contains("Garbage")
            || stdout.contains("clean")
            || stdout.contains("Wow")
    );
}

#[test]
fn test_cli_invalid_rust_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("invalid.rs");

    // Write invalid Rust code
    fs::write(&file_path, "this is not valid rust code { } } {")
        .expect("Failed to write invalid file");

    let output = Command::new("cargo")
        .args(["run", "--", "analyze", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should handle invalid Rust files gracefully
    assert!(output.status.success());
}

#[test]
fn test_cli_brief_mode() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("garbage.rs");
    fs::write(
        &file_path,
        "fn main() { let x = 42; let y = Some(1).unwrap(); }",
    )
    .expect("Failed to write test file");

    // Brief mode should skip boss file and mutation chains
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "analyze",
            "--brief",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "brief mode should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain personality and verdict (non-brief sections)
    assert!(
        stdout.contains("Personality") || stdout.contains("VERDICT"),
        "brief output should contain personality/verdict"
    );

    // Should NOT contain boss file section
    assert!(
        !stdout.contains("FINAL BOSS"),
        "brief output should not contain FINAL BOSS section"
    );
}
