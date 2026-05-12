//! Core types for PR title analysis.

use serde::{Deserialize, Serialize};

/// Source of the PR title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrSource {
    /// Extracted from local git merge commits.
    Local,
    /// Fetched from GitHub API.
    GitHub { repo: String },
}

/// A single PR entry with its title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrEntry {
    /// PR number or merge commit short hash.
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub source: PrSource,
}

/// Severity for PR title issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PrSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl PrSeverity {
    pub fn emoji(&self) -> &'static str {
        match self {
            PrSeverity::Critical => "\u{1f480}",
            PrSeverity::High => "\u{1f621}",
            PrSeverity::Medium => "\u{26a0}\u{fe0f}",
            PrSeverity::Low => "\u{1f4a7}",
            PrSeverity::Info => "\u{2139}\u{fe0f}",
        }
    }

    pub fn penalty(&self) -> f64 {
        match self {
            PrSeverity::Critical => 10.0,
            PrSeverity::High => 5.0,
            PrSeverity::Medium => 2.0,
            PrSeverity::Low => 0.5,
            PrSeverity::Info => 0.0,
        }
    }
}

/// A detected issue with a PR title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrIssue {
    pub rule_id: String,
    pub severity: PrSeverity,
    pub message: String,
    pub pr_id: String,
    pub pr_title: String,
}
