/// Hall of Shame - tracks and ranks the worst code patterns and files
use std::collections::HashMap;
use std::path::PathBuf;
use crate::analyzer::{CodeIssue, Severity};

#[derive(Debug, Clone)]
pub struct ShameEntry {
    pub file_path: PathBuf,
    pub total_issues: usize,
    pub nuclear_issues: usize,
    pub spicy_issues: usize,
    pub mild_issues: usize,
    pub shame_score: f64,
    pub worst_offenses: Vec<String>,
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
    pub garbage_density: f64, // issues per 1000 lines of code
    pub most_common_patterns: Vec<PatternStats>,
    pub hall_of_shame: Vec<ShameEntry>, // worst files
    pub shame_categories: HashMap<String, usize>,
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

    pub fn add_file_analysis(&mut self, file_path: PathBuf, issues: &[CodeIssue], file_lines: usize) {
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
        let shame_score = (nuclear_count as f64 * 10.0) + 
                         (spicy_count as f64 * 3.0) + 
                         (mild_count as f64 * 1.0);

        let entry = ShameEntry {
            file_path,
            total_issues: issues.len(),
            nuclear_issues: nuclear_count,
            spicy_issues: spicy_count,
            mild_issues: mild_count,
            shame_score,
            worst_offenses,
        };

        self.entries.push(entry);
    }

    fn update_pattern_stats(&mut self, rule_name: &str, severity: &Severity, file_path: &PathBuf) {
        let stats = self.pattern_stats.entry(rule_name.to_string()).or_insert_with(|| {
            PatternStats {
                rule_name: rule_name.to_string(),
                count: 0,
                severity_distribution: HashMap::new(),
                example_files: Vec::new(),
            }
        });

        stats.count += 1;
        *stats.severity_distribution.entry(severity.clone()).or_insert(0) += 1;
        
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
        let mut most_common_patterns: Vec<PatternStats> = self.pattern_stats.values().cloned().collect();
        most_common_patterns.sort_by(|a, b| b.count.cmp(&a.count));

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
            most_common_patterns,
            hall_of_shame,
            shame_categories,
        }
    }

    fn categorize_rule(&self, rule_name: &str) -> String {
        match rule_name {
            name if name.contains("naming") => "Naming Issues".to_string(),
            name if name.contains("complexity") || name.contains("nesting") || name.contains("function") => "Complexity Issues".to_string(),
            name if name.contains("unwrap") || name.contains("panic") || name.contains("string") || name.contains("clone") => "Rust-specific Issues".to_string(),
            name if name.contains("println") || name.contains("todo") => "Student Code Issues".to_string(),
            name if name.contains("import") || name.contains("file") || name.contains("module") => "Structure Issues".to_string(),
            name if name.contains("magic") || name.contains("dead") || name.contains("comment") => "Code Smells".to_string(),
            _ => "Other Issues".to_string(),
        }
    }

    pub fn get_worst_files(&self, limit: usize) -> Vec<&ShameEntry> {
        let mut sorted_entries: Vec<&ShameEntry> = self.entries.iter().collect();
        sorted_entries.sort_by(|a, b| b.shame_score.partial_cmp(&a.shame_score).unwrap());
        sorted_entries.into_iter().take(limit).collect()
    }

    pub fn get_most_common_patterns(&self, limit: usize) -> Vec<&PatternStats> {
        let mut patterns: Vec<&PatternStats> = self.pattern_stats.values().collect();
        patterns.sort_by(|a, b| b.count.cmp(&a.count));
        patterns.into_iter().take(limit).collect()
    }

    pub fn generate_shame_heatmap(&self) -> HashMap<String, f64> {
        // Generate a "heatmap" of shame density by file extension or directory
        let mut heatmap = HashMap::new();
        
        for entry in &self.entries {
            let key = if let Some(parent) = entry.file_path.parent() {
                parent.to_string_lossy().to_string()
            } else {
                "root".to_string()
            };
            
            let current_shame = heatmap.get(&key).unwrap_or(&0.0);
            heatmap.insert(key, current_shame + entry.shame_score);
        }
        
        heatmap
    }

    pub fn get_improvement_suggestions(&self, lang: &str) -> Vec<String> {
        let stats = self.generate_shame_report();
        let mut suggestions = Vec::new();

        // Suggest improvements based on most common issues
        for pattern in stats.most_common_patterns.iter().take(3) {
            match pattern.rule_name.as_str() {
                name if name.contains("naming") => {
                    if lang == "zh-CN" {
                        suggestions.push("🏷️ 重点改进变量和函数命名 - 清晰的名称让代码自文档化".to_string());
                    } else {
                        suggestions.push("🏷️ Focus on improving variable and function naming - clear names make code self-documenting".to_string());
                    }
                }
                name if name.contains("unwrap") => {
                    if lang == "zh-CN" {
                        suggestions.push("🛡️ 用适当的错误处理替换 unwrap() 调用，使用 Result 和 Option".to_string());
                    } else {
                        suggestions.push("🛡️ Replace unwrap() calls with proper error handling using Result and Option".to_string());
                    }
                }
                name if name.contains("complexity") || name.contains("nesting") => {
                    if lang == "zh-CN" {
                        suggestions.push("🧩 将复杂函数分解为更小、更专注的函数".to_string());
                    } else {
                        suggestions.push("🧩 Break down complex functions into smaller, focused functions".to_string());
                    }
                }
                name if name.contains("println") => {
                    if lang == "zh-CN" {
                        suggestions.push("🔍 移除调试用的 println! 语句，使用适当的日志记录".to_string());
                    } else {
                        suggestions.push("🔍 Remove debug println! statements and use proper logging instead".to_string());
                    }
                }
                name if name.contains("clone") => {
                    if lang == "zh-CN" {
                        suggestions.push("⚡ 通过使用引用和理解所有权来减少不必要的克隆".to_string());
                    } else {
                        suggestions.push("⚡ Reduce unnecessary clones by using references and understanding ownership".to_string());
                    }
                }
                _ => {
                    if lang == "zh-CN" {
                        suggestions.push(format!("🔧 处理 {} 问题，发现 {} 次", pattern.rule_name, pattern.count));
                    } else {
                        suggestions.push(format!("🔧 Address {} issues found {} times", pattern.rule_name, pattern.count));
                    }
                }
            }
        }

        if stats.garbage_density > 50.0 {
            if lang == "zh-CN" {
                suggestions.push("📊 检测到高问题密度 - 考虑系统性重构方法".to_string());
            } else {
                suggestions.push("📊 High issue density detected - consider a systematic refactoring approach".to_string());
            }
        }

        if suggestions.is_empty() {
            if lang == "zh-CN" {
                suggestions.push("🎉 干得好！你的代码质量看起来不错！".to_string());
            } else {
                suggestions.push("🎉 Great job! Your code quality is looking good!".to_string());
            }
        }

        suggestions
    }
}

impl Default for HallOfShame {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate anonymous team member statistics (for team environments)
pub fn generate_anonymous_stats(shame_entries: &[ShameEntry]) -> HashMap<String, usize> {
    // This would typically integrate with git blame or similar
    // For now, we'll create a simple hash-based anonymization
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut team_stats = HashMap::new();
    
    for entry in shame_entries {
        // Create anonymous identifier based on file path
        let mut hasher = DefaultHasher::new();
        entry.file_path.hash(&mut hasher);
        let anonymous_id = format!("Developer_{}", hasher.finish() % 100);
        
        *team_stats.entry(anonymous_id).or_insert(0) += entry.total_issues;
    }
    
    team_stats
}