//! CAdapter — C language adapter.

use super::c_cpp_common;
use super::{count_dead_code_with, count_duplicate_imports_with, FunctionNode, LanguageAdapter};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;

const C_PATTERNS: &[&str] = &[
    "(function_definition declarator: (function_declarator declarator: (identifier) @ex_name)) @ex_fn",
    "(init_declarator declarator: (identifier) @nv_var)",
    "(call_expression function: (identifier) @dp_func (#match? @dp_func \"^(printf|fprintf|puts|putchar)$\"))",
    "(function_declarator parameters: (parameter_list) @ep_params)",
    "(number_literal) @mn_num",
    "(goto_statement) @ci_goto",
    "(call_expression function: (identifier) @pc_func (#match? @pc_func \"^(exit|abort|assert|_Exit|quick_exit|longjmp)$\"))",
];

pub struct CAdapter;

impl LanguageAdapter for CAdapter {
    fn language(&self) -> Language {
        Language::C
    }

    fn query_patterns(&self) -> &[&str] {
        C_PATTERNS
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        self.count_panic_from_batch(file, &self.batch_captures(file))
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
    }

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize {
        c_cpp_common::c_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        self.count_naming_from_batch(file, &self.batch_captures(file))
    }

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize {
        let mut count = 0;
        c_cpp_common::walk_c_blocks(file.root_node(), 0, 5, &mut count);
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

    fn count_dead_code(&self, file: &ParsedFile) -> usize {
        count_dead_code_with(
            file,
            &["return;", "break;", "continue;"],
            &[
                "return ",
                "exit(",
                "abort(",
                "_Exit(",
                "quick_exit(",
                "goto ",
            ],
            "//",
        )
    }

    fn count_duplicate_imports(&self, file: &ParsedFile) -> usize {
        count_duplicate_imports_with(file, &["#include"])
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
        c_cpp_common::extract_functions_from_batch(batch)
    }

    fn count_naming_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        c_cpp_common::count_naming_from_batch(batch)
    }

    fn count_debug_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        c_cpp_common::count_debug_from_batch(batch)
    }

    fn count_excessive_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        c_cpp_common::count_excessive_from_batch(batch)
    }

    fn count_magic_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        c_cpp_common::count_magic_from_batch(batch)
    }

    fn count_c_from_batch<'a>(&self, file: &ParsedFile, batch: &[Vec<QueryCapture<'a>>]) -> usize {
        c_cpp_common::count_c_issues_from_batch(file, batch, &[])
    }

    fn count_panic_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "pc_func"))
            .count()
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
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_c_count_panic_assert() {
        let code = "void main() { assert(x > 0); }\n";
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 1);
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

    #[test]
    fn test_c_dead_code_after_return() {
        let code = r#"
int foo() {
    return 42;
    printf("dead");
}
"#;
        let file = parse_c(code);
        let adapter = CAdapter;
        assert_eq!(adapter.count_dead_code(&file), 1);
    }
}
