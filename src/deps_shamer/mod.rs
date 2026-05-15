//! Dependency Shame module.
//!
//! Scans project dependency files, identifies problematic patterns,
//! and generates a roasting report about dependency management quality.
//!
//! Supported ecosystems: Rust (Cargo.toml), Node (package.json),
//! Go (go.mod), Python (requirements.txt, pyproject.toml).

pub mod parser;
pub mod report;
pub mod rules;
pub mod types;

use anyhow::Result;
use std::path::Path;

use parser::detect_and_parse;
use report::{format_json, format_terminal};
use rules::check_dep_file;
use types::DepFile;

pub use crate::common::OutputFormat;

/// Run dependency analysis on the given project path.
///
/// Detects dependency files, parses them, applies rules,
/// and returns a formatted report string.
pub fn run(project_path: &Path, format: &OutputFormat) -> Result<String> {
    let dep_files = detect_and_parse(project_path)?;

    if dep_files.is_empty() {
        return match format {
            OutputFormat::Terminal => Ok(format!(
                "\n{}\n\n  No dependency files found in {}\n\n  Supported: Cargo.toml, package.json, go.mod, requirements.txt, pyproject.toml\n",
                "\u{1f4e6} Dependency Shame Report \u{1f4e6}",
                project_path.display()
            )),
            OutputFormat::Json => Ok(r#"{"score":100,"total_deps":0,"issues":[],"files":[]}"#.to_string()),
        };
    }

    let all_issues: Vec<_> = dep_files.iter().flat_map(check_dep_file).collect();

    let output = match format {
        OutputFormat::Terminal => format_terminal(&dep_files, &all_issues),
        OutputFormat::Json => format_json(&dep_files, &all_issues),
    };

    Ok(output)
}

/// Run analysis and return structured data (for CLI integration).
pub fn analyze(project_path: &Path) -> Result<(Vec<DepFile>, Vec<types::DepIssue>)> {
    let dep_files = detect_and_parse(project_path)?;
    let all_issues: Vec<_> = dep_files.iter().flat_map(check_dep_file).collect();
    Ok((dep_files, all_issues))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_run_no_dep_files() {
        let dir = TempDir::new().unwrap();
        let output = run(dir.path(), &OutputFormat::Terminal).unwrap();
        assert!(output.contains("No dependency files found"));
    }

    #[test]
    fn test_run_no_dep_files_json() {
        let dir = TempDir::new().unwrap();
        let output = run(dir.path(), &OutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_deps"], 0);
    }

    #[test]
    fn test_run_with_cargo_toml() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"
[dependencies]
serde = "1.0"
my-git-crate = { git = "https://github.com/foo/bar" }
wildcard = "*"
"#,
        )
        .unwrap();

        let output = run(dir.path(), &OutputFormat::Terminal).unwrap();
        assert!(output.contains("Dependency Shame Report"));
        // Should detect wildcard and git dep issues
        assert!(output.contains("wildcard") || output.contains("git"));
    }

    #[test]
    fn test_run_json_format() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();

        let output = run(dir.path(), &OutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["score"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_analyze_returns_structured_data() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();

        let (dep_files, issues) = analyze(dir.path()).unwrap();
        assert_eq!(dep_files.len(), 1);
        assert_eq!(dep_files[0].ecosystem, types::Ecosystem::Rust);
        assert_eq!(dep_files[0].dependencies.len(), 1);
        // serde = "1.0" should not trigger any issues
        assert!(issues.is_empty());
    }

    #[test]
    fn test_run_multiple_ecosystems() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nserde = \"1.0\"\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies": {"express": "^4.0"}}"#,
        )
        .unwrap();

        let (dep_files, _) = analyze(dir.path()).unwrap();
        assert_eq!(dep_files.len(), 2);
        assert!(dep_files
            .iter()
            .any(|f| f.ecosystem == types::Ecosystem::Rust));
        assert!(dep_files
            .iter()
            .any(|f| f.ecosystem == types::Ecosystem::Node));
    }
}
