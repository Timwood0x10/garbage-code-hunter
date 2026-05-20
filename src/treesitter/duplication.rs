use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::analyzer::{CodeIssue, Severity};
use crate::treesitter::engine::ParsedFile;

// ─── Cross-file duplication ──────────────────────────────────────────────────

type TokenSetPair = (
    std::collections::HashSet<FToken>,
    std::collections::HashSet<(FToken, FToken)>,
);

/// Normalized token for function fingerprinting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FToken {
    Ident(u16),
    Int,
    Float,
    Str,
    Bool,
    Type,
    Kind(u16),
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
fn fingerprint_function(
    file: &ParsedFile,
    node: tree_sitter::Node,
) -> Option<(FuncFingerprint, Vec<FToken>)> {
    let name = extract_func_name(file, node)?;
    let line_start = node.start_position().row + 1;
    let line_end = node.end_position().row + 1;
    let line_count = line_end - line_start + 1;

    if line_count < 8 || is_adapter_batch_helper(&name) {
        return None;
    }

    let tokens = normalize_node(file, node);
    if tokens.len() < 20 {
        return None;
    }

    let mut hasher = DefaultHasher::new();
    tokens.hash(&mut hasher);
    let hash = hasher.finish();

    Some((
        FuncFingerprint {
            hash,
            name,
            file: file.path.clone(),
            line_start,
        },
        tokens,
    ))
}

fn hash_text(text: &str) -> u16 {
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    (h.finish() & u16::MAX as u64) as u16
}

fn is_adapter_batch_helper(name: &str) -> bool {
    (name.starts_with("count_") || name.starts_with("extract_"))
        && (name.ends_with("_from_batch") || name.ends_with("_from_batch_with"))
}

fn is_skippable_dup_path(path: &std::path::Path) -> bool {
    let name = path.to_string_lossy();
    name.contains("_test.")
        || name.starts_with("example/")
        || name.starts_with("examples/")
        || name.starts_with("demo/")
        || name.contains("/tests/")
        || name.contains("/test/")
        || name.contains("/example/")
        || name.contains("/examples/")
        || name.contains("/demo/")
        || name.contains("/fixtures/")
        || name.contains("/mocks/")
        || name.contains("/benches/")
}

#[allow(clippy::only_used_in_recursion)]
fn normalize_node(file: &ParsedFile, node: tree_sitter::Node) -> Vec<FToken> {
    let kind = node.kind();
    match kind {
        "identifier"
        | "field_identifier"
        | "shorthand_property_identifier"
        | "variable_name"
        | "name" => {
            let start = node.start_byte();
            let end = node.end_byte();
            let text = &file.content[start..end];
            return vec![FToken::Ident(hash_text(text))];
        }
        "type_identifier" | "primitive_type" | "mutable_specifier" => return vec![FToken::Type],
        "integer_literal" | "integer" | "number" => return vec![FToken::Int],
        "float_literal" | "float" => return vec![FToken::Float],
        "string_literal" | "string" | "interpreted_string_literal" | "char_literal" => {
            return vec![FToken::Str];
        }
        "boolean" | "true" | "false" => return vec![FToken::Bool],
        "let" | "const" | "mut" | "fn" | "def" | "function" | "fun" | "return" | "if" | "else"
        | "for" | "while" | "match" | "use" | "mod" | "struct" | "enum" | "impl" | "trait"
        | "pub" | "pub(crate)" | "unsafe" | "async" | "await" | "where" | "in" | "as"
        | "static" | "extern" | "ref" | "type" => return vec![],
        _ => {}
    }

    let mut tokens = vec![FToken::Kind(hash_text(kind))];
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
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

fn build_token_bigrams(tokens: &[FToken]) -> std::collections::HashSet<(FToken, FToken)> {
    tokens.windows(2).map(|w| (w[0], w[1])).collect()
}

/// Recursively find function-like nodes and fingerprint them.
fn find_functions_recursive(
    file: &ParsedFile,
    node: tree_sitter::Node,
    fingerprints: &mut Vec<FuncFingerprint>,
    token_sets: &mut Vec<TokenSetPair>,
    processed: &mut usize,
) {
    let kind = node.kind();
    if FN_NODE_KINDS.contains(&kind) {
        if let Some((fp, tokens)) = fingerprint_function(file, node) {
            let type_set: std::collections::HashSet<FToken> = tokens.iter().copied().collect();
            let bigram_set = build_token_bigrams(&tokens);
            fingerprints.push(fp);
            token_sets.push((type_set, bigram_set));
            *processed += 1;
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_functions_recursive(file, child, fingerprints, token_sets, processed);
    }
}

/// Cross-file duplication analyzer using tree-sitter fingerprints.
pub struct CrossFileDupDetector {
    fingerprints: Vec<FuncFingerprint>,
    token_sets: Vec<TokenSetPair>,
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
            token_sets: Vec::new(),
            processed: 0,
        }
    }

    /// Process a parsed file, extracting function fingerprints.
    pub fn process_file(&mut self, file: &ParsedFile) {
        if is_skippable_dup_path(&file.path) {
            return;
        }
        let root = file.root_node();
        find_functions_recursive(
            file,
            root,
            &mut self.fingerprints,
            &mut self.token_sets,
            &mut self.processed,
        );
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

            let representative = group
                .iter()
                .min_by_key(|fp| (fp.file.as_path(), fp.line_start))
                .copied()
                .unwrap_or(group[0]);
            issues.push(CodeIssue {
                file_path: representative.file.clone(),
                line: representative.line_start,
                column: 1,
                rule_name: "cross-file-duplication".to_string(),
                message: format!(
                    "Function '{}' duplicated in {} files ({} occurrences)",
                    representative.name,
                    file_count,
                    group.len()
                ),
                severity: sev.clone(),
            });
        }
        issues
    }

    /// Find near-duplicate functions using Jaccard similarity on normalized tokens.
    pub fn find_near_duplicates(&self) -> Vec<CodeIssue> {
        if self.fingerprints.len() < 2 {
            return vec![];
        }

        let mut by_len: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..self.fingerprints.len() {
            let bucket = self.token_sets[i].1.len() / 5;
            by_len.entry(bucket).or_default().push(i);
        }

        let mut reported = std::collections::HashSet::new();
        let mut issues = Vec::new();

        for indices in by_len.values() {
            for pair in indices.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                if self.fingerprints[a].file == self.fingerprints[b].file {
                    continue;
                }
                if self.fingerprints[a].hash == self.fingerprints[b].hash {
                    continue;
                }

                let (types_a, bigrams_a) = &self.token_sets[a];
                let (types_b, bigrams_b) = &self.token_sets[b];

                let bigram_inter = bigrams_a.intersection(bigrams_b).count();
                let bigram_union = bigrams_a.union(bigrams_b).count();
                if bigram_union == 0 {
                    continue;
                }
                let similarity = bigram_inter as f64 / bigram_union as f64;

                let type_inter = types_a.intersection(types_b).count();
                let type_union = types_a.union(types_b).count();
                let type_ratio = if type_union > 0 {
                    type_inter as f64 / type_union as f64
                } else {
                    0.0
                };

                if similarity >= 0.95
                    && bigrams_a.len() >= 20
                    && bigrams_b.len() >= 20
                    && type_ratio >= 0.85
                {
                    for &idx in &[a, b] {
                        if reported.insert(idx) {
                            issues.push(CodeIssue {
                                file_path: self.fingerprints[idx].file.clone(),
                                line: self.fingerprints[idx].line_start,
                                column: 1,
                                rule_name: "near-duplicate".to_string(),
                                message: format!(
                                    "Function '{}' is near-identical to '{}' ({:.0}% similar)",
                                    self.fingerprints[a].name,
                                    self.fingerprints[b].name,
                                    similarity * 100.0,
                                ),
                                severity: Severity::Mild,
                            });
                        }
                    }
                }
            }
        }

        issues
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
        is_skippable_dup_path(path)
    }

    fn is_substantive_line(line: &str) -> bool {
        let trimmed = line.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with('#')
            && !matches!(trimmed, "{" | "}" | ");" | "," | "else" | "} else {")
            && !trimmed.starts_with("}")
            && !trimmed.ends_with("{")
            && !trimmed.ends_with("=>")
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
                .filter(|l| Self::is_substantive_line(l))
                .collect();
            if chunk.len() < 4 {
                continue;
            }

            let mut hasher = DefaultHasher::new();
            for line in &chunk {
                line.hash(&mut hasher);
            }
            let hash = hasher.finish();
            seen.entry(hash).or_default().push(i);
        }

        let mut groups: Vec<(usize, usize)> = seen
            .values()
            .filter_map(|positions| {
                if positions.len() < 2 {
                    return None;
                }
                let is_spaced = positions.windows(2).any(|w| w[1] - w[0] > chunk_size);
                if is_spaced {
                    Some((positions[0], positions.len()))
                } else {
                    None
                }
            })
            .collect();
        groups.sort_by_key(|(start, _)| *start);

        let mut issues = Vec::new();
        let mut last_reported_end = 0usize;
        for (start, occurrences) in groups {
            if start < last_reported_end {
                continue;
            }
            last_reported_end = start + chunk_size;

            issues.push(CodeIssue {
                file_path: file.path.clone(),
                line: start + 1,
                column: 1,
                rule_name: "code-duplication".to_string(),
                message: format!(
                    "Duplicate code block ({} lines) found {} times starting at line {}",
                    chunk_size,
                    occurrences,
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
    use super::*;
    use crate::treesitter::engine::TreeSitterEngine;
    use std::path::Path;
    use std::sync::OnceLock;

    fn shared_engine() -> &'static TreeSitterEngine {
        static ENGINE: OnceLock<TreeSitterEngine> = OnceLock::new();
        ENGINE.get_or_init(|| {
            let engine = TreeSitterEngine::new();
            engine.ensure_parser(crate::language::Language::Rust);
            engine.ensure_parser(crate::language::Language::Python);
            engine
        })
    }

    fn parse_rust(code: &str) -> ParsedFile {
        shared_engine()
            .parse_file(Path::new("main.rs"), code)
            .expect("Rust parse should succeed")
    }

    fn parse_rust_as(filename: &str, code: &str) -> ParsedFile {
        shared_engine()
            .parse_file(Path::new(filename), code)
            .expect("Rust parse should succeed")
    }

    fn parse_python_as(filename: &str, code: &str) -> ParsedFile {
        shared_engine()
            .parse_file(Path::new(filename), code)
            .expect("Python parse should succeed")
    }

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
        let code = r#"
fn compute(a: i32, b: i32) -> i32 {
    let base = a + b;
    let doubled = base * 2;
    let shifted = doubled + 10;
    let checked = shifted - a;
    let final_value = checked + b;
    final_value
}
"#;
        let a = parse_rust_as("file_a.rs", code);
        let b = parse_rust_as("file_b.rs", code);
        detector.process_file(&a);
        detector.process_file(&b);
        let issues = detector.find_duplicates();
        assert!(!issues.is_empty(), "Should detect cross-file duplicate");
    }

    #[test]
    fn test_cross_file_duplicate_group_reports_once() {
        let mut detector = CrossFileDupDetector::new();
        let code = r#"
fn compute(a: i32, b: i32) -> i32 {
    let base = a + b;
    let doubled = base * 2;
    let shifted = doubled + 10;
    let checked = shifted - a;
    let final_value = checked + b;
    final_value
}
"#;
        let a = parse_rust_as("file_a.rs", code);
        let b = parse_rust_as("file_b.rs", code);
        let c = parse_rust_as("file_c.rs", code);
        detector.process_file(&a);
        detector.process_file(&b);
        detector.process_file(&c);
        let issues = detector.find_duplicates();
        assert_eq!(issues.len(), 1, "Duplicate group should report once");
        assert!(
            issues[0].message.contains("3 occurrences"),
            "message should preserve occurrence count"
        );
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
        let code = r#"
def add(a, b):
    result = a + b
    doubled = result * 2
    shifted = doubled + 10
    checked = shifted - a
    final_value = checked + b
    print(final_value)
    return final_value
"#;
        let a = parse_python_as("mod_a.py", code);
        let b = parse_python_as("mod_b.py", code);
        detector.process_file(&a);
        detector.process_file(&b);
        let issues = detector.find_duplicates();
        assert!(!issues.is_empty(), "Should detect Python cross-file dup");
    }

    #[test]
    fn test_cross_file_skips_adapter_batch_helpers() {
        let mut detector = CrossFileDupDetector::new();
        let code = r#"
fn count_debug_from_batch<'a>(batch: &[Vec<QueryCapture<'a>>]) -> usize {
    batch
        .iter()
        .filter(|m| m.iter().any(|c| c.name == "dp_method"))
        .count()
}
"#;
        let a = parse_rust_as("adapter_a.rs", code);
        let b = parse_rust_as("adapter_b.rs", code);
        detector.process_file(&a);
        detector.process_file(&b);
        let issues = detector.find_duplicates();
        assert!(issues.is_empty(), "Adapter batch helpers should be skipped");
    }

    #[test]
    fn test_cross_file_skips_demo_paths() {
        let mut detector = CrossFileDupDetector::new();
        let code = r#"
fn compute(a: i32, b: i32) -> i32 {
    let base = a + b;
    let doubled = base * 2;
    let shifted = doubled + 10;
    let checked = shifted - a;
    let final_value = checked + b;
    final_value
}
"#;
        let a = parse_rust_as("demo/file_a.rs", code);
        let b = parse_rust_as("demo/file_b.rs", code);
        detector.process_file(&a);
        detector.process_file(&b);
        let issues = detector.find_duplicates();
        assert!(issues.is_empty(), "Demo paths should be skipped");
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
    fn test_intra_file_dup_merges_overlapping_chunks() {
        let file = parse_rust(
            r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let marker = 0;
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
}
"#,
        );
        let issues = IntraFileDupDetector::check(&file);
        assert_eq!(issues.len(), 1, "Overlapping duplicate chunks should merge");
    }

    #[test]
    fn test_intra_file_dup_skips_example_paths() {
        let file = parse_rust_as(
            "example/demo.rs",
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
        assert!(issues.is_empty(), "Example paths should be skipped");
    }

    #[test]
    fn test_intra_file_dup_ignores_structural_lines() {
        let file = parse_rust(
            r#"
fn first(a: bool, b: bool, c: bool, d: bool) {
    if a {
        if b {
            if c {
                if d {
                    println!("one");
                }
            }
        }
    }
}

fn second(a: bool, b: bool, c: bool, d: bool) {
    if a {
        if b {
            if c {
                if d {
                    println!("two");
                }
            }
        }
    }
}
"#,
        );
        let issues = IntraFileDupDetector::check(&file);
        assert!(
            issues.is_empty(),
            "Structural lines should not trigger duplication"
        );
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
