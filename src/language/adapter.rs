//! LanguageAdapter trait — unified semantic extraction from parsed AST.
//!
//! SignalDetectors delegate to LanguageAdapter instead of writing
//! per-language tree-sitter queries directly. This makes detectors
//! language-agnostic and consolidates query logic per language.

use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;

use regex::Regex;

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

    /// Count `.unwrap()`, `.expect()`, `panic!()` calls in a file.
    fn count_panic_calls(&self, file: &ParsedFile) -> usize;

    /// Extract all function/method definitions with metadata.
    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode>;

    /// Maximum nesting depth (scope blocks) across the file.
    fn max_nesting_depth(&self, file: &ParsedFile) -> usize;

    /// Count naming violations: single-letter vars, terrible/meaningless names,
    /// Hungarian notation, and abbreviation abuse.
    fn count_naming_violations(&self, file: &ParsedFile) -> usize;

    /// Count deeply-nested block scopes (nesting depth >= threshold).
    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize;
}

// ── Rust Adapter ──────────────────────────────────────────────────

pub struct RustAdapter;

impl LanguageAdapter for RustAdapter {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        let mut count = 0usize;

        if let Ok(groups) =
            collect_captures(file, "(field_expression field: (field_identifier) @method)")
        {
            for group in groups {
                if let Some(cap) = group.first() {
                    if cap.text == "unwrap" || cap.text == "expect" {
                        count += 1;
                    }
                }
            }
        }

        if let Ok(groups) = collect_captures(file, "(macro_invocation macro: (identifier) @m)") {
            for group in groups {
                if let Some(cap) = group.first() {
                    if cap.text == "panic" || cap.text == "panic!" {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        let mut functions = Vec::new();

        let pattern = "(function_item name: (identifier) @name) @fn";
        let Ok(groups) = collect_captures(file, pattern) else {
            return functions;
        };

        for group in &groups {
            let mut name = String::new();
            let mut start_line = 0usize;
            let mut end_line = 0usize;

            for cap in group {
                match cap.name.as_str() {
                    "name" => name = cap.text.to_string(),
                    "fn" => {
                        start_line = cap.node.start_position().row + 1;
                        end_line = cap.node.end_position().row + 1;
                    }
                    _ => {}
                }
            }

            if !name.is_empty() {
                let nesting_depth = count_block_ancestors(group);
                functions.push(FunctionNode {
                    name,
                    start_line,
                    end_line,
                    nesting_depth,
                });
            }
        }

        functions
    }

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize {
        max_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        let mut count = 0usize;

        // 1. Single-letter variables
        if let Ok(groups) = collect_captures(
            file,
            "(let_declaration pattern: (identifier) @var (#match? @var \"^[a-z]$\"))",
        ) {
            count += groups.len();
        }

        // 2. Terrible naming + meaningless naming (share the same query)
        let terrible_re = Regex::new(
            r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$",
        )
        .ok();
        let meaningless: &[&str] = &[
            "foo", "bar", "baz", "qux", "quux", "quuz", "aaa", "bbb", "ccc", "ddd", "eee", "xxx",
            "yyy", "zzz", "test1", "test2", "test3",
        ];

        if let Ok(groups) = collect_captures(file, "(let_declaration pattern: (identifier) @name)")
        {
            for group in &groups {
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    let name_lower = name.to_lowercase();
                    if let Some(ref re) = terrible_re {
                        if re.is_match(&name_lower) {
                            count += 1;
                            continue;
                        }
                    }
                    if meaningless.contains(&name) || is_repeating_chars(name) {
                        count += 1;
                        continue;
                    }
                }
            }
        }

        // 3. Hungarian notation + abbreviation abuse (share the all-identifiers query)
        let hungarian_prefixes: &[&str] = &[
            "str", "int", "bool", "float", "double", "char", "arr", "vec", "list", "map", "set",
        ];
        let scope_prefixes: &[&str] = &["g_", "m_", "s_", "p_"];
        let bad_abbrevs: &[&str] = &[
            "mgr", "mngr", "ctrl", "hdlr", "usr", "pwd", "prefs", "btn", "lbl", "pic", "tbl",
            "col", "cnt",
        ];

        if let Ok(groups) = collect_captures(file, "(identifier) @id") {
            for group in &groups {
                if count > 2000 {
                    break; // safety cap
                }
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    let name_lower = name.to_lowercase();

                    // Hungarian notation check
                    if scope_prefixes.iter().any(|p| name_lower.starts_with(p))
                        || hungarian_prefixes.iter().any(|p| {
                            name_lower.starts_with(p)
                                && name.len() > p.len()
                                && name.as_bytes()[p.len()].is_ascii_uppercase()
                        })
                    {
                        count += 1;
                        continue;
                    }

                    // Abbreviation abuse check
                    if bad_abbrevs
                        .iter()
                        .any(|a| name_lower == *a || name_lower.starts_with(&format!("{}_", a)))
                    {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize {
        let threshold = 5;
        let mut count = 0;
        count_nested_blocks(file.root_node(), 0, threshold, &mut count);
        count
    }
}

fn count_nested_blocks(node: tree_sitter::Node, depth: usize, threshold: usize, count: &mut usize) {
    if node.kind() == "block" && depth >= threshold {
        *count += 1;
    }
    let child_depth = match node.kind() {
        "block" => depth + 1,
        _ => depth,
    };
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            count_nested_blocks(child, child_depth, threshold, count);
        }
    }
}

fn max_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
    let mut max = depth;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            let child_depth = if is_scope_node(&child) {
                depth + 1
            } else {
                depth
            };
            max = max.max(max_scope_depth(child, child_depth));
        }
    }
    max
}

fn is_scope_node(node: &tree_sitter::Node) -> bool {
    matches!(node.kind(), "block")
}

fn is_repeating_chars(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    chars.len() >= 3 && chars.iter().all(|c| *c == chars[0])
}

fn count_block_ancestors(group: &[crate::treesitter::query::QueryCapture]) -> usize {
    if let Some(cap) = group.first() {
        let mut depth = 0usize;
        let mut current = Some(cap.node);
        while let Some(node) = current {
            if let Some(parent) = node.parent() {
                if parent.kind() == "block" {
                    depth += 1;
                }
                current = Some(parent);
            } else {
                break;
            }
        }
        depth
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::TreeSitterEngine;
    use std::path::Path;

    fn parse_rust(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(Path::new("test.rs"), code)
            .expect("parse")
    }

    // ── count_panic_calls ─────────────────────────────────────────

    /// Objective: Verify count_panic_calls detects .unwrap() and .expect().
    /// Invariants: Each call is counted once per occurrence.
    #[test]
    fn test_rust_count_panic_unwrap_expect() {
        let code = "fn main() { let x = foo().unwrap(); let y = bar().expect(\"msg\"); }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    /// Objective: Verify count_panic_calls detects panic!() macro.
    #[test]
    fn test_rust_count_panic_macro() {
        let code = "fn main() { panic!(\"boom\"); }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 1);
    }

    /// Objective: Verify count_panic_calls returns 0 for clean code.
    #[test]
    fn test_rust_count_panic_clean() {
        let code = "fn main() { let x = 42; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    // ── extract_functions ─────────────────────────────────────────

    /// Objective: Verify extract_functions finds function names and line ranges.
    /// Invariants: Each function_item is extracted with correct metadata.
    #[test]
    fn test_rust_extract_functions() {
        let code = r#"
fn foo() {}
fn bar(x: i32) -> i32 { x + 1 }
"#;
        let file = parse_rust(code);
        let adapter = RustAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 2, "should find 2 functions");
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
        assert!(fns[0].start_line < fns[1].start_line, "foo before bar");
    }

    // ── max_nesting_depth ─────────────────────────────────────────

    /// Objective: Verify max_nesting_depth returns 0 for top-level code.
    #[test]
    fn test_rust_max_nesting_depth_flat() {
        let code = "fn main() { let x = 1; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.max_nesting_depth(&file), 1);
    }

    /// Objective: Verify max_nesting_depth increases with nested blocks.
    #[test]
    fn test_rust_max_nesting_depth_nested() {
        let code = r#"
fn main() {
    if true {
        for i in 0..10 {
            let x = i;
        }
    }
}
"#;
        let file = parse_rust(code);
        let adapter = RustAdapter;
        let depth = adapter.max_nesting_depth(&file);
        assert!(
            depth >= 2,
            "nested if+for should have depth >= 2, got {depth}"
        );
    }

    /// Objective: Verify max_nesting_depth returns 0 for empty file.
    #[test]
    fn test_rust_max_nesting_depth_empty() {
        let code = "";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.max_nesting_depth(&file), 0);
    }

    // ── count_naming_violations ────────────────────────────────────

    /// Objective: Verify count_naming_violations detects single-letter vars.
    #[test]
    fn test_naming_single_letter() {
        let code = "fn main() { let a = 1; let bb = 2; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 1);
    }

    /// Objective: Verify count_naming_violations detects terrible naming.
    #[test]
    fn test_naming_terrible() {
        let code = "fn main() { let data = 1; let manager = 2; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    /// Objective: Verify count_naming_violations detects meaningless names.
    #[test]
    fn test_naming_meaningless() {
        let code = "fn main() { let foo = 1; let aaa = 2; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    /// Objective: Verify count_naming_violations detects Hungarian notation.
    #[test]
    fn test_naming_hungarian() {
        let code = "fn main() { let strName = \"hello\"; let g_count = 0; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    /// Objective: Verify count_naming_violations detects abbreviation abuse.
    #[test]
    fn test_naming_abbreviation() {
        let code = "fn main() { let mgr = \"boss\"; let btn_submit = true; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    /// Objective: Verify count_naming_violations returns 0 for clean code.
    #[test]
    fn test_naming_clean() {
        let code = "fn main() { let user_name = \"alice\"; let item_count = 42; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 0);
    }

    /// Objective: Verify is_repeating_chars returns false for short names.
    #[test]
    fn test_is_repeating_chars_short() {
        assert!(!is_repeating_chars("a"));
        assert!(!is_repeating_chars("ab"));
    }

    /// Objective: Verify is_repeating_chars detects repeating chars.
    #[test]
    fn test_is_repeating_chars_detects() {
        assert!(is_repeating_chars("aaa"));
        assert!(is_repeating_chars("bbb"));
        assert!(is_repeating_chars("zzz"));
    }

    /// Objective: Verify is_repeating_chars rejects non-repeating.
    #[test]
    fn test_is_repeating_chars_non_repeating() {
        assert!(!is_repeating_chars("abc"));
        assert!(!is_repeating_chars("aba"));
    }
}
