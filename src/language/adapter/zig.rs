//! ZigAdapter — Zig language adapter.

use super::{
    count_params, is_boolean_or_null, is_common_safe_number, is_inside_declaration,
    is_repeating_chars, FunctionNode, LanguageAdapter, MEANINGLESS_NAMES,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

const ZIG_PATTERNS: &[&str] = &[
    "(builtin_function (builtin_identifier) @pc_f (#eq? @pc_f \"@panic\"))",
    "(function_declaration name: (identifier) @ex_name) @ex_fn",
    "(variable_declaration (identifier) @nv_var)",
    "(call_expression function: (field_expression member: (identifier) @dp_method (#eq? @dp_method \"print\")))",
    "(function_declaration (parameters) @ep_params)",
    "[(integer) @mn_num (float) @mn_num]",
];

pub struct ZigAdapter;

impl LanguageAdapter for ZigAdapter {
    fn language(&self) -> Language {
        Language::Zig
    }

    fn query_patterns(&self) -> &[&str] {
        ZIG_PATTERNS
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        self.count_panic_from_batch(file, &self.batch_captures(file))
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
    }

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize {
        fn zig_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
            let mut max = depth;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    let child_depth = match child.kind() {
                        "block" => depth + 1,
                        _ => depth,
                    };
                    max = max.max(zig_scope_depth(child, child_depth));
                }
            }
            max
        }
        zig_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        self.count_naming_from_batch(file, &self.batch_captures(file))
    }

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize {
        fn walk_zig_nodes(
            node: tree_sitter::Node,
            depth: usize,
            threshold: usize,
            count: &mut usize,
        ) {
            if node.kind() == "block" && depth >= threshold {
                *count += 1;
            }
            let child_depth = match node.kind() {
                "block" => depth + 1,
                _ => depth,
            };
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    walk_zig_nodes(child, child_depth, threshold, count);
                }
            }
        }
        let mut count = 0;
        walk_zig_nodes(file.root_node(), 0, 5, &mut count);
        count
    }

    fn count_debug_calls(&self, file: &ParsedFile) -> usize {
        self.count_debug_from_batch(file, &self.batch_captures(file))
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        self.count_excessive_from_batch_with(file, &self.batch_captures(file), threshold)
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        self.count_magic_from_batch(file, &self.batch_captures(file))
    }

    fn count_panic_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "pc_f"))
            .count()
    }

    fn extract_functions_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> Vec<FunctionNode> {
        let mut functions = Vec::new();
        for m in batch {
            let has_ex = m.iter().any(|c| c.name.starts_with("ex_"));
            if !has_ex {
                continue;
            }
            let mut name = String::new();
            let mut start_line = 0usize;
            let mut end_line = 0usize;
            for c in m {
                match c.name.as_str() {
                    "ex_name" => name = c.text.to_string(),
                    "ex_fn" => {
                        start_line = c.node.start_position().row + 1;
                        end_line = c.node.end_position().row + 1;
                    }
                    _ => {}
                }
            }
            if !name.is_empty() {
                functions.push(FunctionNode {
                    name,
                    start_line,
                    end_line,
                    nesting_depth: 0,
                });
            }
        }
        functions
    }

    fn count_naming_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let mut count = 0usize;
        static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
            Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();
        let idiomatic_single: &[&str] = &["i", "j", "k", "n", "e", "x"];

        for m in batch {
            for c in m {
                if c.name == "nv_var" {
                    let name = c.text;
                    if name.len() == 1 && name.chars().all(|ch| ch.is_ascii_lowercase()) {
                        if !idiomatic_single.contains(&name) {
                            count += 1;
                        }
                        continue;
                    }
                    if let Some(re) = terrible_re {
                        if re.is_match(&name.to_lowercase()) {
                            count += 1;
                            continue;
                        }
                    }
                    if MEANINGLESS_NAMES.contains(&name) || is_repeating_chars(name) {
                        count += 1;
                        continue;
                    }
                }
            }
        }
        count
    }

    fn count_debug_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "dp_method"))
            .count()
    }

    fn count_excessive_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_excessive_from_batch_with(_file, batch, 5)
    }

    fn count_magic_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let mut count = 0;
        for m in batch {
            for c in m {
                if c.name == "mn_num" && !is_inside_declaration(c.node) {
                    let text = c.text;
                    if text != "0"
                        && text != "1"
                        && text != "-1"
                        && !is_common_safe_number(text)
                        && !is_boolean_or_null(text)
                    {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

impl ZigAdapter {
    fn count_excessive_from_batch_with<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
        threshold: usize,
    ) -> usize {
        let mut count = 0;
        for m in batch {
            for c in m {
                if c.name == "ep_params" && count_params(c.text) > threshold {
                    count += 1;
                }
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse_code;
    use super::*;

    fn parse_zig(code: &str) -> ParsedFile {
        parse_code(code, "test.zig").expect("parse")
    }

    #[test]
    fn test_zig_count_panic_at_panic() {
        let code = r#"
fn main() void {
    @panic("boom");
    @panic("bang");
}
"#;
        let file = parse_zig(code);
        let adapter = ZigAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_zig_count_panic_clean() {
        let code = "fn add(x: i32) i32 { return x + 1; }\n";
        let file = parse_zig(code);
        let adapter = ZigAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_zig_extract_functions() {
        let code = r#"
fn foo() void {}
fn bar(x: i32) i32 { return x; }
"#;
        let file = parse_zig(code);
        let adapter = ZigAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
    }

    #[test]
    fn test_zig_naming_single_letter() {
        let code = r#"
fn main() void {
    const a: i32 = 1;
    var b: i32 = 2;
}
"#;
        let file = parse_zig(code);
        let adapter = ZigAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_zig_debug_print() {
        let code = r#"
const std = @import("std");
fn main() void {
    std.debug.print("hello", .{});
}
"#;
        let file = parse_zig(code);
        let adapter = ZigAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 1);
    }

    #[test]
    fn test_zig_excessive_params() {
        let code = "fn process(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) void {}\n";
        let file = parse_zig(code);
        let adapter = ZigAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1);
    }

    #[test]
    fn test_zig_magic_numbers() {
        let code = r#"
fn main() void {
    foo(41);
    bar(100);
}
"#;
        let file = parse_zig(code);
        let adapter = ZigAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_zig_magic_numbers_skips_trivial() {
        let code = "fn main() void { foo(0); bar(1); }\n";
        let file = parse_zig(code);
        let adapter = ZigAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }
}
