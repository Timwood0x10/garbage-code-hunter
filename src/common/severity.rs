//! Shared severity level used by all analysis modules.

use serde::{Deserialize, Serialize};

/// Severity level for any detected issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    /// Returns the emoji representation for terminal output.
    pub fn emoji(&self) -> &'static str {
        match self {
            Severity::Critical => "\u{1f480}",
            Severity::High => "\u{1f621}",
            Severity::Medium => "\u{26a0}\u{fe0f}",
            Severity::Low => "\u{1f4a7}",
            Severity::Info => "\u{2139}\u{fe0f}",
        }
    }

    /// Returns the numeric penalty weight for scoring.
    pub fn penalty(&self) -> f64 {
        match self {
            Severity::Critical => 10.0,
            Severity::High => 5.0,
            Severity::Medium => 2.0,
            Severity::Low => 0.5,
            Severity::Info => 0.0,
        }
    }

    /// Alias for penalty() — used by commit_roaster which calls it weight().
    pub fn weight(&self) -> f64 {
        self.penalty()
    }

    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Medium => "Medium",
            Severity::Low => "Low",
            Severity::Info => "Info",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ordering ──────────────────────────────────────────────────

    /// Objective: Verify PartialOrd orders by declaration order (Critical highest).
    #[test]
    fn test_severity_ordering_adjacent() {
        assert!(Severity::Critical < Severity::High, "Critical < High");
        assert!(Severity::High < Severity::Medium, "High < Medium");
        assert!(Severity::Medium < Severity::Low, "Medium < Low");
        assert!(Severity::Low < Severity::Info, "Low < Info");
    }

    /// Objective: Verify ordering is transitive across non-adjacent pairs.
    #[test]
    fn test_severity_ordering_transitive() {
        assert!(Severity::Critical < Severity::Medium, "Critical < Medium");
        assert!(Severity::Critical < Severity::Low, "Critical < Low");
        assert!(Severity::Critical < Severity::Info, "Critical < Info");
        assert!(Severity::High < Severity::Low, "High < Low");
        assert!(Severity::High < Severity::Info, "High < Info");
        assert!(Severity::Medium < Severity::Info, "Medium < Info");
    }

    /// Objective: Verify equality is reflexive.
    #[test]
    fn test_severity_ordering_equal() {
        assert_eq!(Severity::Critical, Severity::Critical);
        assert_eq!(Severity::High, Severity::High);
        assert_eq!(Severity::Medium, Severity::Medium);
        assert_eq!(Severity::Low, Severity::Low);
        assert_eq!(Severity::Info, Severity::Info);
    }

    // ── emoji ─────────────────────────────────────────────────────

    /// Objective: Verify each severity has a specific non-empty emoji.
    #[test]
    fn test_emoji_specific_values() {
        assert_eq!(Severity::Critical.emoji(), "\u{1f480}", "Critical → skull");
        assert_eq!(Severity::High.emoji(), "\u{1f621}", "High → angry");
        assert_eq!(
            Severity::Medium.emoji(),
            "\u{26a0}\u{fe0f}",
            "Medium → warning"
        );
        assert_eq!(Severity::Low.emoji(), "\u{1f4a7}", "Low → droplet");
        assert_eq!(Severity::Info.emoji(), "\u{2139}\u{fe0f}", "Info → info");
    }

    // ── penalty ───────────────────────────────────────────────────

    /// Objective: Verify penalty values match expected weights.
    #[test]
    fn test_penalty_values() {
        assert_eq!(Severity::Critical.penalty(), 10.0);
        assert_eq!(Severity::High.penalty(), 5.0);
        assert_eq!(Severity::Medium.penalty(), 2.0);
        assert_eq!(Severity::Low.penalty(), 0.5);
        assert_eq!(Severity::Info.penalty(), 0.0);
    }

    /// Objective: Verify penalty ordering matches severity ordering.
    #[test]
    fn test_penalty_ordering_matches_severity() {
        assert!(Severity::Critical.penalty() > Severity::High.penalty());
        assert!(Severity::High.penalty() > Severity::Medium.penalty());
        assert!(Severity::Medium.penalty() > Severity::Low.penalty());
        assert!(Severity::Low.penalty() > Severity::Info.penalty());
    }

    // ── weight alias ──────────────────────────────────────────────

    /// Objective: Verify weight() is an alias for penalty().
    #[test]
    fn test_weight_is_alias_for_penalty() {
        for sev in [
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ] {
            assert_eq!(
                sev.weight(),
                sev.penalty(),
                "{:?}.weight() should equal .penalty()",
                sev
            );
        }
    }

    // ── label ─────────────────────────────────────────────────────

    /// Objective: Verify label for all variants.
    #[test]
    fn test_label_all_variants() {
        assert_eq!(Severity::Critical.label(), "Critical");
        assert_eq!(Severity::High.label(), "High");
        assert_eq!(Severity::Medium.label(), "Medium");
        assert_eq!(Severity::Low.label(), "Low");
        assert_eq!(Severity::Info.label(), "Info");
    }

    /// Objective: Verify labels are all unique.
    #[test]
    fn test_label_unique() {
        let labels: Vec<&str> = [
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ]
        .iter()
        .map(|s| s.label())
        .collect();
        let unique: std::collections::HashSet<&&str> = labels.iter().collect();
        assert_eq!(unique.len(), labels.len(), "all labels must be unique");
    }

    // ── serde ─────────────────────────────────────────────────────

    /// Objective: Verify Severity round-trips through JSON without data loss.
    #[test]
    fn test_severity_serde_roundtrip() {
        for sev in [
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ] {
            let json = serde_json::to_string(&sev).unwrap();
            let parsed: Severity = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, sev, "{sev:?} round-trips through JSON");
        }
    }

    /// Objective: Verify Severity deserializes from JSON strings case-insensitively
    /// (if serde is configured for it).
    #[test]
    fn test_severity_serde_from_string() {
        // serde by default uses unit variants for enums — serialize as strings
        let parsed: Severity = serde_json::from_str("\"Critical\"").unwrap();
        assert_eq!(parsed, Severity::Critical);
        let parsed: Severity = serde_json::from_str("\"Info\"").unwrap();
        assert_eq!(parsed, Severity::Info);
    }
}
