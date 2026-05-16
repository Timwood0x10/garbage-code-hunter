use crate::analyzer::{CodeIssue, Severity};
use std::collections::HashMap;

/// Code quality rating system — two-tier log model.
/// Score starts at 0 (best). Higher score = worse code quality.
/// 0-20: Excellent  |  21-40: Good  |  41-60: Average  |  61-80: Poor  |  81+: Terrible
///
/// Tier 1: Nuclear issues (high confidence) → log-scaled absolute count, cap 40
/// Tier 2: Spicy + Mild issues (noisy) → log-scaled density per 1k lines, cap 60
#[derive(Debug, Clone)]
pub struct CodeQualityScore {
    pub total_score: f64,
    pub n_score: f64,
    pub d_score: f64,
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
    Terrible,  // 81+
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

pub struct CodeScorer;

impl CodeScorer {
    pub fn new() -> Self {
        Self
    }

    /// Accumulation model: start at 0, each issue adds points.
    pub fn calculate_score(
        &self,
        issues: &[CodeIssue],
        file_count: usize,
        total_lines: usize,
    ) -> CodeQualityScore {
        if issues.is_empty() {
            return CodeQualityScore {
                total_score: 0.0,
                n_score: 0.0,
                d_score: 0.0,
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

        let severity_distribution = self.calculate_severity_distribution(issues);

        // Category breakdown: log-scaled density per category (informational only)
        let categories = self.build_categories();
        let k_lines = total_lines as f64 / 1000.0;
        let mut category_scores = HashMap::new();
        for (cat_name, rules) in &categories {
            let cat_count = issues
                .iter()
                .filter(|i| rules.contains(&i.rule_name.as_str()))
                .count();
            let cat_density = if k_lines > 0.0 {
                cat_count as f64 / k_lines
            } else {
                0.0
            };
            let cat_score = ((cat_density + 1.0).log2() * 6.0).min(20.0);
            category_scores.insert(cat_name.to_string(), cat_score);
        }

        // Two-tier log scoring (0-100)
        //
        // Tier 1: Nuclear — absolute count, log-scaled.
        //   Nuclear issues are high-confidence (deep nesting, god function, bare except).
        //   Even 1 Nuclear is meaningful. Log prevents large counts from dominating.
        //   log2(1 + n) * 8: 0→0, 1→8, 2→12.7, 5→20.7, 10→27.7, 30→39.6
        //   Cap at 40.
        //
        // Tier 2: Noisy density — Spicy + Mild combined, density-normalized, log-scaled.
        //   Non-Nuclear issues are noisy (magic-number, naming, println are often FPs).
        //   Must use density (per 1k lines) to be fair across project sizes.
        //   Spicy counts 1.5x vs Mild 1x (slightly more reliable, but still noisy).
        //   log2(1 + d) * 6: d=0→0, d=1→6, d=7→18, d=31→30, d=127→42
        //   Cap at 60.
        let n_score = (severity_distribution.nuclear as f64 + 1.0).log2() * 8.0;
        let n_score = n_score.min(40.0);

        let noisy_density = if k_lines > 0.0 {
            (severity_distribution.spicy as f64 * 1.5 + severity_distribution.mild as f64) / k_lines
        } else {
            0.0
        };
        let d_score = (noisy_density + 1.0).log2() * 6.0;
        let d_score = d_score.min(60.0);

        let total_score = n_score + d_score;

        let issue_density = if total_lines > 0 {
            issues.len() as f64 / total_lines as f64 * 1000.0
        } else {
            0.0
        };

        CodeQualityScore {
            total_score,
            n_score,
            d_score,
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

    fn build_categories(&self) -> Vec<(&str, Vec<&str>)> {
        vec![
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
            (
                "duplication",
                vec!["code-duplication", "cross-file-duplication"],
            ),
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
                    "global-variable",
                    "bare-rescue",
                    "wildcard-import",
                    "bare-except",
                    "empty-catch",
                    "any-type",
                ],
            ),
            (
                "student-code",
                vec![
                    "println-debugging",
                    "panic-abuse",
                    "todo-comment",
                    "todo-fixme",
                    "todo-bug",
                    "todo-hack",
                ],
            ),
        ]
    }
}

impl Default for CodeScorer {
    fn default() -> Self {
        Self::new()
    }
}
