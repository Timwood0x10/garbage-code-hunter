//! Utility functions for text-based analysis.

/// Check if a line is a comment
fn is_comment_line(trimmed: &str) -> bool {
    trimmed.starts_with("///")
        || trimmed.starts_with("//!")
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with("*")
}

/// Find the line number of a string literal in source content (skipping comments)
pub fn find_line_of_str(content: &str, target: &str) -> usize {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if is_comment_line(trimmed) {
            continue;
        }
        if line.contains(target) {
            return i + 1;
        }
    }
    1
}

/// Find the line number of a string literal, skipping comments and import statements
pub fn find_line_of_str_non_import(content: &str, target: &str) -> usize {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if is_comment_line(trimmed) || trimmed.starts_with("use ") {
            continue;
        }
        if line.contains(target) {
            return i + 1;
        }
    }
    1
}

/// Count non-comment occurrences of a pattern in source content
pub fn count_non_comment_matches(content: &str, target: &str) -> usize {
    let mut count = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if is_comment_line(trimmed) {
            continue;
        }
        count += line.matches(target).count();
    }
    count
}

/// Get (line, column) from a byte offset in source content.
pub fn get_position_from_content(content: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in content.char_indices() {
        if i >= byte_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Truncate a string to a maximum length, appending "..." if truncated.
///
/// Uses char-aware slicing to avoid panicking on multi-byte UTF-8 boundaries.
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut end = max.saturating_sub(3);
        // Find a valid char boundary
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

/// Count non-comment, non-import occurrences of a pattern in source content
pub fn count_non_import_matches(content: &str, target: &str) -> usize {
    let mut count = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if is_comment_line(trimmed) || trimmed.starts_with("use ") {
            continue;
        }
        count += line.matches(target).count();
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── find_line_of_str ──────────────────────────────────────────

    /// Objective: Verify find_line_of_str returns the correct line (1-indexed)
    ///            when the target appears in code, skipping comment lines.
    /// Invariants: Returned line must be > 0. Comment-only lines are skipped.
    #[test]
    fn test_find_line_of_str_finds_target() {
        let content = "fn main() {\n    let x = 1;\n    println!(\"{}\", x);\n}";
        let line = find_line_of_str(content, "println");
        assert_eq!(line, 3, "println appears on line 3, got {line}");
    }

    /// Objective: Verify that when the target string does not exist
    ///            in the content, the function returns the fallback value 1.
    /// Invariants: Never panics on missing target. Always returns a valid line number.
    #[test]
    fn test_find_line_of_str_returns_fallback_when_missing() {
        assert_eq!(
            find_line_of_str("fn main() {}", "nonexistent"),
            1,
            "should return fallback line 1 when target is absent"
        );
    }

    /// Objective: Verify that commented-out target strings are not matched.
    /// Invariants: Lines starting with // are skipped regardless of content.
    #[test]
    fn test_find_line_of_str_skips_comments() {
        let content = "// println!(\"hidden\")\nfn main() {\n    println!(\"real\");\n}";
        let line = find_line_of_str(content, "println");
        assert_eq!(
            line, 3,
            "should skip the comment on line 1 and find println on line 3"
        );
    }

    /// Objective: Verify behavior with empty content — no crash, returns fallback.
    #[test]
    fn test_find_line_of_str_empty_content_does_not_crash() {
        assert_eq!(
            find_line_of_str("", "anything"),
            1,
            "empty content should return fallback line 1"
        );
    }

    // ── find_line_of_str_non_import ────────────────────────────────

    /// Objective: Verify that `use` lines are skipped when searching for a target.
    /// Invariants: Lines starting with "use " are ignored.
    #[test]
    fn test_find_line_of_str_non_import_skips_use_lines() {
        let content = "use std::io;\nfn main() {\n    io::stdout();\n}";
        let line = find_line_of_str_non_import(content, "io");
        assert_eq!(line, 3, "should skip 'use std::io;' and find io on line 3");
    }

    /// Objective: Verify both comments AND imports are skipped simultaneously.
    #[test]
    fn test_find_line_of_str_non_import_skips_comments_and_imports() {
        let content = "// comment\nuse std::fmt;\nfn run() {\n    fmt::format(\"hi\");\n}";
        let line = find_line_of_str_non_import(content, "format");
        assert_eq!(line, 4, "should skip comment line 1 and use line 2");
    }

    /// Objective: Verify that target appearing inside a `use` line is NOT matched.
    #[test]
    fn test_find_line_of_str_non_import_target_only_in_use_not_found() {
        let content = "use std::fmt;\nfn run() {}";
        let line = find_line_of_str_non_import(content, "fmt");
        assert_eq!(
            line, 1,
            "'fmt' only appears in use line, should fallback to 1"
        );
    }

    // ── count_non_comment_matches ──────────────────────────────────

    /// Objective: Verify that comments are excluded from match counting.
    /// Invariants: `//` lines contribute 0 matches regardless of content.
    #[test]
    fn test_count_non_comment_matches_excludes_comments() {
        let content = "let x = 1;\n// let x = 2;\nlet y = 1;";
        assert_eq!(
            count_non_comment_matches(content, "let"),
            2,
            "should count 'let' in lines 1 and 3 only, skipping the comment on line 2"
        );
    }

    /// Objective: Verify empty result when target does not exist in non-comment code.
    #[test]
    fn test_count_non_comment_matches_returns_zero_for_absent_target() {
        assert_eq!(
            count_non_comment_matches("fn main() {}", "println"),
            0,
            "no println in code, should be 0"
        );
    }

    /// Objective: Verify that doc comments (///) are also excluded.
    /// Invariants: `///` lines are treated as comments and skipped.
    #[test]
    fn test_count_non_comment_matches_excludes_doc_comments() {
        let content = "/// some code: let a = 1\nfn foo() { let b = 2; }";
        assert_eq!(
            count_non_comment_matches(content, "let"),
            1,
            "doc comment 'let' should not be counted; only the code 'let' on line 2"
        );
    }

    /// Objective: Verify that block comment start (/*) lines are excluded.
    #[test]
    fn test_count_non_comment_matches_excludes_block_comment_start() {
        let content = "/* let hidden = 1; */\nfn foo() { let visible = 2; }";
        assert_eq!(
            count_non_comment_matches(content, "let"),
            1,
            "block comment line should be excluded"
        );
    }

    // ── get_position_from_content ──────────────────────────────────

    /// Objective: Verify byte offset 0 maps to (line 1, col 1).
    #[test]
    fn test_get_position_at_start() {
        let (line, col) = get_position_from_content("hello", 0);
        assert_eq!((line, col), (1, 1), "byte 0 should be line 1, col 1");
    }

    /// Objective: Verify position advances correctly within a single line.
    #[test]
    fn test_get_position_mid_line() {
        let (line, col) = get_position_from_content("abcde", 3);
        assert_eq!(
            (line, col),
            (1, 4),
            "byte 3 (0-indexed) is the 4th character"
        );
    }

    /// Objective: Verify newline advances the line counter and resets column.
    #[test]
    fn test_get_position_at_newline_boundary() {
        let content = "first\nsecond\nthird";
        let pos = content.find("second").expect("second should exist");
        let (line, col) = get_position_from_content(content, pos);
        assert_eq!((line, col), (2, 1), "'second' starts at line 2, col 1");
    }

    /// Objective: Verify that an offset beyond content length does not panic
    ///            and returns the position at the end of the last character.
    #[test]
    fn test_get_position_beyond_end_does_not_crash() {
        let (line, col) = get_position_from_content("hi", 999);
        assert_eq!(
            (line, col),
            (1, 3),
            "beyond-end offset should land at end of content"
        );
    }

    /// Objective: Verify behavior with empty content — no crash.
    #[test]
    fn test_get_position_empty_content_does_not_crash() {
        let (line, col) = get_position_from_content("", 0);
        assert_eq!(
            (line, col),
            (1, 1),
            "empty content at offset 0 is line 1, col 1"
        );
    }

    // ── truncate ───────────────────────────────────────────────────

    /// Objective: Verify that strings shorter than max are returned unchanged.
    #[test]
    fn test_truncate_short_string_unchanged() {
        let result = truncate("hello", 10);
        assert_eq!(
            result, "hello",
            "string shorter than max should not be truncated"
        );
    }

    /// Objective: Verify that strings exactly at max length are NOT truncated.
    /// Invariants: Only strings longer than max get truncated.
    #[test]
    fn test_truncate_exact_length_kept() {
        let result = truncate("hello", 5);
        assert_eq!(
            result, "hello",
            "string equal to max should not be truncated"
        );
    }

    /// Objective: Verify truncation appends "..." and shortens the string appropriately.
    #[test]
    fn test_truncate_appends_ellipsis() {
        let result = truncate("hello world", 8);
        assert_eq!(result, "hello...", "should keep 5 chars + '...' = 8 total");
    }

    /// Objective: Verify that multi-byte UTF-8 characters do not cause a panic
    ///            at character boundary split points.
    /// Invariants: The function must not slice in the middle of a multi-byte char.
    #[test]
    fn test_truncate_multi_byte_char_boundary_no_panic() {
        let s = "héllo wörld";
        // max=5 -> end=2, which lands in the middle of 2-byte 'é', forcing is_char_boundary fallback
        let result = truncate(s, 5);
        assert_eq!(
            result, "h...",
            "max=5 on 'héllo wörld' should fall back past 2-byte é and produce 'h...', got '{result}'"
        );
    }

    /// Objective: Verify that truncate with max=0 returns "..." only.
    /// Invariants: When max < 3, the function still produces valid output.
    #[test]
    fn test_truncate_max_zero_returns_ellipsis_only() {
        let result = truncate("hello", 0);
        assert_eq!(result, "...", "max=0 should produce '...'");
    }

    /// Objective: Verify that max < 3 still produces valid (non-panicking) output.
    #[test]
    fn test_truncate_max_one_produces_ellipsis() {
        let result = truncate("hello", 1);
        assert_eq!(result, "...", "max=1 should produce '...'");
    }

    // ── count_non_import_matches ───────────────────────────────────

    /// Objective: Verify that both `use` lines and comment lines are excluded.
    /// Invariants: Lines starting with "use " or comment markers are skipped.
    #[test]
    fn test_count_non_import_matches_excludes_use_and_comments() {
        let content = "use std::fmt;\n// use std::io;\nfn main() { fmt::println!(); }";
        assert_eq!(
            count_non_import_matches(content, "fmt"),
            1,
            "only line 3 should match, lines 1 (use) and 2 (comment) are excluded"
        );
    }

    /// Objective: Verify that only code lines are counted.
    #[test]
    fn test_count_non_import_matches_code_only() {
        let content = "let a = 1;\nlet b = 2;\nfn add(x: i32, y: i32) -> i32 { x + y }";
        assert_eq!(
            count_non_import_matches(content, "let"),
            2,
            "both 'let' in code lines count, got wrong count"
        );
    }

    /// Objective: Verify zero matches when target is absent from all non-import/non-comment lines.
    #[test]
    fn test_count_non_import_matches_zero_for_absent_target() {
        let content = "fn main() { loop {} }";
        assert_eq!(
            count_non_import_matches(content, "println"),
            0,
            "no println in code => 0"
        );
    }
}
