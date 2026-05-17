//! Personality profiles based on code issue patterns.

use super::Personality;
use crate::analyzer::CodeIssue;
use crate::signals::{classify_rule, StyleSignal};
use std::collections::HashMap;

/// Analyze issues and determine a personality profile.
pub fn analyze(issues: &[CodeIssue]) -> Personality {
    let total = issues.len() as f64;

    if total == 0.0 {
        return Personality {
            title: "The Perfectionist",
            emoji: "\u{1f45f}",
            traits: vec![
                "No issues detected — suspiciously clean code",
                "Probably over-engineers everything",
                "Definitely has a linter on save",
                "Has never shipped a bug (or a feature on time)",
            ],
            advice: vec![
                "Ship something imperfect once in a while",
                "Your code is great but your deadlines are crying",
                "Perfect is the enemy of shipped",
            ],
            score: 100.0,
        };
    }

    let mut counts: HashMap<StyleSignal, u32> = HashMap::new();
    for issue in issues {
        let signal = classify_rule(&issue.rule_name.to_lowercase());
        *counts.entry(signal).or_insert(0) += 1;
    }

    let get = |s| counts.get(&s).copied().unwrap_or(0);

    let candidates = [
        (get(StyleSignal::PanicAddiction), "unwrap"),
        (get(StyleSignal::NamingChaos), "naming"),
        (get(StyleSignal::NestedHell), "nesting"),
        (get(StyleSignal::OverEngineering), "long_fn"),
        (get(StyleSignal::CodeSmells), "magic"),
        (get(StyleSignal::Duplication), "dup"),
    ];

    let dominant = candidates
        .iter()
        .max_by_key(|(c, _)| *c)
        .unwrap_or(&(0, "none"));

    match dominant.1 {
        "unwrap" => panic_personality(get(StyleSignal::PanicAddiction), total),
        "naming" => naming_personality(get(StyleSignal::NamingChaos), total),
        "nesting" => nesting_personality(get(StyleSignal::NestedHell), total),
        "long_fn" => long_fn_personality(get(StyleSignal::OverEngineering), total),
        "magic" => magic_personality(get(StyleSignal::CodeSmells), total),
        "dup" => dup_personality(get(StyleSignal::Duplication), total),
        _ => balanced_personality(total),
    }
}

fn panic_personality(count: u32, _total: f64) -> Personality {
    Personality {
        title: "The Optimist",
        emoji: "\u{1f60f}",
        traits: vec![
            "Believes the world is full of happy paths",
            "unwrap() is your safety blanket",
            "Error handling is someone else's problem",
            "Probably says 'it works on my machine' a lot",
            "Treats panics as 'unexpected features'",
        ],
        advice: vec![
            "Learn Result<T, E> — your future self will thank you",
            "Every unwrap() is a potential production incident",
            "Try `.unwrap_or_default()` at minimum",
            "Use `?` operator to propagate errors gracefully",
        ],
        score: (100.0 - count as f64 * 3.0).max(0.0),
    }
}

fn naming_personality(count: u32, _total: f64) -> Personality {
    Personality {
        title: "The Minimalist",
        emoji: "\u{270d}\u{fe0f}",
        traits: vec![
            "Why use many word when few letter do trick",
            "Variables named like chess coordinates",
            "Your code reads like a math textbook",
            "Comments explain what x, y, z mean",
            "Considers 'data' a descriptive name",
        ],
        advice: vec![
            "Descriptive names are not a luxury",
            "Your IDE has autocomplete — use it",
            "Future you won't remember what `d` meant",
            "A good variable name eliminates the need for a comment",
        ],
        score: (100.0 - count as f64 * 2.0).max(0.0),
    }
}

fn nesting_personality(count: u32, _total: f64) -> Personality {
    Personality {
        title: "The Architect",
        emoji: "\u{1f3d7}\u{fe0f}",
        traits: vec![
            "Loves building pyramids of doom",
            "Indentation is a competitive sport",
            "Each function is a journey through layers",
            "Probably dreams in nested brackets",
            "Thinks 'flat is justice' only applies to anime",
        ],
        advice: vec![
            "Extract inner logic into helper functions",
            "Use early returns to reduce nesting",
            "Consider the 'guard clause' pattern",
            "If you need 4+ levels of nesting, the logic needs refactoring",
        ],
        score: (100.0 - count as f64 * 4.0).max(0.0),
    }
}

fn long_fn_personality(count: u32, _total: f64) -> Personality {
    Personality {
        title: "The Storyteller",
        emoji: "\u{1f4dd}",
        traits: vec![
            "Every function tells a complete story",
            "Believes in 'single responsibility' — for files, not functions",
            "Your scroll wheel gets a workout",
            "Probably writes long commit messages too",
            "Considers 200 lines a 'concise' function",
        ],
        advice: vec![
            "If a function needs a comment to explain its sections, split it",
            "Aim for functions that fit on one screen",
            "The Single Responsibility Principle applies to functions too",
            "Break complex logic into smaller, testable units",
        ],
        score: (100.0 - count as f64 * 3.0).max(0.0),
    }
}

fn magic_personality(count: u32, _total: f64) -> Personality {
    Personality {
        title: "The Sorcerer",
        emoji: "\u{1f9d9}",
        traits: vec![
            "Numbers have meaning — only to you",
            "42 appears in your code more than in Hitchhiker's Guide",
            "Constants are for the weak",
            "Your code has its own secret numerology",
            "Believes named constants are 'over-engineering'",
        ],
        advice: vec![
            "Extract magic numbers into named constants",
            "Your future self won't remember what 86400 means",
            "Use enums or constants for repeated values",
            "If a number appears twice, it needs a name",
        ],
        score: (100.0 - count as f64 * 2.0).max(0.0),
    }
}

fn dup_personality(count: u32, _total: f64) -> Personality {
    Personality {
        title: "The Copy-Paste Artist",
        emoji: "\u{1f4cb}",
        traits: vec![
            "Ctrl+C, Ctrl+V is your IDE's most used shortcut",
            "Why abstract when you can duplicate",
            "Same bug in 5 places = 5x the debugging fun",
            "DRY stands for 'Don't Repeat... wait, too late'",
            "Thinks 'reusable code' means copying it again",
        ],
        advice: vec![
            "Extract common code into shared functions",
            "One bug fix should fix it everywhere",
            "Consider a utility module for repeated patterns",
            "If you're copying code, you're copying bugs too",
        ],
        score: (100.0 - count as f64 * 3.0).max(0.0),
    }
}

fn balanced_personality(total: f64) -> Personality {
    Personality {
        title: "The Pragmatist",
        emoji: "\u{2696}\u{fe0f}",
        traits: vec![
            "A balanced mix of code smells",
            "Not great at anything, not terrible at anything",
            "The 'average developer' experience",
            "Your code has character — like a diverse zoo",
            "Jack of all trades, master of technical debt",
        ],
        advice: vec![
            "Pick one area to improve at a time",
            "Focus on the highest-severity issues first",
            "Consistency is better than perfection",
            "Tackle your highest-count issue category first",
        ],
        score: (100.0 - total * 1.5).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_issue(rule: &str) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: 0,
            rule_name: rule.to_string(),
            message: "test".to_string(),
            severity: crate::analyzer::Severity::Spicy,
        }
    }

    // ── empty input ──────────────────────────────────────────────

    /// Objective: Verify empty issues return "The Perfectionist" with score 100.
    /// Invariants: The early-return path is taken when total == 0.
    #[test]
    fn test_empty_issues() {
        let p = analyze(&[]);
        assert_eq!(p.title, "The Perfectionist", "empty => Perfectionist");
        assert_eq!(p.score, 100.0, "empty => score 100");
    }

    // ── dominant archetype detection ─────────────────────────────

    /// Objective: Verify each archetype is selected when its category has the highest count.
    /// Invariants: The category with the most issues determines the archetype.
    #[test]
    fn test_unwrap_dominant() {
        let issues = vec![
            make_issue("unwrap-abuse"),
            make_issue("unwrap-abuse"),
            make_issue("unwrap-abuse"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Optimist", "3 unwrap => Optimist");
    }

    #[test]
    fn test_naming_dominant() {
        let issues = vec![
            make_issue("single-letter-variable"),
            make_issue("meaningless-naming"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Minimalist", "2 naming => Minimalist");
    }

    #[test]
    fn test_nesting_dominant() {
        let issues = vec![
            make_issue("deep-nesting"),
            make_issue("cyclomatic-complexity"),
            make_issue("complex-closure"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Architect", "3 nesting/complex => Architect");
    }

    #[test]
    fn test_long_fn_dominant() {
        let issues = vec![make_issue("long-function"), make_issue("file-too-long")];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Storyteller", "2 long-fn => Storyteller");
    }

    #[test]
    fn test_magic_dominant() {
        let issues = vec![make_issue("magic-number"), make_issue("magic-number")];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Sorcerer", "2 magic => Sorcerer");
    }

    #[test]
    fn test_dup_dominant() {
        let issues = vec![
            make_issue("code-duplication"),
            make_issue("code-duplication"),
            make_issue("code-duplication"),
        ];
        let p = analyze(&issues);
        assert_eq!(
            p.title, "The Copy-Paste Artist",
            "3 dup => Copy-Paste Artist"
        );
    }

    // ── score edge cases ─────────────────────────────────────────

    /// Objective: Verify score floors at 0.0 when count * multiplier >= 100.
    /// Invariants: score = max(100 - count * multiplier, 0). Must not go negative.
    #[test]
    fn test_score_boundary_floor_at_zero() {
        // 34 unwraps => 100 - 34*3 = -2 => clamped to 0
        let issues: Vec<_> = (0..34).map(|_| make_issue("unwrap-abuse")).collect();
        let p = analyze(&issues);
        assert_eq!(p.title, "The Optimist");
        assert_eq!(
            p.score, 0.0,
            "34 unwraps => score should floor at 0.0, got {}",
            p.score
        );
    }

    /// Objective: Verify score is exactly 100 - n*multiplier for small n (not clamped).
    #[test]
    fn test_score_exact_value_for_small_count() {
        let issues = vec![make_issue("unwrap-abuse")];
        let p = analyze(&issues);
        assert_eq!(p.score, 97.0, "1 unwrap => 100 - 3 = 97, got {}", p.score);
    }

    /// Objective: Verify each archetype has its own multiplier.
    /// Invariants: Same count but different category => different score.
    #[test]
    fn test_archetype_specific_multipliers() {
        // naming has multiplier 2.0, nesting has 4.0
        let naming = analyze(&[
            make_issue("terrible-naming"),
            make_issue("single-letter-variable"),
        ]);
        let nesting = analyze(&[make_issue("deep-nesting"), make_issue("complex-closure")]);
        assert_eq!(naming.title, "The Minimalist");
        assert_eq!(nesting.title, "The Architect");
        assert!(
            nesting.score < naming.score,
            "nesting (mult 4) should have lower score than naming (mult 2) for same count: {} < {}",
            nesting.score,
            naming.score
        );
    }

    // ── unrecognized rules ───────────────────────────────────────

    /// Objective: Verify that unrecognized rule names fall into CodeSmells (catch-all) and
    ///            contribute to "The Sorcerer" personality, reflecting uncategorized smells.
    /// Invariants: classify_rule maps all unlisted rule names to StyleSignal::CodeSmells.
    #[test]
    fn test_unrecognized_rules_fall_to_sorcerer() {
        let issues = vec![make_issue("random_rule"), make_issue("another_unknown")];
        let p = analyze(&issues);
        // Both map to CodeSmells => magic_count = 2 => The Sorcerer, score = 100 - 2*2 = 96
        assert_eq!(
            p.title, "The Sorcerer",
            "2 unknown => CodeSmells => Sorcerer"
        );
        assert!(
            (p.score - 96.0).abs() < f64::EPSILON,
            "2 magic => score should be 96 (100 - 2*2), got {}",
            p.score
        );
    }

    // ── case insensitivity ───────────────────────────────────────

    /// Objective: Verify rule name matching is case-insensitive via to_lowercase() before classify_rule.
    /// Invariants: to_lowercase() normalizes UPPER/Mixed case to match classify_rule's lowercase strings.
    #[test]
    fn test_case_insensitivity() {
        let issues = vec![
            make_issue("UNWRAP-ABUSE"),
            make_issue("Unwrap-Abuse"),
            make_issue("DEEP-NESTING"),
        ];
        let p = analyze(&issues);
        // 2 PanicAddiction + 1 NestedHell => panic_addiction=2 dominant => Optimist
        assert_eq!(
            p.title, "The Optimist",
            "case-insensitive matching via to_lowercase: UPPER/mixed should match"
        );
    }

    // ── balanced personality ──────────────────────────────────────

    /// Objective: Verify that when categories are tied, max_by_key returns the LAST tied max.
    /// Invariants: unwrap=1, nesting=1, others=0 => last max with value 1 is nesting => Architect.
    #[test]
    fn test_tied_categories_pick_last() {
        let issues = vec![
            make_issue("unwrap-abuse"),
            make_issue("terrible-naming"),
            make_issue("deep-nesting"),
        ];
        let p = analyze(&issues);
        // PanicAddiction=1, NamingChaos=1, NestedHell=1 => last max value 1 is nesting => Architect
        assert_eq!(
            p.title, "The Architect",
            "tied at 1 between unwrap/nesting => last max (nesting) => Architect"
        );
    }

    /// Objective: Verify score is positive for 4 issues with a clear dominant category.
    /// Invariants: 3 dups + 1 nesting => dup dominant => score = 100 - 3*3 = 91.
    #[test]
    fn test_score_formula_with_dominant_category() {
        let issues = vec![
            make_issue("code-duplication"),
            make_issue("code-duplication"),
            make_issue("code-duplication"),
            make_issue("deep-nesting"),
        ];
        let p = analyze(&issues);
        assert_eq!(
            p.title, "The Copy-Paste Artist",
            "3 dup + 1 nesting => dup dominant"
        );
        assert!(
            (p.score - 91.0).abs() < f64::EPSILON,
            "score should be 91 (100 - 3*3), got {}",
            p.score
        );
    }
}
