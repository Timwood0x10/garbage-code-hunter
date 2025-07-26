use regex::Regex;
use std::path::Path;
use syn::{visit::Visit, File, Ident};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;

pub struct TerribleNamingRule;

impl Rule for TerribleNamingRule {
    fn name(&self) -> &'static str {
        "terrible-naming"
    }

    fn check(&self, file_path: &Path, syntax_tree: &File, _content: &str) -> Vec<CodeIssue> {
        let mut visitor = NamingVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

pub struct SingleLetterVariableRule;

impl Rule for SingleLetterVariableRule {
    fn name(&self) -> &'static str {
        "single-letter-variable"
    }

    fn check(&self, file_path: &Path, syntax_tree: &File, _content: &str) -> Vec<CodeIssue> {
        let mut visitor = SingleLetterVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct NamingVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    terrible_names: Regex,
}

impl NamingVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        let terrible_names = Regex::new(r"^(data|info|temp|tmp|val|value|item|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").unwrap();

        Self {
            file_path,
            issues: Vec::new(),
            terrible_names,
        }
    }

    fn check_name(&mut self, ident: &Ident) {
        let name = ident.to_string();

        if self.terrible_names.is_match(&name.to_lowercase()) {
            let messages = vec![
                format!("变量名 '{}' 比我的编程技能还要抽象", name),
                format!("'{}' 这个名字告诉我你已经放弃治疗了", name),
                format!("用 '{}' 做变量名？你是想让维护代码的人哭吗？", name),
                format!("'{}' - 恭喜你发明了最没有意义的变量名", name),
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1, // Simplified handling, real projects could use proc-macro2's LineColumn
                column: 1,
                rule_name: "terrible-naming".to_string(),
                message: messages[self.issues.len() % messages.len()].clone(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }
    }
}

impl<'ast> Visit<'ast> for NamingVisitor {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.check_name(ident);
        syn::visit::visit_ident(self, ident);
    }
}

struct SingleLetterVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
}

impl SingleLetterVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for SingleLetterVisitor {
    fn visit_pat_ident(&mut self, pat_ident: &'ast syn::PatIdent) {
        let name = pat_ident.ident.to_string();

        // Exclude common single-letter variables (like loop counters i, j, k)
        if name.len() == 1 && !matches!(name.as_str(), "i" | "j" | "k" | "x" | "y" | "z") {
            let messages = vec![
                format!(
                    "单字母变量 '{}'？你是在写数学公式还是在折磨读代码的人？",
                    name
                ),
                format!("'{}'？这是变量名还是你键盘坏了？", name),
                format!(
                    "用 '{}' 做变量名，你可能需要一本《如何给变量起名》的书",
                    name
                ),
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1, // Simplified handling
                column: 1,
                rule_name: "single-letter-variable".to_string(),
                message: messages[self.issues.len() % messages.len()].clone(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
            });
        }

        syn::visit::visit_pat_ident(self, pat_ident);
    }
}
