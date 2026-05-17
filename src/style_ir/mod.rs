//! Style IR — language-neutral style facts extracted from parsed source.
//!
//! This module is intentionally smaller than a general AST. It only stores
//! facts needed by scoring, signal detection, and friend-style feedback.

use crate::language::adapter::{adapter_for, FunctionNode};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use serde::Serialize;

/// Stable threshold facts included in the Style IR summary output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StyleIrThresholdSummary {
    pub excessive_param_threshold: usize,
    pub god_function_line_threshold: usize,
}

/// Stable JSON-ready summary of a Style IR snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StyleIrSummary {
    pub language: String,
    pub line_count: usize,
    pub function_count: usize,
    pub god_function_count: usize,
    pub panic_call_count: usize,
    pub naming_violation_count: usize,
    pub deeply_nested_block_count: usize,
    pub debug_call_count: usize,
    pub excessive_param_count: usize,
    pub unsafe_block_count: usize,
    pub magic_number_count: usize,
    pub over_engineering_count: usize,
    pub code_smell_count: usize,
    pub is_clean_signal_baseline: bool,
    pub thresholds: StyleIrThresholdSummary,
}

/// Language-neutral style facts for one parsed source file.
///
/// Style IR stores observed facts, not user-facing interpretation. Detector
/// methods may combine facts into weighted signals, but the public fields stay
/// close to adapter-extracted source evidence.
#[derive(Debug, Clone)]
pub struct StyleIr {
    /// Source language used by the adapter that produced this IR.
    pub language: Language,

    /// Physical source line count measured from the original file content.
    pub line_count: usize,

    /// Function facts required by structure-level style detectors.
    pub functions: Vec<FunctionNode>,

    /// Count of panic-prone calls or macros such as unwrap, expect, and panic.
    pub panic_call_count: usize,

    /// Count of adapter-defined naming violations such as unclear identifiers.
    pub naming_violation_count: usize,

    /// Count of block nodes that cross the nesting threshold.
    pub deeply_nested_block_count: usize,

    /// Count of debug or temporary-output calls such as println and dbg.
    pub debug_call_count: usize,

    /// Count of functions whose parameter count exceeds the stable threshold.
    pub excessive_param_count: usize,

    /// Count of explicit unsafe blocks observed in the source file.
    pub unsafe_block_count: usize,

    /// Count of literal numbers that adapters classify as magic numbers.
    pub magic_number_count: usize,
}

impl StyleIr {
    const EXCESSIVE_PARAM_THRESHOLD: usize = 5;
    const GOD_FUNCTION_LINE_THRESHOLD: usize = 50;

    /// Build Style IR from a tree-sitter parsed file.
    ///
    /// Returns `None` when a language has no semantic adapter yet. Callers can
    /// keep using legacy rule logic while individual detectors migrate to IR.
    pub fn from_parsed(file: &ParsedFile) -> Option<Self> {
        let adapter = adapter_for(file.language)?;
        Some(Self {
            language: file.language,
            line_count: file.content.lines().count(),
            functions: adapter.extract_functions(file),
            panic_call_count: adapter.count_panic_calls(file),
            naming_violation_count: adapter.count_naming_violations(file),
            deeply_nested_block_count: adapter.count_deeply_nested_blocks(file),
            debug_call_count: adapter.count_debug_calls(file),
            excessive_param_count: adapter
                .count_excessive_params(file, Self::EXCESSIVE_PARAM_THRESHOLD),
            unsafe_block_count: adapter.count_unsafe_blocks(file),
            magic_number_count: adapter.count_magic_numbers(file),
        })
    }

    /// Count functions that exceed the project-level god-function threshold.
    pub fn god_function_count(&self) -> usize {
        self.functions
            .iter()
            .filter(|function| {
                function.end_line.saturating_sub(function.start_line)
                    > Self::GOD_FUNCTION_LINE_THRESHOLD
            })
            .count()
    }

    /// Count the combined over-engineering signal violations.
    pub fn over_engineering_count(&self) -> usize {
        self.god_function_count() + self.excessive_param_count
    }

    /// Count the combined code-smell signal violations.
    pub fn code_smell_count(&self) -> usize {
        self.unsafe_block_count * 2 + self.magic_number_count
    }

    /// Build a stable, JSON-ready summary for downstream consumers.
    pub fn summary(&self) -> StyleIrSummary {
        StyleIrSummary {
            language: self.language.display_name().to_string(),
            line_count: self.line_count,
            function_count: self.functions.len(),
            god_function_count: self.god_function_count(),
            panic_call_count: self.panic_call_count,
            naming_violation_count: self.naming_violation_count,
            deeply_nested_block_count: self.deeply_nested_block_count,
            debug_call_count: self.debug_call_count,
            excessive_param_count: self.excessive_param_count,
            unsafe_block_count: self.unsafe_block_count,
            magic_number_count: self.magic_number_count,
            over_engineering_count: self.over_engineering_count(),
            code_smell_count: self.code_smell_count(),
            is_clean_signal_baseline: self.is_clean_signal_baseline(),
            thresholds: StyleIrThresholdSummary {
                excessive_param_threshold: Self::EXCESSIVE_PARAM_THRESHOLD,
                god_function_line_threshold: Self::GOD_FUNCTION_LINE_THRESHOLD,
            },
        }
    }

    /// Return true when the IR has no extracted style signals.
    pub fn is_clean_signal_baseline(&self) -> bool {
        self.panic_call_count == 0
            && self.naming_violation_count == 0
            && self.deeply_nested_block_count == 0
            && self.debug_call_count == 0
            && self.excessive_param_count == 0
            && self.unsafe_block_count == 0
            && self.magic_number_count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::engine::TreeSitterEngine;
    use std::path::PathBuf;

    fn parse_rust(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(&PathBuf::from("sample.rs"), code)
            .expect("Rust parser should parse valid source")
    }

    /// Objective: Verify Style IR extracts panic-related facts from Rust code.
    /// Invariants: Panic call count is language-neutral and line count is stable.
    #[test]
    fn test_style_ir_panic_count() {
        let file = parse_rust("fn main() { value.unwrap(); panic!(\"boom\"); }");
        let ir = StyleIr::from_parsed(&file).expect("Rust should have a style adapter");

        assert_eq!(ir.language, Language::Rust);
        assert_eq!(ir.line_count, 1);
        assert_eq!(ir.panic_call_count, 2);
    }

    /// Objective: Verify clean code has no baseline signal counts.
    /// Invariants: A simple function should not emit direct style signals.
    #[test]
    fn test_style_ir_clean_baseline() {
        let file = parse_rust("fn add(left: i32, right: i32) -> i32 { left + right }");
        let ir = StyleIr::from_parsed(&file).expect("Rust should have a style adapter");

        assert!(ir.is_clean_signal_baseline());
    }

    /// Objective: Verify Style IR extracts naming violations from Rust code.
    /// Invariants: A single-letter local variable is counted exactly once.
    #[test]
    fn test_style_ir_naming_count() {
        let file = parse_rust("fn main() { let x = 1; }");
        let ir = StyleIr::from_parsed(&file).expect("Rust should have a style adapter");

        assert_eq!(ir.naming_violation_count, 1);
    }

    /// Objective: Verify Style IR extracts deeply nested block counts.
    /// Invariants: Six nested blocks must cross the direct signal threshold.
    #[test]
    fn test_style_ir_nested_count() {
        let file = parse_rust(
            r#"
fn main() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            let value = 1;
                        }
                    }
                }
            }
        }
    }
}
"#,
        );
        let ir = StyleIr::from_parsed(&file).expect("Rust should have a style adapter");

        assert!(
            ir.deeply_nested_block_count >= 1,
            "deep nesting should produce at least one style fact"
        );
    }

    /// Objective: Verify Style IR extracts debug-output style facts.
    /// Invariants: println! and dbg! each contribute one debug call.
    #[test]
    fn test_style_ir_debug_count() {
        let file = parse_rust(
            r#"
fn main() {
    println!("hello");
    dbg!(42);
}
"#,
        );
        let ir = StyleIr::from_parsed(&file).expect("Rust should have a style adapter");

        assert_eq!(ir.debug_call_count, 2);
    }

    /// Objective: Verify Style IR preserves excessive-parameter threshold semantics.
    /// Invariants: Six parameters must count as one over-engineering signal.
    #[test]
    fn test_style_ir_over_engineering_count() {
        let file = parse_rust(
            r#"
fn process(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    a + b + c + d + e + f
}
"#,
        );
        let ir = StyleIr::from_parsed(&file).expect("Rust should have a style adapter");

        assert_eq!(ir.excessive_param_count, 1);
        assert_eq!(ir.over_engineering_count(), 1);
    }

    /// Objective: Verify Style IR preserves code-smell weighting.
    /// Invariants: Unsafe blocks count double and magic numbers count once.
    #[test]
    fn test_style_ir_code_smell_count() {
        let file = parse_rust(
            r#"
fn main() {
    unsafe {
        let ptr = 42 as *const i32;
        let _ = *ptr;
    }
    foo(100);
}
"#,
        );
        let ir = StyleIr::from_parsed(&file).expect("Rust should have a style adapter");

        assert!(ir.unsafe_block_count >= 1);
        assert!(ir.magic_number_count >= 1);
        assert_eq!(
            ir.code_smell_count(),
            ir.unsafe_block_count * 2 + ir.magic_number_count
        );
    }

    /// Objective: Verify the Style IR summary exposes stable JSON-ready fields.
    /// Invariants: Summary counts must mirror the underlying Style IR snapshot.
    #[test]
    fn test_style_ir_summary_schema() {
        let file = parse_rust(
            r#"
fn process(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    unsafe {
        let value = 42;
        value + a + b + c + d + e + f
    }
}
"#,
        );
        let ir = StyleIr::from_parsed(&file).expect("Rust should have a style adapter");
        let summary = ir.summary();

        assert_eq!(summary.language, "Rust");
        assert_eq!(summary.line_count, ir.line_count);
        assert_eq!(summary.function_count, ir.functions.len());
        assert_eq!(summary.god_function_count, ir.god_function_count());
        assert_eq!(summary.excessive_param_count, ir.excessive_param_count);
        assert_eq!(summary.unsafe_block_count, ir.unsafe_block_count);
        assert_eq!(summary.code_smell_count, ir.code_smell_count());
        assert_eq!(summary.over_engineering_count, ir.over_engineering_count());
        assert_eq!(summary.thresholds.excessive_param_threshold, 5);
        assert_eq!(summary.thresholds.god_function_line_threshold, 50);

        let json = serde_json::to_value(&summary).expect("summary should serialize");
        assert!(
            json.get("language").is_some(),
            "summary JSON should include language"
        );
        assert!(
            json.get("thresholds").is_some(),
            "summary JSON should include thresholds"
        );
    }
}
