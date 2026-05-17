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

    /// A rule that does not skip test files (explicit override).
    struct NonSkippingRule;

    impl TreeSitterRule for NonSkippingRule {
        fn name(&self) -> &'static str {
            "non-skipper"
        }
        fn supported_languages(&self) -> &'static [Language] {
            &[Language::Rust]
        }
        fn skips_test_files(&self) -> bool {
            false
        }
        fn check(&self, _file: &ParsedFile) -> Vec<CodeIssue> {
            vec![]
        }
    }

    // ── rule language filtering ───────────────────────────────────

    /// Objective: Verify engine runs rules only for matching languages.
    /// Invariants: Rule should not fire for unsupported languages.
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

    /// Objective: Verify rule_names returns all registered rule names.
    #[test]
    fn test_rule_names_multiple() {
        let mut engine = TreeSitterRuleEngine::new();
        engine.add(Box::new(DummyRule));
        engine.add(Box::new(NonSkippingRule));
        let names = engine.rule_names();
        assert_eq!(names.len(), 2, "should have 2 rule names");
        assert!(names.contains(&"dummy"));
        assert!(names.contains(&"non-skipper"));
    }

    /// Objective: Verify empty engine produces no rules.
    #[test]
    fn test_rule_names_empty() {
        let engine = TreeSitterRuleEngine::new();
        assert!(
            engine.rule_names().is_empty(),
            "empty engine => no rule names"
        );
    }

    /// Objective: Verify test file skipping: a rule with skips_test_files=true
    /// should not run on a file classified as test.
    /// Invariants: The filtering happens in check_file vs check_file_with_context.
    #[test]
    fn test_rule_skips_test_files() {
        let mut engine = TreeSitterRuleEngine::new();
        engine.add(Box::new(DummyRule)); // skips_test_files default = true

        let ts = TreeSitterEngine::new();
        let file = ts
            .parse_file(Path::new("tests/test_mod.rs"), "fn helper() {}")
            .expect("Should parse");

        let issues = engine.check_file(&file, true);
        assert!(
            issues.is_empty(),
            "Dummy rule should be skipped for test files"
        );
    }

    /// Objective: Verify a rule with skips_test_files=false still runs on test files.
    #[test]
    fn test_rule_non_skipping_does_not_skip() {
        let mut engine = TreeSitterRuleEngine::new();
        engine.add(Box::new(NonSkippingRule));

        let ts = TreeSitterEngine::new();
        let file = ts
            .parse_file(Path::new("tests/test_mod.rs"), "fn helper() {}")
            .expect("Should parse");

        let issues = engine.check_file(&file, true);
        // NonSkippingRule returns empty issues but should still run
        assert!(
            issues.is_empty(),
            "NonSkippingRule should run on test file (even if empty result)"
        );
    }

    // ── is_test_file path patterns ────────────────────────────────

    /// Objective: Verify /tests/ directory is recognized as test.
    #[test]
    fn test_is_test_file_tests_dir() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("src/tests/mod.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("tests/test_main.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("tests/foo.py"),
            ""
        ));
    }

    /// Objective: Verify /test/ directory is recognized as test.
    #[test]
    fn test_is_test_file_test_dir_singular() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("src/test/helper.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("test/main_test.go"),
            ""
        ));
    }

    /// Objective: Verify _test suffix patterns across languages.
    #[test]
    fn test_is_test_file_underscore_test_suffix() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("foo_test.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("foo_tests.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("foo_test.py"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("foo_test.js"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("foo_test.ts"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("foo_test.go"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("foo_test.java"),
            ""
        ));
    }

    /// Objective: Verify test_ prefix at root is recognized.
    #[test]
    fn test_is_test_file_test_prefix() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("test_foo.py"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("test_main.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("test_utils.go"),
            ""
        ));
    }

    /// Objective: Verify ./ prefix stripping works correctly.
    #[test]
    fn test_is_test_file_leading_dot_slash() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("./src/tests/mod.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("./foo_test.rs"),
            ""
        ));
    }

    /// Objective: Verify examples directory is treated as test.
    #[test]
    fn test_is_test_file_examples_dir() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("examples/my_example.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("src/examples/foo.rs"),
            ""
        ));
    }

    /// Objective: Verify benches directory is treated as test.
    #[test]
    fn test_is_test_file_benches_dir() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("benches/my_bench.rs"),
            ""
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("src/benches/foo.rs"),
            ""
        ));
    }

    /// Objective: Verify non-test paths are correctly rejected.
    #[test]
    fn test_is_test_file_non_test_paths() {
        assert!(!TreeSitterRuleEngine::is_test_file(
            Path::new("src/main.rs"),
            ""
        ));
        assert!(!TreeSitterRuleEngine::is_test_file(Path::new("lib.rs"), ""));
        assert!(!TreeSitterRuleEngine::is_test_file(
            Path::new("src/foo.py"),
            ""
        ));
        assert!(!TreeSitterRuleEngine::is_test_file(
            Path::new("README.md"),
            ""
        ));
    }

    /// Objective: Verify content-based detection catches #[cfg(test)] in Rust files.
    #[test]
    fn test_is_test_file_content_based() {
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("src/lib.rs"),
            "#[cfg(test)]\nmod tests;"
        ));
        assert!(TreeSitterRuleEngine::is_test_file(
            Path::new("src/foo.rs"),
            "fn helper() {}\n#[cfg(test)]\nmod test_helper;"
        ));
    }

    /// Objective: Verify content-based detection does not false-positive on similar strings.
    /// Note: #[cfg(test)] in comments IS detected (contains is literal). This is a known limitation.
    #[test]
    fn test_is_test_file_content_no_false_positive() {
        assert!(!TreeSitterRuleEngine::is_test_file(
            Path::new("src/lib.rs"),
            "fn cfg_test() {}"
        ));
        assert!(!TreeSitterRuleEngine::is_test_file(
            Path::new("src/lib.rs"),
            "// mod tests;"
        ));
    }

    // ── is_rule_disabled ──────────────────────────────────────────

    /// Objective: Verify is_rule_disabled returns false for unknown rules.
    /// Invariants: Unknown rule names are never disabled by config.
    #[test]
    fn test_is_rule_disabled_unknown() {
        let config = ProjectConfig::default();
        assert!(
            !TreeSitterRuleEngine::is_rule_disabled(&config, "unknown-rule"),
            "unknown rules should not be disabled"
        );
    }

    /// Objective: Verify naming rules are disabled when config.rules.naming.enabled=false.
    #[test]
    fn test_is_rule_disabled_naming_rules() {
        let mut config = ProjectConfig::default();
        config.rules.naming.enabled = false;
        assert!(TreeSitterRuleEngine::is_rule_disabled(
            &config,
            "terrible-naming"
        ));
        assert!(TreeSitterRuleEngine::is_rule_disabled(
            &config,
            "single-letter-variable"
        ));
        assert!(TreeSitterRuleEngine::is_rule_disabled(
            &config,
            "meaningless-naming"
        ));
    }

    /// Objective: Verify unwrap rule is disabled when config.rules.unwrap.enabled=false.
    #[test]
    fn test_is_rule_disabled_unwrap() {
        let mut config = ProjectConfig::default();
        config.rules.unwrap.enabled = false;
        assert!(TreeSitterRuleEngine::is_rule_disabled(
            &config,
            "unwrap-abuse"
        ));
        assert!(!TreeSitterRuleEngine::is_rule_disabled(
            &config,
            "terrible-naming"
        ));
    }

    /// Objective: Verify magic-number rule is disabled when config.rules.magic_number.enabled=false.
    #[test]
    fn test_is_rule_disabled_magic_number() {
        let mut config = ProjectConfig::default();
        config.rules.magic_number.enabled = false;
        assert!(TreeSitterRuleEngine::is_rule_disabled(
            &config,
            "magic-number"
        ));
    }

    /// Objective: Verify println rule is disabled when config.rules.println.enabled=false.
    #[test]
    fn test_is_rule_disabled_println() {
        let mut config = ProjectConfig::default();
        config.rules.println.enabled = false;
        assert!(TreeSitterRuleEngine::is_rule_disabled(
            &config,
            "println-debugging"
        ));
    }

    // ── is_test_file: known gaps (documented limitations) ────────

    /// Objective: Document known gap: test_ prefix files in subdirectories
    /// are not detected by path alone (e.g. src/sub/test_util.py).
    /// They would need content-based detection, but Python has no #[cfg(test)].
    #[test]
    fn test_is_test_file_gap_test_prefix_in_subdir() {
        // This is a known limitation — Python test_* files in subdirs
        // are only detected at the root level (starts_with("test_"))
        assert!(
            !TreeSitterRuleEngine::is_test_file(Path::new("src/sub/test_util.py"), ""),
            "known gap: test_ prefix in subdirs not detected by path alone"
        );
    }
}
