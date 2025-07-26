use std::collections::HashMap;
use std::path::Path;
use syn::{visit::Visit, Block, File};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;

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
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = DuplicationVisitor::new(file_path.to_path_buf(), content);
        visitor.visit_file(syntax_tree);
        visitor.find_duplications()
    }
}

struct DuplicationVisitor {
    file_path: std::path::PathBuf,
    content: String,
    code_blocks: Vec<String>,
    line_hashes: HashMap<String, Vec<usize>>,
}

impl DuplicationVisitor {
    fn new(file_path: std::path::PathBuf, content: &str) -> Self {
        Self {
            file_path,
            content: content.to_string(),
            code_blocks: Vec::new(),
            line_hashes: HashMap::new(),
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

            let normalized = normalize_line(trimmed);
            self.line_hashes
                .entry(normalized)
                .or_default()
                .push(line_num + 1);
        }

        // find duplicate lines
        for line_numbers in self.line_hashes.values() {
            if line_numbers.len() >= 3 {
                // 3 times or more duplicate
                let messages = [
                    format!(
                        "检测到 {} 次重复代码！你是复制粘贴大师吗？",
                        line_numbers.len()
                    ),
                    format!("这行代码重复了 {} 次，建议提取成函数", line_numbers.len()),
                    format!("重复代码警报！{} 次重复让维护变成噩梦", line_numbers.len()),
                    format!(
                        "Copy-paste ninja detected! {} identical lines found",
                        line_numbers.len()
                    ),
                    format!(
                        "DRY principle violation: {} duplicated lines",
                        line_numbers.len()
                    ),
                ];

                let severity = if line_numbers.len() >= 5 {
                    Severity::Nuclear
                } else if line_numbers.len() >= 4 {
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
                    roast_level: RoastLevel::Sarcastic,
                });
            }
        }
    }

    fn detect_block_duplications(&self, issues: &mut Vec<CodeIssue>) {
        // simple block duplication detection
        let mut block_signatures = HashMap::new();

        for (i, block) in self.code_blocks.iter().enumerate() {
            if block.len() > 50 {
                // only detect larger code blocks
                let signature = generate_block_signature(block);
                block_signatures
                    .entry(signature)
                    .or_insert_with(Vec::new)
                    .push(i);
            }
        }

        for (_, block_indices) in block_signatures {
            if block_indices.len() >= 2 {
                let messages = [
                    format!("发现 {} 个相似代码块，考虑重构成函数", block_indices.len()),
                    format!("代码块重复度过高，DRY原则哭了",),
                    format!(
                        "Similar code blocks detected: {} instances",
                        block_indices.len()
                    ),
                    format!(
                        "Refactoring opportunity: {} similar blocks found",
                        block_indices.len()
                    ),
                ];

                issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line: 1,
                    column: 1,
                    rule_name: "code-duplication".to_string(),
                    message: messages[issues.len() % messages.len()].clone(),
                    severity: Severity::Spicy,
                    roast_level: RoastLevel::Sarcastic,
                });
            }
        }
    }
}

impl<'ast> Visit<'ast> for DuplicationVisitor {
    fn visit_block(&mut self, block: &'ast Block) {
        // collect code blocks for duplication detection
        let block_str = format!("{:?}", block);
        if block_str.len() > 20 {
            self.code_blocks.push(block_str);
        }
        syn::visit::visit_block(self, block);
    }
}

fn normalize_line(line: &str) -> String {
    // normalize code line, remove variable name differences
    line.trim()
        .replace(char::is_whitespace, "")
        .replace("let", "VAR")
        .replace("mut", "")
        .to_lowercase()
}

fn is_simple_statement(line: &str) -> bool {
    // check if the line is a simple statement
    matches!(line.trim(), "{" | "}" | ";" | "(" | ")" | "[" | "]")
}

fn generate_block_signature(block: &str) -> String {
    // generate code block signature for similarity detection
    block
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(100)
        .collect::<String>()
        .to_lowercase()
}
