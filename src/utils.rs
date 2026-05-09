use syn::spanned::Spanned;

/// Get line number from syn AST node using proc_macro2 span-locations
pub fn get_line_number<T: Spanned>(node: &T) -> usize {
    node.span().start().line
}

/// Get column number from syn AST node using proc_macro2 span-locations
pub fn get_column_number<T: Spanned>(node: &T) -> usize {
    node.span().start().column + 1 // Convert 0-based to 1-based
}

/// Get position info (line, column) from syn AST node
pub fn get_position<T: Spanned>(node: &T) -> (usize, usize) {
    (get_line_number(node), get_column_number(node))
}

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
