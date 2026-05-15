//! PR title quality rules.

use super::types::{PrEntry, PrIssue, Severity};
use regex::Regex;
use std::sync::LazyLock;

/// A rule that checks PR titles.
pub trait PrRule: Send + Sync {
    fn id(&self) -> &str;
    fn check(&self, pr: &PrEntry) -> Option<PrIssue>;
}

/// Title is empty or whitespace only.
pub struct EmptyTitleRule;

impl PrRule for EmptyTitleRule {
    fn id(&self) -> &str {
        "empty-title"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        if pr.title.trim().is_empty() {
            Some(PrIssue {
                rule_id: self.id().to_string(),
                severity: Severity::Critical,
                message: "PR title is empty — did you submit by accident?".to_string(),
                pr_id: pr.id.clone(),
                pr_title: pr.title.clone(),
            })
        } else {
            None
        }
    }
}

/// Title is too short (<= 5 chars).
pub struct TooShortRule {
    pub min_length: usize,
}

impl PrRule for TooShortRule {
    fn id(&self) -> &str {
        "too-short"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        let trimmed = pr.title.trim();
        if !trimmed.is_empty() && trimmed.chars().count() <= self.min_length {
            Some(PrIssue {
                rule_id: self.id().to_string(),
                severity: Severity::High,
                message: format!(
                    "PR title '{}' is {} chars — sending a telegram?",
                    trimmed,
                    trimmed.chars().count()
                ),
                pr_id: pr.id.clone(),
                pr_title: pr.title.clone(),
            })
        } else {
            None
        }
    }
}

/// Generic / meaningless single-word title.
static GENERIC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(fix|update|change|modify|refactor|patch|chore|misc|wip|tmp|test)$").unwrap()
});

pub struct GenericTitleRule;

impl PrRule for GenericTitleRule {
    fn id(&self) -> &str {
        "generic-title"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        let trimmed = pr.title.trim();
        if GENERIC_RE.is_match(trimmed) {
            Some(PrIssue {
                rule_id: self.id().to_string(),
                severity: Severity::High,
                message: format!(
                    "PR title is '{}' — are you a robot? Say what you actually changed.",
                    trimmed
                ),
                pr_id: pr.id.clone(),
                pr_title: pr.title.clone(),
            })
        } else {
            None
        }
    }
}

/// Title is only a ticket/issue number (e.g., "PROJ-123", "#456").
static TICKET_ONLY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Z]+-\d+|#\d+)$").unwrap());

pub struct TicketOnlyRule;

impl PrRule for TicketOnlyRule {
    fn id(&self) -> &str {
        "ticket-only"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        let trimmed = pr.title.trim();
        if TICKET_ONLY_RE.is_match(trimmed) {
            Some(PrIssue {
                rule_id: self.id().to_string(),
                severity: Severity::Medium,
                message: format!(
                    "PR title is just '{}' — titles are for humans, not JIRA.",
                    trimmed
                ),
                pr_id: pr.id.clone(),
                pr_title: pr.title.clone(),
            })
        } else {
            None
        }
    }
}

/// WIP / Draft PR title.
static WIP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(wip|draft|do not merge|dnm|work.in.progress)").unwrap());

pub struct WipTitleRule;

impl PrRule for WipTitleRule {
    fn id(&self) -> &str {
        "wip-title"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        let trimmed = pr.title.trim();
        if WIP_RE.is_match(trimmed) {
            Some(PrIssue {
                rule_id: self.id().to_string(),
                severity: Severity::Info,
                message: "WIP PR? Then why open a PR at all?".to_string(),
                pr_id: pr.id.clone(),
                pr_title: pr.title.clone(),
            })
        } else {
            None
        }
    }
}

/// Excessive exclamation marks.
pub struct ExclamationRule;

impl PrRule for ExclamationRule {
    fn id(&self) -> &str {
        "exclamation-marks"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        let count = pr.title.matches('!').count();
        if count >= 3 {
            Some(PrIssue {
                rule_id: self.id().to_string(),
                severity: Severity::Low,
                message: format!("{} exclamation marks? How excited are you?", count),
                pr_id: pr.id.clone(),
                pr_title: pr.title.clone(),
            })
        } else {
            None
        }
    }
}

/// ALL CAPS title.
pub struct AllCapsRule;

impl PrRule for AllCapsRule {
    fn id(&self) -> &str {
        "all-caps"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        let trimmed = pr.title.trim();
        let alpha_chars: String = trimmed.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha_chars.len() >= 3 && alpha_chars == alpha_chars.to_uppercase() {
            Some(PrIssue {
                rule_id: self.id().to_string(),
                severity: Severity::Low,
                message: "ALL CAPS TITLE? STOP SHOUTING!".to_string(),
                pr_id: pr.id.clone(),
                pr_title: pr.title.clone(),
            })
        } else {
            None
        }
    }
}

/// Keyboard mash detection (asdf, qwer, etc.).
static MASH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(asdf|qwer|zxcv| hjkl|aaaa+|xxx+|zzz+)$").unwrap());

pub struct KeyboardMashRule;

impl PrRule for KeyboardMashRule {
    fn id(&self) -> &str {
        "keyboard-mash"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        let trimmed = pr.title.trim();
        if MASH_RE.is_match(trimmed) {
            Some(PrIssue {
                rule_id: self.id().to_string(),
                severity: Severity::Critical,
                message: "Keyboard mash as PR title? You're not testing your keyboard.".to_string(),
                pr_id: pr.id.clone(),
                pr_title: pr.title.clone(),
            })
        } else {
            None
        }
    }
}

/// Title starts with lowercase letter (excluding conventional commits).
static CONVENTIONAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?(!)?:\s")
        .unwrap()
});

pub struct LowercaseStartRule;

impl PrRule for LowercaseStartRule {
    fn id(&self) -> &str {
        "lowercase-start"
    }

    fn check(&self, pr: &PrEntry) -> Option<PrIssue> {
        let trimmed = pr.title.trim();
        if CONVENTIONAL_RE.is_match(trimmed) {
            return None;
        }
        if let Some(first) = trimmed.chars().next() {
            if first.is_ascii_lowercase() && trimmed.len() > 3 {
                return Some(PrIssue {
                    rule_id: self.id().to_string(),
                    severity: Severity::Low,
                    message: format!(
                        "Title starts with lowercase '{}' — proper nouns deserve capital letters.",
                        first
                    ),
                    pr_id: pr.id.clone(),
                    pr_title: pr.title.clone(),
                });
            }
        }
        None
    }
}

/// Get all default PR title rules.
pub fn default_rules() -> Vec<Box<dyn PrRule>> {
    vec![
        Box::new(EmptyTitleRule),
        Box::new(TooShortRule { min_length: 5 }),
        Box::new(GenericTitleRule),
        Box::new(TicketOnlyRule),
        Box::new(WipTitleRule),
        Box::new(ExclamationRule),
        Box::new(AllCapsRule),
        Box::new(KeyboardMashRule),
        Box::new(LowercaseStartRule),
    ]
}

/// Apply all rules to a PR entry and return issues.
pub fn check_pr(pr: &PrEntry) -> Vec<PrIssue> {
    default_rules()
        .iter()
        .filter_map(|rule| rule.check(pr))
        .collect()
}

/// Check multiple PRs and return all issues.
pub fn check_prs(prs: &[PrEntry]) -> Vec<PrIssue> {
    prs.iter().flat_map(check_pr).collect()
}

#[cfg(test)]
mod tests {
    use super::super::types::PrSource;
    use super::*;

    fn make_pr(id: &str, title: &str) -> PrEntry {
        PrEntry {
            id: id.to_string(),
            title: title.to_string(),
            author: None,
            source: PrSource::Local,
        }
    }

    #[test]
    fn test_empty_title() {
        let issues = check_pr(&make_pr("1", ""));
        assert!(issues.iter().any(|i| i.rule_id == "empty-title"));
    }

    #[test]
    fn test_whitespace_title() {
        let issues = check_pr(&make_pr("1", "   "));
        assert!(issues.iter().any(|i| i.rule_id == "empty-title"));
    }

    #[test]
    fn test_too_short() {
        let issues = check_pr(&make_pr("1", "fix"));
        assert!(issues.iter().any(|i| i.rule_id == "too-short"));
    }

    #[test]
    fn test_normal_title_no_issues() {
        let issues = check_pr(&make_pr(
            "1",
            "feat(auth): implement OAuth2 login flow with PKCE",
        ));
        assert!(issues.is_empty());
    }

    #[test]
    fn test_generic_title_fix() {
        let issues = check_pr(&make_pr("1", "fix"));
        // Should trigger both too-short and generic-title (and maybe others)
        assert!(issues.iter().any(|i| i.rule_id == "generic-title"));
    }

    #[test]
    fn test_generic_title_update() {
        let issues = check_pr(&make_pr("1", "update"));
        assert!(issues.iter().any(|i| i.rule_id == "generic-title"));
    }

    #[test]
    fn test_ticket_only() {
        let issues = check_pr(&make_pr("1", "PROJ-123"));
        assert!(issues.iter().any(|i| i.rule_id == "ticket-only"));
    }

    #[test]
    fn test_ticket_hash() {
        let issues = check_pr(&make_pr("1", "#456"));
        assert!(issues.iter().any(|i| i.rule_id == "ticket-only"));
    }

    #[test]
    fn test_wip_title() {
        let issues = check_pr(&make_pr("1", "WIP: new feature"));
        assert!(issues.iter().any(|i| i.rule_id == "wip-title"));
    }

    #[test]
    fn test_draft_title() {
        let issues = check_pr(&make_pr("1", "Draft: refactoring"));
        assert!(issues.iter().any(|i| i.rule_id == "wip-title"));
    }

    #[test]
    fn test_exclamation_marks() {
        let issues = check_pr(&make_pr("1", "fix the bug!!!"));
        assert!(issues.iter().any(|i| i.rule_id == "exclamation-marks"));
    }

    #[test]
    fn test_all_caps() {
        let issues = check_pr(&make_pr("1", "FIX ALL THE THINGS"));
        assert!(issues.iter().any(|i| i.rule_id == "all-caps"));
    }

    #[test]
    fn test_keyboard_mash() {
        let issues = check_pr(&make_pr("1", "asdf"));
        assert!(issues.iter().any(|i| i.rule_id == "keyboard-mash"));
    }

    #[test]
    fn test_lowercase_start() {
        let issues = check_pr(&make_pr("1", "fix the login bug"));
        assert!(issues.iter().any(|i| i.rule_id == "lowercase-start"));
    }

    #[test]
    fn test_conventional_commit_ok() {
        let issues = check_pr(&make_pr("1", "feat(api): add user search endpoint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_prs_multiple() {
        let prs = vec![
            make_pr("1", "feat: good title"),
            make_pr("2", "fix"),
            make_pr("3", "asdf"),
        ];
        let issues = check_prs(&prs);
        assert!(issues.len() >= 3); // At least one issue per bad PR
    }
}
