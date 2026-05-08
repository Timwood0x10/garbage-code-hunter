use std::path::Path;
use syn::{visit::Visit, ExprMacro, File};

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;
use crate::utils::get_position;

/// Detect println! debugging statements everywhere
pub struct PrintlnDebuggingRule;

impl Rule for PrintlnDebuggingRule {
    fn name(&self) -> &'static str {
        "println-debugging"
    }

    fn check(
        &self,
        file_path: &Path,
        _syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }

        let mut issues = Vec::new();

        // Check if this is a main.rs or lib.rs file (CLI tools legitimately use println!)
        let file_name = file_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");
        let is_main_file = file_name == "main.rs" || file_name == "lib.rs";

        // Count println! calls
        let println_count = content.matches("println!").count();

        // Only flag if there are many println! calls (likely debugging)
        // or if it's not a main/lib file (where println! is usually for output)
        if !is_main_file && println_count > 0 {
            // Flag println! in non-main files as potential debugging
            let messages = if lang == "zh-CN" {
                vec![
                    format!("发现 {} 个 println!，这是调试代码吗？", println_count),
                    format!("{} 个 println! 可能是调试残留", println_count),
                    format!("println! 出现在非主文件中，考虑使用 logging",),
                ]
            } else {
                vec![
                    format!("Found {} println! statements - is this debugging code?", println_count),
                    format!("{} println!s might be debug leftovers", println_count),
                    format!("println! in non-main file - consider using logging",),
                ]
            };

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line: 1,
                column: 1,
                rule_name: "println-debugging".to_string(),
                message: messages[println_count % messages.len()].clone(),
                severity: Severity::Mild,
            });
        } else if is_main_file && println_count > 10 {
            // Flag excessive println! even in main files
            let messages = if lang == "zh-CN" {
                vec![
                    format!("{} 个 println!？你是在开演唱会吗？", println_count),
                    format!("这么多 println!，控制台都要被刷屏了",),
                    format!("{} 个打印语句，建议学学 debugger 的使用", println_count),
                ]
            } else {
                vec![
                    format!("{} println! statements? Are you hosting a concert?", println_count),
                    format!("So many println!s, the console is crying"),
                    format!("{} print statements - time to learn about debuggers", println_count),
                ]
            };

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line: 1,
                column: 1,
                rule_name: "println-debugging".to_string(),
                message: messages[println_count % messages.len()].clone(),
                severity: Severity::Spicy,
            });
        }

        issues
    }
}

/// Detect casual use of panic! and unwrap()
pub struct PanicAbuseRule;

impl Rule for PanicAbuseRule {
    fn name(&self) -> &'static str {
        "panic-abuse"
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
        let mut visitor = PanicAbuseVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);

        // Check panic! and unwrap usage in content
        let panic_count = content.matches("panic!").count();
        let unwrap_count = content.matches(".unwrap()").count();

        if panic_count > 2 {
            visitor.add_excessive_panic_issue(panic_count);
        }
        if unwrap_count > 3 {
            visitor.add_excessive_unwrap_issue(unwrap_count);
        }

        visitor.issues
    }
}

/// Detect excessive TODO comments
pub struct TodoCommentRule;

impl Rule for TodoCommentRule {
    fn name(&self) -> &'static str {
        "todo-comment"
    }

    fn check(
        &self,
        file_path: &Path,
        _syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }
        let mut issues = Vec::new();

        // Check various TODO patterns
        let todo_patterns = [
            "TODO",
            "FIXME",
            "XXX",
            "HACK",
            "BUG",
            "NOTE",
            "todo!",
            "unimplemented!",
            "unreachable!",
        ];

        let mut total_todos = 0;
        for pattern in &todo_patterns {
            total_todos += content.matches(pattern).count();
        }

        if total_todos > 5 {
            let messages = if lang == "zh-CN" {
                vec![
                    format!("发现 {} 个 TODO/FIXME，这是代码还是购物清单？", total_todos),
                    format!("{} 个未完成项目？你是在写代码还是在记日记？", total_todos),
                    format!("TODO 比实际代码还多，建议改名叫 'TODO Hunter'",),
                    format!("{} 个 TODO，看来这个项目还在'施工中'", total_todos),
                    format!("这么多 TODO，是不是该考虑换个职业了？",),
                ]
            } else {
                vec![
                    format!(
                        "Found {} TODOs/FIXMEs - is this code or a shopping list?",
                        total_todos
                    ),
                    format!(
                        "{} unfinished items? Are you coding or journaling?",
                        total_todos
                    ),
                    format!("More TODOs than actual code, consider renaming to 'TODO Hunter'"),
                    format!(
                        "{} TODOs - looks like this project is still 'under construction'",
                        total_todos
                    ),
                    format!("So many TODOs, maybe consider a career change?"),
                ]
            };

            let severity = if total_todos > 10 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line: 1,
                column: 1,
                rule_name: "todo-comment".to_string(),
                message: messages[total_todos % messages.len()].clone(),
                severity,
            });
        }

        issues
    }
}

// ============================================================================
// Panic abuse detection
// ============================================================================

struct PanicAbuseVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl PanicAbuseVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn add_excessive_panic_issue(&mut self, count: usize) {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("{} 个 panic!？你的程序是定时炸弹吗？", count),
                format!("这么多 panic!，用户体验堪忧",),
                format!("{} 个 panic!，建议学学错误处理", count),
                format!("panic! 用得这么随意，Rust 编译器都要哭了",),
            ]
        } else {
            vec![
                format!("{} panic!s? Is your program a time bomb?", count),
                format!("So many panic!s, user experience is questionable"),
                format!("{} panic!s - time to learn error handling", count),
                format!("Using panic! so casually, even Rust compiler is crying"),
            ]
        };

        self.issues.push(CodeIssue {
            file_path: self.file_path.clone(),
            line: 1,
            column: 1,
            rule_name: "panic-abuse".to_string(),
            message: messages[count % messages.len()].clone(),
            severity: Severity::Nuclear,
        });
    }

    fn add_excessive_unwrap_issue(&mut self, count: usize) {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("{} 个 unwrap()，你对代码很有信心啊", count),
                format!("unwrap() 用得这么多，建议买个保险",),
                format!("{} 个 unwrap()，错误处理呢？", count),
                format!("这么多 unwrap()，程序随时可能崩溃",),
            ]
        } else {
            vec![
                format!("{} unwrap()s - you're very confident in your code", count),
                format!("So many unwrap()s, consider buying insurance"),
                format!("{} unwrap()s - where's the error handling?", count),
                format!("So many unwrap()s, program might crash anytime"),
            ]
        };

        self.issues.push(CodeIssue {
            file_path: self.file_path.clone(),
            line: 1,
            column: 1,
            rule_name: "panic-abuse".to_string(),
            message: messages[count % messages.len()].clone(),
            severity: Severity::Spicy,
        });
    }
}

impl<'ast> Visit<'ast> for PanicAbuseVisitor {
    fn visit_expr_macro(&mut self, expr_macro: &'ast ExprMacro) {
        if let Some(ident) = expr_macro.mac.path.get_ident() {
            if ident == "panic" {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        "发现一个 panic!，程序要爆炸了",
                        "panic! 出现，用户体验-1",
                        "又见 panic!，优雅的错误处理在哪里？",
                        "panic! 大法好，但是用户不这么想",
                    ]
                } else {
                    vec![
                        "Found a panic! - program is about to explode",
                        "panic! spotted, user experience -1",
                        "Another panic! - where's the graceful error handling?",
                        "panic! is great, but users disagree",
                    ]
                };

                let (line, column) = get_position(expr_macro);
                self.issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line,
                    column,
                    rule_name: "panic-abuse".to_string(),
                    message: messages[self.issues.len() % messages.len()].to_string(),
                    severity: Severity::Spicy,
                });
            }
        }
        syn::visit::visit_expr_macro(self, expr_macro);
    }
}
