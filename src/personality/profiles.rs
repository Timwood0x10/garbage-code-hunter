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

    #[test]
    fn test_empty_issues() {
        let p = analyze(&[]);
        assert_eq!(p.title, "The Perfectionist");
    }

    #[test]
    fn test_unwrap_dominant() {
        let issues = vec![
            make_issue("unwrap_abuse"),
            make_issue("unwrap_abuse"),
            make_issue("unwrap_abuse"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Optimist");
    }

    #[test]
    fn test_naming_dominant() {
        let issues = vec![
            make_issue("single_letter_variable"),
            make_issue("meaningless_name"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Minimalist");
    }

    #[test]
    fn test_nesting_dominant() {
        let issues = vec![
            make_issue("deep_nesting"),
            make_issue("complex_function"),
            make_issue("high_complexity"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Architect");
    }

    #[test]
    fn test_long_fn_dominant() {
        let issues = vec![make_issue("long_function"), make_issue("function_length")];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Storyteller");
    }

    #[test]
    fn test_magic_dominant() {
        let issues = vec![make_issue("magic_number"), make_issue("magic_number")];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Sorcerer");
    }

    #[test]
    fn test_dup_dominant() {
        let issues = vec![
            make_issue("code_duplication"),
            make_issue("code_duplication"),
            make_issue("code_duplication"),
        ];
        let p = analyze(&issues);
        assert_eq!(p.title, "The Copy-Paste Artist");
    }

    #[test]
    fn test_balanced_mixed() {
        let issues = vec![
            make_issue("unwrap_abuse"),
            make_issue("single_letter_variable"),
            make_issue("deep_nesting"),
        ];
        let p = analyze(&issues);
        // All tied at 1 each - should pick one (first max)
        assert!(!p.title.is_empty());
        assert!(p.score > 0.0);
    }
}
