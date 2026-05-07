//! Prompt construction for LLM-powered roast generation.
//!
//! Builds a single prompt containing all issues with code context,
//! instructing the LLM to generate sarcastic roasts in JSON format.

use std::collections::HashMap;

use crate::analyzer::{CodeIssue, Severity};

/// Maximum number of issues to send to the LLM in a single prompt.
/// Prevents context window overflow for large codebases.
const MAX_ISSUES_FOR_LLM: usize = 50;

/// Build a single prompt that instructs the LLM to generate roasts for all issues.
///
/// The prompt includes:
/// - Personality instructions for the "Garbage Code Hunter" persona
/// - Language-specific output instructions
/// - Code context (±5 lines) for each issue
/// - JSON output format specification
pub fn build_roast_prompt(
    issues: &[CodeIssue],
    code_contexts: &HashMap<String, String>,
    lang: &str,
) -> String {
    let lang_instruction = if lang == "zh-CN" {
        "请用中文回复所有 roast 消息。"
    } else {
        "Respond in English."
    };

    // Cap issues to avoid exceeding LLM context window
    let issues_to_send = if issues.len() > MAX_ISSUES_FOR_LLM {
        &issues[..MAX_ISSUES_FOR_LLM]
    } else {
        issues
    };

    let issues_text = build_issues_section(issues_to_send, code_contexts);

    format!(
        r#"You are "Garbage Code Hunter", a brutally sarcastic and witty code reviewer. Your personality is a mix of a stand-up comedian and a disappointed senior developer. You find creative, hilarious ways to roast bad code while still being technically accurate.

{lang_instruction}

Your task: For each code issue below, generate a unique, creative, and sarcastic roast message. The roast should:
1. Be specific to the actual code shown in the context (not generic)
2. Be funny, witty, and memorable
3. Reference the actual variable names, function names, or patterns in the code
4. Be technically accurate about WHY the code is bad
5. Match the severity: Nuclear issues get savage roasts, Mild issues get gentle ribbing
6. Be 1-2 sentences max

IMPORTANT: Respond ONLY with a valid JSON object mapping issue indices (as strings) to roast messages.
Format: {{"0": "roast message for issue 0", "1": "roast message for issue 1", ...}}

Here are the issues to roast:

{issues_text}

Remember: Be creative, specific to the code shown, and hilariously savage. Each roast should feel like it was written by someone who actually read the code and is personally offended by it.

Respond with JSON only:"#,
        lang_instruction = lang_instruction,
        issues_text = issues_text,
    )
}

/// Build the issues section of the prompt with code context for each issue.
fn build_issues_section(issues: &[CodeIssue], code_contexts: &HashMap<String, String>) -> String {
    let mut sections = Vec::with_capacity(issues.len());

    for (idx, issue) in issues.iter().enumerate() {
        let key = format!(
            "{}:{}:{}",
            issue.file_path.display(),
            issue.line,
            issue.rule_name
        );
        let context = code_contexts
            .get(&key)
            .map(String::as_str)
            .unwrap_or("(context unavailable)");

        let severity_str = match issue.severity {
            Severity::Nuclear => "Nuclear (critical)",
            Severity::Spicy => "Spicy (moderate)",
            Severity::Mild => "Mild (minor)",
        };

        sections.push(format!(
            "---\nIssue #{idx}:\n  File: {}\n  Line: {}\n  Rule: {}\n  Severity: {}\n  Message: {}\n  Code context:\n```\n{}\n```",
            issue.file_path.display(),
            issue.line,
            issue.rule_name,
            severity_str,
            issue.message,
            context,
        ));
    }

    sections.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Severity;
    use std::path::PathBuf;

    fn make_issue(rule: &str, line: usize, severity: Severity) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("src/main.rs"),
            line,
            column: 1,
            rule_name: rule.to_string(),
            message: format!("issue with {}", rule),
            severity,
        }
    }

    #[test]
    fn test_prompt_contains_all_issues() {
        // Objective: Verify every issue appears in the generated prompt.
        // Invariants: Each issue's rule name and file path must be present.
        let issues = vec![
            make_issue("unwrap-abuse", 10, Severity::Nuclear),
            make_issue("deep-nesting", 20, Severity::Spicy),
        ];
        let contexts = HashMap::new();

        let prompt = build_roast_prompt(&issues, &contexts, "en-US");

        assert!(
            prompt.contains("unwrap-abuse"),
            "Prompt must contain the first issue's rule name"
        );
        assert!(
            prompt.contains("deep-nesting"),
            "Prompt must contain the second issue's rule name"
        );
        assert!(
            prompt.contains("src/main.rs"),
            "Prompt must contain the file path"
        );
    }

    #[test]
    fn test_prompt_uses_chinese_instruction_for_zh_cn() {
        // Objective: Verify Chinese language produces Chinese instruction.
        // Invariants: zh-CN prompt must contain Chinese characters.
        let issues = vec![make_issue("test-rule", 1, Severity::Mild)];
        let contexts = HashMap::new();

        let prompt = build_roast_prompt(&issues, &contexts, "zh-CN");

        assert!(
            prompt.contains("请用中文"),
            "zh-CN prompt must contain Chinese language instruction"
        );
    }

    #[test]
    fn test_prompt_uses_english_instruction_for_en_us() {
        // Objective: Verify English language produces English instruction.
        // Invariants: en-US prompt must contain "Respond in English".
        let issues = vec![make_issue("test-rule", 1, Severity::Mild)];
        let contexts = HashMap::new();

        let prompt = build_roast_prompt(&issues, &contexts, "en-US");

        assert!(
            prompt.contains("Respond in English"),
            "en-US prompt must contain English language instruction"
        );
    }

    #[test]
    fn test_prompt_includes_code_context_when_available() {
        // Objective: Verify code context is embedded in the prompt.
        // Invariants: Context text must appear verbatim in the prompt.
        let issues = vec![make_issue("unwrap-abuse", 10, Severity::Nuclear)];
        let mut contexts = HashMap::new();
        contexts.insert(
            "src/main.rs:10:unwrap-abuse".to_string(),
            "   9 | let x = Some(42);\n  10 | let y = x.unwrap();\n  11 | println!(\"{}\", y);"
                .to_string(),
        );

        let prompt = build_roast_prompt(&issues, &contexts, "en-US");

        assert!(
            prompt.contains("x.unwrap()"),
            "Prompt must include the code context verbatim"
        );
    }

    #[test]
    fn test_prompt_caps_issues_at_max() {
        // Objective: Verify prompt generation handles large issue lists without panic.
        // Invariants: Must not panic with more than MAX_ISSUES_FOR_LLM issues.
        let issues: Vec<CodeIssue> = (0..100)
            .map(|i| make_issue("test-rule", i, Severity::Mild))
            .collect();
        let contexts = HashMap::new();

        // Should not panic even with 100 issues
        let prompt = build_roast_prompt(&issues, &contexts, "en-US");
        assert!(
            !prompt.is_empty(),
            "Prompt must be generated even with many issues"
        );
    }

    #[test]
    fn test_prompt_requests_json_output() {
        // Objective: Verify the prompt explicitly requests JSON format.
        // Invariants: Must contain JSON format instruction.
        let issues = vec![make_issue("test", 1, Severity::Mild)];
        let prompt = build_roast_prompt(&issues, &HashMap::new(), "en-US");

        assert!(
            prompt.contains("JSON"),
            "Prompt must request JSON output format"
        );
    }

    #[test]
    fn test_severity_appears_in_prompt() {
        // Objective: Verify severity levels are included in the prompt.
        // Invariants: Each severity variant must appear as a string.
        let issues = vec![
            make_issue("a", 1, Severity::Nuclear),
            make_issue("b", 2, Severity::Spicy),
            make_issue("c", 3, Severity::Mild),
        ];
        let prompt = build_roast_prompt(&issues, &HashMap::new(), "en-US");

        assert!(prompt.contains("Nuclear"), "Must include Nuclear severity");
        assert!(prompt.contains("Spicy"), "Must include Spicy severity");
        assert!(prompt.contains("Mild"), "Must include Mild severity");
    }
}
