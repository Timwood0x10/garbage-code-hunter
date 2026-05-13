//! Developer Personality Analysis — profile your coding style.

pub mod profiles;

use crate::analyzer::CodeAnalyzer;
use crate::common::i18n_ext::t;
use crate::common::OutputFormat;
use anyhow::Result;
use std::path::Path;

/// A developer personality profile.
#[derive(Debug, Clone)]
pub struct Personality {
    pub title: &'static str,
    pub emoji: &'static str,
    pub traits: Vec<&'static str>,
    pub advice: Vec<&'static str>,
    pub score: f64,
}

/// Run personality analysis on a path.
pub fn run(path: &Path, format: &OutputFormat, lang: &str) -> Result<String> {
    let analyzer = CodeAnalyzer::new(&[], lang);
    let issues = analyzer.analyze_path(path);
    let personality = profiles::analyze(&issues);

    let output = match format {
        OutputFormat::Terminal => display_terminal(&personality, lang),
        OutputFormat::Json => display_json(&personality),
    };

    Ok(output)
}

fn display_terminal(p: &Personality, lang: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\n{}\n",
        t(
            lang,
            "\u{1f52e} 开发者人格分析",
            "\u{1f52e} Developer Personality Analysis"
        )
        .bold()
    ));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));
    out.push_str(&format!(
        "  {}\n  {} {}\n\n",
        t(lang, "你是：", "You are:"),
        p.emoji,
        p.title.bold()
    ));

    out.push_str(&format!("  {}\n", t(lang, "特征：", "Traits:")));
    for tr in &p.traits {
        out.push_str(&format!("  \u{2022} {}\n", tr));
    }
    out.push('\n');
    out.push_str(&format!("  {}\n", t(lang, "建议：", "Advice:")));
    for a in &p.advice {
        out.push_str(&format!("  \u{1f4a1} {}\n", a));
    }
    out
}

fn display_json(p: &Personality) -> String {
    serde_json::json!({
        "title": p.title,
        "emoji": p.emoji,
        "traits": p.traits,
        "advice": p.advice,
        "score": p.score,
    })
    .to_string()
}

use colored::Colorize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_on_current_dir() {
        let result = run(std::path::Path::new("."), &OutputFormat::Terminal, "en-US");
        assert!(result.is_ok());
    }
}
