mod display;

use colored::*;
use std::collections::{BTreeMap, HashMap};

use std::path::Path;

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

    fn is_test_path(path: &Path) -> bool {
        let name = path.to_string_lossy();
        name.contains("/tests/")
            || name.contains("/test/")
            || name.ends_with("_test.rs")
            || name.ends_with("_tests.rs")
            || name.ends_with("_test.go")
            || name.ends_with("_test.py")
            || name.ends_with("_test.js")
            || name.ends_with("_test.ts")
            || name.ends_with("_test.java")
            || name.starts_with("test_")
            || name.contains("/test-files/")
            || name.contains("/fixtures/")
            || name.contains("/mocks/")
            || name.contains("/examples/")
            || name.contains("/benches/")
    }

    pub fn report_with_metrics(
        &self,
        mut issues: Vec<CodeIssue>,
        file_count: usize,
        total_lines: usize,
    ) {
        // Split into production and test code issues
        let prod_issues: Vec<CodeIssue> = issues
            .iter()
            .filter(|i| !Self::is_test_path(&i.file_path))
            .cloned()
            .collect();
        let test_issues: Vec<CodeIssue> = issues
            .iter()
            .filter(|i| Self::is_test_path(&i.file_path))
            .cloned()
            .collect();

        // Calculate separate scores
        let scorer = CodeScorer::new();
        let combined_score = scorer.calculate_score(&issues, file_count, total_lines);

        if issues.is_empty() {
            self.print_clean_code_message_with_score(&combined_score);
            return;
        }

        // Sort by severity
        issues.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Nuclear => 3,
                Severity::Spicy => 2,
                Severity::Mild => 1,
            };
            severity_order(&b.severity).cmp(&severity_order(&a.severity))
        });

        // Harsh mode: only show the most severe issues
        if self.harsh_mode {
            issues.retain(|issue| matches!(issue.severity, Severity::Nuclear | Severity::Spicy));
        }

        // Generate roasts
        let roasts = self
            .roast_provider
            .generate_roasts(&issues, &self.i18n.lang);

        if self.markdown {
            self.print_markdown_report(&issues, &roasts);
        } else {
            if !self.summary_only {
                self.print_header(&issues);
                self.print_quality_score(&combined_score);

                // Show production vs test breakdown
                println!();
                println!(
                    "{}  Production code: {} issues",
                    "📦".bright_blue(),
                    prod_issues.len().to_string().bright_yellow(),
                );
                println!(
                    "{}  Test code:      {} issues",
                    "🧪".bright_cyan(),
                    test_issues.len().to_string().bright_yellow(),
                );
                println!();

                if self.verbose {
                    self.print_detailed_analysis(&issues);
                }
                self.print_top_files(&issues);
                self.print_issues(&issues);
            }
            self.print_summary_with_score(&issues, &combined_score);
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

            // Sort rule groups by count (most issues first)
            let mut sorted_rules: Vec<_> = rule_groups.into_iter().collect();
            sorted_rules.sort_by_key(|b| std::cmp::Reverse(b.1.len()));

            for (rule_name, rule_issues) in sorted_rules {
                let count = rule_issues.len();

                // Collect line numbers, deduplicated and sorted
                let mut lines: Vec<usize> = rule_issues
                    .iter()
                    .map(|i| i.line)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                lines.sort_unstable();

                // Format: show up to 6 line numbers, then "+N more"
                let max_show = 6;
                let line_str = if lines.is_empty() {
                    String::new()
                } else if lines.len() <= max_show {
                    lines
                        .iter()
                        .map(|l| format!(":{l}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    let shown: Vec<String> =
                        lines[..max_show].iter().map(|l| format!(":{l}")).collect();
                    format!("{}, +{} more", shown.join(", "), lines.len() - max_show)
                };

                // Severity icon
                let max_sev = rule_issues
                    .iter()
                    .map(|i| &i.severity)
                    .max_by_key(|s| match s {
                        Severity::Nuclear => 3,
                        Severity::Spicy => 2,
                        Severity::Mild => 1,
                    })
                    .unwrap();
                let icon = match max_sev {
                    Severity::Nuclear => "💥",
                    Severity::Spicy => "🌶️ ",
                    Severity::Mild => "😐",
                };

                // Display name
                let display_name = self.rule_display_name(&rule_name);

                // Print: "  💥 magic-number  × 15  [:42, :55, :78, :91, :104, :120, +9 more]"
                println!(
                    "  {} {} {} {}  [{}]",
                    icon,
                    display_name.bright_yellow(),
                    "×".bright_black(),
                    count.to_string().bright_red().bold(),
                    line_str.bright_black()
                );
            }
            println!();
        }
    }

    fn rule_display_name(&self, rule_name: &str) -> String {
        let name = match (self.i18n.lang.as_str(), rule_name) {
            ("zh-CN", "panic-abuse") => "panic滥用",
            ("zh-CN", "god-function") => "上帝函数",
            ("zh-CN", "magic-number") => "魔法数字",
            ("zh-CN", "todo-comment")
            | ("zh-CN", "todo-fixme")
            | ("zh-CN", "todo-bug")
            | ("zh-CN", "todo-hack") => "TODO注释",
            ("zh-CN", "println-debugging") => "println调试",
            ("zh-CN", "string-abuse") => "String滥用",
            ("zh-CN", "vec-abuse") => "Vec滥用",
            ("zh-CN", "hungarian-notation") => "匈牙利命名",
            ("zh-CN", "abbreviation-abuse") => "过度缩写",
            ("zh-CN", "meaningless-naming") => "无意义命名",
            ("zh-CN", "commented-code") => "注释代码",
            ("zh-CN", "dead-code") => "死代码",
            ("zh-CN", "single-letter-variable") => "单字母变量",
            ("zh-CN", "terrible-naming") => "糟糕命名",
            ("zh-CN", "code-duplication") => "代码重复",
            ("zh-CN", "cross-file-duplication") => "跨文件重复",
            ("zh-CN", "deep-nesting") => "深层嵌套",
            ("zh-CN", "long-function") => "过长函数",
            ("zh-CN", "file-too-long") => "过长文件",
            ("zh-CN", "any-type") => "any类型",
            ("zh-CN", "bare-except") => "裸except",
            ("zh-CN", "bare-rescue") => "裸rescue",
            ("zh-CN", "empty-catch") => "空catch",
            ("zh-CN", "unwrap-abuse") => "unwrap滥用",
            ("zh-CN", "box-abuse") => "Box滥用",
            ("zh-CN", "global-variable") => "全局变量",
            ("zh-CN", "wildcard-import") => "通配符导入",
            ("zh-CN", "defer-in-loop") => "循环中defer",
            ("zh-CN", "goroutine-abuse") => "goroutine滥用",
            ("zh-CN", "duplicate-imports") => "重复导入",
            _ => return rule_name.replace('-', " "),
        };
        name.to_string()
    }

    fn print_footer(&self, _issues: &[CodeIssue]) {
        println!();
        println!("{}", self.i18n.get("suggestions").bright_cyan().bold());
        println!("{}", "─".repeat(50).bright_black());

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

        println!();
    }
}
