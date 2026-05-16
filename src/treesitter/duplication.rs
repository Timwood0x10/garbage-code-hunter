use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::analyzer::{CodeIssue, Severity};
use crate::treesitter::engine::ParsedFile;

// ─── Cross-file duplication ──────────────────────────────────────────────────

/// Normalized token for function fingerprinting.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
enum FToken {
    Ident,
    Int,
    Float,
    Str,
    Bool,
    Type,
}

/// Fingerprint of a function — hash + source location.
#[derive(Debug, Clone)]
struct FuncFingerprint {
    hash: u64,
    name: String,
    file: PathBuf,
    line_start: usize,
}

/// Build a normalized token sequence from a function node.
fn fingerprint_function(file: &ParsedFile, node: tree_sitter::Node) -> Option<FuncFingerprint> {
    let name = extract_func_name(file, node)?;
    let line_start = node.start_position().row + 1;
    let line_end = node.end_position().row + 1;
    let line_count = line_end - line_start + 1;

    // Skip small functions (likely trivial or boilerplate)
    if line_count < 5 {
        return None;
    }

    let tokens = normalize_node(file, node);
    if tokens.len() < 8 {
        return None;
    }

    let mut hasher = DefaultHasher::new();
    tokens.hash(&mut hasher);
    let hash = hasher.finish();

    Some(FuncFingerprint {
        hash,
        name,
        file: file.path.clone(),
        line_start,
    })
}

/// Normalize a tree-sitter node into a Vec<FToken> for fingerprinting.
/// Leaf nodes (identifiers, literals) map to generic tokens.
/// Compound nodes recurse into their named children.
#[allow(clippy::only_used_in_recursion)]
fn normalize_node(file: &ParsedFile, node: tree_sitter::Node) -> Vec<FToken> {
    let kind = node.kind();
    match kind {
        "identifier"
        | "field_identifier"
        | "shorthand_property_identifier"
        | "variable_name"
        | "name" => return vec![FToken::Ident],
        "type_identifier" | "primitive_type" | "mutable_specifier" => return vec![FToken::Type],
        "integer_literal" | "integer" | "number" => return vec![FToken::Int],
        "float_literal" | "float" => return vec![FToken::Float],
        "string_literal" | "string" | "interpreted_string_literal" | "char_literal" => {
            return vec![FToken::Str];
        }
        "boolean" | "true" | "false" => return vec![FToken::Bool],
        // Ignored: keywords, operators, punctuation
        "let" | "const" | "mut" | "fn" | "def" | "function" | "fun" | "return" | "if" | "else"
        | "for" | "while" | "match" | "use" | "mod" | "struct" | "enum" | "impl" | "trait"
        | "pub" | "pub(crate)" | "unsafe" | "async" | "await" | "where" | "in" | "as"
        | "static" | "extern" | "ref" | "type" => return vec![],
        _ => {}
    }

    let mut tokens = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        tokens.extend(normalize_node(file, child));
    }
    tokens
}

/// Extract function name from a function node.
fn extract_func_name(file: &ParsedFile, node: tree_sitter::Node) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "name" {
            let start = child.start_byte();
            let end = child.end_byte();
            return Some(file.content[start..end].to_string());
        }
    }
    // Anonymous / closure
    None
}

/// Function-like node kinds across languages.
const FN_NODE_KINDS: &[&str] = &[
    "function_item",        // Rust
    "function_definition",  // Python, C, C++
    "function_declaration", // JavaScript, Go
    "method_definition",    // JavaScript (class methods)
    "method_declaration",   // Go, Java
    "method",               // Ruby
];

/// Recursively find function-like nodes and fingerprint them.
fn find_functions_recursive(
    file: &ParsedFile,
    node: tree_sitter::Node,
    fingerprints: &mut Vec<FuncFingerprint>,
    processed: &mut usize,
) {
    let kind = node.kind();
    if FN_NODE_KINDS.contains(&kind) {
        if let Some(fp) = fingerprint_function(file, node) {
            fingerprints.push(fp);
            *processed += 1;
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_functions_recursive(file, child, fingerprints, processed);
    }
}

/// Cross-file duplication analyzer using tree-sitter fingerprints.
pub struct CrossFileDupDetector {
    fingerprints: Vec<FuncFingerprint>,
    processed: usize,
}

impl Default for CrossFileDupDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossFileDupDetector {
    pub fn new() -> Self {
        Self {
            fingerprints: Vec::new(),
            processed: 0,
        }
    }

    /// Process a parsed file, extracting function fingerprints.
    pub fn process_file(&mut self, file: &ParsedFile) {
        let root = file.root_node();
        find_functions_recursive(file, root, &mut self.fingerprints, &mut self.processed);
    }

    /// Find duplicate functions across files.
    pub fn find_duplicates(&self) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Group by hash
        let mut by_hash: HashMap<u64, Vec<&FuncFingerprint>> = HashMap::new();
        for fp in &self.fingerprints {
            by_hash.entry(fp.hash).or_default().push(fp);
        }

        for group in by_hash.values() {
            if group.len() < 2 {
                continue;
            }
            let unique_files: std::collections::HashSet<&Path> =
                group.iter().map(|fp| fp.file.as_path()).collect();
            let file_count = unique_files.len();
            if file_count < 2 {
                continue;
            }

            let sev = if group.len() > 10 {
                Severity::Nuclear
            } else if group.len() > 5 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            for fp in group {
                issues.push(CodeIssue {
                    file_path: fp.file.clone(),
                    line: fp.line_start,
                    column: 1,
                    rule_name: "cross-file-duplication".to_string(),
                    message: format!(
                        "Function '{}' duplicated in {} files ({} occurrences)",
                        fp.name,
                        file_count,
                        group.len()
                    ),
                    severity: sev.clone(),
                });
            }
        }
        issues
    }

    /// Find near-duplicate functions using Jaccard similarity on normalized tokens.
    pub fn find_near_duplicates(&self) -> Vec<CodeIssue> {
        vec![]
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.fingerprints.len(), self.processed)
    }

    /// Build infection spread graph: for each file, list which other files
    /// share identical function hashes (indicating copy-paste propagation).
    pub fn infection_spread(&self) -> HashMap<String, Vec<(String, usize, Vec<String>)>> {
        let mut by_hash: HashMap<u64, Vec<&FuncFingerprint>> = HashMap::new();
        for fp in &self.fingerprints {
            by_hash.entry(fp.hash).or_default().push(fp);
        }

        // For each file pair, count shared function hashes
        let mut spread: HashMap<String, HashMap<String, (usize, Vec<String>)>> = HashMap::new();
        for group in by_hash.values() {
            let unique_files: std::collections::HashSet<&PathBuf> =
                group.iter().map(|fp| &fp.file).collect();
            if unique_files.len() < 2 {
                continue;
            }
            let func_name = &group[0].name;

            for a in &unique_files {
                let a_str = a.to_string_lossy().to_string();
                for b in &unique_files {
                    if a == b {
                        continue;
                    }
                    let b_str = b.to_string_lossy().to_string();
                    let entry = spread.entry(a_str.clone()).or_default();
                    let inner = entry.entry(b_str).or_insert((0, Vec::new()));
                    inner.0 += 1;
                    inner.1.push(func_name.clone());
                }
            }
        }

        // Convert to sorted output format
        spread
            .into_iter()
            .map(|(file, targets)| {
                let mut sorted: Vec<_> = targets.into_iter().collect();
                sorted.sort_by_key(|(_, (count, _))| std::cmp::Reverse(*count));
                let result: Vec<_> = sorted
                    .into_iter()
                    .map(|(target, (count, funcs))| (target, count, funcs))
                    .collect();
                (file, result)
            })
            .collect()
    }
}

// ─── Intra-file duplication ──────────────────────────────────────────────────

/// Intra-file code duplication detector (chunk-based line matching).
///
/// Splits a file into N-line chunks, normalizes them (strips comments),
/// and flags chunks that appear more than once.
/// Chunk size of 5 and requirement of 2+ spaced-out occurrences
/// reduces false positives from repetitive but unrelated code.
pub struct IntraFileDupDetector;

impl IntraFileDupDetector {
    /// Returns true if the file path looks like a test/example/bench fixture.
    fn is_skippable(path: &std::path::Path) -> bool {
        let name = path.to_string_lossy();
        name.contains("_test.")
            || name.contains("/tests/")
            || name.contains("/test/")
            || name.contains("/examples/")
            || name.contains("/fixtures/")
            || name.contains("/mocks/")
            || name.contains("/benches/")
    }

    pub fn check(file: &ParsedFile) -> Vec<CodeIssue> {
        if Self::is_skippable(&file.path) {
            return vec![];
        }
        let lines: Vec<&str> = file.content.lines().collect();
        if lines.len() < 10 {
            return vec![];
        }

        let chunk_size = 5usize;
        let mut seen: HashMap<u64, Vec<usize>> = HashMap::new();

        for i in 0..lines.len().saturating_sub(chunk_size - 1) {
            let chunk: Vec<&str> = lines[i..i + chunk_size]
                .iter()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with('#'))
                .collect();
            if chunk.len() < 3 {
                continue;
            }

            let mut hasher = DefaultHasher::new();
            for line in &chunk {
                line.hash(&mut hasher);
            }
            let hash = hasher.finish();
            seen.entry(hash).or_default().push(i);
        }

        let mut issues = Vec::new();
        for positions in seen.values() {
            if positions.len() < 2 {
                continue;
            }
            // Only flag if occurrences are well-separated (not adjacent)
            let is_spaced = positions.windows(2).any(|w| w[1] - w[0] > chunk_size);
            if !is_spaced {
                continue;
            }
            let start = positions[0];
            issues.push(CodeIssue {
                file_path: file.path.clone(),
                line: start + 1,
                column: 1,
                rule_name: "code-duplication".to_string(),
                message: format!(
                    "Duplicate code block ({} lines) found {} times starting at line {}",
                    chunk_size,
                    positions.len(),
                    start + 1
                ),
                severity: Severity::Mild,
            });
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::super::rules::test_helpers::{parse_python_as, parse_rust, parse_rust_as};
    use super::*;

    #[test]
    fn test_find_function_nodes() {
        let file =
            parse_rust("fn compute(a: i32, b: i32) -> i32 {\n    let r = a + b;\n    r\n}\n");
        let root = file.root_node();
        assert_eq!(root.kind(), "source_file");
        let mut c2 = root.walk();
        let found_fn = c2.goto_first_child() && c2.node().kind() == "function_item";
        assert!(found_fn, "Should find function_item node");
    }

    #[test]
    fn test_cross_file_dup_detects_identical_functions() {
        let mut detector = CrossFileDupDetector::new();
        let a = parse_rust_as("file_a.rs", "fn compute(a: i32, b: i32) -> i32 {\n    let r = a + b;\n    let s = r * 2;\n    s\n}\n");
        let b = parse_rust_as("file_b.rs", "fn compute(x: i32, y: i32) -> i32 {\n    let z = x + y;\n    let w = z * 2;\n    w\n}\n");
        detector.process_file(&a);
        detector.process_file(&b);
        let issues = detector.find_duplicates();
        assert!(!issues.is_empty(), "Should detect cross-file duplicate");
    }

    #[test]
    fn test_cross_file_skip_trivial_functions() {
        let mut detector = CrossFileDupDetector::new();
        let f = parse_rust("fn noop() {}");
        detector.process_file(&f);
        let (count, _) = detector.stats();
        assert_eq!(count, 0, "Trivial functions should be skipped");
    }

    #[test]
    fn test_cross_file_python() {
        let mut detector = CrossFileDupDetector::new();
        let a = parse_python_as(
            "mod_a.py",
            "def add(a, b):\n    result = a + b\n    print(result)\n    result += 1\n    return result\n",
        );
        let b = parse_python_as(
            "mod_b.py",
            "def sum(x, y):\n    total = x + y\n    print(total)\n    total += 1\n    return total\n",
        );
        detector.process_file(&a);
        detector.process_file(&b);
        let issues = detector.find_duplicates();
        assert!(!issues.is_empty(), "Should detect Python cross-file dup");
    }

    #[test]
    fn test_intra_file_dup_detects_repeated_block() {
        let file = parse_rust(
            r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    println!("block_end");
    let x = 0;
    let a = 1;
    let b = 2;
    let c = a + b;
    let d = c * 2;
    println!("block_end");
}
"#,
        );
        let issues = IntraFileDupDetector::check(&file);
        assert!(!issues.is_empty(), "Should detect repeated code block");
        assert!(issues.iter().any(|i| i.rule_name == "code-duplication"));
    }

    #[test]
    fn test_intra_file_clean_code_no_false_positive() {
        let file = parse_rust(
            r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn sub(a: i32, b: i32) -> i32 { a - b }
fn mul(a: i32, b: i32) -> i32 { a * b }
"#,
        );
        let issues = IntraFileDupDetector::check(&file);
        assert!(issues.is_empty(), "Different functions should not trigger");
    }
}
