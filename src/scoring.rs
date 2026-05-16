use crate::analyzer::{CodeIssue, Severity};
use std::collections::HashMap;

/// Code quality rating system — accumulation model.
/// Score starts at 0 (best). Each issue adds points.
/// Higher score = worse code quality.
/// 0-20: Excellent  |  21-40: Good  |  41-60: Average  |  61-80: Poor  |  81+: Terrible
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

        // Group issues by (category, rule_name), severity-weighted
        let categories = self.build_categories();
        let sev_weight = |s: &Severity| -> f64 {
            match s {
                Severity::Nuclear => 3.0,
                Severity::Spicy => 1.5,
                Severity::Mild => 0.5,
            }
        };
        let mut cat_rule_weighted: HashMap<String, HashMap<String, f64>> = HashMap::new();

        for issue in issues {
            let w = sev_weight(&issue.severity);
            for (cat_name, rules) in &categories {
                if rules.contains(&issue.rule_name.as_str()) {
                    *cat_rule_weighted
                        .entry(cat_name.to_string())
                        .or_default()
                        .entry(issue.rule_name.clone())
                        .or_insert(0.0) += w;
                }
            }
        }

        // Calculate per-category scores
        let mut category_scores = HashMap::new();
        for (cat_name, _) in &categories {
            let score =
                self.category_accumulated_score(cat_rule_weighted.get(*cat_name), total_lines);
            category_scores.insert(cat_name.to_string(), score);
        }

        let total_score = self.weighted_final_score(&category_scores);

        let issue_density = if total_lines > 0 {
            issues.len() as f64 / total_lines as f64 * 1000.0
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

    /// Weighted final score (0-100). Each category is 0-100.
    /// Weighted by category importance.
    fn weighted_final_score(&self, category_scores: &HashMap<String, f64>) -> f64 {
        let weights: [(&str, f64); 5] = [
            ("naming", 0.20),
            ("complexity", 0.25),
            ("duplication", 0.10),
            ("code-smells", 0.30),
            ("student-code", 0.15),
        ];
        let mut score = 0.0;
        let mut total_w = 0.0;
        for (cat, w) in &weights {
            if let Some(s) = category_scores.get(*cat) {
                score += s * w;
                total_w += w;
            }
        }
        if total_w > 0.0 {
            score / total_w
        } else {
            0.0
        }
    }

    /// Category score: sum of (weighted_count × base_penalty) per rule, normalized to per-1k-lines.
    fn category_accumulated_score(
        &self,
        rule_weights: Option<&HashMap<String, f64>>,
        total_lines: usize,
    ) -> f64 {
        let Some(rules) = rule_weights else {
            return 0.0;
        };
        if total_lines == 0 {
            return 0.0;
        }

        let mut total_penalty = 0.0;
        for (rule_name, &weighted_count) in rules {
            let base = self.rule_base_penalty(rule_name);
            total_penalty += weighted_count * base;
        }
        // Normalize to per-1k-lines
        (total_penalty / total_lines as f64 * 1000.0).min(100.0)
    }

    /// Base penalty per issue — tuned by rule reliability.
    /// Rules verified as reliable get higher penalties.
    /// Rules known to be noisy get lower penalties.
    fn rule_base_penalty(&self, rule: &str) -> f64 {
        match rule {
            // ── Reliable rules (TP ~85-100%) ──────────────────────
            "deep-nesting" => 2.0,
            "god-function" => 2.0,
            "long-function" => 1.5,
            "any-type" => 1.5,    // TS, ~95% TP
            "bare-except" => 2.0, // Python, ~100% TP
            "bare-rescue" => 2.0, // Ruby, ~100% TP
            "empty-catch" => 2.0, // Java, ~100% TP
            "panic-abuse" => 1.0, // detection correct, line:1 bug

            // ── Moderate rules (TP ~40-70%) ───────────────────────
            "magic-number" => 0.3,
            "code-duplication" => 0.4,
            "cross-file-duplication" => 0.3,
            "file-too-long" => 0.5,
            "complex-closure" => 0.8,

            // ── Noisy rules (TP ~0-20%, need fixing) ──────────────
            "single-letter-variable" => 0.1,
            "commented-code" => 0.1,
            "dead-code" => 0.1,
            "terrible-naming" => 0.1,
            "hungarian-notation" => 0.1,
            "abbreviation-abuse" => 0.1,
            "global-variable" => 0.1,
            "println-debugging" => 0.2,

            // ── Rust-specific ─────────────────────────────────────
            "unwrap-abuse" => 0.5,
            "box-abuse" => 0.1, // line:1 + fabricated
            "unnecessary-clone" => 0.5,
            "macro-abuse" => 0.3,
            "lifetime-abuse" => 0.2,
            "generic-abuse" => 0.2,
            "pattern-matching-abuse" => 0.2,
            "reference-abuse" => 0.2,
            "string-abuse" => 0.3,
            "vec-abuse" => 0.3,
            "module-complexity" => 0.3,
            "trait-complexity" => 0.3,

            // ── Go-specific ───────────────────────────────────────
            "defer-in-loop" => 0.8,
            "goroutine-abuse" => 0.5,

            // ── Python-specific ───────────────────────────────────
            "wildcard-import" => 0.3,

            // ── Other ─────────────────────────────────────────────
            "duplicate-imports" => 0.3,
            "todo-comment" | "todo-fixme" | "todo-bug" | "todo-hack" => 0.1,
            "meaningless-naming" => 0.2,

            // ── C/C++ ─────────────────────────────────────────────
            "c-naming" | "c-nesting" | "c-long-function" => 1.0,
            "c-magic-number" | "c-god-function" => 0.5,
            "c-commented-code" | "c-dead-code" => 0.2,
            "c-include-chaos" | "c-goto-abuse" | "c-malloc-leak" => 1.0,

            _ => 0.3, // unknown rules get a moderate default
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
