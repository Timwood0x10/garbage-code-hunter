// Library interface for garbage-code-hunter
// This allows the crate to be used both as a binary and a library

pub mod analyzer;
pub mod autopsy;
pub mod badge;
pub mod ci_bot;
pub mod commit_roaster;
pub mod common;
pub mod config;
pub mod context;
pub mod cross_file;
pub mod danger_zone;
pub mod debt_invoice;
pub mod decay;
pub mod deps_shamer;
pub mod educational;
pub mod hall_of_shame;
pub mod i18n;
pub mod last_words;
pub mod llm;
pub mod personality;
pub mod personas;
pub mod pr_title_hunter;
pub mod radar;
pub mod reporter;
pub mod rules;
pub mod scoring;
pub mod team_roast;
pub mod trend;
pub mod utils;

pub use analyzer::{CodeAnalyzer, CodeIssue, Severity};
pub use educational::{EducationalAdvice, EducationalAdvisor};
pub use hall_of_shame::{HallOfShame, ProjectShameStats, ShameEntry};
pub use i18n::I18n;
pub use llm::{LlmConfig, LlmRoastProvider, LocalRoastProvider, RoastMap, RoastProvider};
pub use reporter::Reporter;
pub use scoring::{CodeQualityScore, CodeScorer, QualityLevel};
