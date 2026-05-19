use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::context::{FileContext, ProjectConfig};
use crate::finding::StyleFinding;
use crate::language::adapter::adapter_for;
use crate::language::{Language, SUPPORTED_EXTENSIONS};
use crate::rules::generic::GenericRuleEngine;
use crate::signals::{aggregate_detector_scores, SignalDetector, StyleSignal};
use crate::style_ir::{StyleIr, StyleIrSummary};
use crate::treesitter::duplication::{CrossFileDupDetector, IntraFileDupDetector};
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::{TreeSitterEngine, TreeSitterRuleEngine};

pub struct StyleIrFileInfo {
    pub file_path: String,
    pub summary: StyleIrSummary,
}

pub struct FullAnalysisResult {
    pub findings: Vec<StyleFinding>,
    pub file_count: usize,
    pub total_lines: usize,
    pub style_ir_files: Vec<StyleIrFileInfo>,
}

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
    cross_detector: RefCell<CrossFileDupDetector>,
    detectors: Vec<Box<dyn SignalDetector>>,
    direct_scores: RefCell<HashMap<StyleSignal, f64>>,
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
            ".venv",
            "venv",
            "vendor",
        ];

        let mut all_patterns: Vec<String> =
            default_excludes.iter().map(|s| s.to_string()).collect();
        all_patterns.extend(exclude_patterns.iter().cloned());

        // Also add exclude patterns from project config
        all_patterns.extend(config.whitelists.exclude_patterns.clone());

        let patterns = all_patterns
            .iter()
            .filter_map(|pattern| {
                // Convert glob patterns to regular expressions with path-boundary anchoring.
                // Without anchors, "build" would match "mybuild/foo.o" — a substring false positive.
                let glob_pattern = pattern
                    .replace(".", r"\.")
                    .replace("*", ".*")
                    .replace("?", ".");
                let regex_pattern = format!(r"(?:^|/){}(?:/|$)", glob_pattern);
                Regex::new(&regex_pattern).ok()
            })
            .collect();

        let mut ts_rule_engine = TreeSitterRuleEngine::new();
        crate::treesitter::rules::common_rules::register_common_rules(&mut ts_rule_engine);
        crate::treesitter::rules::rust_rules::register_rust_rules(&mut ts_rule_engine);

        Self {
            generic_engine: GenericRuleEngine::new(),
            ts_engine: TreeSitterEngine::new(),
            ts_rule_engine,
            exclude_patterns: patterns,
            project_config: config,
            lang: lang.to_string(),
            cross_detector: RefCell::new(CrossFileDupDetector::new()),
            detectors: Vec::new(),
            direct_scores: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_detectors(mut self, detectors: Vec<Box<dyn SignalDetector>>) -> Self {
        self.detectors = detectors;
        self
    }

    pub fn direct_signal_scores(&self) -> HashMap<StyleSignal, f64> {
        self.direct_scores.borrow().clone()
    }

    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns
            .iter()
            .any(|pattern| pattern.is_match(&path_str))
    }

    /// Collect source files from a path (file or directory). Excludes
    /// unsupported extensions and should_exclude paths. Includes generated files.
    fn collect_source_files(&self, path: &Path) -> Vec<PathBuf> {
        if path.is_file() {
            if !self.should_exclude(path) {
                let lang = Language::from_path(path);
                if lang != Language::Unknown {
                    return vec![path.to_path_buf()];
                }
            }
            return Vec::new();
        }
        if !path.is_dir() {
            return Vec::new();
        }
        WalkDir::new(path)
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
            .collect()
    }

    /// Compatibility wrapper — runs the full pipeline and converts back to `CodeIssue`s.
    pub fn analyze_path(&self, path: &Path) -> Vec<CodeIssue> {
        self.analyze_to_findings(path)
            .into_iter()
            .map(|f| f.to_code_issue())
            .collect()
    }

    /// Full analysis pipeline returning `StyleFinding`s.
    ///
    /// - Phase 1: Tree-sitter rule analysis per file (caches `ParsedFile` for Phase 4)
    /// - Phase 2: Cross-file duplication detection
    /// - Phase 3: Intra-file duplication detection
    /// - Phase 4: Direct signal detection (scores + findings)
    ///
    /// Also populates `self.direct_scores` for downstream consumers.
    pub fn analyze_to_findings(&self, path: &Path) -> Vec<StyleFinding> {
        let files = self.collect_source_files(path);
        if files.is_empty() {
            return Vec::new();
        }

        // Phase 1: Rule analysis + cache parsed files for Phase 4
        let mut issues: Vec<CodeIssue> = Vec::new();
        let mut parsed_files: Vec<(ParsedFile, PathBuf, bool)> = Vec::new();

        for file_path in &files {
            if Self::is_generated_file(file_path) {
                continue;
            }
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let lang = Language::from_path(file_path);
            if lang == Language::Unknown {
                continue;
            }
            let is_test_file = Self::is_test_file(file_path, &content);

            // Parse once — use for both rule analysis and Phase 4
            if let Some(parsed) = self.ts_engine.parse_file(file_path, &content) {
                // AST-level test detection (e.g., #[test] in Rust)
                let ast_test = adapter_for(lang)
                    .map(|a| a.has_test_nodes(&parsed))
                    .unwrap_or(false);
                let effective_test = is_test_file || ast_test;

                let context = FileContext::from_path(file_path);
                issues.extend(self.ts_rule_engine.check_file_with_context(
                    &parsed,
                    effective_test,
                    &context,
                    &self.project_config,
                ));
                parsed_files.push((parsed, file_path.clone(), effective_test));
            } else if lang == Language::C || lang == Language::Cpp {
                issues.extend(
                    self.generic_engine
                        .check_file(file_path, &content, &self.lang),
                );
            }
        }

        // Phase 2: Cross-file duplication detection (reuse Phase 1 parsed files)
        *self.cross_detector.borrow_mut() = CrossFileDupDetector::new();
        for (parsed, _, _) in &parsed_files {
            self.cross_detector.borrow_mut().process_file(parsed);
        }
        issues.extend(self.cross_detector.borrow().find_duplicates());
        issues.extend(self.cross_detector.borrow().find_near_duplicates());

        // Phase 3: Intra-file code duplication (reuse Phase 1 parsed files)
        for (parsed, _, _) in &parsed_files {
            issues.extend(IntraFileDupDetector::check(parsed));
        }
        // Convert rule issues to findings
        let mut findings: Vec<StyleFinding> = issues.iter().map(From::from).collect();

        // Phase 4: Direct signal detection (scores + findings)
        if !self.detectors.is_empty() && !parsed_files.is_empty() {
            let parsed_for_scores: Vec<ParsedFile> =
                parsed_files.iter().map(|(p, _, _)| p.clone()).collect();
            let test_flags: Vec<bool> = parsed_files
                .iter()
                .map(|(_, _, is_test)| *is_test)
                .collect();
            let skip_tests_config = self.project_config.signals.skip_tests;
            *self.direct_scores.borrow_mut() = aggregate_detector_scores(
                &self.detectors,
                &parsed_for_scores,
                &test_flags,
                skip_tests_config,
            );

            for (parsed, file_path, is_test_file) in &parsed_files {
                let lang = parsed.language;
                let ir = StyleIr::from_parsed(parsed);
                for detector in &self.detectors {
                    if !detector.supported_languages().contains(&lang) {
                        continue;
                    }
                    let findings_iter = if let Some(ref ir) = ir {
                        detector.detect_findings_with_ir(
                            ir,
                            parsed,
                            *is_test_file,
                            skip_tests_config,
                        )
                    } else {
                        detector.detect_findings(parsed, *is_test_file, skip_tests_config)
                    };
                    for (signal, count) in findings_iter {
                        findings.push(StyleFinding::for_signal(signal, count, file_path.clone()));
                    }
                }
            }
        }

        findings
    }

    pub fn analyze_full(&self, path: &Path) -> FullAnalysisResult {
        let files = self.collect_source_files(path);
        if files.is_empty() {
            return FullAnalysisResult {
                findings: Vec::new(),
                file_count: 0,
                total_lines: 0,
                style_ir_files: Vec::new(),
            };
        }

        let mut issues: Vec<CodeIssue> = Vec::new();
        let mut parsed_files: Vec<(ParsedFile, PathBuf, bool)> = Vec::new();
        let mut style_ir_files: Vec<StyleIrFileInfo> = Vec::new();
        let mut file_count: usize = 0;
        let mut total_lines: usize = 0;

        for file_path in &files {
            if Self::is_generated_file(file_path) {
                continue;
            }
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let lang = Language::from_path(file_path);
            if lang == Language::Unknown {
                continue;
            }
            file_count += 1;
            total_lines += content.lines().count();
            let is_test_file = Self::is_test_file(file_path, &content);

            if let Some(parsed) = self.ts_engine.parse_file(file_path, &content) {
                let ast_test = adapter_for(lang)
                    .map(|a| a.has_test_nodes(&parsed))
                    .unwrap_or(false);
                let effective_test = is_test_file || ast_test;

                let context = FileContext::from_path(file_path);
                issues.extend(self.ts_rule_engine.check_file_with_context(
                    &parsed,
                    effective_test,
                    &context,
                    &self.project_config,
                ));
                if let Some(ir) = StyleIr::from_parsed(&parsed) {
                    style_ir_files.push(StyleIrFileInfo {
                        file_path: file_path.to_string_lossy().to_string(),
                        summary: ir.summary(),
                    });
                }
                parsed_files.push((parsed, file_path.clone(), effective_test));
            } else if lang == Language::C || lang == Language::Cpp {
                issues.extend(
                    self.generic_engine
                        .check_file(file_path, &content, &self.lang),
                );
            }
        }

        *self.cross_detector.borrow_mut() = CrossFileDupDetector::new();
        for (parsed, _, _) in &parsed_files {
            self.cross_detector.borrow_mut().process_file(parsed);
        }
        issues.extend(self.cross_detector.borrow().find_duplicates());
        issues.extend(self.cross_detector.borrow().find_near_duplicates());

        for (parsed, _, _) in &parsed_files {
            issues.extend(IntraFileDupDetector::check(parsed));
        }

        let mut findings: Vec<StyleFinding> = issues.iter().map(From::from).collect();

        if !self.detectors.is_empty() && !parsed_files.is_empty() {
            let parsed_for_scores: Vec<ParsedFile> =
                parsed_files.iter().map(|(p, _, _)| p.clone()).collect();
            let test_flags: Vec<bool> = parsed_files
                .iter()
                .map(|(_, _, is_test)| *is_test)
                .collect();
            let skip_tests_config = self.project_config.signals.skip_tests;
            *self.direct_scores.borrow_mut() = aggregate_detector_scores(
                &self.detectors,
                &parsed_for_scores,
                &test_flags,
                skip_tests_config,
            );

            for (parsed, file_path, is_test_file) in &parsed_files {
                let lang = parsed.language;
                let ir = StyleIr::from_parsed(parsed);
                for detector in &self.detectors {
                    if !detector.supported_languages().contains(&lang) {
                        continue;
                    }
                    let findings_iter = if let Some(ref ir) = ir {
                        detector.detect_findings_with_ir(
                            ir,
                            parsed,
                            *is_test_file,
                            skip_tests_config,
                        )
                    } else {
                        detector.detect_findings(parsed, *is_test_file, skip_tests_config)
                    };
                    for (signal, count) in findings_iter {
                        findings.push(StyleFinding::for_signal(signal, count, file_path.clone()));
                    }
                }
            }
        }

        FullAnalysisResult {
            findings,
            file_count,
            total_lines,
            style_ir_files,
        }
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
        // Generated files from code generators
            || name.contains(".gen.")
            || name.contains(".generated.")
        // Minified / bundled JavaScript
            || name.ends_with(".min.js")
            || name.ends_with(".bundle.js")
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

    // ── analyze_to_findings ───────────────────────────────────────

    /// Objective: Verify that `analyze_to_findings()` produces both rule-based
    /// findings (from CodeIssue conversion) AND direct detector signal findings.
    /// Invariants: With a file containing panics + naming issues, the output must
    /// include at least some PanicAddiction findings and some findings with a signal
    /// other than Duplication (the default when no signal is recognized).
    #[test]
    fn test_analyze_to_findings_includes_detector_findings() {
        use crate::detectors::PanicAddictionDetector;
        use std::io::Write;

        let dir = tempfile::tempdir().expect("tempdir");
        let file_path = dir.path().join("code.rs");
        let mut f = std::fs::File::create(&file_path).expect("create temp file");
        write!(
            f,
            "fn main() {{
    let _ = foo.unwrap();
    let _ = bar.expect(\"msg\");
    panic!(\"boom\");
    let x = 1;
}}
"
        )
        .expect("write");

        let analyzer = CodeAnalyzer::new(&[], "en")
            .with_detectors(vec![
                Box::new(PanicAddictionDetector::new()) as Box<dyn SignalDetector>
            ]);

        let findings = analyzer.analyze_to_findings(dir.path());

        // Must have at least one finding with PanicAddiction signal
        let panic_signal_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.signal == StyleSignal::PanicAddiction)
            .collect();
        assert!(
            !panic_signal_findings.is_empty(),
            "expected at least one PanicAddiction finding from detector, got {} total findings",
            findings.len()
        );

        // Verify at least 1 finding exists from the detector
        assert!(
            !findings.is_empty(),
            "expected at least 1 total finding, got {}",
            findings.len()
        );
    }
}
