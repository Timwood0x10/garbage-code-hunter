// Library interface for garbage-code-hunter
// This allows the crate to be used both as a binary and a library

pub mod analyzer;
pub mod i18n;
pub mod reporter;
pub mod rules;
pub mod scoring;
pub mod utils;

pub use analyzer::{CodeAnalyzer, CodeIssue, RoastLevel, Severity};
pub use i18n::I18n;
pub use reporter::Reporter;
pub use scoring::{CodeQualityScore, CodeScorer, QualityLevel};
