use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::language::Language;

use super::parsers;

/// A tree-sitter parsing engine that supports multiple languages.
///
/// Manages a cache of language-specific parsers and provides
/// a unified interface for parsing source files.
/// Uses a Mutex for interior mutability so parsing works with `&self`
/// and is thread-safe (required by rayon-based parallel analysis).
pub struct TreeSitterEngine {
    parsers: Mutex<HashMap<Language, tree_sitter::Parser>>,
}

impl Default for TreeSitterEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeSitterEngine {
    /// Create a new engine with no pre-loaded parsers.
    pub fn new() -> Self {
        Self {
            parsers: Mutex::new(HashMap::new()),
        }
    }

    /// Lock the parsers map, returning None if the mutex is poisoned.
    fn lock_parsers(
        &self,
    ) -> Option<std::sync::MutexGuard<'_, HashMap<Language, tree_sitter::Parser>>> {
        self.parsers.lock().ok()
    }

    /// Ensure a parser is available for the given language.
    /// Returns true if the parser was successfully loaded (or already exists).
    pub fn ensure_parser(&self, lang: Language) -> bool {
        let mut parsers = match self.lock_parsers() {
            Some(p) => p,
            None => return false,
        };
        if parsers.contains_key(&lang) {
            return true;
        }
        let grammar = match parsers::get_grammar(lang) {
            Some(g) => g,
            None => return false,
        };
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&grammar).is_err() {
            return false;
        }
        parsers.insert(lang, parser);
        true
    }

    /// Parse source code for the given language.
    /// Returns None if the language has no loaded parser or parsing fails.
    pub fn parse(&self, lang: Language, content: &str) -> Option<tree_sitter::Tree> {
        if !self.ensure_parser(lang) {
            return None;
        }
        let mut parsers = self.lock_parsers()?;
        parsers.get_mut(&lang).and_then(|p| p.parse(content, None))
    }

    /// Parse a file, detecting language from its path.
    /// Returns None if the language is unsupported or parsing fails.
    pub fn parse_file(&self, path: &Path, content: &str) -> Option<ParsedFile> {
        let lang = Language::from_path(path);
        if lang == Language::Unknown {
            return None;
        }
        let tree = self.parse(lang, content)?;
        Some(ParsedFile {
            path: path.to_path_buf(),
            content: content.to_string(),
            tree,
            language: lang,
        })
    }

    /// Check if a parser can handle the given language.
    pub fn can_parse(&self, lang: Language) -> bool {
        self.ensure_parser(lang)
    }
}

/// A source file that has been parsed by tree-sitter.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub path: std::path::PathBuf,
    pub content: String,
    pub tree: tree_sitter::Tree,
    pub language: Language,
}

impl ParsedFile {
    /// Get the root node of the syntax tree.
    pub fn root_node(&self) -> tree_sitter::Node<'_> {
        self.tree.root_node()
    }

    /// Get source text for a given node range.
    pub fn node_text(&self, node: tree_sitter::Node) -> &str {
        let start = node.start_byte();
        let end = node.end_byte();
        &self.content[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify Rust file parsing produces a valid tree
    /// Invariants: Root node should be "source_file" with children
    #[test]
    fn test_parse_rust_source() {
        let engine = TreeSitterEngine::new();
        let code = "fn main() { let x = 42; }";
        let tree = engine.parse(Language::Rust, code);
        assert!(tree.is_some(), "Rust parsing should succeed");
        let tree = tree.unwrap();
        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        assert!(root.child_count() > 0, "Root should have child nodes");
    }

    /// Objective: Verify parsing invalid code still produces a tree (error recovery)
    /// Invariants: Tree should exist but contain ERROR nodes
    #[test]
    fn test_parse_invalid_rust() {
        let engine = TreeSitterEngine::new();
        let code = "fn main() { let x = ; }";
        let tree = engine.parse(Language::Rust, code);
        assert!(
            tree.is_some(),
            "Should still produce a tree with error recovery"
        );
    }

    /// Objective: Verify unsupported language returns None
    /// Invariants: Unknown language should not crash, just return None
    #[test]
    fn test_unsupported_language() {
        let engine = TreeSitterEngine::new();
        let result = engine.parse(Language::Unknown, "some code");
        assert!(result.is_none(), "Unknown language should return None");
    }

    /// Objective: Verify parse_file from path detects language correctly
    /// Invariants: .rs file should parse as Rust
    #[test]
    fn test_parse_file_by_path() {
        let engine = TreeSitterEngine::new();
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let result = engine.parse_file(Path::new("test.rs"), code);
        assert!(result.is_some(), "Should parse .rs file");
        let parsed = result.unwrap();
        assert_eq!(parsed.language, Language::Rust);
        assert_eq!(parsed.node_text(parsed.root_node()), code);
    }
}
