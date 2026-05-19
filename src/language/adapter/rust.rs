//! RustAdapter — Rust language adapter.

use super::{
    count_block_ancestors, count_nested_blocks, count_params, is_common_safe_number,
    is_inside_declaration, is_repeating_chars, max_scope_depth, FunctionNode, LanguageAdapter,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use regex::Regex;
use std::sync::LazyLock;

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
                    if matches!(cap.text, "panic" | "assert" | "assert_eq" | "assert_ne") {
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
        // Language-idiomatic single-letter names exempt from counting
        let idiomatic_single: &[&str] = &["i", "j", "k", "n", "c", "e", "x", "t", "f"];

        if let Ok(groups) = collect_captures(
            file,
            "(let_declaration pattern: (identifier) @var (#match? @var \"^[a-z]$\"))",
        ) {
            for group in &groups {
                if let Some(cap) = group.first() {
                    if !idiomatic_single.contains(&cap.text) {
                        count += 1;
                    }
                }
            }
        }

        static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
            Regex::new(
                r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$",
            ).ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();
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
                    if let Some(re) = terrible_re {
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
                    break;
                }
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    let name_lower = name.to_lowercase();

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

    fn count_debug_calls(&self, file: &ParsedFile) -> usize {
        let mut count = 0;
        if let Ok(groups) = collect_captures(
            file,
            "(macro_invocation macro: (identifier) @name (#match? @name \"^(println|dbg|eprintln|eprint|todo|unimplemented)$\"))",
        ) {
            count += groups.len();
        }
        count
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        let mut count = 0;
        if let Ok(groups) =
            collect_captures(file, "(function_item parameters: (parameters) @params)")
        {
            for group in &groups {
                for cap in group {
                    if cap.name == "params" {
                        let param_count = count_params(cap.text);
                        if param_count > threshold {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }

    fn count_unsafe_blocks(&self, file: &ParsedFile) -> usize {
        let pattern = "(unsafe_block) @unsafe";
        collect_captures(file, pattern)
            .map(|g| g.len())
            .unwrap_or(0)
    }

    fn has_test_nodes(&self, file: &ParsedFile) -> bool {
        // Detect `#[test]` attribute on individual functions (outside #[cfg(test)] blocks)
        collect_captures(
            file,
            "(attribute_item (attribute) @attr (#eq? @attr \"test\"))",
        )
        .map(|g| !g.is_empty())
        .unwrap_or(false)
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        let Ok(captures) = collect_captures(file, "(integer_literal) @num") else {
            return 0;
        };
        let mut count = 0;
        for group in &captures {
            if let Some(cap) = group.first() {
                if !is_inside_declaration(cap.node) {
                    let text = cap.text;
                    if text != "0" && text != "1" && text != "-1" && !is_common_safe_number(text) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn count_dead_code(&self, file: &ParsedFile) -> usize {
        let mut count = 0;
        let mut dead_start: Option<usize> = None;
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if matches!(
                trimmed,
                "return;" | "break;" | "continue;" | "unreachable!()" | "unreachable!();"
            ) || (trimmed.starts_with("return ") && trimmed.ends_with(';'))
                || (trimmed.starts_with("panic!(") && trimmed.ends_with(';'))
                || (trimmed.starts_with("unreachable!(") && trimmed.ends_with(')'))
            {
                dead_start = Some(line_num + 2);
                continue;
            }
            if let Some(start) = dead_start {
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
                    continue;
                }
                if trimmed == "}"
                    || trimmed.starts_with("} else")
                    || trimmed.starts_with("} else if")
                {
                    dead_start = None;
                    continue;
                }
                if line_num + 1 >= start {
                    count += 1;
                    dead_start = None;
                }
            }
        }
        count
    }

    fn count_duplicate_imports(&self, file: &ParsedFile) -> usize {
        let mut seen = std::collections::HashSet::new();
        let mut count = 0;
        for line in file.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") && !seen.insert(trimmed.to_string()) {
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse_code;
    use super::*;

    fn parse_rust(code: &str) -> ParsedFile {
        parse_code(code, "test.rs").expect("parse")
    }

    #[test]
    fn test_rust_count_panic_unwrap_expect() {
        let code = "fn main() { let x = foo().unwrap(); let y = bar().expect(\"msg\"); }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_rust_count_panic_macro() {
        let code = "fn main() { panic!(\"boom\"); }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 1);
    }

    #[test]
    fn test_rust_count_panic_clean() {
        let code = "fn main() { let x = 42; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

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

    #[test]
    fn test_rust_max_nesting_depth_flat() {
        let code = "fn main() { let x = 1; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.max_nesting_depth(&file), 1);
    }

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

    #[test]
    fn test_rust_max_nesting_depth_empty() {
        let code = "";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.max_nesting_depth(&file), 0);
    }

    #[test]
    fn test_naming_single_letter() {
        let code = "fn main() { let a = 1; let bb = 2; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 1);
    }

    #[test]
    fn test_naming_terrible() {
        let code = "fn main() { let data = 1; let manager = 2; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_naming_meaningless() {
        let code = "fn main() { let foo = 1; let aaa = 2; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_naming_hungarian() {
        let code = "fn main() { let strName = \"hello\"; let g_count = 0; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_naming_abbreviation() {
        let code = "fn main() { let mgr = \"boss\"; let btn_submit = true; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_naming_clean() {
        let code = "fn main() { let user_name = \"alice\"; let item_count = 42; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 0);
    }

    #[test]
    fn test_rust_count_unsafe_blocks() {
        let code = r#"
fn main() {
    unsafe {
        let p = 42 as *const i32;
    }
    unsafe {
        let _ = 0usize;
    }
}
"#;
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_unsafe_blocks(&file), 2);
    }

    #[test]
    fn test_rust_count_unsafe_blocks_clean() {
        let code = "fn main() { let x = 42; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_unsafe_blocks(&file), 0);
    }

    #[test]
    fn test_rust_count_magic_numbers() {
        let code = r#"
fn main() {
    let x = 1;
    foo(42);
    bar(100);
}
"#;
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_rust_count_magic_numbers_const_ok() {
        let code = r#"
const MAX: i32 = 100;
fn main() {
    let x = MAX;
}
"#;
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }

    #[test]
    fn test_rust_count_magic_numbers_skips_trivial() {
        let code = r#"
fn main() {
    let x = 0;
    let y = x + 1;
}
"#;
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }
}
