//! SwiftAdapter — Swift language adapter.

use super::{
    count_dead_code_with, count_duplicate_imports_with, is_boolean_or_null, is_common_safe_number,
    is_inside_declaration, is_repeating_chars, FunctionNode, LanguageAdapter, MEANINGLESS_NAMES,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

const SWIFT_PATTERNS: &[&str] = &[
    "(call_expression (simple_identifier) @pc_f (#match? @pc_f \"^(fatalError|preconditionFailure|assert|assertionFailure|precondition)$\"))",
    "(function_declaration (simple_identifier) @ex_name) @ex_fn",
    "(property_declaration (pattern (simple_identifier) @nv_var))",
    "(call_expression (simple_identifier) @dp_f (#match? @dp_f \"^(print|debugPrint|dump|NSLog)$\"))",
    "(function_declaration) @ep_fn",
    "[(integer_literal) @mn_num (real_literal) @mn_num]",
];

pub struct SwiftAdapter;

impl LanguageAdapter for SwiftAdapter {
    fn language(&self) -> Language {
        Language::Swift
    }

    fn query_patterns(&self) -> &[&str] {
        SWIFT_PATTERNS
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        self.count_panic_from_batch(file, &self.batch_captures(file))
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
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
        self.count_naming_from_batch(file, &self.batch_captures(file))
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
        self.count_debug_from_batch(file, &self.batch_captures(file))
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        self.count_excessive_from_batch_with(file, &self.batch_captures(file), threshold)
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        self.count_magic_from_batch(file, &self.batch_captures(file))
    }

    fn count_dead_code(&self, file: &ParsedFile) -> usize {
        count_dead_code_with(
            file,
            &["return", "break", "continue"],
            &["return ", "throw ", "fatalError(", "preconditionFailure("],
            "//",
        )
    }

    fn count_duplicate_imports(&self, file: &ParsedFile) -> usize {
        count_duplicate_imports_with(file, &["import "])
    }

    fn count_swift_issues(&self, file: &ParsedFile) -> usize {
        let mut count = 0;
        for line in file.content.lines() {
            let t = line.trim();
            if t.starts_with("//") || t.starts_with("/*") || t.starts_with("*") {
                continue;
            }
            if t.contains("try!") {
                count += 1;
            }
            if t.contains("as!") {
                count += 1;
            }
        }
        count
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
            .filter(|m| m.iter().any(|c| c.name == "dp_f"))
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

impl SwiftAdapter {
    fn count_excessive_from_batch_with<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
        threshold: usize,
    ) -> usize {
        let mut count = 0;
        for m in batch {
            for c in m {
                if c.name == "ep_fn" {
                    let text = c.text;
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

    #[test]
    fn test_swift_panic_assert() {
        let code = "func main() { assert(x > 0); precondition(y != nil) }\n";
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_swift_debug_nslog() {
        let code = "NSLog(\"hello\")\n";
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 1);
    }

    #[test]
    fn test_swift_issues_try_bang() {
        let code = "let data = try! Data(contentsOf: url)\n";
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_swift_issues(&file), 1);
    }

    #[test]
    fn test_swift_issues_clean() {
        let code = "let x = 1\nlet y = 2\n";
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_swift_issues(&file), 0);
    }

    #[test]
    fn test_swift_dead_code_after_return() {
        let code = r#"
func foo() -> Int {
    return 42
    print("dead")
}
"#;
        let file = parse_swift(code);
        let adapter = SwiftAdapter;
        assert_eq!(adapter.count_dead_code(&file), 1);
    }
}
