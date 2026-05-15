//! Git Decay Curve — analyze project quality degradation over time.

use crate::common::i18n_ext::t;
use crate::common::OutputFormat;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// A single point on the decay curve.
#[derive(Debug, Clone)]
pub struct DecayPoint {
    pub date: String,
    pub score: f64,
    pub event: Option<String>,
}

/// Full decay analysis result.
#[derive(Debug, Clone)]
pub struct DecayReport {
    pub points: Vec<DecayPoint>,
    pub turning_point: Option<DecayPoint>,
    pub worst_contributor: Option<String>,
    pub current_health: &'static str,
}

/// Run decay analysis on a git repository.
pub fn run(path: &Path, format: &OutputFormat, lang: &str) -> Result<String> {
    let report = analyze_decay(path)?;

    let output = match format {
        OutputFormat::Terminal => display_terminal(&report, lang),
        OutputFormat::Json => display_json(&report),
    };

    Ok(output)
}

/// Analyze git log for quality decay signals.
fn analyze_decay(path: &Path) -> Result<DecayReport> {
    // Get commit history with stats
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--format=%H|%ai|%an|%s",
            "--shortstat",
            "--no-merges",
            "-50",
        ])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Not a git repository or git not available"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = parse_git_log(&stdout);

    // Analyze quality signals per commit
    let mut points = Vec::new();
    let mut worst_author_debt = std::collections::HashMap::<String, u32>::new();

    for commit in &commits {
        let mut score: f64 = 100.0;
        let mut event = None;

        // Penalize bad commit messages
        let msg = &commit.message;
        if msg.len() < 5 {
            score -= 20.0;
            event = Some("minimal commit message".to_string());
        } else if is_generic_message(msg) {
            score -= 10.0;
            event = Some(format!("generic message: '{}'", truncate(msg, 20)));
        }

        // Penalize rapid commits (sign of hotfixes)
        if msg.to_lowercase().contains("fix") || msg.to_lowercase().contains("hotfix") {
            score -= 5.0;
            *worst_author_debt.entry(commit.author.clone()).or_insert(0) += 1;
        }

        if msg.to_lowercase().contains("wip") {
            score -= 15.0;
        }

        points.push(DecayPoint {
            date: commit.date[..10].to_string(),
            score: score.max(0.0),
            event,
        });
    }

    // Find turning point (biggest sustained drop)
    let turning_point = find_turning_point(&points);

    // Find worst contributor
    let worst_contributor = worst_author_debt
        .iter()
        .max_by_key(|(_, v)| *v)
        .map(|(k, _)| k.clone());

    // Determine current health
    let avg_score = if points.is_empty() {
        100.0
    } else {
        points.iter().map(|p| p.score).sum::<f64>() / points.len() as f64
    };
    let current_health = health_label(avg_score);

    Ok(DecayReport {
        points,
        turning_point,
        worst_contributor,
        current_health,
    })
}

#[derive(Debug)]
struct CommitInfo {
    date: String,
    author: String,
    message: String,
}

fn parse_git_log(output: &str) -> Vec<CommitInfo> {
    let mut commits = Vec::new();
    let mut lines = output.lines();

    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            continue;
        }

        commits.push(CommitInfo {
            date: parts[1].to_string(),
            author: parts[2].to_string(),
            message: parts[3].to_string(),
        });

        // Skip shortstat line(s)
        for next in lines.by_ref() {
            if next.trim().is_empty() || next.contains("file changed") {
                break;
            }
        }
    }

    commits
}

fn is_generic_message(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    let generics = [
        "fix", "update", "change", "wip", "tmp", "temp", "asdf", "test",
    ];
    generics
        .iter()
        .any(|g| lower.trim() == *g || lower.starts_with(&format!("{} ", g)))
}

fn find_turning_point(points: &[DecayPoint]) -> Option<DecayPoint> {
    if points.len() < 3 {
        return None;
    }

    // Look for the biggest sustained drop (3+ consecutive decreases)
    let mut max_drop = 0.0;
    let mut turning = None;

    for i in 2..points.len() {
        let drop = points[i - 2].score - points[i].score;
        if drop > max_drop && drop > 10.0 {
            max_drop = drop;
            turning = Some(points[i].clone());
        }
    }

    turning
}

fn health_label(score: f64) -> &'static str {
    match score as u32 {
        90..=100 => "Thriving",
        70..=89 => "Healthy",
        50..=69 => "Declining",
        30..=49 => "Critical",
        _ => "Terminal",
    }
}

fn truncate(s: &str, max: usize) -> String {
    crate::utils::truncate(s, max)
}

fn display_terminal(report: &DecayReport, lang: &str) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n",
        t(
            lang,
            "\u{1f4c9} 项目衰变分析",
            "\u{1f4c9} Project Decay Analysis"
        )
        .bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    // Current health
    out.push_str(&format!(
        "  {}: {}\n\n",
        t(lang, "当前健康度", "Current Health"),
        report.current_health.bold()
    ));

    // Decay timeline (last 10 points)
    let display_points: Vec<_> = if report.points.len() > 10 {
        report.points[report.points.len() - 10..].to_vec()
    } else {
        report.points.clone()
    };

    for point in &display_points {
        let bar_len = (point.score / 5.0) as usize;
        let bar: String = "\u{2588}".repeat(bar_len);
        let bar_colored = if point.score >= 80.0 {
            bar.green()
        } else if point.score >= 50.0 {
            bar.yellow()
        } else {
            bar.red()
        };
        let event_str = point
            .event
            .as_ref()
            .map(|e| format!(" \u{2190} {}", e.dimmed()))
            .unwrap_or_default();
        out.push_str(&format!(
            "  {} {:.0} {}{}\n",
            &point.date, point.score, bar_colored, event_str
        ));
    }

    out.push('\n');

    // Turning point
    if let Some(tp) = &report.turning_point {
        out.push_str(&format!(
            "{}\n",
            t(lang, "\u{1f534} 转折点", "\u{1f534} Turning Point").bold()
        ));
        out.push_str(&format!(
            "  {}\n",
            t(
                lang,
                &format!("{} — 质量显著下降", tp.date),
                &format!("{} — quality dropped significantly", tp.date)
            )
        ));
        if let Some(event) = &tp.event {
            out.push_str(&format!(
                "  {}: {}\n",
                t(lang, "触发因素", "Trigger"),
                event
            ));
        }
        out.push('\n');
    }

    // Worst contributor
    if let Some(author) = &report.worst_contributor {
        out.push_str(&format!(
            "{}\n",
            t(
                lang,
                "\u{1f468}\u{200d}\u{1f4bb} 最多修复提交的作者",
                "\u{1f468}\u{200d}\u{1f4bb} Most Fix-Heavy Author"
            )
            .bold()
        ));
        out.push_str(&format!(
            "  {}\n",
            t(
                lang,
                &format!("{} — 检测到最多的 'fix' 提交", author),
                &format!("{} — most 'fix' commits detected", author)
            )
        ));
    }

    out
}

fn display_json(report: &DecayReport) -> String {
    serde_json::json!({
        "current_health": report.current_health,
        "turning_point": report.turning_point.as_ref().map(|tp| {
            serde_json::json!({
                "date": tp.date,
                "score": tp.score,
                "event": tp.event,
            })
        }),
        "worst_contributor": report.worst_contributor,
        "timeline": report.points.iter().map(|p| {
            serde_json::json!({
                "date": p.date,
                "score": p.score,
                "event": p.event,
            })
        }).collect::<Vec<_>>(),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_generic_message() {
        assert!(is_generic_message("fix"));
        assert!(is_generic_message("update"));
        assert!(is_generic_message("WIP"));
        assert!(!is_generic_message("fix: resolve auth token refresh bug"));
    }

    #[test]
    fn test_health_label() {
        assert_eq!(health_label(95.0), "Thriving");
        assert_eq!(health_label(75.0), "Healthy");
        assert_eq!(health_label(55.0), "Declining");
        assert_eq!(health_label(35.0), "Critical");
        assert_eq!(health_label(10.0), "Terminal");
    }

    #[test]
    fn test_find_turning_point() {
        let points = vec![
            DecayPoint {
                date: "2024-01".to_string(),
                score: 90.0,
                event: None,
            },
            DecayPoint {
                date: "2024-02".to_string(),
                score: 85.0,
                event: None,
            },
            DecayPoint {
                date: "2024-03".to_string(),
                score: 60.0,
                event: None,
            },
        ];
        let tp = find_turning_point(&points);
        assert!(tp.is_some());
    }
}
