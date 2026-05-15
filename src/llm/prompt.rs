use std::collections::HashMap;

use crate::analyzer::{CodeIssue, Severity};

const MAX_ISSUES_FOR_LLM: usize = 50;

/// Detect programming language from file extension for persona selection.
fn detect_lang_from_path(path: &str) -> &'static str {
    if path.ends_with(".rs") {
        "Rust"
    } else if path.ends_with(".py") {
        "Python"
    } else if path.ends_with(".js") || path.ends_with(".ts") || path.ends_with(".tsx") {
        "JavaScript"
    } else if path.ends_with(".go") {
        "Go"
    } else if path.ends_with(".java") {
        "Java"
    } else if path.ends_with(".rb") {
        "Ruby"
    } else if path.ends_with(".c") || path.ends_with(".h") {
        "C"
    } else if path.ends_with(".cpp")
        || path.ends_with(".cxx")
        || path.ends_with(".cc")
        || path.ends_with(".hpp")
        || path.ends_with(".hh")
    {
        "C++"
    } else {
        "Unknown"
    }
}

/// Get the most common language across all issues for the main roast persona.
fn dominant_language(issues: &[CodeIssue]) -> &'static str {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for issue in issues {
        let lang = detect_lang_from_path(&issue.file_path.to_string_lossy());
        *counts.entry(lang).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .map(|(l, _)| l)
        .unwrap_or("Unknown")
}

/// Language-specific roast persona lines.
fn language_persona(lang: &str) -> &'static str {
    match lang {
        "Python" => "Roast Python code with PEP 8 pedantry and 'there should be one way to do it' energy.",
        "JavaScript" => "Roast JavaScript code with the fury of a developer who's debugged 'undefined is not a function' at 3 AM.",
        "Rust" => "Roast Rust code as a disappointed compiler that expectedly yells 'but muh zero-cost abstractions!'",
        "Go" => "Roast Go code with the passive-aggression of a code reviewer who wanted generics for 10 years.",
        "Java" => "Roast Java code with the exhaustion of someone who's written 50 lines to print 'hello world'.",
        "C" => "Roast C code like a systems programmer who's seen one too many buffer overflows.",
        "C++" => "Roast C++ code as someone who's read the error message 'std::__cxx11::basic_string' one too many times.",
        "Ruby" => "Roast Ruby code like a developer who found 5 ways to write the same method.",
        _ => "Roast with technical accuracy and brutal honesty.",
    }
}

pub fn build_roast_prompt(
    issues: &[CodeIssue],
    code_contexts: &HashMap<String, String>,
    lang: &str,
) -> String {
    let lang_instruction = if lang == "zh-CN" {
        "请用中文回复所有 roast 消息。使用中文编程梗和文化参考。"
    } else {
        "Respond in English. Use programming cultural references relevant to the code's language."
    };

    let dominant = dominant_language(issues);
    let lang_persona = language_persona(dominant);

    let issues_to_send = if issues.len() > MAX_ISSUES_FOR_LLM {
        &issues[..MAX_ISSUES_FOR_LLM]
    } else {
        issues
    };

    let issues_text = build_issues_section(issues_to_send, code_contexts);

    format!(
        r#"You are "Garbage Code Hunter", a brutally sarcastic and witty code reviewer.
Your personality is a mix of a stand-up comedian who's seen every coding sin,
and a disappointed senior developer who's tired of fixing the same mistakes.

{lang_instruction}

The code being analyzed is primarily written in {dominant_lang}.

{lang_persona}

For each issue below, generate a UNIQUE roast message. Guidelines:

1. BE SPECIFIC: Reference actual variable names, function names, values from the code context.
2. STYLE BY RULE TYPE:
{rule_styles}
3. SEVERITY TONE:
   - Nuclear (critical): devastating, savage, leave no survivors
   - Spicy (moderate): clever mockery, raised eyebrows
   - Mild (minor): gentle teasing, knowing smirk
4. LENGTH: 1-2 sentences per roast. Punchy. Memorable.
5. UNIQUE: Every issue gets a different roast. NO repeats.

IMPORTANT: Respond ONLY with valid JSON: {{"0": "roast for issue 0", "1": "roast for issue 1", ...}}

Issues:

{issues_text}

Respond with JSON only:"#,
        lang_instruction = lang_instruction,
        dominant_lang = dominant,
        lang_persona = lang_persona,
        rule_styles = "      - Naming issues → sarcasm about creativity\n      - Safety issues → warnings about recklessness\n      - Complexity issues → mock the over-engineering\n      - Duplication issues → copy-paste shaming\n      - Performance issues → waste analogies\n      - Dead/commented code → digital hoarding jokes",
        issues_text = issues_text,
    )
}

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

    fn make_issue(path: &str, rule: &str, line: usize, severity: Severity) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from(path),
            line,
            column: 1,
            rule_name: rule.to_string(),
            message: format!("issue with {}", rule),
            severity,
        }
    }

    #[test]
    fn test_dominant_language() {
        let issues = vec![
            make_issue("a.rs", "test", 1, Severity::Mild),
            make_issue("b.rs", "test", 2, Severity::Mild),
            make_issue("c.py", "test", 3, Severity::Mild),
        ];
        assert_eq!(dominant_language(&issues), "Rust");
    }

    #[test]
    fn test_language_persona_not_empty() {
        for lang in &[
            "Rust",
            "Python",
            "JavaScript",
            "C",
            "C++",
            "Go",
            "Java",
            "Ruby",
            "Unknown",
        ] {
            assert!(
                !language_persona(lang).is_empty(),
                "{} should have a persona",
                lang
            );
        }
    }

    #[test]
    fn test_prompt_contains_all_issues() {
        let issues = vec![
            make_issue("src/main.rs", "unwrap-abuse", 10, Severity::Nuclear),
            make_issue("src/main.rs", "deep-nesting", 20, Severity::Spicy),
        ];
        let contexts = HashMap::new();
        let prompt = build_roast_prompt(&issues, &contexts, "en-US");
        assert!(prompt.contains("unwrap-abuse"));
        assert!(prompt.contains("deep-nesting"));
    }

    #[test]
    fn test_prompt_uses_chinese_instruction() {
        let issues = vec![make_issue("test.rs", "test-rule", 1, Severity::Mild)];
        let prompt = build_roast_prompt(&issues, &HashMap::new(), "zh-CN");
        assert!(prompt.contains("请用中文"));
    }

    #[test]
    fn test_prompt_includes_code_context() {
        let issues = vec![make_issue("main.rs", "unwrap-abuse", 10, Severity::Nuclear)];
        let mut contexts = HashMap::new();
        contexts.insert(
            "main.rs:10:unwrap-abuse".to_string(),
            "   9 | let x = Some(42);\n  10 | let y = x.unwrap();".to_string(),
        );
        let prompt = build_roast_prompt(&issues, &contexts, "en-US");
        assert!(prompt.contains("x.unwrap()"));
    }

    #[test]
    fn test_prompt_requests_json() {
        let prompt = build_roast_prompt(
            &[make_issue("test.rs", "test", 1, Severity::Mild)],
            &HashMap::new(),
            "en-US",
        );
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_prompt_caps_issues() {
        let issues: Vec<CodeIssue> = (0..100)
            .map(|i| make_issue("test.rs", "test-rule", i, Severity::Mild))
            .collect();
        let prompt = build_roast_prompt(&issues, &HashMap::new(), "en-US");
        assert!(!prompt.is_empty());
    }

    #[test]
    fn test_prompt_detects_language_context() {
        let issues = vec![
            make_issue("script.py", "terrible-naming", 10, Severity::Spicy),
            make_issue("script.py", "magic-number", 20, Severity::Mild),
        ];
        let prompt = build_roast_prompt(&issues, &HashMap::new(), "en-US");
        assert!(
            prompt.contains("Python"),
            "Should detect Python as dominant language"
        );
    }
}
