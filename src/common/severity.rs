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

    #[test]
    fn test_severity_ordering() {
        // Derived PartialOrd orders by declaration order
        assert!(Severity::Critical < Severity::High);
        assert!(Severity::High < Severity::Medium);
        assert!(Severity::Medium < Severity::Low);
        assert!(Severity::Low < Severity::Info);
    }

    #[test]
    fn test_emoji_non_empty() {
        for sev in [
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ] {
            assert!(!sev.emoji().is_empty());
        }
    }

    #[test]
    fn test_penalty_values() {
        assert_eq!(Severity::Critical.penalty(), 10.0);
        assert_eq!(Severity::High.penalty(), 5.0);
        assert_eq!(Severity::Medium.penalty(), 2.0);
        assert_eq!(Severity::Low.penalty(), 0.5);
        assert_eq!(Severity::Info.penalty(), 0.0);
    }

    #[test]
    fn test_weight_is_alias_for_penalty() {
        for sev in [
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ] {
            assert_eq!(sev.weight(), sev.penalty());
        }
    }

    #[test]
    fn test_label() {
        assert_eq!(Severity::Critical.label(), "Critical");
        assert_eq!(Severity::Info.label(), "Info");
    }
}
