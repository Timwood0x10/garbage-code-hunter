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

    // ── is_generic_message ────────────────────────────────────────

    /// Objective: Verify all generic message patterns are detected.
    /// Invariants: Exact match "fix" or starts with "fix " are generic.
    #[test]
    fn test_is_generic_message_exact_match() {
        assert!(is_generic_message("fix"), "'fix' exact match");
        assert!(is_generic_message("update"), "'update' exact match");
        assert!(is_generic_message("change"), "'change' exact match");
        assert!(is_generic_message("wip"), "'wip' exact match");
        assert!(is_generic_message("tmp"), "'tmp' exact match");
        assert!(is_generic_message("temp"), "'temp' exact match");
        assert!(is_generic_message("asdf"), "'asdf' exact match");
        assert!(is_generic_message("test"), "'test' exact match");
    }

    /// Objective: Verify case-insensitive matching works.
    #[test]
    fn test_is_generic_message_case_insensitive() {
        assert!(is_generic_message("FIX"), "FIX uppercase");
        assert!(is_generic_message("WIP"), "WIP uppercase");
        assert!(is_generic_message("Temp"), "Temp mixed case");
    }

    /// Objective: Verify generic messages followed by content (starts with "fix ") are detected.
    #[test]
    fn test_is_generic_message_with_trailing() {
        assert!(is_generic_message("fix stuff"), "'fix ' prefix");
        assert!(is_generic_message("update the code"), "'update ' prefix");
        assert!(is_generic_message("wip changes"), "'wip ' prefix");
        assert!(is_generic_message("tmp notes"), "'tmp ' prefix");
        assert!(is_generic_message("test the build"), "'test ' prefix");
    }

    /// Objective: Verify non-generic messages are not flagged.
    /// Invariants: Descriptive messages with context are not generic.
    #[test]
    fn test_is_generic_message_non_generic() {
        assert!(
            !is_generic_message("fix: resolve auth token refresh bug"),
            "fix: prefix is not generic"
        );
        assert!(
            !is_generic_message("fixed the race condition"),
            "'fixed' is not exact 'fix'"
        );
        assert!(
            !is_generic_message("refactor database layer"),
            "descriptive message"
        );
        assert!(
            !is_generic_message("testing new feature"),
            "'testing' does not start with 'test '"
        );
        assert!(
            !is_generic_message("updates the docs"),
            "'updates' is not 'update'"
        );
    }

    // ── health_label ──────────────────────────────────────────────

    /// Objective: Verify health label boundary values.
    /// Invariants: 90+ = Thriving, 70-89 = Healthy, 50-69 = Declining, 30-49 = Critical, <30 = Terminal.
    #[test]
    fn test_health_label_boundaries() {
        assert_eq!(health_label(100.0), "Thriving", "100 => Thriving");
        assert_eq!(health_label(90.0), "Thriving", "90 => Thriving");
        assert_eq!(health_label(89.0), "Healthy", "89 => Healthy");
        assert_eq!(health_label(70.0), "Healthy", "70 => Healthy");
        assert_eq!(health_label(69.0), "Declining", "69 => Declining");
        assert_eq!(health_label(50.0), "Declining", "50 => Declining");
        assert_eq!(health_label(49.0), "Critical", "49 => Critical");
        assert_eq!(health_label(30.0), "Critical", "30 => Critical");
        assert_eq!(health_label(29.0), "Terminal", "29 => Terminal");
        assert_eq!(health_label(0.0), "Terminal", "0 => Terminal");
    }

    // ── find_turning_point ────────────────────────────────────────

    /// Objective: Verify find_turning_point returns None when there are fewer than 3 data points.
    /// Invariants: Need at least 3 points to detect a sustained drop.
    #[test]
    fn test_find_turning_point_too_few_points() {
        assert!(find_turning_point(&[]).is_none(), "empty => None");
        assert!(
            find_turning_point(&[DecayPoint {
                date: "2024-01".into(),
                score: 90.0,
                event: None
            }])
            .is_none(),
            "1 point => None"
        );
        assert!(
            find_turning_point(&[
                DecayPoint {
                    date: "2024-01".into(),
                    score: 90.0,
                    event: None
                },
                DecayPoint {
                    date: "2024-02".into(),
                    score: 85.0,
                    event: None
                },
            ])
            .is_none(),
            "2 points => None"
        );
    }

    /// Objective: Verify turning point is detected when score drops >10 over 3+ points.
    #[test]
    fn test_find_turning_point_detected() {
        let points = vec![
            DecayPoint {
                date: "2024-01".into(),
                score: 90.0,
                event: None,
            },
            DecayPoint {
                date: "2024-02".into(),
                score: 85.0,
                event: None,
            },
            DecayPoint {
                date: "2024-03".into(),
                score: 60.0,
                event: None,
            },
        ];
        let tp = find_turning_point(&points);
        assert!(
            tp.is_some(),
            "drop from 90→60 over 3 points should be detected"
        );
        assert_eq!(
            tp.unwrap().score,
            60.0,
            "turning point is the lowest point in the sequence"
        );
    }

    /// Objective: Verify turning point is NOT detected when drop is <=10.
    /// Invariants: Drop must be >10 (strict greater).
    #[test]
    fn test_find_turning_point_small_drop_not_detected() {
        let points = vec![
            DecayPoint {
                date: "2024-01".into(),
                score: 80.0,
                event: None,
            },
            DecayPoint {
                date: "2024-02".into(),
                score: 75.0,
                event: None,
            },
            DecayPoint {
                date: "2024-03".into(),
                score: 71.0,
                event: None,
            },
        ];
        let tp = find_turning_point(&points);
        assert!(tp.is_none(), "9-point drop should not be a turning point");
    }

    /// Objective: Verify the BIGGEST drop wins when there are multiple drops.
    #[test]
    fn test_find_turning_point_biggest_drop_wins() {
        let points = vec![
            DecayPoint {
                date: "2024-01".into(),
                score: 90.0,
                event: None,
            },
            DecayPoint {
                date: "2024-02".into(),
                score: 80.0,
                event: None,
            },
            DecayPoint {
                date: "2024-03".into(),
                score: 70.0,
                event: None,
            }, // points[0]-points[2] = 20
            DecayPoint {
                date: "2024-04".into(),
                score: 65.0,
                event: None,
            },
            DecayPoint {
                date: "2024-05".into(),
                score: 30.0,
                event: None,
            }, // points[2]-points[4] = 40 ← biggest
            DecayPoint {
                date: "2024-06".into(),
                score: 25.0,
                event: None,
            }, // points[3]-points[5] = 40 (tied, first wins)
        ];
        let tp = find_turning_point(&points);
        assert!(tp.is_some(), "should detect a turning point");
        assert_eq!(
            tp.unwrap().score,
            30.0,
            "biggest drop (40 pts) ends at score 30.0"
        );
    }

    /// Objective: Verify turning point detection with scores that increase (no turning point).
    #[test]
    fn test_find_turning_point_increasing_scores() {
        let points = vec![
            DecayPoint {
                date: "2024-01".into(),
                score: 50.0,
                event: None,
            },
            DecayPoint {
                date: "2024-02".into(),
                score: 60.0,
                event: None,
            },
            DecayPoint {
                date: "2024-03".into(),
                score: 70.0,
                event: None,
            },
        ];
        let tp = find_turning_point(&points);
        assert!(
            tp.is_none(),
            "increasing scores should not produce a turning point"
        );
    }

    // ── parse_git_log ─────────────────────────────────────────────

    /// Objective: Verify parse_git_log handles standard git log format.
    #[test]
    fn test_parse_git_log_standard() {
        let input = "abc123|2024-01-15 10:00:00 +0800|Alice|fix: resolve bug\n 1 file changed, 2 insertions(+)\n\ndef456|2024-01-16 11:00:00 +0800|Bob|refactor module\n 2 files changed, 10 insertions(+), 3 deletions(-)\n";
        let commits = parse_git_log(input);
        assert_eq!(commits.len(), 2, "should parse 2 commits");
        assert_eq!(commits[0].author, "Alice");
        assert_eq!(commits[0].message, "fix: resolve bug");
        assert_eq!(commits[1].author, "Bob");
        assert_eq!(commits[1].message, "refactor module");
    }

    /// Objective: Verify parse_git_log handles empty input.
    #[test]
    fn test_parse_git_log_empty() {
        let commits = parse_git_log("");
        assert!(commits.is_empty(), "empty input => no commits");
    }

    /// Objective: Verify parse_git_log handles malformed lines gracefully.
    #[test]
    fn test_parse_git_log_malformed() {
        let input = "not-enough-parts\nabc123|2024-01-15|Alice|valid message\n 1 file changed, 1 insertion(+)\n";
        let commits = parse_git_log(input);
        assert_eq!(commits.len(), 1, "malformed line should be skipped");
        assert_eq!(commits[0].author, "Alice");
    }

    /// Objective: Verify parse_git_log handles single commit.
    #[test]
    fn test_parse_git_log_single() {
        let input = "abc|2024-06-01|Carol|initial commit\n 1 file changed, 1 insertion(+)\n";
        let commits = parse_git_log(input);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].date, "2024-06-01");
        assert_eq!(commits[0].message, "initial commit");
    }

    // ── JSON output ───────────────────────────────────────────────

    /// Objective: Verify display_json produces valid JSON with all expected fields.
    #[test]
    fn test_display_json_structure() {
        let report = DecayReport {
            points: vec![DecayPoint {
                date: "2024-06-01".into(),
                score: 72.5,
                event: Some("bad commit".into()),
            }],
            turning_point: Some(DecayPoint {
                date: "2024-06-01".into(),
                score: 72.5,
                event: Some("bad commit".into()),
            }),
            worst_contributor: Some("Alice".into()),
            current_health: "Healthy",
        };
        let json = display_json(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["current_health"], "Healthy");
        assert_eq!(parsed["worst_contributor"], "Alice");
        assert!(
            parsed["turning_point"].is_object(),
            "turning_point should be an object"
        );
        assert!(parsed["timeline"].is_array(), "timeline should be an array");
    }

    /// Objective: Verify display_json handles empty timeline.
    #[test]
    fn test_display_json_empty_timeline() {
        let report = DecayReport {
            points: vec![],
            turning_point: None,
            worst_contributor: None,
            current_health: "Terminal",
        };
        let json = display_json(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["current_health"], "Terminal");
        assert!(
            parsed["turning_point"].is_null(),
            "no turning point => null"
        );
        assert!(
            parsed["worst_contributor"].is_null(),
            "no worst author => null"
        );
        assert!(
            parsed["timeline"].as_array().unwrap().is_empty(),
            "empty timeline"
        );
    }

    // ── truncate (delegates to utils) ─────────────────────────────

    /// Objective: Verify truncate delegates to utils::truncate without panicking.
    #[test]
    fn test_truncate_delegates() {
        let result = truncate("hello world", 100);
        assert_eq!(result, "hello world", "within max length => unchanged");
    }
}
