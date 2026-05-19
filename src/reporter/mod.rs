mod autopsy;
mod display;
mod translations;

use colored::*;
use std::collections::{BTreeMap, HashMap};
#[cfg(test)]
use std::path::Path;

use crate::analyzer::{CodeIssue, Severity};
use crate::friend::FriendFeedback;
use crate::i18n::I18n;
use crate::llm::{RoastMap, RoastProvider};
use crate::reporter::autopsy::SpreadTarget;
use crate::scoring::{CodeQualityScore, CodeScorer};
use crate::signals::StyleSignal;
use crate::style_ir::StyleIrSummary;

pub struct Reporter {
    harsh_mode: bool,
    verbose: bool,
    #[expect(dead_code)]
    top_files: usize,
    max_issues_per_file: usize,
    summary_only: bool,
    brief: bool,
    markdown: bool,
    i18n: I18n,
    roast_provider: Box<dyn RoastProvider>,
    direct_scores: HashMap<StyleSignal, f64>,
    style_ir_summary: Option<StyleIrSummary>,
    show_friend_feedback: bool,
}

impl Reporter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        harsh_mode: bool,
        verbose: bool,
        top_files: usize,
        max_issues_per_file: usize,
        summary_only: bool,
        brief: bool,
        markdown: bool,
        lang: &str,
        roast_provider: Box<dyn RoastProvider>,
    ) -> Self {
        Self {
            harsh_mode,
            verbose,
            top_files,
            max_issues_per_file,
            summary_only,
            brief,
            markdown,
            i18n: I18n::new(lang),
            roast_provider,
            direct_scores: HashMap::new(),
            style_ir_summary: None,
            show_friend_feedback: false,
        }
    }

    pub fn with_friend_feedback(mut self, show: bool) -> Self {
        self.show_friend_feedback = show;
        self
    }

    pub fn with_direct_scores(mut self, scores: HashMap<StyleSignal, f64>) -> Self {
        self.direct_scores = scores;
        self
    }

    pub fn with_style_ir_summary(mut self, summary: Option<StyleIrSummary>) -> Self {
        self.style_ir_summary = summary;
        self
    }

    #[cfg(test)]
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
        // Calculate separate scores (merge issue-derived + direct signal scores)
        let scorer = CodeScorer::new();
        let combined_score = if self.direct_scores.is_empty() {
            scorer.calculate_score(&issues, file_count, total_lines)
        } else {
            scorer.calculate_score_with_direct(
                &issues,
                file_count,
                total_lines,
                self.direct_scores.clone(),
            )
        };

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
            self.print_markdown_report(&issues, &roasts, &combined_score, spread);
        } else {
            if self.show_friend_feedback && !issues.is_empty() {
                let is_zh = self.i18n.lang.starts_with("zh");
                let feedback = if is_zh {
                    FriendFeedback::new_zh(&issues, &combined_score, &self.direct_scores)
                } else {
                    FriendFeedback::new(&issues, &combined_score, &self.direct_scores)
                };
                if is_zh {
                    feedback.print_zh();
                } else {
                    feedback.print();
                }
            }

            let (personality, autopsy) =
                autopsy::analyze(&issues, &combined_score, file_count, spread);
            let corruption_pct = combined_score.total_score;

            if !self.summary_only {
                self.print_header();
                self.print_personality(&personality, &combined_score, corruption_pct);
                self.print_autopsy(&autopsy);
                if !self.brief {
                    self.print_boss_file(&issues);
                }
                self.print_behavior_distribution(&combined_score);

                if self.verbose {
                    self.print_symptoms(&issues);
                }
            }
            self.print_final_summary(&combined_score, file_count, Some(personality.project_type));
            self.print_style_ir_summary();
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

    fn print_markdown_report(
        &self,
        issues: &[CodeIssue],
        roasts: &RoastMap,
        combined_score: &CodeQualityScore,
        spread: &HashMap<String, Vec<SpreadTarget>>,
    ) {
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
        println!(
            "**Score:** {:.1}/100 **{}**",
            combined_score.total_score,
            combined_score.quality_level.description(&self.i18n.lang)
        );
        println!();

        // ── Friend Feedback ──
        if self.show_friend_feedback {
            let is_zh = self.i18n.lang.starts_with("zh");
            let feedback = if is_zh {
                FriendFeedback::new_zh(issues, combined_score, &self.direct_scores)
            } else {
                FriendFeedback::new(issues, combined_score, &self.direct_scores)
            };
            if is_zh {
                println!("## 💬 朋友的看法");
                println!();
                println!(
                    "**心情:** {} {}",
                    feedback.mood.emoji(),
                    feedback.mood.vibe_zh()
                );
            } else {
                println!("## 💬 Friend's Take");
                println!();
                println!(
                    "**Mood:** {} {}",
                    feedback.mood.emoji(),
                    feedback.mood.vibe()
                );
            }
            if !feedback.patterns.is_empty() {
                println!();
                if is_zh {
                    println!("**发现的问题模式:**");
                } else {
                    println!("**Patterns I noticed:**");
                }
                for p in &feedback.patterns {
                    let sev = match p.severity {
                        "major" | "严重" => "🔴",
                        "moderate" | "中等" => "🟡",
                        _ => "🔵",
                    };
                    println!("- {} {} — {}", sev, p.description, p.suggestion);
                }
            }
            if !feedback.next_actions.is_empty() {
                println!();
                if is_zh {
                    println!("**快速修复 (前 3 项):**");
                } else {
                    println!("**Quick wins (top 3):**");
                }
                for a in &feedback.next_actions {
                    println!(
                        "- `{}:{}` — {} _( {} )_",
                        a.file, a.line, a.action, a.reason
                    );
                }
            }
            println!();
        }

        // ── Personality ──
        let (personality, autopsy) = autopsy::analyze(issues, combined_score, issues.len(), spread);
        println!("## {} Personality", personality.emoji);
        println!();
        println!("**Type:** {}", personality.project_type);
        println!("**Threat Level:** {}", personality.threat_level);
        println!();
        println!("**Signature traits:**");
        for trait_text in &personality.core_traits {
            println!("- {}", trait_text);
        }
        println!();
        println!("**Diagnosis:** {}", autopsy.cause_of_death);
        println!("**Condition:** {}", autopsy.patient_condition);
        println!();

        // ── Statistics ──
        println!("## {}", self.i18n.get("statistics"));
        println!();
        println!("| Severity | Count |");
        println!("| --- | --- |");
        println!("| 🔥 Nuclear | {} |", nuclear);
        println!("| 🌶️ Spicy | {} |", spicy);
        println!("| 😐 Mild | {} |", mild);
        println!("| **Total** | **{}** |", total);
        println!();

        // ── Behavior Distribution ──
        if !combined_score.signal_scores.is_empty() {
            println!("## Signal Distribution");
            println!();
            let mut signals: Vec<_> = combined_score.signal_scores.iter().collect();
            signals.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            println!("| Signal | Score | Severity |");
            println!("| --- | --- | --- |");
            for (signal, &score) in signals {
                let label = if score >= 12.0 {
                    "🔴 High"
                } else if score >= 6.0 {
                    "🟡 Medium"
                } else {
                    "🟢 Low"
                };
                println!("| {} | {:.1} | {} |", signal.display_name(), score, label);
            }
            println!();
        }

        // ── Detailed Analysis ──
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

        // ── Issues by File ──
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

        // ── StyleIr Summary ──
        if let Some(ref sir) = self.style_ir_summary {
            println!("## Code Metrics");
            println!();
            println!("| Metric | Value |");
            println!("| --- | --- |");
            println!("| Lines | {} |", sir.line_count);
            println!("| Panic Calls | {} |", sir.panic_call_count);
            println!("| Naming Violations | {} |", sir.naming_violation_count);
            println!("| Deep Nesting | {} |", sir.deeply_nested_block_count);
            println!("| Debug Calls | {} |", sir.debug_call_count);
            println!("| God Functions | {} |", sir.god_function_count);
            println!("| Unsafe Blocks | {} |", sir.unsafe_block_count);
            println!("| Magic Numbers | {} |", sir.magic_number_count);
            println!();
        }

        println!("## {}", self.i18n.get("suggestions"));
        println!();
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify is_test_path returns true for all supported test directory patterns.
    /// Invariants: /tests/, /test/, /test-files/, /fixtures/, /mocks/, /examples/, /benches/
    ///             are all recognized as test paths regardless of file extension.
    #[test]
    fn test_is_test_path_detects_test_directories() {
        assert!(
            Reporter::is_test_path(Path::new("/project/src/tests/helper.rs")),
            "/tests/ should be detected"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/tests/fixtures/data.json")),
            "/fixtures/ should be detected"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/tests/mocks/service.rs")),
            "/mocks/ should be detected"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/examples/demo.rs")),
            "/examples/ should be detected"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/benches/perf.rs")),
            "/benches/ should be detected"
        );
    }

    /// Objective: Verify is_test_path detects files ending with _test.{rs,go,py,js,ts,java}.
    /// Invariants: Each supported language suffix must be recognized independently.
    #[test]
    fn test_is_test_path_detects_all_language_test_suffixes() {
        assert!(
            Reporter::is_test_path(Path::new("/project/src/foo_test.rs")),
            "_test.rs should be test"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/src/handler_test.go")),
            "_test.go should be test"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/src/util_test.py")),
            "_test.py should be test"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/src/app_test.js")),
            "_test.js should be test"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/src/app_test.ts")),
            "_test.ts should be test"
        );
        assert!(
            Reporter::is_test_path(Path::new("/project/src/Foo_test.java")),
            "_test.java should be test"
        );
    }

    /// Objective: Verify is_test_path detects files starting with "test_" at the root.
    /// Invariants: `name.starts_with("test_")` only matches when the filename is the
    ///             entire path (e.g., "test_main.py"), not when embedded in a directory.
    #[test]
    fn test_is_test_path_detects_test_prefix_at_root() {
        assert!(
            Reporter::is_test_path(Path::new("test_main.py")),
            "bare filename starting with test_ should be test"
        );
    }

    /// Objective: Verify is_test_path returns false for normal production source files.
    /// Invariants: Source files in any language under src/ (or similar) should not match.
    #[test]
    fn test_is_test_path_does_not_flag_production_code() {
        assert!(
            !Reporter::is_test_path(Path::new("/project/src/main.rs")),
            "src/main.rs should not be test"
        );
        assert!(
            !Reporter::is_test_path(Path::new("/project/src/lib.rs")),
            "src/lib.rs should not be test"
        );
        assert!(
            !Reporter::is_test_path(Path::new("/project/src/server.go")),
            "src/server.go should not be test"
        );
    }

    /// Objective: Verify Reporter construction normalizes "en" to "en-US" correctly.
    #[test]
    fn test_reporter_creates_with_english_i18n() {
        let reporter = Reporter::new(
            false,
            false,
            10,
            5,
            false,
            false,
            false,
            "en",
            Box::new(crate::llm::LocalRoastProvider),
        );
        assert_eq!(
            reporter.i18n.lang, "en-US",
            "Reporter::new with 'en' should normalize to 'en-US', got '{}'",
            reporter.i18n.lang
        );
    }

    /// Objective: Verify Reporter construction preserves "zh-CN" correctly.
    #[test]
    fn test_reporter_creates_with_chinese_i18n() {
        let reporter = Reporter::new(
            false,
            false,
            10,
            5,
            false,
            false,
            false,
            "zh-CN",
            Box::new(crate::llm::LocalRoastProvider),
        );
        assert_eq!(
            reporter.i18n.lang, "zh-CN",
            "Reporter::new with 'zh-CN' should keep 'zh-CN', got '{}'",
            reporter.i18n.lang
        );
    }
}
