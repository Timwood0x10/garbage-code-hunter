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

/// Breakdown of issues by severity level.
#[derive(Debug, Clone)]
pub struct SeverityDistribution {
    pub nuclear: usize,
    pub spicy: usize,
    pub mild: usize,
}

/// Overall code quality rating derived from the score.
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

/// Calculates severity-weighted, category-based code quality scores.
pub struct CodeScorer;

impl CodeScorer {
    pub fn new() -> Self {
        Self
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
                total_score: 0.0, // Perfect score when no issues (0 = best)
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

    /// Calculate normalized category scores (0-100 for each category)
    fn calculate_normalized_category_scores(
        &self,
        issues: &[CodeIssue],
        total_lines: usize,
    ) -> HashMap<String, f64> {
        let mut category_scores = HashMap::new();
        let mut category_weighted_counts: HashMap<String, f64> = HashMap::new();

        // Universal categories — all rules apply to all languages.
        // Language-specific rules (e.g. unwrap-abuse, box-abuse) are
        // grouped into code-smells: they naturally yield 0 issues in
        // languages where they don't apply, correctly lowering the score.
        let categories = [
            (
                "naming",
                vec![
                    "terrible-naming",
                    "single-letter-variable",
                    "meaningless-naming",
                    "hungarian-notation",
                    "abbreviation-abuse",
                    "c-naming",
                ],
            ),
            (
                "complexity",
                vec![
                    "deep-nesting",
                    "long-function",
                    "god-function",
                    "cyclomatic-complexity",
                    "c-nesting",
                    "c-long-function",
                    "complex-closure",
                ],
            ),
            ("duplication", vec!["code-duplication"]),
            (
                "code-smells",
                vec![
                    "magic-number",
                    "commented-code",
                    "dead-code",
                    "file-too-long",
                    "unwrap-abuse",
                    "unnecessary-clone",
                    "string-abuse",
                    "vec-abuse",
                    "macro-abuse",
                    "channel-abuse",
                    "async-abuse",
                    "dyn-trait-abuse",
                    "unsafe-abuse",
                    "ffi-abuse",
                    "box-abuse",
                    "slice-abuse",
                    "reference-abuse",
                    "module-complexity",
                    "pattern-matching-abuse",
                    "duplicate-imports",
                    "deep-module-nesting",
                    "lifetime-abuse",
                    "trait-complexity",
                    "generic-abuse",
                    "c-include-chaos",
                    "c-magic-number",
                    "c-god-function",
                    "c-commented-code",
                    "c-dead-code",
                    "c-goto-abuse",
                    "c-malloc-leak",
                    "defer-in-loop",
                    "goroutine-abuse",
                    // ruby
                    "global-variable",
                    "bare-rescue",
                    // python
                    "wildcard-import",
                    "bare-except",
                    // java
                    "empty-catch",
                    // typescript
                    "any-type",
                ],
            ),
            (
                "student-code",
                vec!["println-debugging", "panic-abuse", "todo-comment"],
            ),
        ];

        // Severity weights: Nuclear issues count 6x as much as Mild
        let severity_weight = |severity: &Severity| -> f64 {
            match severity {
                Severity::Nuclear => 3.0,
                Severity::Spicy => 1.5,
                Severity::Mild => 0.5,
            }
        };

        // Accumulate severity-weighted counts per category
        for issue in issues {
            let weight = severity_weight(&issue.severity);
            for (category_name, rules) in &categories {
                if rules.contains(&issue.rule_name.as_str()) {
                    *category_weighted_counts
                        .entry(category_name.to_string())
                        .or_insert(0.0) += weight;
                }
            }
        }

        // Calculate normalized scores for each category (0-100)
        for (category_name, _) in &categories {
            let weighted_count = category_weighted_counts.get(*category_name).unwrap_or(&0.0);
            let score = self.calculate_category_score(*weighted_count, total_lines, category_name);
            category_scores.insert(category_name.to_string(), score);
        }

        category_scores
    }

    /// Calculate score for a specific category (0-100, where 0 is perfect, 100 is terrible, maximum 90)
    fn calculate_category_score(
        &self,
        weighted_count: f64,
        total_lines: usize,
        category: &str,
    ) -> f64 {
        if total_lines == 0 {
            return 0.0; // Perfect score when no code
        }

        // Calculate weighted issues per 1000 lines for this category
        let issues_per_1k_lines = (weighted_count / total_lines as f64) * 1000.0;

        // Different thresholds for different categories
        let (excellent_threshold, good_threshold, average_threshold, poor_threshold) =
            match category {
                "naming" => (0.0, 2.0, 5.0, 10.0),
                "complexity" => (0.0, 1.0, 3.0, 6.0),
                "duplication" => (0.0, 0.5, 2.0, 4.0),
                "code-smells" => (0.0, 1.5, 4.0, 8.0),
                "student-code" => (0.0, 1.0, 3.0, 6.0),
                _ => (0.0, 1.0, 3.0, 6.0),
            };

        // Calculate score based on thresholds (0 = excellent, 100 = terrible)

        if issues_per_1k_lines <= excellent_threshold {
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
        }
    }

    /// Calculate weighted final score from category scores.
    /// All categories are universal — no language-specific exclusion needed.
    fn calculate_weighted_final_score(&self, category_scores: &HashMap<String, f64>) -> f64 {
        let weights: Vec<(&str, f64)> = vec![
            ("naming", 0.20),
            ("complexity", 0.20),
            ("duplication", 0.15),
            ("code-smells", 0.30),
            ("student-code", 0.15),
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
            100.0
        }
    }
}

impl Default for CodeScorer {
    fn default() -> Self {
        Self::new()
    }
}
