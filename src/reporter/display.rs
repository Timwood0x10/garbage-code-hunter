use colored::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::analyzer::{CodeIssue, Severity};
use crate::scoring::CodeQualityScore;

use super::Reporter;

impl Reporter {
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
                    if score >= 16.0 {
                        vec![
                            "恭喜！你成功让变量名比注释还难懂 🏆",
                            "这些变量名是用随机字符生成器起的吗？ 🎲",
                            "变量命名水平堪比密码设置 🔐",
                            "看到这些变量名，我想起了古代象形文字 📜",
                            "变量名比我的人生还迷茫 😵‍💫",
                            "这命名风格很有'艺术感' 🎨",
                            "变量名的创意程度超越了我的理解 🚀",
                        ]
                    } else if score >= 12.0 {
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
                    if score >= 16.0 {
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
                    } else if score >= 12.0 {
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
                    if score >= 16.0 {
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
                    } else if score >= 12.0 {
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
                "代码异味" => {
                    if score >= 16.0 {
                        vec![
                            "magic number 比星座还多 ✨",
                            "println! 调试大法好啊 🖨️",
                            "注释掉的代码比活的还多 🧟",
                            "unwrap() 用得比呼吸还自然 😅",
                            "这代码的味道隔着屏幕都能闻到 👃",
                            "dead code 比墓地还安静 🪦",
                            "代码异味已经变成了代码毒气 ☣️",
                        ]
                    } else if score >= 12.0 {
                        vec![
                            "代码有些异味，建议清理 🧹",
                            "magic number 需要提取为常量 🔢",
                            "有些代码可以精简一下 ✂️",
                            "代码整洁度需要提升 🧼",
                        ]
                    } else {
                        vec!["代码异味不多 👍", "代码还算干净 ✅"]
                    }
                }
                "学生代码" => {
                    if score >= 16.0 {
                        vec![
                            "println! 调试大法重出江湖 🖨️",
                            "TODO 注释比代码还多 📝",
                            "panic! 用得这么随意，像极了期末赶作业 😱",
                            "这代码散发着浓浓的作业气息 📚",
                            "调试完记得删 println 啊亲 😅",
                        ]
                    } else if score >= 12.0 {
                        vec![
                            "有些 TODO 需要处理 📝",
                            "调试代码可以清理一下 🧹",
                            "代码可以更专业一些 💼",
                        ]
                    } else {
                        vec!["代码挺专业的 👍", "没有学生代码的味道 ✅"]
                    }
                }
                _ => vec!["代码需要改进 🔧"],
            }
        } else {
            // English version roasts
            match category {
                "Naming" => {
                    if score >= 16.0 {
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
                    } else if score >= 12.0 {
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
                        vec![
                            "Variable naming is decent 👍",
                            "Naming style is acceptable ✅",
                        ]
                    }
                }
                "Complexity" => {
                    if score >= 16.0 {
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
                    } else if score >= 12.0 {
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
                        vec![
                            "Code structure is fairly clear 👌",
                            "Complexity is well controlled ✅",
                        ]
                    }
                }
                "Duplication" => {
                    if score >= 16.0 {
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
                    } else if score >= 12.0 {
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
                        vec![
                            "Code duplication is well controlled 👍",
                            "Duplication within acceptable range ✅",
                        ]
                    }
                }
                "Code Smells" => {
                    if score >= 16.0 {
                        vec![
                            "More magic numbers than a wizard's spellbook ✨",
                            "println! debugging is not a lifestyle 🖨️",
                            "Commented-out code outnumbers living code 🧟",
                            "unwrap() used more naturally than breathing 😅",
                            "This code smells through the screen 👃",
                            "Dead code quieter than a graveyard 🪦",
                            "Code smell has become code toxic gas ☣️",
                        ]
                    } else if score >= 12.0 {
                        vec![
                            "Some code smells, consider cleaning 🧹",
                            "Magic numbers should be constants 🔢",
                            "Some code could be trimmed ✂️",
                            "Code cleanliness needs improvement 🧼",
                        ]
                    } else {
                        vec!["Not many code smells 👍", "Code is fairly clean ✅"]
                    }
                }
                "Student Code" => {
                    if score >= 16.0 {
                        vec![
                            "println! debugging makes a comeback 🖨️",
                            "More TODO comments than actual code 📝",
                            "panic! used casually like a deadline rush 😱",
                            "This code radiates homework energy 📚",
                            "Remember to remove println after debugging 😅",
                        ]
                    } else if score >= 12.0 {
                        vec![
                            "Some TODOs need attention 📝",
                            "Debug code could use cleanup 🧹",
                            "Code could be more professional 💼",
                        ]
                    } else {
                        vec!["Code looks professional 👍", "No student code vibes ✅"]
                    }
                }
                _ => vec!["Code needs improvement 🔧"],
            }
        }
    }

    pub(super) fn print_quality_score(&self, quality_score: &CodeQualityScore) {
        let title = match self.i18n.lang.as_str() {
            "zh-CN" => "🏆 代码质量评分",
            _ => "🏆 Code Quality Score",
        };
        println!("{}", title.bright_yellow().bold());
        println!("{}", "─".repeat(50).bright_black());

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

        // Display category scores (if any)
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

    pub(super) fn print_summary_with_score(
        &self,
        issues: &[CodeIssue],
        quality_score: &CodeQualityScore,
    ) {
        // Print detailed scoring breakdown
        self.print_scoring_breakdown(issues, quality_score);

        println!("{}", self.i18n.get("summary").bright_white().bold());
        println!("{}", "─".repeat(50).bright_black());

        // Display scoring summary
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

        // Show category breakdown by severity count
        self.print_category_scores(&quality_score.category_scores, _issues);

        // Show weighted calculation
        self.print_weighted_calculation(quality_score);

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

    fn print_category_scores(
        &self,
        _category_scores: &std::collections::HashMap<String, f64>,
        issues: &[CodeIssue],
    ) {
        let title = if self.i18n.lang == "zh-CN" {
            "📋 分类问题统计:"
        } else {
            "📋 Issues by Category:"
        };

        println!("{}", title.bright_yellow());

        // Category→rule mapping (same as scoring.rs build_categories)
        let categories: [(&str, &str, &str, &str, Vec<&str>); 5] = [
            (
                "naming",
                "命名规范",
                "Naming",
                "🏷️",
                vec![
                    "terrible-naming",
                    "single-letter-variable",
                    "meaningless-naming",
                    "hungarian-notation",
                    "abbreviation-abuse",
                    "c-naming",
                ],
            ),
            (
                "complexity",
                "复杂度",
                "Complexity",
                "🧩",
                vec![
                    "deep-nesting",
                    "long-function",
                    "god-function",
                    "cyclomatic-complexity",
                    "c-nesting",
                    "c-long-function",
                    "complex-closure",
                ],
            ),
            (
                "duplication",
                "代码重复",
                "Duplication",
                "🔄",
                vec!["code-duplication", "cross-file-duplication"],
            ),
            (
                "code-smells",
                "代码异味",
                "Code Smells",
                "⚠️",
                vec![
                    "commented-code",
                    "dead-code",
                    "file-too-long",
                    "unwrap-abuse",
                    "unnecessary-clone",
                    "string-abuse",
                    "vec-abuse",
                    "macro-abuse",
                    "box-abuse",
                    "slice-abuse",
                    "reference-abuse",
                    "module-complexity",
                    "pattern-matching-abuse",
                    "duplicate-imports",
                    "lifetime-abuse",
                    "trait-complexity",
                    "generic-abuse",
                    "defer-in-loop",
                    "goroutine-abuse",
                    "global-variable",
                    "bare-rescue",
                    "wildcard-import",
                    "bare-except",
                    "empty-catch",
                    "any-type",
                    "c-include-chaos",
                    "c-magic-number",
                    "c-god-function",
                    "c-commented-code",
                    "c-dead-code",
                    "c-goto-abuse",
                    "c-malloc-leak",
                    "channel-abuse",
                    "async-abuse",
                    "dyn-trait-abuse",
                    "unsafe-abuse",
                    "ffi-abuse",
                    "deep-module-nesting",
                ],
            ),
            (
                "student-code",
                "学生代码",
                "Student Code",
                "📚",
                vec![
                    "println-debugging",
                    "panic-abuse",
                    "todo-comment",
                    "todo-fixme",
                    "todo-bug",
                    "todo-hack",
                ],
            ),
        ];

        for (key, zh_name, en_name, icon, rules) in &categories {
            let cat_issues: Vec<&CodeIssue> = issues
                .iter()
                .filter(|i| rules.contains(&i.rule_name.as_str()))
                .collect();
            if cat_issues.is_empty() {
                continue;
            }
            let n = cat_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Nuclear))
                .count();
            let s = cat_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Spicy))
                .count();
            let m = cat_issues.len() - n - s;

            let display_name = if self.i18n.lang == "zh-CN" {
                zh_name
            } else {
                en_name
            };

            println!(
                "  {} {}  💥{} 🌶️{} 😐{}",
                icon,
                display_name.bright_white(),
                n,
                s,
                m,
            );

            // Add roast for categories with many issues
            let total_score = (n * 3 + s) as f64;
            if total_score > 5.0 {
                if let Some(roast) = self.get_category_roast(key, total_score) {
                    println!("    💬 {}", roast.bright_yellow().italic());
                }
            }
        }
        if !issues.is_empty() {
            println!();
        }
    }

    fn get_category_roast(&self, category: &str, score: f64) -> Option<String> {
        // Category scores are 0-20; roast when score indicates real problems
        if score < 12.0 {
            return None;
        }

        // Use the new random roast system
        let category_name = match (self.i18n.lang.as_str(), category) {
            ("zh-CN", "naming") => "命名规范",
            ("zh-CN", "complexity") => "复杂度",
            ("zh-CN", "duplication") => "代码重复",
            ("zh-CN", "code-smells") => "代码异味",
            ("zh-CN", "student-code") => "学生代码",
            ("en-US", "naming") => "Naming",
            ("en-US", "complexity") => "Complexity",
            ("en-US", "duplication") => "Duplication",
            ("en-US", "code-smells") => "Code Smells",
            ("en-US", "student-code") => "Student Code",
            (_, other) => other,
        };

        // Use timestamp as seed to ensure different roasts each run
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let seed = timestamp + (score * 1000.0) as u64;
        Some(self.get_random_roast(category_name, score, seed))
    }

    fn print_weighted_calculation(&self, quality_score: &CodeQualityScore) {
        let title = if self.i18n.lang == "zh-CN" {
            "🧮 评分明细:"
        } else {
            "🧮 Score Breakdown:"
        };
        println!("{}", title.bright_yellow());

        let n = quality_score.severity_distribution.nuclear;
        let s = quality_score.severity_distribution.spicy;
        let m = quality_score.severity_distribution.mild;

        if self.i18n.lang == "zh-CN" {
            println!(
                "  {} Tier1 Nuclear ({}个): log2({})×8 = {:.1}/40",
                "💥".bright_red(),
                n,
                n + 1,
                quality_score.n_score
            );
            println!(
                "  {} Tier2 Noisy ({}×1.5+{}): {:.1}/60",
                "🌶️".bright_yellow(),
                s,
                m,
                quality_score.d_score
            );
            println!(
                "  {} 总分: {:.1} + {:.1} = {:.1}/100",
                "📊".bright_blue(),
                quality_score.n_score,
                quality_score.d_score,
                quality_score.total_score
            );
        } else {
            println!(
                "  {} Tier1 Nuclear ({}): log2({})×8 = {:.1}/40",
                "💥".bright_red(),
                n,
                n + 1,
                quality_score.n_score
            );
            println!(
                "  {} Tier2 Noisy ({}×1.5+{}): {:.1}/60",
                "🌶️".bright_yellow(),
                s,
                m,
                quality_score.d_score
            );
            println!(
                "  {} Total: {:.1} + {:.1} = {:.1}/100",
                "📊".bright_blue(),
                quality_score.n_score,
                quality_score.d_score,
                quality_score.total_score
            );
        }
    }
}
