// StyleSignal — maps rule issues to behavioral style signals.

use crate::analyzer::CodeIssue;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleSignal {
    Duplication,
    PanicAddiction,
    NamingChaos,
    NestedHell,
    HotfixCulture,
    OverEngineering,
    CodeSmells,
}

impl StyleSignal {
    pub fn all() -> &'static [StyleSignal] {
        &[
            StyleSignal::Duplication,
            StyleSignal::PanicAddiction,
            StyleSignal::NamingChaos,
            StyleSignal::NestedHell,
            StyleSignal::HotfixCulture,
            StyleSignal::OverEngineering,
            StyleSignal::CodeSmells,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            StyleSignal::Duplication => "Duplication",
            StyleSignal::PanicAddiction => "Panic Addiction",
            StyleSignal::NamingChaos => "Naming Chaos",
            StyleSignal::NestedHell => "Nested Hell",
            StyleSignal::HotfixCulture => "Hotfix Culture",
            StyleSignal::OverEngineering => "Over-Engineering",
            StyleSignal::CodeSmells => "Code Smells",
        }
    }

    pub fn display_name_zh(&self) -> String {
        match self {
            StyleSignal::Duplication => "重复代码",
            StyleSignal::PanicAddiction => "恐慌成瘾",
            StyleSignal::NamingChaos => "命名混乱",
            StyleSignal::NestedHell => "嵌套地狱",
            StyleSignal::HotfixCulture => "热修复文化",
            StyleSignal::OverEngineering => "过度工程",
            StyleSignal::CodeSmells => "代码异味",
        }
        .to_string()
    }
}

pub fn classify_rule(rule_name: &str) -> StyleSignal {
    match rule_name {
        "code-duplication" | "cross-file-duplication" => StyleSignal::Duplication,
        "unwrap-abuse" | "panic-abuse" | "bare-except" | "bare-rescue" | "empty-catch"
        | "println-debugging" => StyleSignal::PanicAddiction,
        "terrible-naming"
        | "single-letter-variable"
        | "meaningless-naming"
        | "hungarian-notation"
        | "abbreviation-abuse"
        | "c-naming"
        | "go-receiver-name"
        | "go-mixed-caps"
        | "ruby-predicate-method"
        | "python-naming"
        | "constant-name" => StyleSignal::NamingChaos,
        "deep-nesting"
        | "cyclomatic-complexity"
        | "c-nesting"
        | "complex-closure"
        | "go-else-return"
        | "negated-if" => StyleSignal::NestedHell,
        "todo-comment" | "todo-fixme" | "todo-bug" | "todo-hack" | "commented-code"
        | "dead-code" | "c-commented-code" | "c-dead-code" => StyleSignal::HotfixCulture,
        "too-many-params" | "god-function" | "long-function" | "c-long-function"
        | "c-god-function" | "file-too-long" | "module-complexity" | "trait-complexity"
        | "generic-abuse" => StyleSignal::OverEngineering,
        _ => StyleSignal::CodeSmells,
    }
}

pub fn compute_signal_scores(
    issues: &[CodeIssue],
    total_lines: usize,
) -> HashMap<StyleSignal, f64> {
    let k_lines = total_lines as f64 / 1000.0;
    let mut counts: HashMap<StyleSignal, usize> = HashMap::new();

    for issue in issues {
        let signal = classify_rule(&issue.rule_name);
        *counts.entry(signal).or_insert(0) += 1;
    }

    let mut scores = HashMap::new();
    for signal in StyleSignal::all() {
        let count = counts.get(signal).copied().unwrap_or(0);
        let density = if k_lines > 0.0 {
            count as f64 / k_lines
        } else {
            0.0
        };
        let score = ((density + 1.0).log2() * 6.0).min(25.0);
        scores.insert(*signal, score);
    }

    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_issue(rule_name: &str) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: 0,
            rule_name: rule_name.to_string(),
            message: String::new(),
            severity: crate::analyzer::Severity::Spicy,
        }
    }

    #[test]
    fn test_classify_duplication() {
        assert_eq!(classify_rule("code-duplication"), StyleSignal::Duplication);
    }

    #[test]
    fn test_classify_panic() {
        assert_eq!(classify_rule("unwrap-abuse"), StyleSignal::PanicAddiction);
        assert_eq!(classify_rule("panic-abuse"), StyleSignal::PanicAddiction);
    }

    #[test]
    fn test_classify_naming() {
        assert_eq!(classify_rule("terrible-naming"), StyleSignal::NamingChaos);
        assert_eq!(
            classify_rule("single-letter-variable"),
            StyleSignal::NamingChaos
        );
    }

    #[test]
    fn test_classify_nested() {
        assert_eq!(classify_rule("deep-nesting"), StyleSignal::NestedHell);
    }

    #[test]
    fn test_classify_hotfix() {
        assert_eq!(classify_rule("todo-comment"), StyleSignal::HotfixCulture);
        assert_eq!(classify_rule("dead-code"), StyleSignal::HotfixCulture);
    }

    #[test]
    fn test_classify_over_engineering() {
        assert_eq!(classify_rule("god-function"), StyleSignal::OverEngineering);
    }

    #[test]
    fn test_classify_code_smells_fallback() {
        assert_eq!(classify_rule("magic-number"), StyleSignal::CodeSmells);
        assert_eq!(classify_rule("unknown-rule"), StyleSignal::CodeSmells);
    }

    #[test]
    fn test_compute_signal_scores_empty() {
        let scores = compute_signal_scores(&[], 1000);
        assert_eq!(scores.len(), 7);
        assert!(scores.values().all(|&s| s == 0.0));
    }

    #[test]
    fn test_compute_signal_scores_mixed() {
        let issues = vec![
            make_issue("unwrap-abuse"),
            make_issue("unwrap-abuse"),
            make_issue("deep-nesting"),
            make_issue("terrible-naming"),
        ];
        let scores = compute_signal_scores(&issues, 1000);
        assert!(scores[&StyleSignal::PanicAddiction] > scores[&StyleSignal::NamingChaos]);
    }
}
