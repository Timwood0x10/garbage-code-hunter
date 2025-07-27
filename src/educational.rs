/// Educational advice system that provides detailed explanations and improvement suggestions
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EducationalAdvice {
    pub why_bad: String,
    pub how_to_fix: String,
    pub example_bad: Option<String>,
    pub example_good: Option<String>,
    pub rust_docs_link: Option<String>,
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
                example_bad: Some("let d = get_user_data();".to_string()),
                example_good: Some("let user_profile = get_user_data();".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html".to_string()),
                best_practice_tip: Some("变量名应该是自文档化的，读代码的人应该能从名字就理解变量的用途。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Poor variable naming severely impacts code readability, making it difficult for other developers (including future you) to understand the code's intent.".to_string(),
                how_to_fix: "Use descriptive, meaningful variable names that clearly express the variable's purpose and meaning.".to_string(),
                example_bad: Some("let d = get_user_data();".to_string()),
                example_good: Some("let user_profile = get_user_data();".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html".to_string()),
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
                example_bad: Some("let data = process_foo(bar);".to_string()),
                example_good: Some(
                    "let processed_orders = process_customer_orders(raw_orders);".to_string(),
                ),
                rust_docs_link: Some(
                    "https://rust-lang.github.io/api-guidelines/naming.html".to_string(),
                ),
                best_practice_tip: Some(
                    "避免使用通用词汇，选择能准确描述数据性质的词汇。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Using placeholder names like foo, bar, data, temp makes code lose expressiveness and increases maintenance cost.".to_string(),
                how_to_fix: "Choose specific, meaningful names based on the variable's actual purpose.".to_string(),
                example_bad: Some("let data = process_foo(bar);".to_string()),
                example_good: Some("let processed_orders = process_customer_orders(raw_orders);".to_string()),
                rust_docs_link: Some("https://rust-lang.github.io/api-guidelines/naming.html".to_string()),
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
                example_bad: Some("let strUserName: String = get_name();".to_string()),
                example_good: Some("let user_name: String = get_name();".to_string()),
                rust_docs_link: Some(
                    "https://rust-lang.github.io/api-guidelines/naming.html".to_string(),
                ),
                best_practice_tip: Some(
                    "Rust 的强类型系统使得类型前缀变得多余，专注于语义而非类型。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Hungarian notation is outdated in modern programming languages, Rust's type system already provides type safety guarantees.".to_string(),
                how_to_fix: "Use descriptive names instead of type prefixes, let Rust's type system handle type checking.".to_string(),
                example_bad: Some("let strUserName: String = get_name();".to_string()),
                example_good: Some("let user_name: String = get_name();".to_string()),
                rust_docs_link: Some("https://rust-lang.github.io/api-guidelines/naming.html".to_string()),
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
                example_bad: Some("let usr_mgr = UserMgr::new();".to_string()),
                example_good: Some("let user_manager = UserManager::new();".to_string()),
                rust_docs_link: Some(
                    "https://rust-lang.github.io/api-guidelines/naming.html".to_string(),
                ),
                best_practice_tip: Some(
                    "清晰胜过简洁，代码被阅读的次数远超过被编写的次数。".to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Excessive abbreviations make code hard to understand, especially for new team members or yourself months later.".to_string(),
                how_to_fix: "Use complete, clear words. Modern editors' auto-completion makes long names no longer a problem.".to_string(),
                example_bad: Some("let usr_mgr = UserMgr::new();".to_string()),
                example_good: Some("let user_manager = UserManager::new();".to_string()),
                rust_docs_link: Some("https://rust-lang.github.io/api-guidelines/naming.html".to_string()),
                best_practice_tip: Some("Clarity over brevity - code is read far more often than it's written.".to_string()),
            }
        }
    }

    fn create_deep_nesting_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "深层嵌套增加了代码的认知复杂度，使得逻辑难以跟踪和调试。".to_string(),
                how_to_fix: "使用早期返回、提取函数、或者 Rust 的 ? 操作符来减少嵌套层级。".to_string(),
                example_bad: Some("if condition1 {\n    if condition2 {\n        if condition3 {\n            // deep logic\n        }\n    }\n}".to_string()),
                example_good: Some("if !condition1 { return; }\nif !condition2 { return; }\nif !condition3 { return; }\n// logic here".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html".to_string()),
                best_practice_tip: Some("保持嵌套层级在 3 层以内，使用卫语句和早期返回。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Deep nesting increases cognitive complexity, making logic hard to follow and debug.".to_string(),
                how_to_fix: "Use early returns, extract functions, or Rust's ? operator to reduce nesting levels.".to_string(),
                example_bad: Some("if condition1 {\n    if condition2 {\n        if condition3 {\n            // deep logic\n        }\n    }\n}".to_string()),
                example_good: Some("if !condition1 { return; }\nif !condition2 { return; }\nif !condition3 { return; }\n// logic here".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html".to_string()),
                best_practice_tip: Some("Keep nesting levels within 3, use guard clauses and early returns.".to_string()),
            }
        }
    }

    fn create_unwrap_abuse_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "过度使用 unwrap() 会导致程序在遇到错误时直接崩溃，无法优雅地处理异常情况。".to_string(),
                how_to_fix: "使用 match、if let、或者 ? 操作符来正确处理 Option 和 Result 类型。".to_string(),
                example_bad: Some("let value = some_option.unwrap();".to_string()),
                example_good: Some("let value = some_option.unwrap_or_default();\n// or\nlet value = some_option?;".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch09-00-error-handling.html".to_string()),
                best_practice_tip: Some("只在你确定不会失败的情况下使用 unwrap()，并添加注释说明原因。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Excessive use of unwrap() causes programs to crash directly when encountering errors, unable to handle exceptions gracefully.".to_string(),
                how_to_fix: "Use match, if let, or the ? operator to properly handle Option and Result types.".to_string(),
                example_bad: Some("let value = some_option.unwrap();".to_string()),
                example_good: Some("let value = some_option.unwrap_or_default();\n// or\nlet value = some_option?;".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch09-00-error-handling.html".to_string()),
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
                example_bad: Some("fn process_name(name: String) -> String".to_string()),
                example_good: Some("fn process_name(name: &str) -> String".to_string()),
                rust_docs_link: Some(
                    "https://doc.rust-lang.org/book/ch04-03-slices.html".to_string(),
                ),
                best_practice_tip: Some(
                    "优先使用 &str 作为函数参数，这样可以接受 String 和 &str 两种类型。"
                        .to_string(),
                ),
            }
        } else {
            EducationalAdvice {
                why_bad: "Unnecessary String allocations increase memory usage and performance overhead, especially in read-only scenarios.".to_string(),
                how_to_fix: "Use &str where you only need to read strings, use String only when you need ownership.".to_string(),
                example_bad: Some("fn process_name(name: String) -> String".to_string()),
                example_good: Some("fn process_name(name: &str) -> String".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch04-03-slices.html".to_string()),
                best_practice_tip: Some("Prefer &str as function parameters, this way you can accept both String and &str types.".to_string()),
            }
        }
    }

    fn create_println_debugging_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "遗留的 println! 调试语句会污染输出，在生产环境中可能泄露敏感信息。".to_string(),
                how_to_fix: "使用 log 库进行日志记录，或者使用 dbg! 宏进行临时调试（记得删除）。".to_string(),
                example_bad: Some("println!(\"Debug: {:?}\", sensitive_data);".to_string()),
                example_good: Some("log::debug!(\"Processing data: {:?}\", data);\n// or for temporary debugging:\ndbg!(&data);".to_string()),
                rust_docs_link: Some("https://docs.rs/log/latest/log/".to_string()),
                best_practice_tip: Some("使用条件编译 #[cfg(debug_assertions)] 来确保调试代码不会进入生产环境。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Leftover println! debug statements pollute output and may leak sensitive information in production.".to_string(),
                how_to_fix: "Use the log crate for logging, or use the dbg! macro for temporary debugging (remember to remove).".to_string(),
                example_bad: Some("println!(\"Debug: {:?}\", sensitive_data);".to_string()),
                example_good: Some("log::debug!(\"Processing data: {:?}\", data);\n// or for temporary debugging:\ndbg!(&data);".to_string()),
                rust_docs_link: Some("https://docs.rs/log/latest/log/".to_string()),
                best_practice_tip: Some("Use conditional compilation #[cfg(debug_assertions)] to ensure debug code doesn't reach production.".to_string()),
            }
        }
    }

    fn create_file_too_long_advice(&self) -> EducationalAdvice {
        if self.lang == "zh-CN" {
            EducationalAdvice {
                why_bad: "过长的文件难以导航和维护，违反了单一职责原则，增加了代码的复杂性。".to_string(),
                how_to_fix: "将大文件拆分成多个小模块，每个模块负责特定的功能领域。".to_string(),
                example_bad: Some("// 一个包含 2000 行代码的 main.rs 文件".to_string()),
                example_good: Some("// 拆分为：\n// - user_management.rs\n// - data_processing.rs\n// - api_handlers.rs\n// - main.rs (只包含启动逻辑)".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html".to_string()),
                best_practice_tip: Some("保持文件在 500-1000 行以内，超过时考虑按功能拆分模块。".to_string()),
            }
        } else {
            EducationalAdvice {
                why_bad: "Overly long files are hard to navigate and maintain, violate the single responsibility principle, and increase code complexity.".to_string(),
                how_to_fix: "Split large files into multiple small modules, each responsible for specific functional areas.".to_string(),
                example_bad: Some("// A main.rs file containing 2000 lines of code".to_string()),
                example_good: Some("// Split into:\n// - user_management.rs\n// - data_processing.rs\n// - api_handlers.rs\n// - main.rs (startup logic only)".to_string()),
                rust_docs_link: Some("https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html".to_string()),
                best_practice_tip: Some("Keep files within 500-1000 lines, consider splitting by functionality when exceeded.".to_string()),
            }
        }
    }

    // Add more advice creation methods for other rules...
    fn create_god_function_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Functions that do too much violate the single responsibility principle and are hard to test and maintain.".to_string(),
            how_to_fix: "Break down large functions into smaller, focused functions that each do one thing well.".to_string(),
            example_bad: Some("fn process_everything() { /* 100+ lines of mixed logic */ }".to_string()),
            example_good: Some("fn process_data() {\n    let validated = validate_input();\n    let processed = transform_data(validated);\n    save_result(processed);\n}".to_string()),
            rust_docs_link: Some("https://doc.rust-lang.org/book/ch03-03-how-functions-work.html".to_string()),
            best_practice_tip: Some("Keep functions under 20-30 lines and focused on a single task.".to_string()),
        }
    }

    fn create_long_function_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Long functions are harder to understand, test, and maintain.".to_string(),
            how_to_fix: "Extract logical blocks into separate functions with descriptive names."
                .to_string(),
            example_bad: None,
            example_good: None,
            rust_docs_link: Some(
                "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html".to_string(),
            ),
            best_practice_tip: Some(
                "If you can't see the entire function on your screen, it's probably too long."
                    .to_string(),
            ),
        }
    }

    fn create_magic_number_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Magic numbers make code hard to understand and maintain.".to_string(),
            how_to_fix: "Replace magic numbers with named constants that explain their purpose."
                .to_string(),
            example_bad: Some("if age > 18 { /* ... */ }".to_string()),
            example_good: Some(
                "const LEGAL_AGE: u32 = 18;\nif age > LEGAL_AGE { /* ... */ }".to_string(),
            ),
            rust_docs_link: Some(
                "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html#constants"
                    .to_string(),
            ),
            best_practice_tip: Some(
                "Use const declarations for values that have semantic meaning.".to_string(),
            ),
        }
    }

    fn create_commented_code_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Commented-out code clutters the codebase and creates confusion about what's actually used.".to_string(),
            how_to_fix: "Remove commented code - version control systems preserve history.".to_string(),
            example_bad: Some("// let old_logic = process_old_way();\nlet new_logic = process_new_way();".to_string()),
            example_good: Some("let new_logic = process_new_way();".to_string()),
            rust_docs_link: None,
            best_practice_tip: Some("Trust your version control system - delete dead code instead of commenting it out.".to_string()),
        }
    }

    fn create_dead_code_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Dead code increases maintenance burden and can confuse developers."
                .to_string(),
            how_to_fix: "Remove unused functions, variables, and imports regularly.".to_string(),
            example_bad: None,
            example_good: None,
            rust_docs_link: None,
            best_practice_tip: Some(
                "Use #[allow(dead_code)] only temporarily during development.".to_string(),
            ),
        }
    }

    fn create_unnecessary_clone_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Unnecessary clones waste memory and CPU cycles.".to_string(),
            how_to_fix: "Use references when possible, clone only when you need ownership."
                .to_string(),
            example_bad: Some("let copied = original.clone();\nprocess(&copied);".to_string()),
            example_good: Some("process(&original);".to_string()),
            rust_docs_link: Some(
                "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html".to_string(),
            ),
            best_practice_tip: Some(
                "Understand Rust's borrowing rules to minimize unnecessary allocations."
                    .to_string(),
            ),
        }
    }

    fn create_iterator_abuse_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Manual loops are often less efficient and expressive than iterator chains.".to_string(),
            how_to_fix: "Use iterator methods like map, filter, fold instead of manual loops when appropriate.".to_string(),
            example_bad: Some("let mut result = Vec::new();\nfor item in items {\n    if item > 0 {\n        result.push(item * 2);\n    }\n}".to_string()),
            example_good: Some("let result: Vec<_> = items.iter()\n    .filter(|&&x| x > 0)\n    .map(|&x| x * 2)\n    .collect();".to_string()),
            rust_docs_link: Some("https://doc.rust-lang.org/book/ch13-02-iterators.html".to_string()),
            best_practice_tip: Some("Iterator chains are often more efficient due to lazy evaluation and compiler optimizations.".to_string()),
        }
    }

    fn create_panic_abuse_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Excessive panics make programs unreliable and hard to debug in production.".to_string(),
            how_to_fix: "Use Result types for recoverable errors, reserve panics for truly unrecoverable situations.".to_string(),
            example_bad: Some("if input.is_empty() { panic!(\"Input cannot be empty!\"); }".to_string()),
            example_good: Some("if input.is_empty() { return Err(\"Input cannot be empty\".into()); }".to_string()),
            rust_docs_link: Some("https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html".to_string()),
            best_practice_tip: Some("Panics should be used for programming errors, not for expected error conditions.".to_string()),
        }
    }

    fn create_todo_comment_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Too many TODO comments indicate incomplete or poorly planned code.".to_string(),
            how_to_fix: "Either implement the TODOs or create proper issue tracking for future work.".to_string(),
            example_bad: Some("// TODO: implement this\n// TODO: fix this bug\n// TODO: optimize this".to_string()),
            example_good: Some("// Create GitHub issues for planned improvements\n// Implement critical functionality before committing".to_string()),
            rust_docs_link: None,
            best_practice_tip: Some("Use TODO sparingly and always with a specific plan for resolution.".to_string()),
        }
    }

    fn create_unordered_imports_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Unordered imports make it hard to find and manage dependencies.".to_string(),
            how_to_fix: "Use rustfmt to automatically sort imports, or sort them manually by: std, external crates, local modules.".to_string(),
            example_bad: Some("use my_crate::module;\nuse std::collections::HashMap;\nuse serde::Serialize;".to_string()),
            example_good: Some("use std::collections::HashMap;\n\nuse serde::Serialize;\n\nuse my_crate::module;".to_string()),
            rust_docs_link: Some("https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html".to_string()),
            best_practice_tip: Some("Configure your editor to run rustfmt on save to maintain consistent formatting.".to_string()),
        }
    }

    fn create_deep_module_nesting_advice(&self) -> EducationalAdvice {
        EducationalAdvice {
            why_bad: "Deep module nesting makes code navigation difficult and indicates poor architecture.".to_string(),
            how_to_fix: "Flatten module structure, use re-exports to maintain clean public APIs.".to_string(),
            example_bad: Some("mod a { mod b { mod c { mod d { /* code */ } } } }".to_string()),
            example_good: Some("mod handlers;\nmod models;\nmod utils;\n\npub use handlers::*;".to_string()),
            rust_docs_link: Some("https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html".to_string()),
            best_practice_tip: Some("Keep module nesting to 2-3 levels maximum, use re-exports for convenience.".to_string()),
        }
    }
}
