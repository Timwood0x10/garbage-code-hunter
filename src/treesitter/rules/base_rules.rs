use crate::analyzer::{CodeIssue, Severity};
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::collect_captures;
use crate::treesitter::rule::TreeSitterRule;

/// Node types that contribute to nesting depth, across all supported languages.
pub(crate) const BLOCK_PARENT_TYPES: &[&str] = &[
    // Rust
    "if_expression",
    "for_expression",
    "while_expression",
    "loop_expression",
    "match_expression",
    "match_arm",
    // Python
    "if_statement",
    "for_statement",
    "while_statement",
    "try_statement",
    "with_statement",
    "function_definition",
    "class_definition",
    // JavaScript
    "if_statement",
    "for_statement",
    "while_statement",
    "do_statement",
    "try_statement",
    "switch_case",
    "function_declaration",
    // C/C++
    "if_statement",
    "for_statement",
    "while_statement",
    "do_statement",
    "switch_statement",
    "case_statement",
    // Go
    "expression_switch_statement",
    "type_switch_statement",
    "select_statement",
    // Ruby
    "begin_block",
    // Swift
    "switch_statement",
    // Zig
    "switch_expression",
];

/// Block-like node kinds across languages (where nesting depth increments).
pub(crate) const BLOCK_KINDS: &[&str] = &[
    "block",              // Rust, Python
    "statement_block",    // JavaScript
    "compound_statement", // C/C++
];

/// Count-and-report rule: runs a tree-sitter query, counts matches,
/// reports a single aggregated issue when count exceeds threshold.
pub(crate) struct CountRule {
    pub(crate) name: &'static str,
    pub(crate) pattern: &'static str,
    pub(crate) threshold: usize,
    pub(crate) severity: Severity,
    pub(crate) message_fn: fn(usize) -> String,
    pub(crate) languages: &'static [Language],
}

impl TreeSitterRule for CountRule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn supported_languages(&self) -> &'static [Language] {
        self.languages
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let captures = match collect_captures(file, self.pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let count: usize = captures.iter().map(|c| c.len()).sum();
        if count > self.threshold {
            vec![CodeIssue {
                file_path: file.path.clone(),
                line: 1,
                column: 1,
                rule_name: self.name.to_string(),
                message: (self.message_fn)(count),
                severity: self.severity.clone(),
            }]
        } else {
            vec![]
        }
    }
}

/// Macro-count rule: finds `name!()` calls by querying macro_invocation
/// with identifier capture, then filters by macro name in Rust code.
pub(crate) struct MacroRule {
    pub(crate) name: &'static str,
    pub(crate) macro_name: &'static str,
    pub(crate) threshold: usize,
    pub(crate) severity: Severity,
    pub(crate) message_fn: fn(usize) -> String,
}

impl TreeSitterRule for MacroRule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let pattern = "(macro_invocation macro: (identifier) @m)";
        let captures = match collect_captures(file, pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let count = captures
            .iter()
            .filter_map(|group| group.first())
            .filter(|cap| cap.text == self.macro_name)
            .count();
        if count > self.threshold {
            vec![CodeIssue {
                file_path: file.path.clone(),
                line: 1,
                column: 1,
                rule_name: self.name.to_string(),
                message: (self.message_fn)(count),
                severity: self.severity.clone(),
            }]
        } else {
            vec![]
        }
    }
}

/// Method-call-count rule: finds `.method()` calls by querying field_expression
/// with identifier capture, then filters by method name in Rust code.
pub(crate) struct MethodCallRule {
    pub(crate) name: &'static str,
    pub(crate) method_name: &'static str,
    pub(crate) threshold: usize,
    pub(crate) severity_fn: fn(usize) -> Severity,
    pub(crate) message_fn: fn(usize) -> String,
}

impl TreeSitterRule for MethodCallRule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        // Query all field accesses, then filter by method name
        let pattern = "(field_expression field: (field_identifier) @method)";
        let captures = match collect_captures(file, pattern) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let count = captures
            .iter()
            .filter_map(|group| group.first())
            .filter(|cap| cap.text == self.method_name)
            .count();
        if count > self.threshold {
            vec![CodeIssue {
                file_path: file.path.clone(),
                line: 1,
                column: 1,
                rule_name: self.name.to_string(),
                message: (self.message_fn)(count),
                severity: (self.severity_fn)(count),
            }]
        } else {
            vec![]
        }
    }
}

pub(crate) fn find_function_name(node: tree_sitter::Node, content: &[u8]) -> String {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "identifier" {
            if let Ok(text) = child.utf8_text(content) {
                return text.to_string();
            }
        }
    }
    "<anonymous>".to_string()
}

pub(crate) fn count_parameters(func_node: tree_sitter::Node) -> u32 {
    let mut cursor = func_node.walk();
    for child in func_node.named_children(&mut cursor) {
        if child.kind() == "parameters" {
            return child.named_child_count() as u32;
        }
    }
    0
}

pub(crate) fn count_descendants_of_types(node: tree_sitter::Node, types: &[&str]) -> u32 {
    let mut count = 0;
    let mut cursor = node.walk();
    let mut visited_children = false;

    loop {
        let current = cursor.node();
        if !visited_children && types.contains(&current.kind()) {
            count += 1;
        }

        if (!visited_children && cursor.goto_first_child()) || cursor.goto_next_sibling() {
            visited_children = false;
        } else if !cursor.goto_parent() {
            break;
        } else {
            visited_children = true;
        }
    }

    count
}

pub(crate) fn closure_depth(node: tree_sitter::Node) -> u32 {
    let mut depth = 1;
    let mut current = node;
    while let Some(parent) = current.parent() {
        if parent.kind() == "closure_expression" {
            depth += 1;
        }
        current = parent;
    }
    depth
}
