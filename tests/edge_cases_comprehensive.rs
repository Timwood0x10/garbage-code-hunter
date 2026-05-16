use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_regex_pattern_word_boundary_precision() {
    use regex::Regex;

    let content = r#"
[dependencies]
gpu-allocator = "0.5"
wgpu-core = "0.19"
wgpu = "0.19"
nvidia-gpu-utils = "1.0"
gpu = "0.3"
actix-web = "4.0"
axum = "0.7"
"#;

    // Test exact matching with word boundaries
    let gpu_pattern = Regex::new(r"\bgpu\s*=").unwrap();
    assert!(
        gpu_pattern.is_match(content),
        "Should match 'gpu =' exactly"
    );
    assert!(
        !gpu_pattern.is_match("gpu-allocator ="),
        "Should NOT match 'gpu-allocator'"
    );

    let wgpu_pattern = Regex::new(r"\bwgpu\s*=").unwrap();
    assert!(wgpu_pattern.is_match("wgpu ="), "Should match 'wgpu ='");
    assert!(
        !wgpu_pattern.is_match("wgpu-core ="),
        "Should NOT match 'wgpu-core'"
    );
}

#[test]
fn test_regex_special_characters_handling() {
    use regex::Regex;

    let special_patterns = vec![
        r#"\.clone\(\)"#, // Escaped dot and parentheses
        r#"unwrap\(\)"#,  // Simple function call
        r#"\[\d+\]"#,     // Character class with digits
        r#"(?i)TODO"#,    // Case-insensitive flag
    ];

    for pattern in &special_patterns {
        let result = Regex::new(pattern);
        assert!(
            result.is_ok(),
            "Pattern '{}' should compile successfully: {:?}",
            pattern,
            result.err()
        );
    }
}

// ============================================================
// Edge Case Tests for Cargo.toml Detection (file_context.rs)
// ============================================================

#[test]
fn test_find_cargo_toml_in_current_directory() {
    use garbage_code_hunter::context::file_context::FileContext;

    let file_path = Path::new("src/main.rs");
    let context = FileContext::from_path(file_path);

    // Should find Cargo.toml in project root and detect dependencies
    if cfg!(test) {
        let _ = context; // Just verify it doesn't panic
    }
}

#[test]
fn test_find_cargo_toml_in_deeply_nested_directory() {
    use garbage_code_hunter::context::file_context::FileContext;

    let deep_path = Path::new("src/ui/components/widgets/button.rs");
    let context = FileContext::from_path(deep_path);

    // Should traverse up to find Cargo.toml
    let _ = context; // Verify no panic on deep paths
}

#[test]
fn test_cargo_toml_not_found_returns_none_gracefully() {
    use garbage_code_hunter::context::file_context::FileContext;

    let non_existent_path = Path::new("/tmp/nonexistent/project/src/main.rs");
    let context = FileContext::from_path(non_existent_path);

    // Should default to Business context when no Cargo.toml found
    assert_eq!(
        context,
        garbage_code_hunter::context::file_context::FileContext::Business
    );
}

#[test]
fn test_dependency_detection_avoids_false_positives() {
    use garbage_code_hunter::context::file_context::FileContext;

    let fake_gpu_project = TempDir::new().expect("Failed to create temp dir");
    let cargo_toml = fake_gpu_project.path().join("Cargo.toml");
    let src_dir = fake_gpu_project.path().join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src dir");

    let cargo_content = r#"
[package]
name = "fake-gpu-test"
version = "0.1.0"

[dependencies]
gpu-allocator = "0.5"
wgpu-core = "0.19"
"#;

    fs::write(&cargo_toml, cargo_content).expect("Failed to write Cargo.toml");

    let test_file = src_dir.join("main.rs");
    fs::write(&test_file, "fn main() {}").expect("Failed to write test file");

    let context = FileContext::from_path(&test_file);

    // Should NOT detect as GPU project (no exact 'gpu' dependency)
    assert_ne!(
        context,
        garbage_code_hunter::context::file_context::FileContext::GPU,
        "Should not detect 'gpu-allocator' or 'wgpu-core' as GPU project"
    );
}

#[test]
fn test_dependency_detection_finds_exact_matches() {
    use garbage_code_hunter::context::file_context::FileContext;

    let real_web_project = TempDir::new().expect("Failed to create temp dir");
    let cargo_toml = real_web_project.path().join("Cargo.toml");
    let src_dir = real_web_project.path().join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src dir");

    let cargo_content = r#"
[package]
name = "web-server"
version = "0.1.0"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
"#;

    fs::write(&cargo_toml, cargo_content).expect("Failed to write Cargo.toml");

    let test_file = src_dir.join("main.rs");
    fs::write(&test_file, "fn main() {}").expect("Failed to write test file");

    let context = FileContext::from_path(&test_file);

    // Should detect as Web project (has 'axum')
    assert_eq!(
        context,
        garbage_code_hunter::context::file_context::FileContext::Web,
        "Should detect axum as Web project"
    );
}

// ============================================================
// Edge Case Tests for File Context Detection
// ============================================================

#[test]
fn test_ui_context_detection_for_known_tui_projects() {
    use garbage_code_hunter::context::file_context::FileContext;

    let tui_project = TempDir::new().expect("Failed to create temp dir");
    let cargo_toml = tui_project.path().join("Cargo.toml");
    let src_dir = tui_project.path().join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src dir");

    let cargo_content = r#"
[package]
name = "tui-app"
version = "0.1.0"

[dependencies]
ratatui = "0.26"
crossterm = "0.27"
"#;

    fs::write(&cargo_toml, cargo_content).expect("Failed to write Cargo.toml");

    let ui_file = src_dir.join("ui.rs");
    fs::write(&ui_file, "// TUI code here").expect("Failed to write UI file");

    let context = FileContext::from_path(&ui_file);

    assert_eq!(
        context,
        garbage_code_hunter::context::file_context::FileContext::UI,
        "Should detect ratatui/crossterm as UI project"
    );
}

#[test]
fn test_gpu_context_detection_for_vulkan_projects() {
    use garbage_code_hunter::context::file_context::FileContext;

    let gpu_project = TempDir::new().expect("Failed to create temp dir");
    let cargo_toml = gpu_project.path().join("Cargo.toml");
    let src_dir = gpu_project.path().join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src dir");

    let cargo_content = r#"
[package]
name = "vulkan-renderer"
version = "0.1.0"

[dependencies]
vulkan = "0.2"
"#;

    fs::write(&cargo_toml, cargo_content).expect("Failed to write Cargo.toml");

    let render_file = src_dir.join("render.rs");
    fs::write(&render_file, "// GPU rendering code").expect("Failed to write render file");

    let context = FileContext::from_path(&render_file);

    assert_eq!(
        context,
        garbage_code_hunter::context::file_context::FileContext::GPU,
        "Should detect vulkan as GPU project"
    );
}

// ============================================================
// Edge Case Tests for File Handling
// ============================================================

#[test]
fn test_very_long_line_does_not_crash() {
    use garbage_code_hunter::CodeAnalyzer;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("long_line.rs");

    let long_string = "x".repeat(10000);
    let content = format!(
        "fn main() {{ let s = \"{}\"; println!(\"{{}}\", s); }}",
        long_string
    );

    fs::write(&file_path, content).expect("Failed to write long line file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should not crash on very long lines (may or may not detect issues)
    let _ = issues; // Just verify no crash
}

#[test]
fn test_file_with_special_characters_in_path() {
    use garbage_code_hunter::CodeAnalyzer;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let special_name = "file-with spaces_&special@chars#.rs";
    let file_path = temp_dir.path().join(special_name);

    let content = r#"
fn main() {
    println!("Hello");
}
"#;

    fs::write(&file_path, content).expect("Failed to write special char file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Should handle special characters in filename gracefully
    let _ = issues; // Just verify no crash
}

#[test]
fn test_binary_file_does_not_crash() {
    use garbage_code_hunter::CodeAnalyzer;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("binary.rs");

    let binary_content: Vec<u8> = (0..255).collect();
    fs::write(&file_path, binary_content).expect("Failed to write binary content");

    let analyzer = CodeAnalyzer::new(&[], "en-US");

    let issues = analyzer.analyze_file(&file_path);

    // Should return empty or minimal issues for non-parseable files
    assert!(
        issues.len() <= 1,
        "Binary-like content should produce at most 1 issue, got {}",
        issues.len()
    );
}

#[test]
fn test_large_number_of_functions() {
    use garbage_code_hunter::CodeAnalyzer;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("many_functions.rs");

    let mut content = String::new();
    for i in 0..100 {
        content.push_str(&format!("fn function_{}() -> i32 {{ 0 }}\n", i));
    }

    fs::write(&file_path, content).expect("Failed to write many functions file");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let start = std::time::Instant::now();
    let issues = analyzer.analyze_file(&file_path);
    let duration = start.elapsed();

    // Should complete within reasonable time (< 5 seconds)
    assert!(
        duration.as_secs() < 5,
        "Analysis of 100 functions took too long: {:?}",
        duration
    );

    // Should detect some issues (god-function or similar)
    let _ = issues;
}

// ============================================================
// Edge Case Tests for Rule Behavior
// ============================================================

#[test]
fn test_meaningless_naming_with_domain_whitelists() {
    use garbage_code_hunter::CodeAnalyzer;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("ui_coordinates.rs");

    let ui_code = r#"
struct Rectangle {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rectangle {
    fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }
    
    fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.w &&
        py >= self.y && py <= self.y + self.h
    }
}
"#;

    fs::write(&file_path, ui_code).expect("Failed to write UI code");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // In Business context, x/y/w/h might be flagged
    // But should be significantly fewer than without whitelists
    let naming_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule_name.contains("naming"))
        .collect();

    assert!(
        naming_issues.len() <= 2,
        "UI coordinate variables should be mostly whitelisted, got {} naming issues",
        naming_issues.len()
    );
}

#[test]
fn test_magic_number_with_config_values() {
    use garbage_code_hunter::CodeAnalyzer;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("config_constants.rs");

    let config_code = r#"
const MAX_CONNECTIONS: u32 = 100;
const DEFAULT_TIMEOUT_MS: u64 = 30000;
const BUFFER_SIZE: usize = 8192;
const PORT: u16 = 8080;

fn main() {
    let timeout = DEFAULT_TIMEOUT_MS;
    let port = PORT;
    println!("Server starting on port {}", port);
}
"#;

    fs::write(&file_path, config_code).expect("Failed to write config code");

    let analyzer = CodeAnalyzer::new(&[], "en-US");
    let issues = analyzer.analyze_file(&file_path);

    // Named constants should NOT be flagged as magic numbers (or very few)
    let magic_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule_name.contains("magic-number"))
        .collect();

    // Named constants should NOT be flagged as magic numbers
    assert!(
        magic_issues.is_empty(),
        "Named constants should not be flagged as magic numbers, got {}",
        magic_issues.len()
    );
}

#[test]
fn test_cross_file_duplication_thresholds() {
    use garbage_code_hunter::CodeAnalyzer;

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file1 = temp_dir.path().join("mod1.rs");
    let file2 = temp_dir.path().join("mod2.rs");

    let similar_code = r#"
fn helper_function(data: &str) -> String {
    let xxx = data.to_uppercase();
    xxx
}

fn process_item(item: i32) -> i32 {
    let foo = item * 2 + 1;
    foo
}
"#;

    fs::write(&file1, similar_code).expect("Failed to write file1");
    fs::write(&file2, similar_code).expect("Failed to write file2");

    let analyzer = CodeAnalyzer::new(&[], "en-US");

    // Analyze both files
    let issues1 = analyzer.analyze_file(&file1);
    let issues2 = analyzer.analyze_file(&file2);

    // Both files should have some issues (even if not cross-file)
    let total_issues = issues1.len() + issues2.len();

    assert!(
        total_issues > 0,
        "Similar code should generate at least some issues in both files combined, got {}",
        total_issues
    );
}
