//! TSAdapter — TypeScript language adapter.

use super::{
    count_nested_blocks, count_params, is_inside_declaration, is_repeating_chars, FunctionNode,
    LanguageAdapter, MEANINGLESS_NAMES,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use regex::Regex;
use std::sync::LazyLock;

pub struct TSAdapter;

impl LanguageAdapter for TSAdapter {
    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        let Ok(groups) = collect_captures(file, "(throw_statement) @throw") else {
            return 0;
        };
        groups.len()
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        let mut functions = Vec::new();
        let Ok(groups) = collect_captures(
            file,
            "[(function_declaration name: (identifier) @name)
              (method_definition name: (property_identifier) @name)] @fn",
        ) else {
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
        fn ts_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
            let mut max = depth;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    let child_depth = match child.kind() {
                        "block" => depth + 1,
                        _ => depth,
                    };
                    max = max.max(ts_scope_depth(child, child_depth));
                }
            }
            max
        }
        ts_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        let mut count = 0usize;
        static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
            Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();

        if let Ok(groups) = collect_captures(file, "(variable_declarator name: (identifier) @var)")
        {
            for group in &groups {
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    if name.len() == 1 && name.chars().all(|c| c.is_ascii_lowercase()) {
                        count += 1;
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
        let mut count = 0;
        count_nested_blocks(file.root_node(), 0, 5, &mut count);
        count
    }

    fn count_debug_calls(&self, file: &ParsedFile) -> usize {
        let mut count = 0;
        if let Ok(groups) = collect_captures(
            file,
            r#"(call_expression
  function: (member_expression
    property: (property_identifier) @method)
  (#match? @method "^(log|debug|warn|error|info|trace)$"))"#,
        ) {
            count += groups.len();
        }
        if let Ok(groups) = collect_captures(file, "(debugger_statement) @debug") {
            count += groups.len();
        }
        count
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        let mut count = 0;
        let Ok(groups) = collect_captures(
            file,
            "[(function_declaration parameters: (formal_parameters) @params)
              (arrow_function parameters: (formal_parameters) @params)
              (method_definition parameters: (formal_parameters) @params)]",
        ) else {
            return 0;
        };
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
        count
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        let Ok(captures) = collect_captures(file, "(number) @num") else {
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

    fn count_ts_issues(&self, file: &ParsedFile) -> usize {
        let mut count = 0;

        // any-type: predefined_type "any"
        if let Ok(groups) = collect_captures(file, "(predefined_type) @t") {
            for group in &groups {
                if let Some(cap) = group.first() {
                    if cap.text.trim() == "any" {
                        count += 1;
                    }
                }
            }
        }

        // prefer-interface: type_alias_declaration with object type (not unions/primitives/functions)
        if let Ok(groups) =
            collect_captures(file, "(type_alias_declaration value: (object_type) @alias)")
        {
            count += groups.len();
        }

        // ts-no-enum: enum_declaration
        if let Ok(groups) = collect_captures(file, "(enum_declaration) @enum") {
            count += groups.len();
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse_code;
    use super::*;

    fn parse_ts(code: &str) -> ParsedFile {
        parse_code(code, "test.ts").expect("parse")
    }

    #[test]
    fn test_ts_count_panic_throw() {
        let code = r#"
function main(): void {
    throw new Error("boom");
    throw "bang";
}
"#;
        let file = parse_ts(code);
        let adapter = TSAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_ts_count_panic_clean() {
        let code = "function main(): void { return 42; }";
        let file = parse_ts(code);
        let adapter = TSAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_ts_extract_functions() {
        let code = r#"
function foo(): void {}
function bar(x: number): number { return x; }
const obj = { baz(): void {} };
"#;
        let file = parse_ts(code);
        let adapter = TSAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 3);
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
        assert_eq!(fns[2].name, "baz");
    }

    #[test]
    fn test_ts_naming_single_letter() {
        let code = r#"
function main(): void {
    let x = 1;
    const y = 2;
}
"#;
        let file = parse_ts(code);
        let adapter = TSAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_ts_debug_console_log() {
        let code = r#"
console.log("hello");
console.error("bad");
debugger;
"#;
        let file = parse_ts(code);
        let adapter = TSAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 3);
    }

    #[test]
    fn test_ts_excessive_params() {
        let code = "function process(a: number, b: number, c: number, d: number, e: number, f: number): void {}\n";
        let file = parse_ts(code);
        let adapter = TSAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1);
    }

    #[test]
    fn test_ts_magic_numbers() {
        let code = r#"
function main(): void {
    foo(41);
    bar(100);
}
"#;
        let file = parse_ts(code);
        let adapter = TSAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_ts_magic_numbers_skips_trivial() {
        let code = "function main(): void { foo(0); bar(1); }\n";
        let file = parse_ts(code);
        let adapter = TSAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }
}
