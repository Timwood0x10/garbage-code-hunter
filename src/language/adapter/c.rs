//! CAdapter — C language adapter.

use super::{
    count_block_ancestors, count_params, is_boolean_or_null, is_common_safe_number,
    is_inside_declaration, is_repeating_chars, FunctionNode, LanguageAdapter, MEANINGLESS_NAMES,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

const C_PATTERNS: &[&str] = &[
    "(function_definition declarator: (function_declarator declarator: (identifier) @ex_name)) @ex_fn",
    "(init_declarator declarator: (identifier) @nv_var)",
    "(call_expression function: (identifier) @dp_func (#match? @dp_func \"^(printf|fprintf|puts|putchar)$\"))",
    "(function_declarator parameters: (parameter_list) @ep_params)",
    "(number_literal) @mn_num",
    "(goto_statement) @ci_goto",
];

pub struct CAdapter;

impl LanguageAdapter for CAdapter {
    fn language(&self) -> Language {
        Language::C
    }

    fn query_patterns(&self) -> &[&str] {
        C_PATTERNS
    }

    fn count_panic_calls(&self, _file: &ParsedFile) -> usize {
        // exit()/abort() are normal C idioms, not anti-patterns
        0
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
    }

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize {
        fn c_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
            let mut max = depth;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    let child_depth = match child.kind() {
                        "compound_statement" => depth + 1,
                        _ => depth,
                    };
                    max = max.max(c_scope_depth(child, child_depth));
                }
            }
            max
        }
        c_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        self.count_naming_from_batch(file, &self.batch_captures(file))
    }

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize {
        fn walk_c_blocks(
            node: tree_sitter::Node,
            depth: usize,
            threshold: usize,
            count: &mut usize,
        ) {
            if node.kind() == "compound_statement" && depth >= threshold {
                *count += 1;
            }
            let child_depth = match node.kind() {
                "compound_statement" => depth + 1,
                _ => depth,
            };
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    walk_c_blocks(child, child_depth, threshold, count);
                }
            }
        }
        let mut count = 0;
        walk_c_blocks(file.root_node(), 0, 5, &mut count);
        count
    }

    fn count_debug_calls(&self, file: &ParsedFile) -> usize {
        self.count_debug_from_batch(file, &self.batch_captures(file))
    }

    fn count_excessive_params(&self, file: &ParsedFile, _threshold: usize) -> usize {
        self.count_excessive_from_batch(file, &self.batch_captures(file))
    }

    fn count_magic_numbers(&self, file: &ParsedFile) -> usize {
        self.count_magic_from_batch(file, &self.batch_captures(file))
    }

    fn count_c_issues(&self, file: &ParsedFile) -> usize {
        self.count_c_from_batch(file, &self.batch_captures(file))
    }

    // -- _from_batch overrides --

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
        static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
            Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();
        let idiomatic_single: &[&str] = &["i", "j", "k", "n", "c", "e"];

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
            .filter(|m| m.iter().any(|c| c.name == "dp_func"))
            .count()
    }

    fn count_excessive_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let mut count = 0;
        for m in batch {
            for c in m {
                if c.name == "ep_params" && count_params(c.text) > 5 {
                    count += 1;
                }
            }
        }
        count
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

    fn count_c_from_batch<'a>(&self, file: &ParsedFile, batch: &[Vec<QueryCapture<'a>>]) -> usize {
        let mut count = 0;

        // c-goto-abuse from batch
        for m in batch {
            for c in m {
                if c.name == "ci_goto" {
                    count += 1;
                }
            }
        }

        // c-sizeof-type: sizeof(type) should be sizeof(expr) — text scanning
        for line in file.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            if trimmed.contains("sizeof(") {
                let start = trimmed.find("sizeof(").unwrap() + 7;
                let rest = &trimmed[start..];
                let mut depth = 1u32;
                let mut inner = String::new();
                for ch in rest.chars() {
                    if ch == '(' {
                        depth += 1;
                    } else if ch == ')' {
                        depth -= 1;
                    }
                    if depth == 0 {
                        break;
                    }
                    inner.push(ch);
                }
                let inner = inner.trim().trim_end_matches(')');
                if inner.starts_with(|c: char| c.is_alphabetic() || c == '_')
                    && !inner.contains(|c: char| {
                        c == '+' || c == '-' || c == '*' || c == '/' || c == '('
                    })
                {
                    let type_keywords = [
                        "int", "char", "float", "double", "long", "short", "unsigned", "signed",
                        "void", "size_t", "bool", "struct", "union", "enum",
                    ];
                    if type_keywords.iter().any(|t| inner.starts_with(t)) {
                        count += 1;
                    }
                    if inner.ends_with("_t") && inner.len() > 2 {
                        count += 1;
                    }
                }
            }
        }

        // c-malloc-check: malloc without NULL check — text scanning
        let lines: Vec<&str> = file.content.lines().collect();
        for i in 0..lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }
            if trimmed.contains("malloc(") && trimmed.ends_with(';') {
                let check_range = (i + 1).min(lines.len())..(i + 4).min(lines.len());
                let has_null_check = lines[check_range].iter().any(|l| {
                    let l = l.trim();
                    l.contains("== NULL")
                        || l.contains("!= NULL")
                        || l.contains("== 0")
                        || l.contains("!= 0")
                        || l.contains("if (!")
                        || l.contains("if (NULL")
                });
                if !has_null_check {
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

    fn parse_c(code: &str) -> ParsedFile {
        parse_code(code, "test.c").expect("parse")
    }

    #[test]
    fn test_c_count_panic_exit() {
        let code = r#"
void main() {
    exit(1);
    abort();
}
"#;
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_c_count_panic_clean() {
        let code = "int add(int x) { return x + 1; }\n";
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_c_extract_functions() {
        let code = r#"
int foo() { return 1; }
void bar(int x) {}
"#;
        let file = parse_c(code);
        let adapter = CAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
    }

    #[test]
    fn test_c_naming_single_letter() {
        let code = r#"
void main() {
    int x = 1;
    int y = 2;
}
"#;
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_c_debug_printf() {
        let code = r#"
void main() {
    printf("hello");
    fprintf(stderr, "bad");
}
"#;
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 2);
    }

    #[test]
    fn test_c_excessive_params() {
        let code = "void process(int a, int b, int c, int d, int e, int f) {}\n";
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1);
    }

    #[test]
    fn test_c_magic_numbers() {
        let code = r#"
void main() {
    foo(41);
    bar(100);
}
"#;
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_c_magic_numbers_skips_trivial() {
        let code = "void main() { foo(0); bar(1); }\n";
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }
}
