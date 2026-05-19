//! Shared functions for C and C++ adapters.

use super::{
    count_block_ancestors, count_params, is_boolean_or_null, is_common_safe_number,
    is_inside_declaration, is_repeating_chars, FunctionNode, MEANINGLESS_NAMES,
};
use crate::treesitter::engine::ParsedFile;
use crate::treesitter::query::QueryCapture;
use regex::Regex;
use std::sync::LazyLock;

pub fn c_scope_depth(node: tree_sitter::Node, depth: usize) -> usize {
    let mut max = depth;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            let child_depth = match child.kind() {
                "compound_statement" => depth + 1,
                _ => depth,
            };
            max = max.max(c_scope_depth(child, child_depth));
        }
    }
    max
}

pub fn walk_c_blocks(node: tree_sitter::Node, depth: usize, threshold: usize, count: &mut usize) {
    if node.kind() == "compound_statement" && depth >= threshold {
        *count += 1;
    }
    let child_depth = match node.kind() {
        "compound_statement" => depth + 1,
        _ => depth,
    };
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            walk_c_blocks(child, child_depth, threshold, count);
        }
    }
}

pub fn extract_functions_from_batch<'a>(batch: &[Vec<QueryCapture<'a>>]) -> Vec<FunctionNode> {
    let mut functions = Vec::new();
    for m in batch {
        let has_ex = m.iter().any(|c| c.name.starts_with("ex_"));
        if !has_ex {
            continue;
        }
        let mut name = String::new();
        let mut start_line = 0usize;
        let mut end_line = 0usize;
        for c in m {
            match c.name.as_str() {
                "ex_name" => name = c.text.to_string(),
                "ex_fn" => {
                    start_line = c.node.start_position().row + 1;
                    end_line = c.node.end_position().row + 1;
                }
                _ => {}
            }
        }
        if !name.is_empty() {
            let nesting_depth = count_block_ancestors(m);
            functions.push(FunctionNode {
                name,
                start_line,
                end_line,
                nesting_depth,
            });
        }
    }
    functions
}

pub fn count_naming_from_batch<'a>(batch: &[Vec<QueryCapture<'a>>]) -> usize {
    let mut count = 0usize;
    static TERRIBLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
        Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").ok()
    });
    let terrible_re = TERRIBLE_RE.as_ref();
    let idiomatic_single: &[&str] = &["i", "j", "k", "n", "c", "e"];

    for m in batch {
        for c in m {
            if c.name == "nv_var" {
                let name = c.text;
                if name.len() == 1 && name.chars().all(|ch| ch.is_ascii_lowercase()) {
                    if !idiomatic_single.contains(&name) {
                        count += 1;
                    }
                    continue;
                }
                if let Some(re) = terrible_re {
                    if re.is_match(&name.to_lowercase()) {
                        count += 1;
                        continue;
                    }
                }
                if MEANINGLESS_NAMES.contains(&name) || is_repeating_chars(name) {
                    count += 1;
                }
            }
        }
    }
    count
}

pub fn count_debug_from_batch<'a>(batch: &[Vec<QueryCapture<'a>>]) -> usize {
    batch
        .iter()
        .filter(|m| m.iter().any(|c| c.name == "dp_func"))
        .count()
}

pub fn count_excessive_from_batch<'a>(batch: &[Vec<QueryCapture<'a>>]) -> usize {
    let mut count = 0;
    for m in batch {
        for c in m {
            if c.name == "ep_params" && count_params(c.text) > 5 {
                count += 1;
            }
        }
    }
    count
}

pub fn count_magic_from_batch<'a>(batch: &[Vec<QueryCapture<'a>>]) -> usize {
    let mut count = 0;
    for m in batch {
        for c in m {
            if c.name == "mn_num" && !is_inside_declaration(c.node) {
                let text = c.text;
                if text != "0"
                    && text != "1"
                    && text != "-1"
                    && !is_common_safe_number(text)
                    && !is_boolean_or_null(text)
                {
                    count += 1;
                }
            }
        }
    }
    count
}

pub fn count_c_issues_from_batch<'a>(
    file: &ParsedFile,
    batch: &[Vec<QueryCapture<'a>>],
    extra_captures: &[&str],
) -> usize {
    let mut count = 0;

    for m in batch {
        for c in m {
            if c.name == "ci_goto" || extra_captures.contains(&c.name.as_str()) {
                count += 1;
            }
        }
    }

    for line in file.content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        if trimmed.contains("sizeof(") {
            let start = trimmed.find("sizeof(").unwrap() + 7;
            let rest = &trimmed[start..];
            let mut depth = 1u32;
            let mut inner = String::new();
            for ch in rest.chars() {
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                }
                if depth == 0 {
                    break;
                }
                inner.push(ch);
            }
            let inner = inner.trim().trim_end_matches(')');
            if inner.starts_with(|c: char| c.is_alphabetic() || c == '_')
                && !inner.contains(['+', '-', '*', '/', '('])
            {
                let type_keywords = [
                    "int", "char", "float", "double", "long", "short", "unsigned", "signed",
                    "void", "size_t", "bool", "struct", "union", "enum",
                ];
                if type_keywords.iter().any(|t| inner.starts_with(t)) {
                    count += 1;
                }
                if inner.ends_with("_t") && inner.len() > 2 {
                    count += 1;
                }
            }
        }
    }

    let lines: Vec<&str> = file.content.lines().collect();
    for i in 0..lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        if trimmed.contains("malloc(") && trimmed.ends_with(';') {
            let check_range = (i + 1).min(lines.len())..(i + 4).min(lines.len());
            let has_null_check = lines[check_range].iter().any(|l| {
                let l = l.trim();
                l.contains("== NULL")
                    || l.contains("!= NULL")
                    || l.contains("== 0")
                    || l.contains("!= 0")
                    || l.contains("if (!")
                    || l.contains("if (NULL")
            });
            if !has_null_check {
                count += 1;
            }
        }
    }

    count
}
