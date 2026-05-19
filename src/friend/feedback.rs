use crate::analyzer::{CodeIssue, Severity};
use crate::scoring::CodeQualityScore;
use crate::signals::StyleSignal;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FriendMood {
    Proud,
    Concerned,
    Sarcastic,
    Alarmed,
    Exhausted,
}

impl FriendMood {
    pub fn from_score(score: f64) -> Self {
        if score >= 90.0 {
            FriendMood::Proud
        } else if score >= 70.0 {
            FriendMood::Concerned
        } else if score >= 50.0 {
            FriendMood::Sarcastic
        } else if score >= 30.0 {
            FriendMood::Alarmed
        } else {
            FriendMood::Exhausted
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            FriendMood::Proud => "😎",
            FriendMood::Concerned => "🤔",
            FriendMood::Sarcastic => "😏",
            FriendMood::Alarmed => "😰",
            FriendMood::Exhausted => "😩",
        }
    }

    pub fn vibe(&self) -> &'static str {
        match self {
            FriendMood::Proud => "Hey, this is actually pretty good!",
            FriendMood::Concerned => "Not bad, but we need to talk.",
            FriendMood::Sarcastic => "Oh wow. Just... wow.",
            FriendMood::Alarmed => "Dude, we need to have an intervention.",
            FriendMood::Exhausted => "I'm tired just looking at this.",
        }
    }

    pub fn vibe_zh(&self) -> &'static str {
        match self {
            FriendMood::Proud => "嘿，这代码还不错嘛！",
            FriendMood::Concerned => "还行，但咱得聊聊。",
            FriendMood::Sarcastic => "哇哦。真是……绝了。",
            FriendMood::Alarmed => "兄弟，我们需要 intervention 一下。",
            FriendMood::Exhausted => "光看这代码我就累了。",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    pub signal: StyleSignal,
    pub severity: &'static str,
    pub description: String,
    pub suggestion: String,
}

impl BehaviorPattern {
    fn desc_zh(signal: &StyleSignal) -> (&'static str, &'static str) {
        match signal {
            StyleSignal::Duplication => (
                "同样的代码写了好几遍，而不是复用",
                "把共享逻辑提取到函数或模块中",
            ),
            StyleSignal::PanicAddiction => (
                "用 unwrap/expect/panic 代替正确的错误处理",
                "使用 Result<T, E> 并用 '?' 传播错误",
            ),
            StyleSignal::NamingChaos => ("变量名看不出是干什么的", "用能表达意图的描述性名称"),
            StyleSignal::NestedHell => (
                "嵌套太深，代码难以阅读",
                "用 early return 和 guard clause 减少嵌套",
            ),
            StyleSignal::HotfixCulture => {
                ("残留的调试打印、TODO 和注释掉的代码", "提交前清理调试残留")
            }
            StyleSignal::OverEngineering => ("一个函数干太多事", "把大函数拆成职责单一的小函数"),
            StyleSignal::CodeSmells => (
                "unsafe 块、魔法数字和可疑的写法",
                "优先用安全抽象；给常量起个好名字",
            ),
            StyleSignal::LegacyCode => ("源文件里留着注释掉的代码", "删掉死代码，git 有历史记录"),
            StyleSignal::TodoMountain => (
                "堆积的 TODO/FIXME/BUG/HACK 标记",
                "用 issue 跟踪器管理待办，别写在代码里",
            ),
            StyleSignal::LineCountSmell => (
                "文件行数超过了合理阈值",
                "把大文件拆成更小、职责更清晰的模块",
            ),
        }
    }

    pub fn from_signals(scores: &HashMap<StyleSignal, f64>) -> Vec<Self> {
        let mut pairs: Vec<(&StyleSignal, f64)> = scores.iter().map(|(s, v)| (s, *v)).collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        pairs
            .into_iter()
            .filter(|(_, v)| *v >= 3.0)
            .take(3)
            .map(|(signal, score)| {
                let severity = if score >= 12.0 {
                    "major"
                } else if score >= 6.0 {
                    "moderate"
                } else {
                    "minor"
                };
                let (description, suggestion) = match signal {
                    StyleSignal::Duplication => (
                        "Writing the same code multiple times instead of reusing it".into(),
                        "Extract shared logic into functions or modules".into(),
                    ),
                    StyleSignal::PanicAddiction => (
                        "Using unwrap/expect/panic instead of proper error handling".into(),
                        "Use Result<T, E> and propagate errors with '?'".into(),
                    ),
                    StyleSignal::NamingChaos => (
                        "Variable names that don't explain what they do".into(),
                        "Use descriptive names that convey intent".into(),
                    ),
                    StyleSignal::NestedHell => (
                        "Deeply nested blocks that are hard to follow".into(),
                        "Early returns and guard clauses reduce nesting".into(),
                    ),
                    StyleSignal::HotfixCulture => (
                        "Leftover debug prints, TODOs, and commented code".into(),
                        "Clean up debug artifacts before committing".into(),
                    ),
                    StyleSignal::OverEngineering => (
                        "Functions that try to do too many things at once".into(),
                        "Split large functions into focused smaller ones".into(),
                    ),
                    StyleSignal::CodeSmells => (
                        "Unsafe blocks, magic numbers, and questionable patterns".into(),
                        "Prefer safe abstractions; name constants clearly".into(),
                    ),
                    StyleSignal::LegacyCode => (
                        "Commented-out code left in source files".into(),
                        "Delete dead code instead of commenting it out; git has history".into(),
                    ),
                    StyleSignal::TodoMountain => (
                        "Accumulated TODO/FIXME/BUG/HACK markers".into(),
                        "Track todos in an issue tracker, not in source code".into(),
                    ),
                    StyleSignal::LineCountSmell => (
                        "Files that exceed reasonable line count thresholds".into(),
                        "Split large files into smaller focused modules".into(),
                    ),
                };
                BehaviorPattern {
                    signal: *signal,
                    severity,
                    description,
                    suggestion,
                }
            })
            .collect()
    }

    pub fn from_signals_zh(scores: &HashMap<StyleSignal, f64>) -> Vec<Self> {
        let mut pairs: Vec<(&StyleSignal, f64)> = scores.iter().map(|(s, v)| (s, *v)).collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        pairs
            .into_iter()
            .filter(|(_, v)| *v >= 3.0)
            .take(3)
            .map(|(signal, score)| {
                let severity = if score >= 12.0 {
                    "严重"
                } else if score >= 6.0 {
                    "中等"
                } else {
                    "轻微"
                };
                let (description, suggestion) = Self::desc_zh(signal);
                BehaviorPattern {
                    signal: *signal,
                    severity,
                    description: description.into(),
                    suggestion: suggestion.into(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct NextAction {
    pub priority: u8,
    pub file: String,
    pub line: usize,
    pub action: String,
    pub reason: String,
}

impl NextAction {
    pub fn from_issues(issues: &[CodeIssue]) -> Vec<Self> {
        // Skip signal-level findings (line=0) — only show actionable per-line issues
        let mut actionable: Vec<&CodeIssue> = issues.iter().filter(|i| i.line > 0).collect();
        actionable.sort_by(|a, b| {
            let order = |s: &Severity| match s {
                Severity::Nuclear => 3,
                Severity::Spicy => 2,
                Severity::Mild => 1,
            };
            order(&b.severity).cmp(&order(&a.severity))
        });

        actionable
            .into_iter()
            .take(3)
            .enumerate()
            .map(|(i, issue)| {
                let file = issue
                    .file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| issue.file_path.to_string_lossy().to_string());
                let action = format!("Fix '{}'", issue.rule_name);
                let reason = issue.message.clone();
                NextAction {
                    priority: (i + 1) as u8,
                    file,
                    line: issue.line,
                    action,
                    reason,
                }
            })
            .collect()
    }

    pub fn from_issues_zh(issues: &[CodeIssue]) -> Vec<Self> {
        let mut actionable: Vec<&CodeIssue> = issues.iter().filter(|i| i.line > 0).collect();
        actionable.sort_by(|a, b| {
            let order = |s: &Severity| match s {
                Severity::Nuclear => 3,
                Severity::Spicy => 2,
                Severity::Mild => 1,
            };
            order(&b.severity).cmp(&order(&a.severity))
        });

        actionable
            .into_iter()
            .take(3)
            .enumerate()
            .map(|(i, issue)| {
                let file = issue
                    .file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| issue.file_path.to_string_lossy().to_string());
                let action = format!("修复 '{}'", issue.rule_name);
                let reason = issue.message.clone();
                NextAction {
                    priority: (i + 1) as u8,
                    file,
                    line: issue.line,
                    action,
                    reason,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct FriendFeedback {
    pub mood: FriendMood,
    pub patterns: Vec<BehaviorPattern>,
    pub next_actions: Vec<NextAction>,
    pub total_issues: usize,
    pub total_score: f64,
}

impl FriendFeedback {
    pub fn new(
        issues: &[CodeIssue],
        score: &CodeQualityScore,
        signal_scores: &HashMap<StyleSignal, f64>,
    ) -> Self {
        let mood = FriendMood::from_score(score.total_score);
        let patterns = BehaviorPattern::from_signals(signal_scores);
        let next_actions = NextAction::from_issues(issues);
        FriendFeedback {
            mood,
            patterns,
            next_actions,
            total_issues: issues.len(),
            total_score: score.total_score,
        }
    }

    pub fn new_zh(
        issues: &[CodeIssue],
        score: &CodeQualityScore,
        signal_scores: &HashMap<StyleSignal, f64>,
    ) -> Self {
        let mood = FriendMood::from_score(score.total_score);
        let patterns = BehaviorPattern::from_signals_zh(signal_scores);
        let next_actions = NextAction::from_issues_zh(issues);
        FriendFeedback {
            mood,
            patterns,
            next_actions,
            total_issues: issues.len(),
            total_score: score.total_score,
        }
    }

    pub fn print(&self) {
        use colored::*;
        println!();
        println!(
            "{} Friend's Take {}",
            "💬".bright_cyan(),
            "─".repeat(60).bright_black()
        );
        println!(
            "{}  {} {}",
            self.mood.emoji(),
            self.mood.vibe().bright_cyan().bold(),
            if self.total_issues == 0 {
                "".to_string()
            } else {
                format!(
                    "  ({} issue{})",
                    self.total_issues.to_string().yellow(),
                    if self.total_issues == 1 { "" } else { "s" }
                )
            }
        );
        println!("{}  Score: {:.1}/100", "📊".bright_blue(), self.total_score);

        if !self.patterns.is_empty() {
            println!();
            println!("{}  Patterns I noticed:", "🔍".bright_yellow());
            for p in &self.patterns {
                let sev_color = match p.severity {
                    "major" => "red",
                    "moderate" => "yellow",
                    _ => "blue",
                };
                println!(
                    "  {} [{}] {}",
                    match p.severity {
                        "major" => "🔴",
                        "moderate" => "🟡",
                        _ => "🔵",
                    },
                    p.severity.bold().color(sev_color),
                    p.description,
                );
                println!("     → {}", p.suggestion.dimmed());
            }
        }

        if !self.next_actions.is_empty() {
            println!();
            println!("{}  Quick wins (top 3):", "🎯".bright_green());
            for a in &self.next_actions {
                let location = format!("{}:{}", a.file, a.line).bright_white();
                println!("  {}. {} — {}", a.priority, location, a.action.bold(),);
                println!("     {}", a.reason.dimmed());
            }
        }
        println!();
    }

    pub fn print_zh(&self) {
        use colored::*;
        println!();
        println!(
            "{} 朋友的看法 {}",
            "💬".bright_cyan(),
            "─".repeat(60).bright_black()
        );
        println!(
            "{}  {} {}",
            self.mood.emoji(),
            self.mood.vibe_zh().bright_cyan().bold(),
            if self.total_issues == 0 {
                "".to_string()
            } else {
                format!("  ({} 个问题)", self.total_issues.to_string().yellow(),)
            }
        );
        println!("{}  评分: {:.1}/100", "📊".bright_blue(), self.total_score);

        if !self.patterns.is_empty() {
            println!();
            println!("{}  发现的问题模式:", "🔍".bright_yellow());
            for p in &self.patterns {
                let sev_color = match p.severity {
                    "严重" => "red",
                    "中等" => "yellow",
                    _ => "blue",
                };
                println!(
                    "  {} [{}] {}",
                    match p.severity {
                        "严重" => "🔴",
                        "中等" => "🟡",
                        _ => "🔵",
                    },
                    p.severity.bold().color(sev_color),
                    p.description,
                );
                println!("     → {}", p.suggestion.dimmed());
            }
        }

        if !self.next_actions.is_empty() {
            println!();
            println!("{}  快速修复 (前 3 项):", "🎯".bright_green());
            for a in &self.next_actions {
                let location = format!("{}:{}", a.file, a.line).bright_white();
                println!("  {}. {} — {}", a.priority, location, a.action.bold(),);
                println!("     {}", a.reason.dimmed());
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::{CodeQualityScore, QualityLevel, SeverityDistribution};
    use std::path::PathBuf;

    fn make_score(total: f64) -> CodeQualityScore {
        CodeQualityScore {
            total_score: total,
            n_score: total,
            d_score: 0.0,
            category_scores: HashMap::new(),
            signal_scores: HashMap::new(),
            file_count: 1,
            total_lines: 100,
            issue_density: 0.0,
            quality_level: QualityLevel::from_score(total),
            severity_distribution: SeverityDistribution {
                nuclear: 0,
                spicy: 0,
                mild: 0,
            },
        }
    }

    fn make_issue(severity: Severity, line: usize, rule: &str) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("src/main.rs"),
            line,
            column: 1,
            rule_name: rule.to_string(),
            message: format!("{} issue", rule),
            severity,
        }
    }

    #[test]
    fn test_mood_proud_at_high_score() {
        assert_eq!(FriendMood::from_score(95.0), FriendMood::Proud);
    }

    #[test]
    fn test_mood_concerned_at_mid_score() {
        assert_eq!(FriendMood::from_score(75.0), FriendMood::Concerned);
    }

    #[test]
    fn test_mood_exhausted_at_low_score() {
        assert_eq!(FriendMood::from_score(10.0), FriendMood::Exhausted);
    }

    #[test]
    fn test_behavior_patterns_top_3_signals() {
        let mut scores = HashMap::new();
        scores.insert(StyleSignal::PanicAddiction, 18.0);
        scores.insert(StyleSignal::NamingChaos, 12.0);
        scores.insert(StyleSignal::NestedHell, 3.0);
        let patterns = BehaviorPattern::from_signals(&scores);
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0].signal, StyleSignal::PanicAddiction);
        assert_eq!(patterns[1].signal, StyleSignal::NamingChaos);
    }

    #[test]
    fn test_behavior_patterns_filters_low_scores() {
        let mut scores = HashMap::new();
        scores.insert(StyleSignal::PanicAddiction, 2.0);
        scores.insert(StyleSignal::NamingChaos, 1.0);
        let patterns = BehaviorPattern::from_signals(&scores);
        assert!(patterns.is_empty(), "signals below 3.0 should be filtered");
    }

    #[test]
    fn test_next_actions_top_3_by_severity() {
        let issues = vec![
            make_issue(Severity::Mild, 1, "mild-rule"),
            make_issue(Severity::Nuclear, 2, "nuclear-rule"),
            make_issue(Severity::Spicy, 3, "spicy-rule"),
            make_issue(Severity::Mild, 4, "another-mild"),
        ];
        let actions = NextAction::from_issues(&issues);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].action, "Fix 'nuclear-rule'");
        assert_eq!(actions[1].action, "Fix 'spicy-rule'");
        assert_eq!(actions[2].action, "Fix 'mild-rule'");
    }

    #[test]
    fn test_next_actions_empty_issues() {
        let actions = NextAction::from_issues(&[]);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_friend_feedback_construction() {
        let issues = vec![make_issue(Severity::Spicy, 10, "unwrap-abuse")];
        let score = make_score(65.0);
        let mut signal_scores = HashMap::new();
        signal_scores.insert(StyleSignal::PanicAddiction, 15.0);
        let feedback = FriendFeedback::new(&issues, &score, &signal_scores);
        assert_eq!(feedback.mood, FriendMood::Sarcastic);
        assert_eq!(feedback.total_issues, 1);
        assert!(!feedback.patterns.is_empty());
        assert!(!feedback.next_actions.is_empty());
    }
}
