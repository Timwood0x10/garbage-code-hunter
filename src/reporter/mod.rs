mod display;

use colored::*;
use std::collections::{BTreeMap, HashMap};

use crate::analyzer::{CodeIssue, Severity};
use crate::i18n::I18n;
use crate::llm::{RoastMap, RoastProvider};
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
    roast_provider: Box<dyn RoastProvider>,
}

impl Reporter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        harsh_mode: bool,
        savage_mode: bool,
        verbose: bool,
        top_files: usize,
        max_issues_per_file: usize,
        summary_only: bool,
        markdown: bool,
        lang: &str,
        roast_provider: Box<dyn RoastProvider>,
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
            roast_provider,
        }
    }

    pub fn report_with_metrics(
        &self,
        mut issues: Vec<CodeIssue>,
        file_count: usize,
        total_lines: usize,
    ) {
        // calculate quality score
        let scorer = CodeScorer::new();
        let quality_score = scorer.calculate_score(&issues, file_count, total_lines);

        if issues.is_empty() {
            self.print_clean_code_message_with_score(&quality_score);
            return;
        }

        //sort by severity
        issues.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Nuclear => 3,
                Severity::Spicy => 2,
                Severity::Mild => 1,
            };
            severity_order(&b.severity).cmp(&severity_order(&a.severity))
        });

        // if harsh mode  only show the most severe issue
        if self.harsh_mode {
            issues.retain(|issue| matches!(issue.severity, Severity::Nuclear | Severity::Spicy));
        }

        // Generate roasts via provider (one call for all issues)
        let roasts = self
            .roast_provider
            .generate_roasts(&issues, &self.i18n.lang);

        if self.markdown {
            self.print_markdown_report(&issues, &roasts);
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
            let mut rule_groups: BTreeMap<String, Vec<&CodeIssue>> = BTreeMap::new();
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
                                issue.message[start + 1..].find("'").map(|end| {
                                    issue.message[start + 1..start + 1 + end].to_string()
                                })
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
                                if self.i18n.lang == "zh-CN" {
                                    "多个代码块".to_string()
                                } else {
                                    "multiple blocks".to_string()
                                }
                            }
                        } else {
                            if self.i18n.lang == "zh-CN" {
                                "多个代码块".to_string()
                            } else {
                                "multiple blocks".to_string()
                            }
                        }
                    } else {
                        if self.i18n.lang == "zh-CN" {
                            "多个代码块".to_string()
                        } else {
                            "multiple blocks".to_string()
                        }
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
                            format!("depth {min_depth}")
                        } else {
                            format!("depth {min_depth}-{max_depth}")
                        }
                    } else {
                        if self.i18n.lang == "zh-CN" {
                            "深度嵌套".to_string()
                        } else {
                            "deep nesting".to_string()
                        }
                    };

                    println!(
                        "  📦 {}: {} ({})",
                        label.bright_magenta().bold(),
                        rule_issues_len.to_string().bright_magenta().bold(),
                        depth_info.bright_black()
                    );
                    total_shown += 1;
                } else {
                    // For other types, show a generic summary with proper translation
                    let display_name = match (self.i18n.lang.as_str(), rule_name.as_str()) {
                        ("zh-CN", "panic-abuse") => "panic 滥用",
                        ("zh-CN", "god-function") => "上帝函数",
                        ("zh-CN", "magic-number") => "魔法数字",
                        ("zh-CN", "todo-comment") => "TODO 注释",
                        ("zh-CN", "println-debugging") => "println 调试",
                        ("zh-CN", "string-abuse") => "String 滥用",
                        ("zh-CN", "vec-abuse") => "Vec 滥用",
                        ("zh-CN", "iterator-abuse") => "迭代器滥用",
                        ("zh-CN", "match-abuse") => "Match 滥用",
                        ("zh-CN", "hungarian-notation") => "匈牙利命名法",
                        ("zh-CN", "abbreviation-abuse") => "过度缩写",
                        ("zh-CN", "meaningless-naming") => "无意义命名",
                        ("zh-CN", "commented-code") => "被注释代码",
                        ("zh-CN", "dead-code") => "死代码",
                        _ => &rule_name.replace("-", " "),
                    };
                    println!(
                        "  ⚠️ {}: {}",
                        display_name.bright_yellow().bold(),
                        rule_issues_len.to_string().bright_yellow().bold()
                    );
                    total_shown += 1;
                }
            }
            println!();
        }
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

        println!("{color}");
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
        sorted_files.sort_by_key(|b| std::cmp::Reverse(b.1));

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

            // Get the display name for the rule
            let display_name = if self.i18n.lang == "zh-CN" {
                match rule_name_str {
                    "terrible-naming" => "糟糕的变量命名",
                    "single-letter-variable" => "单字母变量",
                    "deep-nesting" => "过度嵌套",
                    "long-function" => "超长函数",
                    "unwrap-abuse" => "unwrap() 滥用",
                    "unnecessary-clone" => "不必要的 clone()",
                    "panic-abuse" => "panic 滥用",
                    "god-function" => "上帝函数",
                    "magic-number" => "魔法数字",
                    "todo-comment" => "TODO 注释",
                    "println-debugging" => "println 调试",
                    "string-abuse" => "String 滥用",
                    "vec-abuse" => "Vec 滥用",
                    "iterator-abuse" => "迭代器滥用",
                    "match-abuse" => "Match 滥用",
                    "hungarian-notation" => "匈牙利命名法",
                    "abbreviation-abuse" => "过度缩写",
                    "meaningless-naming" => "无意义命名",
                    "commented-code" => "被注释代码",
                    "dead-code" => "死代码",
                    "code-duplication" => "代码重复",
                    "macro-abuse" => "宏滥用",
                    _ => rule_name_str,
                }
            } else {
                rule_descriptions
                    .get(rule_name_str)
                    .unwrap_or(&rule_name_str)
            };

            let issues_text = if self.i18n.lang == "zh-CN" {
                "个问题"
            } else {
                "issues"
            };

            println!(
                "   📌 {}: {} {}",
                display_name.cyan(),
                count.to_string().yellow(),
                issues_text
            );
        }
        println!();
    }

    fn print_markdown_report(&self, issues: &[CodeIssue], roasts: &RoastMap) {
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

        let mut file_groups: BTreeMap<String, Vec<&CodeIssue>> = BTreeMap::new();
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

                let key = format!(
                    "{}:{}:{}",
                    issue.file_path.display(),
                    issue.line,
                    issue.rule_name
                );
                let message = roasts
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| issue.message.clone());

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
