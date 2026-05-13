//! Technical Debt Invoice — estimate the cost of your code's problems.

pub mod cost_model;
pub mod display;

use crate::analyzer::CodeAnalyzer;
use crate::common::OutputFormat;
use anyhow::Result;
use std::path::Path;

/// Run the debt invoice analysis on a path.
pub fn run(path: &Path, format: &OutputFormat, lang: &str) -> Result<String> {
    let analyzer = CodeAnalyzer::new(&[], lang);
    let issues = analyzer.analyze_path(path);

    // Count total lines
    let total_lines = count_lines(path);
    let invoice = cost_model::generate_invoice(&issues, total_lines);

    let output = match format {
        OutputFormat::Terminal => display::format_terminal(&invoice, lang),
        OutputFormat::Json => display::format_json(&invoice),
    };

    Ok(output)
}

fn count_lines(path: &Path) -> usize {
    if path.is_file() {
        return std::fs::read_to_string(path)
            .map(|c| c.lines().count())
            .unwrap_or(0);
    }

    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .map(|c| c.lines().count())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_on_current_dir() {
        let result = run(std::path::Path::new("."), &OutputFormat::Terminal, "en-US");
        assert!(result.is_ok());
    }
}
