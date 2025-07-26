#[allow(dead_code)]
use colored::*;
use std::collections::HashMap;

use crate::analyzer::{CodeIssue, Severity};
use crate::i18n::I18n;
use crate::scoring::{CodeQualityScore, CodeScorer};

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

    pub fn report_with_metrics(
        &self,
        mut issues: Vec<CodeIssue>,
        file_count: usize,
        total_lines: usize,
    ) {
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
            println!(
                "**评分**: {:.1}/100 {}",
                quality_score.total_score,
                quality_score.quality_level.emoji()
            );
            println!(
                "**等级**: {}",
                quality_score.quality_level.description(&self.i18n.lang)
            );
            println!();
            println!("{}", self.i18n.get("clean_code"));
            println!();
            println!("{}", self.i18n.get("clean_code_warning"));
        } else {
            println!("{}", self.i18n.get("clean_code").bright_green().bold());
            println!();
            println!(
                "{} 代码质量评分: {:.1}/100 {}",
                "🏆".bright_yellow(),
                quality_score.total_score.to_string().bright_green().bold(),
                quality_score.quality_level.emoji()
            );
            println!(
                "{} 质量等级: {}",
                "📊".bright_blue(),
                quality_score
                    .quality_level
                    .description(&self.i18n.lang)
                    .bright_green()
                    .bold()
            );
            println!("{}", self.i18n.get("clean_code_warning").yellow());
        }
    }

    fn print_quality_score(&self, quality_score: &CodeQualityScore) {
        let title = match self.i18n.lang.as_str() {
            "zh-CN" => "🏆 代码质量评分",
            _ => "🏆 Code Quality Score",
        };
        println!("{}", title.bright_yellow().bold());
        println!("{}", "─".repeat(50).bright_black());

        let _score_color = match quality_score.quality_level {
            crate::scoring::QualityLevel::Excellent => {
                quality_score.total_score.to_string().bright_green().bold()
            }
            crate::scoring::QualityLevel::Good => quality_score.total_score.to_string().green(),
            crate::scoring::QualityLevel::Average => quality_score.total_score.to_string().yellow(),
            crate::scoring::QualityLevel::Poor => quality_score.total_score.to_string().red(),
            crate::scoring::QualityLevel::Terrible => {
                quality_score.total_score.to_string().bright_red().bold()
            }
        };

        let (score_label, level_label) = match self.i18n.lang.as_str() {
            "zh-CN" => ("📊 总分", "🎯 等级"),
            _ => ("📊 Score", "🎯 Level"),
        };

        println!(
            "   {}: {:.1}/100 {}",
            score_label,
            quality_score.total_score,
            quality_score.quality_level.emoji()
        );
        println!(
            "   {}: {}",
            level_label,
            quality_score
                .quality_level
                .description(&self.i18n.lang)
                .bright_white()
                .bold()
        );

        if quality_score.total_lines > 0 {
            let (lines_label, files_label, density_label) = match self.i18n.lang.as_str() {
                "zh-CN" => ("📏 代码行数", "📁 文件数量", "🔍 问题密度"),
                _ => ("📏 Lines of Code", "📁 Files", "🔍 Issue Density"),
            };
            let density_unit = match self.i18n.lang.as_str() {
                "zh-CN" => "问题/千行",
                _ => "issues/1k lines",
            };

            println!(
                "   {}: {}",
                lines_label,
                quality_score.total_lines.to_string().cyan()
            );
            println!(
                "   {}: {}",
                files_label,
                quality_score.file_count.to_string().cyan()
            );
            println!(
                "   {}: {:.2} {}",
                density_label,
                quality_score.issue_density.to_string().cyan(),
                density_unit
            );
        }

        // 显示严重程度分布
        if quality_score.severity_distribution.nuclear > 0
            || quality_score.severity_distribution.spicy > 0
            || quality_score.severity_distribution.mild > 0
        {
            println!();
            let distribution_title = match self.i18n.lang.as_str() {
                "zh-CN" => "🎭 问题分布:",
                _ => "🎭 Issue Distribution:",
            };
            let (nuclear_label, spicy_label, mild_label) = match self.i18n.lang.as_str() {
                "zh-CN" => ("💥 核弹级", "🌶️  严重", "😐 轻微"),
                _ => ("💥 Nuclear", "🌶️  Spicy", "😐 Mild"),
            };

            println!("   {}", distribution_title);
            if quality_score.severity_distribution.nuclear > 0 {
                println!(
                    "      {}: {}",
                    nuclear_label,
                    quality_score
                        .severity_distribution
                        .nuclear
                        .to_string()
                        .red()
                        .bold()
                );
            }
            if quality_score.severity_distribution.spicy > 0 {
                println!(
                    "      {}: {}",
                    spicy_label,
                    quality_score
                        .severity_distribution
                        .spicy
                        .to_string()
                        .yellow()
                );
            }
            if quality_score.severity_distribution.mild > 0 {
                println!(
                    "      {}: {}",
                    mild_label,
                    quality_score.severity_distribution.mild.to_string().blue()
                );
            }
        }

        // 显示分类得分（如果有的话）
        if !quality_score.category_scores.is_empty() && self.verbose {
            println!();
            let category_title = match self.i18n.lang.as_str() {
                "zh-CN" => "📋 分类得分:",
                _ => "📋 Category Scores:",
            };
            println!("   {}", category_title);
            let mut sorted_categories: Vec<_> = quality_score.category_scores.iter().collect();
            sorted_categories
                .sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (category, score) in sorted_categories.iter().take(5) {
                let category_name = match (self.i18n.lang.as_str(), category.as_str()) {
                    ("zh-CN", "naming") => "命名规范",
                    ("zh-CN", "complexity") => "复杂度",
                    ("zh-CN", "rust-basics") => "Rust基础",
                    ("zh-CN", "advanced-rust") => "高级特性",
                    ("zh-CN", "rust-features") => "Rust功能",
                    ("zh-CN", "structure") => "代码结构",
                    ("zh-CN", "duplication") => "重复代码",
                    (_, "naming") => "Naming",
                    (_, "complexity") => "Complexity",
                    (_, "rust-basics") => "Rust Basics",
                    (_, "advanced-rust") => "Advanced Rust",
                    (_, "rust-features") => "Rust Features",
                    (_, "structure") => "Code Structure",
                    (_, "duplication") => "Code Duplication",
                    _ => category,
                };
                println!(
                    "      {} {:.1}",
                    category_name.cyan(),
                    score.to_string().yellow()
                );
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

            // Group issues by rule type
            let mut rule_groups: HashMap<String, Vec<&CodeIssue>> = HashMap::new();
            for issue in &file_issues {
                rule_groups
                    .entry(issue.rule_name.clone())
                    .or_default()
                    .push(issue);
            }

            // Show limited number of issues per rule type
            let _max_per_rule = 5;
            let mut total_shown = 0;
            let max_total = if self.max_issues_per_file > 0 {
                self.max_issues_per_file
            } else {
                usize::MAX
            };

            // Sort rule groups by severity (most severe first)
            let mut sorted_rules: Vec<_> = rule_groups.into_iter().collect();
            sorted_rules.sort_by(|a, b| {
                let severity_order = |s: &Severity| match s {
                    Severity::Nuclear => 3,
                    Severity::Spicy => 2,
                    Severity::Mild => 1,
                };
                let max_severity_a =
                    a.1.iter()
                        .map(|i| severity_order(&i.severity))
                        .max()
                        .unwrap_or(1);
                let max_severity_b =
                    b.1.iter()
                        .map(|i| severity_order(&i.severity))
                        .max()
                        .unwrap_or(1);
                max_severity_b.cmp(&max_severity_a)
            });

            for (rule_name, rule_issues) in sorted_rules {
                if total_shown >= max_total {
                    break;
                }

                let rule_issues_len = rule_issues.len();

                // Create compact summary for each rule type
                if rule_name.contains("naming") || rule_name.contains("single-letter") {
                    // Collect variable names for naming issues
                    let bad_names: Vec<String> = rule_issues
                        .iter()
                        .filter_map(|issue| {
                            if let Some(start) = issue.message.find("'") {
                                if let Some(end) = issue.message[start + 1..].find("'") {
                                    Some(issue.message[start + 1..start + 1 + end].to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .take(5)
                        .collect();

                    let names_display = if bad_names.len() < rule_issues_len {
                        format!("{}, ...", bad_names.join(", "))
                    } else {
                        bad_names.join(", ")
                    };

                    let label = if self.i18n.lang == "zh-CN" {
                        "变量命名问题"
                    } else {
                        "Variable naming issues"
                    };

                    println!(
                        "  🏷️ {}: {} ({})",
                        label.bright_yellow().bold(),
                        rule_issues_len.to_string().bright_red().bold(),
                        names_display.bright_black()
                    );
                    total_shown += 1;
                } else if rule_name.contains("duplication") {
                    let label = if self.i18n.lang == "zh-CN" {
                        "代码重复问题"
                    } else {
                        "Code duplication issues"
                    };

                    // Extract instance count from message if available
                    let instance_info = if let Some(first_issue) = rule_issues.first() {
                        if first_issue.message.contains("instances") {
                            let parts: Vec<&str> = first_issue.message.split_whitespace().collect();
                            if let Some(pos) = parts.iter().position(|&x| x == "instances") {
                                if pos > 0 {
                                    format!("{} instances", parts[pos - 1])
                                } else {
                                    "multiple instances".to_string()
                                }
                            } else {
                                "multiple blocks".to_string()
                            }
                        } else {
                            "multiple blocks".to_string()
                        }
                    } else {
                        "multiple blocks".to_string()
                    };

                    println!(
                        "  🔄 {}: {} ({})",
                        label.bright_cyan().bold(),
                        rule_issues_len.to_string().bright_cyan().bold(),
                        instance_info.bright_black()
                    );
                    total_shown += 1;
                } else if rule_name.contains("nesting") {
                    let label = if self.i18n.lang == "zh-CN" {
                        "嵌套深度问题"
                    } else {
                        "Nesting depth issues"
                    };

                    // Extract depth range from messages
                    let depths: Vec<usize> = rule_issues
                        .iter()
                        .filter_map(|issue| {
                            if let Some(start) = issue.message.find("depth: ") {
                                let depth_str = &issue.message[start + 7..];
                                if let Some(end) = depth_str.find(')') {
                                    depth_str[..end].parse().ok()
                                } else {
                                    None
                                }
                            } else if let Some(start) = issue.message.find("深度: ") {
                                let depth_str = &issue.message[start + 6..];
                                if let Some(end) = depth_str.find(')') {
                                    depth_str[..end].parse().ok()
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    let depth_info = if !depths.is_empty() {
                        let min_depth = depths.iter().min().unwrap_or(&4);
                        let max_depth = depths.iter().max().unwrap_or(&8);
                        if min_depth == max_depth {
                            format!("depth {}", min_depth)
                        } else {
                            format!("depth {}-{}", min_depth, max_depth)
                        }
                    } else {
                        "deep nesting".to_string()
                    };

                    println!(
                        "  📦 {}: {} ({})",
                        label.bright_magenta().bold(),
                        rule_issues_len.to_string().bright_magenta().bold(),
                        depth_info.bright_black()
                    );
                    total_shown += 1;
                } else {
                    // For other types, show a generic summary
                    let clean_rule_name = rule_name.replace("-", " ");
                    println!(
                        "  ⚠️ {}: {}",
                        clean_rule_name.bright_yellow().bold(),
                        rule_issues_len.to_string().bright_yellow().bold()
                    );
                    total_shown += 1;
                }
            }
            println!();
        }
    }

    fn print_issue(&self, issue: &CodeIssue) {
        // Choose icon and color based on rule type
        if issue.rule_name.contains("duplication") {
            let message = if self.i18n.lang == "zh-CN" {
                &issue.message
            } else {
                // Translate common duplication messages to English
                if issue.message.contains("相似代码块") {
                    "Found similar code blocks, consider refactoring into functions"
                } else if issue.message.contains("DRY原则哭了") {
                    "Code duplication detected, DRY principle violated"
                } else {
                    &issue.message
                }
            };
            println!(
                "  {} {} {}",
                "🔄".bright_cyan(),
                "duplicate".bright_black(),
                message.bright_cyan().bold()
            );
        } else if issue.rule_name.contains("nesting") {
            let message = if self.i18n.lang == "zh-CN" {
                &issue.message
            } else {
                // Translate common nesting messages to English
                if issue.message.contains("俄罗斯套娃") {
                    "Nesting deeper than Russian dolls, are you writing a maze?"
                } else if issue.message.contains("挖到地心") {
                    "Nesting so deep, trying to dig to the Earth's core?"
                } else if issue.message.contains("像洋葱一样") {
                    "Code nested like an onion, makes me want to cry"
                } else {
                    &issue.message
                }
            };
            println!(
                "  {} {} {}",
                "📦".bright_magenta(),
                "nesting".bright_black(),
                message.bright_magenta()
            );
        } else {
            // Default based on severity
            let severity_icon = match issue.severity {
                Severity::Nuclear => "💥",
                Severity::Spicy => "🌶️",
                Severity::Mild => "😐",
            };

            let line_info = format!("{}:{}", issue.line, issue.column);
            let colored_message = match issue.severity {
                Severity::Nuclear => issue.message.red().bold(),
                Severity::Spicy => issue.message.yellow(),
                Severity::Mild => issue.message.blue(),
            };

            let _final_message = if self.savage_mode {
                self.make_message_savage(&issue.message)
            } else {
                issue.message.clone()
            };

            println!(
                "  {} {} {}",
                severity_icon.bright_yellow(),
                line_info.bright_black(),
                colored_message
            );
        }
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
        // Print detailed scoring breakdown
        self.print_scoring_breakdown(issues, quality_score);
        let _nuclear_count = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let _total_count = issues.len();

        println!("{}", self.i18n.get("summary").bright_white().bold());
        println!("{}", "─".repeat(50).bright_black());

        // 显示评分总结
        let score_summary = match quality_score.quality_level {
            crate::scoring::QualityLevel::Excellent => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "🏆 代码质量优秀！评分: {:.1}/100",
                    quality_score.total_score
                ),
                _ => format!(
                    "🏆 Excellent code quality! Score: {:.1}/100",
                    quality_score.total_score
                ),
            },
            crate::scoring::QualityLevel::Good => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "👍 代码质量良好，评分: {:.1}/100",
                    quality_score.total_score
                ),
                _ => format!(
                    "👍 Good code quality, Score: {:.1}/100",
                    quality_score.total_score
                ),
            },
            crate::scoring::QualityLevel::Average => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "😐 代码质量一般，评分: {:.1}/100，还有改进空间",
                    quality_score.total_score
                ),
                _ => format!(
                    "😐 Average code quality, Score: {:.1}/100, room for improvement",
                    quality_score.total_score
                ),
            },
            crate::scoring::QualityLevel::Poor => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "😟 代码质量较差，评分: {:.1}/100，建议重构",
                    quality_score.total_score
                ),
                _ => format!(
                    "😟 Poor code quality, Score: {:.1}/100, refactoring recommended",
                    quality_score.total_score
                ),
            },
            crate::scoring::QualityLevel::Terrible => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "💀 代码质量糟糕，评分: {:.1}/100，急需重写",
                    quality_score.total_score
                ),
                _ => format!(
                    "💀 Terrible code quality, Score: {:.1}/100, rewrite urgently needed",
                    quality_score.total_score
                ),
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

        let nuclear_count = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let total_count = issues.len();

        let summary_message = if nuclear_count > 0 {
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

        let color = if nuclear_count > 0 {
            summary_message.red().bold()
        } else if total_count > 10 {
            summary_message.yellow()
        } else {
            summary_message.green()
        };

        println!("{}", color);
    }

    fn print_scoring_breakdown(&self, _issues: &[CodeIssue], quality_score: &CodeQualityScore) {
        let title = if self.i18n.lang == "zh-CN" {
            "📊 评分详情"
        } else {
            "📊 Scoring Details"
        };

        println!("\n{}", title.bright_cyan().bold());
        println!("{}", "─".repeat(50).bright_black());

        // Show category scores
        self.print_category_scores(&quality_score.category_scores);

        // Show weighted calculation
        self.print_weighted_calculation(&quality_score.category_scores, quality_score.total_score);

        // Show scoring scale
        let scale_title = if self.i18n.lang == "zh-CN" {
            "\n📏 评分标准 (分数越高代码越烂):"
        } else {
            "\n📏 Scoring Scale (higher score = worse code):"
        };

        println!("{}", scale_title.bright_yellow());
        if self.i18n.lang == "zh-CN" {
            println!("  💀 81-100: 糟糕    🔥 61-80: 较差    ⚠️ 41-60: 一般");
            println!("  ✅ 21-40: 良好     🌟 0-20: 优秀");
        } else {
            println!("  💀 81-100: Terrible    🔥 61-80: Poor    ⚠️ 41-60: Average");
            println!("  ✅ 21-40: Good         🌟 0-20: Excellent");
        }
    }

    fn calculate_base_score_for_display(
        &self,
        issues: &[CodeIssue],
        scorer: &crate::scoring::CodeScorer,
    ) -> f64 {
        let mut score = 0.0;
        for issue in issues {
            let rule_weight = scorer.rule_weights.get(&issue.rule_name).unwrap_or(&1.0);
            let severity_weight = match issue.severity {
                crate::analyzer::Severity::Nuclear => 10.0,
                crate::analyzer::Severity::Spicy => 5.0,
                crate::analyzer::Severity::Mild => 2.0,
            };
            score += rule_weight * severity_weight;
        }
        score
    }

    fn calculate_density_penalty_for_display(
        &self,
        issue_count: usize,
        file_count: usize,
        total_lines: usize,
    ) -> f64 {
        if total_lines == 0 || file_count == 0 {
            return 0.0;
        }

        let issues_per_1000_lines = (issue_count as f64 / total_lines as f64) * 1000.0;
        let issues_per_file = issue_count as f64 / file_count as f64;

        let density_penalty = match issues_per_1000_lines {
            x if x > 50.0 => 25.0,
            x if x > 30.0 => 15.0,
            x if x > 20.0 => 10.0,
            x if x > 10.0 => 5.0,
            _ => 0.0,
        };

        let file_penalty = match issues_per_file {
            x if x > 20.0 => 15.0,
            x if x > 10.0 => 10.0,
            x if x > 5.0 => 5.0,
            _ => 0.0,
        };

        density_penalty + file_penalty
    }

    fn calculate_severity_penalty_for_display(
        &self,
        distribution: &crate::scoring::SeverityDistribution,
    ) -> f64 {
        let mut penalty = 0.0;

        if distribution.nuclear > 0 {
            penalty += 20.0 + (distribution.nuclear as f64 - 1.0) * 5.0;
        }

        if distribution.spicy > 5 {
            penalty += (distribution.spicy as f64 - 5.0) * 2.0;
        }

        if distribution.mild > 20 {
            penalty += (distribution.mild as f64 - 20.0) * 0.5;
        }

        penalty
    }

    fn print_category_scores(&self, category_scores: &std::collections::HashMap<String, f64>) {
        let title = if self.i18n.lang == "zh-CN" {
            "📋 分类评分详情:"
        } else {
            "📋 Category Scores:"
        };

        println!("{}", title.bright_yellow());

        // Define category display order and info
        let categories = [
            ("naming", "命名规范", "Naming", "🏷️"),
            ("complexity", "复杂度", "Complexity", "🧩"),
            ("duplication", "代码重复", "Duplication", "🔄"),
            ("rust-basics", "Rust基础", "Rust Basics", "🦀"),
            ("advanced-rust", "高级特性", "Advanced Rust", "⚡"),
            ("rust-features", "Rust功能", "Rust Features", "🚀"),
            ("structure", "代码结构", "Code Structure", "🏗️"),
        ];

        for (category_key, zh_name, en_name, icon) in &categories {
            if let Some(score) = category_scores.get(*category_key) {
                let display_name = if self.i18n.lang == "zh-CN" {
                    zh_name
                } else {
                    en_name
                };
                let (status_icon, status_text) = self.get_score_status(*score);

                // basic display
                println!(
                    "  {} {} {}分     {}",
                    status_icon,
                    format!("{} {}", icon, display_name).bright_white(),
                    format!("{:.0}", score).bright_cyan(),
                    status_text.bright_black()
                );

                // if score is high (code is bad), add a roast
                if let Some(roast) = self.get_category_roast(category_key, *score) {
                    println!("    💬 {}", roast.bright_yellow().italic());
                }
            }
        }
        println!();
    }

    fn get_score_status(&self, score: f64) -> (&str, &str) {
        // 注意：分数越高代码越烂
        match score as u32 {
            81..=100 => (
                "⚠",
                if self.i18n.lang == "zh-CN" {
                    "糟糕，急需修复"
                } else {
                    "Terrible, urgent fixes needed"
                },
            ),
            61..=80 => (
                "•",
                if self.i18n.lang == "zh-CN" {
                    "较差，建议重构"
                } else {
                    "Poor, refactoring recommended"
                },
            ),
            41..=60 => (
                "○",
                if self.i18n.lang == "zh-CN" {
                    "一般，需要改进"
                } else {
                    "Average, needs improvement"
                },
            ),
            21..=40 => (
                "✓",
                if self.i18n.lang == "zh-CN" {
                    "良好，还有提升空间"
                } else {
                    "Good, room for improvement"
                },
            ),
            _ => (
                "✓✓",
                if self.i18n.lang == "zh-CN" {
                    "优秀，继续保持"
                } else {
                    "Excellent, keep it up"
                },
            ),
        }
    }

    fn get_category_roast(&self, category: &str, score: f64) -> Option<String> {
        // only roast if score is high (code is bad)
        if score < 60.0 {
            return None;
        }

        let roasts = match (self.i18n.lang.as_str(), category) {
            ("zh-CN", "naming") => vec![
                "变量命名比我的编程技能还要抽象 🤔",
                "这些变量名让维护者想哭着辞职 😭",
                "变量名的创意程度约等于给孩子起名叫'小明' 🙄",
                "恭喜！你成功让变量名比注释还难懂 🏆",
            ],
            ("zh-CN", "complexity") => vec![
                "这复杂度比俄罗斯套娃还要深 🪆",
                "代码复杂得像洋葱一样，剥一层哭一次 🧅",
                "这函数比我的人际关系还复杂 😵‍💫",
                "复杂度爆表！连AI都看不懂了 🤖",
            ],
            ("zh-CN", "duplication") => vec![
                "检测到重复代码！你是复制粘贴大师吗？ 🥷",
                "DRY原则哭了，你的代码湿得像雨季 🌧️",
                "这些重复代码比双胞胎还像 👯‍♀️",
                "建议改名为copy-paste.rs 📋",
            ],
            ("zh-CN", "rust-features") => vec![
                "宏定义比我的借口还多 🎭",
                "这么多宏，IDE都要罢工了 💻",
                "宏滥用！编译时间都被你搞长了 ⏰",
            ],
            ("en-US", "naming") => vec![
                "Variable names more abstract than modern art 🎨",
                "These names make maintainers want to quit and sell hotdogs 🌭",
                "Variable naming creativity level: calling a kid 'Child' 👶",
                "Congrats! Variables harder to understand than comments 🏆",
            ],
            ("en-US", "complexity") => vec![
                "Complexity deeper than Russian dolls 🪆",
                "Code nested like an onion, peel one layer, cry once 🧅",
                "This function is more complex than my relationships 😵‍💫",
                "Complexity off the charts! Even AI gave up 🤖",
            ],
            ("en-US", "duplication") => vec![
                "Copy-paste ninja detected! 🥷",
                "DRY principle crying while your code drowns in repetition 🌧️",
                "More duplicates than a hall of mirrors 🪞",
                "Suggest renaming to ctrl-c-ctrl-v.rs 📋",
            ],
            ("en-US", "rust-features") => vec![
                "More macros than my excuses 🎭",
                "So many macros, even the IDE wants to quit 💻",
                "Macro abuse! Compile time extended indefinitely ⏰",
            ],
            _ => vec![],
        };

        if roasts.is_empty() {
            None
        } else {
            // select roast based on score (the higher the score, the more severe the roast)
            let index = ((score - 60.0) / 10.0) as usize;
            let roast_index = index.min(roasts.len() - 1);
            Some(roasts[roast_index].to_string())
        }
    }

    fn print_weighted_calculation(
        &self,
        category_scores: &std::collections::HashMap<String, f64>,
        _total_score: f64,
    ) {
        let calc_title = if self.i18n.lang == "zh-CN" {
            "🧮 加权计算:"
        } else {
            "🧮 Weighted Calculation:"
        };

        println!("{}", calc_title.bright_yellow());

        // Show the calculation formula
        let weights = [
            ("naming", 0.25, "命名规范", "Naming"),
            ("complexity", 0.20, "复杂度", "Complexity"),
            ("duplication", 0.15, "代码重复", "Duplication"),
            ("rust-basics", 0.15, "Rust基础", "Rust Basics"),
            ("advanced-rust", 0.10, "高级特性", "Advanced Rust"),
            ("rust-features", 0.10, "Rust功能", "Rust Features"),
            ("structure", 0.05, "代码结构", "Code Structure"),
        ];

        let mut calculation_parts = Vec::new();
        let mut weighted_sum = 0.0;

        for (category_key, weight, _zh_name, _en_name) in &weights {
            if let Some(score) = category_scores.get(*category_key) {
                let weighted_value = score * weight;
                weighted_sum += weighted_value;
                calculation_parts.push(format!("{:.1}×{:.2}", score, weight));
            }
        }

        if self.i18n.lang == "zh-CN" {
            println!(
                "  评分计算: ({}) ÷ 1.00 = {}",
                calculation_parts.join(" + ").bright_white(),
                format!("{:.1}", weighted_sum).bright_green().bold()
            );
        } else {
            println!(
                "  Score calculation: ({}) ÷ 1.00 = {}",
                calculation_parts.join(" + ").bright_white(),
                format!("{:.1}", weighted_sum).bright_green().bold()
            );
        }
    }

    fn print_detailed_base_score_breakdown(
        &self,
        issues: &[CodeIssue],
        scorer: &crate::scoring::CodeScorer,
    ) {
        // Group issues by rule type and calculate scores
        let mut rule_scores: std::collections::HashMap<String, (usize, f64)> =
            std::collections::HashMap::new();

        for issue in issues {
            let rule_weight = scorer.rule_weights.get(&issue.rule_name).unwrap_or(&1.0);
            let severity_weight = match issue.severity {
                crate::analyzer::Severity::Nuclear => 10.0,
                crate::analyzer::Severity::Spicy => 5.0,
                crate::analyzer::Severity::Mild => 2.0,
            };
            let issue_score = rule_weight * severity_weight;

            let entry = rule_scores
                .entry(issue.rule_name.clone())
                .or_insert((0, 0.0));
            entry.0 += 1; // count
            entry.1 += issue_score; // total score
        }

        // Sort by score (highest first)
        let mut sorted_rules: Vec<_> = rule_scores.into_iter().collect();
        sorted_rules.sort_by(|a, b| {
            b.1 .1
                .partial_cmp(&a.1 .1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let breakdown_title = if self.i18n.lang == "zh-CN" {
            "🔍 基础分数详细计算:"
        } else {
            "🔍 Base score detailed calculation:"
        };

        println!("{}", breakdown_title.bright_yellow());

        for (rule_name, (count, total_score)) in sorted_rules.iter().take(10) {
            let rule_weight = scorer.rule_weights.get(rule_name).unwrap_or(&1.0);

            let rule_display = match (self.i18n.lang.as_str(), rule_name.as_str()) {
                ("zh-CN", "terrible-naming") => "糟糕命名",
                ("zh-CN", "single-letter-variable") => "单字母变量",
                ("zh-CN", "deep-nesting") => "深度嵌套",
                ("zh-CN", "code-duplication") => "代码重复",
                ("zh-CN", "long-function") => "超长函数",
                ("zh-CN", "macro-abuse") => "宏滥用",
                (_, "terrible-naming") => "Terrible naming",
                (_, "single-letter-variable") => "Single letter vars",
                (_, "deep-nesting") => "Deep nesting",
                (_, "code-duplication") => "Code duplication",
                (_, "long-function") => "Long function",
                (_, "macro-abuse") => "Macro abuse",
                _ => rule_name,
            };

            if self.i18n.lang == "zh-CN" {
                println!(
                    "  • {} × {} (权重{:.1}) = {}",
                    format!("{}", count).cyan(),
                    rule_display.bright_white(),
                    format!("{:.1}", rule_weight).yellow(),
                    format!("{:.1}", total_score).bright_red()
                );
            } else {
                println!(
                    "  • {} × {} (weight {:.1}) = {}",
                    format!("{}", count).cyan(),
                    rule_display.bright_white(),
                    format!("{:.1}", rule_weight).yellow(),
                    format!("{:.1}", total_score).bright_red()
                );
            }
        }
        println!();
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
