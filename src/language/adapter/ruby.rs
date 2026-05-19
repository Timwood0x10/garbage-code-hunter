//! RubyAdapter — Ruby language adapter.

use super::{
    count_params, is_boolean_or_null, is_common_safe_number, is_inside_declaration,
    is_repeating_chars, FunctionNode, LanguageAdapter,
};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

const RUBY_PATTERNS: &[&str] = &[
    "(call method: (identifier) @pc_raise (#eq? @pc_raise \"raise\"))",
    "(method name: (identifier) @ex_name) @ex_fn",
    "(assignment left: (identifier) @nv_var)",
    "(method parameters: (_) @ep_params)",
    "[(integer) @mn_num (float) @mn_num]",
    "(global_variable) @ri_gv",
    "(call method: (identifier) @dp_method (#match? @dp_method \"^(puts|p|print|byebug|pry)$\"))",
];

pub struct RubyAdapter;

impl LanguageAdapter for RubyAdapter {
    fn language(&self) -> Language {
        Language::Ruby
    }

    fn query_patterns(&self) -> &[&str] {
        RUBY_PATTERNS
    }

    fn count_panic_calls(&self, file: &ParsedFile) -> usize {
        self.count_panic_from_batch(file, &self.batch_captures(file))
    }

    fn extract_functions(&self, file: &ParsedFile) -> Vec<FunctionNode> {
        self.extract_functions_from_batch(file, &self.batch_captures(file))
    }

    fn max_nesting_depth(&self, file: &ParsedFile) -> usize {
        fn ruby_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
            let mut max = depth;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    let child_depth = if child.kind() == "body_statement" {
                        depth + 1
                    } else {
                        depth
                    };
                    max = max.max(ruby_scope_depth(child, child_depth));
                }
            }
            max
        }
        ruby_scope_depth(file.root_node(), 0)
    }

    fn count_naming_violations(&self, file: &ParsedFile) -> usize {
        self.count_naming_from_batch(file, &self.batch_captures(file))
    }

    fn count_deeply_nested_blocks(&self, file: &ParsedFile) -> usize {
        fn walk_body(node: tree_sitter::Node, depth: usize, threshold: usize, count: &mut usize) {
            if node.kind() == "body_statement" && depth >= threshold {
                *count += 1;
            }
            let child_depth = if node.kind() == "body_statement" {
                depth + 1
            } else {
                depth
            };
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    walk_body(child, child_depth, threshold, count);
                }
            }
        }
        let threshold = 5;
        let mut count = 0;
        walk_body(file.root_node(), 0, threshold, &mut count);
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

    fn count_ruby_issues(&self, file: &ParsedFile) -> usize {
        self.count_ruby_from_batch(file, &self.batch_captures(file))
    }

    fn count_panic_from_batch<'a>(
        &self,
        _file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        batch
            .iter()
            .filter(|m| m.iter().any(|c| c.name == "pc_raise"))
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
        let idiomatic_single: &[&str] = &["e", "i", "j", "k", "x", "n"];

        static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
            Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").ok()
        });
        let terrible_re = TERRIBLE_RE.as_ref();
        let meaningless: &[&str] = &[
            "foo", "bar", "baz", "qux", "quux", "quuz", "aaa", "bbb", "ccc", "ddd", "eee", "xxx",
            "yyy", "zzz", "test1", "test2", "test3",
        ];

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
                    let name_lower = name.to_lowercase();
                    if let Some(re) = terrible_re {
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

        // ruby-predicate-method: boolean methods should end with ?
        for line in file.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if let Some(name) = trimmed
                .strip_prefix("def ")
                .and_then(|s| s.split(&['(', ' ', '\t'][..]).next())
            {
                let is_predicate = name.starts_with("is_")
                    || name.starts_with("has_")
                    || name.starts_with("can_")
                    || name.starts_with("should_");
                if is_predicate && !name.ends_with('?') {
                    count += 1;
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
            .filter(|m| m.iter().any(|c| c.name == "dp_method"))
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

    fn count_ruby_from_batch<'a>(
        &self,
        file: &ParsedFile,
        batch: &[Vec<QueryCapture<'a>>],
    ) -> usize {
        let mut count = 0;

        // global-variable: non-built-in global variables ($xxx)
        let acceptable: &[&str] = &[
            "$stdout",
            "$stderr",
            "$stdin",
            "$VERBOSE",
            "$DEBUG",
            "$SAFE",
            "$LOAD_PATH",
            "$LOADED_FEATURES",
            "$PROGRAM_NAME",
            "$FILENAME",
            "$.",
            "$,",
            "$;",
            "$/",
            "$\\",
            "$&",
            "$`",
            "$'",
            "$+",
            "$~",
            "$=",
            "$<",
            "$>",
            "$!",
            "$?",
            "$0",
            "$*",
            "$_",
            "$-d",
            "$-v",
            "$-w",
            "$-W",
        ];
        for m in batch {
            for c in m {
                if c.name == "ri_gv" && !acceptable.contains(&c.text.trim()) {
                    count += 1;
                }
            }
        }

        // bare-rescue: rescue without exception class
        fn walk_rescue(node: tree_sitter::Node, count: &mut usize) {
            if node.kind() == "rescue" && node.is_named() {
                let has_exceptions = (0..node.child_count())
                    .filter_map(|i| node.child(i as u32))
                    .any(|c| c.kind() == "exceptions");
                if !has_exceptions {
                    *count += 1;
                }
            }
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i as u32) {
                    walk_rescue(child, count);
                }
            }
        }
        walk_rescue(file.root_node(), &mut count);

        // frozen-string: missing # frozen_string_literal: true
        let first_line = file.content.lines().next().unwrap_or("");
        if !first_line.contains("frozen_string_literal: true") {
            count += 1;
        }

        // negated-if: if !x → should use unless x
        for line in file.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if (trimmed.starts_with("if !") || trimmed.starts_with("if("))
                && trimmed.contains('!')
                && !trimmed.contains("!= ")
            {
                count += 1;
            }
        }

        // ruby-two-space-indent: indentation must be multiple of 2
        for line in file.content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            if indent > 0 && indent % 2 != 0 {
                count += 1;
            }
        }

        count
    }
}

impl RubyAdapter {
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

    fn parse_ruby(code: &str) -> ParsedFile {
        parse_code(code, "test.rb").expect("parse")
    }

    #[test]
    fn test_ruby_count_panic_raise() {
        let code = r#"
def foo
  raise "error"
end
"#;
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 1);
    }

    #[test]
    fn test_ruby_count_panic_clean() {
        let code = r#"
def foo
  puts "hello"
end
"#;
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_panic_calls(&file), 0);
    }

    #[test]
    fn test_ruby_extract_functions() {
        let code = "def foo; end\ndef bar(x); end\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        let fns = adapter.extract_functions(&file);
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "foo");
        assert_eq!(fns[1].name, "bar");
    }

    #[test]
    fn test_ruby_max_nesting_depth_flat() {
        let code = "def foo; 1; end\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        let depth = adapter.max_nesting_depth(&file);
        assert!(depth >= 1, "method body should give depth >= 1");
    }

    #[test]
    fn test_ruby_max_nesting_depth_nested() {
        let code = r#"
def foo
  if true
    if true
      puts "hi"
    end
  end
end
"#;
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        let depth = adapter.max_nesting_depth(&file);
        assert!(
            depth >= 1,
            "nested bodies should give depth >= 1, got {depth}"
        );
    }

    #[test]
    fn test_ruby_max_nesting_depth_empty() {
        let code = "";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.max_nesting_depth(&file), 0);
    }

    #[test]
    fn test_ruby_naming_single_letter() {
        let code = "a = 1\nb = 2\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_ruby_naming_clean() {
        let code = "user_name = \"alice\"\nitem_count = 42\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 0);
    }

    #[test]
    fn test_ruby_naming_terrible() {
        let code = "data = 1\nmanager = 2\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_ruby_naming_meaningless() {
        let code = "foo = 1\naaa = 2\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_naming_violations(&file), 2);
    }

    #[test]
    fn test_ruby_debug_puts() {
        let code = r#"
puts "hello"
print "world"
p x
binding.pry
byebug
"#;
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 4);
    }

    #[test]
    fn test_ruby_debug_clean() {
        let code = "result = add(1, 2)\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_debug_calls(&file), 0);
    }

    #[test]
    fn test_ruby_excessive_params() {
        let code = "def process(a, b, c, d, e, f); end\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 1);
    }

    #[test]
    fn test_ruby_excessive_params_ok() {
        let code = "def process(a, b); end\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_excessive_params(&file, 5), 0);
    }

    #[test]
    fn test_ruby_magic_numbers_expression() {
        let code = "foo(42)\nbar(100)\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 2);
    }

    #[test]
    fn test_ruby_magic_numbers_skips_trivial() {
        let code = "foo(0)\nbar(1)\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }

    #[test]
    fn test_ruby_magic_numbers_skips_declaration() {
        let code = "x = 42\n";
        let file = parse_ruby(code);
        let adapter = RubyAdapter;
        assert_eq!(adapter.count_magic_numbers(&file), 0);
    }
}
