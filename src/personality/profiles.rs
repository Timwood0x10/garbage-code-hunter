//! Personality profiles based on code issue patterns.

use super::Personality;
use crate::analyzer::CodeIssue;

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

    // Count issue types
    let mut unwrap_count = 0u32;
    let mut naming_count = 0u32;
    let mut nesting_count = 0u32;
    let mut long_fn_count = 0u32;
    let mut magic_count = 0u32;
    let mut dup_count = 0u32;

    for issue in issues {
        let rule = issue.rule_name.to_lowercase();
        if rule.contains("unwrap") {
            unwrap_count += 1;
        } else if rule.contains("name")
            || rule.contains("single_letter")
            || rule.contains("meaningless")
        {
            naming_count += 1;
        } else if rule.contains("nest") || rule.contains("complex") {
            nesting_count += 1;
        } else if rule.contains("long") || rule.contains("function_length") {
            long_fn_count += 1;
        } else if rule.contains("magic") {
            magic_count += 1;
        } else if rule.contains("duplicat") {
            dup_count += 1;
        }
    }

    // Determine dominant pattern
    let counts = [
        (unwrap_count, "unwrap"),
        (naming_count, "naming"),
        (nesting_count, "nesting"),
        (long_fn_count, "long_fn"),
        (magic_count, "magic"),
        (dup_count, "dup"),
    ];

    let dominant = counts
        .iter()
        .max_by_key(|(c, _)| *c)
        .unwrap_or(&(0, "none"));

    match dominant.1 {
        "unwrap" => panic_personality(unwrap_count, total),
        "naming" => naming_personality(naming_count, total),
        "nesting" => nesting_personality(nesting_count, total),
        "long_fn" => long_fn_personality(long_fn_count, total),
        "magic" => magic_personality(magic_count, total),
        "dup" => dup_personality(dup_count, total),
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
            make_issue("unwrap_abuse"),
            make_issue("unwrap_abuse"),
            make_issue("unwrap_abuse"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Optimist", "3 unwrap => Optimist");
    }

    #[test]
    fn test_naming_dominant() {
        let issues = vec![
            make_issue("single_letter_variable"),
            make_issue("meaningless_name"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Minimalist", "2 naming => Minimalist");
    }

    #[test]
    fn test_nesting_dominant() {
        let issues = vec![
            make_issue("deep_nesting"),
            make_issue("complex_function"),
            make_issue("high_complexity"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Architect", "3 nesting/complex => Architect");
    }

    #[test]
    fn test_long_fn_dominant() {
        let issues = vec![make_issue("long_function"), make_issue("function_length")];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Storyteller", "2 long-fn => Storyteller");
    }

    #[test]
    fn test_magic_dominant() {
        let issues = vec![make_issue("magic_number"), make_issue("magic_number")];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Sorcerer", "2 magic => Sorcerer");
    }

    #[test]
    fn test_dup_dominant() {
        let issues = vec![
            make_issue("code_duplication"),
            make_issue("code_duplication"),
            make_issue("code_duplication"),
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
        let issues: Vec<_> = (0..34).map(|_| make_issue("unwrap_abuse")).collect();
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
        let issues = vec![make_issue("unwrap_abuse")];
        let p = analyze(&issues);
        assert_eq!(p.score, 97.0, "1 unwrap => 100 - 3 = 97, got {}", p.score);
    }

    /// Objective: Verify each archetype has its own multiplier.
    /// Invariants: Same count but different category => different score.
    #[test]
    fn test_archetype_specific_multipliers() {
        // naming has multiplier 2.0, nesting has 4.0
        let naming = analyze(&[make_issue("terrible_naming"), make_issue("single_letter")]);
        let nesting = analyze(&[make_issue("deep_nesting"), make_issue("complex_closure")]);
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

    /// Objective: Verify that issues with unrecognized rule names still count toward total
    ///            but do NOT affect any category count → last max (dup) is picked when all tied.
    /// Invariants: Unrecognized rules fall through all if-else branches without incrementing.
    #[test]
    fn test_unrecognized_rules_fall_to_balanced() {
        let issues = vec![make_issue("random_rule"), make_issue("another_unknown")];
        let p = analyze(&issues);
        // All categories are 0, max_by_key returns last max when tied => "dup" (last in array)
        assert_eq!(
            p.title, "The Copy-Paste Artist",
            "all 0 => last tied max is dup => Copy-Paste Artist"
        );
        assert_eq!(
            p.score, 100.0,
            "0 dupes => 100 - 0*3 = 100, got {}",
            p.score
        );
    }

    // ── case insensitivity ───────────────────────────────────────

    /// Objective: Verify rule name matching is case-insensitive.
    /// Invariants: The code lowercases rule_name before substring matching.
    #[test]
    fn test_case_insensitivity() {
        let issues = vec![
            make_issue("UNWRAP_ABUSE"),
            make_issue("Unwrap_Abuse"),
            make_issue("DEEP_NESTING"),
        ];
        let p = analyze(&issues);
        // 2 unwrap + 1 nesting => unwrap dominant => Optimist
        assert_eq!(
            p.title, "The Optimist",
            "case-insensitive matching: UPPER/mixed should match unwrap"
        );
    }

    // ── balanced personality ──────────────────────────────────────

    /// Objective: Verify that when categories are tied, max_by_key returns the LAST tied max.
    /// Invariants: unwrap=1, nesting=1, others=0 => last max with value 1 is nesting => Architect.
    #[test]
    fn test_tied_categories_pick_last() {
        let issues = vec![
            make_issue("unwrap_abuse"),
            make_issue("terrible_naming"), // doesn't match "name": "naming" ≠ "name"
            make_issue("deep_nesting"),
        ];
        let p = analyze(&issues);
        // unwrap=1, naming=0, nesting=1 => last max value 1 is nesting => Architect
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
            make_issue("code_duplication"),
            make_issue("code_duplication"),
            make_issue("code_duplication"),
            make_issue("deep_nesting"),
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
