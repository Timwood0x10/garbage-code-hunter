use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse_file;
use walkdir::WalkDir;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

        let mut all_patterns: Vec<String> = default_excludes
            .iter()
            .map(|s| s.to_string())
            .collect();
        all_patterns.extend(exclude_patterns.iter().cloned());

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
            rule_engine: RuleEngine::new(),
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
        let mut issues = Vec::new();

        if path.is_file() {
            if !self.should_exclude(path) {
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        issues.extend(self.analyze_file(path));
                    }
                }
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| !self.should_exclude(e.path()))
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                issues.extend(self.analyze_file(entry.path()));
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
