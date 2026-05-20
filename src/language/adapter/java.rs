//! JavaAdapter — Java language adapter.

use super::{
    count_dead_code_with, count_duplicate_imports_with, count_nested_blocks, count_params,
    is_boolean_or_null, is_common_safe_number, is_inside_declaration, is_repeating_chars,
    FunctionNode, LanguageAdapter, MEANINGLESS_NAMES,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

const JAVA_PATTERNS: &[&str] = &[
    "(throw_statement) @pc_throw",
    "(method_declaration name: (identifier) @ex_name) @ex_fn",
    "(variable_declarator name: (identifier) @nv_var)",
    "(method_invocation name: (identifier) @dp_method (#match? @dp_method \"^(println|printStackTrace)$\"))",
    "(method_declaration parameters: (formal_parameters) @ep_params)",
    "[(decimal_integer_literal) @mn_num (decimal_floating_point_literal) @mn_num]",
];

fn find_empty_catch(node: tree_sitter::Node, count: &mut usize) {
    if node.kind() == "catch_clause" {
        if let Some(body) = node.child_by_field_name("body") {
            if body.named_child_count() == 0 {
                *count += 1;
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            find_empty_catch(child, count);
        }
    }
}

pub struct JavaAdapter;

impl LanguageAdapter for JavaAdapter {
    fn language(&self) -> Language {
        Language::Java
    }

    fn query_patterns(&self) -> &[&str] {
        JAVA_PATTERNS
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        self.count_panic_from_batch(file, &self.batch_captures(file))
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
    }

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize {
        fn java_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
            let mut max = depth;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    let child_depth = match child.kind() {
                        "block" => depth + 1,
                        _ => depth,
                    };
                    max = max.max(java_scope_depth(child, child_depth));
                }
            }
            max
        }
        java_scope_depth(file.root_node(), 0)
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

    fn count_dead_code(&self, file: &ParsedFile) -> usize {
        count_dead_code_with(
            file,
            &["return;", "break;", "continue;"],
            &["return ", "throw ", "System.exit("],
            "//",
        )
    }

    fn count_duplicate_imports(&self, file: &ParsedFile) -> usize {
        count_duplicate_imports_with(file, &["import "])
    }

    fn count_java_issues(&self, file: &ParsedFile) -> usize {
        let mut count = 0;

        // empty-catch: catch blocks with no named children in body
        find_empty_catch(file.root_node(), &mut count);

        let lines: Vec<&str> = file.content.lines().collect();

        // java-javadoc-missing: public/protected methods without Javadoc
        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim();
            if (trimmed.starts_with("public ") || trimmed.starts_with("protected "))
                && trimmed.contains("(")
                && (trimmed.contains(")")
                    || lines.get(i + 1).is_some_and(|l| l.trim().contains(")")))
            {
                let mut j = i as i32 - 1;
                while j >= 0 && lines[j as usize].trim().is_empty() {
                    j -= 1;
                }
                let has_javadoc = j >= 0
                    && (lines[j as usize].trim().starts_with("/**")
                        || lines[j as usize].trim().ends_with("*/"));
                let has_annotation = j >= 0
                    && (lines[j as usize].trim().starts_with("@Override")
                        || lines[j as usize].trim().starts_with("@Suppress")
                        || lines[j as usize].trim().starts_with("@"));
                if !has_javadoc && !has_annotation {
                    count += 1;
                }
            }
            i += 1;
        }

        // java-try-resource: finally with .close() within 3 lines
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("finally") {
                for k in 1..=3 {
                    if lines
                        .get(line_num + k)
                        .is_some_and(|n| n.trim().contains(".close()"))
                    {
                        count += 1;
                        break;
                    }
                }
            }
        }

        // java-string-concat: += within 10 lines of for/while
        let has_loop = file.content.contains("for ") || file.content.contains("while ");
        if has_loop {
            for (line_num, line) in file.content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.contains(" += ") {
                    let start = line_num.saturating_sub(10);
                    for k in (start..line_num).rev() {
                        let prev = lines[k].trim();
                        if prev.starts_with("for ") || prev.starts_with("while ") {
                            count += 1;
                            break;
                        }
                    }
                }
            }
        }

        // java-wildcard-import: import ...*;
        for line in &lines {
            if line.trim().starts_with("import ") && line.trim().ends_with(".*;") {
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
        file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let mut count = 0usize;
        static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
            Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();
        let idiomatic_single: &[&str] = &["i", "j", "k", "e", "n"];

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

        // constant-name: static final fields should be UPPER_SNAKE_CASE
        for line in file.content.lines() {
            let trimmed = line.trim();
            if !trimmed.contains("static final") && !trimmed.contains("final static") {
                continue;
            }
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let name = parts
                .iter()
                .position(|p| *p == "=" || p.ends_with('=') || p.ends_with(';'))
                .and_then(|idx| {
                    if idx > 0 {
                        parts
                            .get(idx - 1)
                            .map(|s| s.trim_end_matches('=').trim_end_matches(';'))
                    } else {
                        None
                    }
                })
                .unwrap_or("");
            if !name.is_empty()
                && name != name.to_uppercase()
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
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
        let base = batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "dp_method"))
            .count();
        let log_calls = file
            .content
            .lines()
            .filter(|l| {
                let t = l.trim();
                if t.starts_with("//") || t.starts_with("/*") || t.starts_with("*") {
                    return false;
                }
                t.contains(".info(")
                    || t.contains(".debug(")
                    || t.contains(".warn(")
                    || t.contains(".error(")
                    || t.contains(".fine(")
                    || t.contains(".finest(")
                    || t.contains(".severe(")
            })
            .count();
        base + log_calls
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

impl JavaAdapter {
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

    fn parse_java(code: &str) -> ParsedFile {
        parse_code(code, "Test.java").expect("parse")
    }

    #[test]
    fn test_java_count_panic_throw() {
        let code = r#"
class Test {
    void main() {
        throw new RuntimeException("boom");
        throw new Exception("bang");
    }
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 2);
    }

    #[test]
    fn test_java_count_panic_clean() {
        let code = r#"
class Test {
    void main() {
        return;
    }
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_java_extract_functions() {
        let code = r#"
class Test {
    void foo() {}
    void bar(int x) {}
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
    }

    #[test]
    fn test_java_naming_single_letter() {
        let code = r#"
class Test {
    void main() {
        int x = 1;
        int y = 2;
    }
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_java_debug_sout() {
        let code = r#"
class Test {
    void main() {
        System.out.println("hello");
        System.err.println("bad");
    }
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 2);
    }

    #[test]
    fn test_java_debug_print_stack_trace() {
        let code = r#"
class Test {
    void main() {
        e.printStackTrace();
    }
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 1);
    }

    #[test]
    fn test_java_excessive_params() {
        let code = r#"
class Test {
    void process(int a, int b, int c, int d, int e, int f) {}
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1);
    }

    #[test]
    fn test_java_magic_numbers() {
        let code = r#"
class Test {
    void main() {
        foo(42);
        bar(100);
    }
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_java_magic_numbers_skips_trivial() {
        let code = r#"
class Test {
    void main() {
        foo(0);
        bar(1);
    }
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }

    #[test]
    fn test_java_dead_code_after_return() {
        let code = r#"
void foo() {
    return;
    System.out.println("dead");
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_dead_code(&file), 1);
    }

    #[test]
    fn test_java_debug_logging() {
        let code = r#"
class Test {
    void main() {
        logger.info("started");
        log.debug("step 1");
        log.error("failed");
    }
}
"#;
        let file = parse_java(code);
        let adapter = JavaAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 3);
    }
}
