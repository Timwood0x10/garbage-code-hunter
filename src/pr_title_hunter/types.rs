//! Core types for PR title analysis.

pub use crate::common::Severity;
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

/// A detected issue with a PR title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub pr_id: String,
    pub pr_title: String,
}
