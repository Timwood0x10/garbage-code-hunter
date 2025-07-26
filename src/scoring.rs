#[allow(dead_code)]
use crate::analyzer::{CodeIssue, Severity};
use std::collections::HashMap;

/// Code quality rating system
/// Score range: 0-100, the higher the score, the worse the code quality
/// 0-20: Excellent
/// 21-40: Good
/// 41-60: Average
/// 61-80: Poor
/// 81-100: Terrible
#[derive(Debug, Clone)]
pub struct CodeQualityScore {
    pub total_score: f64,
    pub category_scores: HashMap<String, f64>,
    pub file_count: usize,
    pub total_lines: usize,
    pub issue_density: f64,
    pub severity_distribution: SeverityDistribution,
    pub quality_level: QualityLevel,
}

#[derive(Debug, Clone)]
pub struct SeverityDistribution {
    pub nuclear: usize,
    pub spicy: usize,
    pub mild: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QualityLevel {
    Excellent, // 0-20
    Good,      // 21-40
    Average,   // 41-60
    Poor,      // 61-80
    Terrible,  // 81-100
}

impl QualityLevel {
    pub fn from_score(score: f64) -> Self {
        match score as u32 {
            0..=20 => QualityLevel::Excellent,
            21..=40 => QualityLevel::Good,
            41..=60 => QualityLevel::Average,
            61..=80 => QualityLevel::Poor,
            _ => QualityLevel::Terrible,
        }
    }

    pub fn description(&self, lang: &str) -> &'static str {
        match (self, lang) {
            (QualityLevel::Excellent, "zh-CN") => "优秀",
            (QualityLevel::Good, "zh-CN") => "良好",
            (QualityLevel::Average, "zh-CN") => "一般",
            (QualityLevel::Poor, "zh-CN") => "较差",
            (QualityLevel::Terrible, "zh-CN") => "糟糕",
            (QualityLevel::Excellent, _) => "Excellent",
            (QualityLevel::Good, _) => "Good",
            (QualityLevel::Average, _) => "Average",
            (QualityLevel::Poor, _) => "Poor",
            (QualityLevel::Terrible, _) => "Terrible",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            QualityLevel::Excellent => "🏆",
            QualityLevel::Good => "👍",
            QualityLevel::Average => "😐",
            QualityLevel::Poor => "😞",
            QualityLevel::Terrible => "💀",
        }
    }
}

#[allow(dead_code)]
pub struct CodeScorer {
    /// rule weights
    pub rule_weights: HashMap<String, f64>,
    /// severity weights
    pub severity_weights: HashMap<Severity, f64>,
}

#[allow(dead_code)]
impl CodeScorer {
    pub fn new() -> Self {
        let mut rule_weights = HashMap::new();

        // Basic code quality issues
        rule_weights.insert("terrible-naming".to_string(), 0.2);
        rule_weights.insert("single-letter-variable".to_string(), 1.5);

        // Complexity issues
        rule_weights.insert("deep-nesting".to_string(), 0.3);
        rule_weights.insert("long-function".to_string(), 2.5);
        rule_weights.insert("cyclomatic-complexity".to_string(), 3.5);
        rule_weights.insert("code-duplication".to_string(), 0.4);

        // Rust specific issues
        rule_weights.insert("unwrap-abuse".to_string(), 0.4); // high weight, because it may cause panic
        rule_weights.insert("unnecessary-clone".to_string(), 0.3);

        // Advanced Rust features abuse
        rule_weights.insert("complex-closure".to_string(), 0.3);
        rule_weights.insert("lifetime-abuse".to_string(), 0.35);
        rule_weights.insert("trait-complexity".to_string(), 0.35);
        rule_weights.insert("generic-abuse".to_string(), 0.35);

        // Rust features abuse
        rule_weights.insert("channel-abuse".to_string(), 0.4);
        rule_weights.insert("async-abuse".to_string(), 0.4);
        rule_weights.insert("dyn-trait-abuse".to_string(), 0.4);
        rule_weights.insert("unsafe-abuse".to_string(), 0.5); // highest weight, because it's a safety issue
        rule_weights.insert("ffi-abuse".to_string(), 0.6); // high weight, because it's a safety issue
        rule_weights.insert("macro-abuse".to_string(), 0.6);
        rule_weights.insert("module-complexity".to_string(), 0.3);
        rule_weights.insert("pattern-matching-abuse".to_string(), 0.3);
        rule_weights.insert("reference-abuse".to_string(), 0.3);
        rule_weights.insert("box-abuse".to_string(), 0.3);
        rule_weights.insert("slice-abuse".to_string(), 0.4);

        let mut severity_weights = HashMap::new();
        severity_weights.insert(Severity::Nuclear, 10.0); // nuclear penalty: first nuclear +20, each subsequent +5
        severity_weights.insert(Severity::Spicy, 5.0); // spicy penalty: first 5 spicy +2, each subsequent +2
        severity_weights.insert(Severity::Mild, 2.0); // mild penalty: first 20 mild +0.5, each subsequent +0.5

        Self {
            rule_weights,
            severity_weights,
        }
    }

    /// calculate code quality score using normalized category-based approach
    pub fn calculate_score(
        &self,
        issues: &[CodeIssue],
        file_count: usize,
        total_lines: usize,
    ) -> CodeQualityScore {
        if issues.is_empty() {
            return CodeQualityScore {
                total_score: 100.0, // Perfect score when no issues
                category_scores: HashMap::new(),
                file_count,
                total_lines,
                issue_density: 0.0,
                severity_distribution: SeverityDistribution {
                    nuclear: 0,
                    spicy: 0,
                    mild: 0,
                },
                quality_level: QualityLevel::Excellent,
            };
        }

        // calculate severity distribution
        let severity_distribution = self.calculate_severity_distribution(issues);

        // calculate category scores (0-100 for each category)
        let category_scores = self.calculate_normalized_category_scores(issues, total_lines);

        // calculate weighted final score
        let total_score = self.calculate_weighted_final_score(&category_scores);

        let issue_density = if total_lines > 0 {
            issues.len() as f64 / total_lines as f64 * 1000.0 // issues per 1000 lines
        } else {
            0.0
        };

        CodeQualityScore {
            total_score,
            category_scores,
            file_count,
            total_lines,
            issue_density,
            severity_distribution,
            quality_level: QualityLevel::from_score(total_score),
        }
    }

    fn calculate_severity_distribution(&self, issues: &[CodeIssue]) -> SeverityDistribution {
        let mut nuclear = 0;
        let mut spicy = 0;
        let mut mild = 0;

        for issue in issues {
            match issue.severity {
                Severity::Nuclear => nuclear += 1,
                Severity::Spicy => spicy += 1,
                Severity::Mild => mild += 1,
            }
        }

        SeverityDistribution {
            nuclear,
            spicy,
            mild,
        }
    }

    fn calculate_base_score(&self, issues: &[CodeIssue]) -> f64 {
        let mut score = 0.0;

        for issue in issues {
            let rule_weight = self.rule_weights.get(&issue.rule_name).unwrap_or(&1.0);
            let severity_weight = self.severity_weights.get(&issue.severity).unwrap_or(&1.0);

            // calculate base score
            score += rule_weight * severity_weight;
        }

        score
    }

    fn calculate_density_penalty(
        &self,
        issue_count: usize,
        file_count: usize,
        total_lines: usize,
    ) -> f64 {
        if total_lines == 0 || file_count == 0 {
            return 0.0;
        }

        // calculate issues density (issues per 1000 lines)
        let issues_per_1000_lines = (issue_count as f64 / total_lines as f64) * 1000.0;

        // calculate average issues per file
        let issues_per_file = issue_count as f64 / file_count as f64;

        // calculate density penalty
        let density_penalty = match issues_per_1000_lines {
            x if x > 50.0 => 25.0, // high density
            x if x > 30.0 => 15.0, // medium density
            x if x > 20.0 => 10.0, // low density
            x if x > 10.0 => 5.0,  // very low density
            _ => 0.0,              // very low density
        };

        // calculate file penalty
        let file_penalty = match issues_per_file {
            x if x > 20.0 => 15.0,
            x if x > 10.0 => 10.0,
            x if x > 5.0 => 5.0,
            _ => 0.0,
        };

        density_penalty + file_penalty
    }

    fn calculate_severity_penalty(&self, distribution: &SeverityDistribution) -> f64 {
        let mut penalty = 0.0;

        // calculate nuclear penalty
        if distribution.nuclear > 0 {
            penalty += 20.0 + (distribution.nuclear as f64 - 1.0) * 5.0; // nuclear penalty: first nuclear +20, each subsequent +5
        }

        // calculate spicy penalty
        if distribution.spicy > 5 {
            penalty += (distribution.spicy as f64 - 5.0) * 2.0; // spicy penalty: first 5 spicy +2, each subsequent +2
        }

        // calculate mild penalty
        if distribution.mild > 20 {
            penalty += (distribution.mild as f64 - 20.0) * 0.5; // mild penalty: first 20 mild +0.5, each subsequent +0.5
        }

        penalty
    }

    fn calculate_category_scores(&self, issues: &[CodeIssue]) -> HashMap<String, f64> {
        let mut category_scores = HashMap::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();

        // define issue categories
        let categories = [
            ("naming", vec!["terrible-naming", "single-letter-variable"]),
            (
                "complexity",
                vec!["deep-nesting", "long-function", "cyclomatic-complexity"],
            ),
            ("duplication", vec!["code-duplication"]),
            ("rust-basics", vec!["unwrap-abuse", "unnecessary-clone"]),
            (
                "advanced-rust",
                vec![
                    "complex-closure",
                    "lifetime-abuse",
                    "trait-complexity",
                    "generic-abuse",
                ],
            ),
            (
                "rust-features",
                vec![
                    "channel-abuse",
                    "async-abuse",
                    "dyn-trait-abuse",
                    "unsafe-abuse",
                    "ffi-abuse",
                    "macro-abuse",
                ],
            ),
            (
                "structure",
                vec![
                    "module-complexity",
                    "pattern-matching-abuse",
                    "reference-abuse",
                    "box-abuse",
                    "slice-abuse",
                ],
            ),
        ];

        // calculate category scores
        for issue in issues {
            for (category_name, rules) in &categories {
                if rules.contains(&issue.rule_name.as_str()) {
                    *category_counts
                        .entry(category_name.to_string())
                        .or_insert(0) += 1;

                    let rule_weight = self.rule_weights.get(&issue.rule_name).unwrap_or(&1.0);
                    let severity_weight =
                        self.severity_weights.get(&issue.severity).unwrap_or(&1.0);

                    *category_scores
                        .entry(category_name.to_string())
                        .or_insert(0.0) += rule_weight * severity_weight;
                }
            }
        }

        category_scores
    }

    /// Calculate normalized category scores (0-100 for each category)
    fn calculate_normalized_category_scores(
        &self,
        issues: &[CodeIssue],
        total_lines: usize,
    ) -> HashMap<String, f64> {
        let mut category_scores = HashMap::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();

        // Define categories with weights and thresholds
        let categories = [
            ("naming", vec!["terrible-naming", "single-letter-variable"]),
            (
                "complexity",
                vec!["deep-nesting", "long-function", "cyclomatic-complexity"],
            ),
            ("duplication", vec!["code-duplication"]),
            ("rust-basics", vec!["unwrap-abuse", "unnecessary-clone"]),
            (
                "advanced-rust",
                vec![
                    "complex-closure",
                    "lifetime-abuse",
                    "trait-complexity",
                    "generic-abuse",
                ],
            ),
            (
                "rust-features",
                vec![
                    "channel-abuse",
                    "async-abuse",
                    "dyn-trait-abuse",
                    "unsafe-abuse",
                    "ffi-abuse",
                    "macro-abuse",
                ],
            ),
            (
                "structure",
                vec![
                    "module-complexity",
                    "pattern-matching-abuse",
                    "reference-abuse",
                    "box-abuse",
                    "slice-abuse",
                ],
            ),
        ];

        // Count issues per category
        for issue in issues {
            for (category_name, rules) in &categories {
                if rules.contains(&issue.rule_name.as_str()) {
                    *category_counts
                        .entry(category_name.to_string())
                        .or_insert(0) += 1;
                }
            }
        }

        // Calculate normalized scores for each category (0-100)
        for (category_name, _) in &categories {
            let count = category_counts.get(*category_name).unwrap_or(&0);
            let score = self.calculate_category_score(*count, total_lines, category_name);
            category_scores.insert(category_name.to_string(), score);
        }

        category_scores
    }

    /// Calculate score for a specific category (0-100, where 0 is perfect, 100 is terrible, maximum 90)
    fn calculate_category_score(
        &self,
        issue_count: usize,
        total_lines: usize,
        category: &str,
    ) -> f64 {
        if total_lines == 0 {
            return 0.0; // Perfect score when no code
        }

        // Calculate issues per 1000 lines for this category
        let issues_per_1k_lines = (issue_count as f64 / total_lines as f64) * 1000.0;

        // Different thresholds for different categories
        let (excellent_threshold, good_threshold, average_threshold, poor_threshold) =
            match category {
                "naming" => (0.0, 2.0, 5.0, 10.0), // Naming should be very clean
                "complexity" => (0.0, 1.0, 3.0, 6.0), // Complexity should be low
                "duplication" => (0.0, 0.5, 2.0, 4.0), // Duplication should be minimal
                "rust-basics" => (0.0, 1.0, 3.0, 6.0), // Basic Rust issues
                "advanced-rust" => (0.0, 0.5, 2.0, 4.0), // Advanced features should be used carefully
                "rust-features" => (0.0, 0.5, 1.5, 3.0), // Special features should be rare
                "structure" => (0.0, 1.0, 3.0, 6.0),     // Structure issues
                _ => (0.0, 1.0, 3.0, 6.0),               // Default thresholds
            };

        // Calculate score based on thresholds (0 = excellent, 100 = terrible)
        let score = if issues_per_1k_lines <= excellent_threshold {
            0.0 // Perfect score
        } else if issues_per_1k_lines <= good_threshold {
            (issues_per_1k_lines - excellent_threshold) / (good_threshold - excellent_threshold)
                * 20.0
        } else if issues_per_1k_lines <= average_threshold {
            20.0 + (issues_per_1k_lines - good_threshold) / (average_threshold - good_threshold)
                * 20.0
        } else if issues_per_1k_lines <= poor_threshold {
            40.0 + (issues_per_1k_lines - average_threshold) / (poor_threshold - average_threshold)
                * 20.0
        } else {
            // Beyond poor threshold, score increases rapidly but caps at 90
            let excess = issues_per_1k_lines - poor_threshold;
            (60.0 + excess * 2.0).min(90.0) // Cap at 90 to avoid perfect 100
        };

        score
    }

    /// Calculate weighted final score from category scores
    fn calculate_weighted_final_score(&self, category_scores: &HashMap<String, f64>) -> f64 {
        // Category weights (should sum to 1.0)
        let weights = [
            ("naming", 0.25),        // 25% - Very important
            ("complexity", 0.20),    // 20% - Very important
            ("duplication", 0.15),   // 15% - Important
            ("rust-basics", 0.15),   // 15% - Important
            ("advanced-rust", 0.10), // 10% - Moderate
            ("rust-features", 0.10), // 10% - Moderate
            ("structure", 0.05),     // 5% - Less critical
        ];

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (category, weight) in &weights {
            if let Some(score) = category_scores.get(*category) {
                weighted_sum += score * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            100.0 // Default to perfect score if no categories found
        }
    }
}

impl Default for CodeScorer {
    fn default() -> Self {
        Self::new()
    }
}
