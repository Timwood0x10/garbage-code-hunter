use std::path::Path;
use syn::{visit::Visit, ExprMethodCall, File};

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;

pub struct UnwrapAbuseRule;

impl Rule for UnwrapAbuseRule {
    fn name(&self) -> &'static str {
        "unwrap-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = UnwrapVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

pub struct UnnecessaryCloneRule;

impl Rule for UnnecessaryCloneRule {
    fn name(&self) -> &'static str {
        "unnecessary-clone"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = CloneVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct UnwrapVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    unwrap_count: usize,
    lang: String,
}

impl UnwrapVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            unwrap_count: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for UnwrapVisitor {
    fn visit_expr_method_call(&mut self, method_call: &'ast ExprMethodCall) {
        if method_call.method == "unwrap" {
            self.unwrap_count += 1;

            let messages = if self.lang == "zh-CN" {
                [
                    "又一个 unwrap()！你是想让程序在生产环境里爆炸吗？",
                    "unwrap() 大师！错误处理是什么？能吃吗？",
                    "看到这个 unwrap()，我仿佛听到了程序崩溃的声音",
                    "unwrap() 使用者，恭喜你获得了'程序炸弹制造专家'称号",
                    "这个 unwrap() 就像定时炸弹，不知道什么时候会爆",
                    "又见 unwrap()！建议使用 match 或 if let，除非你喜欢 panic",
                ]
            } else {
                [
                    "Another unwrap()! Trying to blow up production?",
                    "unwrap() master! What's error handling? Can you eat it?",
                    "Seeing this unwrap(), I can already hear the crash",
                    "unwrap() user, congrats on the 'bomb-making expert' certificate",
                    "This unwrap() is a ticking time bomb - no idea when it'll explode",
                    "unwrap() again! Use match or if let, unless you enjoy panicking",
                ]
            };

            let severity = if self.unwrap_count > 5 {
                Severity::Nuclear
            } else if self.unwrap_count > 2 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1, // Simplified handling
                column: 1,
                rule_name: "unwrap-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity,
            });
        }

        syn::visit::visit_expr_method_call(self, method_call);
    }
}

struct CloneVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    clone_count: usize,
    lang: String,
}

impl CloneVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            clone_count: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for CloneVisitor {
    fn visit_expr_method_call(&mut self, method_call: &'ast ExprMethodCall) {
        if method_call.method == "clone" {
            self.clone_count += 1;

            // Simple heuristic detection: if there are multiple clones on the same line or nearby, they might be unnecessary
            if self.clone_count > 3 {
                let messages = if self.lang == "zh-CN" {
                    [
                        "clone() 狂魔！你是想把内存用完吗？",
                        "这么多 clone()，你确定不是在写 Java？",
                        "clone() 使用过度！Rust 的借用检查器在哭泣",
                        "又见 clone()！也许你需要重新学习一下 Rust 的所有权系统",
                        "这些 clone() 让我想起了复印机店的老板",
                    ]
                } else {
                    [
                        "clone() fanatic! Trying to use up all the memory?",
                        "This many clone()s - are you sure you're not writing Java?",
                        "clone() abuse! Rust's borrow checker is crying",
                        "clone() again! Maybe you need to relearn Rust's ownership system",
                        "These clone()s remind me of a copy shop owner",
                    ]
                };

                self.issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line: 1, // Simplified handling
                    column: 1,
                    rule_name: "unnecessary-clone".to_string(),
                    message: messages[self.issues.len() % messages.len()].to_string(),
                    severity: Severity::Spicy,
                });
            }
        }

        syn::visit::visit_expr_method_call(self, method_call);
    }
}
