use std::collections::HashMap;
use crate::analyzer::{CodeIssue, Severity};

/// 代码质量评分系统
/// 分数范围：0-100分，分数越低代码质量越好
/// 0-20: 优秀 (Excellent)
/// 21-40: 良好 (Good) 
/// 41-60: 一般 (Average)
/// 61-80: 较差 (Poor)
/// 81-100: 糟糕 (Terrible)
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
    /// 权重配置：不同规则类型的权重
    rule_weights: HashMap<String, f64>,
    /// 严重程度权重
    severity_weights: HashMap<Severity, f64>,
}

impl CodeScorer {
    pub fn new() -> Self {
        let mut rule_weights = HashMap::new();
        
        // 基础代码质量问题权重
        rule_weights.insert("terrible-naming".to_string(), 2.0);
        rule_weights.insert("single-letter-variable".to_string(), 1.5);
        
        // 复杂度问题权重（影响较大）
        rule_weights.insert("deep-nesting".to_string(), 3.0);
        rule_weights.insert("long-function".to_string(), 2.5);
        
        // Rust特定问题权重
        rule_weights.insert("unwrap-abuse".to_string(), 4.0);  // 高权重，因为可能导致panic
        rule_weights.insert("unnecessary-clone".to_string(), 2.0);
        
        // 高级Rust特性滥用权重
        rule_weights.insert("complex-closure".to_string(), 2.5);
        rule_weights.insert("lifetime-abuse".to_string(), 3.5);
        rule_weights.insert("trait-complexity".to_string(), 3.0);
        rule_weights.insert("generic-abuse".to_string(), 2.5);
        
        // 综合Rust特性问题权重
        rule_weights.insert("channel-abuse".to_string(), 3.0);
        rule_weights.insert("async-abuse".to_string(), 3.5);
        rule_weights.insert("dyn-trait-abuse".to_string(), 2.5);
        rule_weights.insert("unsafe-abuse".to_string(), 5.0);  // 最高权重，安全问题
        rule_weights.insert("ffi-abuse".to_string(), 4.5);     // 高权重，FFI安全问题
        rule_weights.insert("macro-abuse".to_string(), 3.0);
        rule_weights.insert("module-complexity".to_string(), 2.0);
        rule_weights.insert("pattern-matching-abuse".to_string(), 2.0);
        rule_weights.insert("reference-abuse".to_string(), 2.5);
        rule_weights.insert("box-abuse".to_string(), 2.0);
        rule_weights.insert("slice-abuse".to_string(), 1.5);

        let mut severity_weights = HashMap::new();
        severity_weights.insert(Severity::Nuclear, 10.0);  // 核弹级问题权重最高
        severity_weights.insert(Severity::Spicy, 5.0);     // 中等问题
        severity_weights.insert(Severity::Mild, 2.0);      // 轻微问题

        Self {
            rule_weights,
            severity_weights,
        }
    }

    /// 计算代码质量评分
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

        // 统计严重程度分布
        let severity_distribution = self.calculate_severity_distribution(issues);
        
        // 计算基础分数
        let base_score = self.calculate_base_score(issues);
        
        // 计算密度惩罚
        let density_penalty = self.calculate_density_penalty(issues.len(), file_count, total_lines);
        
        // 计算严重程度惩罚
        let severity_penalty = self.calculate_severity_penalty(&severity_distribution);
        
        // 计算分类分数
        let category_scores = self.calculate_category_scores(issues);
        
        // 计算最终分数
        let total_score = (base_score + density_penalty + severity_penalty).min(100.0);
        
        let issue_density = if total_lines > 0 {
            issues.len() as f64 / total_lines as f64 * 1000.0  // 每千行代码的问题数
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
            
            // 基础分数 = 规则权重 × 严重程度权重
            score += rule_weight * severity_weight;
        }

        score
    }

    fn calculate_density_penalty(&self, issue_count: usize, file_count: usize, total_lines: usize) -> f64 {
        if total_lines == 0 || file_count == 0 {
            return 0.0;
        }

        // 计算问题密度（每千行代码的问题数）
        let issues_per_1000_lines = (issue_count as f64 / total_lines as f64) * 1000.0;
        
        // 计算文件平均问题数
        let issues_per_file = issue_count as f64 / file_count as f64;

        // 密度惩罚：问题密度越高，惩罚越重
        let density_penalty = match issues_per_1000_lines {
            x if x > 50.0 => 25.0,   // 极高密度
            x if x > 30.0 => 15.0,   // 高密度
            x if x > 20.0 => 10.0,   // 中等密度
            x if x > 10.0 => 5.0,    // 低密度
            _ => 0.0,                // 很低密度
        };

        // 文件问题数惩罚
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

        // 核弹级问题的额外惩罚
        if distribution.nuclear > 0 {
            penalty += 20.0 + (distribution.nuclear as f64 - 1.0) * 5.0;  // 第一个核弹级+20，后续每个+5
        }

        // 严重问题的额外惩罚
        if distribution.spicy > 5 {
            penalty += (distribution.spicy as f64 - 5.0) * 2.0;  // 超过5个严重问题后，每个+2
        }

        // 轻微问题的累积惩罚
        if distribution.mild > 20 {
            penalty += (distribution.mild as f64 - 20.0) * 0.5;  // 超过20个轻微问题后，每个+0.5
        }

        penalty
    }

    fn calculate_category_scores(&self, issues: &[CodeIssue]) -> HashMap<String, f64> {
        let mut category_scores = HashMap::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();

        // 定义问题分类
        let categories = [
            ("naming", vec!["terrible-naming", "single-letter-variable"]),
            ("complexity", vec!["deep-nesting", "long-function"]),
            ("rust-basics", vec!["unwrap-abuse", "unnecessary-clone"]),
            ("advanced-rust", vec!["complex-closure", "lifetime-abuse", "trait-complexity", "generic-abuse"]),
            ("rust-features", vec!["channel-abuse", "async-abuse", "dyn-trait-abuse", "unsafe-abuse", "ffi-abuse", "macro-abuse"]),
            ("structure", vec!["module-complexity", "pattern-matching-abuse", "reference-abuse", "box-abuse", "slice-abuse"]),
        ];

        // 统计每个分类的问题
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