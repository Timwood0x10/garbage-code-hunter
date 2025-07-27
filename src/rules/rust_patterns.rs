use std::path::Path;
use syn::{
    visit::Visit, Expr, ExprForLoop, ExprMatch, ExprMethodCall, File, Pat, PatIdent, Type, TypePath,
};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;
use crate::utils::get_position;

/// 检测到处用 String 而不用 &str
pub struct StringAbuseRule;

impl Rule for StringAbuseRule {
    fn name(&self) -> &'static str {
        "string-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = StringAbuseVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);

        // 检查内容中的 String 使用模式
        let string_new_count = content.matches("String::new()").count();
        let string_from_count = content.matches("String::from(").count();
        let to_string_count = content.matches(".to_string()").count();

        if string_new_count + string_from_count + to_string_count > 5 {
            visitor.add_excessive_string_conversion_issue(
                string_new_count + string_from_count + to_string_count,
            );
        }

        visitor.issues
    }
}

/// 检测不必要的 Vec 分配
pub struct VecAbuseRule;

impl Rule for VecAbuseRule {
    fn name(&self) -> &'static str {
        "vec-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = VecAbuseVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);

        // 检查内容中的 Vec 使用模式
        let vec_new_count = content.matches("Vec::new()").count();
        let _vec_with_capacity_count = content.matches("Vec::with_capacity(").count();
        let _vec_macro_count = content.matches("vec![").count();

        if vec_new_count > 3 {
            visitor.add_excessive_vec_allocation_issue(vec_new_count);
        }

        visitor.issues
    }
}

/// 检测用循环代替迭代器的情况
pub struct IteratorAbuseRule;

impl Rule for IteratorAbuseRule {
    fn name(&self) -> &'static str {
        "iterator-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = IteratorAbuseVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// 检测可以用 if let 的复杂 match
pub struct MatchAbuseRule;

impl Rule for MatchAbuseRule {
    fn name(&self) -> &'static str {
        "match-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = MatchAbuseVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

// ============================================================================
// Visitor 实现
// ============================================================================

struct StringAbuseVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl StringAbuseVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn add_excessive_string_conversion_issue(&mut self, count: usize) {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("{} 次 String 转换？你是在开字符串工厂吗？", count),
                format!("这么多 String 分配，内存都要哭了",),
                format!("{} 个 String 转换，考虑用 &str 吧", count),
                format!("String 用得比我换衣服还频繁",),
                format!("到处都是 String::from()，性能在哭泣",),
            ]
        } else {
            vec![
                format!(
                    "{} String conversions? Are you running a string factory?",
                    count
                ),
                format!("So many String allocations, memory is crying"),
                format!("{} String conversions - consider using &str", count),
                format!("You use String more than I change clothes"),
                format!("String::from() everywhere - performance is weeping"),
            ]
        };

        self.issues.push(CodeIssue {
            file_path: self.file_path.clone(),
            line: 1,
            column: 1,
            rule_name: "string-abuse".to_string(),
            message: messages[count % messages.len()].clone(),
            severity: Severity::Spicy,
            roast_level: RoastLevel::Sarcastic,
        });
    }

    fn check_string_parameter(&mut self, ty: &Type) {
        if let Type::Path(TypePath { path, .. }) = ty {
            if let Some(segment) = path.segments.last() {
                if segment.ident == "String" {
                    let messages = if self.lang == "zh-CN" {
                        vec![
                            "参数用 String？考虑用 &str 吧",
                            "String 参数会强制调用者分配内存",
                            "这个 String 参数设计得不太优雅",
                            "用 &str 参数会更灵活",
                        ]
                    } else {
                        vec![
                            "String parameter? Consider using &str",
                            "String parameter forces caller to allocate",
                            "This String parameter design isn't elegant",
                            "&str parameter would be more flexible",
                        ]
                    };

                    let (line, column) = get_position(ty);
                    self.issues.push(CodeIssue {
                        file_path: self.file_path.clone(),
                        line,
                        column,
                        rule_name: "string-abuse".to_string(),
                        message: messages[self.issues.len() % messages.len()].to_string(),
                        severity: Severity::Mild,
                        roast_level: RoastLevel::Gentle,
                    });
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for StringAbuseVisitor {
    fn visit_expr_method_call(&mut self, method_call: &'ast ExprMethodCall) {
        if method_call.method == "to_string" {
            let messages = if self.lang == "zh-CN" {
                vec![
                    "又见 to_string()，真的需要分配内存吗？",
                    "to_string() 调用，考虑是否必要",
                    "这个 to_string() 可能可以避免",
                    "to_string() 会分配内存，确定需要吗？",
                ]
            } else {
                vec![
                    "Another to_string() - do you really need to allocate?",
                    "to_string() call - consider if necessary",
                    "This to_string() might be avoidable",
                    "to_string() allocates memory - sure you need it?",
                ]
            };

            let (line, column) = get_position(method_call);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "string-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
            });
        }
        syn::visit::visit_expr_method_call(self, method_call);
    }

    fn visit_type(&mut self, ty: &'ast Type) {
        self.check_string_parameter(ty);
        syn::visit::visit_type(self, ty);
    }
}

// ============================================================================
// Vec 滥用检测
// ============================================================================

struct VecAbuseVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl VecAbuseVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn add_excessive_vec_allocation_issue(&mut self, count: usize) {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("{} 个 Vec::new()？你在收集什么？", count),
                format!("这么多 Vec 分配，考虑用数组或切片",),
                format!("{} 次 Vec 分配，内存分配器很忙", count),
                format!("Vec 用得这么多，确定都需要动态数组吗？",),
            ]
        } else {
            vec![
                format!("{} Vec::new()s? What are you collecting?", count),
                format!("So many Vec allocations - consider arrays or slices"),
                format!("{} Vec allocations - memory allocator is busy", count),
                format!("So many Vecs - sure you need all these dynamic arrays?"),
            ]
        };

        self.issues.push(CodeIssue {
            file_path: self.file_path.clone(),
            line: 1,
            column: 1,
            rule_name: "vec-abuse".to_string(),
            message: messages[count % messages.len()].clone(),
            severity: Severity::Mild,
            roast_level: RoastLevel::Gentle,
        });
    }
}

impl<'ast> Visit<'ast> for VecAbuseVisitor {
    fn visit_expr_method_call(&mut self, method_call: &'ast ExprMethodCall) {
        // 检测 Vec::new() 调用
        if let Expr::Path(path_expr) = &*method_call.receiver {
            if let Some(segment) = path_expr.path.segments.last() {
                if segment.ident == "Vec" && method_call.method == "new" {
                    let messages = if self.lang == "zh-CN" {
                        vec![
                            "Vec::new() 出现，确定需要动态数组吗？",
                            "又一个 Vec::new()，考虑用数组",
                            "Vec::new() 会分配堆内存",
                            "这个 Vec::new() 可能可以优化",
                        ]
                    } else {
                        vec![
                            "Vec::new() spotted - sure you need a dynamic array?",
                            "Another Vec::new() - consider using an array",
                            "Vec::new() allocates heap memory",
                            "This Vec::new() might be optimizable",
                        ]
                    };

                    let (line, column) = get_position(method_call);
                    self.issues.push(CodeIssue {
                        file_path: self.file_path.clone(),
                        line,
                        column,
                        rule_name: "vec-abuse".to_string(),
                        message: messages[self.issues.len() % messages.len()].to_string(),
                        severity: Severity::Mild,
                        roast_level: RoastLevel::Gentle,
                    });
                }
            }
        }
        syn::visit::visit_expr_method_call(self, method_call);
    }
}

// ============================================================================
// 迭代器滥用检测
// ============================================================================

struct IteratorAbuseVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl IteratorAbuseVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn check_simple_for_loop(&mut self, for_loop: &ExprForLoop) {
        // 检测简单的 for 循环，可能可以用迭代器替代
        if let Pat::Ident(PatIdent { ident, .. }) = for_loop.pat.as_ref() {
            let _var_name = ident.to_string();

            // 检测常见的可以用迭代器替代的模式
            let loop_body = format!("{:?}", for_loop.body);

            // 如果循环体很简单，建议用迭代器
            if loop_body.lines().count() < 5 {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        format!("简单的 for 循环，考虑用迭代器链"),
                        format!("这个循环可以用 .iter().for_each() 替代"),
                        format!("迭代器比传统循环更 Rust 风格"),
                        format!("考虑用函数式编程风格重写这个循环"),
                    ]
                } else {
                    vec![
                        format!("Simple for loop - consider using iterator chains"),
                        format!("This loop could use .iter().for_each() instead"),
                        format!("Iterators are more idiomatic than traditional loops"),
                        format!("Consider rewriting this loop in functional style"),
                    ]
                };

                let (line, column) = get_position(for_loop);
                self.issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line,
                    column,
                    rule_name: "iterator-abuse".to_string(),
                    message: messages[self.issues.len() % messages.len()].to_string(),
                    severity: Severity::Mild,
                    roast_level: RoastLevel::Gentle,
                });
            }
        }
    }
}

impl<'ast> Visit<'ast> for IteratorAbuseVisitor {
    fn visit_expr_for_loop(&mut self, for_loop: &'ast ExprForLoop) {
        self.check_simple_for_loop(for_loop);
        syn::visit::visit_expr_for_loop(self, for_loop);
    }
}

// ============================================================================
// Match 滥用检测
// ============================================================================

struct MatchAbuseVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl MatchAbuseVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn check_simple_match(&mut self, match_expr: &ExprMatch) {
        let arms_count = match_expr.arms.len();

        // 检测只有两个分支的 match，可能可以用 if let 替代
        if arms_count == 2 {
            let match_str = format!("{match_expr:?}");

            // 检测 Option 或 Result 的简单匹配
            if match_str.contains("Some") && match_str.contains("None") {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        "简单的 Option match，考虑用 if let",
                        "这个 match 可以用 if let Some() 替代",
                        "Option 的二分支 match 不如 if let 简洁",
                        "if let 比 match 更适合这种情况",
                    ]
                } else {
                    vec![
                        "Simple Option match - consider using if let",
                        "This match could use if let Some() instead",
                        "Two-arm Option match is less concise than if let",
                        "if let is more suitable for this case than match",
                    ]
                };

                let (line, column) = get_position(match_expr);
                self.issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line,
                    column,
                    rule_name: "match-abuse".to_string(),
                    message: messages[self.issues.len() % messages.len()].to_string(),
                    severity: Severity::Mild,
                    roast_level: RoastLevel::Gentle,
                });
            }

            if match_str.contains("Ok") && match_str.contains("Err") {
                let messages = if self.lang == "zh-CN" {
                    vec![
                        "简单的 Result match，考虑用 if let",
                        "这个 match 可以用 if let Ok() 替代",
                        "Result 的二分支 match 可以简化",
                        "或者考虑用 ? 操作符",
                    ]
                } else {
                    vec![
                        "Simple Result match - consider using if let",
                        "This match could use if let Ok() instead",
                        "Two-arm Result match can be simplified",
                        "Or consider using the ? operator",
                    ]
                };

                let (line, column) = get_position(match_expr);
                self.issues.push(CodeIssue {
                    file_path: self.file_path.clone(),
                    line,
                    column,
                    rule_name: "match-abuse".to_string(),
                    message: messages[self.issues.len() % messages.len()].to_string(),
                    severity: Severity::Mild,
                    roast_level: RoastLevel::Gentle,
                });
            }
        }
    }
}

impl<'ast> Visit<'ast> for MatchAbuseVisitor {
    fn visit_expr_match(&mut self, match_expr: &'ast ExprMatch) {
        self.check_simple_match(match_expr);
        syn::visit::visit_expr_match(self, match_expr);
    }
}
