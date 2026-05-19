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
use crate::treesitter::query::{collect_captures_multi, QueryCapture};

/// Metadata for a function extracted from source code.
#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub nesting_depth: usize,
}

/// All adapter-computed counts in a single batch result.
#[derive(Debug, Clone, Default)]
pub struct AdapterCounts {
    pub functions: Vec<FunctionNode>,
    pub panic_calls: usize,
    pub naming_violations: usize,
    pub deeply_nested_blocks: usize,
    pub debug_calls: usize,
    pub excessive_params: usize,
    pub unsafe_blocks: usize,
    pub magic_numbers: usize,
    pub commented_out_lines: usize,
    pub todo_markers: usize,
    pub goroutine_spawns: usize,
    pub defer_in_loop: usize,
    pub go_conventions: usize,
    pub python_issues: usize,
    pub java_issues: usize,
    pub ruby_issues: usize,
    pub c_issues: usize,
    pub ts_issues: usize,
    pub dead_code: usize,
    pub duplicate_imports: usize,
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
                if is_code || block_size > 0 {
                    block_size += 1;
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

    /// Return merged tree-sitter query patterns for this language.
    ///
    /// Each pattern must use prefixed capture names (e.g. `@pc_method`)
    /// to avoid collisions across patterns. Override in each adapter.
    fn query_patterns(&self) -> &[&str] {
        &[]
    }

    /// Run all query patterns in a single cursor traversal.
    ///
    /// Default implementation calls `collect_captures_multi` with
    /// the patterns from `query_patterns()`.
    fn batch_captures<'a>(&self, file: &'a ParsedFile) -> Vec<Vec<QueryCapture<'a>>> {
        let patterns = self.query_patterns();
        if patterns.is_empty() {
            return Vec::new();
        }
        collect_captures_multi(file, patterns).unwrap_or_default()
    }

    /// Compute all adapter counts in a single batch pass.
    ///
    /// This is the main entry point for `StyleIr::from_parsed()`.
    /// It calls `batch_captures()` once and passes the result to
    /// `_from_batch` helper methods, avoiding redundant traversals.
    fn compute_all(&self, file: &ParsedFile) -> AdapterCounts {
        let batch = self.batch_captures(file);
        AdapterCounts {
            functions: self.extract_functions_from_batch(file, &batch),
            panic_calls: self.count_panic_from_batch(file, &batch),
            naming_violations: self.count_naming_from_batch(file, &batch),
            deeply_nested_blocks: self.count_deeply_nested_blocks(file),
            debug_calls: self.count_debug_from_batch(file, &batch),
            excessive_params: self.count_excessive_from_batch(file, &batch),
            unsafe_blocks: self.count_unsafe_from_batch(file, &batch),
            magic_numbers: self.count_magic_from_batch(file, &batch),
            commented_out_lines: self.count_commented_out_code(file),
            todo_markers: self.count_todo_markers(file),
            goroutine_spawns: self.count_goroutine_from_batch(file, &batch),
            defer_in_loop: self.count_defer_in_loop(file),
            go_conventions: self.count_go_convention_from_batch(file, &batch),
            python_issues: self.count_python_from_batch(file, &batch),
            java_issues: self.count_java_from_batch(file, &batch),
            ruby_issues: self.count_ruby_from_batch(file, &batch),
            c_issues: self.count_c_from_batch(file, &batch),
            ts_issues: self.count_ts_from_batch(file, &batch),
            dead_code: self.count_dead_code(file),
            duplicate_imports: self.count_duplicate_imports(file),
        }
    }

    fn extract_functions_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> Vec<FunctionNode> {
        self.extract_functions(file)
    }

    fn count_panic_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_panic_calls(file)
    }

    fn count_naming_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_naming_violations(file)
    }

    fn count_debug_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_debug_calls(file)
    }

    fn count_excessive_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_excessive_params(file, 5)
    }

    fn count_unsafe_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_unsafe_blocks(file)
    }

    fn count_magic_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_magic_numbers(file)
    }

    fn count_goroutine_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_goroutine_spawns(file)
    }

    fn count_go_convention_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_go_convention_violations(file)
    }

    fn count_python_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_python_issues(file)
    }

    fn count_java_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_java_issues(file)
    }

    fn count_ruby_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_ruby_issues(file)
    }

    fn count_c_from_batch<'a>(&self, file: &ParsedFile, _batch: &[Vec<QueryCapture<'a>>]) -> usize {
        self.count_c_issues(file)
    }

    fn count_ts_from_batch<'a>(
        &self,
        file: &ParsedFile,
        _batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_ts_issues(file)
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
    count_block_ancestors, count_nested_blocks, count_params, is_boolean_or_null,
    is_common_safe_number, is_inside_declaration, is_repeating_chars, max_scope_depth,
    MEANINGLESS_NAMES,
};

#[cfg(test)]
pub(crate) use helpers::parse_code;
