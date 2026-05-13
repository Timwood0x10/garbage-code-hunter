use std::path::Path;
use syn::{visit::Visit, ExprLit, File, ItemFn, Lit};

use crate::analyzer::{CodeIssue, Severity};
use crate::context::FileContext;
use crate::rules::Rule;
use crate::utils::get_position;

// ============================================================================
// Magic number detection
// ============================================================================

/// Categories of magic numbers for better error messages
enum MagicNumberCategory {
    /// Timeout values in milliseconds (100, 500, 1000, etc.)
    Timeout,
    /// Buffer sizes (1024, 2048, 4096, etc.)
    BufferSize,
    /// Network port numbers (80, 443, 3000, 8080, etc.)
    PortNumber,
    /// Threshold/limit values (10, 50, 100, etc.)
    Threshold,
    /// Unclassified magic numbers
    General,
}

/// Detects hardcoded magic numbers that should be named constants.
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
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = MagicNumberVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }

    fn check_with_context(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        _is_test_file: bool,
        context: &FileContext,
        _config: &crate::context::ProjectConfig,
    ) -> Vec<CodeIssue> {
        // Test/Documentation/Benchmark: 完全跳过
        let weight = context.rule_weight_multiplier();
        if weight < 0.3 {
            return Vec::new();
        }

        // 获取所有问题
        let issues = self.check(file_path, syntax_tree, content, lang, _is_test_file);

        // UI/TUI 文件过滤：检测到 UI 相关路径时，仅保留严重问题
        let path_str = file_path.to_string_lossy().to_lowercase();
        let is_ui_file = path_str.contains("ui")
            || path_str.contains("tui")
            || path_str.contains("gui")
            || path_str.contains("view")
            || path_str.contains("screen")
            || path_str.contains("layout")
            || path_str.contains("display")
            || path_str.contains("render");

        // 如果是 UI 上下文或 UI 文件，过滤掉 Mild 级别的 magic-number 问题
        if matches!(context, FileContext::Business) && is_ui_file {
            return issues
                .into_iter()
                .filter(|issue| matches!(issue.severity, Severity::Spicy | Severity::Nuclear))
                .collect();
        }

        issues
    }
}

/// Detect functions that do too much (god functions)
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
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = GodFunctionVisitor::new(file_path.to_path_buf(), content, lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// Detect commented-out code blocks
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
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut _commented_code_blocks = 0;
        let mut current_block_size = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Detect commented code lines
            if trimmed.starts_with("//") {
                let comment_content = trimmed.trim_start_matches("//").trim();

                // Check if it looks like code (contains common code patterns)
                if is_likely_code(comment_content) {
                    current_block_size += 1;
                } else if current_block_size > 0 {
                    // End a code block
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
                // Non-comment line, end current block
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

        // Handle code block at end of file
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

/// Detect obvious dead code
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
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut dead_code_start: Option<usize> = None;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check if this line is a control flow terminator
            if is_control_flow_terminator(trimmed) {
                // Mark that subsequent lines are dead code
                dead_code_start = Some(line_num + 1);
                continue;
            }

            // If we're in a dead code region and this line has actual code
            if dead_code_start.is_some() {
                // Skip empty lines and comments
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
                    continue;
                }

                // Check if this line closes a block (could be end of if/match/etc)
                if trimmed == "}" || trimmed == "} else {" || trimmed == "} else if" {
                    // Reset dead code region - we've exited the block
                    dead_code_start = None;
                    continue;
                }

                // This is actual dead code after a terminator
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
                });

                // Only report one dead code block per terminator
                dead_code_start = None;
            }
        }

        issues
    }
}

// ============================================================================
// Helper functions
// ============================================================================

fn is_likely_code(content: &str) -> bool {
    // Patterns that look like code
    let code_patterns = [
        "let ", "fn ", "if ", "else", "for ", "while ", "match ", "struct ", "enum ", "impl ",
        "use ", "mod ", "return ", "break", "continue", "{", "}", "(", ")", "[", "]", ";", "=",
        "==", "!=", "&&", "||", "->", "::",
    ];

    let rust_keywords = [
        "pub", "const", "static", "mut", "ref", "move", "async", "await", "unsafe", "extern",
        "crate",
    ];

    // If it contains multiple code patterns, it's likely code
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
    }
}

fn is_control_flow_terminator(line: &str) -> bool {
    // Check if this line is a pure control flow terminator
    // (the line itself terminates execution, not just contains these keywords)
    // Explicit parentheses used to clarify operator precedence (&& binds tighter than ||)
    let exact_matches = matches!(
        line,
        "return;"
            | "break;"
            | "continue;"
            | "unreachable!()"
            | "unreachable!();"
            | "std::process::exit(0);"
            | "std::process::exit(1);"
    );

    let has_return_statement =
        line.starts_with("return ") && line.ends_with(';') && !line.contains("//");

    let has_panic = line.starts_with("panic!(") && line.ends_with(';');

    let has_unreachable = line.starts_with("unreachable!(") && line.ends_with(')');

    exact_matches || has_return_statement || has_panic || has_unreachable
}

// ============================================================================
// Visitor implementations
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
        // Common NON-magic numbers (powers of 2, common sizes, small counts, etc.)
        // These are so common they're almost always intentional
        let safe_numbers = [
            -1,      // Sentinel/error value
            0,       // Zero/initial/default
            1,       // Single/first/true
            2,       // Pair/dual/binary
            3,       // Triple/RGB/xyz
            4,       // Quad/nibble
            5,       // Quint/count
            6,       // Hex/week
            7,       // Week/oct
            8,       // Byte/octet
            9,       // Max digit
            10,      // Decimal base
            12,      // Dozen/months
            16,      // Nibble/4 bits
            20,      // Score
            24,      // Hours/day
            32,      // 5 bits
            64,      // 6 bits
            100,     // Percent/century
            128,     // 7 bits
            256,     // Byte/8 bits
            365,     // Days/year (approx)
            512,     // 9 bits
            1024,    // KB/10 bits
            2048,    // 11 bits
            4096,    // 12 bits / page size
            8192,    // 13 bits
            65536,   // u16 max / 16 bits
            100000,  // 100K
            1000000, // 1M
        ];

        !safe_numbers.contains(&value)
    }

    fn create_magic_number_issue(&self, value: i64, line: usize, column: usize) -> CodeIssue {
        // Categorize the magic number for better messages
        let category = self.categorize_magic_number(value);

        let messages = if self.lang == "zh-CN" {
            match category {
                MagicNumberCategory::Timeout => vec![
                    format!("超时值 {}？建议定义为常量如 TIMEOUT_MS", value),
                    format!("硬编码超时 {}ms，维护性-1，建议用常量", value),
                    format!("魔法数字 {} 看起来像超时，提取为命名常量", value),
                ],
                MagicNumberCategory::BufferSize => vec![
                    format!("缓冲区大小 {}？这是什么咒语？", value),
                    format!("硬编码缓冲区 {}，维护性-1", value),
                    format!("缓冲区大小 {} 从天而降，没人知道它的含义", value),
                ],
                MagicNumberCategory::PortNumber => vec![
                    format!("端口号 {}？硬编码端口不安全且难以维护", value),
                    format!("发现硬编码端口 {}，建议使用配置文件或环境变量", value),
                    format!("端口号 {} 应该定义为常量或从配置读取", value),
                ],
                MagicNumberCategory::Threshold => vec![
                    format!("阈值 {}？这个数字有什么特殊含义？", value),
                    format!("硬编码阈值 {}，建议定义为有意义的常量", value),
                    format!("阈值 {} 缺乏语义，维护者会困惑", value),
                ],
                MagicNumberCategory::General => vec![
                    format!("魔法数字 {}？这是什么咒语？", value),
                    format!("硬编码数字 {}，维护性-1", value),
                    format!("数字 {} 从天而降，没人知道它的含义", value),
                    format!("魔法数字 {}，建议定义为常量", value),
                    format!("看到数字 {}，我陷入了沉思", value),
                ],
            }
        } else {
            match category {
                MagicNumberCategory::Timeout => vec![
                    format!("Timeout value {}? Define as TIMEOUT_MS constant", value),
                    format!("Hardcoded timeout {}ms - extract to constant", value),
                    format!("Magic number {} looks like a timeout, name it", value),
                ],
                MagicNumberCategory::BufferSize => vec![
                    format!("Buffer size {}? What spell is this?", value),
                    format!("Hardcoded buffer {} - maintainability -1", value),
                    format!("Buffer size {} fell from sky, no meaning", value),
                ],
                MagicNumberCategory::PortNumber => vec![
                    format!("Port {}? Hardcoded ports are unmaintainable", value),
                    format!("Hardcoded port {} - use config or env var", value),
                    format!("Port {} should be constant or config-driven", value),
                ],
                MagicNumberCategory::Threshold => vec![
                    format!("Threshold {}? What's special about this?", value),
                    format!("Hardcoded threshold {} - define meaningful const", value),
                    format!("Threshold {} lacks semantics, confusing", value),
                ],
                MagicNumberCategory::General => vec![
                    format!("Magic number {}? What spell is this?", value),
                    format!("Hardcoded number {} - maintainability -1", value),
                    format!(
                        "Number {} fell from the sky, nobody knows its meaning",
                        value
                    ),
                    format!("Magic number {} - consider defining as a constant", value),
                    format!("Seeing number {}, I'm lost in thought", value),
                ],
            }
        };

        // Severity based on value magnitude and category
        let severity = match category {
            MagicNumberCategory::Timeout | MagicNumberCategory::Threshold => {
                if value > 1000 {
                    Severity::Spicy
                } else {
                    Severity::Mild
                }
            }
            MagicNumberCategory::PortNumber => Severity::Spicy, // Ports are always important to name
            _ => {
                if !(-100..=100).contains(&value)
                    && value != 800
                    && value != 1000
                    && value != 2000
                    && value != 3000
                    && value != 5000
                {
                    Severity::Spicy
                } else {
                    Severity::Mild
                }
            }
        };

        CodeIssue {
            file_path: self.file_path.clone(),
            line,
            column,
            rule_name: "magic-number".to_string(),
            message: messages[self.issues.len() % messages.len()].clone(),
            severity,
        }
    }

    fn categorize_magic_number(&self, value: i64) -> MagicNumberCategory {
        // Common timeout values (milliseconds)
        if [
            100, 200, 300, 500, 800, 1000, 1500, 2000, 3000, 5000, 10000, 30000, 60000,
        ]
        .contains(&value)
        {
            return MagicNumberCategory::Timeout;
        }

        // Common buffer sizes
        if [1024, 2048, 4096, 8192, 16384, 32768, 65536].contains(&value) {
            return MagicNumberCategory::BufferSize;
        }

        // Common port numbers
        if (3000..=9999).contains(&value) || value == 80 || value == 443 || value == 8080 {
            return MagicNumberCategory::PortNumber;
        }

        // Common threshold values
        if [
            10, 25, 50, 75, 90, 95, 99, 100, 150, 200, 250, 500, 750, 1000,
        ]
        .contains(&value)
        {
            return MagicNumberCategory::Threshold;
        }

        MagicNumberCategory::General
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
// God function detection
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

        // Calculate various complexity metrics for the function
        let mut complexity_score = 0;

        // 1. Parameter count
        let param_count = func.sig.inputs.len();
        if param_count > 5 {
            complexity_score += (param_count - 5) * 2;
        }

        // 2. Function body size (estimated via string analysis)
        let func_str = format!("{func:?}");
        let line_count = func_str.lines().count();
        if line_count > 50 {
            complexity_score += (line_count - 50) / 10;
        }

        // 3. Nesting depth and control flow complexity
        let control_keywords = ["if", "else", "for", "while", "match", "loop"];
        for keyword in &control_keywords {
            complexity_score += func_str.matches(keyword).count();
        }

        // If complexity is too high, report the issue
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

#[cfg(test)]
mod tests {
    use super::*;

    fn check_rule(rule: &impl Rule, code: &str) -> Vec<CodeIssue> {
        let path = std::path::Path::new("test.rs");
        let ast = syn::parse_file(code).unwrap();
        rule.check(path, &ast, code, "en-US", false)
    }

    #[test]
    fn test_magic_number_detects_hardcoded_value() {
        let code = r#"
            fn main() {
                let timeout = 3742;
                let retries = 99;
            }
        "#;
        let issues = check_rule(&MagicNumberRule, code);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.rule_name == "magic-number"));
    }

    #[test]
    fn test_magic_number_ignores_common_values() {
        let code = r#"
            fn main() {
                let x = 0;
                let y = 1;
                let z = 100;
            }
        "#;
        let issues = check_rule(&MagicNumberRule, code);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_rule_names() {
        assert_eq!(MagicNumberRule.name(), "magic-number");
        assert_eq!(GodFunctionRule.name(), "god-function");
        assert_eq!(CommentedCodeRule.name(), "commented-code");
        assert_eq!(DeadCodeRule.name(), "dead-code");
    }
}
