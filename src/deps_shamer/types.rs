//! Core types for dependency analysis.

use serde::{Deserialize, Serialize};

/// Supported package managers / ecosystems.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ecosystem {
    Rust,
    Node,
    Python,
    Go,
    Java,
    Ruby,
    Unknown,
}

impl Ecosystem {
    pub fn display_name(&self) -> &'static str {
        match self {
            Ecosystem::Rust => "Rust/Cargo",
            Ecosystem::Node => "Node/npm",
            Ecosystem::Python => "Python/pip",
            Ecosystem::Go => "Go/modules",
            Ecosystem::Java => "Java/Maven",
            Ecosystem::Ruby => "Ruby/Bundler",
            Ecosystem::Unknown => "Unknown",
        }
    }
}

/// Source of a dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepSource {
    Registry,
    Git { url: String },
    Path { path: String },
    Unknown,
}

/// A single dependency entry parsed from a dependency file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source: DepSource,
    pub is_dev: bool,
    pub is_optional: bool,
}

/// Severity level for dependency issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DepSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl DepSeverity {
    pub fn emoji(&self) -> &'static str {
        match self {
            DepSeverity::Critical => "\u{1f480}",
            DepSeverity::High => "\u{1f621}",
            DepSeverity::Medium => "\u{26a0}\u{fe0f}",
            DepSeverity::Low => "\u{1f4a7}",
            DepSeverity::Info => "\u{2139}\u{fe0f}",
        }
    }

    pub fn penalty(&self) -> f64 {
        match self {
            DepSeverity::Critical => 10.0,
            DepSeverity::High => 5.0,
            DepSeverity::Medium => 2.0,
            DepSeverity::Low => 0.5,
            DepSeverity::Info => 0.0,
        }
    }
}

/// A detected issue with a dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepIssue {
    pub rule_id: String,
    pub severity: DepSeverity,
    pub message: String,
    pub dep_name: Option<String>,
}

/// Parsed dependency file result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepFile {
    pub path: String,
    pub ecosystem: Ecosystem,
    pub dependencies: Vec<Dependency>,
}
