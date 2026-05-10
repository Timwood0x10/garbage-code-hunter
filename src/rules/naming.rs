use regex::Regex;
use std::path::Path;
use syn::{visit::Visit, File, Ident};

use crate::analyzer::{CodeIssue, Severity};
use crate::context::FileContext;
use crate::rules::Rule;
use crate::utils::get_position;

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
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }

        let mut visitor = NamingVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }

    fn check_with_context(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
        context: &FileContext,
        _config: &crate::context::ProjectConfig,
    ) -> Vec<CodeIssue> {
        // Example/Demo/Documentation: 完全跳过
        let weight = context.rule_weight_multiplier();
        if weight < 0.5 {
            return Vec::new();
        }

        self.check(file_path, syntax_tree, content, lang, is_test_file)
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
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }

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
        // Note: 'item' is excluded as it's a common Rust iterator variable
        let terrible_names = Regex::new(r"^(data|info|temp|tmp|val|value|thing|stuff|obj|object|manager|handler|helper|util|utils)(\d+)?$").unwrap();

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
            // Select messages based on language setting
            let messages = if self.lang == "zh-CN" {
                // Chinese messages
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
                // English messages
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

            let (line, column) = get_position(ident);

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "terrible-naming".to_string(),
                message: messages[message_index].clone(),
                severity: Severity::Spicy,
            });
        }
    }
}

impl<'ast> Visit<'ast> for NamingVisitor {
    // Check variable names (pattern identifiers)
    fn visit_pat_ident(&mut self, pat_ident: &'ast syn::PatIdent) {
        let context = if self.lang == "zh-CN" {
            "变量名"
        } else {
            "Variable"
        };
        self.check_name(&pat_ident.ident, context);
        syn::visit::visit_pat_ident(self, pat_ident);
    }

    // Check function names
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

        // Exclude common single-letter variables (loop counters, closure params, etc.)
        if name.len() == 1
            && !matches!(
                name.as_str(),
                "i" | "j"
                    | "k"
                    | "x"
                    | "y"
                    | "z"
                    | "e"
                    | "a"
                    | "b"
                    | "c"
                    | "d"
                    | "f"
                    | "n"
                    | "r"
                    | "s"
                    | "t"
                    | "v"
                    | "w"
                    | "p"
                    | "l"
            )
        {
            let messages = if self.lang == "zh-CN" {
                [
                    format!("单字母变量 '{name}'？你是在写数学公式还是在折磨读代码的人？"),
                    format!("变量 '{name}'？这是变量名还是你键盘坏了？"),
                    format!("用 '{name}' 做变量名，你可能需要一本《如何给变量起名》的书"),
                    format!("单字母变量 '{name}'：让代码比古埃及象形文字还难懂"),
                    format!("变量 '{name}' 的信息量约等于一个句号"),
                ]
            } else {
                [
                    format!("Single-letter variable '{name}'? Writing math formulas or torturing readers?"),
                    format!("Variable '{name}'? Is this a name or did your keyboard break?"),
                    format!("Using '{name}' as a variable name? You need a book on naming"),
                    format!("Single-letter variable '{name}': harder to read than hieroglyphics"),
                    format!("Variable '{name}' has about as much info as a period"),
                ]
            };

            let message_index =
                (self.issues.len() + name.len() + name.chars().next().unwrap_or('a') as usize)
                    % messages.len();

            let (line, column) = get_position(&pat_ident.ident);

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "single-letter-variable".to_string(),
                message: messages[message_index].clone(),
                severity: Severity::Mild,
            });
        }

        syn::visit::visit_pat_ident(self, pat_ident);
    }
}
