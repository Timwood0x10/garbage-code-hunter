//! Report generation for commit-roaster.
//!
//! Generates formatted output with issues, stats, and roasts.

use crate::commit_roaster::analyzer::CommitInfo;
use crate::commit_roaster::rules::Issue;
use crate::common::score_to_grade;
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;

/// Aggregated statistics about analyzed commits.
#[derive(Debug, Clone, Serialize)]
pub struct CommitStats {
    pub total_commits: usize,
    pub total_issues: usize,
    pub avg_message_length: f64,
    pub empty_messages: usize,
    pub single_word_messages: usize,
    pub wip_commits: usize,
    pub fix_only_commits: usize,
}

/// A commit paired with its detected issues.
#[derive(Debug, Clone)]
pub struct ScoredCommit {
    pub commit: CommitInfo,
    pub issues: Vec<Issue>,
}

/// Full analysis report.
#[derive(Debug, Clone)]
pub struct Report {
    pub scored_commits: Vec<ScoredCommit>,
    pub stats: CommitStats,
    pub total_score: f64,
}

/// Build a report from scored commits.
pub fn build_report(scored_commits: Vec<ScoredCommit>) -> Report {
    let total_commits = scored_commits.len();
    let mut total_issues = 0;
    let mut total_msg_len = 0usize;
    let mut empty_messages = 0usize;
    let mut single_word_messages = 0usize;
    let mut wip_commits = 0usize;
    let mut fix_only_commits = 0usize;

    for sc in &scored_commits {
        total_issues += sc.issues.len();
        let msg = sc.commit.message.trim();
        total_msg_len += msg.len();

        if msg.is_empty() {
            empty_messages += 1;
        }
        if msg.split_whitespace().count() == 1 && !msg.is_empty() {
            single_word_messages += 1;
        }
        if msg.to_uppercase().starts_with("WIP") {
            wip_commits += 1;
        }
        if msg.to_lowercase() == "fix" {
            fix_only_commits += 1;
        }
    }

    let avg_message_length = if total_commits > 0 {
        total_msg_len as f64 / total_commits as f64
    } else {
        0.0
    };

    // Score: start at 100, subtract penalties
    let penalty: f64 = scored_commits
        .iter()
        .flat_map(|sc| &sc.issues)
        .map(|i| i.severity.weight())
        .sum();
    let total_score = (100.0 - penalty).max(0.0);

    let stats = CommitStats {
        total_commits,
        total_issues,
        avg_message_length,
        empty_messages,
        single_word_messages,
        wip_commits,
        fix_only_commits,
    };

    Report {
        scored_commits,
        stats,
        total_score,
    }
}

/// Format report as colored terminal output.
pub fn format_terminal(report: &Report) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "\n{}\n",
        "\u{1f525} Commit Roast Report \u{1f525}".bold().red()
    ));
    out.push_str(&format!("{}\n", "\u{2501}".repeat(40)));
    out.push_str(&format!(
        "Scanned {} commits, found {} issues\n\n",
        report.stats.total_commits.to_string().yellow(),
        report.stats.total_issues.to_string().red()
    ));

    // Group issues by severity
    let mut by_severity: HashMap<&str, Vec<&ScoredCommit>> = HashMap::new();
    for sc in &report.scored_commits {
        if sc.issues.is_empty() {
            continue;
        }
        let max_sev = sc
            .issues
            .iter()
            .max_by(|a, b| {
                a.severity
                    .weight()
                    .partial_cmp(&b.severity.weight())
                    .unwrap()
            })
            .unwrap();
        by_severity
            .entry(max_sev.severity.label())
            .or_default()
            .push(sc);
    }

    // Print by severity
    let order = ["Critical", "High", "Medium", "Low", "Info"];
    for sev_label in &order {
        if let Some(commits) = by_severity.get(*sev_label) {
            let emoji = match *sev_label {
                "Critical" => "\u{1f480}",
                "High" => "\u{1f621}",
                "Medium" => "\u{26a0}\u{fe0f}",
                "Low" => "\u{1f4a7}",
                "Info" => "\u{2139}\u{fe0f}",
                _ => "",
            };
            out.push_str(&format!(
                "{} {} ({})\n",
                emoji,
                sev_label,
                commits.len().to_string().yellow()
            ));
            for sc in commits.iter().take(5) {
                let msg_display = if sc.commit.message.trim().is_empty() {
                    "<empty>".to_string()
                } else {
                    truncate(sc.commit.message.trim(), 60)
                };
                for issue in &sc.issues {
                    out.push_str(&format!(
                        "  \u{2022} {} \"{}\" \u{2014} {}\n",
                        sc.commit.short_hash.cyan(),
                        msg_display.dimmed(),
                        issue.message
                    ));
                }
            }
            if commits.len() > 5 {
                out.push_str(&format!(
                    "  ... and {} more\n",
                    (commits.len() - 5).to_string().dimmed()
                ));
            }
            out.push('\n');
        }
    }

    // Stats
    out.push_str(&format!("{}\n", "\u{1f4ca} Statistics".bold()));
    out.push_str(&format!(
        "  Avg message length: {} chars\n",
        format!("{:.1}", report.stats.avg_message_length).yellow()
    ));
    out.push_str(&format!(
        "  Empty messages: {}\n",
        report.stats.empty_messages.to_string().red()
    ));
    out.push_str(&format!(
        "  Single-word messages: {}\n",
        report.stats.single_word_messages.to_string().red()
    ));
    out.push_str(&format!(
        "  WIP commits: {}\n",
        report.stats.wip_commits.to_string().yellow()
    ));
    out.push_str(&format!(
        "  'fix' only commits: {}\n\n",
        report.stats.fix_only_commits.to_string().red()
    ));

    // Score
    let grade = score_to_grade(report.total_score);
    out.push_str(&format!(
        "{} {}/100 ({})\n",
        "\u{1f3c6} Score".bold(),
        format!("{:.0}", report.total_score).bold(),
        grade
    ));

    out
}

/// Format report as JSON.
pub fn format_json(report: &Report) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct JsonReport {
        total_commits: usize,
        total_issues: usize,
        score: f64,
        grade: String,
        stats: CommitStats,
        issues: Vec<JsonIssue>,
    }

    #[derive(Serialize)]
    struct JsonIssue {
        hash: String,
        author: String,
        message: String,
        rule_id: String,
        severity: String,
        roast: String,
    }

    let json_report = JsonReport {
        total_commits: report.stats.total_commits,
        total_issues: report.stats.total_issues,
        score: report.total_score,
        grade: score_to_grade(report.total_score),
        stats: report.stats.clone(),
        issues: report
            .scored_commits
            .iter()
            .flat_map(|sc| {
                sc.issues.iter().map(move |issue| JsonIssue {
                    hash: sc.commit.short_hash.clone(),
                    author: sc.commit.author.clone(),
                    message: sc.commit.message.trim().to_string(),
                    rule_id: issue.rule_id.clone(),
                    severity: issue.severity.label().to_string(),
                    roast: issue.message.clone(),
                })
            })
            .collect(),
    };

    serde_json::to_string_pretty(&json_report)
}

fn truncate(s: &str, max: usize) -> String {
    crate::utils::truncate(s, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Severity;

    fn make_commit(hash: &str, message: &str) -> CommitInfo {
        CommitInfo {
            hash: hash.to_string(),
            short_hash: hash[..7].to_string(),
            author: "test".to_string(),
            message: message.to_string(),
            timestamp: 0,
            files_changed: 1,
            insertions: 10,
            deletions: 5,
        }
    }

    fn make_issue(rule_id: &str, severity: Severity) -> Issue {
        Issue {
            rule_id: rule_id.to_string(),
            rule_name: "test".to_string(),
            severity,
            message: "test roast".to_string(),
        }
    }

    #[test]
    fn test_build_report_empty() {
        let report = build_report(vec![]);
        assert_eq!(report.stats.total_commits, 0);
        assert_eq!(report.total_score, 100.0);
    }

    #[test]
    fn test_build_report_with_issues() {
        let scored = vec![
            ScoredCommit {
                commit: make_commit("abc1234", "fix"),
                issues: vec![make_issue("fix-only", Severity::High)],
            },
            ScoredCommit {
                commit: make_commit("def5678", "good commit message here"),
                issues: vec![],
            },
        ];
        let report = build_report(scored);
        assert_eq!(report.stats.total_commits, 2);
        assert_eq!(report.stats.total_issues, 1);
        assert!(report.total_score < 100.0);
    }

    #[test]
    fn test_format_json_valid() {
        let scored = vec![ScoredCommit {
            commit: make_commit("abc1234", "fix"),
            issues: vec![make_issue("fix-only", Severity::High)],
        }];
        let report = build_report(scored);
        let json = format_json(&report).expect("JSON should serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Valid JSON");
        assert!(parsed.get("score").is_some());
        assert!(parsed.get("issues").is_some());
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world foo bar", 10), "hello w...");
    }
}
