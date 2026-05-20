use streaming_iterator::StreamingIterator;

use crate::analyzer::Severity;
use crate::language::Language;
use std::cell::RefCell;
use std::collections::HashMap;

use super::engine::ParsedFile;

/// A tree-sitter query based rule definition.
///
/// This is the primary building block for tree-sitter rules.
/// Each rule specifies a query pattern (in tree-sitter query syntax),
/// the languages it applies to, and a handler that converts
/// query matches into `CodeIssue`s.
pub struct QueryRule {
    /// Unique rule identifier (e.g. "single-letter-variable").
    pub name: &'static str,

    /// Languages this rule applies to.
    pub languages: &'static [Language],

    /// The tree-sitter query pattern string.
    /// Uses standard tree-sitter query syntax with named captures.
    pub pattern: &'static str,

    /// Default severity when match is found.
    pub severity: Severity,

    /// Custom handler to produce issues from a match.
    /// If None, a default handler is used (one issue per match at the capture node).
    pub handler: Option<QueryHandler>,

    /// Whether to skip test files.
    pub skips_test_files: bool,
}

/// Function signature for custom query match handlers.
pub type QueryHandler =
    fn(file: &ParsedFile, captures: &[QueryCapture], match_index: usize) -> Vec<IssueCandidate>;

/// A single named capture from a tree-sitter query match.
#[derive(Debug, Clone)]
pub struct QueryCapture<'a> {
    /// The capture name from the query pattern (e.g. "ident" from `(identifier) @ident`).
    pub name: String,

    /// The matched syntax node.
    pub node: tree_sitter::Node<'a>,

    /// Source text of the matched node.
    pub text: &'a str,
}

/// An issue candidate produced by a query rule handler.
#[derive(Debug, Clone)]
pub struct IssueCandidate {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: Severity,
}

thread_local! {
    static QUERY_CACHE: RefCell<HashMap<(Language, String), tree_sitter::Query>> =
        RefCell::new(HashMap::new());
}

/// Execute a tree-sitter query against a parsed file and collect captures.
///
/// Returns a list of capture groups, one per query match.
/// Each group contains all named captures for that match.
/// Uses a thread-local cache to avoid recompiling the same query pattern.
pub fn collect_captures<'a>(
    file: &'a ParsedFile,
    pattern: &str,
) -> Result<Vec<Vec<QueryCapture<'a>>>, String> {
    QUERY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let key = (file.language, pattern.to_string());
        if !cache.contains_key(&key) {
            let grammar = super::parsers::get_grammar(file.language).ok_or_else(|| {
                format!(
                    "No tree-sitter grammar available for {}",
                    file.language.display_name()
                )
            })?;
            let query = tree_sitter::Query::new(&grammar, pattern)
                .map_err(|e| format!("Failed to create query: {}", e))?;
            cache.insert(key.clone(), query);
        }
        let query = cache
            .get(&key)
            .ok_or_else(|| "Query cache miss for pattern".to_string())?;

        let mut cursor = tree_sitter::QueryCursor::new();
        let root = file.root_node();
        let mut matches = cursor.matches(query, root, file.content.as_bytes());

        let capture_names: Vec<String> = query
            .capture_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut result = Vec::new();

        while let Some(match_) = matches.next() {
            let captures: Vec<QueryCapture> = match_
                .captures
                .iter()
                .map(|capture| {
                    let name_idx = capture.index as usize;
                    let name = capture_names.get(name_idx).cloned().unwrap_or_else(|| {
                        tracing::warn!(
                            "capture index {} out of bounds (max {}); using 'unknown'",
                            name_idx,
                            capture_names.len()
                        );
                        "unknown".to_string()
                    });
                    let node = capture.node;
                    let start = node.start_byte();
                    let end = node.end_byte();
                    QueryCapture {
                        name,
                        node,
                        text: &file.content[start..end],
                    }
                })
                .collect();
            result.push(captures);
        }

        Ok(result)
    })
}

/// Execute a merged multi-pattern query in a single cursor traversal.
///
/// Concatenates all patterns with newlines and runs one query pass.
/// Capture names must be unique across all patterns (use prefixed names).
pub fn collect_captures_multi<'a>(
    file: &'a ParsedFile,
    patterns: &[&str],
) -> Result<Vec<Vec<QueryCapture<'a>>>, String> {
    let merged = patterns.join("\n");
    collect_captures(file, &merged)
}

/// Run a `QueryRule` against a parsed file and produce issues.
pub fn run_query_rule(file: &ParsedFile, rule: &QueryRule) -> Vec<IssueCandidate> {
    let captures_group = match collect_captures(file, rule.pattern) {
        Ok(groups) => groups,
        Err(e) => {
            tracing::warn!("Query rule '{}' error: {}", rule.name, e);
            return vec![];
        }
    };

    let mut results = Vec::new();

    for (match_index, captures) in captures_group.iter().enumerate() {
        if let Some(handler) = rule.handler {
            results.extend(handler(file, captures, match_index));
        } else {
            // Default handler: use the first capture's location
            if let Some(first) = captures.first() {
                results.push(IssueCandidate {
                    line: first.node.start_position().row + 1,
                    column: first.node.start_position().column + 1,
                    message: format!("{} detected", rule.name),
                    severity: rule.severity.clone(),
                });
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::TreeSitterEngine;

    /// Objective: Verify basic query matching works for Rust source
    /// Invariants: Query for identifiers should find all identifier nodes
    #[test]
    fn test_collect_captures_basic() {
        let engine = TreeSitterEngine::new();
        let code = "fn main() { let x = 42; }";
        let file = engine
            .parse_file(std::path::Path::new("test.rs"), code)
            .expect("Should parse");

        let captures = collect_captures(&file, "(identifier) @id").expect("Query should succeed");
        assert!(!captures.is_empty(), "Should find at least one identifier");
        // Should find: main, x
        assert_eq!(captures.len(), 2, "Should find 2 identifiers: main, x");
    }

    /// Objective: Verify single-letter variable detection via query
    /// Invariants: Pattern matching single-character identifiers should catch them
    #[test]
    fn test_single_letter_variable_query() {
        let engine = TreeSitterEngine::new();

        let code = "fn compute() { let a = 1; let bb = 2; let ccc = 3; }";
        let file = engine
            .parse_file(std::path::Path::new("test.rs"), code)
            .expect("Should parse");

        // Match `let` bindings with single-letter patterns
        let pattern = "
            (let_declaration
                pattern: (identifier) @var
                (#match? @var \"^[a-z]$\")
            )
        ";
        let captures = collect_captures(&file, pattern).expect("Query should succeed");

        // The `let` pattern: `let a = ...; let bb = ...; let ccc = ...;`
        // Only `a` should match (single letter)
        assert_eq!(
            captures.len(),
            1,
            "Only 'a' should match single-letter pattern"
        );
        if let Some(first) = captures.first().and_then(|c| c.first()) {
            assert_eq!(first.text, "a", "Should capture 'a'");
        }
    }

    /// Objective: Verify invalid query returns an error
    /// Invariants: Malformed query pattern should not panic
    #[test]
    fn test_invalid_query_returns_error() {
        let engine = TreeSitterEngine::new();
        let code = "fn main() {}";
        let file = engine
            .parse_file(std::path::Path::new("test.rs"), code)
            .expect("Should parse");

        let result = collect_captures(&file, "(nonexistent_node) @x");
        // Unknown node type in query should be an error
        assert!(result.is_err(), "Query with unknown node type should error");
    }

    /// Objective: Verify QueryRule default handler produces issues
    /// Invariants: A QueryRule with no custom handler should still produce issues
    #[test]
    fn test_query_rule_default_handler() {
        let engine = TreeSitterEngine::new();
        let code = "fn main() { let x = 1; let y = 2; }";
        let file = engine
            .parse_file(std::path::Path::new("test.rs"), code)
            .expect("Should parse");

        let rule = QueryRule {
            name: "single-letter-var",
            languages: &[Language::Rust],
            pattern: "
                (let_declaration
                    pattern: (identifier) @var
                    (#match? @var \"^[a-z]$\")
                )
            ",
            severity: Severity::Spicy,
            handler: None,
            skips_test_files: false,
        };

        let issues = run_query_rule(&file, &rule);
        assert_eq!(issues.len(), 2, "Should find 2 single-letter variables");
        assert_eq!(issues[0].message, "single-letter-var detected");
    }
}
