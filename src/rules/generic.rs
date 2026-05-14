use std::path::Path;

use crate::analyzer::CodeIssue;

/// A language-agnostic code quality rule that works on raw text content.
///
/// Kept as fallback for languages without tree-sitter grammar.
pub trait GenericRule: Send + Sync {
    /// Returns the unique identifier for this rule (e.g. `"c-goto-abuse"`).
    fn name(&self) -> &'static str;

    /// Analyze file content and return detected issues.
    fn check_content(&self, file_path: &Path, content: &str, lang: &str) -> Vec<CodeIssue>;
}

/// Engine that runs all registered generic (text-based) rules against source files.
pub struct GenericRuleEngine {
    rules: Vec<Box<dyn GenericRule>>,
}

impl Default for GenericRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericRuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn check_file(&self, file_path: &Path, content: &str, lang: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for rule in &self.rules {
            issues.extend(rule.check_content(file_path, content, lang));
        }
        issues
    }

    pub fn rule_names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|r| r.name()).collect()
    }
}
