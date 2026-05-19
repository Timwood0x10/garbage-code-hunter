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

    /// Count goroutine spawns (Go-specific).
    fn count_goroutine_spawns(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count `defer` statements inside `for` loops (Go-specific).
    fn count_defer_in_loop(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count Go convention violations: uppercase error strings,
    /// context.Context not first param, if-else with return.
    fn count_go_convention_violations(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count Python-specific code issues: wildcard imports, redundant
    /// bool comparisons, identity comparison violations, type:ignore comments,
    /// legacy string formatting, custom dunder methods, import order.
    fn count_python_issues(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count Java-specific code issues: empty catch, missing javadoc,
    /// try-finally close, string concat in loop, wildcard imports.
    fn count_java_issues(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count Ruby-specific code issues: global variables, bare rescue,
    /// missing frozen_string_literal, negated if, predicate naming, indent.
    fn count_ruby_issues(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count C/C++ code issues: goto, new-expression, sizeof-type, free-mismatch, malloc-check.
    fn count_c_issues(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count TypeScript code issues: any-type, prefer-interface, no-enum.
    fn count_ts_issues(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count dead code blocks — unreachable code after return/break/continue/panic.
    fn count_dead_code(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count duplicate import statements in a file (language-specific).
    fn count_duplicate_imports(&self, file: &ParsedFile) -> usize {
        let _ = file;
        0
    }

    /// Count commented-out code blocks in the file.
    /// Default implementation uses content-based detection.
    fn count_commented_out_code(&self, file: &ParsedFile) -> usize {
        let line_comment = file.language.line_comment();
        let mut total = 0;
        let mut block_size = 0;
        for line in file.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(line_comment) {
                // Guard: skip doc-comments (///, /**) but do NOT skip //// (4-slash lines)
                if (trimmed.starts_with("///") && !trimmed.starts_with("////"))
                    || trimmed.starts_with("/**")
                {
                    if block_size > 0 {
                        total += block_size;
                        block_size = 0;
                    }
                    continue;
                }
                let text = trimmed.strip_prefix(line_comment).unwrap_or("").trim();
                let is_code = CODEC_PATTERNS.iter().any(|p| text.contains(p));
                if is_code {
                    block_size += 1;
                } else if block_size > 0 {
                    if block_size >= 3 {
                        total += block_size;
                    }
                    block_size = 0;
                }
            } else if !trimmed.is_empty() {
                if block_size >= 3 {
                    total += block_size;
                }
                block_size = 0;
            }
        }
        if block_size >= 3 {
            total += block_size;
        }
        total
    }

    /// Count TODO/FIXME/BUG/HACK markers in comments.
    /// Default implementation uses content-based detection.
    fn count_todo_markers(&self, file: &ParsedFile) -> usize {
        let line_comment = file.language.line_comment();
        let mut count = 0;
        for line in file.content.lines() {
            let trimmed = line.trim();
            if let Some(pos) = trimmed.find(line_comment) {
                // Ensure the comment marker is genuinely a comment, not a # inside a string
                // literal (e.g. `{"#TODO": "done"}` in Python). Real comments start at position
                // 0 or are preceded by a space/tab in the trimmed line (inline comment).
                if pos > 0 {
                    let prev = trimmed.as_bytes()[pos - 1];
                    if prev != b' ' && prev != b'\t' {
                        continue;
                    }
                }
                let comment = trimmed[pos + line_comment.len()..].trim().to_uppercase();
                if comment.starts_with("TODO")
                    || comment.contains(" TODO ")
                    || comment.starts_with("FIXME")
                    || comment.contains(" FIXME ")
                    || comment.starts_with("BUG")
                    || comment.contains(" BUG ")
                    || comment.starts_with("HACK")
                    || comment.contains(" HACK ")
                {
                    count += 1;
                }
            }
        }
        count
    }
}

const CODEC_PATTERNS: &[&str] = &[
    "fn ", "if ", "else", "for ", "while ", "struct ", "enum ", "impl ", "let ", "return ", "use ",
    "mod ", "break", "continue", "{", "}", "(", ")", "[", "]", ";", "=", "==", "!=", "&&", "||",
    "->", "::",
];

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
    count_block_ancestors, count_nested_blocks, count_params, get_node_text, is_boolean_or_null,
    is_common_safe_number, is_inside_declaration, is_repeating_chars, max_scope_depth,
    MEANINGLESS_NAMES,
};

#[cfg(test)]
pub(crate) use helpers::parse_code;
