//! Shared helper functions used across multiple language adapters.

pub(crate) fn get_node_text<'a>(node: tree_sitter::Node, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

pub(crate) fn count_nested_blocks(
    node: tree_sitter::Node,
    depth: usize,
    threshold: usize,
    count: &mut usize,
) {
    if node.kind() == "block" && depth >= threshold {
        *count += 1;
    }
    let child_depth = match node.kind() {
        "block" => depth + 1,
        _ => depth,
    };
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            count_nested_blocks(child, child_depth, threshold, count);
        }
    }
}

pub(crate) fn max_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
    let mut max = depth;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            let child_depth = if is_scope_node(&child) {
                depth + 1
            } else {
                depth
            };
            max = max.max(max_scope_depth(child, child_depth));
        }
    }
    max
}

fn is_scope_node(node: &tree_sitter::Node) -> bool {
    matches!(node.kind(), "block")
}

pub(crate) fn is_repeating_chars(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    chars.len() >= 3 && chars.iter().all(|c| *c == chars[0])
}

pub(crate) fn count_block_ancestors(group: &[crate::treesitter::query::QueryCapture]) -> usize {
    if let Some(cap) = group.first() {
        let mut depth = 0usize;
        let mut current = Some(cap.node);
        while let Some(node) = current {
            if let Some(parent) = node.parent() {
                if parent.kind() == "block" {
                    depth += 1;
                }
                current = Some(parent);
            } else {
                break;
            }
        }
        depth
    } else {
        0
    }
}

/// Common meaningless/placeholder names to flag as naming violations.
pub(crate) const MEANINGLESS_NAMES: &[&str] = &[
    "foo", "bar", "baz", "qux", "quux", "quuz", "aaa", "bbb", "ccc", "ddd", "eee", "xxx", "yyy",
    "zzz", "test1", "test2", "test3",
];

/// Numbers that are widely understood in context and should not be flagged as magic.
///
/// Includes HTTP status codes, standard network ports, powers of two,
/// and common time constants — all self-explanatory in most contexts.
pub(crate) fn is_common_safe_number(text: &str) -> bool {
    matches!(
        text,
        // HTTP status codes
        "200" | "201" | "204" | "301" | "302" | "304" | "307" | "400" | "401" | "403"
        | "404" | "405" | "408" | "409" | "410" | "412" | "415" | "422" | "429"
        | "500" | "502" | "503" | "504"
        // Standard network ports
        | "22" | "80" | "443" | "3000" | "8080" | "8443" | "5432" | "6379" | "27017"
        // Powers of 2 (computing fundamentals)
        | "2" | "4" | "8" | "16" | "32" | "64" | "128" | "256" | "512" | "1024" | "2048"
        | "4096" | "8192"
        // Common time constants (seconds in a minute/hour/day)
        | "60" | "3600" | "86400"
    )
}

pub(crate) fn is_inside_declaration(node: tree_sitter::Node) -> bool {
    let mut current = Some(node);
    while let Some(n) = current {
        match n.kind() {
            "const_item"
            | "static_item"
            | "let_declaration"
            | "assignment"
            | "lexical_declaration"
            | "variable_declaration" => return true,
            "function_item"
            | "method_item"
            | "function_definition"
            | "macro_invocation"
            | "attribute_item"
            | "function_declaration"
            | "method_definition" => return false,
            _ => {}
        }
        current = n.parent();
    }
    false
}

pub(crate) fn count_params(text: &str) -> usize {
    let inner = text.trim();
    if inner.len() < 2 {
        return 0;
    }
    let inner = &inner[1..inner.len() - 1];
    if inner.trim().is_empty() {
        0
    } else {
        inner.bytes().filter(|&b| b == b',').count() + 1
    }
}

/// Parse source code for a given language, returning the ParsedFile.
#[cfg(test)]
pub(crate) fn parse_code(
    code: &str,
    filename: &str,
) -> Option<crate::treesitter::engine::ParsedFile> {
    let engine = crate::treesitter::TreeSitterEngine::new();
    engine.parse_file(std::path::Path::new(filename), code)
}
