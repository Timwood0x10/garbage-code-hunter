use std::path::Path;
use syn::{visit::Visit, ExprLit, File, ItemFn, Lit};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;
use crate::utils::get_position;

/// 检测魔法数字（硬编码的数字常量）
pub struct MagicNumberRule;

impl Rule for MagicNumberRule {
    fn name(&self) -> &'static str {
        "magic-number"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = MagicNumberVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// 检测做太多事的函数（上帝函数）
pub struct GodFunctionRule;

impl Rule for GodFunctionRule {
    fn name(&self) -> &'static str {
        "god-function"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = GodFunctionVisitor::new(file_path.to_path_buf(), content, lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// 检测被注释掉的代码块
pub struct CommentedCodeRule;

impl Rule for CommentedCodeRule {
    fn name(&self) -> &'static str {
        "commented-code"
    }

    fn check(
        &self,
        file_path: &Path,
        _syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut _commented_code_blocks = 0;
        let mut current_block_size = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // 检测被注释的代码行
            if trimmed.starts_with("//") {
                let comment_content = trimmed.trim_start_matches("//").trim();

                // 检测是否像代码（包含常见的代码模式）
                if is_likely_code(comment_content) {
                    current_block_size += 1;
                } else if current_block_size > 0 {
                    // 结束一个代码块
                    if current_block_size >= 3 {
                        _commented_code_blocks += 1;
                        issues.push(create_commented_code_issue(
                            file_path,
                            line_num + 1 - current_block_size,
                            current_block_size,
                            lang,
                        ));
                    }
                    current_block_size = 0;
                }
            } else if current_block_size > 0 {
                // 非注释行，结束当前块
                if current_block_size >= 3 {
                    _commented_code_blocks += 1;
                    issues.push(create_commented_code_issue(
                        file_path,
                        line_num - current_block_size,
                        current_block_size,
                        lang,
                    ));
                }
                current_block_size = 0;
            }
        }

        // 处理文件末尾的代码块
        if current_block_size >= 3 {
            issues.push(create_commented_code_issue(
                file_path,
                lines.len() - current_block_size,
                current_block_size,
                lang,
            ));
        }

        issues
    }
}

/// 检测明显的死代码
pub struct DeadCodeRule;

impl Rule for DeadCodeRule {
    fn name(&self) -> &'static str {
        "dead-code"
    }

    fn check(
        &self,
        file_path: &Path,
        _syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // 检测明显的死代码模式
            if is_dead_code_pattern(trimmed) {
                let messages = if lang == "zh-CN" {
                    vec![
                        "发现死代码，这行永远不会执行",
                        "这行代码比我的社交生活还死",
                        "死代码警告：这里是代码的坟墓",
                        "这行代码已经去世了，建议删除",
                        "发现僵尸代码，需要清理",
                    ]
                } else {
                    vec![
                        "Dead code detected - this line will never execute",
                        "This code is deader than my social life",
                        "Dead code alert: code graveyard found here",
                        "This line of code has passed away, consider removal",
                        "Zombie code detected, cleanup needed",
                    ]
                };

                issues.push(CodeIssue {
                    file_path: file_path.to_path_buf(),
                    line: line_num + 1,
                    column: 1,
                    rule_name: "dead-code".to_string(),
                    message: messages[line_num % messages.len()].to_string(),
                    severity: Severity::Mild,
                    roast_level: RoastLevel::Sarcastic,
                });
            }
        }

        issues
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

fn is_likely_code(content: &str) -> bool {
    // 检测是否像代码的模式
    let code_patterns = [
        "let ", "fn ", "if ", "else", "for ", "while ", "match ", "struct ", "enum ", "impl ",
        "use ", "mod ", "return ", "break", "continue", "{", "}", "(", ")", "[", "]", ";", "=",
        "==", "!=", "&&", "||", "->", "::",
    ];

    let rust_keywords = [
        "pub", "const", "static", "mut", "ref", "move", "async", "await", "unsafe", "extern",
        "crate",
    ];

    // 如果包含多个代码模式，很可能是代码
    let pattern_count = code_patterns
        .iter()
        .filter(|&&pattern| content.contains(pattern))
        .count();

    let keyword_count = rust_keywords
        .iter()
        .filter(|&&keyword| content.contains(keyword))
        .count();

    pattern_count >= 2 || keyword_count >= 1
}

fn create_commented_code_issue(
    file_path: &Path,
    line: usize,
    block_size: usize,
    lang: &str,
) -> CodeIssue {
    let messages = if lang == "zh-CN" {
        vec![
            format!("发现 {} 行被注释的代码，是舍不得删除吗？", block_size),
            format!("{} 行注释代码，版本控制系统不香吗？", block_size),
            format!("这 {} 行注释代码就像前任，该放手就放手", block_size),
            format!("{} 行死代码注释，建议断舍离", block_size),
            format!("注释了 {} 行代码，Git 会记住它们的", block_size),
        ]
    } else {
        vec![
            format!(
                "Found {} lines of commented code - can't let go?",
                block_size
            ),
            format!(
                "{} lines of commented code - isn't version control enough?",
                block_size
            ),
            format!(
                "These {} commented lines are like an ex - time to let go",
                block_size
            ),
            format!(
                "{} lines of dead commented code - Marie Kondo would disapprove",
                block_size
            ),
            format!(
                "Commented {} lines of code - Git remembers them anyway",
                block_size
            ),
        ]
    };

    let severity = if block_size > 10 {
        Severity::Spicy
    } else {
        Severity::Mild
    };

    CodeIssue {
        file_path: file_path.to_path_buf(),
        line,
        column: 1,
        rule_name: "commented-code".to_string(),
        message: messages[block_size % messages.len()].clone(),
        severity,
        roast_level: RoastLevel::Sarcastic,
    }
}

fn is_dead_code_pattern(line: &str) -> bool {
    // 检测明显的死代码模式
    let dead_patterns = [
        "return;",
        "return ", // return 后的代码
        "break;",
        "continue;", // break/continue 后的代码
        "panic!(",
        "unreachable!(", // panic 后的代码
        "std::process::exit(",
    ];

    dead_patterns.iter().any(|&pattern| line.contains(pattern))
}

// ============================================================================
// Visitor 实现
// ============================================================================

struct MagicNumberVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl MagicNumberVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn is_magic_number(&self, value: i64) -> bool {
        // 常见的非魔法数字
        !matches!(value, -1 | 0 | 1 | 2 | 10 | 100 | 1000)
    }

    fn create_magic_number_issue(&self, value: i64, line: usize, column: usize) -> CodeIssue {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("魔法数字 {}？这是什么咒语？", value),
                format!("硬编码数字 {}，维护性-1", value),
                format!("数字 {} 从天而降，没人知道它的含义", value),
                format!("魔法数字 {}，建议定义为常量", value),
                format!("看到数字 {}，我陷入了沉思", value),
            ]
        } else {
            vec![
                format!("Magic number {}? What spell is this?", value),
                format!("Hardcoded number {} - maintainability -1", value),
                format!(
                    "Number {} fell from the sky, nobody knows its meaning",
                    value
                ),
                format!("Magic number {} - consider defining as a constant", value),
                format!("Seeing number {}, I'm lost in thought", value),
            ]
        };

        let severity = if !(-100..=1000).contains(&value) {
            Severity::Spicy
        } else {
            Severity::Mild
        };

        CodeIssue {
            file_path: self.file_path.clone(),
            line,
            column,
            rule_name: "magic-number".to_string(),
            message: messages[self.issues.len() % messages.len()].clone(),
            severity,
            roast_level: RoastLevel::Gentle,
        }
    }
}

impl<'ast> Visit<'ast> for MagicNumberVisitor {
    fn visit_expr_lit(&mut self, expr_lit: &'ast ExprLit) {
        if let Lit::Int(lit_int) = &expr_lit.lit {
            if let Ok(value) = lit_int.base10_parse::<i64>() {
                if self.is_magic_number(value) {
                    let (line, column) = get_position(expr_lit);
                    self.issues
                        .push(self.create_magic_number_issue(value, line, column));
                }
            }
        }
        syn::visit::visit_expr_lit(self, expr_lit);
    }
}

// ============================================================================
// 上帝函数检测
// ============================================================================

struct GodFunctionVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    _content: String,
    lang: String,
}

impl GodFunctionVisitor {
    fn new(file_path: std::path::PathBuf, content: &str, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            _content: content.to_string(),
            lang: lang.to_string(),
        }
    }

    fn analyze_function_complexity(&mut self, func: &ItemFn) {
        let func_name = func.sig.ident.to_string();

        // 计算函数的各种复杂度指标
        let mut complexity_score = 0;

        // 1. 参数数量
        let param_count = func.sig.inputs.len();
        if param_count > 5 {
            complexity_score += (param_count - 5) * 2;
        }

        // 2. 函数体大小（通过字符串分析估算）
        let func_str = format!("{func:?}");
        let line_count = func_str.lines().count();
        if line_count > 50 {
            complexity_score += (line_count - 50) / 10;
        }

        // 3. 嵌套深度和控制流复杂度
        let control_keywords = ["if", "else", "for", "while", "match", "loop"];
        for keyword in &control_keywords {
            complexity_score += func_str.matches(keyword).count();
        }

        // 如果复杂度过高，报告问题
        if complexity_score > 15 {
            let messages = if self.lang == "zh-CN" {
                vec![
                    format!("函数 '{}' 做的事情比我一天做的还多", func_name),
                    format!("'{}' 是上帝函数吗？什么都想管", func_name),
                    format!("函数 '{}' 复杂得像我的感情生活", func_name),
                    format!("'{}' 这个函数需要拆分，太臃肿了", func_name),
                    format!("函数 '{}' 违反了单一职责原则", func_name),
                ]
            } else {
                vec![
                    format!(
                        "Function '{}' does more things than I do in a day",
                        func_name
                    ),
                    format!(
                        "Is '{}' a god function? Wants to control everything",
                        func_name
                    ),
                    format!("Function '{}' is as complex as my love life", func_name),
                    format!("Function '{}' needs to be split - too bloated", func_name),
                    format!(
                        "Function '{}' violates single responsibility principle",
                        func_name
                    ),
                ]
            };

            let severity = if complexity_score > 25 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            let (line, column) = get_position(func);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "god-function".to_string(),
                message: messages[self.issues.len() % messages.len()].clone(),
                severity,
                roast_level: RoastLevel::Sarcastic,
            });
        }
    }
}

impl<'ast> Visit<'ast> for GodFunctionVisitor {
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        self.analyze_function_complexity(func);
        syn::visit::visit_item_fn(self, func);
    }
}
