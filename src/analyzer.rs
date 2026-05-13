use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse_file;
use walkdir::WalkDir;

use crate::context::ProjectConfig;
use crate::cross_file::{CrossFileAnalyzer, CrossFileConfig};
use crate::rules::RuleEngine;

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
    rule_engine: RuleEngine,
    exclude_patterns: Vec<Regex>,
    lang: String,
}

impl CodeAnalyzer {
    pub fn rule_names(&self) -> Vec<&'static str> {
        self.rule_engine.rule_names()
    }

    pub fn new(exclude_patterns: &[String], lang: &str) -> Self {
        Self::with_config(exclude_patterns, lang, ProjectConfig::default())
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

        Self {
            rule_engine: RuleEngine::with_config(config),
            exclude_patterns: patterns,
            lang: lang.to_string(),
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
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        return self.analyze_file(path);
                    }
                }
            }
            return Vec::new();
        }

        if !path.is_dir() {
            return Vec::new();
        }

        // Collect all matching file paths
        let files: Vec<PathBuf> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !self.should_exclude(e.path()))
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            .map(|e| e.path().to_path_buf())
            .collect();

        // Phase 1: Parallel single-file analysis
        let mut issues: Vec<CodeIssue> = files
            .par_iter()
            .flat_map(|file_path| self.analyze_file(file_path))
            .collect();

        // Phase 2: Cross-file duplication detection (sequential — accumulates state)
        let mut cross_file = CrossFileAnalyzer::with_config(CrossFileConfig::default());
        for file_path in &files {
            if let Ok(content) = fs::read_to_string(file_path) {
                let _ = cross_file.process_file(file_path, &content);
            }
        }

        let duplicates = cross_file.find_all_duplicates();
        for dup in duplicates {
            let severity = dup.severity.clone();
            for location in &dup.fingerprint.locations {
                issues.push(CodeIssue {
                    file_path: location.file_path.clone(),
                    line: location.line_start,
                    column: 0,
                    rule_name: "cross-file-duplication".to_string(),
                    message: format!(
                        "Duplicated function '{}' found in {} files ({} occurrences)",
                        dup.fingerprint.function_name, dup.file_count, dup.total_occurrences
                    ),
                    severity: severity.clone(),
                });
            }
        }

        issues
    }

    pub fn analyze_file(&self, file_path: &Path) -> Vec<CodeIssue> {
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return vec![],
        };

        let syntax_tree = match parse_file(&content) {
            Ok(tree) => tree,
            Err(_) => return vec![],
        };

        let is_test_file = Self::is_test_file(file_path, &content);

        self.rule_engine
            .check_file(file_path, &syntax_tree, &content, &self.lang, is_test_file)
    }

    fn is_test_file(path: &Path, content: &str) -> bool {
        let path_str = path.to_string_lossy();
        // Normalize: strip leading "./" for consistent matching
        let normalized = path_str.strip_prefix("./").unwrap_or(&path_str);

        // Check file path patterns
        if normalized.contains("/tests/")
            || normalized.contains("\\tests\\")
            || normalized.starts_with("tests/")
            || normalized.starts_with("tests\\")
            || normalized.ends_with("_test.rs")
            || normalized.ends_with("_tests.rs")
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
        // Check for #[cfg(test)] module in content
        content.contains("#[cfg(test)]")
    }
}
