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
    #[allow(dead_code)]
    pub roast_level: RoastLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Severity {
    Mild,    // Minor issues
    Spicy,   // Medium issues
    Nuclear, // Serious issues
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoastLevel {
    Gentle,    // Gentle roasting
    Sarcastic, // Sarcastic comments
    Savage,    // Brutal honesty
}

pub struct CodeAnalyzer {
    rule_engine: RuleEngine,
    exclude_patterns: Vec<Regex>,
    lang: String,
}

impl CodeAnalyzer {
    pub fn new(exclude_patterns: &[String], lang: &str) -> Self {
        let patterns = exclude_patterns
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

        self.rule_engine
            .check_file(file_path, &syntax_tree, &content, &self.lang)
    }
}
