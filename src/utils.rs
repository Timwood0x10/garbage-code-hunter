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
