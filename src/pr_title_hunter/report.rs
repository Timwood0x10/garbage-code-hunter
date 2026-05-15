//! Report generation for PR title analysis.

use super::types::{PrEntry, PrIssue, Severity};
use colored::Colorize;

/// Build statistics from PR analysis.
pub struct PrStats {
    pub total_prs: usize,
    pub issues_found: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub avg_title_length: f64,
    pub score: f64,
}

pub fn build_stats(prs: &[PrEntry], issues: &[PrIssue]) -> PrStats {
    let total_prs = prs.len();
    let issues_found = issues.len();

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

    let avg_title_length = if total_prs > 0 {
        prs.iter().map(|p| p.title.len()).sum::<usize>() as f64 / total_prs as f64
    } else {
        0.0
    };

    let penalty: f64 = issues.iter().map(|i| i.severity.penalty()).sum();
    let score = (100.0 - penalty).max(0.0);

    PrStats {
        total_prs,
        issues_found,
        critical_count,
        high_count,
        medium_count,
        low_count,
        avg_title_length,
        score,
    }
}

/// Format analysis report for terminal output.
pub fn format_terminal(prs: &[PrEntry], issues: &[PrIssue]) -> String {
    let stats = build_stats(prs, issues);
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n",
        "\u{1f3af} PR Title Roast Report \u{1f3af}".bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    out.push_str(&format!(
        "  Checked {} PRs\n\n",
        stats.total_prs.to_string().cyan()
    ));

    if issues.is_empty() {
        out.push_str(&format!(
            "{}\n",
            "\u{2728} All PR titles look good! No issues found."
                .green()
                .bold()
        ));
        return out;
    }

    // Group issues by severity and show them
    let mut by_severity: Vec<(&Severity, &PrIssue)> =
        issues.iter().map(|i| (&i.severity, i)).collect();
    by_severity.sort_by(|a, b| a.0.cmp(b.0));

    let severity_groups = [
        (Severity::Critical, "Critical"),
        (Severity::High, "High"),
        (Severity::Medium, "Medium"),
        (Severity::Low, "Low"),
        (Severity::Info, "Info"),
    ];

    for (sev, label) in &severity_groups {
        let group: Vec<_> = by_severity.iter().filter(|(s, _)| **s == *sev).collect();
        if group.is_empty() {
            continue;
        }

        let header = match sev {
            Severity::Critical => {
                format!("{} {} ({})", sev.emoji(), label.red().bold(), group.len())
            }
            Severity::High => format!("{} {} ({})", sev.emoji(), label.red(), group.len()),
            Severity::Medium => format!("{} {} ({})", sev.emoji(), label.yellow(), group.len()),
            Severity::Low => format!("{} {} ({})", sev.emoji(), label.blue(), group.len()),
            Severity::Info => format!("{} {} ({})", sev.emoji(), label.dimmed(), group.len()),
        };
        out.push_str(&format!("{}\n", header));

        for (_, issue) in &group {
            out.push_str(&format!(
                "  {} #{}: \"{}\"\n",
                "\u{2022}",
                issue.pr_id.cyan(),
                issue.pr_title
            ));
            out.push_str(&format!("    {}\n", issue.message.dimmed()));
        }
        out.push('\n');
    }

    // Worst PRs (sorted by issue count)
    let mut pr_issue_counts: std::collections::HashMap<&str, Vec<&PrIssue>> =
        std::collections::HashMap::new();
    for issue in issues {
        pr_issue_counts.entry(&issue.pr_id).or_default().push(issue);
    }
    let mut worst: Vec<_> = pr_issue_counts.iter().collect();
    worst.sort_by_key(|b| std::cmp::Reverse(b.1.len()));

    if !worst.is_empty() {
        out.push_str(&format!("{}\n", "\u{1f3c6} Worst PR Titles".bold()));
        out.push_str(&format!("{}\n", "\u{2500}".repeat(30)));
        for (i, (pr_id, pr_issues)) in worst.iter().take(5).enumerate() {
            let title = pr_issues.first().map(|i| i.pr_title.as_str()).unwrap_or("");
            out.push_str(&format!(
                "  {}. #{}: \"{}\" — {} issues\n",
                i + 1,
                pr_id.cyan(),
                title,
                pr_issues.len().to_string().red()
            ));
        }
        out.push('\n');
    }

    // Statistics
    out.push_str(&format!("{}\n", "\u{1f4ca} Statistics".bold()));
    out.push_str(&format!("{}\n", "\u{2500}".repeat(30)));
    out.push_str(&format!(
        "  Average title length: {:.0} chars\n",
        stats.avg_title_length
    ));
    out.push_str(&format!(
        "  Issues found:        {}\n",
        stats.issues_found.to_string().red()
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
    out.push_str(&format!("{} PR Title Health: {}\n", "\u{1f3af}", score_str));

    out
}

/// Format analysis report as JSON.
pub fn format_json(prs: &[PrEntry], issues: &[PrIssue]) -> String {
    let stats = build_stats(prs, issues);
    let output = serde_json::json!({
        "score": stats.score,
        "total_prs": stats.total_prs,
        "issues": issues.iter().map(|i| {
            serde_json::json!({
                "rule_id": i.rule_id,
                "severity": format!("{:?}", i.severity),
                "message": i.message,
                "pr_id": i.pr_id,
                "pr_title": i.pr_title,
            })
        }).collect::<Vec<_>>(),
        "stats": {
            "avg_title_length": stats.avg_title_length,
            "issues_found": stats.issues_found,
        }
    });

    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::super::types::PrSource;
    use super::*;

    fn make_pr(id: &str, title: &str) -> PrEntry {
        PrEntry {
            id: id.to_string(),
            title: title.to_string(),
            author: None,
            source: PrSource::Local,
        }
    }

    fn make_issue(pr_id: &str, title: &str, rule_id: &str, severity: Severity) -> PrIssue {
        PrIssue {
            rule_id: rule_id.to_string(),
            severity,
            message: "test issue".to_string(),
            pr_id: pr_id.to_string(),
            pr_title: title.to_string(),
        }
    }

    #[test]
    fn test_build_stats_counts() {
        let prs = vec![make_pr("1", "title"), make_pr("2", "title")];
        let issues = vec![
            make_issue("1", "title", "test", Severity::High),
            make_issue("2", "title", "test", Severity::Low),
        ];
        let stats = build_stats(&prs, &issues);
        assert_eq!(stats.total_prs, 2);
        assert_eq!(stats.issues_found, 2);
        assert_eq!(stats.high_count, 1);
        assert_eq!(stats.low_count, 1);
    }

    #[test]
    fn test_format_terminal_empty() {
        let prs = vec![make_pr("1", "feat: good title")];
        let output = format_terminal(&prs, &[]);
        assert!(output.contains("All PR titles look good"));
    }

    #[test]
    fn test_format_terminal_with_issues() {
        let prs = vec![make_pr("1", "fix")];
        let issues = vec![make_issue("1", "fix", "generic-title", Severity::High)];
        let output = format_terminal(&prs, &issues);
        assert!(output.contains("High"));
        assert!(output.contains("fix"));
    }

    #[test]
    fn test_format_json_valid() {
        let prs = vec![make_pr("1", "title")];
        let issues = vec![make_issue("1", "title", "test", Severity::Medium)];
        let json = format_json(&prs, &issues);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["score"].as_f64().is_some());
    }
}
