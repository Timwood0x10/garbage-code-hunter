use colored::*;
use std::collections::HashMap;

use crate::analyzer::{CodeIssue, Severity};
use crate::i18n::I18n;
use crate::scoring::{CodeScorer, CodeQualityScore};

pub struct Reporter {
    harsh_mode: bool,
    savage_mode: bool,
    verbose: bool,
    top_files: usize,
    max_issues_per_file: usize,
    summary_only: bool,
    markdown: bool,
    i18n: I18n,
}

impl Reporter {
    pub fn new(
        harsh_mode: bool,
        savage_mode: bool,
        verbose: bool,
        top_files: usize,
        max_issues_per_file: usize,
        summary_only: bool,
        markdown: bool,
        lang: &str,
    ) -> Self {
        Self {
            harsh_mode,
            savage_mode,
            verbose,
            top_files,
            max_issues_per_file,
            summary_only,
            markdown,
            i18n: I18n::new(lang),
        }
    }

    #[allow(dead_code)]
    pub fn report(&self, issues: Vec<CodeIssue>) {
        self.report_with_metrics(issues, 1, 100);
    }

    pub fn report_with_metrics(&self, mut issues: Vec<CodeIssue>, file_count: usize, total_lines: usize) {
        // 计算代码质量评分
        let scorer = CodeScorer::new();
        let quality_score = scorer.calculate_score(&issues, file_count, total_lines);

        if issues.is_empty() {
            self.print_clean_code_message_with_score(&quality_score);
            return;
        }

        // 按严重程度排序
        issues.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Nuclear => 3,
                Severity::Spicy => 2,
                Severity::Mild => 1,
            };
            severity_order(&b.severity).cmp(&severity_order(&a.severity))
        });

        // 如果是 harsh 模式，只显示最严重的问题
        if self.harsh_mode {
            issues.retain(|issue| matches!(issue.severity, Severity::Nuclear | Severity::Spicy));
        }

        if self.markdown {
            self.print_markdown_report(&issues);
        } else {
            if !self.summary_only {
                self.print_header(&issues);
                self.print_quality_score(&quality_score);
                if self.verbose {
                    self.print_detailed_analysis(&issues);
                }
                self.print_top_files(&issues);
                self.print_issues(&issues);
            }
            self.print_summary_with_score(&issues, &quality_score);
            if !self.summary_only {
                self.print_footer(&issues);
            }
        }
    }

    #[allow(dead_code)]
    fn print_clean_code_message(&self) {
        if self.markdown {
            println!("# {}", self.i18n.get("title"));
            println!();
            println!("{}", self.i18n.get("clean_code"));
            println!();
            println!("{}", self.i18n.get("clean_code_warning"));
        } else {
            println!("{}", self.i18n.get("clean_code").bright_green().bold());
            println!("{}", self.i18n.get("clean_code_warning").yellow());
        }
    }

    fn print_clean_code_message_with_score(&self, quality_score: &CodeQualityScore) {
        if self.markdown {
            println!("# {}", self.i18n.get("title"));
            println!();
            println!("## 🏆 代码质量评分");
            println!();
            println!("**评分**: {:.1}/100 {}", quality_score.total_score, quality_score.quality_level.emoji());
            println!("**等级**: {}", quality_score.quality_level.description(&self.i18n.lang));
            println!();
            println!("{}", self.i18n.get("clean_code"));
            println!();
            println!("{}", self.i18n.get("clean_code_warning"));
        } else {
            println!("{}", self.i18n.get("clean_code").bright_green().bold());
            println!();
            println!("{} 代码质量评分: {:.1}/100 {}", 
                "🏆".bright_yellow(),
                quality_score.total_score.to_string().bright_green().bold(),
                quality_score.quality_level.emoji()
            );
            println!("{} 质量等级: {}", 
                "📊".bright_blue(),
                quality_score.quality_level.description(&self.i18n.lang).bright_green().bold()
            );
            println!("{}", self.i18n.get("clean_code_warning").yellow());
        }
    }

    fn print_quality_score(&self, quality_score: &CodeQualityScore) {
        println!("{}", "🏆 代码质量评分".bright_yellow().bold());
        println!("{}", "─".repeat(50).bright_black());
        
        let score_color = match quality_score.quality_level {
            crate::scoring::QualityLevel::Excellent => quality_score.total_score.to_string().bright_green().bold(),
            crate::scoring::QualityLevel::Good => quality_score.total_score.to_string().green(),
            crate::scoring::QualityLevel::Average => quality_score.total_score.to_string().yellow(),
            crate::scoring::QualityLevel::Poor => quality_score.total_score.to_string().red(),
            crate::scoring::QualityLevel::Terrible => quality_score.total_score.to_string().bright_red().bold(),
        };

        println!("   📊 总分: {:.1}/100 {}", 
            score_color,
            quality_score.quality_level.emoji()
        );
        println!("   🎯 等级: {}", 
            quality_score.quality_level.description(&self.i18n.lang).bright_white().bold()
        );
        
        if quality_score.total_lines > 0 {
            println!("   📏 代码行数: {}", quality_score.total_lines.to_string().cyan());
            println!("   📁 文件数量: {}", quality_score.file_count.to_string().cyan());
            println!("   🔍 问题密度: {:.2} 问题/千行", quality_score.issue_density.to_string().cyan());
        }

        // 显示严重程度分布
        if quality_score.severity_distribution.nuclear > 0 || 
           quality_score.severity_distribution.spicy > 0 || 
           quality_score.severity_distribution.mild > 0 {
            println!();
            println!("   🎭 问题分布:");
            if quality_score.severity_distribution.nuclear > 0 {
                println!("      💥 核弹级: {}", quality_score.severity_distribution.nuclear.to_string().red().bold());
            }
            if quality_score.severity_distribution.spicy > 0 {
                println!("      🌶️  严重: {}", quality_score.severity_distribution.spicy.to_string().yellow());
            }
            if quality_score.severity_distribution.mild > 0 {
                println!("      😐 轻微: {}", quality_score.severity_distribution.mild.to_string().blue());
            }
        }

        // 显示分类得分（如果有的话）
        if !quality_score.category_scores.is_empty() && self.verbose {
            println!();
            println!("   📋 分类得分:");
            let mut sorted_categories: Vec<_> = quality_score.category_scores.iter().collect();
            sorted_categories.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            for (category, score) in sorted_categories.iter().take(5) {
                let category_name = match category.as_str() {
                    "naming" => "命名规范",
                    "complexity" => "复杂度",
                    "rust-basics" => "Rust基础",
                    "advanced-rust" => "高级特性",
                    "rust-features" => "Rust功能",
                    "structure" => "代码结构",
                    _ => category,
                };
                println!("      {} {:.1}", category_name.cyan(), score.to_string().yellow());
            }
        }
        
        println!();
    }

    fn print_header(&self, issues: &[CodeIssue]) {
        let total = issues.len();
        let nuclear = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let spicy = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Spicy))
            .count();
        let mild = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Mild))
            .count();

        println!("{}", self.i18n.get("title").bright_red().bold());
        println!("{}", self.i18n.get("preparing").yellow());
        println!();

        println!("{}", self.i18n.get("report_title").bright_red().bold());
        println!("{}", "─".repeat(50).bright_black());

        if self.savage_mode {
            println!("{}", self.i18n.get("found_issues").red().bold());
        } else {
            println!("{}", self.i18n.get("found_issues").yellow());
        }

        println!();
        println!("{}", self.i18n.get("statistics"));
        println!(
            "   {} {}",
            nuclear.to_string().red().bold(),
            self.i18n.get("nuclear_issues")
        );
        println!(
            "   {} {}",
            spicy.to_string().yellow().bold(),
            self.i18n.get("spicy_issues")
        );
        println!(
            "   {} {}",
            mild.to_string().blue().bold(),
            self.i18n.get("mild_issues")
        );
        println!(
            "   {} {}",
            total.to_string().bright_white().bold(),
            self.i18n.get("total")
        );
        println!();
    }

    fn print_issues(&self, issues: &[CodeIssue]) {
        let mut file_groups: HashMap<String, Vec<&CodeIssue>> = HashMap::new();

        for issue in issues {
            let file_name = issue
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            file_groups.entry(file_name).or_default().push(issue);
        }

        for (file_name, file_issues) in file_groups {
            println!("{} {}", "📁".bright_blue(), file_name.bright_blue().bold());

            let issues_to_show = if self.max_issues_per_file > 0 {
                file_issues
                    .into_iter()
                    .take(self.max_issues_per_file)
                    .collect::<Vec<_>>()
            } else {
                file_issues
            };

            for issue in issues_to_show {
                self.print_issue(issue);
            }
            println!();
        }
    }

    fn print_issue(&self, issue: &CodeIssue) {
        let severity_icon = match issue.severity {
            Severity::Nuclear => "💥",
            Severity::Spicy => "🌶️",
            Severity::Mild => "😐",
        };

        let line_info = format!("{}:{}", issue.line, issue.column).bright_black();

        
        let messages = self.i18n.get_roast_messages(&issue.rule_name);
        let message = if !messages.is_empty() {
            messages[issue.line % messages.len()].clone()
        } else {
            issue.message.clone()
        };

     
        let final_message = if self.savage_mode {
            self.make_message_savage(&message)
        } else {
            message
        };

        let colored_message = match issue.severity {
            Severity::Nuclear => final_message.red().bold(),
            Severity::Spicy => final_message.yellow(),
            Severity::Mild => final_message.blue(),
        };

        println!("  {} {} {}", severity_icon, line_info, colored_message);
    }

    fn make_message_savage(&self, message: &str) -> String {
        let savage_prefixes = vec![
            "🔥 严重警告：",
            "💀 代码死刑：",
            "🗑️ 垃圾警报：",
            "😱 恐怖发现：",
            "🤮 令人作呕：",
        ];

        let prefix = savage_prefixes[message.len() % savage_prefixes.len()];
        format!("{} {}", prefix, message)
    }

    fn print_summary_with_score(&self, issues: &[CodeIssue], quality_score: &CodeQualityScore) {
        let _nuclear_count = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let total_count = issues.len();

        println!("{}", self.i18n.get("summary").bright_white().bold());
        println!("{}", "─".repeat(50).bright_black());

        // 显示评分总结
        let score_summary = match quality_score.quality_level {
            crate::scoring::QualityLevel::Excellent => {
                match self.i18n.lang.as_str() {
                    "zh-CN" => format!("🏆 代码质量优秀！评分: {:.1}/100", quality_score.total_score),
                    _ => format!("🏆 Excellent code quality! Score: {:.1}/100", quality_score.total_score),
                }
            },
            crate::scoring::QualityLevel::Good => {
                match self.i18n.lang.as_str() {
                    "zh-CN" => format!("👍 代码质量良好，评分: {:.1}/100", quality_score.total_score),
                    _ => format!("👍 Good code quality, Score: {:.1}/100", quality_score.total_score),
                }
            },
            crate::scoring::QualityLevel::Average => {
                match self.i18n.lang.as_str() {
                    "zh-CN" => format!("😐 代码质量一般，评分: {:.1}/100，还有改进空间", quality_score.total_score),
                    _ => format!("😐 Average code quality, Score: {:.1}/100, room for improvement", quality_score.total_score),
                }
            },
            crate::scoring::QualityLevel::Poor => {
                match self.i18n.lang.as_str() {
                    "zh-CN" => format!("😟 代码质量较差，评分: {:.1}/100，建议重构", quality_score.total_score),
                    _ => format!("😟 Poor code quality, Score: {:.1}/100, refactoring recommended", quality_score.total_score),
                }
            },
            crate::scoring::QualityLevel::Terrible => {
                match self.i18n.lang.as_str() {
                    "zh-CN" => format!("💀 代码质量糟糕，评分: {:.1}/100，急需重写", quality_score.total_score),
                    _ => format!("💀 Terrible code quality, Score: {:.1}/100, rewrite urgently needed", quality_score.total_score),
                }
            },
        };

        let score_color = match quality_score.quality_level {
            crate::scoring::QualityLevel::Excellent => score_summary.bright_green().bold(),
            crate::scoring::QualityLevel::Good => score_summary.green(),
            crate::scoring::QualityLevel::Average => score_summary.yellow(),
            crate::scoring::QualityLevel::Poor => score_summary.red(),
            crate::scoring::QualityLevel::Terrible => score_summary.bright_red().bold(),
        };

        println!("{}", score_color);
        println!();

        // 原有的总结逻辑
        let _nuclear_count = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let _total_count = issues.len();

        println!("{}", self.i18n.get("summary").bright_white().bold());
        println!("{}", "─".repeat(50).bright_black());

        let summary_message = if _nuclear_count > 0 {
            if self.savage_mode {
                match self.i18n.lang.as_str() {
                    "zh-CN" => "你的代码质量堪忧，建议重新学习编程基础 💀".to_string(),
                    _ => "Your code quality is concerning, suggest learning programming basics again 💀".to_string(),
                }
            } else {
                match self.i18n.lang.as_str() {
                    "zh-CN" => "发现了一些严重问题，建议优先修复核弹级问题 🔥".to_string(),
                    _ => "Found some serious issues, suggest fixing nuclear problems first 🔥"
                        .to_string(),
                }
            }
        } else if total_count > 10 {
            match self.i18n.lang.as_str() {
                "zh-CN" => "问题有点多，建议分批修复 📝".to_string(),
                _ => "Quite a few issues, suggest fixing them in batches 📝".to_string(),
            }
        } else {
            match self.i18n.lang.as_str() {
                "zh-CN" => "问题不多，稍微改进一下就好了 👍".to_string(),
                _ => "Not many issues, just need some minor improvements 👍".to_string(),
            }
        };

        let color = if _nuclear_count > 0 {
            summary_message.red().bold()
        } else if _total_count > 10 {
            summary_message.yellow()
        } else {
            summary_message.green()
        };

        println!("{}", color);
    }

    fn print_footer(&self, issues: &[CodeIssue]) {
        println!();
        println!("{}", self.i18n.get("suggestions").bright_cyan().bold());
        println!("{}", "─".repeat(50).bright_black());

        let rule_names: Vec<String> = issues
            .iter()
            .map(|issue| issue.rule_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let suggestions = self.i18n.get_suggestions(&rule_names);
        for suggestion in suggestions {
            println!("   {}", suggestion.cyan());
        }

        println!();
        let footer_message = if self.savage_mode {
            match self.i18n.lang.as_str() {
                "zh-CN" => "记住：写垃圾代码容易，写好代码需要用心 💪".to_string(),
                _ => "Remember: writing garbage code is easy, writing good code requires effort 💪"
                    .to_string(),
            }
        } else {
            self.i18n.get("keep_improving")
        };

        let color = if self.savage_mode {
            footer_message.bright_red().bold()
        } else {
            footer_message.bright_green().bold()
        };

        println!("{}", color);
    }

    fn print_top_files(&self, issues: &[CodeIssue]) {
        if self.top_files == 0 {
            return;
        }

        let mut file_issue_counts: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            let file_name = issue
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            *file_issue_counts.entry(file_name).or_insert(0) += 1;
        }

        let mut sorted_files: Vec<_> = file_issue_counts.into_iter().collect();
        sorted_files.sort_by(|a, b| b.1.cmp(&a.1));

        if !sorted_files.is_empty() {
            println!("{}", self.i18n.get("top_files").bright_yellow().bold());
            println!("{}", "─".repeat(50).bright_black());

            for (i, (file_name, count)) in sorted_files.iter().take(self.top_files).enumerate() {
                let rank = format!("{}.", i + 1);
                println!(
                    "   {} {} ({} issues)",
                    rank.bright_white(),
                    file_name.bright_blue(),
                    count.to_string().red()
                );
            }
            println!();
        }
    }

    fn print_detailed_analysis(&self, issues: &[CodeIssue]) {
        println!(
            "{}",
            self.i18n.get("detailed_analysis").bright_magenta().bold()
        );
        println!("{}", "─".repeat(50).bright_black());

        let mut rule_stats: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            *rule_stats.entry(issue.rule_name.clone()).or_insert(0) += 1;
        }

        let rule_descriptions = match self.i18n.lang.as_str() {
            "zh-CN" => [
                ("terrible-naming", "糟糕的变量命名"),
                ("single-letter-variable", "单字母变量"),
                ("deep-nesting", "过度嵌套"),
                ("long-function", "超长函数"),
                ("unwrap-abuse", "unwrap() 滥用"),
                ("unnecessary-clone", "不必要的 clone()"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
            _ => [
                ("terrible-naming", "Terrible variable naming"),
                ("single-letter-variable", "Single letter variables"),
                ("deep-nesting", "Deep nesting"),
                ("long-function", "Long functions"),
                ("unwrap-abuse", "unwrap() abuse"),
                ("unnecessary-clone", "Unnecessary clone()"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        };

        for (rule_name, count) in rule_stats {
            let rule_name_str = rule_name.as_str();
            let description = rule_descriptions
                .get(rule_name_str)
                .unwrap_or(&rule_name_str);
            println!(
                "   📌 {}: {} issues",
                description.cyan(),
                count.to_string().yellow()
            );
        }
        println!();
    }

    fn print_markdown_report(&self, issues: &[CodeIssue]) {
        let total = issues.len();
        let nuclear = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let spicy = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Spicy))
            .count();
        let mild = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Mild))
            .count();

        println!("# {}", self.i18n.get("title"));
        println!();
        println!("## {}", self.i18n.get("statistics"));
        println!();
        println!("| Severity | Count | Description |");
        println!("| --- | --- | --- |");
        println!(
            "| 🔥 Nuclear | {} | {} |",
            nuclear,
            self.i18n.get("nuclear_issues")
        );
        println!(
            "| 🌶️ Spicy | {} | {} |",
            spicy,
            self.i18n.get("spicy_issues")
        );
        println!("| 😐 Mild | {} | {} |", mild, self.i18n.get("mild_issues"));
        println!(
            "| **Total** | **{}** | **{}** |",
            total,
            self.i18n.get("total")
        );
        println!();

        if self.verbose {
            println!("## {}", self.i18n.get("detailed_analysis"));
            println!();

            let mut rule_stats: HashMap<String, usize> = HashMap::new();
            for issue in issues {
                *rule_stats.entry(issue.rule_name.clone()).or_insert(0) += 1;
            }

            for (rule_name, count) in rule_stats {
                println!("- **{}**: {} issues", rule_name, count);
            }
            println!();
        }

        println!("## Issues by File");
        println!();

        let mut file_groups: HashMap<String, Vec<&CodeIssue>> = HashMap::new();
        for issue in issues {
            let file_name = issue
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            file_groups.entry(file_name).or_default().push(issue);
        }

        for (file_name, file_issues) in file_groups {
            println!("### 📁 {}", file_name);
            println!();

            let issues_to_show = if self.max_issues_per_file > 0 {
                file_issues
                    .into_iter()
                    .take(self.max_issues_per_file)
                    .collect::<Vec<_>>()
            } else {
                file_issues
            };

            for issue in issues_to_show {
                let severity_icon = match issue.severity {
                    Severity::Nuclear => "💥",
                    Severity::Spicy => "🌶️",
                    Severity::Mild => "😐",
                };

                let messages = self.i18n.get_roast_messages(&issue.rule_name);
                let message = if !messages.is_empty() {
                    messages[issue.line % messages.len()].clone()
                } else {
                    issue.message.clone()
                };

                println!(
                    "- {} **Line {}:{}** - {}",
                    severity_icon, issue.line, issue.column, message
                );
            }
            println!();
        }

        println!("## {}", self.i18n.get("suggestions"));
        println!();

        let rule_names: Vec<String> = issues
            .iter()
            .map(|issue| issue.rule_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let suggestions = self.i18n.get_suggestions(&rule_names);
        for suggestion in suggestions {
            println!("- {}", suggestion);
        }
    }
}
