use std::collections::HashMap;
use crate::analyzer::{CodeIssue, Severity};

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
    Excellent,  // 0-20
    Good,       // 21-40
    Average,    // 41-60
    Poor,       // 61-80
    Terrible,   // 81-100
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
            QualityLevel::Poor => "😟",
            QualityLevel::Terrible => "💀",
        }
    }
}

pub struct CodeScorer {
    /// rule weights
    rule_weights: HashMap<String, f64>,
    /// severity weights
    severity_weights: HashMap<Severity, f64>,
}

impl CodeScorer {
    pub fn new() -> Self {
        let mut rule_weights = HashMap::new();
        
        // Basic code quality issues
        rule_weights.insert("terrible-naming".to_string(), 2.0);
        rule_weights.insert("single-letter-variable".to_string(), 1.5);
        
        // Complexity issues
        rule_weights.insert("deep-nesting".to_string(), 3.0);
        rule_weights.insert("long-function".to_string(), 2.5);
        rule_weights.insert("cyclomatic-complexity".to_string(), 3.5);
        rule_weights.insert("code-duplication".to_string(), 4.0);
        
        // Rust specific issues
        rule_weights.insert("unwrap-abuse".to_string(), 4.0);  // high weight, because it may cause panic
        rule_weights.insert("unnecessary-clone".to_string(), 2.0);
        
        // Advanced Rust features abuse
        rule_weights.insert("complex-closure".to_string(), 2.5);
        rule_weights.insert("lifetime-abuse".to_string(), 3.5);
        rule_weights.insert("trait-complexity".to_string(), 3.0);
        rule_weights.insert("generic-abuse".to_string(), 2.5);
        
        // Rust features abuse
        rule_weights.insert("channel-abuse".to_string(), 3.0);
        rule_weights.insert("async-abuse".to_string(), 3.5);
        rule_weights.insert("dyn-trait-abuse".to_string(), 2.5);
        rule_weights.insert("unsafe-abuse".to_string(), 5.0);  // highest weight, because it's a safety issue
        rule_weights.insert("ffi-abuse".to_string(), 4.5);     // high weight, because it's a safety issue
        rule_weights.insert("macro-abuse".to_string(), 3.0);
        rule_weights.insert("module-complexity".to_string(), 2.0);
        rule_weights.insert("pattern-matching-abuse".to_string(), 2.0);
        rule_weights.insert("reference-abuse".to_string(), 2.5);
        rule_weights.insert("box-abuse".to_string(), 2.0);
        rule_weights.insert("slice-abuse".to_string(), 1.5);

        let mut severity_weights = HashMap::new();
        severity_weights.insert(Severity::Nuclear, 10.0);  // nuclear penalty: first nuclear +20, each subsequent +5
        severity_weights.insert(Severity::Spicy, 5.0);     // spicy penalty: first 5 spicy +2, each subsequent +2
        severity_weights.insert(Severity::Mild, 2.0);      // mild penalty: first 20 mild +0.5, each subsequent +0.5

        Self {
            rule_weights,
            severity_weights,
        }
    }

    /// calculate code quality score
    pub fn calculate_score(&self, issues: &[CodeIssue], file_count: usize, total_lines: usize) -> CodeQualityScore {
        if issues.is_empty() {
            return CodeQualityScore {
                total_score: 0.0,
                category_scores: HashMap::new(),
                file_count,
                total_lines,
                issue_density: 0.0,
                severity_distribution: SeverityDistribution { nuclear: 0, spicy: 0, mild: 0 },
                quality_level: QualityLevel::Excellent,
            };
        }

        // calculate severity distribution
        let severity_distribution = self.calculate_severity_distribution(issues);
        
        // calculate base score
        let base_score = self.calculate_base_score(issues);
        
        // calculate density penalty
        let density_penalty = self.calculate_density_penalty(issues.len(), file_count, total_lines);
        
        // calculate severity penalty
        let severity_penalty = self.calculate_severity_penalty(&severity_distribution);
        
        // calculate category scores
        let category_scores = self.calculate_category_scores(issues);
        
        // calculate final score (higher score = worse code)
        let total_score = (base_score + density_penalty + severity_penalty).min(100.0);
        
        let issue_density = if total_lines > 0 {
            issues.len() as f64 / total_lines as f64 * 1000.0  // issues per 1000 lines
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

        SeverityDistribution { nuclear, spicy, mild }
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

    fn calculate_density_penalty(&self, issue_count: usize, file_count: usize, total_lines: usize) -> f64 {
        if total_lines == 0 || file_count == 0 {
            return 0.0;
        }

        // calculate issues density (issues per 1000 lines)
        let issues_per_1000_lines = (issue_count as f64 / total_lines as f64) * 1000.0;
        
        // calculate average issues per file
        let issues_per_file = issue_count as f64 / file_count as f64;

        // calculate density penalty
        let density_penalty = match issues_per_1000_lines {
            x if x > 50.0 => 25.0,   // high density
            x if x > 30.0 => 15.0,   // medium density
            x if x > 20.0 => 10.0,   // low density
            x if x > 10.0 => 5.0,    // very low density
            _ => 0.0,                // very low density
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
            penalty += 20.0 + (distribution.nuclear as f64 - 1.0) * 5.0;  // nuclear penalty: first nuclear +20, each subsequent +5
        }

        // calculate spicy penalty
        if distribution.spicy > 5 {
            penalty += (distribution.spicy as f64 - 5.0) * 2.0;  // spicy penalty: first 5 spicy +2, each subsequent +2
        }

        // calculate mild penalty
        if distribution.mild > 20 {
            penalty += (distribution.mild as f64 - 20.0) * 0.5;  // mild penalty: first 20 mild +0.5, each subsequent +0.5
        }

        penalty
    }

    fn calculate_category_scores(&self, issues: &[CodeIssue]) -> HashMap<String, f64> {
        let mut category_scores = HashMap::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();

        // define issue categories
        let categories = [
            ("naming", vec!["terrible-naming", "single-letter-variable"]),
            ("complexity", vec!["deep-nesting", "long-function", "cyclomatic-complexity"]),
            ("duplication", vec!["code-duplication"]),
            ("rust-basics", vec!["unwrap-abuse", "unnecessary-clone"]),
            ("advanced-rust", vec!["complex-closure", "lifetime-abuse", "trait-complexity", "generic-abuse"]),
            ("rust-features", vec!["channel-abuse", "async-abuse", "dyn-trait-abuse", "unsafe-abuse", "ffi-abuse", "macro-abuse"]),
            ("structure", vec!["module-complexity", "pattern-matching-abuse", "reference-abuse", "box-abuse", "slice-abuse"]),
        ];

        // calculate category scores
        for issue in issues {
            for (category_name, rules) in &categories {
                if rules.contains(&issue.rule_name.as_str()) {
                    *category_counts.entry(category_name.to_string()).or_insert(0) += 1;
                    
                    let rule_weight = self.rule_weights.get(&issue.rule_name).unwrap_or(&1.0);
                    let severity_weight = self.severity_weights.get(&issue.severity).unwrap_or(&1.0);
                    
                    *category_scores.entry(category_name.to_string()).or_insert(0.0) += 
                        rule_weight * severity_weight;
                }
            }
        }

        category_scores
    }
}

impl Default for CodeScorer {
    fn default() -> Self {
        Self::new()
    }
}