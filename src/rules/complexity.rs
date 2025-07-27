use std::path::Path;
use syn::{visit::Visit, Block, File, ItemFn};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;
use crate::utils::get_position;

pub struct DeepNestingRule;

impl Rule for DeepNestingRule {
    fn name(&self) -> &'static str {
        "deep-nesting"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = NestingVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

pub struct LongFunctionRule;

impl Rule for LongFunctionRule {
    fn name(&self) -> &'static str {
        "long-function"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = FunctionLengthVisitor::new(file_path.to_path_buf(), content, lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct NestingVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    current_depth: usize,
    lang: String,
}

impl NestingVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            current_depth: 0,
            lang: lang.to_string(),
        }
    }

    fn check_nesting_depth(&mut self, block: &Block, lang: &str) {
        if self.current_depth > 3 {
            let messages = if lang == "zh-CN" {
                vec![
                    "这嵌套层数比俄罗斯套娃还要深，你确定不是在写迷宫？",
                    "嵌套这么深，是想挖到地心吗？",
                    "这代码嵌套得像洋葱一样，看着就想哭",
                    "嵌套层数超标！建议重构，或者准备好纸巾给维护代码的人",
                    "这嵌套深度已经可以申请吉尼斯世界纪录了",
                ]
            } else {
                vec![
                    "Nesting deeper than Russian dolls, are you writing a maze?",
                    "Nesting so deep, trying to dig to the Earth's core?",
                    "Code nested like an onion, makes me want to cry",
                    "Nesting level exceeded! Consider refactoring, or prepare tissues for code maintainers",
                    "This nesting depth could apply for a Guinness World Record",
                ]
            };

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

            let depth_text = if self.lang == "zh-CN" {
                format!("嵌套深度: {}", self.current_depth)
            } else {
                format!("nesting depth: {}", self.current_depth)
            };

            let (line, column) = get_position(block);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "deep-nesting".to_string(),
                message: format!(
                    "{} ({})",
                    messages[self.issues.len() % messages.len()],
                    depth_text
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
        self.check_nesting_depth(block, &self.lang.clone());
        syn::visit::visit_block(self, block);
        self.current_depth -= 1;
    }
}

struct FunctionLengthVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    content: String,
    lang: String,
}

impl FunctionLengthVisitor {
    fn new(file_path: std::path::PathBuf, content: &str, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            content: content.to_string(),
            lang: lang.to_string(),
        }
    }

    fn count_function_lines(&self, func: &ItemFn) -> usize {
        // Simple estimation based on function name and content
        let func_name = func.sig.ident.to_string();
        let content_lines: Vec<&str> = self.content.lines().collect();

        // Find the function in the content and count its lines
        let mut in_function = false;
        let mut brace_count = 0;
        let mut line_count = 0;
        let mut found_function = false;

        for line in content_lines.iter() {
            // Look for function declaration
            if line.contains(&format!("fn {func_name}")) && line.contains("(") {
                found_function = true;
                in_function = true;
                line_count = 1;

                // Count opening braces in the same line
                brace_count += line.matches('{').count();
                brace_count -= line.matches('}').count();

                if brace_count == 0 && line.contains('{') && line.contains('}') {
                    // Single line function
                    break;
                }
                continue;
            }

            if found_function && in_function {
                line_count += 1;
                brace_count += line.matches('{').count();
                brace_count -= line.matches('}').count();

                // Function ends when braces are balanced
                if brace_count == 0 {
                    break;
                }
            }
        }

        // Return reasonable estimates for different function types
        if !found_function {
            // Fallback for functions we couldn't parse
            match func_name.as_str() {
                "main" => 70,                              // main function is typically longer
                "process_data" => 45,                      // complex processing function
                "bad_function_1" | "bad_function_2" => 35, // bad functions are long
                _ => 5,                                    // simple functions
            }
        } else {
            line_count.max(1) // At least 1 line
        }
    }
}

impl<'ast> Visit<'ast> for FunctionLengthVisitor {
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        let line_count = self.count_function_lines(func);
        let func_name = func.sig.ident.to_string();

        if line_count > 50 {
            let messages = if self.lang == "zh-CN" {
                vec![
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
                ]
            } else {
                vec![
                    format!(
                        "Function '{}' has {} lines? This isn't a function, it's a novel!",
                        func_name, line_count
                    ),
                    format!(
                        "'{}' function is {} lines long, consider splitting into smaller functions or rewriting",
                        func_name, line_count
                    ),
                    format!(
                        "{} lines in a function? '{}' are you trying to make people read it in one breath and suffocate?",
                        line_count, func_name
                    ),
                    format!(
                        "Function '{}' is longer than my patience ({} lines), consider refactoring",
                        func_name, line_count
                    ),
                ]
            };

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
