//! PythonAdapter — Python language adapter.

use super::{
    count_block_ancestors, count_nested_blocks, get_node_text, is_inside_declaration,
    is_repeating_chars, max_scope_depth, FunctionNode, LanguageAdapter,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use regex::Regex;

pub struct PythonAdapter;

impl LanguageAdapter for PythonAdapter {
    fn language(&self) -> Language {
        Language::Python
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        let Ok(groups) = collect_captures(file, "(except_clause) @e") else {
            return 0;
        };
        let source = file.content.as_bytes();
        let mut count = 0;
        for group in &groups {
            if let Some(cap) = group.first() {
                let node = cap.node;
                if let Some(value) = node.child_by_field_name("value") {
                    let text = get_node_text(value, source);
                    if text == "BaseException" || text == "Exception" {
                        count += 1;
                    }
                } else {
                    count += 1;
                }
            }
        }
        count
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        let Ok(groups) =
            collect_captures(file, "(function_definition name: (identifier) @name) @fn")
        else {
            return vec![];
        };
        let mut functions = Vec::new();
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

        if let Ok(groups) = collect_captures(
            file,
            "(assignment left: (identifier) @var (#match? @var \"^[a-z]$\"))",
        ) {
            count += groups.len();
        }

        let terrible_re = Regex::new(
            r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$",
        )
        .ok();
        let meaningless: &[&str] = &[
            "foo", "bar", "baz", "qux", "quux", "quuz", "aaa", "bbb", "ccc", "ddd", "eee", "xxx",
            "yyy", "zzz", "test1", "test2", "test3",
        ];

        if let Ok(groups) = collect_captures(file, "(assignment left: (identifier) @name)") {
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
                    }
                }
            }
        }

        if let Ok(groups) = collect_captures(file, "(function_definition name: (identifier) @name)")
        {
            for group in &groups {
                if count > 2000 {
                    break;
                }
                if let Some(cap) = group.first() {
                    let name = cap.text;
                    if name.starts_with("__") || name.starts_with('_') {
                        continue;
                    }
                    if name.chars().any(|c| c.is_uppercase()) {
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
            "(call function: (identifier) @fn (#eq? @fn \"print\"))",
        ) {
            count += groups.len();
        }
        count
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        let mut count = 0;
        if let Ok(groups) = collect_captures(
            file,
            "(function_definition parameters: (parameters) @params)",
        ) {
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
        }
        count
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        let Ok(captures) = collect_captures(file, "(integer) @num") else {
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

    fn parse_python(code: &str) -> ParsedFile {
        parse_code(code, "test.py").expect("parse")
    }

    #[test]
    fn test_python_count_panic_calls_bare_except() {
        let code = r#"
try:
    do_something()
except:
    pass
"#;
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 1, "bare except = 1");
    }

    #[test]
    fn test_python_count_panic_calls_base_exception() {
        let code = r#"
try:
    do_something()
except BaseException:
    pass
"#;
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(
            adapter.count_panic_calls(&file),
            1,
            "except BaseException = 1"
        );
    }

    #[test]
    fn test_python_count_panic_calls_specific_ok() {
        let code = r#"
try:
    do_something()
except ValueError:
    pass
"#;
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0, "specific except = 0");
    }

    #[test]
    fn test_python_naming_single_letter() {
        let code = "x = 1\ny = 2\n";
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2, "x and y");
    }

    #[test]
    fn test_python_naming_camel_case_fn() {
        let code = "def getData(): pass\n";
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 1, "camelCase fn");
    }

    #[test]
    fn test_python_debug_print() {
        let code = r#"
print("hello")
print(x)
"#;
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 2, "two print calls");
    }

    #[test]
    fn test_python_debug_clean() {
        let code = "result = add(1, 2)\n";
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 0, "no debug calls");
    }

    #[test]
    fn test_python_extract_functions() {
        let code = "def foo(): pass\ndef bar(x): return x\n";
        let file = parse_python(code);
        let adapter = PythonAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 2, "2 functions");
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
    }

    #[test]
    fn test_python_excessive_params() {
        let code = "def process(a, b, c, d, e, f): pass\n";
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1, "6 > 5");
    }

    #[test]
    fn test_python_magic_numbers() {
        let code = "foo(42)\nbar(100)\n";
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_python_magic_numbers_skips_trivial() {
        let code = "x = 1 + 0\n";
        let file = parse_python(code);
        let adapter = PythonAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0, "0 and 1 skipped");
    }
}
