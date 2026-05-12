//! Shared types and utilities used across multiple analysis modules.

pub mod severity;
pub use severity::Severity;

use serde::{Deserialize, Serialize};

/// Output format for all analysis reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Terminal,
    Json,
}

/// Grade label for a numeric score.
pub fn score_to_grade(score: f64) -> String {
    match score as u32 {
        90..=100 => "S \u{2728}".to_string(),
        80..=89 => "A \u{1f44d}".to_string(),
        70..=79 => "B \u{1f44d}".to_string(),
        60..=69 => "C \u{1f610}".to_string(),
        50..=59 => "D \u{1f615}".to_string(),
        _ => "F \u{1f4a9}".to_string(),
    }
}

/// Format a score as a colored string (green >= 80, yellow >= 60, red < 60).
pub fn format_score_color(score: f64) -> colored::ColoredString {
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
    use super::*;

    #[test]
    fn test_score_to_grade() {
        assert_eq!(score_to_grade(95.0), "S \u{2728}");
        assert_eq!(score_to_grade(85.0), "A \u{1f44d}");
        assert_eq!(score_to_grade(75.0), "B \u{1f44d}");
        assert_eq!(score_to_grade(65.0), "C \u{1f610}");
        assert_eq!(score_to_grade(55.0), "D \u{1f615}");
        assert_eq!(score_to_grade(30.0), "F \u{1f4a9}");
    }

    #[test]
    fn test_output_format_serde() {
        let f = OutputFormat::Json;
        let s = serde_json::to_string(&f).unwrap();
        let d: OutputFormat = serde_json::from_str(&s).unwrap();
        assert_eq!(d, OutputFormat::Json);
    }
}
