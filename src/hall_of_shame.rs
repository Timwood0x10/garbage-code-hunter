use crate::analyzer::{CodeIssue, Severity};
/// Hall of Shame - tracks and ranks the worst code patterns and files
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ShameEntry {
    pub file_path: PathBuf,
    pub total_issues: usize,
    pub shame_score: f64,
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

        // Analyze issues for this file
        for issue in issues {
            match issue.severity {
                Severity::Nuclear => nuclear_count += 1,
                Severity::Spicy => spicy_count += 1,
                Severity::Mild => mild_count += 1,
            }

            // Track pattern statistics
            self.update_pattern_stats(&issue.rule_name, &issue.severity, &file_path);
        }

        // Calculate shame score (weighted by severity)
        let shame_score =
            (nuclear_count as f64 * 10.0) + (spicy_count as f64 * 3.0) + (mild_count as f64 * 1.0);

        let entry = ShameEntry {
            file_path,
            total_issues: issues.len(),
            shame_score,
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

        // Calculate garbage density (issues per 1000 lines)
        let total_issues: usize = self.entries.iter().map(|e| e.total_issues).sum();
        let garbage_density = if self.total_lines > 0 {
            (total_issues as f64 / self.total_lines as f64) * 1000.0
        } else {
            0.0
        };

        ProjectShameStats {
            total_files_analyzed: self.entries.len(),
            total_issues,
            garbage_density,
            hall_of_shame,
        }
    }
}

impl Default for HallOfShame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Severity;
    use std::path::PathBuf;

    fn make_issue(rule: &str, sev: Severity) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            rule_name: rule.to_string(),
            message: String::new(),
            severity: sev,
        }
    }

    /// Objective: Verify that adding a file with zero issues does NOT create a ShameEntry.
    /// Invariants: entries count should stay 0; total_lines still accumulates.
    #[test]
    fn test_empty_issues_produces_no_entry() {
        let mut h = HallOfShame::new();
        h.add_file_analysis(PathBuf::from("foo.rs"), &[], 100);
        assert!(
            h.entries.is_empty(),
            "no entry should be added when issues is empty, got {} entries",
            h.entries.len()
        );
        assert_eq!(h.total_lines, 100, "total_lines should still accumulate");
    }

    /// Objective: Verify that shame score weights are correctly applied:
    ///            Nuclear=10, Spicy=3, Mild=1 per issue.
    /// Invariants: Score is strictly sum(weight * count_for_severity).
    #[test]
    fn test_shame_score_weights_per_severity() {
        let mut h = HallOfShame::new();
        let issues = vec![
            make_issue("nuc", Severity::Nuclear),
            make_issue("spi", Severity::Spicy),
            make_issue("mid", Severity::Mild),
            make_issue("nuc2", Severity::Nuclear),
        ];
        let file_path = PathBuf::from("bad.rs");
        h.add_file_analysis(file_path, &issues, 100);
        let score = h.entries[0].shame_score;
        assert_eq!(score, 24.0, "expected 10*2 + 3 + 1 = 24, got {score}");
    }

    /// Objective: Verify that multiple files with different line counts
    ///            correctly accumulate total_lines.
    #[test]
    fn test_multiple_files_accumulate_lines() {
        let mut h = HallOfShame::new();
        h.add_file_analysis(
            PathBuf::from("a.rs"),
            &[make_issue("x", Severity::Nuclear)],
            30,
        );
        h.add_file_analysis(
            PathBuf::from("b.rs"),
            &[make_issue("y", Severity::Mild)],
            70,
        );
        assert_eq!(h.total_lines, 100, "30 + 70 should = 100");
    }

    /// Objective: Verify pattern_stats tracks per-rule count accurately
    ///            when the same rule fires in multiple files.
    #[test]
    fn test_pattern_stats_tracks_rule_count_across_files() {
        let mut h = HallOfShame::new();
        h.add_file_analysis(
            PathBuf::from("a.rs"),
            &[make_issue("unwrap-abuse", Severity::Nuclear)],
            10,
        );
        h.add_file_analysis(
            PathBuf::from("b.rs"),
            &[make_issue("unwrap-abuse", Severity::Nuclear)],
            20,
        );
        let stats = h
            .pattern_stats
            .get("unwrap-abuse")
            .expect("unwrap-abuse should have been tracked");
        assert_eq!(
            stats.count, 2,
            "same rule in 2 files should count 2, got {}",
            stats.count
        );
    }

    /// Objective: Verify that severity distribution within a pattern is correct.
    /// Invariants: The count for each severity must match the exact number of issues at that severity.
    #[test]
    fn test_pattern_stats_tracks_severity_distribution() {
        let mut h = HallOfShame::new();
        let issues = vec![
            make_issue("x", Severity::Nuclear),
            make_issue("x", Severity::Nuclear),
            make_issue("x", Severity::Mild),
        ];
        h.add_file_analysis(PathBuf::from("bad.rs"), &issues, 100);
        let stats = h.pattern_stats.get("x").expect("rule 'x' should exist");
        assert_eq!(
            stats.severity_distribution.get(&Severity::Nuclear),
            Some(&2),
            "expected 2 nuclear issues"
        );
        assert_eq!(
            stats.severity_distribution.get(&Severity::Mild),
            Some(&1),
            "expected 1 mild issue"
        );
        assert_eq!(
            stats.severity_distribution.get(&Severity::Spicy),
            None,
            "expected 0 spicy issues"
        );
    }

    /// Objective: Verify example_files cap at 5 unique files per pattern.
    /// Invariants: After 10 different files with the same rule, only 5 examples are stored.
    #[test]
    fn test_pattern_stats_example_files_capped_at_five() {
        let mut h = HallOfShame::new();
        let issue = make_issue("dup", Severity::Nuclear);
        for i in 0..10 {
            let path = PathBuf::from(format!("file_{i}.rs"));
            h.add_file_analysis(path, std::slice::from_ref(&issue), 10);
        }
        let stats = h.pattern_stats.get("dup").expect("rule 'dup' should exist");
        assert_eq!(
            stats.example_files.len(),
            5,
            "max example files should be 5, got {}",
            stats.example_files.len()
        );
    }

    /// Objective: Verify generate_shame_report sorts entries by score descending.
    /// Invariants: The highest-scoring file is first in the list.
    #[test]
    fn test_report_sorted_by_score_descending() {
        let mut h = HallOfShame::new();
        h.add_file_analysis(
            PathBuf::from("low.rs"),
            &[make_issue("x", Severity::Mild)],
            10,
        );
        h.add_file_analysis(
            PathBuf::from("high.rs"),
            &[make_issue("x", Severity::Nuclear)],
            10,
        );
        let report = h.generate_shame_report();
        assert_eq!(
            report.hall_of_shame[0].shame_score, 10.0,
            "highest score (10) should be first"
        );
        assert_eq!(
            report.hall_of_shame[1].shame_score, 1.0,
            "lowest score (1) should be second"
        );
    }

    /// Objective: Verify the report caps at 10 entries even when more files exist.
    #[test]
    fn test_report_limited_to_ten_entries() {
        let mut h = HallOfShame::new();
        for i in 0..20 {
            let f = format!("f{i}.rs");
            h.add_file_analysis(PathBuf::from(f), &[make_issue("x", Severity::Nuclear)], 10);
        }
        let report = h.generate_shame_report();
        assert_eq!(
            report.hall_of_shame.len(),
            10,
            "should contain at most 10 entries, got {}",
            report.hall_of_shame.len()
        );
    }

    /// Objective: Verify garbage_density = (total_issues / total_lines) * 1000.
    /// Invariants: Density scales linearly with issues and inversely with lines.
    #[test]
    fn test_garbage_density_formula_correct() {
        let mut h = HallOfShame::new();
        h.add_file_analysis(
            PathBuf::from("a.rs"),
            &[make_issue("x", Severity::Nuclear)],
            500,
        );
        h.add_file_analysis(
            PathBuf::from("b.rs"),
            &[make_issue("y", Severity::Mild)],
            500,
        );
        let report = h.generate_shame_report();
        assert!(
            (report.garbage_density - 2.0).abs() < 1e-6,
            "2 issues / 1000 lines = 2.0 per 1k, got {}",
            report.garbage_density
        );
    }

    /// Objective: Verify zero total_lines does not cause NaN or crash.
    /// Invariants: garbage_density is 0.0 when total_lines is 0.
    #[test]
    fn test_zero_total_lines_does_not_crash() {
        let h = HallOfShame::new();
        let report = h.generate_shame_report();
        assert_eq!(
            report.garbage_density, 0.0,
            "density should be 0 when no files added, got {}",
            report.garbage_density
        );
    }

    /// Objective: Verify that the same file added multiple times creates
    ///            multiple ShameEntries (the system does NOT deduplicate).
    #[test]
    fn test_duplicate_file_path_creates_multiple_entries() {
        let mut h = HallOfShame::new();
        let fp = PathBuf::from("same.rs");
        h.add_file_analysis(fp.clone(), &[make_issue("a", Severity::Mild)], 10);
        h.add_file_analysis(fp, &[make_issue("b", Severity::Mild)], 10);
        assert_eq!(
            h.entries.len(),
            2,
            "same path added twice should create 2 entries, got {}",
            h.entries.len()
        );
    }

    /// Objective: Verify that pattern_stats with mixed severity distributions
    ///            correctly counts each severity bucket independently.
    #[test]
    fn test_pattern_stats_mixed_severities() {
        let mut h = HallOfShame::new();
        let issues = vec![
            make_issue("mix", Severity::Nuclear),
            make_issue("mix", Severity::Spicy),
            make_issue("mix", Severity::Mild),
            make_issue("mix", Severity::Nuclear),
            make_issue("mix", Severity::Spicy),
        ];
        h.add_file_analysis(PathBuf::from("mix.rs"), &issues, 50);
        let stats = h.pattern_stats.get("mix").expect("rule 'mix' should exist");
        assert_eq!(
            stats.count, 5,
            "total 5 issues for 'mix', got {}",
            stats.count
        );
        assert_eq!(
            stats.severity_distribution.get(&Severity::Nuclear),
            Some(&2),
            "expected 2 nuclear"
        );
        assert_eq!(
            stats.severity_distribution.get(&Severity::Spicy),
            Some(&2),
            "expected 2 spicy"
        );
        assert_eq!(
            stats.severity_distribution.get(&Severity::Mild),
            Some(&1),
            "expected 1 mild"
        );
    }
}
