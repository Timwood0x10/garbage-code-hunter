//! GoAdapter — Go language adapter.

use super::{count_nested_blocks, is_inside_declaration, FunctionNode, LanguageAdapter};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use regex::Regex;

pub struct GoAdapter;

impl LanguageAdapter for GoAdapter {
    fn language(&self) -> Language {
        Language::Go
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        let Ok(groups) = collect_captures(
            file,
            "(call_expression function: (identifier) @f (#eq? @f \"panic\"))",
        ) else {
            return 0;
        };
        groups.len()
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        let mut functions = Vec::new();
        let Ok(groups) = collect_captures(
            file,
            "[(function_declaration name: (identifier) @name) (method_declaration name: (field_identifier) @name)] @fn",
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
        fn go_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
            let mut max = depth;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    let child_depth = match child.kind() {
                        "block" => depth + 1,
                        _ => depth,
                    };
                    max = max.max(go_scope_depth(child, child_depth));
                }
            }
            max
        }
        go_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        let mut count = 0usize;
        let terrible_re = Regex::new(
            r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$",
        )
        .ok();

        if let Ok(groups) = collect_captures(
            file,
            "[(short_var_declaration left: (expression_list (identifier) @var))
              (var_spec name: (identifier) @var)]",
        ) {
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
        let mut count = 0;
        count_nested_blocks(file.root_node(), 0, 5, &mut count);
        count
    }

    fn count_debug_calls(&self, file: &ParsedFile) -> usize {
        let mut count = 0;
        if let Ok(groups) = collect_captures(
            file,
            r#"(call_expression
  function: (selector_expression
    operand: (identifier) @pkg
    field: (field_identifier) @method)
  (#match? @pkg "^(fmt|log)$")
  (#match? @method "^(Print|Println|Printf|Fprint|Fprintln|Fprintf|Sprint|Sprintln|Sprintf)$"))"#,
        ) {
            count += groups.len();
        }
        if let Ok(groups) = collect_captures(
            file,
            "(call_expression function: (identifier) @f (#eq? @f \"panic\"))",
        ) {
            count += groups.len();
        }
        count
    }

    fn count_excessive_params(&self, file: &ParsedFile, threshold: usize) -> usize {
        let mut count = 0;
        let Ok(groups) = collect_captures(
            file,
            "[(function_declaration parameters: (parameter_list) @params)
              (method_declaration parameters: (parameter_list) @params)]",
        ) else {
            return 0;
        };
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
        let Ok(captures) = collect_captures(file, "[(int_literal) @num (float_literal) @num]")
        else {
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

    fn parse_go(code: &str) -> ParsedFile {
        parse_code(code, "test.go").expect("parse")
    }

    #[test]
    fn test_go_count_panic_calls() {
        let code = r#"
package main
func main() {
    panic("boom")
    panic("bang")
}
"#;
        let file = parse_go(code);
        let adapter = GoAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_go_count_panic_calls_clean() {
        let code = "package main\nfunc main() { println(\"ok\") }\n";
        let file = parse_go(code);
        let adapter = GoAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_go_extract_functions() {
        let code = r#"
package main
func foo() {}
func bar(x int) int { return x }
"#;
        let file = parse_go(code);
        let adapter = GoAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
    }

    #[test]
    fn test_go_naming_single_letter() {
        let code = r#"
package main
func main() {
    x := 1
    y := 2
}
"#;
        let file = parse_go(code);
        let adapter = GoAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_go_debug_fmt_println() {
        let code = r#"
package main
import "fmt"
func main() {
    fmt.Println("hello")
    fmt.Printf("x=%d", 1)
}
"#;
        let file = parse_go(code);
        let adapter = GoAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 2);
    }

    #[test]
    fn test_go_excessive_params() {
        let code = "package main\nfunc process(a, b, c, d, e, f int) {}\n";
        let file = parse_go(code);
        let adapter = GoAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1);
    }

    #[test]
    fn test_go_magic_numbers() {
        let code = r#"
package main
func main() {
    x := 41 + 1
    y := x * 100
}
"#;
        let file = parse_go(code);
        let adapter = GoAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_go_magic_numbers_skips_trivial() {
        let code = r#"
package main
func main() {
    x := 0 + 1
}
"#;
        let file = parse_go(code);
        let adapter = GoAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }
}
