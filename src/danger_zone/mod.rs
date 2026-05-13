//! Danger Zone — identify the most dangerous files in the codebase.

use crate::analyzer::{CodeAnalyzer, CodeIssue};
use crate::common::i18n_ext::t;
use crate::common::OutputFormat;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A dangerous file entry.
#[derive(Debug, Clone)]
pub struct DangerEntry {
    pub file: PathBuf,
    pub risk_score: f64,
    pub risk_level: &'static str,
    pub issue_count: usize,
    pub churn: usize,
    pub contributors: usize,
    pub reasons: Vec<String>,
}

/// Run danger zone analysis.
pub fn run(path: &Path, format: &OutputFormat, lang: &str) -> Result<String> {
    let analyzer = CodeAnalyzer::new(&[], lang);
    let issues = analyzer.analyze_path(path);

    // Group issues by file
    let mut by_file: HashMap<PathBuf, Vec<&CodeIssue>> = HashMap::new();
    for issue in &issues {
        by_file
            .entry(issue.file_path.clone())
            .or_default()
            .push(issue);
    }

    // Get git churn data (optional)
    let churn_data = get_churn_data(path);
    let blame_data = get_contributor_counts(path);

    let mut entries: Vec<DangerEntry> = Vec::new();

    for (file, file_issues) in &by_file {
        let issue_count = file_issues.len();
        let churn = churn_data.get(file).copied().unwrap_or(0);
        let contributors = blame_data.get(file).copied().unwrap_or(1);

        // Calculate risk score
        let issue_score = (issue_count as f64 * 3.0).min(50.0);
        let churn_score = (churn as f64 * 0.5).min(30.0);
        let contributor_score = (contributors as f64 * 2.0).min(20.0);
        let risk_score = issue_score + churn_score + contributor_score;

        let risk_level = match risk_score as u32 {
            0..=10 => "LOW",
            11..=30 => "MEDIUM",
            31..=60 => "HIGH",
            _ => "EXTREME",
        };

        let mut reasons = Vec::new();
        if issue_count > 5 {
            reasons.push(format!("{} code issues", issue_count));
        }
        if churn > 20 {
            reasons.push(format!("modified {} times recently", churn));
        }
        if contributors > 5 {
            reasons.push(format!("{} authors touched this file", contributors));
        }

        entries.push(DangerEntry {
            file: file.clone(),
            risk_score,
            risk_level,
            issue_count,
            churn,
            contributors,
            reasons,
        });
    }

    // Sort by risk score descending
    entries.sort_by(|a, b| b.risk_score.partial_cmp(&a.risk_score).unwrap());

    let output = match format {
        OutputFormat::Terminal => format_terminal(&entries, lang),
        OutputFormat::Json => format_json(&entries),
    };

    Ok(output)
}

fn format_terminal(entries: &[DangerEntry], lang: &str) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n",
        t(lang, "\u{1f525} 危险区域", "\u{1f525} Danger Zone").bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    if entries.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            t(lang, "没有分析到文件。", "No files analyzed.")
        ));
        return out;
    }

    for (i, entry) in entries.iter().take(10).enumerate() {
        let risk_colored = match entry.risk_level {
            "EXTREME" => entry.risk_level.red().bold(),
            "HIGH" => entry.risk_level.red(),
            "MEDIUM" => entry.risk_level.yellow(),
            _ => entry.risk_level.green(),
        };
        let file_short = entry
            .file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.file.display().to_string());

        out.push_str(&format!(
            "  #{} {} [Risk: {}]\n",
            i + 1,
            file_short.bold(),
            risk_colored
        ));
        out.push_str(&format!("     Score: {:.0}/100\n", entry.risk_score));
        for reason in &entry.reasons {
            out.push_str(&format!("     \u{2022} {}\n", reason.dimmed()));
        }
        out.push('\n');
    }

    out
}

fn format_json(entries: &[DangerEntry]) -> String {
    serde_json::json!({
        "files": entries.iter().take(10).map(|e| {
            serde_json::json!({
                "file": e.file.display().to_string(),
                "risk_score": e.risk_score,
                "risk_level": e.risk_level,
                "issue_count": e.issue_count,
                "churn": e.churn,
                "contributors": e.contributors,
                "reasons": e.reasons,
            })
        }).collect::<Vec<_>>(),
    })
    .to_string()
}

/// Get file churn (number of commits touching each file) from git log.
fn get_churn_data(path: &Path) -> HashMap<PathBuf, usize> {
    let mut churn = HashMap::new();

    let output = match std::process::Command::new("git")
        .args(["log", "--name-only", "--pretty=format:", "-30"])
        .current_dir(path)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return churn,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *churn.entry(PathBuf::from(trimmed)).or_insert(0) += 1;
        }
    }

    churn
}

/// Get unique contributor counts per file from git blame.
fn get_contributor_counts(path: &Path) -> HashMap<PathBuf, usize> {
    let mut counts = HashMap::new();

    // Get list of tracked files
    let output = match std::process::Command::new("git")
        .args(["ls-files"])
        .current_dir(path)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return counts,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    for file_line in stdout.lines().take(50) {
        let file_path = path.join(file_line.trim());
        if !file_line.trim().ends_with(".rs") {
            continue;
        }

        // Get unique authors for this file
        let blame_output = match std::process::Command::new("git")
            .args(["blame", "--line-porcelain", file_line.trim()])
            .current_dir(path)
            .output()
        {
            Ok(o) if o.status.success() => o,
            _ => continue,
        };

        let blame_stdout = String::from_utf8_lossy(&blame_output.stdout);
        let mut authors = std::collections::HashSet::new();
        for blame_line in blame_stdout.lines() {
            if let Some(author) = blame_line.strip_prefix("author ") {
                authors.insert(author.to_string());
            }
        }

        counts.insert(file_path, authors.len());
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_on_current_dir() {
        let result = run(std::path::Path::new("."), &OutputFormat::Terminal, "en-US");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_on_current_dir_chinese() {
        let result = run(std::path::Path::new("."), &OutputFormat::Terminal, "zh-CN");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_json_format() {
        let result = run(std::path::Path::new("."), &OutputFormat::Json, "en-US");
        assert!(result.is_ok());
    }
}
