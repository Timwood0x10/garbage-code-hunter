use std::path::Path;
use syn::{visit::Visit, File, ItemMod, ItemUse};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;

/// Detects files that are too long (>1000 lines)
pub struct FileStructureRule;

impl Rule for FileStructureRule {
    fn name(&self) -> &'static str {
        "file-structure"
    }

    fn check(
        &self,
        file_path: &Path,
        _syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let line_count = content.lines().count();

        if line_count > 1000 {
            let messages_zh = [
                "这个文件比我的毕业论文还长！建议拆分成多个模块 📚",
                "文件长度突破天际！是想创造吉尼斯纪录吗？ 🚀",
                "这么长的文件，建议配个目录和索引 📖",
                "文件行数比我一年的代码还多！ 📈",
                "这个文件需要电梯才能到底部 🏢",
                "建议把这个巨型文件拆分成几个小文件，拯救一下可读性 🆘",
            ];

            let messages_en = [
                "This file is longer than my thesis! Consider splitting into modules 📚",
                "File length has reached the stratosphere! Going for a world record? 🚀",
                "Such a long file needs a table of contents and index 📖",
                "More lines than I write in a year! 📈",
                "This file needs an elevator to reach the bottom 🏢",
                "Please split this monster file into smaller ones, save the readability 🆘",
            ];

            let messages = if lang == "zh-CN" { &messages_zh } else { &messages_en };
            let message = messages[line_count % messages.len()];

            let severity = if line_count > 2000 {
                Severity::Nuclear
            } else if line_count > 1500 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line: 1,
                column: 1,
                rule_name: "file-too-long".to_string(),
                message: format!("{message} ({line_count}行)"),
                severity,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        issues
    }
}

/// Detects chaotic import order
pub struct ImportChaosRule;

impl Rule for ImportChaosRule {
    fn name(&self) -> &'static str {
        "import-chaos"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = ImportChaosVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);

        // Check for duplicate use statements
        let use_lines: Vec<&str> = content.lines()
            .filter(|line| line.trim().starts_with("use "))
            .collect();

        if use_lines.len() > 1 {
            let mut sorted_uses = use_lines.clone();
            sorted_uses.sort();
            
            if use_lines != sorted_uses {
                visitor.add_unordered_imports_issue();
            }

            // Check for duplicate imports
            let mut seen_imports = std::collections::HashSet::new();
            for use_line in &use_lines {
                if !seen_imports.insert(use_line) {
                    visitor.add_duplicate_import_issue();
                }
            }
        }

        visitor.issues
    }
}

/// Detects overly deep module nesting
pub struct ModuleNestingRule;

impl Rule for ModuleNestingRule {
    fn name(&self) -> &'static str {
        "module-nesting"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = ModuleNestingVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

struct ImportChaosVisitor {
    file_path: std::path::PathBuf,
    lang: String,
    issues: Vec<CodeIssue>,
    use_count: usize,
}

impl ImportChaosVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            lang: lang.to_string(),
            issues: Vec::new(),
            use_count: 0,
        }
    }

    fn add_unordered_imports_issue(&mut self) {
        let messages_zh = [
            "import 顺序比我的房间还乱！建议按字母顺序排列 🔤",
            "这些 use 语句的顺序让我想起了洗牌后的扑克牌 🃏",
            "import 排序混乱，建议使用 rustfmt 整理一下 🧹",
            "use 语句顺序比我的作息时间还乱 ⏰",
        ];

        let messages_en = [
            "Import order is messier than my room! Consider alphabetical sorting 🔤",
            "These use statements remind me of shuffled playing cards 🃏",
            "Import sorting is chaotic, consider using rustfmt 🧹",
            "Use statement order is more chaotic than my sleep schedule ⏰",
        ];

        let messages = if self.lang == "zh-CN" { &messages_zh } else { &messages_en };
        let message = messages[self.issues.len() % messages.len()];

        self.issues.push(CodeIssue {
            file_path: self.file_path.clone(),
            line: 1,
            column: 1,
            rule_name: "unordered-imports".to_string(),
            message: message.to_string(),
            severity: Severity::Mild,
            roast_level: RoastLevel::Sarcastic,
        });
    }

    fn add_duplicate_import_issue(&mut self) {
        let messages_zh = [
            "重复的 import！是想强调重要性吗？ 🔄",
            "同样的 use 语句出现了多次，建议去重 🗑️",
            "重复 import 比我重复的话还多 💬",
        ];

        let messages_en = [
            "Duplicate imports! Trying to emphasize importance? 🔄",
            "Same use statement appears multiple times, consider deduplication 🗑️",
            "More duplicate imports than my repeated words 💬",
        ];

        let messages = if self.lang == "zh-CN" { &messages_zh } else { &messages_en };
        let message = messages[self.issues.len() % messages.len()];

        self.issues.push(CodeIssue {
            file_path: self.file_path.clone(),
            line: 1,
            column: 1,
            rule_name: "duplicate-imports".to_string(),
            message: message.to_string(),
            severity: Severity::Mild,
            roast_level: RoastLevel::Sarcastic,
        });
    }
}

impl<'ast> Visit<'ast> for ImportChaosVisitor {
    fn visit_item_use(&mut self, use_item: &'ast ItemUse) {
        self.use_count += 1;
        syn::visit::visit_item_use(self, use_item);
    }
}

struct ModuleNestingVisitor {
    file_path: std::path::PathBuf,
    lang: String,
    issues: Vec<CodeIssue>,
    nesting_depth: usize,
    max_depth: usize,
}

impl ModuleNestingVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            lang: lang.to_string(),
            issues: Vec::new(),
            nesting_depth: 0,
            max_depth: 0,
        }
    }

    fn check_nesting_depth(&mut self) {
        if self.nesting_depth > 3 {
            let messages_zh = [
                "模块嵌套比俄罗斯套娃还深！建议扁平化结构 🪆",
                "这个嵌套深度需要GPS导航才能找到出口 🗺️",
                "模块嵌套层数比我的心理防线还多 🏰",
                "建议重新设计模块结构，当前嵌套过深 📐",
            ];

            let messages_en = [
                "Module nesting deeper than Russian dolls! Consider flattening 🪆",
                "This nesting depth needs GPS navigation to find the exit 🗺️",
                "More module nesting layers than my psychological defenses 🏰",
                "Consider redesigning module structure, current nesting too deep 📐",
            ];

            let messages = if self.lang == "zh-CN" { &messages_zh } else { &messages_en };
            let message = messages[self.issues.len() % messages.len()];

            let severity = if self.nesting_depth > 5 {
                Severity::Spicy
            } else {
                Severity::Mild
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "deep-module-nesting".to_string(),
                message: format!("{} (深度: {})", message, self.nesting_depth),
                severity,
                roast_level: RoastLevel::Sarcastic,
            });
        }
    }
}

impl<'ast> Visit<'ast> for ModuleNestingVisitor {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.nesting_depth += 1;
        self.max_depth = self.max_depth.max(self.nesting_depth);
        
        self.check_nesting_depth();
        
        syn::visit::visit_item_mod(self, module);
        self.nesting_depth -= 1;
    }
}