//! LanguageAdapter trait — unified semantic extraction from parsed AST.
//!
//! SignalDetectors delegate to LanguageAdapter instead of writing
//! per-language tree-sitter queries directly. This makes detectors
//! language-agnostic and consolidates query logic per language.

mod c;
mod cpp;
mod go;
mod helpers;
mod java;
mod js;
mod python;
mod ruby;
mod rust;
mod swift;
mod ts;
mod zig;

pub use self::c::CAdapter;
pub use self::cpp::CppAdapter;
pub use self::go::GoAdapter;
pub use self::java::JavaAdapter;
pub use self::js::JSAdapter;
pub use self::python::PythonAdapter;
pub use self::ruby::RubyAdapter;
pub use self::rust::RustAdapter;
pub use self::swift::SwiftAdapter;
pub use self::ts::TSAdapter;
pub use self::zig::ZigAdapter;

use crate::language::Language;
use crate::treesitter::engine::ParsedFile;

/// Metadata for a function extracted from source code.
#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub nesting_depth: usize,
}

/// LanguageAdapter provides language-specific semantic extraction.
///
/// Each supported language has an adapter implementation that knows
/// the tree-sitter query patterns for that language. SignalDetectors
/// use these methods instead of writing per-language queries.
pub trait LanguageAdapter: Send + Sync {
    fn language(&self) -> Language;

    fn count_panic_calls(&self, file: &ParsedFile) -> usize;

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode>;

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize;

    fn count_naming_violations(&self, file: &ParsedFile) -> usize;

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize;

    fn count_debug_calls(&self, file: &ParsedFile) -> usize;

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize;

    fn count_unsafe_blocks(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Whether the file contains test-specific AST nodes
    /// (e.g., `#[test]` in Rust, `def test_` in Python).
    /// Default returns false — override for language-specific detection.
    fn has_test_nodes(&self, file: &ParsedFile) -> bool {
        let _ = file;
        false
    }
}

/// Dispatch to the correct LanguageAdapter for a given language.
pub fn adapter_for(lang: Language) -> Option<&'static dyn LanguageAdapter> {
    match lang {
        Language::Rust => Some(&RustAdapter),
        Language::Python => Some(&PythonAdapter),
        Language::Go => Some(&GoAdapter),
        Language::JavaScript => Some(&JSAdapter),
        Language::Ruby => Some(&RubyAdapter),
        Language::TypeScript => Some(&TSAdapter),
        Language::Java => Some(&JavaAdapter),
        Language::C => Some(&CAdapter),
        Language::Cpp => Some(&CppAdapter),
        Language::Swift => Some(&SwiftAdapter),
        Language::Zig => Some(&ZigAdapter),
        _ => None,
    }
}

/// Re-export helpers for use by sibling adapters.
pub(crate) use helpers::{
    count_block_ancestors, count_nested_blocks, get_node_text, is_inside_declaration,
    is_repeating_chars, max_scope_depth,
};

#[cfg(test)]
pub(crate) use helpers::parse_code;
