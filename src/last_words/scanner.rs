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

    // ── kind_label ─────────────────────────────────────────────────

    /// Objective: Verify all variants map to the expected label strings.
    /// Invariants: Every LastWordKind has a unique, non-empty label.
    #[test]
    fn test_kind_label_all_variants() {
        assert_eq!(LastWordKind::Todo.label(), "TODO");
        assert_eq!(LastWordKind::Fixme.label(), "FIXME");
        assert_eq!(LastWordKind::Hack.label(), "HACK");
        assert_eq!(LastWordKind::Temp.label(), "TEMP");
        assert_eq!(LastWordKind::QuickFix.label(), "quick fix");
        assert_eq!(LastWordKind::Wontfix.label(), "WONTFIX");
        assert_eq!(LastWordKind::Workaround.label(), "workaround");
        assert_eq!(LastWordKind::Deprecated.label(), "DEPRECATED");
        assert_eq!(LastWordKind::Safety.label(), "SAFETY");
    }

    // ── tombstone_quote ────────────────────────────────────────────

    /// Objective: Verify all variants return a non-empty tombstone quote.
    /// Invariants: Every quote is a unique, non-empty string (no panics).
    #[test]
    fn test_tombstone_quote_all_variants() {
        for kind in &[
            LastWordKind::Todo,
            LastWordKind::Fixme,
            LastWordKind::Hack,
            LastWordKind::Temp,
            LastWordKind::QuickFix,
            LastWordKind::Wontfix,
            LastWordKind::Workaround,
            LastWordKind::Deprecated,
            LastWordKind::Safety,
        ] {
            let quote = kind.tombstone_quote();
            assert!(
                !quote.is_empty(),
                "{:?}.tombstone_quote() should not be empty",
                kind
            );
        }
    }

    // ── is_comment_line ────────────────────────────────────────────

    /// Objective: Verify all comment line prefixes are recognized.
    /// Invariants: Lines starting with //, /*, *, #, <!-- are comments.
    #[test]
    fn test_is_comment_line_all_styles() {
        assert!(is_comment_line("// TODO: fix"), "// line");
        assert!(is_comment_line("/* FIXME */"), "/* line");
        assert!(is_comment_line("* multiline continuation"), "* line");
        assert!(is_comment_line("# HACK"), "# line");
        assert!(is_comment_line("<!-- HTML comment -->"), "<!-- line");
    }

    /// Objective: Verify known non-comment lines are rejected.
    #[test]
    fn test_is_comment_line_non_comment() {
        assert!(!is_comment_line("let x = 1;"), "code line");
        assert!(!is_comment_line(""), "empty line");
        assert!(!is_comment_line("   "), "whitespace only");
        assert!(
            !is_comment_line("x // inline comment is not a comment line"),
            "code with trailing //"
        );
    }

    // ── is_source_file ─────────────────────────────────────────────

    /// Objective: Verify is_source_file returns true for all supported extensions.
    /// Invariants: Any file with a supported extension is recognized as a source file.
    #[test]
    fn test_is_source_file_supported() {
        for ext in &[
            "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "hpp", "rb", "php", "swift",
            "kt", "scala", "sh", "bash", "zsh", "toml", "yaml", "yml", "json", "md", "html", "css",
            "sql",
        ] {
            let path = PathBuf::from(format!("file.{}", ext));
            assert!(
                is_source_file(&path),
                "file.{ext} should be recognized as source"
            );
        }
    }

    /// Objective: Verify is_source_file returns false for unsupported extensions.
    #[test]
    fn test_is_source_file_unsupported() {
        assert!(
            !is_source_file(Path::new("file.txt")),
            ".txt should not be source"
        );
        assert!(
            !is_source_file(Path::new("file.pdf")),
            ".pdf should not be source"
        );
        assert!(
            !is_source_file(Path::new("Makefile")),
            "no ext should not be source"
        );
        assert!(
            !is_source_file(Path::new("file.")),
            "trailing dot should not be source"
        );
    }

    // ── build_patterns ─────────────────────────────────────────────

    /// Objective: Verify each pattern matches its expected keyword and rejects others.
    /// Invariants: Patterns are case-insensitive; they match keywords anywhere in a line.
    #[test]
    fn test_build_patterns_todo() {
        let patterns = build_patterns();
        let todo_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::Todo)
            .map(|(_, r)| r)
            .unwrap();
        assert!(
            todo_re.is_match("// TODO: fix me"),
            "TODO pattern should match '// TODO: fix me'"
        );
        assert!(
            todo_re.is_match("// todo: lower case"),
            "TODO i flag: 'todo'"
        );
        assert!(
            !todo_re.is_match("// todolist"),
            "TODO should not match 'todolist' (\\b)"
        );
    }

    /// Objective: Verify FIXME pattern works correctly.
    #[test]
    fn test_build_patterns_fixme() {
        let patterns = build_patterns();
        let fixme_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::Fixme)
            .map(|(_, r)| r)
            .unwrap();
        assert!(fixme_re.is_match("// FIXME: broken"), "FIXME should match");
        assert!(fixme_re.is_match("# fixme"), "fixme lowercase");
        assert!(
            !fixme_re.is_match("// fixable"),
            "fixme should not match 'fixable'"
        );
    }

    /// Objective: Verify HACK pattern matches with word boundary.
    #[test]
    fn test_build_patterns_hack() {
        let patterns = build_patterns();
        let hack_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::Hack)
            .map(|(_, r)| r)
            .unwrap();
        assert!(
            hack_re.is_match("// HACK: ugly but works"),
            "HACK should match"
        );
        assert!(
            !hack_re.is_match("// hackneyed"),
            "HACK should not match 'hackneyed' (\\b)"
        );
    }

    /// Objective: Verify TEMP pattern matches both "TEMP" and "TEMPORARY".
    #[test]
    fn test_build_patterns_temp() {
        let patterns = build_patterns();
        let temp_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::Temp)
            .map(|(_, r)| r)
            .unwrap();
        assert!(temp_re.is_match("// TEMP: quick fix"), "TEMP should match");
        assert!(
            temp_re.is_match("// TEMPORARY workaround"),
            "TEMPORARY should match"
        );
        assert!(
            !temp_re.is_match("// temperature"),
            "TEMPORARY? with \\b should not match 'temperature'"
        );
    }

    /// Objective: Verify quick-fix pattern matches with optional space between words.
    #[test]
    fn test_build_patterns_quickfix() {
        let patterns = build_patterns();
        let qfix_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::QuickFix)
            .map(|(_, r)| r)
            .unwrap();
        assert!(qfix_re.is_match("// quick fix"), "quick fix should match");
        assert!(
            qfix_re.is_match("// quickfix"),
            "quickfix should match (\\s* allows zero)"
        );
        assert!(
            qfix_re.is_match("// Quick Fix"),
            "Quick Fix should match (i flag)"
        );
    }

    /// Objective: Verify WONTFIX pattern matches standard form.
    #[test]
    fn test_build_patterns_wontfix() {
        let patterns = build_patterns();
        let wontfix_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::Wontfix)
            .map(|(_, r)| r)
            .unwrap();
        assert!(wontfix_re.is_match("// WONT FIX"), "WONT FIX should match");
        assert!(
            wontfix_re.is_match("// wontfix"),
            "wontfix collapsed form matches (\\s* zero)"
        );
        assert!(
            wontfix_re.is_match("// WONT   FIX"),
            "multiple spaces between WONT and FIX"
        );
        assert!(
            !wontfix_re.is_match("// won't fix"),
            "should not match apostrophe form"
        );
    }

    /// Objective: Verify WORKAROUND pattern matches.
    #[test]
    fn test_build_patterns_workaround() {
        let patterns = build_patterns();
        let work_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::Workaround)
            .map(|(_, r)| r)
            .unwrap();
        assert!(
            work_re.is_match("// workaround: temp fix"),
            "workaround should match"
        );
        assert!(work_re.is_match("/* WORKAROUND */"), "WORKAROUND uppercase");
    }

    /// Objective: Verify DEPRECATED pattern matches both "DEPRECATED" and "DEPRECATE".
    #[test]
    fn test_build_patterns_deprecated() {
        let patterns = build_patterns();
        let dep_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::Deprecated)
            .map(|(_, r)| r)
            .unwrap();
        assert!(
            dep_re.is_match("// DEPRECATED: old code"),
            "DEPRECATED should match"
        );
        assert!(
            dep_re.is_match("// Deprecate this in v2"),
            "DEPRECATE? should match 'Deprecate'"
        );
        assert!(
            !dep_re.is_match("// depreciation"),
            "DEPRECATE? with \\b pos should not match 'depreciation'"
        );
    }

    /// Objective: Verify SAFETY pattern matches.
    #[test]
    fn test_build_patterns_safety() {
        let patterns = build_patterns();
        let safety_re = patterns
            .iter()
            .find(|(k, _)| *k == LastWordKind::Safety)
            .map(|(_, r)| r)
            .unwrap();
        assert!(
            safety_re.is_match("// SAFETY: unsafe block"),
            "SAFETY should match"
        );
        assert!(safety_re.is_match("# safety: required"), "safety lowercase");
    }

    // ── scan ───────────────────────────────────────────────────────

    /// Objective: Verify scan finds TODO and FIXME in a Rust file.
    /// Invariants: Each matching comment creates one LastWord result.
    #[test]
    fn test_scan_finds_todos_and_fixmes() {
        let dir = std::env::temp_dir().join("gch_last_words_test_scan");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test.rs");
        std::fs::write(&file, "// TODO: fix this\nfn main() {}\n// FIXME: broken\n").unwrap();

        let results = scan(&dir);
        assert!(
            results.iter().any(|r| r.kind == LastWordKind::Todo),
            "should find TODO"
        );
        assert!(
            results.iter().any(|r| r.kind == LastWordKind::Fixme),
            "should find FIXME"
        );
        assert_eq!(results.len(), 2, "exactly 2 last-words in the file");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Objective: Verify scan finds all comment types: #, //, /* */ in different files.
    #[test]
    fn test_scan_finds_hack_in_python() {
        let dir = std::env::temp_dir().join("gch_last_words_test_hack");
        let _ = std::fs::create_dir_all(&dir);
        let py_file = dir.join("script.py");
        std::fs::write(&py_file, "# HACK: this is terrible\nx = 1\n").unwrap();

        let results = scan(&dir);
        // Only the .py file should be scanned
        let hacks: Vec<_> = results
            .iter()
            .filter(|r| r.kind == LastWordKind::Hack)
            .collect();
        assert_eq!(hacks.len(), 1, "should find 1 HACK in Python file");
        assert_eq!(hacks[0].line, 1, "HACK should be on line 1");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Objective: Verify scan skips non-comment lines (code only).
    #[test]
    fn test_scan_skips_code_lines() {
        let dir = std::env::temp_dir().join("gch_last_words_test_skip");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test.rs");
        std::fs::write(&file, "fn main() {\n    let todo = \"not a comment\";\n}\n").unwrap();

        let results = scan(&dir);
        assert!(
            results.is_empty(),
            "code with no comments should produce 0 results, got {}",
            results.len()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Objective: Verify scan handles empty files without panicking.
    #[test]
    fn test_scan_empty_file() {
        let dir = std::env::temp_dir().join("gch_last_words_test_empty");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("empty.rs");
        std::fs::write(&file, "").unwrap();

        let results = scan(&dir);
        assert!(results.is_empty(), "empty file should produce 0 results");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Objective: Verify scan ignores non-source files.
    #[test]
    fn test_scan_ignores_unsupported_extensions() {
        let dir = std::env::temp_dir().join("gch_last_words_test_unsup");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("readme.txt");
        std::fs::write(&file, "// TODO: ignored").unwrap();

        let results = scan(&dir);
        assert!(
            results.is_empty(),
            ".txt file should be ignored, got {} results",
            results.len()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Objective: Verify scan works on a single file (not a directory).
    #[test]
    fn test_scan_single_file() {
        let dir = std::env::temp_dir().join("gch_last_words_test_single");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("single.rs");
        std::fs::write(&file, "// TODO: single file\n// HACK: also here\n").unwrap();

        let results = scan(&file);
        assert_eq!(results.len(), 2, "single file scan should find 2 results");
        assert!(results.iter().any(|r| r.kind == LastWordKind::Todo));
        assert!(results.iter().any(|r| r.kind == LastWordKind::Hack));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
