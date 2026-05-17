use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::context::{FileContext, ProjectConfig};
use crate::language::{Language, SUPPORTED_EXTENSIONS};
use crate::rules::generic::GenericRuleEngine;
use crate::treesitter::duplication::{CrossFileDupDetector, IntraFileDupDetector};
use crate::treesitter::{TreeSitterEngine, TreeSitterRuleEngine};

#[derive(Debug, Clone)]
pub struct CodeIssue {
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub rule_name: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Mild,    // Minor issues
    Spicy,   // Medium issues
    Nuclear, // Serious issues
}

pub struct CodeAnalyzer {
    generic_engine: GenericRuleEngine,
    ts_engine: TreeSitterEngine,
    ts_rule_engine: TreeSitterRuleEngine,
    exclude_patterns: Vec<Regex>,
    project_config: ProjectConfig,
    lang: String,
    cross_detector: std::cell::RefCell<CrossFileDupDetector>,
}

impl CodeAnalyzer {
    pub fn rule_names(&self) -> Vec<&'static str> {
        self.ts_rule_engine.rule_names()
    }

    pub fn new(exclude_patterns: &[String], lang: &str) -> Self {
        Self::with_config(exclude_patterns, lang, ProjectConfig::default())
    }

    pub fn infection_spread(&self) -> HashMap<String, Vec<(String, usize, Vec<String>)>> {
        self.cross_detector.borrow().infection_spread()
    }

    pub fn with_config(exclude_patterns: &[String], lang: &str, config: ProjectConfig) -> Self {
        // Default exclude patterns for common build/dependency directories
        let default_excludes = [
            "target",
            "node_modules",
            ".git",
            ".svn",
            ".hg",
            "build",
            "dist",
            "out",
            "__pycache__",
            ".DS_Store",
        ];

        let mut all_patterns: Vec<String> =
            default_excludes.iter().map(|s| s.to_string()).collect();
        all_patterns.extend(exclude_patterns.iter().cloned());

        // Also add exclude patterns from project config
        all_patterns.extend(config.whitelists.exclude_patterns.clone());

        let patterns = all_patterns
            .iter()
            .filter_map(|pattern| {
                // Convert glob patterns to regular expressions
                let regex_pattern = pattern
                    .replace(".", r"\.")
                    .replace("*", ".*")
                    .replace("?", ".");
                Regex::new(&regex_pattern).ok()
            })
            .collect();

        let mut ts_rule_engine = TreeSitterRuleEngine::new();
        crate::treesitter::rules::rust_rules::register_rust_rules(&mut ts_rule_engine);
        crate::treesitter::rules::c_rules::register_c_rules(&mut ts_rule_engine);
        crate::treesitter::rules::go_rules::register_go_rules(&mut ts_rule_engine);
        crate::treesitter::rules::python_rules::register_python_rules(&mut ts_rule_engine);
        crate::treesitter::rules::ts_rules::register_ts_rules(&mut ts_rule_engine);
        crate::treesitter::rules::java_rules::register_java_rules(&mut ts_rule_engine);
        crate::treesitter::rules::ruby_rules::register_ruby_rules(&mut ts_rule_engine);

        Self {
            generic_engine: GenericRuleEngine::new(),
            ts_engine: TreeSitterEngine::new(),
            ts_rule_engine,
            exclude_patterns: patterns,
            project_config: config,
            lang: lang.to_string(),
            cross_detector: std::cell::RefCell::new(CrossFileDupDetector::new()),
        }
    }

    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns
            .iter()
            .any(|pattern| pattern.is_match(&path_str))
    }

    pub fn analyze_path(&self, path: &Path) -> Vec<CodeIssue> {
        if path.is_file() {
            if !self.should_exclude(path) {
                let lang = Language::from_path(path);
                if lang != Language::Unknown {
                    return self.analyze_file(path);
                }
            }
            return Vec::new();
        }

        if !path.is_dir() {
            return Vec::new();
        }

        // Collect all supported source files
        let files: Vec<PathBuf> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !self.should_exclude(e.path()))
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| SUPPORTED_EXTENSIONS.contains(&ext))
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // Phase 1: Parallel single-file analysis for all languages
        let mut issues: Vec<CodeIssue> = files
            .iter()
            .flat_map(|file_path| self.analyze_file(file_path))
            .collect();

        // Phase 1.5: Filter out generated files from further phases
        let real_files: Vec<&PathBuf> = files
            .iter()
            .filter(|p| !Self::is_generated_file(p))
            .collect();

        // Phase 2: Cross-file duplication detection (tree-sitter based)
        *self.cross_detector.borrow_mut() = CrossFileDupDetector::new();
        for file_path in &real_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                if let Some(parsed) = self.ts_engine.parse_file(file_path, &content) {
                    self.cross_detector.borrow_mut().process_file(&parsed);
                }
            }
        }
        issues.extend(self.cross_detector.borrow().find_duplicates());
        issues.extend(self.cross_detector.borrow().find_near_duplicates());

        // Phase 3: Intra-file code duplication
        for file_path in &real_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                if let Some(parsed) = self.ts_engine.parse_file(file_path, &content) {
                    issues.extend(IntraFileDupDetector::check(&parsed));
                }
            }
        }

        issues
    }

    fn is_generated_file(path: &Path) -> bool {
        let name = path.to_string_lossy();
        // Protobuf generated files
        name.ends_with(".pb.go")
            || name.contains("_grpc.pb.go")
            || name.ends_with(".pb.gw.go")
            || name.ends_with(".pulsar.go")
            || name.ends_with(".pb.cc")
            || name.ends_with(".pb.h")
        // Dependencies
            || name.contains("/node_modules/")
            || name.contains("\\node_modules\\")
            || name.contains("/vendor/")
            || name.contains("\\vendor\\")
        // Minified bundles
            || name.contains("/swagger-ui/")
    }

    pub fn analyze_file(&self, file_path: &Path) -> Vec<CodeIssue> {
        if Self::is_generated_file(file_path) {
            return vec![];
        }

        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return vec![],
        };

        let lang = Language::from_path(file_path);
        let is_test_file = Self::is_test_file(file_path, &content);

        // Use tree-sitter for all languages with grammar support
        if let Some(parsed) = self.ts_engine.parse_file(file_path, &content) {
            let context = FileContext::from_path(file_path);
            self.ts_rule_engine.check_file_with_context(
                &parsed,
                is_test_file,
                &context,
                &self.project_config,
            )
        } else if lang == Language::C || lang == Language::Cpp {
            // Fallback to generic text-based rules for C/C++
            self.generic_engine
                .check_file(file_path, &content, &self.lang)
        } else {
            vec![]
        }
    }

    fn is_test_file(path: &Path, content: &str) -> bool {
        let path_str = path.to_string_lossy();
        // Normalize: strip leading "./" for consistent matching
        let normalized = path_str.strip_prefix("./").unwrap_or(&path_str);

        // Check file path patterns (Rust + C/C++)
        if normalized.contains("/tests/")
            || normalized.contains("\\tests\\")
            || normalized.starts_with("tests/")
            || normalized.starts_with("tests\\")
            || normalized.contains("/test/")
            || normalized.contains("\\test\\")
            || normalized.ends_with("_test.rs")
            || normalized.ends_with("_tests.rs")
            || normalized.ends_with("_test.c")
            || normalized.ends_with("_test.cpp")
            || normalized.ends_with("_test.cc")
            || normalized.starts_with("test_")
        {
            return true;
        }
        // Check for example files (singular and plural)
        if normalized.contains("/examples/")
            || normalized.contains("\\examples\\")
            || normalized.starts_with("examples/")
            || normalized.starts_with("examples\\")
            || normalized.contains("/example/")
            || normalized.contains("\\example\\")
            || normalized.starts_with("example/")
            || normalized.starts_with("example\\")
            || normalized.ends_with("_example.rs")
            || normalized.ends_with("_examples.rs")
        {
            return true;
        }
        // Check for benchmark files
        if normalized.contains("/benches/")
            || normalized.contains("\\benches\\")
            || normalized.starts_with("benches/")
            || normalized.starts_with("benches\\")
            || normalized.ends_with("_bench.rs")
            || normalized.ends_with("_benches.rs")
        {
            return true;
        }
        // Check for test-files directories
        if normalized.contains("/test-files/")
            || normalized.contains("\\test-files\\")
            || normalized.starts_with("test-files/")
            || normalized.starts_with("test-files\\")
            || normalized.contains("/test_files/")
            || normalized.contains("\\test_files\\")
        {
            return true;
        }
        // Check for fixture/mock directories
        if normalized.contains("/fixtures/")
            || normalized.contains("\\fixtures\\")
            || normalized.contains("/mocks/")
            || normalized.contains("\\mocks\\")
        {
            return true;
        }
        // Check for #[cfg(test)] module in content (Rust)
        content.contains("#[cfg(test)]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // ── is_generated_file ────────────────────────────────────────

    /// Objective: Verify that protobuf-generated files (.pb.go, _grpc.pb.go, .pb.gw.go,
    ///            .pulsar.go, .pb.cc, .pb.h) are correctly identified as generated.
    /// Invariants: All protobuf suffix patterns must be detected regardless of path prefix.
    #[test]
    fn test_is_generated_file_detects_all_protobuf_suffixes() {
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("api.pb.go")),
            "expected .pb.go to be generated"
        );
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("service_grpc.pb.go")),
            "expected _grpc.pb.go to be generated"
        );
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("gateway.pb.gw.go")),
            "expected .pb.gw.go to be generated"
        );
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("topic.pulsar.go")),
            "expected .pulsar.go to be generated"
        );
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("types.pb.cc")),
            "expected .pb.cc to be generated"
        );
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("types.pb.h")),
            "expected .pb.h to be generated"
        );
    }

    /// Objective: Verify that dependency/vendor directories are detected.
    /// Invariants: Paths containing /node_modules/, /vendor/, or /swagger-ui/ are generated,
    ///             regardless of the file extension.
    #[test]
    fn test_is_generated_file_detects_dependency_directories() {
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("/project/node_modules/foo/index.js")),
            "node_modules should be generated"
        );
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("/project/vendor/bar/main.rs")),
            "vendor should be generated"
        );
        assert!(
            CodeAnalyzer::is_generated_file(Path::new("/project/swagger-ui/index.html")),
            "swagger-ui should be generated"
        );
    }

    /// Objective: Verify that user-written source files are NOT marked as generated.
    /// Invariants: Any path that does not match a generated suffix or generated directory
    ///             pattern must return false.
    #[test]
    fn test_is_generated_file_does_not_flag_user_code() {
        assert!(
            !CodeAnalyzer::is_generated_file(Path::new("src/main.rs")),
            "src/main.rs should not be generated"
        );
        assert!(
            !CodeAnalyzer::is_generated_file(Path::new("src/server.go")),
            "src/server.go (Go source) should not be generated"
        );
        assert!(
            !CodeAnalyzer::is_generated_file(Path::new("app.py")),
            "app.py should not be generated"
        );
    }

    /// Objective: Verify that a file ending in .go but not matching any protobuf pattern
    ///            is correctly treated as user code, even in a path containing "vendor"
    ///            as a substring (not the /vendor/ directory).
    /// Invariants: Only exact /vendor/ path component must match, not partial substring.
    #[test]
    fn test_is_generated_file_does_not_false_positive_go_source() {
        assert!(
            !CodeAnalyzer::is_generated_file(Path::new("src/vendor_service.go")),
            "vendor_service.go should not be treated as generated just because 'vendor' appears in the name"
        );
    }

    // ── is_test_file ─────────────────────────────────────────────

    /// Objective: Verify that paths containing /tests/, /test/, /examples/, /benches/,
    ///            /fixtures/, /mocks/, /test-files/ are classified as test files.
    /// Invariants: Path-based heuristics take precedence over content analysis.
    #[test]
    fn test_is_test_file_detects_test_directories() {
        assert!(
            CodeAnalyzer::is_test_file(Path::new("src/tests/helper.rs"), ""),
            "path containing /tests/ should be test"
        );
        assert!(
            CodeAnalyzer::is_test_file(Path::new("examples/hello.rs"), ""),
            "examples/ should be test"
        );
        assert!(
            CodeAnalyzer::is_test_file(Path::new("benches/perf.rs"), ""),
            "benches/ should be test"
        );
        assert!(
            CodeAnalyzer::is_test_file(Path::new("tests/fixtures/data.rs"), ""),
            "fixtures/ should be test"
        );
        assert!(
            CodeAnalyzer::is_test_file(Path::new("tests/mocks/service.rs"), ""),
            "mocks/ should be test"
        );
        assert!(
            CodeAnalyzer::is_test_file(Path::new("test-files/input.txt"), ""),
            "test-files/ should be test"
        );
    }

    /// Objective: Verify that file names with test suffixes (_test.rs, _tests.rs)
    ///            or test prefix (test_) are classified as test files.
    #[test]
    fn test_is_test_file_detects_test_naming_conventions() {
        assert!(
            CodeAnalyzer::is_test_file(Path::new("src/foo_test.rs"), ""),
            "*_test.rs should be test"
        );
        assert!(
            CodeAnalyzer::is_test_file(Path::new("test_main.go"), ""),
            "test_* prefix should be test"
        );
    }

    /// Objective: Verify that #[cfg(test)] in file content is detected even when
    ///            the path does not contain any test indicators.
    /// Invariants: Content analysis is the fallback when path heuristics fail.
    #[test]
    fn test_is_test_file_uses_content_fallback_for_cfg_test() {
        assert!(
            CodeAnalyzer::is_test_file(Path::new("src/foo.rs"), "#[cfg(test)]\nmod tests {}"),
            "#[cfg(test)] in content should mark a file as test"
        );
    }

    /// Objective: Verify that normal source files (no test dir, no test suffix, no #[cfg(test)])
    ///            are NOT classified as test files.
    #[test]
    fn test_is_test_file_does_not_flag_normal_source() {
        assert!(
            !CodeAnalyzer::is_test_file(Path::new("src/main.rs"), "fn main() {}"),
            "src/main.rs without #[cfg(test)] should not be test"
        );
    }

    /// Objective: Verify that leading ./ is stripped before path pattern matching.
    /// Invariants: "./tests/test.rs" normalizes to "tests/test.rs" => matches /tests/.
    #[test]
    fn test_is_test_file_strips_leading_dot_slash() {
        assert!(
            CodeAnalyzer::is_test_file(Path::new("./tests/test.rs"), ""),
            "leading './' should be stripped and path should match /tests/"
        );
    }

    // ── should_exclude ───────────────────────────────────────────

    /// Objective: Verify that default exclude patterns (target, node_modules, .git etc.)
    ///            are applied automatically even without custom patterns.
    /// Invariants: CodeAnalyzer::new with empty custom patterns still excludes common dirs.
    #[test]
    fn test_should_exclude_applies_default_patterns() {
        let analyzer = CodeAnalyzer::new(&[], "en");
        assert!(
            analyzer.should_exclude(Path::new("node_modules/foo")),
            "node_modules should be excluded by default"
        );
        assert!(
            analyzer.should_exclude(Path::new("target/debug/build")),
            "target/ should be excluded by default"
        );
        assert!(
            !analyzer.should_exclude(Path::new("src/main.rs")),
            "src/ should not be excluded"
        );
    }

    /// Objective: Verify that custom exclude patterns are added alongside defaults.
    /// Invariants: Both custom and default patterns are checked.
    #[test]
    fn test_should_exclude_combines_custom_and_default_patterns() {
        let analyzer = CodeAnalyzer::new(&["generated".to_string()], "en");
        assert!(
            analyzer.should_exclude(Path::new("build/generated/code.rs")),
            "custom pattern 'generated' should match"
        );
        assert!(
            analyzer.should_exclude(Path::new("target/release/exe")),
            "default pattern 'target' should still match"
        );
    }

    /// Objective: Verify that a pattern does NOT match unrelated directories.
    /// Invariants: Glob-to-regex conversion creates "build" => "build.*", which should
    ///             match "build/..." but not "src/main.rs".
    #[test]
    fn test_should_exclude_only_matches_intended_directories() {
        let analyzer = CodeAnalyzer::new(&["build".to_string()], "en");
        assert!(
            analyzer.should_exclude(Path::new("build/foo.o")),
            "'build' pattern should match build/ path"
        );
        assert!(
            !analyzer.should_exclude(Path::new("src/main.rs")),
            "'build' pattern should NOT match src/ path"
        );
    }
}
