//! Persona Modes — different roast personalities for code analysis.

use crate::analyzer::{CodeAnalyzer, CodeIssue};
use crate::common::OutputFormat;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// Available roast personas.
#[derive(Debug, Clone, Copy)]
pub enum Persona {
    LinuxKernel,
    SiliconValley,
    JapaneseEnterprise,
    RustFanatic,
}

impl Persona {
    pub fn parse_persona(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "linux-kernel" | "linux" => Some(Self::LinuxKernel),
            "silicon-valley" | "sv" => Some(Self::SiliconValley),
            "japanese-enterprise" | "jp" => Some(Self::JapaneseEnterprise),
            "rust-fanatic" | "rust" => Some(Self::RustFanatic),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::LinuxKernel => "Grumpy Linux Kernel Maintainer",
            Self::SiliconValley => "Silicon Valley Bro CTO",
            Self::JapaneseEnterprise => "Japanese Enterprise Engineer",
            Self::RustFanatic => "Rust Evangelist",
        }
    }

    pub fn intro(&self) -> &'static str {
        match self {
            Self::LinuxKernel => "Who wrote this garbage?",
            Self::SiliconValley => "Bro, this code has zero 10x energy.",
            Self::JapaneseEnterprise => {
                "The following code contains areas for improvement. (It's actually terrible.)"
            }
            Self::RustFanatic => "unsafe detected. friendship terminated.",
        }
    }
}

/// Run persona-based analysis.
pub fn run(path: &Path, persona: Persona, format: &OutputFormat, lang: &str) -> Result<String> {
    let analyzer = CodeAnalyzer::new(&[], lang);
    let issues = analyzer.analyze_path(path);

    let output = match format {
        OutputFormat::Terminal => format_terminal(&issues, persona, lang),
        OutputFormat::Json => format_json(&issues, persona),
    };

    Ok(output)
}

fn format_terminal(issues: &[CodeIssue], persona: Persona, _lang: &str) -> String {
    let mut out = String::new();

    out.push_str(&format!("\n{} {}\n", persona.name().bold(), "\u{1f3ad}"));
    out.push_str(&format!("{}\n\n", "\u{2501}".repeat(40)));
    out.push_str(&format!("  \"{}\"\n\n", persona.intro().italic()));

    if issues.is_empty() {
        out.push_str(&format!("  {}\n", persona.no_issues_roast()));
        return out;
    }

    // Roast top issues
    for (i, issue) in issues.iter().take(10).enumerate() {
        let roast = persona.roast_issue(issue, i);
        let file_short = issue
            .file_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| issue.file_path.display().to_string());
        out.push_str(&format!(
            "  {}. {}:{} — {}\n",
            i + 1,
            file_short.dimmed(),
            issue.line,
            roast
        ));
    }

    out.push_str(&format!("\n  {}\n", persona.closing()));

    out
}

fn format_json(issues: &[CodeIssue], persona: Persona) -> String {
    serde_json::json!({
        "persona": persona.name(),
        "intro": persona.intro(),
        "total_issues": issues.len(),
        "roasts": issues.iter().take(10).enumerate().map(|(i, issue)| {
            serde_json::json!({
                "file": issue.file_path.display().to_string(),
                "line": issue.line,
                "rule": issue.rule_name,
                "roast": persona.roast_issue(issue, i),
            })
        }).collect::<Vec<_>>(),
        "closing": persona.closing(),
    })
    .to_string()
}

impl Persona {
    fn no_issues_roast(&self) -> &'static str {
        match self {
            Self::LinuxKernel => {
                "No issues? Either the code is actually good, or my scanner is broken."
            }
            Self::SiliconValley => {
                "Clean code? This doesn't look like it was built in a hackathon."
            }
            Self::JapaneseEnterprise => {
                "No issues found. Please submit form 27B/6 to verify this is correct."
            }
            Self::RustFanatic => {
                "No issues detected. This code has been blessed by the borrow checker."
            }
        }
    }

    fn roast_issue(&self, issue: &CodeIssue, _index: usize) -> String {
        let rule = &issue.rule_name;
        match self {
            Self::LinuxKernel => match rule.to_lowercase().as_str() {
                r if r.contains("unwrap") => {
                    "Who taught you error handling? A toddler?".to_string()
                }
                r if r.contains("name") => {
                    "Single-letter variables are for mathematicians, not programmers.".to_string()
                }
                r if r.contains("nest") => {
                    "Your indentation is deeper than my disappointment.".to_string()
                }
                r if r.contains("magic") => {
                    "Magic numbers? This isn't a card game. Use constants.".to_string()
                }
                r if r.contains("duplicat") => {
                    "Copy-paste is not a design pattern. Refactor this mess.".to_string()
                }
                r if r.contains("long") => {
                    "This function is longer than my patience. Split it.".to_string()
                }
                _ => "This would get you banned from LKML.".to_string(),
            },
            Self::SiliconValley => match rule.to_lowercase().as_str() {
                r if r.contains("unwrap") => {
                    "Bro, just `.unwrap()` everything? That's not very blockchain of you."
                        .to_string()
                }
                r if r.contains("name") => {
                    "Variables named `x`? Not very 10x engineer of you.".to_string()
                }
                r if r.contains("nest") => {
                    "This nesting is deeper than our seed round cap table.".to_string()
                }
                r if r.contains("magic") => {
                    "Random numbers in code? That's not how you do A/B testing, bro.".to_string()
                }
                r if r.contains("duplicat") => {
                    "Duplication? That's not scaling, that's copy-paste with extra steps.".to_string()
                }
                r if r.contains("long") => {
                    "This function is longer than our last pivot pitch deck.".to_string()
                }
                _ => "This code wouldn't pass a Series A due diligence.".to_string(),
            },
            Self::JapaneseEnterprise => match rule.to_lowercase().as_str() {
                r if r.contains("unwrap") => {
                    "The unwrap() usage suggests a certain... boldness. (Please fix immediately.)"
                        .to_string()
                }
                r if r.contains("name") => {
                    "Variable naming could benefit from additional consideration. (It's terrible.)"
                        .to_string()
                }
                r if r.contains("nest") => {
                    "The indentation level indicates room for structural improvement. (Rewrite it.)"
                        .to_string()
                }
                r if r.contains("magic") => {
                    "Unexplained numbers detected. Please submit a constants specification document. (Seriously.)"
                        .to_string()
                }
                r if r.contains("duplicat") => {
                    "Code duplication found. This violates section 4.2 of our coding standards. (How did this pass review?)"
                        .to_string()
                }
                r if r.contains("long") => {
                    "This function exceeds the recommended line count. (It exceeds all reasonable limits.)"
                        .to_string()
                }
                _ => "This code has potential for growth. (It needs to be completely rewritten.)"
                    .to_string(),
            },
            Self::RustFanatic => match rule.to_lowercase().as_str() {
                r if r.contains("unwrap") => {
                    "UNSAFE DETECTED. The borrow checker weeps.".to_string()
                }
                r if r.contains("name") => {
                    "Meaningless names? This is why people still use Go.".to_string()
                }
                r if r.contains("nest") => {
                    "This nesting would be unnecessary with proper pattern matching.".to_string()
                }
                r if r.contains("magic") => {
                    "Magic numbers? A true Rustacean uses const and type-safe enums.".to_string()
                }
                r if r.contains("duplicat") => {
                    "DRY violation detected. The Rust gods demand a trait implementation.".to_string()
                }
                r if r.contains("long") => {
                    "This function is so long, it needs its own crate. Split with mod.".to_string()
                }
                _ => "This code is why C++ developers laugh at us.".to_string(),
            },
        }
    }

    fn closing(&self) -> &'static str {
        match self {
            Self::LinuxKernel => "Fix this or I'll NAK your patch.",
            Self::SiliconValley => {
                "Let's circle back and ideate on a solution. Maybe over kombucha."
            }
            Self::JapaneseEnterprise => {
                "Please address these findings at your earliest convenience. (By tomorrow.)"
            }
            Self::RustFanatic => "Rewrite it in Rust. Oh wait, it IS Rust. Then rewrite it BETTER.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_from_str() {
        assert!(Persona::parse_persona("linux-kernel").is_some());
        assert!(Persona::parse_persona("rust").is_some());
        assert!(Persona::parse_persona("unknown").is_none());
    }

    #[test]
    fn test_persona_name() {
        assert_eq!(
            Persona::LinuxKernel.name(),
            "Grumpy Linux Kernel Maintainer"
        );
    }

    #[test]
    fn test_run_on_current_dir() {
        let result = run(
            std::path::Path::new("."),
            Persona::LinuxKernel,
            &OutputFormat::Terminal,
            "en-US",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_persona_names() {
        assert_eq!(Persona::SiliconValley.name(), "Silicon Valley Bro CTO");
        assert_eq!(
            Persona::JapaneseEnterprise.name(),
            "Japanese Enterprise Engineer"
        );
        assert_eq!(Persona::RustFanatic.name(), "Rust Evangelist");
    }

    #[test]
    fn test_all_persona_intros() {
        assert!(!Persona::LinuxKernel.intro().is_empty());
        assert!(!Persona::SiliconValley.intro().is_empty());
        assert!(!Persona::JapaneseEnterprise.intro().is_empty());
        assert!(!Persona::RustFanatic.intro().is_empty());
    }

    #[test]
    fn test_persona_from_str_all_variants() {
        assert!(Persona::parse_persona("linux-kernel").is_some());
        assert!(Persona::parse_persona("linux").is_some());
        assert!(Persona::parse_persona("silicon-valley").is_some());
        assert!(Persona::parse_persona("sv").is_some());
        assert!(Persona::parse_persona("japanese-enterprise").is_some());
        assert!(Persona::parse_persona("jp").is_some());
        assert!(Persona::parse_persona("rust-fanatic").is_some());
        assert!(Persona::parse_persona("rust").is_some());
        assert!(Persona::parse_persona("unknown").is_none());
        assert!(Persona::parse_persona("").is_none());
    }
}
