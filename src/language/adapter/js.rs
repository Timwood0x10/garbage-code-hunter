//! JSAdapter — JavaScript language adapter.

use super::{
    count_nested_blocks, count_params, is_boolean_or_null, is_common_safe_number,
    is_inside_declaration, is_repeating_chars, FunctionNode, LanguageAdapter, MEANINGLESS_NAMES,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

const JS_PATTERNS: &[&str] = &[
    // pc_ — panic calls (throw)
    "(throw_statement) @pc_throw",
    // ex_ — extract functions
    "[(function_declaration name: (identifier) @ex_name) (method_definition name: (property_identifier) @ex_name)] @ex_fn",
    // nv_ — naming violations
    "(variable_declarator name: (identifier) @nv_var)",
    // dp_ — debug calls
    r#"(call_expression function: (member_expression property: (property_identifier) @dp_method) (#match? @dp_method "^(log|debug|warn|error|info|trace)$"))"#,
    "(debugger_statement) @dp_debug",
    // ep_ — excessive params
    "[(function_declaration parameters: (formal_parameters) @ep_params) (arrow_function parameters: (formal_parameters) @ep_params) (method_definition parameters: (formal_parameters) @ep_params)]",
    // mn_ — magic numbers
    "(number) @mn_num",
];

pub struct JSAdapter;

impl LanguageAdapter for JSAdapter {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn query_patterns(&self) -> &[&str] {
        JS_PATTERNS
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        self.count_panic_from_batch(file, &self.batch_captures(file))
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
    }

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize {
        fn js_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
            let mut max = depth;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    let child_depth = match child.kind() {
                        "block" => depth + 1,
                        _ => depth,
                    };
                    max = max.max(js_scope_depth(child, child_depth));
                }
            }
            max
        }
        js_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        self.count_naming_from_batch(file, &self.batch_captures(file))
    }

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize {
        let mut count = 0;
        count_nested_blocks(file.root_node(), 0, 5, &mut count);
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

    // -- _from_batch overrides --

    fn count_panic_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "pc_throw"))
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
            Regex::new(
                r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$",
            ).ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();
        let idiomatic_single: &[&str] = &["i", "j", "k", "e", "x", "y"];

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
            .filter(|m| {
                m.iter()
                    .any(|c| c.name == "dp_method" || c.name == "dp_debug")
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

impl JSAdapter {
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

    fn parse_js(code: &str) -> ParsedFile {
        parse_code(code, "test.js").expect("parse")
    }

    #[test]
    fn test_js_count_panic_throw() {
        let code = r#"
function main() {
    throw new Error("boom");
    throw "bang";
}
"#;
        let file = parse_js(code);
        let adapter = JSAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_js_count_panic_clean() {
        let code = "function main() { return 42; }";
        let file = parse_js(code);
        let adapter = JSAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_js_extract_functions() {
        let code = r#"
function foo() {}
function bar(x) { return x; }
const obj = { baz() {} };
"#;
        let file = parse_js(code);
        let adapter = JSAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 3);
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
        assert_eq!(fns[2].name, "baz");
    }

    #[test]
    fn test_js_naming_single_letter() {
        let code = r#"
function main() {
    let a = 1;
    let b = 2;
}
"#;
        let file = parse_js(code);
        let adapter = JSAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_js_debug_console_log() {
        let code = r#"
console.log("hello");
console.error("bad");
debugger;
"#;
        let file = parse_js(code);
        let adapter = JSAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 3);
    }

    #[test]
    fn test_js_excessive_params() {
        let code = "function process(a, b, c, d, e, f) {}\n";
        let file = parse_js(code);
        let adapter = JSAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1);
    }

    #[test]
    fn test_js_magic_numbers() {
        let code = r#"
function main() {
    foo(41);
    bar(100);
}
"#;
        let file = parse_js(code);
        let adapter = JSAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_js_magic_numbers_skips_trivial() {
        let code = "function main() { foo(0); bar(1); }\n";
        let file = parse_js(code);
        let adapter = JSAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }
}
