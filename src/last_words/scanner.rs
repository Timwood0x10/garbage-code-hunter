//! Scan source files for "last words" — TODO, FIXME, HACK, TEMP comments.

use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A discovered "last word" comment in the codebase.
#[derive(Debug, Clone)]
pub struct LastWord {
    pub file: PathBuf,
    pub line: usize,
    pub kind: LastWordKind,
    pub text: String,
    pub age_days: Option<u64>,
}

/// The type of last-word comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LastWordKind {
    Todo,
    Fixme,
    Hack,
    Temp,
    QuickFix,
    Wontfix,
    Workaround,
    Deprecated,
    Safety,
}

impl LastWordKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Fixme => "FIXME",
            Self::Hack => "HACK",
            Self::Temp => "TEMP",
            Self::QuickFix => "quick fix",
            Self::Wontfix => "WONTFIX",
            Self::Workaround => "workaround",
            Self::Deprecated => "DEPRECATED",
            Self::Safety => "SAFETY",
        }
    }

    pub fn tombstone_quote(&self) -> &'static str {
        match self {
            Self::Todo => "I'll do it later",
            Self::Fixme => "This is fine... probably",
            Self::Hack => "Don't touch this",
            Self::Temp => "Temporary workaround",
            Self::QuickFix => "Quick fix for now",
            Self::Wontfix => "Won't fix, not my problem",
            Self::Workaround => "It works, don't ask how",
            Self::Deprecated => "Dead code walking",
            Self::Safety => "Unsafe but necessary",
        }
    }
}

/// Scan a directory for last-word comments.
pub fn scan(path: &Path) -> Vec<LastWord> {
    let patterns = build_patterns();
    let mut results = Vec::new();

    let entries: Vec<_> = if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| is_source_file(e.path()))
            .map(|e| e.path().to_path_buf())
            .collect()
    };

    for file_path in entries {
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip lines that are just code, not comments
            if !is_comment_line(trimmed) {
                continue;
            }

            for (kind, re) in &patterns {
                if re.is_match(trimmed) {
                    results.push(LastWord {
                        file: file_path.clone(),
                        line: line_num + 1,
                        kind: kind.clone(),
                        text: trimmed.to_string(),
                        age_days: None,
                    });
                }
            }
        }
    }

    results
}

/// Try to get the age of a TODO comment via git blame.
/// Returns age in days if successful.
pub fn try_get_age(file: &Path, line: usize) -> Option<u64> {
    let output = std::process::Command::new("git")
        .args([
            "blame",
            "-L",
            &format!("{},{}", line, line),
            "--porcelain",
            &file.to_string_lossy(),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for l in stdout.lines() {
        if let Some(rest) = l.strip_prefix("committer-time ") {
            let timestamp: u64 = rest.trim().parse().ok()?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            return now.checked_sub(timestamp).map(|d| d / 86400);
        }
    }

    None
}

fn build_patterns() -> Vec<(LastWordKind, Regex)> {
    vec![
        (LastWordKind::Fixme, Regex::new(r"(?i)\bFIXME\b").unwrap()),
        (LastWordKind::Todo, Regex::new(r"(?i)\bTODO\b").unwrap()),
        (LastWordKind::Hack, Regex::new(r"(?i)\bHACK\b").unwrap()),
        (
            LastWordKind::Temp,
            Regex::new(r"(?i)\bTEMP(ORARY)?\b").unwrap(),
        ),
        (
            LastWordKind::QuickFix,
            Regex::new(r"(?i)\bquick\s*fix\b").unwrap(),
        ),
        (
            LastWordKind::Wontfix,
            Regex::new(r"(?i)\bWONT\s*FIX\b").unwrap(),
        ),
        (
            LastWordKind::Workaround,
            Regex::new(r"(?i)\bworkaround\b").unwrap(),
        ),
        (
            LastWordKind::Deprecated,
            Regex::new(r"(?i)\bDEPRECATED?\b").unwrap(),
        ),
        (LastWordKind::Safety, Regex::new(r"(?i)\bSAFETY\b").unwrap()),
    ]
}

fn is_comment_line(line: &str) -> bool {
    line.starts_with("//")
        || line.starts_with("/*")
        || line.starts_with("*")
        || line.starts_with("#")
        || line.starts_with("<!--")
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(
            "rs" | "py"
                | "js"
                | "ts"
                | "go"
                | "java"
                | "c"
                | "cpp"
                | "h"
                | "hpp"
                | "rb"
                | "php"
                | "swift"
                | "kt"
                | "scala"
                | "sh"
                | "bash"
                | "zsh"
                | "toml"
                | "yaml"
                | "yml"
                | "json"
                | "md"
                | "html"
                | "css"
                | "sql"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_finds_todos() {
        let dir = std::env::temp_dir().join("gch_last_words_test");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test.rs");
        std::fs::write(&file, "// TODO: fix this\nfn main() {}\n// FIXME: broken\n").unwrap();

        let results = scan(&dir);
        assert!(results.iter().any(|r| r.kind == LastWordKind::Todo));
        assert!(results.iter().any(|r| r.kind == LastWordKind::Fixme));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_kind_label() {
        assert_eq!(LastWordKind::Todo.label(), "TODO");
        assert_eq!(LastWordKind::Hack.label(), "HACK");
    }

    #[test]
    fn test_is_comment_line() {
        assert!(is_comment_line("// TODO: fix"));
        assert!(is_comment_line("/* FIXME */"));
        assert!(is_comment_line("# HACK"));
        assert!(!is_comment_line("let x = 1;"));
    }
}
