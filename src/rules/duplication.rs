use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;
use syn::{visit::Visit, Block, File};

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;
use crate::utils::get_position;

/// Static regex for string literal stripping, compiled once for performance.
/// Matches content within double quotes for normalization purposes.
static STRING_LITERAL_REGEX: OnceLock<Regex> = OnceLock::new();

/// code duplication detection rule
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

        // detect line duplications
        self.detect_line_duplications(&mut issues);

        // detect block duplications
        self.detect_block_duplications(&mut issues);

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
                || trimmed.len() < 10
                || is_simple_statement(trimmed)
            {
                continue;
            }

            // Skip lines that are inside string literals (format! args, messages, etc.)
            if is_string_literal_line(trimmed) {
                continue;
            }

            let normalized = normalize_line(trimmed);
            self.line_hashes
                .entry(normalized)
                .or_default()
                .push(line_num + 1);
        }

        // find duplicate lines
        for line_numbers in self.line_hashes.values() {
            if line_numbers.len() >= 10 {
                // 10 times or more duplicate
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!(
                            "检测到 {} 次重复代码！你是复制粘贴大师吗？",
                            line_numbers.len()
                        ),
                        format!("这行代码重复了 {} 次，建议提取成函数", line_numbers.len()),
                        format!("重复代码警报！{} 次重复让维护变成噩梦", line_numbers.len()),
                        format!("复制粘贴忍者出现！{} 行相同代码", line_numbers.len()),
                        format!("违反 DRY 原则：{} 行重复代码", line_numbers.len()),
                    ]
                } else {
                    vec![
                        format!(
                            "Copy-paste ninja detected! {} identical lines found",
                            line_numbers.len()
                        ),
                        format!(
                            "DRY principle violation: {} duplicated lines",
                            line_numbers.len()
                        ),
                        format!(
                            "Code duplication alert! {} repetitions found",
                            line_numbers.len()
                        ),
                        format!(
                            "This line repeated {} times - consider extracting to function",
                            line_numbers.len()
                        ),
                        format!(
                            "Maintenance nightmare: {} duplicate lines detected",
                            line_numbers.len()
                        ),
                    ]
                };

                let severity = if line_numbers.len() >= 20 {
                    Severity::Nuclear
                } else if line_numbers.len() >= 15 {
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
            }
        }
    }

    fn detect_block_duplications(&self, issues: &mut Vec<CodeIssue>) {
        // simple block duplication detection
        let mut block_signatures: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, (block_str, _line)) in self.code_blocks.iter().enumerate() {
            if block_str.len() > 300 {
                // only detect larger code blocks
                let signature = generate_block_signature(block_str);
                block_signatures
                    .entry(signature)
                    .or_insert_with(Vec::new)
                    .push(i);
            }
        }

        for (_, block_indices) in block_signatures {
            if block_indices.len() >= 5 {
                // 3 or more similar blocks (raised from 2)
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!("发现 {} 个相似代码块，考虑重构成函数", block_indices.len()),
                        "代码块重复度过高，DRY原则哭了".to_string(),
                        format!("检测到 {} 个相似代码块，重构时间到了", block_indices.len()),
                        format!("代码重复警报：{} 个相似块需要整理", block_indices.len()),
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
                        format!(
                            "Maintenance alert: {} similar blocks need attention",
                            block_indices.len()
                        ),
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
}

impl<'ast> Visit<'ast> for DuplicationVisitor {
    fn visit_block(&mut self, block: &'ast Block) {
        // collect code blocks for duplication detection
        let block_str = format!("{block:?}");
        if block_str.len() > 20 {
            let (line, _) = get_position(block);
            self.code_blocks.push((block_str, line));
        }
        syn::visit::visit_block(self, block);
    }
}

fn normalize_line(line: &str) -> String {
    // Get or initialize the regex once, then reuse for all lines.
    let re = STRING_LITERAL_REGEX.get_or_init(|| Regex::new(r#""[^"]*""#).unwrap());
    // normalize code line: strip string literals, remove variable name differences
    let stripped = re.replace_all(line.trim(), "STR");
    stripped
        .replace(char::is_whitespace, "")
        .replace("let", "VAR")
        .replace("mut", "")
        .to_lowercase()
}

fn is_simple_statement(line: &str) -> bool {
    // check if the line is a simple statement
    matches!(line.trim(), "{" | "}" | ";" | "(" | ")" | "[" | "]")
}

fn is_string_literal_line(line: &str) -> bool {
    // Skip lines that are primarily string literal content (format args, messages, etc.)
    // These are often intentionally similar across rules
    let trimmed = line.trim();
    // Lines that are just a string literal in a vec or format macro
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        return true;
    }
    if trimmed.starts_with('"') && (trimmed.ends_with("\",") || trimmed.ends_with('"')) {
        return true;
    }
    // Lines with format! patterns
    if trimmed.starts_with("format!") || trimmed.starts_with("format!(") {
        return true;
    }
    // Lines inside vec![] that are just messages
    if trimmed.starts_with("\"") && !trimmed.contains("fn ") && !trimmed.contains("let ") {
        return true;
    }
    false
}

fn generate_block_signature(block: &str) -> String {
    // generate code block signature for similarity detection
    // Use first 500 characters for more accurate matching
    block
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(500)
        .collect::<String>()
        .to_lowercase()
}
