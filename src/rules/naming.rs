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

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = NamingVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

pub struct SingleLetterVariableRule;

impl Rule for SingleLetterVariableRule {
    fn name(&self) -> &'static str {
        "single-letter-variable"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = SingleLetterVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct NamingVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    terrible_names: Regex,
    lang: String,
}

impl NamingVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        let terrible_names = Regex::new(r"^(data|info|temp|tmp|val|value|item|thing|stuff|obj|object|manager|handler|helper|util|utils|a|b|c|d|e|f|g|h|test|func|function)(\d+)?$").unwrap();

        Self {
            file_path,
            issues: Vec::new(),
            terrible_names,
            lang: lang.to_string(),
        }
    }

    fn check_name(&mut self, ident: &Ident, context: &str) {
        let name = ident.to_string();

        if self.terrible_names.is_match(&name.to_lowercase()) {
            // 根据语言设置选择消息
            let messages = if self.lang == "zh-CN" {
                // 中文消息
                let ctx = if context == "函数名" {
                    "函数名"
                } else {
                    "变量名"
                };
                vec![
                    format!("{} '{}' - 比我的编程技能还要抽象", ctx, name),
                    format!(
                        "{} '{}' - 这个名字告诉我你已经放弃治疗了，建议直接转行卖煎饼果子",
                        ctx, name
                    ),
                    format!(
                        "{} '{}' - 用这个做名字？你是想让维护代码的人哭着辞职吗？",
                        ctx, name
                    ),
                    format!("{} '{}' - 恭喜你发明了最没有意义的标识符", ctx, name),
                    format!("{} '{}' - 创意程度约等于给孩子起名叫'小明'", ctx, name),
                    format!(
                        "{} '{}' - 看到这个名字，我的智商都下降了，现在只能数到3了",
                        ctx, name
                    ),
                ]
            } else {
                // 英文消息
                let ctx = if context == "Function" {
                    "Function"
                } else {
                    "Variable"
                };
                vec![
                    format!("{} '{}' - more abstract than my programming skills", ctx, name),
                    format!("{} '{}' - this name tells me you've given up on life and should sell hotdogs", ctx, name),
                    format!("{} '{}' - using this name? trying to make maintainers cry and quit?", ctx, name),
                    format!("{} '{}' - congrats on inventing the most meaningless identifier", ctx, name),
                    format!("{} '{}' - creativity level of naming a kid 'Child'", ctx, name),
                    format!("{} '{}' - seeing this name, my IQ dropped to single digits", ctx, name),
                ]
            };

            let message_index =
                (self.issues.len() + name.len() + name.chars().next().unwrap_or('a') as usize)
                    % messages.len();

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: self.issues.len() + 2, // 简单的行号估算，避免都是1:1
                column: (name.len() % 10) + 1, // 基于名字长度的列号
                rule_name: "terrible-naming".to_string(),
                message: messages[message_index].clone(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }
    }
}

impl<'ast> Visit<'ast> for NamingVisitor {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        let context = if self.lang == "zh-CN" {
            "变量名"
        } else {
            "Variable"
        };
        self.check_name(ident, context);
        syn::visit::visit_ident(self, ident);
    }

    // 添加函数名检测
    fn visit_item_fn(&mut self, func: &'ast syn::ItemFn) {
        let context = if self.lang == "zh-CN" {
            "函数名"
        } else {
            "Function"
        };
        self.check_name(&func.sig.ident, context);
        syn::visit::visit_item_fn(self, func);
    }
}

struct SingleLetterVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl SingleLetterVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
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
                format!("变量 '{}'？这是变量名还是你键盘坏了？", name),
                format!(
                    "用 '{}' 做变量名，你可能需要一本《如何给变量起名》的书",
                    name
                ),
                format!("单字母变量 '{}'：让代码比古埃及象形文字还难懂", name),
                format!("变量 '{}' 的信息量约等于一个句号", name),
            ];

            let message_index =
                (self.issues.len() + name.len() + name.chars().next().unwrap_or('a') as usize)
                    % messages.len();

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: self.issues.len() + 10, // 不同的行号范围
                column: (name.len() % 5) + 1,
                rule_name: "single-letter-variable".to_string(),
                message: messages[message_index].clone(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
            });
        }

        syn::visit::visit_pat_ident(self, pat_ident);
    }
}
