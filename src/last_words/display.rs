//! Display last-words report in terminal or JSON.

use super::scanner::LastWord;
use crate::common::i18n_ext::t;
use colored::Colorize;

/// Format last-words report for terminal.
pub fn format_terminal(words: &[LastWord], lang: &str) -> String {
    if words.is_empty() {
        return format!(
            "\n  {}\n",
            t(
                lang,
                "没有发现遗留注释。你的代码可疑地干净。",
                "No legacy comments found. Your code is suspiciously clean."
            )
        );
    }

    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        t(
            lang,
            "\u{1f576}\u{fe0f} 代码遗言",
            "\u{1f576}\u{fe0f} Code Last Words"
        )
        .bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));

    // Group by kind
    let mut by_kind: std::collections::HashMap<&str, Vec<&LastWord>> =
        std::collections::HashMap::new();
    for w in words {
        by_kind.entry(w.kind.label()).or_default().push(w);
    }

    // Sort by count descending
    let mut groups: Vec<_> = by_kind.iter().collect();
    groups.sort_by_key(|a| std::cmp::Reverse(a.1.len()));

    for (kind_label, items) in &groups {
        out.push_str(&format!(
            "{} {} ({} found)\n",
            tombstone_emoji(kind_label),
            kind_label.bold(),
            items.len()
        ));

        // Show top 5 longest-lived
        let mut sorted = items.to_vec();
        sorted.sort_by_key(|a| std::cmp::Reverse(a.age_days.unwrap_or(0)));

        for item in sorted.iter().take(5) {
            let age_str = match item.age_days {
                Some(days) if days > 365 => format!("{} {}", days / 365, t(lang, "年", "years"))
                    .red()
                    .to_string(),
                Some(days) if days > 90 => format!("{} {}", days, t(lang, "天", "days"))
                    .yellow()
                    .to_string(),
                Some(days) => format!("{} {}", days, t(lang, "天", "days"))
                    .green()
                    .to_string(),
                None => t(lang, "年龄未知", "age unknown").dimmed().to_string(),
            };
            let quote = item.kind.tombstone_quote();
            let file_short = item
                .file
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| item.file.display().to_string());
            out.push_str(&format!(
                "  {}:{} \"{}\"\n    \u{2514}\u{2500} {}\n    \u{2514}\u{2500} {}\n",
                file_short.dimmed(),
                item.line,
                truncate(&item.text, 60).dimmed(),
                quote,
                age_str
            ));
        }
        out.push('\n');
    }

    // Summary stats
    let total = words.len();
    let with_age: Vec<_> = words.iter().filter_map(|w| w.age_days).collect();
    let oldest = with_age.iter().max().copied().unwrap_or(0);
    let avg_age = if with_age.is_empty() {
        0
    } else {
        with_age.iter().sum::<u64>() / with_age.len() as u64
    };

    out.push_str(&format!(
        "{}\n",
        t(lang, "\u{1f4ca} 统计", "\u{1f4ca} Summary").bold()
    ));
    out.push_str(&format!(
        "  {}: {}\n",
        t(lang, "遗留注释总数", "Total legacy comments"),
        total
    ));
    if oldest > 0 {
        out.push_str(&format!(
            "  {}: {} 天 ({:.1} 年)\n",
            t(lang, "最老", "Oldest"),
            oldest,
            oldest as f64 / 365.0
        ));
        out.push_str(&format!(
            "  {}: {} 天\n",
            t(lang, "平均年龄", "Average age"),
            avg_age
        ));
    }

    out
}

/// Format last-words as JSON.
pub fn format_json(words: &[LastWord]) -> String {
    let items: Vec<serde_json::Value> = words
        .iter()
        .map(|w| {
            serde_json::json!({
                "file": w.file.display().to_string(),
                "line": w.line,
                "kind": w.kind.label(),
                "text": w.text,
                "age_days": w.age_days,
            })
        })
        .collect();

    let total = words.len();
    let with_age: Vec<_> = words.iter().filter_map(|w| w.age_days).collect();
    let oldest = with_age.iter().max().copied().unwrap_or(0);
    let avg_age = if with_age.is_empty() {
        0
    } else {
        with_age.iter().sum::<u64>() / with_age.len() as u64
    };

    serde_json::json!({
        "total": total,
        "oldest_days": oldest,
        "average_age_days": avg_age,
        "items": items,
    })
    .to_string()
}

fn tombstone_emoji(kind: &str) -> &'static str {
    match kind {
        "TODO" => "\u{1f6cf}\u{fe0f}",
        "FIXME" => "\u{1f527}",
        "HACK" => "\u{1f529}",
        "TEMP" => "\u{23f3}",
        "quick fix" => "\u{26a1}",
        "WONTFIX" => "\u{1f6ab}",
        "workaround" => "\u{1f4a0}",
        "DEPRECATED" => "\u{1f480}",
        "SAFETY" => "\u{26a0}\u{fe0f}",
        _ => "\u{1f4dc}",
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::super::scanner::LastWordKind;
    use super::*;
    use std::path::PathBuf;

    fn make_word(kind: LastWordKind, age: Option<u64>) -> LastWord {
        LastWord {
            file: PathBuf::from("test.rs"),
            line: 42,
            kind,
            text: "// TODO: fix this".to_string(),
            age_days: age,
        }
    }

    #[test]
    fn test_format_terminal_empty() {
        let out = format_terminal(&[], "en-US");
        assert!(out.contains("No legacy comments"));
    }

    #[test]
    fn test_format_terminal_empty_chinese() {
        let out = format_terminal(&[], "zh-CN");
        assert!(out.contains("遗留注释"));
    }

    #[test]
    fn test_format_terminal_with_items() {
        let words = vec![
            make_word(LastWordKind::Todo, Some(500)),
            make_word(LastWordKind::Fixme, Some(100)),
        ];
        let out = format_terminal(&words, "en-US");
        assert!(out.contains("Code Last Words"));
        assert!(out.contains("TODO"));
        assert!(out.contains("FIXME"));
    }

    #[test]
    fn test_format_terminal_chinese() {
        let words = vec![make_word(LastWordKind::Todo, Some(500))];
        let out = format_terminal(&words, "zh-CN");
        assert!(out.contains("代码遗言"));
    }

    #[test]
    fn test_format_json() {
        let words = vec![make_word(LastWordKind::Todo, Some(365))];
        let json = format_json(&words);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["total"], 1);
        assert_eq!(parsed["oldest_days"], 365);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world long text", 10), "hello w...");
    }
}
