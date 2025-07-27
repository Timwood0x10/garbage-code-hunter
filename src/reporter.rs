#[allow(dead_code)]
use colored::*;
use std::collections::{HashMap, BTreeMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::analyzer::{CodeIssue, Severity};
use crate::educational::EducationalAdvisor;
use crate::hall_of_shame::HallOfShame;
use crate::i18n::I18n;
use crate::scoring::{CodeQualityScore, CodeScorer};

pub struct Reporter {
    harsh_mode: bool,
    savage_mode: bool,
    verbose: bool,
    top_files: usize,
    max_issues_per_file: usize,
    summary_only: bool,
    markdown: bool,
    i18n: I18n,
}

#[allow(dead_code)]
impl Reporter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        harsh_mode: bool,
        savage_mode: bool,
        verbose: bool,
        top_files: usize,
        max_issues_per_file: usize,
        summary_only: bool,
        markdown: bool,
        lang: &str,
    ) -> Self {
        Self {
            harsh_mode,
            savage_mode,
            verbose,
            top_files,
            max_issues_per_file,
            summary_only,
            markdown,
            i18n: I18n::new(lang),
        }
    }

    /// get random roast message
    fn get_random_roast(&self, category: &str, score: f64, seed: u64) -> String {
        let roasts = self.get_category_roasts(category, score);
        if roasts.is_empty() {
            return if self.i18n.lang == "zh-CN" {
                "代码需要改进 🔧".to_string()
            } else {
                "Code needs improvement 🔧".to_string()
            };
        }
        
        // seed genearte random index
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        category.hash(&mut hasher);
        let hash = hasher.finish();
        let index = (hash as usize) % roasts.len();
        
        roasts[index].to_string()
    }

    /// get roast message
    fn get_category_roasts(&self, category: &str, score: f64) -> Vec<&str> {
        if self.i18n.lang == "zh-CN" {
            match category {
                "命名规范" => {
                    if score >= 80.0 {
                        vec![
                            "恭喜！你成功让变量名比注释还难懂 🏆",
                            "这些变量名是用随机字符生成器起的吗？ 🎲",
                            "变量命名水平堪比密码设置 🔐",
                            "看到这些变量名，我想起了古代象形文字 📜",
                            "变量名比我的人生还迷茫 😵‍💫",
                            "这命名风格很有'艺术感' 🎨",
                            "变量名的创意程度超越了我的理解 🚀",
                        ]
                    } else if score >= 60.0 {
                        vec![
                            "变量命名还有改进空间 📝",
                            "建议给变量起个有意义的名字 💭",
                            "变量名可以更清晰一些 ✨",
                            "命名规范需要加强 📚",
                        ]
                    } else {
                        vec!["变量命名还不错 👍", "命名风格可以接受 ✅"]
                    }
                }
                "复杂度" => {
                    if score >= 80.0 {
                        vec![
                            "复杂度爆表！连AI都看不懂了 🤖",
                            "这代码比迷宫还复杂 🌀",
                            "嵌套层数比俄罗斯套娃还多 🪆",
                            "代码复杂度已经超越了人类理解范围 🧠",
                            "这函数比我的感情生活还复杂 💔",
                            "建议拆分成多个小函数，拯救一下可读性 🆘",
                            "复杂度高到需要GPS导航 🗺️",
                            "这代码比数学公式还抽象 📐",
                            "嵌套深度堪比洋葱，剥一层哭一次 🧅",
                            "代码结构比立体拼图还复杂 🧩",
                            "这复杂度让我想起了哲学问题 🤔",
                            "函数长度已经突破天际 🚀",
                            "这代码需要配个说明书 📖",
                            "复杂度比我的作息时间还乱 ⏰",
                            "建议给这个函数买个保险 🛡️",
                        ]
                    } else if score >= 60.0 {
                        vec![
                            "代码有点复杂，建议简化 🔧",
                            "函数可以拆分得更小一些 ✂️",
                            "嵌套层数有点多 📚",
                            "复杂度需要控制一下 ⚖️",
                            "代码结构可以更清晰 🏗️",
                            "建议重构一下逻辑 🔄",
                            "函数职责可以更单一 🎯",
                            "代码可读性需要提升 👓",
                        ]
                    } else {
                        vec!["代码结构还算清晰 👌", "复杂度控制得不错 ✅"]
                    }
                }
                "代码重复" => {
                    if score >= 80.0 {
                        vec![
                            "建议改名为copy-paste.rs 📋",
                            "重复代码比我重复的梦还多 💤",
                            "Ctrl+C 和 Ctrl+V 是你最好的朋友吗？ ⌨️",
                            "代码重复度堪比复读机 🔄",
                            "这么多重复，建议学学DRY原则 🏜️",
                            "重复代码多到可以开复制店了 🏪",
                            "代码重复率比我的日常还高 📈",
                            "这重复程度可以申请吉尼斯纪录了 🏆",
                            "代码复制粘贴技能满级 🎮",
                            "重复代码比回音还响亮 📢",
                            "这是代码还是复印机作品？ 🖨️",
                            "DRY原则在你这里变成了WET原则 💧",
                            "重复代码比我的口头禅还频繁 🗣️",
                            "建议给复制粘贴键盘买个保险 ⌨️",
                            "代码重复度比镜子还厉害 🪞",
                        ]
                    } else if score >= 60.0 {
                        vec![
                            "有一些重复代码需要处理 🔧",
                            "建议提取公共函数 📦",
                            "重复代码可以优化 ✨",
                            "考虑重构重复的部分 🔄",
                            "代码复用性可以提升 🔗",
                            "建议抽象出通用逻辑 🎯",
                            "重复部分可以模块化 📋",
                            "代码结构需要优化 🏗️",
                        ]
                    } else {
                        vec!["代码重复控制得不错 👍", "重复度在可接受范围 ✅"]
                    }
                }
                "Rust功能" => {
                    if score >= 80.0 {
                        vec![
                            "宏定义比我的借口还多 🎭",
                            "unwrap() 用得比我说'没问题'还频繁 😅",
                            "String 分配比我花钱还随意 💸",
                            "这代码让 Rust 编译器都想罢工 🚫",
                            "panic! 用得这么随意，用户体验堪忧 😱",
                            "迭代器哭了：为什么不用我？ 😢",
                            "match 表示：我可以更简洁的 💪",
                            "Vec::new() 比我换衣服还频繁 👕",
                            "to_string() 调用比我眨眼还多 👁️",
                            "这代码让 Rust 的零成本抽象哭了 😭",
                            "错误处理？什么是错误处理？ 🤷‍♂️",
                            "生命周期标注比我的简历还复杂 📄",
                            "这代码违反了 Rust 的哲学原则 📚",
                            "建议重新学习 Rust 最佳实践 🎓",
                            "Rust 社区看到这代码会流泪 🦀",
                        ]
                    } else if score >= 60.0 {
                        vec![
                            "Rust 特性使用需要改进 🦀",
                            "建议更好地利用 Rust 的特性 ⚡",
                            "代码可以更 Rust 化 🔧",
                            "某些模式可以优化 ✨",
                            "错误处理可以更优雅 🎭",
                            "内存管理还有优化空间 💾",
                            "迭代器使用可以加强 🔄",
                            "类型系统利用不够充分 📊",
                        ]
                    } else {
                        vec!["Rust 特性使用得不错 🦀", "代码很 Rust 化 ⚡"]
                    }
                }
                _ => vec!["代码需要改进 🔧"]
            }
        } else {
            // 英文版本的吐槽
            match category {
                "Naming" => {
                    if score >= 80.0 {
                        vec![
                            "Congrats! Your variable names are more confusing than comments 🏆",
                            "Did you use a random character generator for these names? 🎲",
                            "Variable naming skills rival password creation 🔐",
                            "These names remind me of ancient hieroglyphs 📜",
                            "Variable names are more lost than my life purpose 😵‍💫",
                            "This naming style is very 'artistic' 🎨",
                            "The creativity of these names exceeds my understanding 🚀",
                            "Variable names harder to decode than alien language 👽",
                            "These names are more abstract than modern art 🖼️",
                            "Did you name these variables with your eyes closed? 👀",
                            "Variable naming master class: how to confuse everyone 🎓",
                            "These names could win a cryptography contest 🔍",
                            "Variable names more mysterious than unsolved puzzles 🧩",
                            "I've seen more meaningful names in spam emails 📧",
                            "These names make dictionary words jealous 📚",
                        ]
                    } else if score >= 60.0 {
                        vec![
                            "Variable naming has room for improvement 📝",
                            "Consider giving variables meaningful names 💭",
                            "Variable names could be clearer ✨",
                            "Naming conventions need strengthening 📚",
                            "Variable readability could be enhanced 👀",
                            "Naming is an art - keep practicing! 💪",
                            "Variables could be more expressive 🗣️",
                            "Naming style needs consistency 📐",
                        ]
                    } else {
                        vec!["Variable naming is decent 👍", "Naming style is acceptable ✅"]
                    }
                }
                "Complexity" => {
                    if score >= 80.0 {
                        vec![
                            "Complexity off the charts! Even AI can't understand 🤖",
                            "This code is more complex than a maze 🌀",
                            "More nesting levels than Russian dolls 🪆",
                            "Code complexity has transcended human understanding 🧠",
                            "This function is more complex than my love life 💔",
                            "Consider splitting into smaller functions to save readability 🆘",
                            "Complexity so high it needs GPS navigation 🗺️",
                            "This code is more abstract than quantum physics 📐",
                            "Nesting deeper than an onion, each layer brings tears 🧅",
                            "Code structure more complex than a 3D puzzle 🧩",
                            "This complexity makes philosophy look simple 🤔",
                            "Function length has reached astronomical proportions 🚀",
                            "This code needs a user manual 📖",
                            "Complexity more chaotic than my sleep schedule ⏰",
                            "Consider getting insurance for this function 🛡️",
                        ]
                    } else if score >= 60.0 {
                        vec![
                            "Code is a bit complex, consider simplifying 🔧",
                            "Functions could be split smaller ✂️",
                            "A bit too many nesting levels 📚",
                            "Complexity needs some control ⚖️",
                            "Code structure could be clearer 🏗️",
                            "Consider refactoring the logic 🔄",
                            "Function responsibilities could be more focused 🎯",
                            "Code readability needs improvement 👓",
                        ]
                    } else {
                        vec!["Code structure is fairly clear 👌", "Complexity is well controlled ✅"]
                    }
                }
                "Duplication" => {
                    if score >= 80.0 {
                        vec![
                            "Consider renaming to copy-paste.rs 📋",
                            "More duplicate code than my recurring dreams 💤",
                            "Are Ctrl+C and Ctrl+V your best friends? ⌨️",
                            "Code duplication rivals a parrot 🔄",
                            "So much duplication, time to learn DRY principle 🏜️",
                            "Enough duplicate code to open a copy shop 🏪",
                            "Code duplication rate higher than my daily routine 📈",
                            "This duplication level deserves a Guinness World Record 🏆",
                            "Copy-paste skills have reached maximum level 🎮",
                            "Duplicate code echoes louder than a canyon 📢",
                            "Is this code or a photocopier masterpiece? 🖨️",
                            "DRY principle became WET principle in your hands 💧",
                            "Code repetition more frequent than my catchphrases 🗣️",
                            "Consider buying insurance for your copy-paste keys ⌨️",
                            "Duplication level surpasses hall of mirrors 🪞",
                        ]
                    } else if score >= 60.0 {
                        vec![
                            "Some duplicate code needs handling 🔧",
                            "Consider extracting common functions 📦",
                            "Duplicate code can be optimized ✨",
                            "Consider refactoring repeated parts 🔄",
                            "Code reusability could be improved 🔗",
                            "Consider abstracting common logic 🎯",
                            "Repeated sections could be modularized 📋",
                            "Code structure needs optimization 🏗️",
                        ]
                    } else {
                        vec!["Code duplication is well controlled 👍", "Duplication within acceptable range ✅"]
                    }
                }
                "Rust Features" => {
                    if score >= 80.0 {
                        vec![
                            "More macro definitions than my excuses 🎭",
                            "unwrap() used more frequently than I say 'no problem' 😅",
                            "String allocation more casual than my spending 💸",
                            "This code makes Rust compiler want to quit 🚫",
                            "panic! used so casually, user experience is questionable 😱",
                            "Iterators are crying: why don't you use me? 😢",
                            "match says: I can be more concise 💪",
                            "Vec::new() calls more frequent than my outfit changes 👕",
                            "to_string() calls exceed my blink count 👁️",
                            "This code made Rust's zero-cost abstractions weep 😭",
                            "Error handling? What's error handling? 🤷‍♂️",
                            "Lifetime annotations more complex than my resume 📄",
                            "This code violates Rust's philosophical principles 📚",
                            "Consider retaking Rust best practices course 🎓",
                            "Rust community would shed tears seeing this code 🦀",
                        ]
                    } else if score >= 60.0 {
                        vec![
                            "Rust feature usage needs improvement 🦀",
                            "Consider better utilization of Rust features ⚡",
                            "Code could be more Rust-idiomatic 🔧",
                            "Some patterns can be optimized ✨",
                            "Error handling could be more elegant 🎭",
                            "Memory management has room for optimization 💾",
                            "Iterator usage could be strengthened 🔄",
                            "Type system utilization is insufficient 📊",
                        ]
                    } else {
                        vec!["Rust features used well 🦀", "Code is very Rust-idiomatic ⚡"]
                    }
                }
                _ => vec!["Code needs improvement 🔧"]
            }
        }
    }

    #[allow(dead_code)]
    pub fn report(&self, issues: Vec<CodeIssue>) {
        self.report_with_metrics(issues, 1, 100);
    }

    pub fn report_with_enhanced_features(
        &self, 
        mut issues: Vec<CodeIssue>, 
        file_count: usize, 
        total_lines: usize,
        educational_advisor: Option<&EducationalAdvisor>,
        hall_of_shame: Option<&HallOfShame>,
        show_suggestions: bool,
    ) {
        // calculate quality score
        let scorer = CodeScorer::new();
        let quality_score = scorer.calculate_score(&issues, file_count, total_lines);

        if issues.is_empty() {
            self.print_clean_code_message_with_score(&quality_score);
            return;
        }

        //sort by severity
        issues.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Nuclear => 3,
                Severity::Spicy => 2,
                Severity::Mild => 1,
            };
            severity_order(&b.severity).cmp(&severity_order(&a.severity))
        });

        // if harsh mode  only show the most severe issue
        if self.harsh_mode {
            issues.retain(|issue| matches!(issue.severity, Severity::Nuclear | Severity::Spicy));
        }

        if self.markdown {
            self.print_markdown_report_enhanced(&issues, &quality_score, educational_advisor, hall_of_shame, show_suggestions);
        } else {
            if !self.summary_only {
                self.print_header(&issues);
                self.print_quality_score(&quality_score);
                if self.verbose {
                    self.print_detailed_analysis(&issues);
                }
                self.print_top_files(&issues);
                self.print_issues_enhanced(&issues, educational_advisor);
            }
            self.print_summary_with_score(&issues, &quality_score);
            if !self.summary_only {
                // Print hall of shame if requested
                if let Some(shame) = hall_of_shame {
                    self.print_hall_of_shame(shame);
                }
                
                // Print improvement suggestions if requested
                if show_suggestions {
                    if let Some(shame) = hall_of_shame {
                        self.print_improvement_suggestions(shame);
                    }
                }
                
                // Always show footer for non-enhanced mode
                if !show_suggestions {
                    self.print_footer(&issues);
                }
            }
        }
    }

    pub fn report_with_metrics(
        &self,
        mut issues: Vec<CodeIssue>,
        file_count: usize,
        total_lines: usize,
    ) {
        // calculate quality score
        let scorer = CodeScorer::new();
        let quality_score = scorer.calculate_score(&issues, file_count, total_lines);

        if issues.is_empty() {
            self.print_clean_code_message_with_score(&quality_score);
            return;
        }

        //sort by severity
        issues.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Nuclear => 3,
                Severity::Spicy => 2,
                Severity::Mild => 1,
            };
            severity_order(&b.severity).cmp(&severity_order(&a.severity))
        });

        // if harsh mode  only show the most severe issue
        if self.harsh_mode {
            issues.retain(|issue| matches!(issue.severity, Severity::Nuclear | Severity::Spicy));
        }

        if self.markdown {
            self.print_markdown_report(&issues);
        } else {
            if !self.summary_only {
                self.print_header(&issues);
                self.print_quality_score(&quality_score);
                if self.verbose {
                    self.print_detailed_analysis(&issues);
                }
                self.print_top_files(&issues);
                self.print_issues(&issues);
            }
            self.print_summary_with_score(&issues, &quality_score);
            if !self.summary_only {
                self.print_footer(&issues);
            }
        }
    }

    #[allow(dead_code)]
    fn print_clean_code_message(&self) {
        if self.markdown {
            println!("# {}", self.i18n.get("title"));
            println!();
            println!("{}", self.i18n.get("clean_code"));
            println!();
            println!("{}", self.i18n.get("clean_code_warning"));
        } else {
            println!("{}", self.i18n.get("clean_code").bright_green().bold());
            println!("{}", self.i18n.get("clean_code_warning").yellow());
        }
    }

    fn print_clean_code_message_with_score(&self, quality_score: &CodeQualityScore) {
        if self.markdown {
            println!("# {}", self.i18n.get("title"));
            println!();
            println!("## 🏆 代码质量评分");
            println!();
            println!(
                "**评分**: {:.1}/100 {}",
                quality_score.total_score,
                quality_score.quality_level.emoji()
            );
            println!(
                "**等级**: {}",
                quality_score.quality_level.description(&self.i18n.lang)
            );
            println!();
            println!("{}", self.i18n.get("clean_code"));
            println!();
            println!("{}", self.i18n.get("clean_code_warning"));
        } else {
            println!("{}", self.i18n.get("clean_code").bright_green().bold());
            println!();
            println!(
                "{} 代码质量评分: {:.1}/100 {}",
                "🏆".bright_yellow(),
                quality_score.total_score.to_string().bright_green().bold(),
                quality_score.quality_level.emoji()
            );
            println!(
                "{} 质量等级: {}",
                "📊".bright_blue(),
                quality_score
                    .quality_level
                    .description(&self.i18n.lang)
                    .bright_green()
                    .bold()
            );
            println!("{}", self.i18n.get("clean_code_warning").yellow());
        }
    }

    fn print_quality_score(&self, quality_score: &CodeQualityScore) {
        let title = match self.i18n.lang.as_str() {
            "zh-CN" => "🏆 代码质量评分",
            _ => "🏆 Code Quality Score",
        };
        println!("{}", title.bright_yellow().bold());
        println!("{}", "─".repeat(50).bright_black());

        let _score_color = match quality_score.quality_level {
            crate::scoring::QualityLevel::Excellent => {
                quality_score.total_score.to_string().bright_green().bold()
            }
            crate::scoring::QualityLevel::Good => quality_score.total_score.to_string().green(),
            crate::scoring::QualityLevel::Average => quality_score.total_score.to_string().yellow(),
            crate::scoring::QualityLevel::Poor => quality_score.total_score.to_string().red(),
            crate::scoring::QualityLevel::Terrible => {
                quality_score.total_score.to_string().bright_red().bold()
            }
        };

        let (score_label, level_label) = match self.i18n.lang.as_str() {
            "zh-CN" => ("📊 总分", "🎯 等级"),
            _ => ("📊 Score", "🎯 Level"),
        };

        println!(
            "   {}: {:.1}/100 {}",
            score_label,
            quality_score.total_score,
            quality_score.quality_level.emoji()
        );
        println!(
            "   {}: {}",
            level_label,
            quality_score
                .quality_level
                .description(&self.i18n.lang)
                .bright_white()
                .bold()
        );

        if quality_score.total_lines > 0 {
            let (lines_label, files_label, density_label) = match self.i18n.lang.as_str() {
                "zh-CN" => ("📏 代码行数", "📁 文件数量", "🔍 问题密度"),
                _ => ("📏 Lines of Code", "📁 Files", "🔍 Issue Density"),
            };
            let density_unit = match self.i18n.lang.as_str() {
                "zh-CN" => "问题/千行",
                _ => "issues/1k lines",
            };

            println!(
                "   {}: {}",
                lines_label,
                quality_score.total_lines.to_string().cyan()
            );
            println!(
                "   {}: {}",
                files_label,
                quality_score.file_count.to_string().cyan()
            );
            println!(
                "   {}: {:.2} {}",
                density_label,
                quality_score.issue_density.to_string().cyan(),
                density_unit
            );
        }

        
        if quality_score.severity_distribution.nuclear > 0
            || quality_score.severity_distribution.spicy > 0
            || quality_score.severity_distribution.mild > 0
        {
            println!();
            let distribution_title = match self.i18n.lang.as_str() {
                "zh-CN" => "🎭 问题分布:",
                _ => "🎭 Issue Distribution:",
            };
            let (nuclear_label, spicy_label, mild_label) = match self.i18n.lang.as_str() {
                "zh-CN" => ("💥 核弹级", "🌶️  严重", "😐 轻微"),
                _ => ("💥 Nuclear", "🌶️  Spicy", "😐 Mild"),
            };

            println!("   {distribution_title}");
            if quality_score.severity_distribution.nuclear > 0 {
                println!(
                    "      {}: {}",
                    nuclear_label,
                    quality_score
                        .severity_distribution
                        .nuclear
                        .to_string()
                        .red()
                        .bold()
                );
            }
            if quality_score.severity_distribution.spicy > 0 {
                println!(
                    "      {}: {}",
                    spicy_label,
                    quality_score
                        .severity_distribution
                        .spicy
                        .to_string()
                        .yellow()
                );
            }
            if quality_score.severity_distribution.mild > 0 {
                println!(
                    "      {}: {}",
                    mild_label,
                    quality_score.severity_distribution.mild.to_string().blue()
                );
            }
        }

        // 显示分类得分（如果有的话）
        if !quality_score.category_scores.is_empty() && self.verbose {
            println!();
            let category_title = match self.i18n.lang.as_str() {
                "zh-CN" => "📋 分类得分:",
                _ => "📋 Category Scores:",
            };
            println!("   {category_title}");
            let mut sorted_categories: Vec<_> = quality_score.category_scores.iter().collect();
            sorted_categories
                .sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (category, score) in sorted_categories.iter().take(5) {
                let category_name = match (self.i18n.lang.as_str(), category.as_str()) {
                    ("zh-CN", "naming") => "命名规范",
                    ("zh-CN", "complexity") => "复杂度",
                    ("zh-CN", "rust-basics") => "Rust基础",
                    ("zh-CN", "advanced-rust") => "高级特性",
                    ("zh-CN", "rust-features") => "Rust功能",
                    ("zh-CN", "structure") => "代码结构",
                    ("zh-CN", "duplication") => "重复代码",
                    (_, "naming") => "Naming",
                    (_, "complexity") => "Complexity",
                    (_, "rust-basics") => "Rust Basics",
                    (_, "advanced-rust") => "Advanced Rust",
                    (_, "rust-features") => "Rust Features",
                    (_, "structure") => "Code Structure",
                    (_, "duplication") => "Code Duplication",
                    _ => category,
                };
                println!(
                    "      {} {:.1}",
                    category_name.cyan(),
                    score.to_string().yellow()
                );
            }
        }

        println!();
    }

    fn print_header(&self, issues: &[CodeIssue]) {
        let total = issues.len();
        let nuclear = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let spicy = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Spicy))
            .count();
        let mild = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Mild))
            .count();

        println!("{}", self.i18n.get("title").bright_red().bold());
        println!("{}", self.i18n.get("preparing").yellow());
        println!();

        println!("{}", self.i18n.get("report_title").bright_red().bold());
        println!("{}", "─".repeat(50).bright_black());

        if self.savage_mode {
            println!("{}", self.i18n.get("found_issues").red().bold());
        } else {
            println!("{}", self.i18n.get("found_issues").yellow());
        }

        println!();
        println!("{}", self.i18n.get("statistics"));
        println!(
            "   {} {}",
            nuclear.to_string().red().bold(),
            self.i18n.get("nuclear_issues")
        );
        println!(
            "   {} {}",
            spicy.to_string().yellow().bold(),
            self.i18n.get("spicy_issues")
        );
        println!(
            "   {} {}",
            mild.to_string().blue().bold(),
            self.i18n.get("mild_issues")
        );
        println!(
            "   {} {}",
            total.to_string().bright_white().bold(),
            self.i18n.get("total")
        );
        println!();
    }

    fn print_issues(&self, issues: &[CodeIssue]) {
        let mut file_groups: HashMap<String, Vec<&CodeIssue>> = HashMap::new();

        for issue in issues {
            let file_name = issue
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            file_groups.entry(file_name).or_default().push(issue);
        }

        for (file_name, file_issues) in file_groups {
            println!("{} {}", "📁".bright_blue(), file_name.bright_blue().bold());

            // Group issues by rule type
            let mut rule_groups: BTreeMap<String, Vec<&CodeIssue>> = BTreeMap::new();
            for issue in &file_issues {
                rule_groups
                    .entry(issue.rule_name.clone())
                    .or_default()
                    .push(issue);
            }

            // Show limited number of issues per rule type
            let _max_per_rule = 5;
            let mut total_shown = 0;
            let max_total = if self.max_issues_per_file > 0 {
                self.max_issues_per_file
            } else {
                usize::MAX
            };

            // Sort rule groups by severity (most severe first)
            let mut sorted_rules: Vec<_> = rule_groups.into_iter().collect();
            sorted_rules.sort_by(|a, b| {
                let severity_order = |s: &Severity| match s {
                    Severity::Nuclear => 3,
                    Severity::Spicy => 2,
                    Severity::Mild => 1,
                };
                let max_severity_a =
                    a.1.iter()
                        .map(|i| severity_order(&i.severity))
                        .max()
                        .unwrap_or(1);
                let max_severity_b =
                    b.1.iter()
                        .map(|i| severity_order(&i.severity))
                        .max()
                        .unwrap_or(1);
                max_severity_b.cmp(&max_severity_a)
            });

            for (rule_name, rule_issues) in sorted_rules {
                if total_shown >= max_total {
                    break;
                }

                let rule_issues_len = rule_issues.len();

                // Create compact summary for each rule type
                if rule_name.contains("naming") || rule_name.contains("single-letter") {
                    // Collect variable names for naming issues
                    let bad_names: Vec<String> = rule_issues
                        .iter()
                        .filter_map(|issue| {
                            if let Some(start) = issue.message.find("'") {
                                issue.message[start + 1..].find("'").map(|end| issue.message[start + 1..start + 1 + end].to_string())
                            } else {
                                None
                            }
                        })
                        .take(5)
                        .collect();

                    let names_display = if bad_names.len() < rule_issues_len {
                        format!("{}, ...", bad_names.join(", "))
                    } else {
                        bad_names.join(", ")
                    };

                    let label = if self.i18n.lang == "zh-CN" {
                        "变量命名问题"
                    } else {
                        "Variable naming issues"
                    };

                    println!(
                        "  🏷️ {}: {} ({})",
                        label.bright_yellow().bold(),
                        rule_issues_len.to_string().bright_red().bold(),
                        names_display.bright_black()
                    );
                    total_shown += 1;
                } else if rule_name.contains("duplication") {
                    let label = if self.i18n.lang == "zh-CN" {
                        "代码重复问题"
                    } else {
                        "Code duplication issues"
                    };

                    // Extract instance count from message if available
                    let instance_info = if let Some(first_issue) = rule_issues.first() {
                        if first_issue.message.contains("instances") {
                            let parts: Vec<&str> = first_issue.message.split_whitespace().collect();
                            if let Some(pos) = parts.iter().position(|&x| x == "instances") {
                                if pos > 0 {
                                    format!("{} instances", parts[pos - 1])
                                } else {
                                    "multiple instances".to_string()
                                }
                            } else if self.i18n.lang == "zh-CN" {
                                "多个代码块".to_string()
                            } else {
                                "multiple blocks".to_string()
                            }
                        } else if self.i18n.lang == "zh-CN" {
                            "多个代码块".to_string()
                        } else {
                            "multiple blocks".to_string()
                        }
                    } else if self.i18n.lang == "zh-CN" {
                        "多个代码块".to_string()
                    } else {
                        "multiple blocks".to_string()
                    };

                    println!(
                        "  🔄 {}: {} ({})",
                        label.bright_cyan().bold(),
                        rule_issues_len.to_string().bright_cyan().bold(),
                        instance_info.bright_black()
                    );
                    total_shown += 1;
                } else if rule_name.contains("nesting") {
                    let label = if self.i18n.lang == "zh-CN" {
                        "嵌套深度问题"
                    } else {
                        "Nesting depth issues"
                    };

                    // Extract depth range from messages
                    let depths: Vec<usize> = rule_issues
                        .iter()
                        .filter_map(|issue| {
                            if let Some(start) = issue.message.find("depth: ") {
                                let depth_str = &issue.message[start + 7..];
                                if let Some(end) = depth_str.find(')') {
                                    depth_str[..end].parse().ok()
                                } else {
                                    None
                                }
                            } else if let Some(start) = issue.message.find("深度: ") {
                                let depth_str = &issue.message[start + 6..];
                                if let Some(end) = depth_str.find(')') {
                                    depth_str[..end].parse().ok()
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    let depth_info = if !depths.is_empty() {
                        let min_depth = depths.iter().min().unwrap_or(&4);
                        let max_depth = depths.iter().max().unwrap_or(&8);
                        if min_depth == max_depth {
                            format!("depth {min_depth}")
                        } else {
                            format!("depth {min_depth}-{max_depth}")
                        }
                    } else if self.i18n.lang == "zh-CN" {
                        "深度嵌套".to_string()
                    } else {
                        "deep nesting".to_string()
                    };

                    println!(
                        "  📦 {}: {} ({})",
                        label.bright_magenta().bold(),
                        rule_issues_len.to_string().bright_magenta().bold(),
                        depth_info.bright_black()
                    );
                    total_shown += 1;
                } else {
                    // For other types, show a generic summary with proper translation
                    let display_name = match (self.i18n.lang.as_str(), rule_name.as_str()) {
                        ("zh-CN", "panic-abuse") => "panic 滥用",
                        ("zh-CN", "god-function") => "上帝函数",
                        ("zh-CN", "magic-number") => "魔法数字",
                        ("zh-CN", "todo-comment") => "TODO 注释",
                        ("zh-CN", "println-debugging") => "println 调试",
                        ("zh-CN", "string-abuse") => "String 滥用",
                        ("zh-CN", "vec-abuse") => "Vec 滥用",
                        ("zh-CN", "iterator-abuse") => "迭代器滥用",
                        ("zh-CN", "match-abuse") => "Match 滥用",
                        ("zh-CN", "hungarian-notation") => "匈牙利命名法",
                        ("zh-CN", "abbreviation-abuse") => "过度缩写",
                        ("zh-CN", "meaningless-naming") => "无意义命名",
                        ("zh-CN", "commented-code") => "被注释代码",
                        ("zh-CN", "dead-code") => "死代码",
                        _ => &rule_name.replace("-", " "),
                    };
                    println!(
                        "  ⚠️ {}: {}",
                        display_name.bright_yellow().bold(),
                        rule_issues_len.to_string().bright_yellow().bold()
                    );
                    total_shown += 1;
                }
            }
            println!();
        }
    }

    fn print_issue(&self, issue: &CodeIssue) {
        // Choose icon and color based on rule type
        if issue.rule_name.contains("duplication") {
            let message = if self.i18n.lang == "zh-CN" {
                &issue.message
            } else {
                // Translate common duplication messages to English
                if issue.message.contains("相似代码块") {
                    "Found similar code blocks, consider refactoring into functions"
                } else if issue.message.contains("DRY原则哭了") {
                    "Code duplication detected, DRY principle violated"
                } else {
                    &issue.message
                }
            };
            println!(
                "  {} {} {}",
                "🔄".bright_cyan(),
                "duplicate".bright_black(),
                message.bright_cyan().bold()
            );
        } else if issue.rule_name.contains("nesting") {
            let message = if self.i18n.lang == "zh-CN" {
                &issue.message
            } else {
                // Translate common nesting messages to English
                if issue.message.contains("俄罗斯套娃") {
                    "Nesting deeper than Russian dolls, are you writing a maze?"
                } else if issue.message.contains("挖到地心") {
                    "Nesting so deep, trying to dig to the Earth's core?"
                } else if issue.message.contains("像洋葱一样") {
                    "Code nested like an onion, makes me want to cry"
                } else {
                    &issue.message
                }
            };
            println!(
                "  {} {} {}",
                "📦".bright_magenta(),
                "nesting".bright_black(),
                message.bright_magenta()
            );
        } else {
            // Default based on severity
            let severity_icon = match issue.severity {
                Severity::Nuclear => "💥",
                Severity::Spicy => "🌶️",
                Severity::Mild => "😐",
            };

            let line_info = format!("{}:{}", issue.line, issue.column);
            let colored_message = match issue.severity {
                Severity::Nuclear => issue.message.red().bold(),
                Severity::Spicy => issue.message.yellow(),
                Severity::Mild => issue.message.blue(),
            };

            let _final_message = if self.savage_mode {
                self.make_message_savage(&issue.message)
            } else {
                issue.message.clone()
            };

            println!(
                "  {} {} {}",
                severity_icon.bright_yellow(),
                line_info.bright_black(),
                colored_message
            );
        }
    }

    fn make_message_savage(&self, message: &str) -> String {
        let savage_prefixes = ["🔥 严重警告：",
            "💀 代码死刑：",
            "🗑️ 垃圾警报：",
            "😱 恐怖发现：",
            "🤮 令人作呕："];

        let prefix = savage_prefixes[message.len() % savage_prefixes.len()];
        format!("{prefix} {message}")
    }

    fn print_summary_with_score(&self, issues: &[CodeIssue], quality_score: &CodeQualityScore) {
        // Print enhanced summary with better layout
        self.print_enhanced_summary(issues, quality_score);
    }

    fn print_enhanced_summary(&self, issues: &[CodeIssue], quality_score: &CodeQualityScore) {
        println!();
        
        // Header with decorative border
        if self.i18n.lang == "zh-CN" {
            println!("{}", "🏆 代码质量报告".bright_cyan().bold());
            println!("{}", "═".repeat(60).bright_black());
        } else {
            println!("{}", "🏆 Code Quality Report".bright_cyan().bold());
            println!("{}", "═".repeat(60).bright_black());
        }

        // Overall score section with card-like layout
        let score_bar = self.create_enhanced_score_bar(quality_score.total_score);
        let score_emoji = quality_score.quality_level.emoji();
        let score_desc = quality_score.quality_level.description(&self.i18n.lang);

        if self.i18n.lang == "zh-CN" {
            println!("╭─ 📊 总体评分 ─────────────────────────────────────────╮");
            println!("│                                                      │");
            
            // Format score line with proper alignment
            let score_text = format!("总分: {:.1}/100", quality_score.total_score);
            let status_text = format!("({score_emoji} {score_desc})");
            println!("│  {}  {}  {}│", 
                score_text.bright_red().bold(),
                score_bar,
                status_text.bright_black()
            );
            
            // Add file statistics
            let file_count = issues.iter().map(|i| &i.file_path).collect::<std::collections::HashSet<_>>().len();
            let total_issues = issues.len();
            println!("│                                                      │");
            let stats_text = format!("分析文件: {file_count} 个    问题总数: {total_issues} 个");
            println!("│  {stats_text}                              │");
            println!("│                                                      │");
            println!("╰──────────────────────────────────────────────────────╯");
        } else {
            println!("╭─ 📊 Overall Score ───────────────────────────────────╮");
            println!("│                                                      │");
            
            // Format score line with proper alignment
            let score_text = format!("Score: {:.1}/100", quality_score.total_score);
            let status_text = format!("({score_emoji} {score_desc})");
            println!("│  {}  {}  {}│", 
                score_text.bright_red().bold(),
                score_bar,
                status_text.bright_black()
            );
            
            // Add file statistics
            let file_count = issues.iter().map(|i| &i.file_path).collect::<std::collections::HashSet<_>>().len();
            let total_issues = issues.len();
            println!("│                                                      │");
            let stats_text = format!("Files analyzed: {file_count}    Total issues: {total_issues}");
            println!("│  {stats_text}                           │");
            println!("│                                                      │");
            println!("╰──────────────────────────────────────────────────────╯");
        }

        println!();
        self.print_category_scores_enhanced(&quality_score.category_scores);

        println!();
        self.print_quality_legend();

        // Only show improvement suggestions if explicitly requested via --suggestions flag
        // This makes the --suggestions parameter more meaningful
    }

    fn create_enhanced_score_bar(&self, score: f64) -> String {
        let bar_length = 20;
        // 注意：分数越高代码越烂，所以用红色表示高分
        let filled_length = ((score / 100.0) * bar_length as f64) as usize;
        let empty_length = bar_length - filled_length;
        
        let filled_char = if score >= 80.0 {
            "█".red()
        } else if score >= 60.0 {
            "█".yellow()
        } else if score >= 40.0 {
            "█".blue()
        } else {
            "█".green()
        };
        
        let empty_char = "▒".bright_black();
        
        format!("{}{}",
            filled_char.to_string().repeat(filled_length),
            empty_char.to_string().repeat(empty_length)
        )
    }

    fn print_category_scores_enhanced(&self, category_scores: &std::collections::HashMap<String, f64>) {
        if self.i18n.lang == "zh-CN" {
            println!("{}", "📋 分类评分详情".bright_yellow().bold());
        } else {
            println!("{}", "📋 Category Scores".bright_yellow().bold());
        }
        println!("{}", "─".repeat(60).bright_black());

        // Define category display order and info
        let categories = [
            ("naming", "命名规范", "Naming", "🏷️"),
            ("complexity", "复杂度", "Complexity", "🧩"),
            ("duplication", "代码重复", "Duplication", "🔄"),
            ("rust-basics", "Rust基础", "Rust Basics", "🦀"),
            ("advanced-rust", "高级特性", "Advanced Rust", "⚡"),
            ("rust-features", "Rust功能", "Rust Features", "🚀"),
            ("structure", "代码结构", "Code Structure", "🏗️"),
        ];

        for (category_key, zh_name, en_name, icon) in &categories {
            if let Some(score) = category_scores.get(*category_key) {
                let display_name = if self.i18n.lang == "zh-CN" {
                    zh_name
                } else {
                    en_name
                };
                let (status_icon, status_text) = self.get_score_status(*score);
                let score_bar = self.create_enhanced_score_bar(*score);

                // Enhanced display with progress bar
                let score_unit = if self.i18n.lang == "zh-CN" { "分" } else { "" };
                println!(
                    "   {} {} [{:>3}{}] {} {}",
                    status_icon,
                    format!("{icon} {display_name}").bright_white().bold(),
                    format!("{score:.0}").bright_cyan(),
                    score_unit,
                    score_bar,
                    status_text.bright_black()
                );

                // if score is high (code is bad), add a roast
                if let Some(roast) = self.get_category_roast(category_key, *score) {
                    println!("       💬 {}", roast.bright_yellow().italic());
                }
            }
        }
        println!();
    }

    fn print_quality_legend(&self) {
        if self.i18n.lang == "zh-CN" {
            println!("{}", "📏 评分标准 (分数越高代码越烂)".bright_yellow().bold());
            println!("{}", "─".repeat(40).bright_black());
            println!("   💀 81-100分: 糟糕，急需重写    🔥 61-80分: 较差，建议重构");
            println!("   ⚠️  41-60分: 一般，需要改进    ✅ 21-40分: 良好，还有提升空间");
            println!("   🌟 0-20分: 优秀，继续保持");
        } else {
            println!("{}", "📏 Scoring Scale (higher score = worse code)".bright_yellow().bold());
            println!("{}", "─".repeat(50).bright_black());
            println!("   💀 81-100: Terrible, rewrite needed    🔥 61-80: Poor, refactoring recommended");
            println!("   ⚠️  41-60: Average, needs improvement   ✅ 21-40: Good, room for improvement");
            println!("   🌟 0-20: Excellent, keep it up");
        }
    }

    fn print_improvement_suggestions_enhanced(&self, quality_score: &CodeQualityScore) {
        if self.i18n.lang == "zh-CN" {
            println!("{}", "💡 改进建议".bright_green().bold());
        } else {
            println!("{}", "💡 Improvement Suggestions".bright_green().bold());
        }
        println!("{}", "─".repeat(50).bright_black());

        let suggestions = self.generate_improvement_suggestions_from_score(quality_score);
        for suggestion in suggestions {
            println!("   • {}", suggestion.green());
        }
    }

    fn generate_improvement_suggestions_from_score(&self, quality_score: &CodeQualityScore) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Sort categories by score (worst first)
        let mut sorted_categories: Vec<_> = quality_score.category_scores.iter().collect();
        sorted_categories.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        for (category, score) in sorted_categories.iter().take(3) {
            if **score > 60.0 {
                let suggestion = match (self.i18n.lang.as_str(), category.as_str()) {
                    ("zh-CN", "naming") => "🏷️ 重点改进变量和函数命名 - 清晰的名称让代码自文档化",
                    ("zh-CN", "complexity") => "🧩 将复杂函数分解为更小、更专注的函数",
                    ("zh-CN", "duplication") => "🔄 消除重复代码，提取公共函数和模块",
                    ("zh-CN", "rust-features") => "🦀 学习和应用 Rust 最佳实践，减少不必要的分配",
                    (_, "naming") => "🏷️ Focus on improving variable and function naming - clear names make code self-documenting",
                    (_, "complexity") => "🧩 Break down complex functions into smaller, focused functions",
                    (_, "duplication") => "🔄 Eliminate code duplication, extract common functions and modules",
                    (_, "rust-features") => "🦀 Learn and apply Rust best practices, reduce unnecessary allocations",
                    _ => continue,
                };
                suggestions.push(suggestion.to_string());
            }
        }
        
        if suggestions.is_empty() {
            if self.i18n.lang == "zh-CN" {
                suggestions.push("🎉 代码质量不错！继续保持良好的编程习惯".to_string());
            } else {
                suggestions.push("🎉 Code quality looks good! Keep up the good programming habits".to_string());
            }
        }
        
        suggestions
    }

    fn print_old_summary_with_score(&self, issues: &[CodeIssue], quality_score: &CodeQualityScore) {
        // Print detailed scoring breakdown
        self.print_scoring_breakdown(issues, quality_score);
        let _nuclear_count = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let _total_count = issues.len();

        println!("{}", self.i18n.get("summary").bright_white().bold());
        println!("{}", "─".repeat(50).bright_black());

        // 显示评分总结
        let score_summary = match quality_score.quality_level {
            crate::scoring::QualityLevel::Excellent => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "🏆 代码质量优秀！评分: {:.1}/100",
                    quality_score.total_score
                ),
                _ => format!(
                    "🏆 Excellent code quality! Score: {:.1}/100",
                    quality_score.total_score
                ),
            },
            crate::scoring::QualityLevel::Good => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "👍 代码质量良好，评分: {:.1}/100",
                    quality_score.total_score
                ),
                _ => format!(
                    "👍 Good code quality, Score: {:.1}/100",
                    quality_score.total_score
                ),
            },
            crate::scoring::QualityLevel::Average => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "😐 代码质量一般，评分: {:.1}/100，还有改进空间",
                    quality_score.total_score
                ),
                _ => format!(
                    "😐 Average code quality, Score: {:.1}/100, room for improvement",
                    quality_score.total_score
                ),
            },
            crate::scoring::QualityLevel::Poor => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "😟 代码质量较差，评分: {:.1}/100，建议重构",
                    quality_score.total_score
                ),
                _ => format!(
                    "😟 Poor code quality, Score: {:.1}/100, refactoring recommended",
                    quality_score.total_score
                ),
            },
            crate::scoring::QualityLevel::Terrible => match self.i18n.lang.as_str() {
                "zh-CN" => format!(
                    "💀 代码质量糟糕，评分: {:.1}/100，急需重写",
                    quality_score.total_score
                ),
                _ => format!(
                    "💀 Terrible code quality, Score: {:.1}/100, rewrite urgently needed",
                    quality_score.total_score
                ),
            },
        };

        let score_color = match quality_score.quality_level {
            crate::scoring::QualityLevel::Excellent => score_summary.bright_green().bold(),
            crate::scoring::QualityLevel::Good => score_summary.green(),
            crate::scoring::QualityLevel::Average => score_summary.yellow(),
            crate::scoring::QualityLevel::Poor => score_summary.red(),
            crate::scoring::QualityLevel::Terrible => score_summary.bright_red().bold(),
        };

        println!("{score_color}");
        println!();

        let nuclear_count = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let total_count = issues.len();

        let summary_message = if nuclear_count > 0 {
            if self.savage_mode {
                match self.i18n.lang.as_str() {
                    "zh-CN" => "你的代码质量堪忧，建议重新学习编程基础 💀".to_string(),
                    _ => "Your code quality is concerning, suggest learning programming basics again 💀".to_string(),
                }
            } else {
                match self.i18n.lang.as_str() {
                    "zh-CN" => "发现了一些严重问题，建议优先修复核弹级问题 🔥".to_string(),
                    _ => "Found some serious issues, suggest fixing nuclear problems first 🔥"
                        .to_string(),
                }
            }
        } else if total_count > 10 {
            match self.i18n.lang.as_str() {
                "zh-CN" => "问题有点多，建议分批修复 📝".to_string(),
                _ => "Quite a few issues, suggest fixing them in batches 📝".to_string(),
            }
        } else {
            match self.i18n.lang.as_str() {
                "zh-CN" => "问题不多，稍微改进一下就好了 👍".to_string(),
                _ => "Not many issues, just need some minor improvements 👍".to_string(),
            }
        };

        let color = if nuclear_count > 0 {
            summary_message.red().bold()
        } else if total_count > 10 {
            summary_message.yellow()
        } else {
            summary_message.green()
        };

        println!("{color}");
    }

    fn print_scoring_breakdown(&self, _issues: &[CodeIssue], quality_score: &CodeQualityScore) {
        let title = if self.i18n.lang == "zh-CN" {
            "📊 评分详情"
        } else {
            "📊 Scoring Details"
        };

        println!("\n{}", title.bright_cyan().bold());
        println!("{}", "─".repeat(50).bright_black());

        // Show category scores
        self.print_category_scores(&quality_score.category_scores);

        // Show weighted calculation
        self.print_weighted_calculation(&quality_score.category_scores, quality_score.total_score);

        // Show scoring scale
        let scale_title = if self.i18n.lang == "zh-CN" {
            "\n📏 评分标准 (分数越高代码越烂):"
        } else {
            "\n📏 Scoring Scale (higher score = worse code):"
        };

        println!("{}", scale_title.bright_yellow());
        if self.i18n.lang == "zh-CN" {
            println!("  💀 81-100: 糟糕    🔥 61-80: 较差    ⚠️ 41-60: 一般");
            println!("  ✅ 21-40: 良好     🌟 0-20: 优秀");
        } else {
            println!("  💀 81-100: Terrible    🔥 61-80: Poor    ⚠️ 41-60: Average");
            println!("  ✅ 21-40: Good         🌟 0-20: Excellent");
        }
    }

    fn calculate_base_score_for_display(
        &self,
        issues: &[CodeIssue],
        scorer: &crate::scoring::CodeScorer,
    ) -> f64 {
        let mut score = 0.0;
        for issue in issues {
            let rule_weight = scorer.rule_weights.get(&issue.rule_name).unwrap_or(&1.0);
            let severity_weight = match issue.severity {
                crate::analyzer::Severity::Nuclear => 10.0,
                crate::analyzer::Severity::Spicy => 5.0,
                crate::analyzer::Severity::Mild => 2.0,
            };
            score += rule_weight * severity_weight;
        }
        score
    }

    fn calculate_density_penalty_for_display(
        &self,
        issue_count: usize,
        file_count: usize,
        total_lines: usize,
    ) -> f64 {
        if total_lines == 0 || file_count == 0 {
            return 0.0;
        }

        let issues_per_1000_lines = (issue_count as f64 / total_lines as f64) * 1000.0;
        let issues_per_file = issue_count as f64 / file_count as f64;

        let density_penalty = match issues_per_1000_lines {
            x if x > 50.0 => 25.0,
            x if x > 30.0 => 15.0,
            x if x > 20.0 => 10.0,
            x if x > 10.0 => 5.0,
            _ => 0.0,
        };

        let file_penalty = match issues_per_file {
            x if x > 20.0 => 15.0,
            x if x > 10.0 => 10.0,
            x if x > 5.0 => 5.0,
            _ => 0.0,
        };

        density_penalty + file_penalty
    }

    fn calculate_severity_penalty_for_display(
        &self,
        distribution: &crate::scoring::SeverityDistribution,
    ) -> f64 {
        let mut penalty = 0.0;

        if distribution.nuclear > 0 {
            penalty += 20.0 + (distribution.nuclear as f64 - 1.0) * 5.0;
        }

        if distribution.spicy > 5 {
            penalty += (distribution.spicy as f64 - 5.0) * 2.0;
        }

        if distribution.mild > 20 {
            penalty += (distribution.mild as f64 - 20.0) * 0.5;
        }

        penalty
    }

    fn print_category_scores(&self, category_scores: &std::collections::HashMap<String, f64>) {
        let title = if self.i18n.lang == "zh-CN" {
            "📋 分类评分详情:"
        } else {
            "📋 Category Scores:"
        };

        println!("{}", title.bright_yellow());

        // Define category display order and info
        let categories = [
            ("naming", "命名规范", "Naming", "🏷️"),
            ("complexity", "复杂度", "Complexity", "🧩"),
            ("duplication", "代码重复", "Duplication", "🔄"),
            ("rust-basics", "Rust基础", "Rust Basics", "🦀"),
            ("advanced-rust", "高级特性", "Advanced Rust", "⚡"),
            ("rust-features", "Rust功能", "Rust Features", "🚀"),
            ("structure", "代码结构", "Code Structure", "🏗️"),
        ];

        for (category_key, zh_name, en_name, icon) in &categories {
            if let Some(score) = category_scores.get(*category_key) {
                let display_name = if self.i18n.lang == "zh-CN" {
                    zh_name
                } else {
                    en_name
                };
                let (status_icon, status_text) = self.get_score_status(*score);

                // basic display
                let score_unit = if self.i18n.lang == "zh-CN" { "分" } else { "" };
                println!(
                    "  {} {} {}{}     {}",
                    status_icon,
                    format!("{icon} {display_name}").bright_white(),
                    format!("{score:.0}").bright_cyan(),
                    score_unit,
                    status_text.bright_black()
                );

                // if score is high (code is bad), add a roast
                if let Some(roast) = self.get_category_roast(category_key, *score) {
                    println!("    💬 {}", roast.bright_yellow().italic());
                }
            }
        }
        println!();
    }

    fn get_score_status(&self, score: f64) -> (&str, &str) {
        // 注意：分数越高代码越烂
        match score as u32 {
            81..=100 => (
                "⚠",
                if self.i18n.lang == "zh-CN" {
                    "糟糕，急需修复"
                } else {
                    "Terrible, urgent fixes needed"
                },
            ),
            61..=80 => (
                "•",
                if self.i18n.lang == "zh-CN" {
                    "较差，建议重构"
                } else {
                    "Poor, refactoring recommended"
                },
            ),
            41..=60 => (
                "○",
                if self.i18n.lang == "zh-CN" {
                    "一般，需要改进"
                } else {
                    "Average, needs improvement"
                },
            ),
            21..=40 => (
                "✓",
                if self.i18n.lang == "zh-CN" {
                    "良好，还有提升空间"
                } else {
                    "Good, room for improvement"
                },
            ),
            _ => (
                "✓✓",
                if self.i18n.lang == "zh-CN" {
                    "优秀，继续保持"
                } else {
                    "Excellent, keep it up"
                },
            ),
        }
    }

    fn get_category_roast(&self, category: &str, score: f64) -> Option<String> {
        // only roast if score is high (code is bad)
        if score < 60.0 {
            return None;
        }

        // 使用新的随机吐槽系统，支持中英文
        let category_name = if self.i18n.lang == "zh-CN" {
            match category {
                "naming" => "命名规范",
                "complexity" => "复杂度", 
                "duplication" => "代码重复",
                "rust-features" => "Rust功能",
                _ => category,
            }
        } else {
            match category {
                "naming" => "Naming",
                "complexity" => "Complexity", 
                "duplication" => "Duplication",
                "rust-features" => "Rust Features",
                _ => category,
            }
        };
        
        // 使用时间戳作为种子，确保每次运行都有不同的吐槽
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let seed = timestamp + (score * 1000.0) as u64;
        let roast_message = self.get_random_roast(category_name, score, seed);
        
        if roast_message.is_empty() {
            None
        } else {
            Some(roast_message)
        }
    }

    fn print_weighted_calculation(
        &self,
        category_scores: &std::collections::HashMap<String, f64>,
        _total_score: f64,
    ) {
        let calc_title = if self.i18n.lang == "zh-CN" {
            "🧮 加权计算:"
        } else {
            "🧮 Weighted Calculation:"
        };

        println!("{}", calc_title.bright_yellow());

        // Show the calculation formula
        let weights = [
            ("naming", 0.25, "命名规范", "Naming"),
            ("complexity", 0.20, "复杂度", "Complexity"),
            ("duplication", 0.15, "代码重复", "Duplication"),
            ("rust-basics", 0.15, "Rust基础", "Rust Basics"),
            ("advanced-rust", 0.10, "高级特性", "Advanced Rust"),
            ("rust-features", 0.10, "Rust功能", "Rust Features"),
            ("structure", 0.05, "代码结构", "Code Structure"),
        ];

        let mut calculation_parts = Vec::new();
        let mut weighted_sum = 0.0;

        for (category_key, weight, _zh_name, _en_name) in &weights {
            if let Some(score) = category_scores.get(*category_key) {
                let weighted_value = score * weight;
                weighted_sum += weighted_value;
                calculation_parts.push(format!("{score:.1}×{weight:.2}"));
            }
        }

        if self.i18n.lang == "zh-CN" {
            println!(
                "  评分计算: ({}) ÷ 1.00 = {}",
                calculation_parts.join(" + ").bright_white(),
                format!("{weighted_sum:.1}").bright_green().bold()
            );
        } else {
            println!(
                "  Score calculation: ({}) ÷ 1.00 = {}",
                calculation_parts.join(" + ").bright_white(),
                format!("{weighted_sum:.1}").bright_green().bold()
            );
        }
    }

    fn print_detailed_base_score_breakdown(
        &self,
        issues: &[CodeIssue],
        scorer: &crate::scoring::CodeScorer,
    ) {
        // Group issues by rule type and calculate scores
        let mut rule_scores: std::collections::HashMap<String, (usize, f64)> =
            std::collections::HashMap::new();

        for issue in issues {
            let rule_weight = scorer.rule_weights.get(&issue.rule_name).unwrap_or(&1.0);
            let severity_weight = match issue.severity {
                crate::analyzer::Severity::Nuclear => 10.0,
                crate::analyzer::Severity::Spicy => 5.0,
                crate::analyzer::Severity::Mild => 2.0,
            };
            let issue_score = rule_weight * severity_weight;

            let entry = rule_scores
                .entry(issue.rule_name.clone())
                .or_insert((0, 0.0));
            entry.0 += 1; // count
            entry.1 += issue_score; // total score
        }

        // Sort by score (highest first)
        let mut sorted_rules: Vec<_> = rule_scores.into_iter().collect();
        sorted_rules.sort_by(|a, b| {
            b.1 .1
                .partial_cmp(&a.1 .1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let breakdown_title = if self.i18n.lang == "zh-CN" {
            "🔍 基础分数详细计算:"
        } else {
            "🔍 Base score detailed calculation:"
        };

        println!("{}", breakdown_title.bright_yellow());

        for (rule_name, (count, total_score)) in sorted_rules.iter().take(10) {
            let rule_weight = scorer.rule_weights.get(rule_name).unwrap_or(&1.0);

            let rule_display = match (self.i18n.lang.as_str(), rule_name.as_str()) {
                ("zh-CN", "terrible-naming") => "糟糕命名",
                ("zh-CN", "single-letter-variable") => "单字母变量",
                ("zh-CN", "deep-nesting") => "深度嵌套",
                ("zh-CN", "code-duplication") => "代码重复",
                ("zh-CN", "long-function") => "超长函数",
                ("zh-CN", "macro-abuse") => "宏滥用",
                (_, "terrible-naming") => "Terrible naming",
                (_, "single-letter-variable") => "Single letter vars",
                (_, "deep-nesting") => "Deep nesting",
                (_, "code-duplication") => "Code duplication",
                (_, "long-function") => "Long function",
                (_, "macro-abuse") => "Macro abuse",
                _ => rule_name,
            };

            if self.i18n.lang == "zh-CN" {
                println!(
                    "  • {} × {} (权重{:.1}) = {}",
                    format!("{count}").cyan(),
                    rule_display.bright_white(),
                    format!("{rule_weight:.1}").yellow(),
                    format!("{total_score:.1}").bright_red()
                );
            } else {
                println!(
                    "  • {} × {} (weight {:.1}) = {}",
                    format!("{count}").cyan(),
                    rule_display.bright_white(),
                    format!("{rule_weight:.1}").yellow(),
                    format!("{total_score:.1}").bright_red()
                );
            }
        }
        println!();
    }

    fn print_footer(&self, _issues: &[CodeIssue]) {
        // Footer without improvement suggestions - suggestions are now only shown with --suggestions flag
        println!();
        if self.i18n.lang == "zh-CN" {
            println!("{}", "继续努力，让代码变得更好！🚀".bright_cyan());
        } else {
            println!("{}", "Keep working to make your code better! 🚀".bright_cyan());
        }
    }

    fn print_top_files(&self, issues: &[CodeIssue]) {
        if self.top_files == 0 {
            return;
        }

        let mut file_issue_counts: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            let file_name = issue
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            *file_issue_counts.entry(file_name).or_insert(0) += 1;
        }

        let mut sorted_files: Vec<_> = file_issue_counts.into_iter().collect();
        sorted_files.sort_by(|a, b| b.1.cmp(&a.1));

        if !sorted_files.is_empty() {
            println!("{}", self.i18n.get("top_files").bright_yellow().bold());
            println!("{}", "─".repeat(50).bright_black());

            for (i, (file_name, count)) in sorted_files.iter().take(self.top_files).enumerate() {
                let rank = format!("{}.", i + 1);
                println!(
                    "   {} {} ({} issues)",
                    rank.bright_white(),
                    file_name.bright_blue(),
                    count.to_string().red()
                );
            }
            println!();
        }
    }

    fn print_detailed_analysis(&self, issues: &[CodeIssue]) {
        println!(
            "{}",
            self.i18n.get("detailed_analysis").bright_magenta().bold()
        );
        println!("{}", "─".repeat(50).bright_black());

        let mut rule_stats: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            *rule_stats.entry(issue.rule_name.clone()).or_insert(0) += 1;
        }

        let rule_descriptions = match self.i18n.lang.as_str() {
            "zh-CN" => [
                ("terrible-naming", "糟糕的变量命名"),
                ("single-letter-variable", "单字母变量"),
                ("deep-nesting", "过度嵌套"),
                ("long-function", "超长函数"),
                ("unwrap-abuse", "unwrap() 滥用"),
                ("unnecessary-clone", "不必要的 clone()"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
            _ => [
                ("terrible-naming", "Terrible variable naming"),
                ("single-letter-variable", "Single letter variables"),
                ("deep-nesting", "Deep nesting"),
                ("long-function", "Long functions"),
                ("unwrap-abuse", "unwrap() abuse"),
                ("unnecessary-clone", "Unnecessary clone()"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        };

        for (rule_name, count) in rule_stats {
            let rule_name_str = rule_name.as_str();
            
            // 获取规则的中文显示名称
            let display_name = if self.i18n.lang == "zh-CN" {
                match rule_name_str {
                    "terrible-naming" => "糟糕的变量命名",
                    "single-letter-variable" => "单字母变量",
                    "deep-nesting" => "过度嵌套",
                    "long-function" => "超长函数",
                    "unwrap-abuse" => "unwrap() 滥用",
                    "unnecessary-clone" => "不必要的 clone()",
                    "panic-abuse" => "panic 滥用",
                    "god-function" => "上帝函数",
                    "magic-number" => "魔法数字",
                    "todo-comment" => "TODO 注释",
                    "println-debugging" => "println 调试",
                    "string-abuse" => "String 滥用",
                    "vec-abuse" => "Vec 滥用",
                    "iterator-abuse" => "迭代器滥用",
                    "match-abuse" => "Match 滥用",
                    "hungarian-notation" => "匈牙利命名法",
                    "abbreviation-abuse" => "过度缩写",
                    "meaningless-naming" => "无意义命名",
                    "commented-code" => "被注释代码",
                    "dead-code" => "死代码",
                    "code-duplication" => "代码重复",
                    "macro-abuse" => "宏滥用",
                    _ => rule_name_str,
                }
            } else {
                rule_descriptions
                    .get(rule_name_str)
                    .unwrap_or(&rule_name_str)
            };

            let issues_text = if self.i18n.lang == "zh-CN" {
                "个问题"
            } else {
                "issues"
            };

            println!(
                "   📌 {}: {} {}",
                display_name.cyan(),
                count.to_string().yellow(),
                issues_text
            );
        }
        println!();
    }

    fn print_markdown_report(&self, issues: &[CodeIssue]) {
        let total = issues.len();
        let nuclear = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Nuclear))
            .count();
        let spicy = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Spicy))
            .count();
        let mild = issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Mild))
            .count();

        println!("# {}", self.i18n.get("title"));
        println!();
        println!("## {}", self.i18n.get("statistics"));
        println!();
        println!("| Severity | Count | Description |");
        println!("| --- | --- | --- |");
        println!(
            "| 🔥 Nuclear | {} | {} |",
            nuclear,
            self.i18n.get("nuclear_issues")
        );
        println!(
            "| 🌶️ Spicy | {} | {} |",
            spicy,
            self.i18n.get("spicy_issues")
        );
        println!("| 😐 Mild | {} | {} |", mild, self.i18n.get("mild_issues"));
        println!(
            "| **Total** | **{}** | **{}** |",
            total,
            self.i18n.get("total")
        );
        println!();

        if self.verbose {
            println!("## {}", self.i18n.get("detailed_analysis"));
            println!();

            let mut rule_stats: HashMap<String, usize> = HashMap::new();
            for issue in issues {
                *rule_stats.entry(issue.rule_name.clone()).or_insert(0) += 1;
            }

            for (rule_name, count) in rule_stats {
                println!("- **{rule_name}**: {count} issues");
            }
            println!();
        }

        println!("## Issues by File");
        println!();

        let mut file_groups: BTreeMap<String, Vec<&CodeIssue>> = BTreeMap::new();
        for issue in issues {
            let file_name = issue
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            file_groups.entry(file_name).or_default().push(issue);
        }

        for (file_name, file_issues) in file_groups {
            println!("### 📁 {file_name}");
            println!();

            let issues_to_show = if self.max_issues_per_file > 0 {
                file_issues
                    .into_iter()
                    .take(self.max_issues_per_file)
                    .collect::<Vec<_>>()
            } else {
                file_issues
            };

            for issue in issues_to_show {
                let severity_icon = match issue.severity {
                    Severity::Nuclear => "💥",
                    Severity::Spicy => "🌶️",
                    Severity::Mild => "😐",
                };

                let messages = self.i18n.get_roast_messages(&issue.rule_name);
                let message = if !messages.is_empty() {
                    messages[issue.line % messages.len()].clone()
                } else {
                    issue.message.clone()
                };

                println!(
                    "- {} **Line {}:{}** - {}",
                    severity_icon, issue.line, issue.column, message
                );
            }
            println!();
        }

        println!("## {}", self.i18n.get("suggestions"));
        println!();

        let rule_names: Vec<String> = issues
            .iter()
            .map(|issue| issue.rule_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let suggestions = self.i18n.get_suggestions(&rule_names);
        for suggestion in suggestions {
            println!("- {suggestion}");
        }
    }

    fn print_issues_enhanced(&self, issues: &[CodeIssue], educational_advisor: Option<&EducationalAdvisor>) {
        let mut file_groups: HashMap<String, Vec<&CodeIssue>> = HashMap::new();

        for issue in issues {
            let file_name = issue
                .file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            file_groups.entry(file_name).or_default().push(issue);
        }

        for (file_name, file_issues) in file_groups {
            println!("{} {}", "📁".bright_blue(), file_name.bright_blue().bold());

            // Group issues by rule type and count them
            let mut rule_groups: BTreeMap<String, Vec<&CodeIssue>> = BTreeMap::new();
            for issue in &file_issues {
                rule_groups
                    .entry(issue.rule_name.clone())
                    .or_default()
                    .push(issue);
            }

            // Sort rule groups by count (most frequent first)
            let mut sorted_rules: Vec<_> = rule_groups.into_iter().collect();
            sorted_rules.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

            // Display grouped issues with counts and examples
            for (rule_name, rule_issues) in sorted_rules {
                let count = rule_issues.len();
                let icon = self.get_rule_icon(&rule_name);
                let translated_name = if self.i18n.lang == "zh-CN" {
                    self.translate_rule_display_name(&rule_name)
                } else {
                    rule_name.replace("-", " ")
                };

                // Show count and some example variable names for naming issues
                if rule_name.contains("naming") {
                    let examples: Vec<String> = rule_issues.iter()
                        .take(5)
                        .filter_map(|issue| {
                            // Extract variable name from message
                            if let Some(start) = issue.message.find("'") {
                                issue.message[start+1..].find("'").map(|end| issue.message[start+1..start+1+end].to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    if !examples.is_empty() {
                        println!("  {} {}: {} ({})", icon, translated_name, count, examples.join(", "));
                    } else {
                        println!("  {icon} {translated_name}: {count}");
                    }
                } else if rule_name.contains("duplication") {
                    // Show instance count for duplication
                    if let Some(first_issue) = rule_issues.first() {
                        if let Some(instances_start) = first_issue.message.find("发现 ") {
                            if let Some(instances_end) = first_issue.message[instances_start..].find(" 个") {
                                let instances_str = &first_issue.message[instances_start+3..instances_start+instances_end];
                                println!("  {icon} {translated_name}: {count} ({instances_str} instances)");
                            } else {
                                println!("  {icon} {translated_name}: {count}");
                            }
                        } else if let Some(instances_start) = first_issue.message.find("Similar code blocks detected: ") {
                            if let Some(instances_end) = first_issue.message[instances_start..].find(" instances") {
                                let instances_str = &first_issue.message[instances_start+30..instances_start+instances_end];
                                println!("  {icon} {translated_name}: {count} ({instances_str} instances)");
                            } else {
                                println!("  {icon} {translated_name}: {count}");
                            }
                        } else {
                            println!("  {icon} {translated_name}: {count}");
                        }
                    } else {
                        println!("  {icon} {translated_name}: {count}");
                    }
                } else {
                    println!("  {icon} {translated_name}: {count}");
                }

                // Show educational advice if requested (only for the first occurrence of each rule)
                if let Some(advisor) = educational_advisor {
                    if let Some(advice) = advisor.get_advice(&rule_name) {
                        self.print_educational_advice(advice);
                    }
                }
            }
            println!();
        }
    }

    fn get_rule_icon(&self, rule_name: &str) -> &'static str {
        match rule_name {
            name if name.contains("naming") => "🏷️",
            name if name.contains("nesting") => "📦",
            name if name.contains("duplication") => "🔄",
            name if name.contains("function") => "⚠️",
            name if name.contains("unwrap") => "🛡️",
            name if name.contains("string") => "📝",
            name if name.contains("println") => "🔍",
            name if name.contains("magic") => "🔢",
            name if name.contains("panic") => "💥",
            name if name.contains("todo") => "📋",
            name if name.contains("import") => "📦",
            name if name.contains("file") => "📄",
            name if name.contains("module") => "🏗️",
            _ => "⚠️",
        }
    }

    fn translate_rule_display_name(&self, rule_name: &str) -> String {
        match rule_name {
            "terrible-naming" => "变量命名问题".to_string(),
            "meaningless-naming" => "无意义命名问题".to_string(),
            "deep-nesting" => "嵌套深度问题".to_string(),
            "duplication" => "代码重复问题".to_string(),
            "code-duplication" => "代码重复问题".to_string(),
            "long-function" => "过长函数".to_string(),
            "god-function" => "上帝函数".to_string(),
            "unwrap-abuse" => "unwrap滥用".to_string(),
            "string-abuse" => "字符串滥用".to_string(),
            "println-debugging" => "println调试".to_string(),
            "magic-number" => "魔法数字".to_string(),
            "panic-abuse" => "panic滥用".to_string(),
            "todo-comment" => "TODO注释".to_string(),
            "file-too-long" => "文件过长".to_string(),
            "unordered-imports" => "导入混乱".to_string(),
            "deep-module-nesting" => "模块嵌套过深".to_string(),
            "macro-abuse" => "宏滥用".to_string(),
            "abbreviation-abuse" => "缩写滥用".to_string(),
            "hungarian-notation" => "匈牙利命名法".to_string(),
            "single-letter-variable" => "单字母变量".to_string(),
            "iterator-abuse" => "迭代器滥用".to_string(),
            "match-abuse" => "match滥用".to_string(),
            "vec-abuse" => "Vec滥用".to_string(),
            "dead-code" => "死代码".to_string(),
            "commented-code" => "注释代码".to_string(),
            "unnecessary-clone" => "不必要克隆".to_string(),
            "channel-abuse" => "通道滥用".to_string(),
            "async-abuse" => "异步滥用".to_string(),
            "dyn-trait-abuse" => "动态trait滥用".to_string(),
            "unsafe-abuse" => "unsafe滥用".to_string(),
            "ffi-abuse" => "FFI滥用".to_string(),
            "pattern-matching-abuse" => "模式匹配滥用".to_string(),
            "reference-abuse" => "引用滥用".to_string(),
            "box-abuse" => "Box滥用".to_string(),
            "slice-abuse" => "切片滥用".to_string(),
            "module-complexity" => "模块复杂度".to_string(),
            _ => rule_name.replace("-", " "),
        }
    }

    fn print_educational_advice(&self, advice: &crate::educational::EducationalAdvice) {
        println!("    {}", "💡 Educational Advice:".bright_yellow().bold());
        println!("    {}", format!("Why it's bad: {}", advice.why_bad).yellow());
        println!("    {}", format!("How to fix: {}", advice.how_to_fix).green());
        
        if let Some(ref bad_example) = advice.example_bad {
            println!("    {}", "❌ Bad example:".red());
            println!("    {}", format!("    {bad_example}").bright_black());
        }
        
        if let Some(ref good_example) = advice.example_good {
            println!("    {}", "✅ Good example:".green());
            println!("    {}", format!("    {good_example}").bright_black());
        }
        
        if let Some(ref tip) = advice.best_practice_tip {
            println!("    {}", format!("💡 Tip: {tip}").cyan());
        }
        
        if let Some(ref link) = advice.rust_docs_link {
            println!("    {}", format!("📚 Learn more: {link}").blue());
        }
        println!();
    }

    fn print_hall_of_shame(&self, hall_of_shame: &HallOfShame) {
        let stats = hall_of_shame.generate_shame_report();
        
        println!();
        if self.i18n.lang == "zh-CN" {
            println!("{}", "🏆 问题最多的文件".bright_red().bold());
        } else {
            println!("{}", "🏆 Hall of Shame - Worst Offenders".bright_red().bold());
        }
        println!("{}", "─".repeat(60).bright_black());
        
        if stats.hall_of_shame.is_empty() {
            if self.i18n.lang == "zh-CN" {
                println!("🎉 没有文件进入耻辱榜！干得好！");
            } else {
                println!("🎉 No files in the hall of shame! Great job!");
            }
            return;
        }

        if self.i18n.lang == "zh-CN" {
            println!("📊 项目统计:");
            println!("   分析文件数: {}", stats.total_files_analyzed.to_string().cyan());
            println!("   总问题数: {}", stats.total_issues.to_string().red());
            println!("   垃圾密度: {:.2} 问题/1000行", stats.garbage_density.to_string().yellow());
        } else {
            println!("📊 Project Statistics:");
            println!("   Files analyzed: {}", stats.total_files_analyzed.to_string().cyan());
            println!("   Total issues: {}", stats.total_issues.to_string().red());
            println!("   Garbage density: {:.2} issues/1000 lines", stats.garbage_density.to_string().yellow());
        }
        println!();

        if self.i18n.lang == "zh-CN" {
            println!("🗑️ 问题最多的 {} 个文件:", stats.hall_of_shame.len().min(5));
        } else {
            println!("🗑️ Top {} Worst Files:", stats.hall_of_shame.len().min(5));
        }
        
        for (i, entry) in stats.hall_of_shame.iter().take(5).enumerate() {
            let file_name = entry.file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            
            if self.i18n.lang == "zh-CN" {
                println!("   {}. {} ({} 个问题)", 
                    (i + 1).to_string().bright_white(),
                    file_name.bright_red().bold(),
                    entry.total_issues.to_string().red()
                );
                
                println!("      💥 严重: {}, 🌶️ 中等: {}, 😐 轻微: {}", 
                    entry.nuclear_issues.to_string().red(),
                    entry.spicy_issues.to_string().yellow(),
                    entry.mild_issues.to_string().blue()
                );
            } else {
                println!("   {}. {} ({} issues)", 
                    (i + 1).to_string().bright_white(),
                    file_name.bright_red().bold(),
                    entry.total_issues.to_string().red()
                );
                
                println!("      💥 Nuclear: {}, 🌶️ Spicy: {}, 😐 Mild: {}", 
                    entry.nuclear_issues.to_string().red(),
                    entry.spicy_issues.to_string().yellow(),
                    entry.mild_issues.to_string().blue()
                );
            }
        }
        println!();

        if self.i18n.lang == "zh-CN" {
            println!("🔥 最常见问题:");
        } else {
            println!("🔥 Most Common Issues:");
        }
        
        for (i, pattern) in stats.most_common_patterns.iter().take(5).enumerate() {
            if self.i18n.lang == "zh-CN" {
                println!("   {}. {} ({} 次出现)", 
                    (i + 1).to_string().bright_white(),
                    self.translate_rule_name(&pattern.rule_name).bright_yellow(),
                    pattern.count.to_string().red()
                );
            } else {
                println!("   {}. {} ({} occurrences)", 
                    (i + 1).to_string().bright_white(),
                    pattern.rule_name.bright_yellow(),
                    pattern.count.to_string().red()
                );
            }
        }
        println!();
    }

    fn translate_rule_name(&self, rule_name: &str) -> String {
        if self.i18n.lang != "zh-CN" {
            return rule_name.to_string();
        }
        
        match rule_name {
            "terrible-naming" => "糟糕命名".to_string(),
            "meaningless-naming" => "无意义命名".to_string(),
            "magic-number" => "魔法数字".to_string(),
            "macro-abuse" => "宏滥用".to_string(),
            "deep-nesting" => "深层嵌套".to_string(),
            "unwrap-abuse" => "unwrap滥用".to_string(),
            "string-abuse" => "字符串滥用".to_string(),
            "println-debugging" => "println调试".to_string(),
            "long-function" => "过长函数".to_string(),
            "god-function" => "上帝函数".to_string(),
            "file-too-long" => "文件过长".to_string(),
            "unordered-imports" => "导入混乱".to_string(),
            "deep-module-nesting" => "模块嵌套过深".to_string(),
            _ => rule_name.to_string(),
        }
    }

    fn print_improvement_suggestions(&self, hall_of_shame: &HallOfShame) {
        let suggestions = hall_of_shame.get_improvement_suggestions(&self.i18n.lang);
        
        println!();
        if self.i18n.lang == "zh-CN" {
            println!("{}", "💡 改进建议".bright_green().bold());
        } else {
            println!("{}", "💡 Improvement Suggestions".bright_green().bold());
        }
        println!("{}", "─".repeat(50).bright_black());
        
        for suggestion in suggestions {
            println!("   {}", suggestion.green());
        }
        println!();
    }

    fn print_markdown_report_enhanced(
        &self, 
        issues: &[CodeIssue], 
        quality_score: &CodeQualityScore,
        educational_advisor: Option<&EducationalAdvisor>,
        hall_of_shame: Option<&HallOfShame>,
        show_suggestions: bool,
    ) {
        // First print the regular markdown report
        self.print_markdown_report(issues);
        
        // Add quality score section
        println!("## 🏆 Code Quality Score");
        println!();
        println!("**Score**: {:.1}/100 {}", quality_score.total_score, quality_score.quality_level.emoji());
        println!("**Level**: {}", quality_score.quality_level.description(&self.i18n.lang));
        println!();
        
        // Print hall of shame in markdown if requested
        if let Some(shame) = hall_of_shame {
            self.print_markdown_hall_of_shame(shame);
        }
        
        // Print improvement suggestions in markdown if requested
        if show_suggestions {
            if let Some(shame) = hall_of_shame {
                self.print_markdown_improvement_suggestions(shame);
            }
        }
        
        // Print educational content in markdown if requested
        if educational_advisor.is_some() {
            self.print_markdown_educational_section(issues, educational_advisor);
        }
    }

    fn print_markdown_hall_of_shame(&self, hall_of_shame: &HallOfShame) {
        let stats = hall_of_shame.generate_shame_report();
        
        println!("## 🏆 Hall of Shame");
        println!();
        
        if stats.hall_of_shame.is_empty() {
            println!("🎉 No files in the hall of shame! Great job!");
            return;
        }

        println!("### 📊 Project Statistics");
        println!();
        println!("| Metric | Value |");
        println!("| --- | --- |");
        println!("| Files analyzed | {} |", stats.total_files_analyzed);
        println!("| Total issues | {} |", stats.total_issues);
        println!("| Garbage density | {:.2} issues/1000 lines |", stats.garbage_density);
        println!();

        println!("### 🗑️ Worst Files");
        println!();
        println!("| Rank | File | Shame Score | Nuclear | Spicy | Mild |");
        println!("| --- | --- | --- | --- | --- | --- |");
        
        for (i, entry) in stats.hall_of_shame.iter().take(5).enumerate() {
            let file_name = entry.file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            
            println!("| {} | {} | {:.1} | {} | {} | {} |",
                i + 1,
                file_name,
                entry.shame_score,
                entry.nuclear_issues,
                entry.spicy_issues,
                entry.mild_issues
            );
        }
        println!();

        println!("### 🔥 Most Common Issues");
        println!();
        println!("| Rank | Issue Type | Count |");
        println!("| --- | --- | --- |");
        
        for (i, pattern) in stats.most_common_patterns.iter().take(5).enumerate() {
            println!("| {} | {} | {} |",
                i + 1,
                pattern.rule_name,
                pattern.count
            );
        }
        println!();
    }

    fn print_markdown_improvement_suggestions(&self, hall_of_shame: &HallOfShame) {
        let suggestions = hall_of_shame.get_improvement_suggestions(&self.i18n.lang);
        
        if self.i18n.lang == "zh-CN" {
            println!("## 💡 改进建议");
        } else {
            println!("## 💡 Improvement Suggestions");
        }
        println!();
        
        for suggestion in suggestions {
            println!("- {suggestion}");
        }
        println!();
    }

    fn print_markdown_educational_section(&self, issues: &[CodeIssue], educational_advisor: Option<&EducationalAdvisor>) {
        if let Some(advisor) = educational_advisor {
            println!("## 📚 Educational Content");
            println!();
            
            // Get unique rule names
            let mut rule_names: std::collections::HashSet<String> = std::collections::HashSet::new();
            for issue in issues {
                rule_names.insert(issue.rule_name.clone());
            }
            
            for rule_name in rule_names {
                if let Some(advice) = advisor.get_advice(&rule_name) {
                    println!("### 📖 {}", rule_name.replace("-", " "));
                    println!();
                    println!("**Why it's problematic:**");
                    println!("{}", advice.why_bad);
                    println!();
                    println!("**How to fix:**");
                    println!("{}", advice.how_to_fix);
                    println!();
                    
                    if let Some(ref bad_example) = advice.example_bad {
                        println!("**❌ Bad example:**");
                        println!("```rust");
                        println!("{bad_example}");
                        println!("```");
                        println!();
                    }
                    
                    if let Some(ref good_example) = advice.example_good {
                        println!("**✅ Good example:**");
                        println!("```rust");
                        println!("{good_example}");
                        println!("```");
                        println!();
                    }
                    
                    if let Some(ref tip) = advice.best_practice_tip {
                        println!("**💡 Best Practice Tip:**");
                        println!("{tip}");
                        println!();
                    }
                    
                    if let Some(ref link) = advice.rust_docs_link {
                        println!("**📚 Learn More:**");
                        println!("[Rust Documentation]({link})");
                        println!();
                    }
                }
            }
        }
    }
}
