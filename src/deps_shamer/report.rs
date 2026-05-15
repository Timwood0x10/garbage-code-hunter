//! Report generation for dependency analysis.

use super::types::{DepFile, DepIssue, Severity};
use colored::Colorize;

/// Summary statistics for a dependency analysis.
#[derive(Debug)]
pub struct DepStats {
    pub total_deps: usize,
    pub dev_deps: usize,
    pub optional_deps: usize,
    pub git_deps: usize,
    pub issue_count: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub score: f64,
}

/// Build statistics from dependency files and issues.
pub fn build_stats(dep_files: &[DepFile], issues: &[DepIssue]) -> DepStats {
    let total_deps: usize = dep_files.iter().map(|f| f.dependencies.len()).sum();
    let dev_deps: usize = dep_files
        .iter()
        .flat_map(|f| &f.dependencies)
        .filter(|d| d.is_dev)
        .count();
    let optional_deps: usize = dep_files
        .iter()
        .flat_map(|f| &f.dependencies)
        .filter(|d| d.is_optional)
        .count();
    let git_deps: usize = dep_files
        .iter()
        .flat_map(|f| &f.dependencies)
        .filter(|d| matches!(d.source, super::types::DepSource::Git { .. }))
        .count();

    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;

    for issue in issues {
        match issue.severity {
            Severity::Critical => critical_count += 1,
            Severity::High => high_count += 1,
            Severity::Medium => medium_count += 1,
            Severity::Low => low_count += 1,
            Severity::Info => {}
        }
    }

    let penalty: f64 = issues.iter().map(|i| i.severity.penalty()).sum();
    let score = (100.0 - penalty).max(0.0);

    DepStats {
        total_deps,
        dev_deps,
        optional_deps,
        git_deps,
        issue_count: issues.len(),
        critical_count,
        high_count,
        medium_count,
        low_count,
        score,
    }
}

/// Format analysis report for terminal output.
pub fn format_terminal(dep_files: &[DepFile], issues: &[DepIssue]) -> String {
    let stats = build_stats(dep_files, issues);
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n",
        "\u{1f4e6} Dependency Shame Report \u{1f4e6}".bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    // File summaries
    for dep_file in dep_files {
        out.push_str(&format!(
            "  {} {}: {} dependencies\n",
            "\u{1f4c1}",
            dep_file.ecosystem.display_name().cyan(),
            dep_file.dependencies.len()
        ));
    }
    out.push('\n');

    // Group issues by severity
    let mut by_severity: Vec<(&Severity, &DepIssue)> =
        issues.iter().map(|i| (&i.severity, i)).collect();
    by_severity.sort_by(|a, b| a.0.cmp(b.0));

    if !issues.is_empty() {
        // Critical issues
        let critical: Vec<_> = by_severity
            .iter()
            .filter(|(s, _)| **s == Severity::Critical)
            .collect();
        if !critical.is_empty() {
            out.push_str(&format!(
                "{} {} ({})\n",
                "\u{1f480}",
                "Critical".red().bold(),
                critical.len()
            ));
            for (_, issue) in &critical {
                let dep_info = issue
                    .dep_name
                    .as_ref()
                    .map(|n| format!(" [{}]", n))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  {} {}{}\n",
                    "\u{2022}",
                    issue.message,
                    dep_info.dimmed()
                ));
            }
            out.push('\n');
        }

        // High issues
        let high: Vec<_> = by_severity
            .iter()
            .filter(|(s, _)| **s == Severity::High)
            .collect();
        if !high.is_empty() {
            out.push_str(&format!(
                "{} {} ({})\n",
                "\u{1f621}",
                "High".red(),
                high.len()
            ));
            for (_, issue) in &high {
                let dep_info = issue
                    .dep_name
                    .as_ref()
                    .map(|n| format!(" [{}]", n))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  {} {}{}\n",
                    "\u{2022}",
                    issue.message,
                    dep_info.dimmed()
                ));
            }
            out.push('\n');
        }

        // Medium issues
        let medium: Vec<_> = by_severity
            .iter()
            .filter(|(s, _)| **s == Severity::Medium)
            .collect();
        if !medium.is_empty() {
            out.push_str(&format!(
                "{} {} ({})\n",
                "\u{26a0}\u{fe0f}",
                "Medium".yellow(),
                medium.len()
            ));
            for (_, issue) in &medium {
                let dep_info = issue
                    .dep_name
                    .as_ref()
                    .map(|n| format!(" [{}]", n))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  {} {}{}\n",
                    "\u{2022}",
                    issue.message,
                    dep_info.dimmed()
                ));
            }
            out.push('\n');
        }

        // Low issues
        let low: Vec<_> = by_severity
            .iter()
            .filter(|(s, _)| **s == Severity::Low)
            .collect();
        if !low.is_empty() {
            out.push_str(&format!(
                "{} {} ({})\n",
                "\u{1f4a7}",
                "Low".blue(),
                low.len()
            ));
            for (_, issue) in &low {
                let dep_info = issue
                    .dep_name
                    .as_ref()
                    .map(|n| format!(" [{}]", n))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  {} {}{}\n",
                    "\u{2022}",
                    issue.message,
                    dep_info.dimmed()
                ));
            }
            out.push('\n');
        }
    }

    // Statistics
    out.push_str(&format!("{}\n", "\u{1f4ca} Statistics".bold()));
    out.push_str(&format!("{}\n", "\u{2500}".repeat(30)));
    out.push_str(&format!(
        "  Total dependencies: {}\n",
        stats.total_deps.to_string().cyan()
    ));
    out.push_str(&format!(
        "  Dev dependencies:   {}\n",
        stats.dev_deps.to_string().cyan()
    ));
    out.push_str(&format!(
        "  Optional deps:      {}\n",
        stats.optional_deps.to_string().cyan()
    ));
    out.push_str(&format!(
        "  Git dependencies:   {}\n",
        stats.git_deps.to_string().yellow()
    ));
    out.push_str(&format!(
        "  Issues found:       {}\n",
        stats.issue_count.to_string().red()
    ));

    // Score
    out.push('\n');
    let score_str = if stats.score >= 80.0 {
        format!("{:.0}/100", stats.score).green().bold()
    } else if stats.score >= 60.0 {
        format!("{:.0}/100", stats.score).yellow().bold()
    } else {
        format!("{:.0}/100", stats.score).red().bold()
    };
    out.push_str(&format!(
        "{} Dependency Health Score: {}\n",
        "\u{1f3af}", score_str
    ));

    if issues.is_empty() {
        out.push_str(&format!(
            "\n{}\n",
            "\u{2728} No dependency issues found. Your deps are clean!"
                .green()
                .bold()
        ));
    }

    out
}

/// Format analysis report as JSON.
pub fn format_json(dep_files: &[DepFile], issues: &[DepIssue]) -> String {
    let stats = build_stats(dep_files, issues);
    let json_output = serde_json::json!({
        "score": stats.score,
        "total_deps": stats.total_deps,
        "dev_deps": stats.dev_deps,
        "optional_deps": stats.optional_deps,
        "git_deps": stats.git_deps,
        "issues": issues.iter().map(|i| {
            serde_json::json!({
                "rule_id": i.rule_id,
                "severity": format!("{:?}", i.severity),
                "message": i.message,
                "dep_name": i.dep_name,
            })
        }).collect::<Vec<_>>(),
        "files": dep_files.iter().map(|f| {
            serde_json::json!({
                "path": f.path,
                "ecosystem": format!("{:?}", f.ecosystem),
                "dependency_count": f.dependencies.len(),
            })
        }).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::super::types::{DepSource, Dependency, Ecosystem};
    use super::*;

    fn sample_dep_file() -> DepFile {
        DepFile {
            path: "Cargo.toml".to_string(),
            ecosystem: Ecosystem::Rust,
            dependencies: vec![
                Dependency {
                    name: "serde".to_string(),
                    version: "1.0".to_string(),
                    source: DepSource::Registry,
                    is_dev: false,
                    is_optional: false,
                },
                Dependency {
                    name: "tempfile".to_string(),
                    version: "3.0".to_string(),
                    source: DepSource::Registry,
                    is_dev: true,
                    is_optional: false,
                },
            ],
        }
    }

    #[test]
    fn test_build_stats_counts() {
        let dep_file = sample_dep_file();
        let issues = vec![
            DepIssue {
                rule_id: "test".to_string(),
                severity: Severity::High,
                message: "test".to_string(),
                dep_name: None,
            },
            DepIssue {
                rule_id: "test".to_string(),
                severity: Severity::Low,
                message: "test".to_string(),
                dep_name: None,
            },
        ];

        let stats = build_stats(&[dep_file], &issues);
        assert_eq!(stats.total_deps, 2);
        assert_eq!(stats.dev_deps, 1);
        assert_eq!(stats.issue_count, 2);
        assert_eq!(stats.high_count, 1);
        assert_eq!(stats.low_count, 1);
        assert!(stats.score < 100.0);
    }

    #[test]
    fn test_format_terminal_empty() {
        let dep_file = DepFile {
            path: "Cargo.toml".to_string(),
            ecosystem: Ecosystem::Rust,
            dependencies: vec![],
        };
        let output = format_terminal(&[dep_file], &[]);
        assert!(output.contains("Dependency Shame Report"));
        assert!(output.contains("No dependency issues found"));
    }

    #[test]
    fn test_format_terminal_with_issues() {
        let dep_file = sample_dep_file();
        let issues = vec![DepIssue {
            rule_id: "wildcard-version".to_string(),
            severity: Severity::High,
            message: "Version '*' for 'tokio'".to_string(),
            dep_name: Some("tokio".to_string()),
        }];
        let output = format_terminal(&[dep_file], &issues);
        assert!(output.contains("High"));
        assert!(output.contains("tokio"));
    }

    #[test]
    fn test_format_json_valid() {
        let dep_file = sample_dep_file();
        let issues = vec![DepIssue {
            rule_id: "test".to_string(),
            severity: Severity::Medium,
            message: "test issue".to_string(),
            dep_name: Some("serde".to_string()),
        }];
        let json = format_json(&[dep_file], &issues);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["score"].as_f64().is_some());
        assert!(parsed["issues"].as_array().unwrap().len() == 1);
    }
}
