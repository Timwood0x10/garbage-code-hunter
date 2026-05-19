//! SwiftAdapter — Swift language adapter.

use super::{
    is_common_safe_number, is_inside_declaration, is_repeating_chars, FunctionNode,
    LanguageAdapter, MEANINGLESS_NAMES,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use regex::Regex;
use std::sync::LazyLock;

pub struct SwiftAdapter;

impl LanguageAdapter for SwiftAdapter {
    fn language(&self) -> Language {
        Language::Swift
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        let Ok(groups) = collect_captures(
            file,
            r#"(call_expression (simple_identifier) @f (#match? @f "^(fatalError|preconditionFailure)$"))"#,
        ) else {
            return 0;
        };
        groups.len()
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        let mut functions = Vec::new();
        let Ok(groups) =
            collect_captures(file, "(function_declaration (simple_identifier) @name) @fn")
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
        fn swift_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
            let mut max = depth;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    let child_depth = match child.kind() {
                        "function_body" => depth + 1,
                        _ => depth,
                    };
                    max = max.max(swift_scope_depth(child, child_depth));
                }
            }
            max
        }
        swift_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        let mut count = 0usize;
        static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
            Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();
        // Language-idiomatic single-letter names exempt from counting
        let idiomatic_single: &[&str] = &["i", "j", "k", "n", "e", "x"];

        if let Ok(groups) = collect_captures(
            file,
            "(property_declaration (pattern (simple_identifier) @var))",
        ) {
            for group in &groups {
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    if name.len() == 1 && name.chars().all(|c| c.is_ascii_lowercase()) {
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

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize {
        fn walk_swift_nodes(
            node: tree_sitter::Node,
            depth: usize,
            threshold: usize,
            count: &mut usize,
        ) {
            if node.kind() == "function_body" && depth >= threshold {
                *count += 1;
            }
            let child_depth = match node.kind() {
                "function_body" => depth + 1,
                _ => depth,
            };
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    walk_swift_nodes(child, child_depth, threshold, count);
                }
            }
        }
        let mut count = 0;
        walk_swift_nodes(file.root_node(), 0, 5, &mut count);
        count
    }

    fn count_debug_calls(&self, file: &ParsedFile) -> usize {
        let Ok(groups) = collect_captures(
            file,
            r#"(call_expression (simple_identifier) @f (#match? @f "^(print|debugPrint|dump)$"))"#,
        ) else {
            return 0;
        };
        groups.len()
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        let Ok(groups) = collect_captures(file, "(function_declaration) @fn") else {
            return 0;
        };
        let mut count = 0;
        for group in &groups {
            for cap in group {
                let text = cap.text;
                let params = text.split('(').nth(1).and_then(|s| s.rsplit(')').nth(1));
                if let Some(p) = params {
                    let p = p.trim();
                    if p.is_empty() {
                        continue;
                    }
                    let param_count = p.split(',').count();
                    if param_count > threshold {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        let Ok(captures) = collect_captures(file, "[(integer_literal) @num (real_literal) @num]")
        else {
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
}

#[cfg(test)]
mod tests {
    use super::super::parse_code;
    use super::*;

    fn parse_swift(code: &str) -> ParsedFile {
        parse_code(code, "test.swift").expect("parse")
    }

    #[test]
    fn test_swift_count_panic_fatal_error() {
        let code = r#"
func main() {
    fatalError("boom")
    preconditionFailure("bad")
}
"#;
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_swift_count_panic_clean() {
        let code = "func add(x: Int) -> Int { return x + 1 }\n";
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_swift_extract_functions() {
        let code = r#"
func foo() {}
func bar(x: Int) -> Int { return x }
"#;
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
    }

    #[test]
    fn test_swift_naming_single_letter() {
        let code = r#"
func main() {
    let a = 1
    var b = 2
}
"#;
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_swift_debug_print() {
        let code = r#"
print("hello")
debugPrint(x)
dump(obj)
"#;
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 3);
    }

    #[test]
    fn test_swift_excessive_params() {
        let code = "func process(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int) {}\n";
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1);
    }

    #[test]
    fn test_swift_magic_numbers() {
        let code = r#"
func main() {
    foo(41)
    bar(100)
}
"#;
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_swift_magic_numbers_skips_trivial() {
        let code = "func main() { foo(0); bar(1) }\n";
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }
}
