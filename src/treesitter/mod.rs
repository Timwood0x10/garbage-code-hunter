/// Unified tree-sitter based parsing and analysis engine.
///
/// This module replaces the syn-based AST analysis with a language-agnostic
/// tree-sitter approach, enabling support for multiple programming languages.
pub mod duplication;
pub mod engine;
pub mod parsers;
pub mod query;

pub use duplication::{CrossFileDupDetector, IntraFileDupDetector};
pub use engine::{ParsedFile, TreeSitterEngine};
pub use query::QueryRule;
