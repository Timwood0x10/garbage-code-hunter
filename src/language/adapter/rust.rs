//! RustAdapter — Rust language adapter.

use super::{
    count_block_ancestors, count_nested_blocks, count_params, is_boolean_or_null,
    is_common_safe_number, is_inside_declaration, is_repeating_chars, max_scope_depth,
    FunctionNode, LanguageAdapter,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

/// Returns byte ranges of `#[cfg(test)]` modules in Rust source.
/// Each range covers from the opening brace of the module block to its closing brace.
fn cfg_test_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut search_from = 0;
    while let Some(attr_pos) = content[search_from..].find("#[cfg(test)]") {
        let attr_start = search_from + attr_pos;
        let after_attr = attr_start + "#[cfg(test)]".len();
        if let Some(brace_offset) = content[after_attr..].find('{') {
            let open_brace = after_attr + brace_offset;
            let mut depth = 1i32;
            let mut j = open_brace + 1;
            for ch in content[open_brace + 1..].chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                j += ch.len_utf8();
                if depth == 0 {
                    break;
                }
            }
            if depth == 0 {
                ranges.push((open_brace, j));
            }
            search_from = j;
        } else {
            search_from = after_attr;
        }
    }
    ranges
}

const RUST_PATTERNS: &[&str] = &[
    "(field_expression field: (field_identifier) @pc_method (#eq? @pc_method \"unwrap\"))",
    "(macro_invocation macro: (identifier) @pc_m)",
    "(function_item name: (identifier) @ex_name) @ex_fn",
    "(let_declaration pattern: (identifier) @nv_var (#match? @nv_var \"^[a-z]$\"))",
    "(let_declaration pattern: (identifier) @nv_name)",
    "(identifier) @nv_id",
    "(macro_invocation macro: (identifier) @dp_name (#match? @dp_name \"^(println|dbg|eprintln|eprint|todo|unimplemented)$\"))",
    "(function_item parameters: (parameters) @ep_params)",
    "(unsafe_block) @ub_unsafe",
    "(integer_literal) @mn_num",
];

pub struct RustAdapter;

impl LanguageAdapter for RustAdapter {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn query_patterns(&self) -> &[&str] {
        RUST_PATTERNS
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        self.count_panic_from_batch(file, &self.batch_captures(file))
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
    }

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize {
        max_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        self.count_naming_from_batch(file, &self.batch_captures(file))
    }

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize {
        let threshold = 5;
        let mut count = 0;
        count_nested_blocks(file.root_node(), 0, threshold, &mut count);
        count
    }

    fn count_debug_calls(&self, file: &ParsedFile) -> usize {
        self.count_debug_from_batch(file, &self.batch_captures(file))
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        self.count_excessive_from_batch_with(file, &self.batch_captures(file), threshold)
    }

    fn count_unsafe_blocks(&self, file: &ParsedFile) -> usize {
        self.count_unsafe_from_batch(file, &self.batch_captures(file))
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        self.count_magic_from_batch(file, &self.batch_captures(file))
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

    fn count_panic_from_batch<'a>(
        &self,
        file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let test_ranges = cfg_test_ranges(&file.content);
        let mut count = 0;
        for m in batch {
            for c in m {
                if (c.name == "pc_method" && c.text == "unwrap")
                    || (c.name == "pc_m"
                        && matches!(c.text, "panic" | "assert" | "assert_eq" | "assert_ne"))
                {
                    let byte_offset = c.node.start_byte();
                    if test_ranges
                        .iter()
                        .any(|&(s, e)| byte_offset >= s && byte_offset < e)
                    {
                        continue;
                    }
                    count += 1;
                }
            }
        }
        count
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
                let nesting_depth = count_block_ancestors(m);
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

    fn count_naming_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let mut count = 0usize;
        let idiomatic_single: &[&str] = &["i", "j", "k", "n", "c", "e", "x", "t", "f"];

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

        let hungarian_prefixes: &[&str] = &[
            "str", "int", "bool", "float", "double", "char", "arr", "vec", "list", "map", "set",
        ];
        let scope_prefixes: &[&str] = &["g_", "m_", "s_", "p_"];
        let domain_prefixes: &[&str] = &[
            "ctx", "req", "res", "err", "db", "kv", "fs", "io", "api", "http", "html", "ssh",
            "tls", "uid", "uri", "url",
        ];
        let bad_abbrevs: &[&str] = &[
            "mgr", "mngr", "ctrl", "hdlr", "usr", "pwd", "prefs", "btn", "lbl", "pic", "tbl",
            "col", "cnt",
        ];

        for m in batch {
            for c in m {
                match c.name.as_str() {
                    "nv_var" if !idiomatic_single.contains(&c.text) => {
                        count += 1;
                    }
                    "nv_name" => {
                        let name = c.text;
                        let name_lower = name.to_lowercase();
                        if let Some(re) = terrible_re {
                            if re.is_match(&name_lower) {
                                count += 1;
                                continue;
                            }
                        }
                        if meaningless.contains(&name) || is_repeating_chars(name) {
                            count += 1;
                        }
                    }
                    "nv_id" => {
                        if count > 2000 {
                            continue;
                        }
                        let name = c.text;
                        let name_lower = name.to_lowercase();
                        if domain_prefixes.iter().any(|p| name_lower.starts_with(p)) {
                            continue;
                        }
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
                    _ => {}
                }
            }
        }
        count
    }

    fn count_debug_from_batch<'a>(
        &self,
        file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let test_ranges = cfg_test_ranges(&file.content);
        batch
            .iter()
            .filter(|m| {
                m.iter().any(|c| {
                    if c.name != "dp_name" {
                        return false;
                    }
                    let byte_offset = c.node.start_byte();
                    !test_ranges
                        .iter()
                        .any(|&(s, e)| byte_offset >= s && byte_offset < e)
                })
            })
            .count()
    }

    fn count_excessive_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        self.count_excessive_from_batch_with(_file, batch, 5)
    }

    fn count_unsafe_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "ub_unsafe"))
            .count()
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

impl RustAdapter {
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

    fn parse_rust(code: &str) -> ParsedFile {
        parse_code(code, "test.rs").expect("parse")
    }

    #[test]
    fn test_rust_count_panic_unwrap_only() {
        let code = "fn main() { let x = foo().unwrap(); let y = bar().expect(\"msg\"); }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 1);
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
    fn test_naming_hungarian_exempts_domain_prefixes() {
        let code = "fn main() { let ctxUser = 1; let dbQuery = 2; let kvStore = 3; }";
        let file = parse_rust(code);
        let adapter = RustAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 0);
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

    #[test]
    fn test_rust_compute_all() {
        let code = r#"
fn main() {
    let x = foo().unwrap();
    panic!("boom");
    println!("debug");
    unsafe { let p = 42 as *const i32; }
    foo(100);
}
"#;
        let file = parse_rust(code);
        let adapter = RustAdapter;
        let counts = adapter.compute_all(&file);
        assert!(counts.panic_calls >= 2);
        assert!(counts.debug_calls >= 1);
        assert!(counts.unsafe_blocks >= 1);
        assert!(counts.magic_numbers >= 1);
    }
}
