//! ZigAdapter — Zig language adapter.

use super::{is_inside_declaration, FunctionNode, LanguageAdapter};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use regex::Regex;

pub struct ZigAdapter;

impl LanguageAdapter for ZigAdapter {
    fn language(&self) -> Language {
        Language::Zig
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        let Ok(groups) = collect_captures(
            file,
            r#"(builtin_function (builtin_identifier) @f (#eq? @f "@panic"))"#,
        ) else {
            return 0;
        };
        groups.len()
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        let mut functions = Vec::new();
        let Ok(groups) =
            collect_captures(file, "(function_declaration name: (identifier) @name) @fn")
        else {
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
        let mut count = 0usize;
        let terrible_re = Regex::new(
            r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$",
        )
        .ok();

        if let Ok(groups) = collect_captures(file, "(variable_declaration (identifier) @var)") {
            for group in &groups {
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    if name.len() == 1 && name.chars().all(|c| c.is_ascii_lowercase()) {
                        count += 1;
                        continue;
                    }
                    if let Some(ref re) = terrible_re {
                        if re.is_match(&name.to_lowercase()) {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
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
        let Ok(groups) = collect_captures(
            file,
            r#"(call_expression function: (field_expression member: (identifier) @method (#eq? @method "print")))"#,
        ) else {
            return 0;
        };
        groups.len()
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        let Ok(groups) = collect_captures(file, "(function_declaration (parameters) @params)")
        else {
            return 0;
        };
        let mut count = 0;
        for group in &groups {
            for cap in group {
                if cap.name == "params" {
                    let param_count = cap.text.bytes().filter(|&b| b == b',').count() + 1;
                    if param_count > threshold {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        let Ok(captures) = collect_captures(file, "[(integer) @num (float) @num]") else {
            return 0;
        };
        let mut count = 0;
        for group in &captures {
            if let Some(cap) = group.first() {
                if !is_inside_declaration(cap.node) {
                    let text = cap.text;
                    if text != "0" && text != "1" && text != "-1" {
                        count += 1;
                    }
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
    const x: i32 = 1;
    var y: i32 = 2;
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
