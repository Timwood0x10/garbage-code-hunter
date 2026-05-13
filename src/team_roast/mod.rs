//! Team Roast Mode — per-developer analysis and roasting.

use crate::common::i18n_ext::t;
use crate::common::OutputFormat;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

/// Stats for a single team member.
#[derive(Debug, Clone)]
pub struct MemberStats {
    pub name: String,
    pub commit_count: usize,
    pub fix_commits: usize,
    pub avg_message_length: f64,
    pub worst_message: String,
    pub roast: String,
}

/// Run team analysis.
pub fn run(path: &Path, format: &OutputFormat, limit: usize, lang: &str) -> Result<String> {
    let stats = analyze_team(path, limit)?;

    let output = match format {
        OutputFormat::Terminal => format_terminal(&stats, lang),
        OutputFormat::Json => format_json(&stats),
    };

    Ok(output)
}

fn analyze_team(path: &Path, limit: usize) -> Result<Vec<MemberStats>> {
    let output = std::process::Command::new("git")
        .args([
            "log",
            &format!("-{}", limit),
            "--format=%an|%s",
            "--no-merges",
        ])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Not a git repository"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut author_data: HashMap<String, Vec<String>> = HashMap::new();

    for line in stdout.lines() {
        if let Some((author, message)) = line.split_once('|') {
            author_data
                .entry(author.to_string())
                .or_default()
                .push(message.to_string());
        }
    }

    let mut stats: Vec<MemberStats> = author_data
        .into_iter()
        .map(|(name, messages)| {
            let commit_count = messages.len();
            let fix_commits = messages
                .iter()
                .filter(|m| {
                    let lower = m.to_lowercase();
                    lower.contains("fix")
                        || lower.contains("hotfix")
                        || lower.contains("bug")
                        || lower.contains("patch")
                })
                .count();

            let avg_len = if commit_count > 0 {
                messages.iter().map(|m| m.len()).sum::<usize>() as f64 / commit_count as f64
            } else {
                0.0
            };

            let worst = messages
                .iter()
                .min_by_key(|m| m.len())
                .cloned()
                .unwrap_or_default();

            let fix_ratio = if commit_count > 0 {
                fix_commits as f64 / commit_count as f64
            } else {
                0.0
            };

            let roast = generate_roast(&name, commit_count, fix_ratio, avg_len, &worst);

            MemberStats {
                name,
                commit_count,
                fix_commits,
                avg_message_length: avg_len,
                worst_message: worst,
                roast,
            }
        })
        .collect();

    // Sort by commit count descending
    stats.sort_by_key(|a| std::cmp::Reverse(a.commit_count));

    Ok(stats)
}

fn generate_roast(
    name: &str,
    commits: usize,
    fix_ratio: f64,
    avg_msg_len: f64,
    worst_msg: &str,
) -> String {
    let mut roasts = Vec::new();

    if fix_ratio > 0.4 {
        roasts.push(format!(
            "{:.0}% of commits are fixes — someone's code keeps breaking",
            fix_ratio * 100.0
        ));
    }

    if avg_msg_len < 15.0 {
        roasts.push(format!(
            "average commit message is {:.0} chars — a novel by their standards",
            avg_msg_len
        ));
    }

    if worst_msg.len() < 5 && !worst_msg.is_empty() {
        roasts.push(format!(
            "worst commit message: '{}' — truly poetic",
            worst_msg
        ));
    }

    if commits > 50 && fix_ratio > 0.3 {
        roasts.push("prolific committer, half of it is fixing their own code".to_string());
    }

    if roasts.is_empty() {
        format!("{} — nothing to roast, suspiciously clean", name)
    } else {
        roasts.join("; ")
    }
}

fn format_terminal(stats: &[MemberStats], lang: &str) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n",
        t(lang, "\u{1f465} 团队分析", "\u{1f465} Team Analysis").bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    if stats.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            t(
                lang,
                "在 git 历史中没有找到团队成员。",
                "No team members found in git history."
            )
        ));
        return out;
    }

    for (i, member) in stats.iter().take(10).enumerate() {
        out.push_str(&format!("  #{} {}\n", i + 1, member.name.bold()));
        out.push_str(&format!(
            "     Commits: {} | Fix commits: {} ({:.0}%)\n",
            member.commit_count,
            member.fix_commits,
            if member.commit_count > 0 {
                member.fix_commits as f64 / member.commit_count as f64 * 100.0
            } else {
                0.0
            }
        ));
        out.push_str(&format!(
            "     Avg message length: {:.0} chars\n",
            member.avg_message_length
        ));
        if !member.worst_message.is_empty() {
            out.push_str(&format!(
                "     Worst message: \"{}\"\n",
                truncate(&member.worst_message, 40)
            ));
        }
        out.push_str(&format!("     \u{1f3ad} {}\n\n", member.roast.italic()));
    }

    out
}

fn format_json(stats: &[MemberStats]) -> String {
    serde_json::json!({
        "members": stats.iter().map(|m| {
            serde_json::json!({
                "name": m.name,
                "commit_count": m.commit_count,
                "fix_commits": m.fix_commits,
                "avg_message_length": m.avg_message_length,
                "worst_message": m.worst_message,
                "roast": m.roast,
            })
        }).collect::<Vec<_>>(),
    })
    .to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_roast() {
        let roast = generate_roast("Alice", 10, 0.5, 8.0, "fix");
        assert!(roast.contains("fixes"));
    }

    #[test]
    fn test_generate_roast_clean() {
        let roast = generate_roast("Bob", 10, 0.1, 50.0, "a proper commit message");
        assert!(roast.contains("nothing to roast"));
    }

    #[test]
    fn test_run_on_current_dir() {
        let result = run(
            std::path::Path::new("."),
            &OutputFormat::Terminal,
            50,
            "en-US",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_on_current_dir_chinese() {
        let result = run(
            std::path::Path::new("."),
            &OutputFormat::Terminal,
            50,
            "zh-CN",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_json_format() {
        let result = run(std::path::Path::new("."), &OutputFormat::Json, 50, "en-US");
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_roast_high_fix_ratio() {
        let roast = generate_roast("Charlie", 100, 0.6, 8.0, "x");
        assert!(roast.contains("60%"));
    }

    #[test]
    fn test_generate_roast_short_messages() {
        let roast = generate_roast("Dave", 10, 0.1, 5.0, "ok");
        assert!(roast.contains("chars"));
    }
}
