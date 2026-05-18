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
}

#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    pub signal: StyleSignal,
    pub severity: &'static str,
    pub description: String,
    pub suggestion: String,
}

impl BehaviorPattern {
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
        let mut sorted: Vec<&CodeIssue> = issues.iter().collect();
        sorted.sort_by(|a, b| {
            let order = |s: &Severity| match s {
                Severity::Nuclear => 3,
                Severity::Spicy => 2,
                Severity::Mild => 1,
            };
            order(&b.severity).cmp(&order(&a.severity))
        });

        sorted
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
