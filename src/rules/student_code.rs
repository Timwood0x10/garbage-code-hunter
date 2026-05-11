use std::path::Path;
use syn::{visit::Visit, ExprMacro, File};

use crate::analyzer::{CodeIssue, Severity};
use crate::context::FileContext;
use crate::rules::Rule;
use crate::utils::{count_non_comment_matches, find_line_of_str, get_position};

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
        let file_name = file_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        let is_main_file = file_name == "main.rs" || file_name == "lib.rs";

        // Count different types of println! calls (excluding comments)
        let total_println = count_non_comment_matches(content, "println!");
        let total_eprintln = count_non_comment_matches(content, "eprintln!");

        // Patterns that indicate DEBUGGING println! (not normal output)
        let debug_patterns = [
            // Pure debug statements with no meaningful content
            r#"println!("debug"#,
            r#"println!("check"#,
            r#"println!("test"#,
            r#"println!("DEBUG"#,
            r#"println!("here)"#,
            r#"println!("checkpoint"#,
            r#"println!("step"#,
            r#"println!("line "#,
            r#"println!("=== "#,
            r#"println!("--- "#,
            r#"println!(">>> "#,
            // Print with simple variables (likely debug)
            r#"println!("{:?}"#,    // Debug formatting
            r#"println!("{:#?}"#,   // Pretty debug
            r#"println!("x = "#,    // Variable dump
            r#"println!("val:"#,    // Value check
            r#"println!("result:"#, // Result check
            r#"println!("res:"#,    // Short result
            r#"println!("i = "#,    // Loop variable
            r#"println!("j = "#,    // Loop variable
            r#"println!("k = "#,    // Loop variable
            r#"println!("count"#,   // Count debugging
            r#"println!("len("#,    // Length debugging
            r#"println!("size"#,    // Size debugging
            // Empty or minimal prints
            r#"println!("")"#,
            r#"println!()"#,
            // Array/vec debugging
            r#"println!("{:?"#,  // Debug format start
            r#"println!("vec!"#, // Vec printing
        ];

        let mut debug_count = 0;
        for pattern in &debug_patterns {
            debug_count += count_non_comment_matches(content, pattern);
        }

        // Patterns that indicate NORMAL OUTPUT println!
        let output_patterns = [
            // Error handling (eprintln is legitimate)
            r#"eprintln!("Error"#,
            r#"eprintln!("Warning"#,
            r#"eprintln!("Failed"#,
            r#"eprintln!("error:"#,
            r#"eprintln!("warn:"#,
            // UI/CLI output with emojis (legitimate user-facing messages)
            r#"println!("📊"#,      // Stats/metrics output
            r#"println!("🏆"#,      // Score/results
            r#"println!("🗑️"#,      // Tool branding
            r#"println!("✅"#,      // Success messages
            r#"println!("❌"#,      // Error indicators
            r#"println!("⚠️"#,      // Warnings
            r#"println!("🎓"#,      // Educational
            r#"println!("💡"#,      // Tips
            r#"println!("🔥"#,      // Hall of shame
            r#"println!("📍"#,      // Location markers
            r#"println!("🔍"#,      // Search indicators
            r#"println!("⏱️"#,      // Performance
            r#"println!("💾"#,      // File operations
            r#"println!("📝"#,      // Notes
            r#"println!("🎯"#,      // Target/goal
            r#"println!("🚀"#,      // Launch/start
            r#"println!("✨"#,      // Sparkles/new
            r#"println!("🎨"#,      // Art/styling
            r#"println!("📈"#,      // Charts/growth
            r#"println!("─"#,       // Separator lines (repeat)
            r#"println!("{}", "─"#, // Separator with repeat
            // JSON/formatted output (structured data export)
            r#"serde_json::to"#,
            r#"println!("{{"#, // JSON start
            // User-facing messages in quotes (meaningful output)
            r#"Total files"#,
            r#"issues found"#,
            r#"analyzed"#,
            r#"score"#,
            r#"result"#,
            r#"Usage:"#,      // CLI usage info
            r#"Arguments:"#,  // CLI arguments
            r#"Options:"#,    // CLI options
            r#"Version:"#,    // Version info
            r#"Help:"#,       // Help text
            r#"Example:"#,    // Examples
            r#"Note:"#,       // Notes to users
            r#"Tip:"#,        // Tips for users
            r#"Warning:"#,    // Warnings (println version)
            r#"Error:"#,      // Errors (println version)
            r#"Success:"#,    // Success messages
            r#"Failed:"#,     // Failure messages
            r#"Completed:"#,  // Completion messages
            r#"Started:"#,    // Start messages
            r#"Finished:"#,   // Finish messages
            r#"Processing:"#, // Processing status
            r#"Loading:"#,    // Loading status
            r#"Saving:"#,     // Saving status
            r#"Reading:"#,    // Reading status
            r#"Writing:"#,    // Writing status
            r#"Found "#,      // Found items
            r#"Missing "#,    // Missing items
            r#"Invalid "#,    // Invalid items
            r#"Unknown "#,    // Unknown items
            // Formatted tables/lists
            r#"| "#,  // Table format
            r#"- ─"#, // Table separator (dash + em dash)
            // Progress indicators
            r#"%"#, // Percentage
            r#"/"#, // Progress fraction
            // Time/date output
            r#"ms)"#,      // Milliseconds
            r#"seconds)"#, // Seconds
            r#"minutes)"#, // Minutes
        ];

        let mut output_count = 0;
        for pattern in &output_patterns {
            output_count += count_non_comment_matches(content, pattern);
        }

        // Heuristic: remaining println! are suspicious (ensure non-negative)
        let suspicious_count = total_println
            .saturating_add(total_eprintln)
            .saturating_sub(debug_count)
            .saturating_sub(output_count);

        // Rule 1: Flag excessive debug-style println! even in main files
        if debug_count > 3 || (!is_main_file && suspicious_count > 0) {
            let count_to_report = if debug_count > 0 {
                debug_count
            } else {
                suspicious_count
            };

            let severity = if count_to_report > 10 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            let messages = if lang == "zh-CN" {
                vec![
                    format!(
                        "发现 {} 个疑似调试用 println!，上线前记得删掉",
                        count_to_report
                    ),
                    format!("{} 个 println! 看起来像调试代码", count_to_report),
                    format!(
                        "{} 个打印语句，这是要开演唱会吗？",
                        total_println + total_eprintln
                    ),
                    format!("建议用 log::info! 或 eprintln! 替代调试用的 println!",),
                ]
            } else {
                vec![
                    format!(
                        "Found {}疑似 debug println!s - remove before shipping",
                        count_to_report
                    ),
                    format!("{} println!s look like debug code", count_to_report),
                    format!(
                        "{} print statements - hosting a concert?",
                        total_println + total_eprintln
                    ),
                    format!("Consider using log::info! or eprintln! for debug prints"),
                ]
            };

            let line = find_line_of_str(content, "println!");

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line,
                column: 1,
                rule_name: "println-debugging".to_string(),
                message: messages[issues.len() % messages.len()].clone(),
                severity,
            });
        }

        // Rule 2: Flag excessive TOTAL println! in any file (> 20 is too many)
        let total = total_println + total_eprintln;
        if total > 20 {
            let messages = if lang == "zh-CN" {
                vec![
                    format!("{} 个 println!/eprintln！控制台要爆炸了", total),
                    format!("{} 个打印语句，考虑提取到输出模块", total),
                    format!("这么多输出语句，维护性-10",),
                ]
            } else {
                vec![
                    format!("{} println!/eprintln!s! Console explosion imminent", total),
                    format!(
                        "{} print statements - consider extracting to output module",
                        total
                    ),
                    format!("So many output statements, maintainability -10",),
                ]
            };

            let line = find_line_of_str(content, "println!");

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line,
                column: 1,
                rule_name: "println-debugging".to_string(),
                message: messages[issues.len() % messages.len()].clone(),
                severity: Severity::Spicy,
            });
        }

        issues
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
        // Example, Test, Benchmark, Documentation: skip completely
        let weight = context.rule_weight_multiplier();
        if weight < 0.5 {
            return Vec::new();
        }

        // main.rs/lib.rs files: allow more println (normal for CLI tools)
        let file_name = file_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if file_name == "main.rs" || file_name == "lib.rs" {
            // For entry files, only report Nuclear level issues (excessive debug output)
            let issues = self.check(file_path, syntax_tree, content, lang, is_test_file);
            return issues
                .into_iter()
                .filter(|issue| issue.severity == Severity::Nuclear)
                .collect();
        }

        self.check(file_path, syntax_tree, content, lang, is_test_file)
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

        // Check panic! macro calls in content (not strings/comments)
        // Use "panic!(" to match actual macro calls, not "panic!" in strings
        let panic_count = count_non_comment_matches(content, "panic!(");

        if panic_count > 2 {
            let line = find_line_of_str(content, "panic!(");
            visitor.add_excessive_panic_issue(panic_count, line);
        }

        visitor.issues
    }
}

/// Detect excessive TODO comments (both macro calls and comment markers)
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

        // 1. Check macro calls that cause panics (todo!, unimplemented!, etc.)
        let todo_macros = ["todo!", "unimplemented!", "unreachable!"];

        let mut macro_todos = 0;
        for pattern in &todo_macros {
            macro_todos += count_non_comment_matches(content, pattern);
        }

        // 2. Check comment markers (TODO, FIXME, HACK, XXX, BUG, OPTIMIZE, etc.)
        let mut comment_todos = Vec::new(); // Store (line_number, marker_type)
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for TODO-style comments
            if let Some(comment_start) = trimmed.find("//") {
                let comment_content = &trimmed[comment_start + 2..].trim();

                // Check for various TODO markers (case-insensitive)
                let upper = comment_content.to_uppercase();
                let is_todo_marker = upper.starts_with("TODO")
                    || upper.contains(" TODO ")
                    || upper.starts_with("FIXME")
                    || upper.contains(" FIXME ")
                    || upper.starts_with("HACK")
                    || upper.contains(" HACK ")
                    || upper.starts_with("XXX")
                    || upper.contains(" XXX ")
                    || upper.starts_with("BUG")
                    || upper.contains(" BUG ")
                    || upper.starts_with("OPTIMIZE")
                    || upper.contains(" OPTIMIZE ")
                    || upper.starts_with("TEMP")
                    || upper.contains(" TEMP ")
                    || upper.contains("WORKAROUND")
                    || upper.contains("TEMPORARY");

                if is_todo_marker {
                    let marker_type = if upper.starts_with("TODO") || upper.contains(" TODO ") {
                        "TODO"
                    } else if upper.starts_with("FIXME") || upper.contains(" FIXME ") {
                        "FIXME"
                    } else if upper.starts_with("HACK") || upper.contains(" HACK ") {
                        "HACK"
                    } else if upper.starts_with("XXX") || upper.contains(" XXX ") {
                        "XXX"
                    } else if upper.starts_with("BUG") || upper.contains(" BUG ") {
                        "BUG"
                    } else if upper.starts_with("OPTIMIZE") || upper.contains(" OPTIMIZE ") {
                        "OPTIMIZE"
                    } else {
                        "TEMP"
                    };
                    comment_todos.push((line_num + 1, marker_type));
                }
            }
        }

        let total_todos = macro_todos + comment_todos.len();

        // Report issue if there are too many TODOs
        if total_todos > 3 {
            let messages =
                if lang == "zh-CN" {
                    vec![
                        format!(
                            "发现 {} 个 TODO/FIXME（{}个宏 + {}个注释），这是代码还是购物清单？",
                            total_todos,
                            macro_todos,
                            comment_todos.len()
                        ),
                        format!("{} 个未完成标记？你是在写代码还是在记日记？", total_todos),
                        format!("TODO 比实际代码还多，建议改名叫 'TODO Hunter'"),
                        format!("{} 个 TODO，看来这个项目还在'施工中'", total_todos),
                        format!(
                            "这么多 TODO（{}个 {}，{}个 {}），是不是该考虑清理了？",
                            total_todos,
                            if comment_todos.iter().any(|&(_, t)| t == "TODO") {
                                "TODO"
                            } else {
                                "标记"
                            },
                            comment_todos.iter().filter(|&&(_, t)| t == "TODO").count(),
                            if comment_todos.iter().any(|&(_, t)| t == "FIXME") {
                                "FIXME"
                            } else {
                                "标记"
                            },
                        ),
                    ]
                } else {
                    vec![
                        format!(
                        "Found {} TODOs/FIXMEs ({} macros + {} comments) - code or shopping list?",
                        total_todos, macro_todos, comment_todos.len()
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
                        format!(
                            "So many TODOs ({} {}, {} {}) - time for cleanup?",
                            total_todos,
                            if comment_todos.iter().any(|&(_, t)| t == "TODO") {
                                "TODOs"
                            } else {
                                "markers"
                            },
                            comment_todos.iter().filter(|&&(_, t)| t == "TODO").count(),
                            if comment_todos.iter().any(|&(_, t)| t == "FIXME") {
                                "FIXMEs"
                            } else {
                                "markers"
                            },
                        ),
                    ]
                };

            let severity = if total_todos > 10 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            // Find the first TODO/FIXME line (prefer comment markers over macros)
            let line = if !comment_todos.is_empty() {
                comment_todos[0].0
            } else {
                todo_macros
                    .iter()
                    .filter_map(|p| {
                        let l = find_line_of_str(content, p);
                        if l > 1 {
                            Some(l)
                        } else if content.contains(p) {
                            Some(1)
                        } else {
                            None
                        }
                    })
                    .min()
                    .unwrap_or(1)
            };

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line,
                column: 1,
                rule_name: "todo-comment".to_string(),
                message: messages[total_todos % messages.len()].clone(),
                severity,
            });
        }

        // Also report individual stale TODOs (older than 3 months or with specific markers)
        for &(line_num, marker_type) in &comment_todos {
            // Always report FIXME and BUG markers as they're more urgent
            if marker_type == "FIXME" || marker_type == "BUG" {
                let _line_content = content.lines().nth(line_num - 1).unwrap_or("");
                let messages = if lang == "zh-CN" {
                    match marker_type {
                        "FIXME" => vec![
                            format!("FIXME: 这里有已知问题需要修复",),
                            format!("发现 FIXME 标记，请尽快处理",),
                            format!("FIXME 警告：代码有缺陷待修复",),
                        ],
                        "BUG" => vec![
                            format!("🐛 BUG: 这里确认有 bug！",),
                            format!("发现 BUG 标记，这可不是好兆头",),
                            format!("BUG 标记：生产环境前必须修复！",),
                        ],
                        "HACK" => vec![
                            format!("⚡ HACK: 这是一个 workaround，需要重构",),
                            format!("发现 HACK 标记，临时方案该清理了",),
                            format!("HACK 警告：这里的技术债该还了",),
                        ],
                        _ => vec![format!("{} 标记需要关注", marker_type)],
                    }
                } else {
                    match marker_type {
                        "FIXME" => vec![
                            format!("FIXME: Known issue needs fixing",),
                            format!("FIXME found - please address soon",),
                            format!("FIXME alert: Code defect pending fix",),
                        ],
                        "BUG" => vec![
                            format!("🐛 BUG: Confirmed bug here!",),
                            format!("BUG found - this isn't a good sign",),
                            format!("BUG marker: Must fix before production!",),
                        ],
                        "HACK" => vec![
                            format!("⚡ HACK: This is a workaround, needs refactoring",),
                            format!("HACK found - time to clean up temporary solution",),
                            format!("HACK alert: Technical debt to be paid",),
                        ],
                        _ => vec![format!("{} marker needs attention", marker_type)],
                    }
                };

                let severity = match marker_type {
                    "BUG" => Severity::Spicy,
                    "FIXME" => Severity::Mild,
                    _ => Severity::Mild,
                };

                issues.push(CodeIssue {
                    file_path: file_path.to_path_buf(),
                    line: line_num,
                    column: 1,
                    rule_name: format!("todo-{}", marker_type.to_lowercase()),
                    message: messages[line_num % messages.len()].clone(),
                    severity,
                });
            }
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

    fn add_excessive_panic_issue(&mut self, count: usize, line: usize) {
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
            line,
            column: 1,
            rule_name: "panic-abuse".to_string(),
            message: messages[count % messages.len()].clone(),
            severity: Severity::Nuclear,
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
