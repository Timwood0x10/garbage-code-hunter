//! Roast provider abstraction for generating code review messages.
//!
//! This module defines the `RoastProvider` trait and two implementations:
//! - `LocalRoastProvider`: Uses hardcoded roast messages from the i18n module.
//! - `LlmRoastProvider`: Calls an LLM endpoint to generate dynamic, context-aware roasts.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::analyzer::CodeIssue;
use crate::i18n::I18n;

use super::client::{LlmClient, LlmConfig};
use super::prompt::build_roast_prompt;

/// A map from issue key to roast message.
///
/// Issue key format: `"{file_path}:{line}:{rule_name}"`
pub type RoastMap = HashMap<String, String>;

/// Trait for generating roast messages for code issues.
///
/// Implementors can use local hardcoded messages or call external LLM services.
pub trait RoastProvider {
    /// Generate roast messages for the given issues.
    ///
    /// Returns a `RoastMap` mapping issue keys to roast messages.
    fn generate_roasts(&self, issues: &[CodeIssue], lang: &str) -> RoastMap;
}

/// Local roast provider using hardcoded messages from the i18n module.
///
/// This is the default provider and serves as the fallback when LLM calls fail.
pub struct LocalRoastProvider;

impl RoastProvider for LocalRoastProvider {
    fn generate_roasts(&self, issues: &[CodeIssue], lang: &str) -> RoastMap {
        let i18n = I18n::new(lang);
        let mut map = RoastMap::new();

        for issue in issues {
            let key = format!(
                "{}:{}:{}",
                issue.file_path.display(),
                issue.line,
                issue.rule_name
            );
            let messages = i18n.get_roast_messages(&issue.rule_name);
            let roast = if !messages.is_empty() {
                messages[issue.line % messages.len()].clone()
            } else {
                issue.message.clone()
            };
            map.insert(key, roast);
        }

        map
    }
}

/// LLM-powered roast provider that generates dynamic, context-aware roasts.
///
/// Falls back to `LocalRoastProvider` if the LLM call fails or returns invalid data.
pub struct LlmRoastProvider {
    client: LlmClient,
    fallback: LocalRoastProvider,
}

impl LlmRoastProvider {
    /// Create a new LLM roast provider with the given configuration.
    pub fn new(config: LlmConfig) -> Self {
        Self {
            client: LlmClient::new(config),
            fallback: LocalRoastProvider,
        }
    }
}

impl RoastProvider for LlmRoastProvider {
    fn generate_roasts(&self, issues: &[CodeIssue], lang: &str) -> RoastMap {
        let contexts = extract_code_contexts(issues);
        let prompt = build_roast_prompt(issues, &contexts, lang);

        match self.client.call_blocking(&prompt) {
            Ok(response) => match parse_llm_response(&response, issues) {
                Ok(roasts) => roasts,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse LLM response: {}. Falling back to local roasts.",
                        e
                    );
                    self.fallback.generate_roasts(issues, lang)
                }
            },
            Err(e) => {
                eprintln!(
                    "Warning: LLM call failed: {}. Falling back to local roasts.",
                    e
                );
                self.fallback.generate_roasts(issues, lang)
            }
        }
    }
}

/// Extract code context (±5 lines) around each issue for the LLM prompt.
///
/// Groups issues by file to avoid reading the same file multiple times.
fn extract_code_contexts(issues: &[CodeIssue]) -> HashMap<String, String> {
    // Collect unique file paths
    let file_paths: Vec<PathBuf> = issues
        .iter()
        .map(|i| i.file_path.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Read all file contents upfront
    let file_contents: HashMap<PathBuf, Vec<String>> = file_paths
        .into_iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(&path).ok()?;
            let lines: Vec<String> = content.lines().map(String::from).collect();
            Some((path, lines))
        })
        .collect();

    // Extract context window for each issue
    let mut contexts = HashMap::new();
    for issue in issues {
        let key = format!(
            "{}:{}:{}",
            issue.file_path.display(),
            issue.line,
            issue.rule_name
        );

        if let Some(lines) = file_contents.get(&issue.file_path) {
            let start = issue.line.saturating_sub(6);
            let end = (issue.line + 5).min(lines.len());
            let context: String = lines[start..end]
                .iter()
                .enumerate()
                .map(|(i, l)| format!("{:>4} | {}", start + i + 1, l))
                .collect::<Vec<_>>()
                .join("\n");
            contexts.insert(key, context);
        }
    }

    contexts
}

/// Parse the LLM response JSON into a RoastMap.
///
/// Expected format: `{"0": "roast message", "1": "roast message", ...}`
/// where keys are issue indices (0-based) matching the order in the prompt.
fn parse_llm_response(response: &str, issues: &[CodeIssue]) -> Result<RoastMap, anyhow::Error> {
    let json_str = extract_json_from_response(response);
    let parsed: HashMap<String, String> = serde_json::from_str(json_str)?;

    let mut roasts = RoastMap::new();
    for (idx_str, roast) in parsed {
        let Ok(idx) = idx_str.parse::<usize>() else {
            continue;
        };
        if idx >= issues.len() {
            continue;
        }
        let issue = &issues[idx];
        let key = format!(
            "{}:{}:{}",
            issue.file_path.display(),
            issue.line,
            issue.rule_name
        );
        roasts.insert(key, roast);
    }

    Ok(roasts)
}

/// Extract JSON from LLM response, handling markdown code fences and plain JSON.
fn extract_json_from_response(response: &str) -> &str {
    // Handle ```json ... ``` wrapper
    if let Some(start) = response.find("```json") {
        let json_start = start + 7;
        if let Some(end) = response[json_start..].find("```") {
            return response[json_start..json_start + end].trim();
        }
    }

    // Handle plain JSON object
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            return &response[start..=end];
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Severity;

    /// Helper to create a test CodeIssue with minimal fields.
    fn make_issue(rule: &str, line: usize) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line,
            column: 1,
            rule_name: rule.to_string(),
            message: "test message".to_string(),
            severity: Severity::Spicy,
        }
    }

    #[test]
    fn test_extract_json_from_plain_object() {
        // Objective: Verify plain JSON objects are extracted correctly.
        // Invariants: Output must match the input when it is a valid JSON object.
        let response = r#"{"0": "roast one", "1": "roast two"}"#;
        let result = extract_json_from_response(response);
        assert_eq!(result, response, "Plain JSON should be returned as-is");
    }

    #[test]
    fn test_extract_json_from_markdown_fence() {
        // Objective: Verify JSON wrapped in ```json fences is extracted.
        // Invariants: Only the JSON content between fences is returned.
        let response = "Here is the JSON:\n```json\n{\"0\": \"roast\"}\n```\nDone.";
        let result = extract_json_from_response(response);
        assert_eq!(
            result, "{\"0\": \"roast\"}",
            "JSON inside markdown fences should be extracted"
        );
    }

    #[test]
    fn test_parse_response_maps_indices_to_issue_keys() {
        // Objective: Verify LLM response indices map to correct issue keys.
        // Invariants: Each index maps to the corresponding issue's key format.
        let issues = vec![
            make_issue("unwrap-abuse", 10),
            make_issue("deep-nesting", 25),
        ];
        let response = r#"{"0": "nice unwrap", "1": "so deep"}"#;
        let roasts = parse_llm_response(response, &issues).unwrap();

        assert_eq!(roasts.len(), 2, "Should have roasts for both issues");
        assert!(
            roasts.contains_key("test.rs:10:unwrap-abuse"),
            "First issue key must be test.rs:10:unwrap-abuse"
        );
        assert!(
            roasts.contains_key("test.rs:25:deep-nesting"),
            "Second issue key must be test.rs:25:deep-nesting"
        );
    }

    #[test]
    fn test_parse_response_skips_out_of_range_indices() {
        // Objective: Verify out-of-range indices are silently ignored.
        // Invariants: Only valid indices produce roasts; invalid ones are skipped.
        let issues = vec![make_issue("unwrap-abuse", 10)];
        let response = r#"{"0": "valid", "5": "out of range", "abc": "not a number"}"#;
        let roasts = parse_llm_response(response, &issues).unwrap();

        assert_eq!(
            roasts.len(),
            1,
            "Only the valid index should produce a roast"
        );
        assert!(
            roasts.contains_key("test.rs:10:unwrap-abuse"),
            "Valid index 0 should map to the first issue"
        );
    }

    #[test]
    fn test_local_provider_returns_roasts_for_known_rules() {
        // Objective: Verify LocalRoastProvider produces roasts for rules with i18n messages.
        // Invariants: At least one roast must be returned for a known rule name.
        let issues = vec![make_issue("unwrap-abuse", 1)];
        let provider = LocalRoastProvider;
        let roasts = provider.generate_roasts(&issues, "en-US");

        assert!(
            !roasts.is_empty(),
            "LocalRoastProvider must return at least one roast for known rules"
        );
        assert!(
            roasts.contains_key("test.rs:1:unwrap-abuse"),
            "Roast key must match the issue key format"
        );
    }

    #[test]
    fn test_local_provider_returns_something_for_unknown_rules() {
        // Objective: Verify unknown rules still produce a roast message.
        // Invariants: The i18n module returns a catch-all message for unknown rules.
        let issues = vec![make_issue("unknown-rule-xyz", 42)];
        let provider = LocalRoastProvider;
        let roasts = provider.generate_roasts(&issues, "en-US");

        assert_eq!(
            roasts.len(),
            1,
            "Should have exactly one roast for one issue"
        );
        let roast = roasts.get("test.rs:42:unknown-rule-xyz").unwrap();
        assert!(
            !roast.is_empty(),
            "Unknown rules must still produce a non-empty roast message"
        );
    }

    #[test]
    fn test_parse_response_with_markdown_wrapped_json() {
        // Objective: Verify end-to-end parsing with markdown-wrapped LLM output.
        // Invariants: JSON inside code fences must parse correctly.
        let issues = vec![make_issue("deep-nesting", 5)];
        let response =
            "Sure, here are the roasts:\n```json\n{\"0\": \"nested deeper than inception\"}\n```";
        let roasts = parse_llm_response(response, &issues).unwrap();

        assert_eq!(roasts.len(), 1, "Should parse one roast from fenced JSON");
        let roast = roasts.get("test.rs:5:deep-nesting").unwrap();
        assert_eq!(
            roast, "nested deeper than inception",
            "Roast content must match the JSON value"
        );
    }
}
