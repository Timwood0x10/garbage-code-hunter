//! Commit message rules and matching engine.
//!
//! Defines rules for detecting bad commit message patterns
//! and provides a matching engine to evaluate commits.

use crate::common::Severity;
use serde::Deserialize;

/// Pattern type for rule matching.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Pattern {
    /// Match via regex.
    Regex(String),
    /// Match if message contains substring.
    Contains(String),
    /// Match if message starts with prefix.
    StartsWith(String),
    /// Match if message length is within bounds.
    Length {
        min: Option<usize>,
        max: Option<usize>,
    },
}

/// A single commit message rule.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub pattern: Pattern,
    pub message: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Collection of rules loaded from TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

impl RuleSet {
    /// Load rules from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Returns only enabled rules.
    pub fn active_rules(&self) -> Vec<&Rule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }
}

/// A matched issue from a commit message.
#[derive(Debug, Clone)]
pub struct Issue {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
}

/// Match a commit message against all rules in the set.
pub fn match_rules(message: &str, rules: &[&Rule]) -> Vec<Issue> {
    let mut issues = Vec::new();
    let trimmed = message.trim();

    for rule in rules {
        let matched = match &rule.pattern {
            Pattern::Regex(pattern) => regex::Regex::new(pattern)
                .map(|re| re.is_match(trimmed))
                .unwrap_or(false),
            Pattern::Contains(sub) => trimmed.contains(sub.as_str()),
            Pattern::StartsWith(prefix) => trimmed.starts_with(prefix.as_str()),
            Pattern::Length { min, max } => {
                let len = trimmed.len();
                let above_min = min.is_none_or(|m| len >= m);
                let below_max = max.is_none_or(|m| len <= m);
                above_min && below_max
            }
        };

        if matched {
            issues.push(Issue {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                severity: rule.severity,
                message: rule.message.clone(),
            });
        }
    }

    issues
}

/// Returns the default built-in rules as a TOML string.
pub fn default_rules_toml() -> &'static str {
    include_str!("rules/commit_rules.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_default_rules() -> RuleSet {
        RuleSet::from_toml(default_rules_toml()).expect("Failed to parse default rules TOML")
    }

    #[test]
    fn test_parse_default_rules() {
        let ruleset = load_default_rules();
        assert!(
            !ruleset.rules.is_empty(),
            "Default rules should not be empty"
        );
    }

    #[test]
    fn test_empty_message_matches() {
        let ruleset = load_default_rules();
        let active = ruleset.active_rules();
        let issues = match_rules("", &active);
        assert!(
            issues.iter().any(|i| i.rule_id == "empty-message"),
            "Empty message should trigger empty-message rule"
        );
    }

    #[test]
    fn test_short_message_matches() {
        let ruleset = load_default_rules();
        let active = ruleset.active_rules();
        let issues = match_rules("fix", &active);
        assert!(
            issues.iter().any(|i| i.rule_id == "too-short"),
            "Short message should trigger too-short rule"
        );
    }

    #[test]
    fn test_wip_message_matches() {
        let ruleset = load_default_rules();
        let active = ruleset.active_rules();
        let issues = match_rules("WIP: working on auth", &active);
        assert!(
            issues.iter().any(|i| i.rule_id == "wip-commit"),
            "WIP message should trigger wip-commit rule"
        );
    }

    #[test]
    fn test_good_message_no_issues() {
        let ruleset = load_default_rules();
        let active = ruleset.active_rules();
        let issues = match_rules("feat(auth): implement OAuth2 login flow with PKCE", &active);
        let critical_or_high: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Critical || i.severity == Severity::High)
            .collect();
        assert!(
            critical_or_high.is_empty(),
            "Good message should not trigger critical/high rules, got: {:?}",
            critical_or_high
        );
    }

    #[test]
    fn test_disabled_rules_not_matched() {
        let toml = r#"
[[rules]]
id = "test-rule"
name = "Test"
description = "Test rule"
severity = "High"
pattern = { type = "Contains", value = "test" }
message = "Test matched"
enabled = false
"#;
        let ruleset = RuleSet::from_toml(toml).unwrap();
        let active = ruleset.active_rules();
        let issues = match_rules("this is a test", &active);
        assert!(issues.is_empty(), "Disabled rules should not be matched");
    }
}
