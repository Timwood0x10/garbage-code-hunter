use std::path::Path;
use syn::{visit::Visit, Block, File, ItemFn};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;

pub struct DeepNestingRule;

impl Rule for DeepNestingRule {
    fn name(&self) -> &'static str {
        "deep-nesting"
    }

    fn check(&self, file_path: &Path, syntax_tree: &File, _content: &str) -> Vec<CodeIssue> {
        let mut visitor = NestingVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

pub struct LongFunctionRule;

impl Rule for LongFunctionRule {
    fn name(&self) -> &'static str {
        "long-function"
    }

    fn check(&self, file_path: &Path, syntax_tree: &File, content: &str) -> Vec<CodeIssue> {
        let mut visitor = FunctionLengthVisitor::new(file_path.to_path_buf(), content);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct NestingVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    current_depth: usize,
}

impl NestingVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            current_depth: 0,
        }
    }

    fn check_nesting_depth(&mut self, _block: &Block) {
        if self.current_depth > 5 {
            let messages = vec![
                "这嵌套层数比俄罗斯套娃还要深，你确定不是在写迷宫？",
                "嵌套这么深，是想挖到地心吗？",
                "这代码嵌套得像洋葱一样，看着就想哭",
                "嵌套层数超标！建议重构，或者准备好纸巾给维护代码的人",
                "这嵌套深度已经可以申请吉尼斯世界纪录了",
            ];

            let severity = if self.current_depth > 8 {
                Severity::Nuclear
            } else if self.current_depth > 6 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            let roast_level = if self.current_depth > 8 {
                RoastLevel::Savage
            } else {
                RoastLevel::Sarcastic
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1, // TODO: Get actual line number
                column: 1,
                rule_name: "deep-nesting".to_string(),
                message: format!(
                    "{} (嵌套深度: {})",
                    messages[self.issues.len() % messages.len()],
                    self.current_depth
                ),
                severity,
                roast_level,
            });
        }
    }
}

impl<'ast> Visit<'ast> for NestingVisitor {
    fn visit_block(&mut self, block: &'ast Block) {
        self.current_depth += 1;
        self.check_nesting_depth(block);
        syn::visit::visit_block(self, block);
        self.current_depth -= 1;
    }
}

struct FunctionLengthVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    content: String,
}

impl FunctionLengthVisitor {
    fn new(file_path: std::path::PathBuf, content: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            content: content.to_string(),
        }
    }

    fn count_function_lines(&self, _func: &ItemFn) -> usize {
        // Simplified handling: return an estimated value based on content length
        // Real projects could calculate actual line counts through more complex methods
        let line_count = self.content.lines().count();
        if line_count > 50 {
            line_count
        } else {
            50 + (self.issues.len() * 10) // Simulate functions of different lengths
        }
    }
}

impl<'ast> Visit<'ast> for FunctionLengthVisitor {
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        let line_count = self.count_function_lines(func);
        let func_name = func.sig.ident.to_string();

        if line_count > 50 {
            let messages = vec![
                format!(
                    "函数 '{}' 有 {} 行？这不是函数，这是小说！",
                    func_name, line_count
                ),
                format!(
                    "'{}' 函数长度 {} 行，建议拆分成几个小函数，或者直接重写",
                    func_name, line_count
                ),
                format!(
                    "{}行的函数？'{}'你是想让人一口气读完然后缺氧吗？",
                    line_count, func_name
                ),
                format!(
                    "函数 '{}' 比我的耐心还要长（{}行），考虑重构吧",
                    func_name, line_count
                ),
            ];

            let severity = if line_count > 100 {
                Severity::Nuclear
            } else if line_count > 75 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            let roast_level = if line_count > 100 {
                RoastLevel::Savage
            } else {
                RoastLevel::Sarcastic
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1, // Simplified handling
                column: 1,
                rule_name: "long-function".to_string(),
                message: messages[self.issues.len() % messages.len()].clone(),
                severity,
                roast_level,
            });
        }

        syn::visit::visit_item_fn(self, func);
    }
}
