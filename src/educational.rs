/// Educational advice system that provides detailed explanations and improvement suggestions
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EducationalAdvice {
    pub why_bad: String,
    pub how_to_fix: String,
    pub best_practice_tip: Option<String>,
}

pub struct EducationalAdvisor {
    advice_db: HashMap<String, EducationalAdvice>,
    lang: String,
}

impl EducationalAdvisor {
    pub fn new(lang: &str) -> Self {
        let mut advisor = Self {
            advice_db: HashMap::new(),
            lang: lang.to_string(),
        };
        advisor.initialize_advice_database();
        advisor
    }

    pub fn get_advice(&self, rule_name: &str) -> Option<&EducationalAdvice> {
        self.advice_db.get(rule_name)
    }

    fn initialize_advice_database(&mut self) {
        // Naming convention advice
        self.add_advice("terrible-naming", self.create_terrible_naming_advice());
        self.add_advice(
            "meaningless-naming",
            self.create_meaningless_naming_advice(),
        );
        self.add_advice(
            "hungarian-notation",
            self.create_hungarian_notation_advice(),
        );
        self.add_advice(
            "abbreviation-abuse",
            self.create_abbreviation_abuse_advice(),
        );

        // Complexity advice
        self.add_advice("deep-nesting", self.create_deep_nesting_advice());
        self.add_advice("god-function", self.create_god_function_advice());
        self.add_advice("long-function", self.create_long_function_advice());

        // Code smells advice
        self.add_advice("magic-number", self.create_magic_number_advice());
        self.add_advice("commented-code", self.create_commented_code_advice());
        self.add_advice("dead-code", self.create_dead_code_advice());

        // Rust-specific advice
        self.add_advice("unwrap-abuse", self.create_unwrap_abuse_advice());
        self.add_advice("string-abuse", self.create_string_abuse_advice());
        self.add_advice("unnecessary-clone", self.create_unnecessary_clone_advice());
        self.add_advice("iterator-abuse", self.create_iterator_abuse_advice());

        // Student code advice
        self.add_advice("println-debugging", self.create_println_debugging_advice());
        self.add_advice("panic-abuse", self.create_panic_abuse_advice());
        self.add_advice("todo-comment", self.create_todo_comment_advice());

        // File structure advice
        self.add_advice("file-too-long", self.create_file_too_long_advice());
        self.add_advice("unordered-imports", self.create_unordered_imports_advice());
        self.add_advice(
            "deep-module-nesting",
            self.create_deep_module_nesting_advice(),
        );
    }

    fn add_advice(&mut self, rule_name: &str, advice: EducationalAdvice) {
        self.advice_db.insert(rule_name.to_string(), advice);
    }

    fn create_terrible_naming_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "糟糕的变量命名会严重影响代码可读性，让其他开发者（包括未来的你）难以理解代码意图。".to_string(),
                how_to_fix: "使用描述性的、有意义的变量名，清楚地表达变量的用途和含义。".to_string(),
                best_practice_tip: Some("变量名应该是自文档化的，读代码的人应该能从名字就理解变量的用途。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Poor variable naming severely impacts code readability, making it difficult for other developers (including future you) to understand the code's intent.".to_string(),
                how_to_fix: "Use descriptive, meaningful variable names that clearly express the variable's purpose and meaning.".to_string(),
                best_practice_tip: Some("Variable names should be self-documenting - readers should understand the purpose from the name alone.".to_string()),
            }
        }
    }

    fn create_meaningless_naming_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "使用 foo、bar、data、temp 等占位符命名会让代码失去表达力，增加维护成本。"
                    .to_string(),
                how_to_fix: "根据变量的实际用途选择具体的、有意义的名称。".to_string(),
                best_practice_tip: Some(
                    "避免使用通用词汇，选择能准确描述数据性质的词汇。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Using placeholder names like foo, bar, data, temp makes code lose expressiveness and increases maintenance cost.".to_string(),
                how_to_fix: "Choose specific, meaningful names based on the variable's actual purpose.".to_string(),
                best_practice_tip: Some("Avoid generic words, choose words that accurately describe the nature of the data.".to_string()),
            }
        }
    }

    fn create_hungarian_notation_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad:
                    "匈牙利命名法在现代编程语言中已经过时，Rust 的类型系统已经提供了类型安全保障。"
                        .to_string(),
                how_to_fix: "使用描述性名称而不是类型前缀，让 Rust 的类型系统处理类型检查。"
                    .to_string(),
                best_practice_tip: Some(
                    "Rust 的强类型系统使得类型前缀变得多余，专注于语义而非类型。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Hungarian notation is outdated in modern programming languages, Rust's type system already provides type safety guarantees.".to_string(),
                how_to_fix: "Use descriptive names instead of type prefixes, let Rust's type system handle type checking.".to_string(),
                best_practice_tip: Some("Rust's strong type system makes type prefixes redundant, focus on semantics rather than types.".to_string()),
            }
        }
    }

    fn create_abbreviation_abuse_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "过度缩写会让代码变得难以理解，特别是对新团队成员或几个月后的自己。"
                    .to_string(),
                how_to_fix: "使用完整的、清晰的单词，现代编辑器的自动补全让长名称不再是问题。"
                    .to_string(),
                best_practice_tip: Some(
                    "清晰胜过简洁，代码被阅读的次数远超过被编写的次数。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Excessive abbreviations make code hard to understand, especially for new team members or yourself months later.".to_string(),
                how_to_fix: "Use complete, clear words. Modern editors' auto-completion makes long names no longer a problem.".to_string(),
                best_practice_tip: Some("Clarity over brevity - code is read far more often than it's written.".to_string()),
            }
        }
    }

    fn create_deep_nesting_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "深层嵌套增加了代码的认知复杂度，使得逻辑难以跟踪和调试。".to_string(),
                how_to_fix: "使用早期返回、提取函数、或者 Rust 的 ? 操作符来减少嵌套层级。"
                    .to_string(),
                best_practice_tip: Some(
                    "保持嵌套层级在 3 层以内，使用卫语句和早期返回。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Deep nesting increases cognitive complexity, making logic hard to follow and debug.".to_string(),
                how_to_fix: "Use early returns, extract functions, or Rust's ? operator to reduce nesting levels.".to_string(),
                best_practice_tip: Some("Keep nesting levels within 3, use guard clauses and early returns.".to_string()),
            }
        }
    }

    fn create_unwrap_abuse_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad:
                    "过度使用 unwrap() 会导致程序在遇到错误时直接崩溃，无法优雅地处理异常情况。"
                        .to_string(),
                how_to_fix: "使用 match、if let、或者 ? 操作符来正确处理 Option 和 Result 类型。"
                    .to_string(),
                best_practice_tip: Some(
                    "只在你确定不会失败的情况下使用 unwrap()，并添加注释说明原因。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Excessive use of unwrap() causes programs to crash directly when encountering errors, unable to handle exceptions gracefully.".to_string(),
                how_to_fix: "Use match, if let, or the ? operator to properly handle Option and Result types.".to_string(),
                best_practice_tip: Some("Only use unwrap() when you're certain it won't fail, and add comments explaining why.".to_string()),
            }
        }
    }

    fn create_string_abuse_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad:
                    "不必要的 String 分配会增加内存使用和性能开销，特别是在只需要读取的场景中。"
                        .to_string(),
                how_to_fix: "在只需要读取字符串的地方使用 &str，只在需要拥有所有权时使用 String。"
                    .to_string(),
                best_practice_tip: Some(
                    "优先使用 &str 作为函数参数，这样可以接受 String 和 &str 两种类型。"
                        .to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Unnecessary String allocations increase memory usage and performance overhead, especially in read-only scenarios.".to_string(),
                how_to_fix: "Use &str where you only need to read strings, use String only when you need ownership.".to_string(),
                best_practice_tip: Some("Prefer &str as function parameters, this way you can accept both String and &str types.".to_string()),
            }
        }
    }

    fn create_println_debugging_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "遗留的 println! 调试语句会污染输出，在生产环境中可能泄露敏感信息。"
                    .to_string(),
                how_to_fix: "使用 log 库进行日志记录，或者使用 dbg! 宏进行临时调试（记得删除）。"
                    .to_string(),
                best_practice_tip: Some(
                    "使用条件编译 #[cfg(debug_assertions)] 来确保调试代码不会进入生产环境。"
                        .to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Leftover println! debug statements pollute output and may leak sensitive information in production.".to_string(),
                how_to_fix: "Use the log crate for logging, or use the dbg! macro for temporary debugging (remember to remove).".to_string(),
                best_practice_tip: Some("Use conditional compilation #[cfg(debug_assertions)] to ensure debug code doesn't reach production.".to_string()),
            }
        }
    }

    fn create_file_too_long_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "过长的文件难以导航和维护，违反了单一职责原则，增加了代码复杂度。"
                    .to_string(),
                how_to_fix: "将大文件拆分成多个小模块，每个模块负责特定的功能领域。".to_string(),
                best_practice_tip: Some(
                    "保持文件在 500-1000 行以内，超出时考虑按功能拆分。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Overly long files are hard to navigate and maintain, violate the single responsibility principle, and increase code complexity.".to_string(),
                how_to_fix: "Split large files into multiple small modules, each responsible for specific functional areas.".to_string(),
                best_practice_tip: Some("Keep files within 500-1000 lines, consider splitting by functionality when exceeded.".to_string()),
            }
        }
    }

    // Add more advice creation methods for other rules...
    fn create_god_function_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "做太多事情的函数违反了单一职责原则，难以测试和维护。".to_string(),
                how_to_fix: "将大函数拆分成更小的、专注的函数，每个函数只做一件事。".to_string(),
                best_practice_tip: Some("保持函数在 20-30 行以内，专注于单一任务。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Functions that do too much violate the single responsibility principle and are hard to test and maintain.".to_string(),
                how_to_fix: "Break down large functions into smaller, focused functions that each do one thing well.".to_string(),
                best_practice_tip: Some("Keep functions under 20-30 lines and focused on a single task.".to_string()),
            }
        }
    }

    fn create_long_function_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "长函数更难理解、测试和维护。".to_string(),
                how_to_fix: "将逻辑块提取到具有描述性名称的独立函数中。".to_string(),
                best_practice_tip: Some(
                    "如果你无法在屏幕上看到整个函数，那它可能太长了。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Long functions are harder to understand, test, and maintain.".to_string(),
                how_to_fix:
                    "Extract logical blocks into separate functions with descriptive names."
                        .to_string(),
                best_practice_tip: Some(
                    "If you can't see the entire function on your screen, it's probably too long."
                        .to_string(),
                ),
            }
        }
    }

    fn create_magic_number_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "魔法数字让代码难以理解和维护。".to_string(),
                how_to_fix: "用能解释其用途的命名常量替换魔法数字。".to_string(),
                best_practice_tip: Some("对具有语义含义的值使用 const 声明。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Magic numbers make code hard to understand and maintain.".to_string(),
                how_to_fix:
                    "Replace magic numbers with named constants that explain their purpose."
                        .to_string(),
                best_practice_tip: Some(
                    "Use const declarations for values that have semantic meaning.".to_string(),
                ),
            }
        }
    }

    fn create_commented_code_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "注释掉的代码会污染代码库，让人困惑哪些是真正在使用的代码。".to_string(),
                how_to_fix: "删除注释掉的代码 - 版本控制系统会保留历史记录。".to_string(),
                best_practice_tip: Some(
                    "相信你的版本控制系统 - 删除死代码而不是注释掉它。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Commented-out code clutters the codebase and creates confusion about what's actually used.".to_string(),
                how_to_fix: "Remove commented code - version control systems preserve history.".to_string(),
                best_practice_tip: Some("Trust your version control system - delete dead code instead of commenting it out.".to_string()),
            }
        }
    }

    fn create_dead_code_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "死代码增加了维护负担，会让开发者困惑。".to_string(),
                how_to_fix: "定期删除未使用的函数、变量和导入。".to_string(),
                best_practice_tip: Some("使用 cargo clippy 自动检测死代码。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Dead code increases maintenance burden and can confuse developers."
                    .to_string(),
                how_to_fix: "Remove unused functions, variables, and imports regularly."
                    .to_string(),
                best_practice_tip: Some(
                    "Use cargo clippy to detect dead code automatically.".to_string(),
                ),
            }
        }
    }

    fn create_unnecessary_clone_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "不必要的 clone 会浪费内存和 CPU 周期。".to_string(),
                how_to_fix: "尽可能使用引用，只在需要所有权时才 clone。".to_string(),
                best_practice_tip: Some("理解 Rust 的借用规则以减少不必要的分配。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Unnecessary clones waste memory and CPU cycles.".to_string(),
                how_to_fix: "Use references when possible, clone only when you need ownership."
                    .to_string(),
                best_practice_tip: Some(
                    "Understand Rust's borrowing rules to minimize unnecessary allocations."
                        .to_string(),
                ),
            }
        }
    }

    fn create_iterator_abuse_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "手动循环通常比迭代器链更低效且表达力差。".to_string(),
                how_to_fix: "在适当的情况下使用 map、filter、fold 等迭代器方法代替手动循环。"
                    .to_string(),
                best_practice_tip: Some(
                    "迭代器链由于惰性求值和编译器优化，通常更高效。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Manual loops are often less efficient and expressive than iterator chains.".to_string(),
                how_to_fix: "Use iterator methods like map, filter, fold instead of manual loops when appropriate.".to_string(),
                best_practice_tip: Some("Iterator chains are often more efficient due to lazy evaluation and compiler optimizations.".to_string()),
            }
        }
    }

    fn create_panic_abuse_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "过度使用 panic 会让程序在生产环境中不可靠且难以调试。".to_string(),
                how_to_fix: "对可恢复的错误使用 Result 类型，仅在真正无法恢复的情况下使用 panic。"
                    .to_string(),
                best_practice_tip: Some(
                    "Panic 应该用于编程错误，而不是预期的错误条件。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Excessive panics make programs unreliable and hard to debug in production.".to_string(),
                how_to_fix: "Use Result types for recoverable errors, reserve panics for truly unrecoverable situations.".to_string(),
                best_practice_tip: Some("Panics should be used for programming errors, not for expected error conditions.".to_string()),
            }
        }
    }

    fn create_todo_comment_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "过多的 TODO 注释表示代码不完整或规划不当。".to_string(),
                how_to_fix: "要么实现这些 TODO，要么为未来的工作创建适当的问题跟踪。".to_string(),
                best_practice_tip: Some(
                    "谨慎使用 TODO，并始终附带具体的解决方案计划。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Too many TODO comments indicate incomplete or poorly planned code."
                    .to_string(),
                how_to_fix:
                    "Either implement the TODOs or create proper issue tracking for future work."
                        .to_string(),
                best_practice_tip: Some(
                    "Use TODO sparingly and always with a specific plan for resolution."
                        .to_string(),
                ),
            }
        }
    }

    fn create_unordered_imports_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Unordered imports make it hard to find and manage dependencies.".to_string(),
            how_to_fix: "Use rustfmt to automatically sort imports, or sort them manually by: std, external crates, local modules.".to_string(),
            best_practice_tip: Some("Configure your editor to run rustfmt on save to maintain consistent formatting.".to_string()),
        }
    }

    fn create_deep_module_nesting_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Deep module nesting makes code navigation difficult and indicates poor architecture.".to_string(),
            how_to_fix: "Flatten module structure, use re-exports to maintain clean public APIs.".to_string(),
            best_practice_tip: Some("Keep module nesting to 2-3 levels maximum, use re-exports for convenience.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_RULES: &[&str] = &[
        "terrible-naming",
        "meaningless-naming",
        "hungarian-notation",
        "abbreviation-abuse",
        "deep-nesting",
        "god-function",
        "long-function",
        "magic-number",
        "commented-code",
        "dead-code",
        "unwrap-abuse",
        "string-abuse",
        "unnecessary-clone",
        "iterator-abuse",
        "println-debugging",
        "panic-abuse",
        "todo-comment",
        "file-too-long",
        "unordered-imports",
        "deep-module-nesting",
    ];

    /// Objective: Verify that every known rule has English advice registered.
    /// Invariants: If a rule is added to the engine but not the advice DB,
    ///             calling get_advice must return Some(...).
    #[test]
    fn test_all_rules_have_english_advice() {
        let advisor = EducationalAdvisor::new("en");
        for rule in ALL_RULES {
            let advice = advisor.get_advice(rule);
            assert!(
                advice.is_some(),
                "rule '{rule}' has no advice registered in English database"
            );
            let a = advice.unwrap();
            assert!(!a.why_bad.is_empty(), "rule '{rule}' has empty why_bad");
            assert!(
                !a.how_to_fix.is_empty(),
                "rule '{rule}' has empty how_to_fix"
            );
        }
    }

    /// Objective: Verify that every known rule has Chinese advice registered.
    /// Invariants: Even English-only rules (unordered-imports, deep-module-nesting)
    ///             should still return Some (they store English text even when lang=zh-CN).
    #[test]
    fn test_all_rules_have_chinese_advice() {
        let advisor = EducationalAdvisor::new("zh-CN");
        for rule in ALL_RULES {
            let advice = advisor.get_advice(rule);
            assert!(
                advice.is_some(),
                "rule '{rule}' has no advice when lang=zh-CN"
            );
        }
    }

    /// Objective: Verify unknown rule returns None, not a crash.
    #[test]
    fn test_unknown_rule_returns_none() {
        let advisor = EducationalAdvisor::new("en");
        assert!(
            advisor.get_advice("nonexistent-rule").is_none(),
            "unknown rule should return None"
        );
    }

    /// Objective: Verify that English `terrible-naming` advice contains
    ///            expected English tokens, confirming the i18n path works.
    #[test]
    fn test_english_terrible_naming_has_expected_content() {
        let advisor = EducationalAdvisor::new("en");
        let advice = advisor
            .get_advice("terrible-naming")
            .expect("terrible-naming should exist");
        assert!(
            advice.why_bad.contains("readability"),
            "English why_bad should mention readability, got: {}",
            advice.why_bad
        );
        assert!(
            advice.how_to_fix.contains("descriptive"),
            "English how_to_fix should suggest descriptive names, got: {}",
            advice.how_to_fix
        );
        assert!(
            advice
                .best_practice_tip
                .as_ref()
                .unwrap()
                .contains("self-documenting"),
            "English tip should mention self-documenting"
        );
    }

    /// Objective: Verify that Chinese `terrible-naming` advice contains
    ///            expected Chinese characters, confirming the zh-CN path works.
    #[test]
    fn test_chinese_terrible_naming_has_expected_content() {
        let advisor = EducationalAdvisor::new("zh-CN");
        let advice = advisor
            .get_advice("terrible-naming")
            .expect("terrible-naming should exist");
        assert!(
            advice.why_bad.contains("糟糕"),
            "Chinese why_bad should contain '糟糕', got: {}",
            advice.why_bad
        );
        assert!(
            advice.how_to_fix.contains("描述性"),
            "Chinese how_to_fix should contain '描述性', got: {}",
            advice.how_to_fix
        );
        assert!(
            advice
                .best_practice_tip
                .as_ref()
                .unwrap()
                .contains("自文档化"),
            "Chinese tip should contain '自文档化'"
        );
    }

    /// Objective: Verify ALL English advice entries have a non-None best_practice_tip.
    /// Invariants: best_practice_tip is always Some for every rule in English.
    #[test]
    fn test_all_english_advice_have_best_practice_tip() {
        let advisor = EducationalAdvisor::new("en");
        for (rule, advice) in &advisor.advice_db {
            assert!(
                advice.best_practice_tip.is_some(),
                "rule '{rule}' is missing best_practice_tip in English"
            );
        }
    }

    /// Objective: Verify ALL Chinese advice entries have a non-None best_practice_tip.
    #[test]
    fn test_all_chinese_advice_have_best_practice_tip() {
        let advisor = EducationalAdvisor::new("zh-CN");
        for (rule, advice) in &advisor.advice_db {
            assert!(
                advice.best_practice_tip.is_some(),
                "rule '{rule}' is missing best_practice_tip in Chinese"
            );
        }
    }

    /// Objective: Verify that English-only rules (unordered-imports, deep-module-nesting)
    ///            still contain English text even when lang=zh-CN.
    /// Invariants: These rules have no Chinese translation; they fall back to English.
    #[test]
    fn test_english_only_rules_remain_english_in_chinese_mode() {
        let advisor = EducationalAdvisor::new("zh-CN");
        for rule in &["unordered-imports", "deep-module-nesting"] {
            let advice = advisor
                .get_advice(rule)
                .expect("rule should exist in zh-CN");
            assert!(
                advice.why_bad.contains(' '),
                "rule '{rule}' in zh-CN should still contain English (has spaces): {}",
                advice.why_bad
            );
        }
    }

    /// Objective: Verify the advice DB is initialized with exactly the expected
    ///            number of entries, catching accidental deletions or duplicates.
    /// Invariants: The count must match the number of unique rules that have advice.
    #[test]
    fn test_advice_db_has_expected_rule_count() {
        let advisor = EducationalAdvisor::new("en");
        // 4 naming + 3 complexity + 3 code-smells + 4 Rust-specific + 3 student + 3 file-structure = 20
        assert_eq!(
            advisor.advice_db.len(),
            ALL_RULES.len(),
            "expected {} rules in advice DB, got {}",
            ALL_RULES.len(),
            advisor.advice_db.len()
        );
    }

    /// Objective: Verify that how_to_fix for every English rule is at least 10 chars long,
    ///            meaning it provides meaningful guidance rather than a one-liner.
    /// Invariants: Short how_to_fix would indicate content was accidentally left incomplete.
    #[test]
    fn test_how_to_fix_minimum_length() {
        let advisor = EducationalAdvisor::new("en");
        let min_len = 10;
        for (rule, advice) in &advisor.advice_db {
            assert!(
                advice.how_to_fix.len() >= min_len,
                "rule '{rule}' how_to_fix is too short ({}, min {min_len}): '{}'",
                advice.how_to_fix.len(),
                advice.how_to_fix
            );
        }
    }

    /// Objective: Verify that advice is deterministic — same rule, same lang, same content.
    /// Invariants: Two advisor instances with the same language produce identical advice.
    #[test]
    fn test_advice_is_deterministic() {
        let a1 = EducationalAdvisor::new("en");
        let a2 = EducationalAdvisor::new("en");
        for rule in ALL_RULES {
            let adv1 = a1.get_advice(rule).expect("rule should exist");
            let adv2 = a2.get_advice(rule).expect("rule should exist");
            assert_eq!(
                adv1.why_bad, adv2.why_bad,
                "rule '{rule}' why_bad differs between instances"
            );
            assert_eq!(
                adv1.how_to_fix, adv2.how_to_fix,
                "rule '{rule}' how_to_fix differs between instances"
            );
        }
    }
}
