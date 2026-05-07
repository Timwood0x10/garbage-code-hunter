use crate::analyzer::{CodeIssue, Severity};
/// Hall of Shame - tracks and ranks the worst code patterns and files
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ShameEntry {
    pub file_path: PathBuf,
    pub total_issues: usize,
    pub shame_score: f64,
    pub _worst_offenses: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PatternStats {
    pub rule_name: String,
    pub count: usize,
    pub severity_distribution: HashMap<Severity, usize>,
    pub example_files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ProjectShameStats {
    pub total_files_analyzed: usize,
    pub total_issues: usize,
    pub garbage_density: f64,           // issues per 1000 lines of code
    pub hall_of_shame: Vec<ShameEntry>, // worst files
    pub _shame_categories: HashMap<String, usize>,
}

pub struct HallOfShame {
    entries: Vec<ShameEntry>,
    pattern_stats: HashMap<String, PatternStats>,
    total_lines: usize,
}

impl HallOfShame {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            pattern_stats: HashMap::new(),
            total_lines: 0,
        }
    }

    pub fn add_file_analysis(
        &mut self,
        file_path: PathBuf,
        issues: &[CodeIssue],
        file_lines: usize,
    ) {
        self.total_lines += file_lines;

        if issues.is_empty() {
            return;
        }

        let mut nuclear_count = 0;
        let mut spicy_count = 0;
        let mut mild_count = 0;
        let mut worst_offenses = Vec::new();

        // Analyze issues for this file
        for issue in issues {
            match issue.severity {
                Severity::Nuclear => nuclear_count += 1,
                Severity::Spicy => spicy_count += 1,
                Severity::Mild => mild_count += 1,
            }

            // Track pattern statistics
            self.update_pattern_stats(&issue.rule_name, &issue.severity, &file_path);

            // Collect worst offenses (Nuclear and Spicy issues)
            if matches!(issue.severity, Severity::Nuclear | Severity::Spicy) {
                worst_offenses.push(format!("{}: {}", issue.rule_name, issue.message));
            }
        }

        // Calculate shame score (weighted by severity)
        let shame_score =
            (nuclear_count as f64 * 10.0) + (spicy_count as f64 * 3.0) + (mild_count as f64 * 1.0);

        let entry = ShameEntry {
            file_path,
            total_issues: issues.len(),
            shame_score,
            _worst_offenses: worst_offenses,
        };

        self.entries.push(entry);
    }

    fn update_pattern_stats(&mut self, rule_name: &str, severity: &Severity, file_path: &PathBuf) {
        let stats = self
            .pattern_stats
            .entry(rule_name.to_string())
            .or_insert_with(|| PatternStats {
                rule_name: rule_name.to_string(),
                count: 0,
                severity_distribution: HashMap::new(),
                example_files: Vec::new(),
            });

        stats.count += 1;
        *stats
            .severity_distribution
            .entry(severity.clone())
            .or_insert(0) += 1;

        // Add file to examples if not already present and we have less than 5 examples
        if stats.example_files.len() < 5 && !stats.example_files.contains(file_path) {
            stats.example_files.push(file_path.clone());
        }
    }

    pub fn generate_shame_report(&self) -> ProjectShameStats {
        let mut sorted_entries = self.entries.clone();
        sorted_entries.sort_by(|a, b| b.shame_score.partial_cmp(&a.shame_score).unwrap());

        // Take top 10 worst files
        let hall_of_shame = sorted_entries.into_iter().take(10).collect();

        // Sort patterns by frequency
        let mut most_common_patterns: Vec<PatternStats> =
            self.pattern_stats.values().cloned().collect();
        most_common_patterns.sort_by_key(|b| std::cmp::Reverse(b.count));

        // Calculate garbage density (issues per 1000 lines)
        let total_issues: usize = self.entries.iter().map(|e| e.total_issues).sum();
        let garbage_density = if self.total_lines > 0 {
            (total_issues as f64 / self.total_lines as f64) * 1000.0
        } else {
            0.0
        };

        // Categorize shame by rule types
        let mut shame_categories = HashMap::new();
        for pattern in &most_common_patterns {
            let category = self.categorize_rule(&pattern.rule_name);
            *shame_categories.entry(category).or_insert(0) += pattern.count;
        }

        ProjectShameStats {
            total_files_analyzed: self.entries.len(),
            total_issues,
            garbage_density,
            hall_of_shame,
            _shame_categories: shame_categories,
        }
    }

    fn categorize_rule(&self, rule_name: &str) -> String {
        match rule_name {
            name if name.contains("naming") => "Naming Issues".to_string(),
            name if name.contains("complexity")
                || name.contains("nesting")
                || name.contains("function") =>
            {
                "Complexity Issues".to_string()
            }
            name if name.contains("unwrap")
                || name.contains("panic")
                || name.contains("string")
                || name.contains("clone") =>
            {
                "Rust-specific Issues".to_string()
            }
            name if name.contains("println") || name.contains("todo") => {
                "Student Code Issues".to_string()
            }
            name if name.contains("import") || name.contains("file") || name.contains("module") => {
                "Structure Issues".to_string()
            }
            name if name.contains("magic") || name.contains("dead") || name.contains("comment") => {
                "Code Smells".to_string()
            }
            _ => "Other Issues".to_string(),
        }
    }
}

impl Default for HallOfShame {
    fn default() -> Self {
        Self::new()
    }
}
