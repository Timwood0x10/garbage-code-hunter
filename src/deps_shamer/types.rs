//! Core types for dependency analysis.

pub use crate::common::Severity;
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

/// A detected issue with a dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepIssue {
    pub rule_id: String,
    pub severity: Severity,
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
