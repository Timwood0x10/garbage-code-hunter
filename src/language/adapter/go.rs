//! GoAdapter — Go language adapter.

use super::{
    count_duplicate_imports_with, count_nested_blocks, count_params, is_boolean_or_null,
    is_common_safe_number, is_inside_declaration, is_repeating_chars, FunctionNode,
    LanguageAdapter, MEANINGLESS_NAMES,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

const GO_PATTERNS: &[&str] = &[
    // pc_ — panic calls
    "(call_expression function: (identifier) @pc_fn (#eq? @pc_fn \"panic\"))",
    // ex_ — extract functions
    "[(function_declaration name: (identifier) @ex_name) (method_declaration name: (field_identifier) @ex_name)] @ex_fn",
    // nv_ — naming violations
    "[(short_var_declaration left: (expression_list (identifier) @nv_var)) (var_spec name: (identifier) @nv_var)]",
    "(method_declaration receiver: (parameter_list (parameter_declaration name: (identifier) @nv_rec)))",
    // dp_ — debug calls
    r#"(call_expression
  function: (selector_expression
    operand: (identifier) @dp_pkg
    field: (field_identifier) @dp_method)
  (#match? @dp_pkg "^(fmt|log)$")
  (#match? @dp_method "^(Print|Println|Printf|Fprint|Fprintln|Fprintf|Sprint|Sprintln|Sprintf)$"))"#,
    // ep_ — excessive params
    "[(function_declaration parameters: (parameter_list) @ep_params) (method_declaration parameters: (parameter_list) @ep_params)]",
    // mn_ — magic numbers
    "[(int_literal) @mn_num (float_literal) @mn_num]",
    // gs_ — goroutine spawns
    "(go_statement) @gs_go",
    // cv_ — convention violations (fmt.Errorf / fmt.New)
    r#"(call_expression function: (selector_expression operand: (identifier) @cv_pkg field: (field_identifier) @cv_method) (#eq? @cv_pkg "fmt") (#match? @cv_method "^(Errorf|New)$"))"#,
];

pub struct GoAdapter;

impl LanguageAdapter for GoAdapter {
    fn language(&self) -> Language {
        Language::Go
    }

    fn query_patterns(&self) -> &[&str] {
        GO_PATTERNS
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        self.count_panic_from_batch(file, &self.batch_captures(file))
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
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

    fn count_goroutine_spawns(&self, file: &ParsedFile) -> usize {
        self.count_goroutine_from_batch(file, &self.batch_captures(file))
    }

    fn count_defer_in_loop(&self, file: &ParsedFile) -> usize {
        fn has_defer_child(node: tree_sitter::Node) -> bool {
            let mut cursor = node.walk();
            let mut found = cursor.goto_first_child();
            while found {
                if cursor.node().kind() == "defer_statement" {
                    return true;
                }
                found = cursor.goto_next_sibling();
            }
            false
        }

        fn walk_for_loops(_file: &ParsedFile, node: tree_sitter::Node, count: &mut usize) {
            if node.kind() == "for_statement" && has_defer_child(node) {
                *count += 1;
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                walk_for_loops(_file, child, count);
            }
        }

        let mut count = 0;
        walk_for_loops(file, file.root_node(), &mut count);
        count
    }

    fn count_go_convention_violations(&self, file: &ParsedFile) -> usize {
        self.count_go_convention_from_batch(file, &self.batch_captures(file))
    }

    // -- _from_batch overrides --

    fn count_panic_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "pc_fn"))
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
        file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let mut count = 0usize;
        static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
            Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();
        let idiomatic_single: &[&str] = &["e", "g", "i", "j", "k", "n", "c"];

        for m in batch {
            for c in m {
                match c.name.as_str() {
                    "nv_var" => {
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
                    "nv_rec" if c.text.len() > 2 => {
                        count += 1;
                    }
                    _ => {}
                }
            }
        }

        // go-mixed-caps: snake_case or ALL_CAPS variable names (text scanning)
        let go_idioms = [
            "err", "ok", "ctx", "mu", "wg", "ch", "db", "id", "ip", "tx", "rx", "fd", "fs", "ns",
            "fn", "hp", "os", "rc",
        ];
        for line in file.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            let name = if let Some(rest) = trimmed.strip_prefix("var ") {
                rest.split_whitespace().next().unwrap_or("")
            } else if let Some(idx) = trimmed.find(":=") {
                trimmed[..idx].split_whitespace().last().unwrap_or("")
            } else {
                ""
            };
            if name.is_empty() || name.len() < 2 || go_idioms.contains(&name) || name == "_" {
                continue;
            }
            if name.chars().next().is_some_and(|ch| ch.is_uppercase()) {
                continue;
            }
            let has_underscore = name.contains('_') && name != "_";
            let is_all_caps = name
                .chars()
                .all(|ch| ch.is_uppercase() || ch == '_' || ch.is_numeric())
                && name.chars().any(|ch| ch.is_uppercase());
            if has_underscore || is_all_caps {
                count += 1;
            }
        }

        count
    }

    fn count_debug_from_batch<'a>(
        &self,
        file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let source = file.content.as_bytes();
        let mut count = 0;
        for m in batch {
            let has_dp = m.iter().any(|c| c.name.starts_with("dp_"));
            if !has_dp {
                continue;
            }
            let mut exempt = false;
            for c in m {
                if c.name == "dp_pkg" && c.text == "fmt" {
                    let mut current = Some(c.node);
                    while let Some(n) = current {
                        if n.kind() == "function_declaration" {
                            if let Some(name_node) = n.child_by_field_name("name") {
                                if let Ok(text) = name_node.utf8_text(source) {
                                    if text == "main" {
                                        exempt = true;
                                        break;
                                    }
                                }
                            }
                            break;
                        }
                        current = n.parent();
                    }
                }
                if exempt {
                    break;
                }
            }
            if !exempt {
                count += 1;
            }
        }
        count
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

    fn count_goroutine_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "gs_go"))
            .count()
    }

    fn count_go_convention_from_batch<'a>(
        &self,
        file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let mut count = 0;

        // go-error-string: fmt.Errorf / fmt.New with uppercase first letter
        for m in batch {
            if !m.iter().any(|c| c.name == "cv_method") {
                continue;
            }
            for c in m {
                if c.name == "cv_method" {
                    if let Some(call_node) = c.node.parent().and_then(|p| p.parent()) {
                        for child in call_node.children(&mut call_node.walk()) {
                            if child.kind() == "argument_list" {
                                let text = file.node_text(child);
                                let trimmed = text.trim();
                                let start = trimmed.find('"');
                                let content = start
                                    .map(|s| {
                                        let from = &trimmed[s + 1..];
                                        from.find('"').map(|e| &from[..e]).unwrap_or("")
                                    })
                                    .unwrap_or("");
                                if let Some(first) = content.chars().next() {
                                    if first.is_uppercase() {
                                        count += 1;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        // go-context-first: context.Context not the first parameter
        for line in file.content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("func ") {
                continue;
            }
            let params_start = trimmed.find('(');
            let params_end = trimmed.rfind(')');
            if let (Some(ps), Some(pe)) = (params_start, params_end) {
                let params_str = &trimmed[ps + 1..pe];
                if params_str.contains("context.Context") {
                    let first = params_str.split(',').next().unwrap_or("").trim();
                    if !first.contains("context.Context") {
                        count += 1;
                    }
                }
            }
        }

        // go-else-return: if-else with return in if-block
        fn has_return_statement(n: tree_sitter::Node) -> bool {
            if n.kind() == "return_statement" {
                return true;
            }
            let mut cursor = n.walk();
            let mut inner = cursor.goto_first_child();
            while inner {
                if cursor.node().kind() == "return_statement" {
                    return true;
                }
                inner = cursor.goto_next_sibling();
            }
            false
        }

        fn check_else_return(_file: &ParsedFile, node: tree_sitter::Node, cnt: &mut usize) {
            if node.kind() == "if_statement" {
                let mut cx = node.walk();
                let has_else = node.children(&mut cx).any(|c| c.kind() == "else");
                if has_else {
                    let mut cx2 = node.walk();
                    for child in node.children(&mut cx2) {
                        if child.kind() == "block" || child.kind() == "compound_statement" {
                            let mut cx3 = child.walk();
                            let has_ret = child.children(&mut cx3).any(has_return_statement);
                            if has_ret {
                                *cnt += 1;
                                break;
                            }
                        }
                    }
                }
            }
            let mut cx4 = node.walk();
            for child in node.children(&mut cx4) {
                check_else_return(_file, child, cnt);
            }
        }
        check_else_return(file, file.root_node(), &mut count);

        count
    }

    fn count_dead_code(&self, file: &ParsedFile) -> usize {
        let mut count = 0;
        let mut dead_start: Option<usize> = None;
        for (line_num, line) in file.content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed == "return"
                || trimmed == "return;"
                || trimmed == "break"
                || trimmed == "break;"
                || trimmed == "continue"
                || trimmed == "continue;"
                || (trimmed.starts_with("return ")
                    && (trimmed.ends_with(';') || !trimmed.ends_with('}')))
                || trimmed.starts_with("panic(")
                || trimmed.starts_with("goto ")
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
        count_duplicate_imports_with(file, &["import ", "import ("])
    }
}

impl GoAdapter {
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
func helper() {
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
