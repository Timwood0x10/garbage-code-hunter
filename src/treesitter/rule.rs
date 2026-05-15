use std::path::Path;

use crate::analyzer::CodeIssue;
use crate::context::{FileContext, ProjectConfig};
use crate::language::Language;

use super::engine::ParsedFile;

/// A code quality rule that analyzes source files using tree-sitter AST.
///
/// Unlike the original [`crate::rules::Rule`] trait which requires `syn::File`,
/// this trait works on tree-sitter's language-agnostic CST. Rules declare
/// which languages they support via [`supported_languages`](TreeSitterRule::supported_languages).
pub trait TreeSitterRule: Send + Sync {
    /// Unique identifier for this rule (e.g. `"deep-nesting"`).
    fn name(&self) -> &'static str;

    /// Languages supported by this rule.
    fn supported_languages(&self) -> &'static [Language];

    /// Whether to skip test files (default: true).
    fn skips_test_files(&self) -> bool {
        true
    }

    /// Analyze a parsed file and return detected issues.
    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue>;

    /// Analyze a file with additional context about the file's role in the project.
    /// Override this when the rule needs to adjust behavior based on file context
    /// (e.g. skipping UI files, relaxing thresholds for examples) or config.
    #[allow(clippy::too_many_arguments)]
    fn check_with_context(
        &self,
        file: &ParsedFile,
        _is_test_file: bool,
        _context: &FileContext,
        _config: &ProjectConfig,
    ) -> Vec<CodeIssue> {
        self.check(file)
    }
}

/// Engine that runs all registered tree-sitter rules against parsed files.
///
/// Supports both trait-based [`TreeSitterRule`] implementations and
/// declarative [`QueryRule`](crate::treesitter::query::QueryRule) definitions.
pub struct TreeSitterRuleEngine {
    rules: Vec<Box<dyn TreeSitterRule>>,
}

impl Default for TreeSitterRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeSitterRuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Register a trait-based rule.
    pub fn add(&mut self, rule: Box<dyn TreeSitterRule>) {
        self.rules.push(rule);
    }

    /// Register a declarative query-based rule by wrapping it.
    pub fn add_query_rule(&mut self, query_rule: crate::treesitter::query::QueryRule) {
        self.rules.push(Box::new(QueryRuleAdapter::new(query_rule)));
    }

    /// Register multiple query rules at once.
    pub fn add_query_rules(&mut self, query_rules: Vec<crate::treesitter::query::QueryRule>) {
        for qr in query_rules {
            self.add_query_rule(qr);
        }
    }

    /// Run all applicable rules against a parsed file.
    pub fn check_file(&self, file: &ParsedFile, is_test_file: bool) -> Vec<CodeIssue> {
        self.check_file_with_context(
            file,
            is_test_file,
            &FileContext::from_path(&file.path),
            &ProjectConfig::default(),
        )
    }

    /// Run all applicable rules with full context and config.
    pub fn check_file_with_context(
        &self,
        file: &ParsedFile,
        is_test_file: bool,
        context: &FileContext,
        config: &ProjectConfig,
    ) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for rule in &self.rules {
            if is_test_file && rule.skips_test_files() {
                continue;
            }
            if !rule.supported_languages().contains(&file.language) {
                continue;
            }
            if Self::is_rule_disabled(config, rule.name()) {
                continue;
            }
            issues.extend(rule.check_with_context(file, is_test_file, context, config));
        }
        issues
    }

    /// Check if a rule is disabled by project config.
    fn is_rule_disabled(config: &ProjectConfig, rule_name: &str) -> bool {
        match rule_name {
            "terrible-naming"
            | "single-letter-variable"
            | "meaningless-naming"
            | "hungarian-notation"
            | "abbreviation-abuse" => !config.rules.naming.enabled,
            "unwrap-abuse" => !config.rules.unwrap.enabled,
            "magic-number" => !config.rules.magic_number.enabled,
            "println-debugging" => !config.rules.println.enabled,
            _ => false,
        }
    }

    /// Check if a file path indicates a test file (shared logic).
    pub fn is_test_file(path: &Path, content: &str) -> bool {
        let path_str = path.to_string_lossy();
        let normalized = path_str.strip_prefix("./").unwrap_or(&path_str);

        if normalized.contains("/tests/")
            || normalized.contains("\\tests\\")
            || normalized.starts_with("tests/")
            || normalized.starts_with("tests\\")
            || normalized.contains("/test/")
            || normalized.contains("\\test\\")
            || normalized.ends_with("_test.rs")
            || normalized.ends_with("_tests.rs")
            || normalized.ends_with("_test.py")
            || normalized.ends_with("_test.js")
            || normalized.ends_with("_test.ts")
            || normalized.ends_with("_test.go")
            || normalized.ends_with("_test.java")
            || normalized.starts_with("test_")
        {
            return true;
        }
        if normalized.contains("/examples/")
            || normalized.contains("\\examples\\")
            || normalized.starts_with("examples/")
            || normalized.starts_with("examples\\")
        {
            return true;
        }
        if normalized.contains("/benches/")
            || normalized.contains("\\benches\\")
            || normalized.starts_with("benches/")
            || normalized.starts_with("benches\\")
        {
            return true;
        }

        content.contains("#[cfg(test)]")
    }

    pub fn rule_names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|r| r.name()).collect()
    }
}

/// Adapter that wraps a [`QueryRule`] as a [`TreeSitterRule`] trait object.
///
/// This enables declarative query-based rules to be used alongside
/// imperative trait-based rules within the same engine.
struct QueryRuleAdapter {
    rule: crate::treesitter::query::QueryRule,
}

impl QueryRuleAdapter {
    fn new(rule: crate::treesitter::query::QueryRule) -> Self {
        Self { rule }
    }
}

impl TreeSitterRule for QueryRuleAdapter {
    fn name(&self) -> &'static str {
        self.rule.name
    }

    fn supported_languages(&self) -> &'static [Language] {
        self.rule.languages
    }

    fn skips_test_files(&self) -> bool {
        self.rule.skips_test_files
    }

    fn check(&self, file: &ParsedFile) -> Vec<CodeIssue> {
        let candidates = crate::treesitter::query::run_query_rule(file, &self.rule);
        candidates
            .into_iter()
            .map(|c| CodeIssue {
                file_path: file.path.clone(),
                line: c.line,
                column: c.column,
                rule_name: self.rule.name.to_string(),
                message: c.message,
                severity: c.severity,
            })
            .collect()
    }

    fn check_with_context(
        &self,
        file: &ParsedFile,
        _is_test_file: bool,
        _context: &FileContext,
        _config: &ProjectConfig,
    ) -> Vec<CodeIssue> {
        // Context-aware rules override this; for query rules, just run check
        self.check(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treesitter::engine::TreeSitterEngine;

    struct DummyRule;

    impl TreeSitterRule for DummyRule {
        fn name(&self) -> &'static str {
            "dummy"
        }
        fn supported_languages(&self) -> &'static [Language] {
            &[Language::Rust]
        }
        fn check(&self, _file: &ParsedFile) -> Vec<CodeIssue> {
            vec![]
        }
    }

    /// Objective: Verify engine runs rules only for matching languages
    /// Invariants: Rule should not fire for unsupported languages
    #[test]
    fn test_rule_language_filtering() {
        let mut engine = TreeSitterRuleEngine::new();
        engine.add(Box::new(DummyRule));

        let ts = TreeSitterEngine::new();
        let file = ts
            .parse_file(Path::new("test.rs"), "fn main() {}")
            .expect("Should parse");

        let issues = engine.check_file(&file, false);
        assert!(issues.is_empty(), "Dummy rule produces no issues");

        assert_eq!(engine.rule_names(), vec!["dummy"]);
    }

    /// Objective: Verify test file detection works across languages
    /// Invariants: Various test file naming patterns should be recognized
    #[test]
    fn test_is_test_file_various_patterns() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("src/tests/mod.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("tests/test_main.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("foo_test.py"),
            ""
        ));
        assert!(!TreeSitterRuleEngine::is_test_file(
            Path::new("src/main.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("src/lib.rs"),
            "#[cfg(test)]"
        ));
    }
}
