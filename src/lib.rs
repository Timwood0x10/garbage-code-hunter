// Library interface for garbage-code-hunter
// This allows the crate to be used both as a binary and a library

pub mod analyzer;
pub mod config;
pub mod educational;
pub mod hall_of_shame;
pub mod i18n;
pub mod llm;
pub mod reporter;
pub mod rules;
pub mod scoring;
pub mod utils;

pub use analyzer::{CodeAnalyzer, CodeIssue, Severity};
pub use educational::{EducationalAdvice, EducationalAdvisor};
pub use hall_of_shame::{HallOfShame, ProjectShameStats, ShameEntry};
pub use i18n::I18n;
pub use llm::{LlmConfig, LlmRoastProvider, LocalRoastProvider, RoastMap, RoastProvider};
pub use reporter::Reporter;
pub use scoring::{CodeQualityScore, CodeScorer, QualityLevel};
