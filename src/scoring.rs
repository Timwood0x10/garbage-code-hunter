use crate::analyzer::{CodeIssue, Severity};
use crate::signals::{classify_rule, compute_signal_scores, StyleSignal};
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
    pub signal_scores: HashMap<StyleSignal, f64>,
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
                signal_scores: HashMap::new(),
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
        let k_lines = total_lines as f64 / 1000.0;
        let mut category_counts: HashMap<&str, usize> = HashMap::new();
        for issue in issues {
            let cat = legacy_category_name(classify_rule(&issue.rule_name));
            *category_counts.entry(cat).or_insert(0) += 1;
        }
        let mut category_scores = HashMap::new();
        for &cat_name in &[
            "naming",
            "complexity",
            "duplication",
            "code-smells",
            "student-code",
        ] {
            let cat_count = category_counts.get(cat_name).copied().unwrap_or(0);
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

        let signal_scores = compute_signal_scores(issues, total_lines);

        CodeQualityScore {
            total_score,
            n_score,
            d_score,
            category_scores,
            signal_scores,
            file_count,
            total_lines,
            issue_density,
            severity_distribution,
            quality_level: QualityLevel::from_score(total_score),
        }
    }

    pub fn calculate_score_with_direct(
        &self,
        issues: &[CodeIssue],
        file_count: usize,
        total_lines: usize,
        direct_scores: HashMap<StyleSignal, f64>,
    ) -> CodeQualityScore {
        let mut score = self.calculate_score(issues, file_count, total_lines);
        for (signal, direct_score) in direct_scores {
            let entry = score.signal_scores.entry(signal).or_insert(0.0);
            *entry = (*entry).max(direct_score);
        }
        score
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
}

fn legacy_category_name(signal: StyleSignal) -> &'static str {
    match signal {
        StyleSignal::NamingChaos => "naming",
        StyleSignal::NestedHell => "complexity",
        StyleSignal::Duplication => "duplication",
        StyleSignal::PanicAddiction | StyleSignal::HotfixCulture => "student-code",
        StyleSignal::OverEngineering | StyleSignal::CodeSmells => "code-smells",
        StyleSignal::LegacyCode => "code-smells",
        StyleSignal::TodoMountain => "student-code",
        StyleSignal::LineCountSmell => "complexity",
    }
}

impl Default for CodeScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Severity;
    use std::path::PathBuf;

    fn issue(rule: &str, sev: Severity) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            rule_name: rule.to_string(),
            message: String::new(),
            severity: sev,
        }
    }

    // ── QualityLevel ─────────────────────────────────────────────

    /// Objective: Verify from_score returns the correct level for each tier.
    /// Invariants: Boundaries (20, 40, 60, 80) belong to the lower tier.
    #[test]
    fn test_quality_level_tier_boundaries() {
        assert_eq!(
            QualityLevel::from_score(0.0),
            QualityLevel::Excellent,
            "score 0 should be Excellent"
        );
        assert_eq!(
            QualityLevel::from_score(20.0),
            QualityLevel::Excellent,
            "score 20 should still be Excellent (lower bound)"
        );
        assert_eq!(
            QualityLevel::from_score(21.0),
            QualityLevel::Good,
            "score 21 transitions to Good"
        );
        assert_eq!(
            QualityLevel::from_score(40.0),
            QualityLevel::Good,
            "score 40 should still be Good"
        );
        assert_eq!(
            QualityLevel::from_score(41.0),
            QualityLevel::Average,
            "score 41 transitions to Average"
        );
        assert_eq!(
            QualityLevel::from_score(60.0),
            QualityLevel::Average,
            "score 60 should still be Average"
        );
        assert_eq!(
            QualityLevel::from_score(61.0),
            QualityLevel::Poor,
            "score 61 transitions to Poor"
        );
        assert_eq!(
            QualityLevel::from_score(80.0),
            QualityLevel::Poor,
            "score 80 should still be Poor"
        );
        assert_eq!(
            QualityLevel::from_score(81.0),
            QualityLevel::Terrible,
            "score 81 transitions to Terrible"
        );
        assert_eq!(
            QualityLevel::from_score(100.0),
            QualityLevel::Terrible,
            "score 100 is Terrible"
        );
    }

    /// Objective: Verify QualityLevel::description returns correct English strings.
    #[test]
    fn test_quality_level_description_english() {
        assert_eq!(
            QualityLevel::Excellent.description("en"),
            "Excellent",
            "English description for Excellent"
        );
        assert_eq!(
            QualityLevel::Terrible.description("en"),
            "Terrible",
            "English description for Terrible"
        );
    }

    /// Objective: Verify QualityLevel::description returns correct Chinese strings.
    #[test]
    fn test_quality_level_description_chinese() {
        assert_eq!(
            QualityLevel::Excellent.description("zh-CN"),
            "优秀",
            "Chinese description for Excellent"
        );
        assert_eq!(
            QualityLevel::Terrible.description("zh-CN"),
            "糟糕",
            "Chinese description for Terrible"
        );
    }

    // ── CodeScorer — empty input ──────────────────────────────────

    /// Objective: Verify that empty issues result in zero score with Excellent level.
    /// Invariants: total_score, n_score, d_score are all 0 when issues is empty.
    #[test]
    fn test_empty_issues_score_zero() {
        let scorer = CodeScorer::new();
        let score = scorer.calculate_score(&[], 5, 1000);
        assert_eq!(
            score.total_score, 0.0,
            "empty issues => total_score 0, got {}",
            score.total_score
        );
        assert_eq!(
            score.quality_level,
            QualityLevel::Excellent,
            "empty issues => Excellent quality"
        );
        assert_eq!(score.issue_density, 0.0, "empty issues => density 0");
        assert_eq!(score.n_score, 0.0, "empty issues => n_score 0");
        assert_eq!(score.d_score, 0.0, "empty issues => d_score 0");
    }

    // ── CodeScorer — severity distribution ─────────────────────────

    /// Objective: Verify severity distribution counts each severity bucket correctly.
    /// Invariants: The sum of all counts equals the total number of issues.
    #[test]
    fn test_severity_distribution_counts() {
        let scorer = CodeScorer::new();
        let issues = vec![
            issue("a", Severity::Nuclear),
            issue("b", Severity::Spicy),
            issue("c", Severity::Mild),
            issue("d", Severity::Nuclear),
        ];
        let dist = scorer.calculate_severity_distribution(&issues);
        assert_eq!(dist.nuclear, 2, "should count 2 nuclear issues");
        assert_eq!(dist.spicy, 1, "should count 1 spicy issue");
        assert_eq!(dist.mild, 1, "should count 1 mild issue");
        assert_eq!(
            dist.nuclear + dist.spicy + dist.mild,
            issues.len(),
            "severity counts must sum to total issue count"
        );
    }

    // ── CodeScorer — two-tier log scoring ──────────────────────────

    /// Objective: Verify n_score grows monotonically with nuclear issue count.
    /// Invariants: More nuclear issues => strictly larger n_score.
    #[test]
    fn test_n_score_monotonic_with_nuclear_count() {
        let scorer = CodeScorer::new();
        let one_nuke = scorer.calculate_score(&[issue("n1", Severity::Nuclear)], 1, 1000);
        let two_nukes = scorer.calculate_score(
            &[
                issue("n1", Severity::Nuclear),
                issue("n2", Severity::Nuclear),
            ],
            1,
            1000,
        );
        assert!(
            two_nukes.n_score > one_nuke.n_score,
            "n_score should increase from {} to {} with more nuclear issues",
            one_nuke.n_score,
            two_nukes.n_score
        );
    }

    /// Objective: Verify n_score is capped at 40 (log2 cap).
    /// Invariants: Even with 100 nuclear issues, n_score never exceeds 40.
    #[test]
    fn test_n_score_capped_at_40() {
        let scorer = CodeScorer::new();
        let issues: Vec<CodeIssue> = (0..100)
            .map(|i| issue(&format!("x{i}"), Severity::Nuclear))
            .collect();
        let score = scorer.calculate_score(&issues, 1, 1000);
        assert!(
            score.n_score <= 40.0,
            "n_score cap is 40, got {}",
            score.n_score
        );
    }

    /// Objective: Verify d_score increases when issue density is higher
    ///            (same issues in fewer lines).
    /// Invariants: Denser project produces strictly higher d_score.
    #[test]
    fn test_d_score_higher_with_denser_code() {
        let scorer = CodeScorer::new();
        let issues: Vec<CodeIssue> = (0..50)
            .map(|i| issue(&format!("m{i}"), Severity::Mild))
            .collect();
        let sparse = scorer.calculate_score(&issues, 1, 10000);
        let dense = scorer.calculate_score(&issues, 1, 500);
        assert!(
            dense.d_score > sparse.d_score,
            "dense (50 issues / 500 lines) should score higher d_score than sparse (50 / 10000), got {} vs {}",
            dense.d_score, sparse.d_score
        );
    }

    /// Objective: Verify d_score is capped at 60 (log-scaled density cap).
    #[test]
    fn test_d_score_capped_at_60() {
        let scorer = CodeScorer::new();
        let issues: Vec<CodeIssue> = (0..5000)
            .map(|i| issue(&format!("m{i}"), Severity::Mild))
            .collect();
        let score = scorer.calculate_score(&issues, 1, 100);
        assert!(
            score.d_score <= 60.0,
            "d_score cap is 60, got {}",
            score.d_score
        );
    }

    /// Objective: Verify total_score = n_score + d_score always.
    /// Invariants: total_score must equal the sum of its two components.
    #[test]
    fn test_total_score_is_n_plus_d() {
        let scorer = CodeScorer::new();
        let issues = vec![
            issue("n", Severity::Nuclear),
            issue("s", Severity::Spicy),
            issue("m", Severity::Mild),
        ];
        let score = scorer.calculate_score(&issues, 1, 1000);
        let expected = score.n_score + score.d_score;
        assert!(
            (score.total_score - expected).abs() < 1e-6,
            "total_score ({}) should equal n_score ({}) + d_score ({}) = {}",
            score.total_score,
            score.n_score,
            score.d_score,
            expected
        );
    }

    /// Objective: Verify zero total_lines does not produce NaN or crash.
    /// Invariants: d_score = log2(1)*6 = 0 when density denominator is 0.
    ///             n_score is unaffected.
    #[test]
    fn test_zero_lines_does_not_produce_nan() {
        let scorer = CodeScorer::new();
        let score = scorer.calculate_score(&[issue("x", Severity::Nuclear)], 1, 0);
        assert!(
            score.total_score.is_finite(),
            "total_score must be finite, got {}",
            score.total_score
        );
        assert!(score.n_score.is_finite(), "n_score must be finite");
        assert!(score.d_score.is_finite(), "d_score must be finite");
        assert!(
            score.total_score > 0.0,
            "with a nuclear issue, total_score should be > 0"
        );
    }

    // ── CodeScorer — category scores ───────────────────────────────

    /// Objective: Verify that all five expected category keys exist in the result.
    /// Invariants: The category map always contains naming/complexity/duplication/code-smells/student-code.
    #[test]
    fn test_all_category_keys_present() {
        let scorer = CodeScorer::new();
        let score = scorer.calculate_score(&[issue("terrible-naming", Severity::Mild)], 1, 1000);
        for cat in &[
            "naming",
            "complexity",
            "duplication",
            "code-smells",
            "student-code",
        ] {
            assert!(
                score.category_scores.contains_key(*cat),
                "category '{}' should exist in scores",
                cat
            );
        }
    }

    /// Objective: Verify category score > 0 when at least one rule in that category matches.
    #[test]
    fn test_category_score_positive_when_rule_matches() {
        let scorer = CodeScorer::new();
        let score = scorer.calculate_score(
            &[
                issue("terrible-naming", Severity::Nuclear),
                issue("single-letter-variable", Severity::Spicy),
            ],
            1,
            1000,
        );
        let naming_score = score
            .category_scores
            .get("naming")
            .expect("naming category should exist");
        assert!(
            *naming_score > 0.0,
            "naming category should have non-zero score when naming rules fire, got {}",
            naming_score
        );
    }

    /// Objective: Verify category score is 0 when no rules in that category fire.
    #[test]
    fn test_category_score_zero_when_no_matching_rules() {
        let scorer = CodeScorer::new();
        let score = scorer.calculate_score(&[issue("unwrap-abuse", Severity::Mild)], 1, 1000);
        let naming_score = score
            .category_scores
            .get("naming")
            .expect("naming category should exist");
        assert_eq!(
            *naming_score, 0.0,
            "naming category should be 0 when no naming rules fire"
        );
    }

    /// Objective: Verify category scores across different categories are independent.
    /// Invariants: Rules in category A only affect category A's score, not category B's.
    #[test]
    fn test_categories_are_independent() {
        let scorer = CodeScorer::new();
        let score = scorer.calculate_score(
            &[
                issue("terrible-naming", Severity::Nuclear),
                issue("deep-nesting", Severity::Spicy),
            ],
            1,
            1000,
        );
        let naming = *score
            .category_scores
            .get("naming")
            .expect("naming category");
        let complexity = *score
            .category_scores
            .get("complexity")
            .expect("complexity category");
        assert!(
            naming > 0.0 && complexity > 0.0,
            "both naming ({naming}) and complexity ({complexity}) should be > 0 when their rules fire"
        );
        let duplication = *score
            .category_scores
            .get("duplication")
            .expect("duplication category");
        assert_eq!(
            duplication, 0.0,
            "duplication category should be 0 since no duplication rule fired"
        );
    }
}
