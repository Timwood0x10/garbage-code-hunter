//! ASCII trend chart rendering.

use super::history::HistoryRecord;
use colored::Colorize;

/// Format trend report for terminal output.
pub fn format_terminal(records: &[HistoryRecord], last_n: usize) -> String {
    if records.is_empty() {
        return "\n  No scan history found. Run `garbage-code-hunter scan` first.\n".to_string();
    }

    let display_records: Vec<&HistoryRecord> = if records.len() > last_n {
        records[records.len() - last_n..].iter().collect()
    } else {
        records.iter().collect()
    };

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "\u{1f4c8} Quality Trend".bold()));
    out.push_str(&format!(
        "  (showing last {} scans)\n\n",
        display_records.len()
    ));

    // ASCII chart
    out.push_str(&render_chart(&display_records));
    out.push('\n');

    // Breakdown comparison (first vs last)
    if display_records.len() >= 2 {
        out.push_str(&format!("{}\n", "\u{1f4ca} Breakdown".bold()));
        out.push_str(&format!("  {}\n", "\u{2500}".repeat(40)));

        let first = display_records.first().unwrap();
        let last = display_records.last().unwrap();

        // Compare overall
        let overall_diff = last.overall_score - first.overall_score;
        let overall_arrow = diff_arrow(overall_diff);
        out.push_str(&format!(
            "  {:<20} {:.0} \u{2192} {:.0} ({:+.0}) {}\n",
            "Overall", first.overall_score, last.overall_score, overall_diff, overall_arrow
        ));
        out.push('\n');

        // Compare each tool
        for last_tool in &last.tools {
            if let Some(first_tool) = first.tools.iter().find(|t| t.name == last_tool.name) {
                let diff = last_tool.score - first_tool.score;
                let arrow = diff_arrow(diff);
                out.push_str(&format!(
                    "  {:<20} {:.0} \u{2192} {:.0} ({:+.0}) {}\n",
                    first_tool.name, first_tool.score, last_tool.score, diff, arrow
                ));
            }
        }
        out.push('\n');
    }

    // List recent scans
    out.push_str(&format!("{}\n", "\u{1f4cb} Recent Scans".bold()));
    out.push_str(&format!("  {}\n", "\u{2500}".repeat(50)));
    for record in display_records.iter().rev().take(5) {
        let score_str = format_score(record.overall_score);
        out.push_str(&format!(
            "  {}  {}  {}\n",
            &record.timestamp[..19],
            score_str,
            record.project_path.dimmed()
        ));
    }
    out.push('\n');

    out
}

/// Format trend as JSON.
pub fn format_json(records: &[HistoryRecord]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "records": records.iter().map(|r| {
            serde_json::json!({
                "timestamp": r.timestamp,
                "project_path": r.project_path,
                "overall_score": r.overall_score,
                "tools": r.tools.iter().map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "score": t.score,
                        "item_count": t.item_count,
                    })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    }))
    .unwrap_or_else(|_| "[]".to_string())
}

/// Render an ASCII chart of scores over time.
fn render_chart(records: &[&HistoryRecord]) -> String {
    if records.is_empty() {
        return String::new();
    }

    let scores: Vec<f64> = records.iter().map(|r| r.overall_score).collect();
    let min_score = scores.iter().cloned().fold(f64::MAX, f64::min).max(0.0);
    let max_score = scores.iter().cloned().fold(f64::MIN, f64::max).min(100.0);

    // Chart dimensions
    let chart_height: usize = 10;
    let chart_width = scores.len().max(2);

    // Scale scores to chart height
    let range = (max_score - min_score).max(1.0);
    let scaled: Vec<usize> = scores
        .iter()
        .map(|s| ((s - min_score) / range * (chart_height - 1) as f64).round() as usize)
        .collect();

    let mut out = String::new();
    out.push_str(&format!("  {}\n", "Score".dimmed()));

    // Draw chart top-down
    for row in (0..chart_height).rev() {
        let y_label = min_score + (row as f64 / (chart_height - 1) as f64) * range;
        if row == chart_height - 1 || row == 0 || row == chart_height / 2 {
            out.push_str(&format!(
                "  {:>4} \u{2502}",
                format!("{:.0}", y_label).dimmed()
            ));
        } else {
            out.push_str("       \u{2502}");
        }

        for (i, &s) in scaled.iter().enumerate() {
            if s == row {
                if i + 1 < scaled.len() && scaled[i + 1] > s {
                    out.push_str(" \u{2570}\u{2500}");
                } else if i + 1 < scaled.len() && scaled[i + 1] < s {
                    out.push_str(" \u{256f}");
                } else if i + 1 < scaled.len() {
                    out.push_str(" \u{2500}");
                } else {
                    out.push_str(" \u{25cf}");
                }
            } else if i > 0 && i < scaled.len() {
                let prev = scaled[i - 1];
                let curr = s;
                let should_draw = if prev < curr {
                    row > prev && row < curr
                } else if prev > curr {
                    row < prev && row > curr
                } else {
                    false
                };
                if should_draw {
                    out.push_str(" \u{2502}");
                } else {
                    out.push_str("  ");
                }
            } else {
                out.push_str("  ");
            }
        }
        out.push('\n');
    }

    // X-axis
    out.push_str("       \u{2514}");
    for _ in 0..chart_width {
        out.push_str("\u{2500}\u{2500}\u{2500}");
    }
    out.push('\n');

    // Date labels (first, middle, last)
    out.push_str("        ");
    let len = records.len();
    for (i, record) in records.iter().enumerate() {
        if i == 0 || i == len / 2 || i == len - 1 {
            let label = &record.timestamp[5..10]; // MM-DD
            out.push_str(&format!("{:<6}", label.dimmed()));
        } else {
            out.push_str("   ");
        }
    }
    out.push('\n');

    out
}

fn diff_arrow(diff: f64) -> &'static str {
    if diff > 1.0 {
        "\u{1f4c8}" // up
    } else if diff < -1.0 {
        "\u{1f4c9}" // down
    } else {
        "\u{27a1}" // flat
    }
}

fn format_score(score: f64) -> colored::ColoredString {
    use colored::Colorize;
    if score >= 80.0 {
        format!("{:.0}", score).green()
    } else if score >= 60.0 {
        format!("{:.0}", score).yellow()
    } else {
        format!("{:.0}", score).red()
    }
}

#[cfg(test)]
mod tests {
    use super::super::history::ToolScore;
    use super::*;

    fn make_record(ts: &str, score: f64) -> HistoryRecord {
        HistoryRecord {
            timestamp: ts.to_string(),
            project_path: "/tmp/test".to_string(),
            overall_score: score,
            tools: vec![ToolScore {
                name: "code-hunter".to_string(),
                score,
                item_count: 10,
            }],
        }
    }

    #[test]
    fn test_format_terminal_empty() {
        let out = format_terminal(&[], 10);
        assert!(out.contains("No scan history"));
    }

    #[test]
    fn test_format_terminal_with_records() {
        let records = vec![
            make_record("2024-01-15T14:30:00Z", 65.0),
            make_record("2024-01-16T09:00:00Z", 72.0),
            make_record("2024-01-17T10:00:00Z", 80.0),
        ];
        let out = format_terminal(&records, 10);
        assert!(out.contains("Quality Trend"));
        assert!(out.contains("Breakdown"));
        assert!(out.contains("Recent Scans"));
    }

    #[test]
    fn test_format_json() {
        let records = vec![make_record("2024-01-15T14:30:00Z", 72.5)];
        let json = format_json(&records);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["records"].as_array().unwrap().len() == 1);
    }

    #[test]
    fn test_diff_arrow() {
        assert_eq!(diff_arrow(5.0), "\u{1f4c8}");
        assert_eq!(diff_arrow(-5.0), "\u{1f4c9}");
        assert_eq!(diff_arrow(0.5), "\u{27a1}");
    }
}
