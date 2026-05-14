pub mod base_rules;
pub mod complex_rules;
pub mod remaining_rules;
pub mod rust_rules;

use crate::analyzer::Severity;
use crate::language::Language;

use super::query::QueryRule;

/// Returns all tree-sitter based query rules.
pub fn all_rules() -> Vec<QueryRule> {
    vec![single_letter_variable(), deep_nesting()]
}

/// Detect single-letter variable names in let bindings.
///
/// Matches Rust `let` declarations where the variable name
/// consists of a single lowercase letter (e.g. `let a = 1;`).
fn single_letter_variable() -> QueryRule {
    QueryRule {
        name: "single-letter-variable",
        languages: &[Language::Rust, Language::JavaScript, Language::TypeScript],
        pattern: "
            (let_declaration
                pattern: (identifier) @var
                (#match? @var \"^[a-z]$\")
            )
        ",
        severity: Severity::Spicy,
        handler: None,
        skips_test_files: false,
    }
}

/// Detect deeply nested code (depth > 4 levels of block nesting).
///
/// Walks the tree-sitter tree and counts nesting depth for
/// block-like structures. Reports functions whose maximum
/// nesting depth exceeds the threshold.
fn deep_nesting() -> QueryRule {
    QueryRule {
        name: "deep-nesting",
        languages: &[Language::Rust, Language::JavaScript, Language::TypeScript],
        pattern: "
            (if_expression
                consequence: (block) @block
            )
            (for_expression
                body: (block) @block
            )
            (while_expression
                body: (block) @block
            )
            (loop_expression
                body: (block) @block
            )
        ",
        severity: Severity::Nuclear,
        handler: None,
        skips_test_files: false,
    }
}
