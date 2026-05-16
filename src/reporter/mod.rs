mod autopsy;
mod display;

use colored::*;
use std::collections::{BTreeMap, HashMap};

use std::path::Path;

use crate::analyzer::{CodeIssue, Severity};
use crate::i18n::I18n;
use crate::llm::{RoastMap, RoastProvider};
use crate::reporter::autopsy::SpreadTarget;
use crate::scoring::{CodeQualityScore, CodeScorer};

pub struct Reporter {
    harsh_mode: bool,
    #[expect(dead_code)]
    savage_mode: bool,
    verbose: bool,
    #[expect(dead_code)]
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
        issues: Vec<CodeIssue>,
        file_count: usize,
        total_lines: usize,
    ) {
        self.report_with_spread(issues, file_count, total_lines, &HashMap::new())
    }

    pub fn report_with_spread(
        &self,
        mut issues: Vec<CodeIssue>,
        file_count: usize,
        total_lines: usize,
        spread: &HashMap<String, Vec<SpreadTarget>>,
    ) {
        // Split into production and test code issues
        let _prod_issues: Vec<CodeIssue> = issues
            .iter()
            .filter(|i| !Self::is_test_path(&i.file_path))
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
            let (personality, autopsy) =
                autopsy::analyze(&issues, &combined_score, file_count, spread);
            let corruption_pct = combined_score.total_score;

            if !self.summary_only {
                self.print_header();
                self.print_personality(&personality, &combined_score, corruption_pct);
                self.print_autopsy(&autopsy);
                self.print_boss_file(&issues);

                if self.verbose {
                    self.print_symptoms(&issues);
                }
            }
            self.print_final_summary(&combined_score, file_count);
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
