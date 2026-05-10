use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;
use syn::{visit::Visit, Block, File};

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;
use crate::utils::get_position;

/// Static regex for string literal stripping, compiled once for performance.
static STRING_LITERAL_REGEX: OnceLock<Regex> = OnceLock::new();

/// Raw pattern strings for common Rust patterns (not compiled)
static RUST_COMMON_PATTERN_STRINGS: &[&str] = &[
    // Struct initialization patterns
    r"self\.\w+\.push\(\w+::\{",
    r"\w+\s*\{",
    r"file_path:\s*self\.\w+\.clone\(\)",
    r"rule_name:\s*.*\.to_string\(\)",
    r"message:\s*messages\[",
    r"severity:\s*Severity::",
    // Common method chains
    r"\.clone\(\)",
    r"\.to_string\(\)",
    r"\.to_lowercase\(\)",
    r"\.len\(\)",
    r"\.is_empty\(\)",
    r"\.unwrap\(\)",
    r"\.expect\(",
    // Common control flow
    r"if\s+.*\s*\{",
    r"for\s+.*\s+in\s+",
    r"match\s+.*\s*\{",
    r"let\s+.*=.*;",
    // Common collection operations
    r"\.push\(",
    r"\.insert\(",
    r"\.get\(",
    r".*get_or_insert",
    r"\.entry\(",
    // Visitor pattern (very common in Rust analyzers)
    r"fn\s+visit_\w+",
    r"syn::visit::visit_\w+",
];

/// Pre-compiled regex patterns using OnceLock for performance
static COMPILED_RUST_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_compiled_rust_patterns() -> &'static [Regex] {
    COMPILED_RUST_PATTERNS.get_or_init(|| {
        let mut compiled = Vec::with_capacity(RUST_COMMON_PATTERN_STRINGS.len());
        let mut errors = Vec::new();

        for (index, pattern) in RUST_COMMON_PATTERN_STRINGS.iter().enumerate() {
            match Regex::new(pattern) {
                Ok(regex) => compiled.push(regex),
                Err(e) => {
                    let error_msg = format!(
                        "[{}] Invalid regex pattern at index {}: '{}'\n       Error: {}",
                        file!(),
                        index,
                        pattern,
                        e
                    );
                    eprintln!("{}", error_msg);
                    errors.push(error_msg);
                }
            }

            if !errors.is_empty() && index == RUST_COMMON_PATTERN_STRINGS.len() - 1 {
                panic!(
                    "Failed to compile {} out of {} regex patterns:\n\n{}\n\n\
                     This is a critical error in the static pattern definitions.\n\
                     Please fix the invalid patterns listed above.",
                    errors.len(),
                    RUST_COMMON_PATTERN_STRINGS.len(),
                    errors.join("\n")
                );
            }
        }

        if compiled.is_empty() && !RUST_COMMON_PATTERN_STRINGS.is_empty() {
            panic!(
                "All {} regex patterns failed to compile!\n\
                 This indicates a systematic problem with the pattern definitions.",
                RUST_COMMON_PATTERN_STRINGS.len()
            );
        }

        compiled
    })
}

/// code duplication detection rule with smart anti-false-positive logic
pub struct CodeDuplicationRule;

impl Rule for CodeDuplicationRule {
    fn name(&self) -> &'static str {
        "code-duplication"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }

        let mut visitor = DuplicationVisitor::new(file_path.to_path_buf(), content, lang);
        visitor.visit_file(syntax_tree);
        visitor.find_duplications()
    }
}

struct DuplicationVisitor {
    file_path: std::path::PathBuf,
    content: String,
    code_blocks: Vec<(String, usize)>,
    line_hashes: HashMap<String, Vec<usize>>,
    lang: String,
}

impl DuplicationVisitor {
    fn new(file_path: std::path::PathBuf, content: &str, lang: &str) -> Self {
        Self {
            file_path,
            content: content.to_string(),
            code_blocks: Vec::new(),
            line_hashes: HashMap::new(),
            lang: lang.to_string(),
        }
    }

    fn find_duplications(&mut self) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // detect LINE-LEVEL duplications (with smart filtering)
        self.detect_line_duplications(&mut issues);

        // detect BLOCK-LEVEL duplications (multi-line copy-paste)
        self.detect_block_duplications(&mut issues);

        // detect CONSECUTIVE duplications (the real copy-paste)
        self.detect_consecutive_duplications(&mut issues);

        issues
    }

    fn detect_line_duplications(&mut self, issues: &mut Vec<CodeIssue>) {
        let lines: Vec<&str> = self.content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // ignore empty lines, comments, and simple statements
            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with("*")
                || trimmed.len() < 15
                || is_simple_statement(trimmed)
            {
                continue;
            }

            // Skip lines that match common Rust patterns (anti-false-positive)
            if is_common_rust_pattern(trimmed) {
                continue;
            }

            // Skip lines that are inside string literals
            if is_string_literal_line(trimmed) {
                continue;
            }

            // Skip struct initialization patterns (very common in Rust)
            if is_struct_initialization(trimmed) {
                continue;
            }

            let normalized = normalize_line_smart(trimmed);
            if normalized.len() < 10 {
                continue;
            }

            self.line_hashes
                .entry(normalized)
                .or_default()
                .push(line_num + 1);
        }

        // find duplicate lines with HIGHER threshold to reduce false positives
        for line_numbers in self.line_hashes.values() {
            let count = line_numbers.len();

            // Increased threshold: need at least 25 repetitions (was 10)
            // This filters out common patterns like struct initialization
            if count >= 25 {
                let messages = self.generate_dup_messages(count);

                let severity = if count >= 40 {
                    Severity::Nuclear
                } else if count >= 30 {
                    Severity::Spicy
                } else {
                    Severity::Mild
                };

                issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line: line_numbers[0],
                    column: 1,
                    rule_name: "code-duplication".to_string(),
                    message: messages[issues.len() % messages.len()].clone(),
                    severity,
                });

                // Only report top 3 instances to avoid spam
                if issues.len() >= 3 {
                    break;
                }
            }
        }
    }

    fn detect_block_duplications(&self, issues: &mut Vec<CodeIssue>) {
        let mut block_signatures: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, (block_str, _line)) in self.code_blocks.iter().enumerate() {
            if block_str.len() > 500 {
                let signature = generate_block_signature_smart(block_str);
                block_signatures.entry(signature).or_default().push(i);
            }
        }

        for (_, block_indices) in block_signatures {
            if block_indices.len() >= 8 {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!("发现 {} 个相似代码块，考虑重构成函数", block_indices.len()),
                        "代码块重复度过高，DRY原则哭了".to_string(),
                        format!("检测到 {} 个相似代码块，重构时间到了", block_indices.len()),
                    ]
                } else {
                    vec![
                        format!(
                            "Similar code blocks detected: {} instances",
                            block_indices.len()
                        ),
                        format!(
                            "Refactoring opportunity: {} similar blocks found",
                            block_indices.len()
                        ),
                        "Code block duplication too high, DRY principle is crying".to_string(),
                    ]
                };

                let line = self.code_blocks[block_indices[0]].1;

                issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line,
                    column: 1,
                    rule_name: "code-duplication".to_string(),
                    message: messages[issues.len() % messages.len()].clone(),
                    severity: Severity::Spicy,
                });
            }
        }
    }

    /// Detect consecutive duplicate lines (REAL copy-paste detection)
    /// This finds actual copy-pasted code blocks, not just similar patterns
    fn detect_consecutive_duplications(&self, issues: &mut Vec<CodeIssue>) {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut i = 0;

        while i < lines.len().saturating_sub(3) {
            let current = normalize_line_smart(lines[i].trim());

            if current.is_empty() || current.len() < 15 {
                i += 1;
                continue;
            }

            // Look ahead for consecutive identical patterns
            let mut dup_count = 1;
            let mut start_line = i + 1;

            while start_line < lines.len() && dup_count < 5 {
                let next_normalized = normalize_line_smart(lines[start_line].trim());
                if next_normalized == current && !is_common_rust_pattern(lines[start_line].trim()) {
                    dup_count += 1;
                    start_line += 1;
                } else {
                    break;
                }
            }

            // Report only if we found 4+ consecutive identical lines
            if dup_count >= 4 {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!("发现连续 {} 行完全相同的代码！这是复制粘贴！", dup_count),
                        format!("{} 行重复代码块，建议提取为函数或宏", dup_count),
                    ]
                } else {
                    vec![
                        format!(
                            "Found {} consecutive identical lines! This looks like copy-paste!",
                            dup_count
                        ),
                        format!(
                            "{} line duplicate block detected - consider extracting to function/macro",
                            dup_count
                        ),
                    ]
                };

                issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line: i + 1,
                    column: 1,
                    rule_name: "code-duplication".to_string(),
                    message: messages[0].clone(),
                    severity: Severity::Spicy,
                });

                i = start_line; // Skip past this block
            } else {
                i += 1;
            }
        }
    }

    fn generate_dup_messages(&self, count: usize) -> Vec<String> {
        if self.lang == "zh-CN" {
            vec![
                format!("检测到 {} 次重复代码！你是复制粘贴大师吗？", count),
                format!("这行代码重复了 {} 次，建议提取成函数", count),
                format!("重复代码警报！{} 次重复让维护变成噩梦", count),
                format!("复制粘贴忍者出现！{} 行相同代码", count),
                format!("违反 DRY 原则：{} 行重复代码", count),
            ]
        } else {
            vec![
                format!("Copy-paste ninja detected! {} identical lines found", count),
                format!("DRY principle violation: {} duplicated lines", count),
                format!("Code duplication alert! {} repetitions found", count),
                format!(
                    "This line repeated {} times - consider extracting to function",
                    count
                ),
                format!("Maintenance nightmare: {} duplicate lines detected", count),
            ]
        }
    }
}

impl<'ast> Visit<'ast> for DuplicationVisitor {
    fn visit_block(&mut self, block: &'ast Block) {
        let block_str = format!("{block:?}");
        if block_str.len() > 50 {
            let (line, _) = get_position(block);
            self.code_blocks.push((block_str, line));
        }
        syn::visit::visit_block(self, block);
    }
}

/// Smart normalization that preserves semantic differences
fn normalize_line_smart(line: &str) -> String {
    let re = STRING_LITERAL_REGEX.get_or_init(|| Regex::new(r#""[^"]*""#).unwrap());

    let stripped = re.replace_all(line.trim(), "STR");

    stripped.replace(char::is_whitespace, "").to_lowercase()
}

/// Check if a line matches common Rust patterns that should be ignored
fn is_common_rust_pattern(line: &str) -> bool {
    let trimmed = line.trim();

    for pattern in get_compiled_rust_patterns().iter() {
        if pattern.is_match(trimmed) {
            return true;
        }
    }

    false
}

/// Check if this line is a struct initialization pattern
fn is_struct_initialization(line: &str) -> bool {
    let trimmed = line.trim();

    // Pattern: SomeStruct { field: value, ... }
    if trimmed.contains('{') && trimmed.contains('}') {
        // Count the number of fields being set
        let field_count = trimmed.matches(':').count();

        // If it has multiple fields (>= 3), it's likely a struct init
        if field_count >= 3 {
            return true;
        }
    }

    // Pattern: self.issues.push(CodeIssue { ... })
    if trimmed.contains(".push(") && trimmed.contains("{") {
        return true;
    }

    // Pattern: CodeIssue { ... } or similar struct literals
    if Regex::new(r"\w+\s*\{[^}]*file_path:")
        .map(|re| re.is_match(trimmed))
        .unwrap_or(false)
    {
        return true;
    }

    false
}

fn is_simple_statement(line: &str) -> bool {
    matches!(line.trim(), "{" | "}" | ";" | "(" | ")" | "[" | "]")
}

fn is_string_literal_line(line: &str) -> bool {
    let trimmed = line.trim();

    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        return true;
    }
    if trimmed.starts_with('"') && (trimmed.ends_with("\",") || trimmed.ends_with(',')) {
        return true;
    }
    if trimmed.starts_with("format!") || trimmed.starts_with("format!(") {
        return true;
    }
    if trimmed.starts_with("\"") && !trimmed.contains("fn ") && !trimmed.contains("let ") {
        return true;
    }
    false
}

/// Smart block signature generation that ignores variable names but preserves structure
fn generate_block_signature_smart(block: &str) -> String {
    block
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(300)
        .collect::<String>()
        .to_lowercase()
}
