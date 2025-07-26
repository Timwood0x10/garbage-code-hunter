use std::path::Path;
use syn::{visit::Visit, File, Ident};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;
use crate::utils::get_position;

/// 检测无意义的占位符命名：foo, bar, baz, qux, test, temp 等
pub struct MeaninglessNamingRule;

impl Rule for MeaninglessNamingRule {
    fn name(&self) -> &'static str {
        "meaningless-naming"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = MeaninglessNamingVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// 检测过时的匈牙利命名法：strName, intCount, bIsValid 等
pub struct HungarianNotationRule;

impl Rule for HungarianNotationRule {
    fn name(&self) -> &'static str {
        "hungarian-notation"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = HungarianNotationVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

/// 检测过度缩写：mgr, ctrl, btn, usr, pwd 等
pub struct AbbreviationAbuseRule;

impl Rule for AbbreviationAbuseRule {
    fn name(&self) -> &'static str {
        "abbreviation-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = AbbreviationAbuseVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

// ============================================================================
// Visitor 实现
// ============================================================================

struct MeaninglessNamingVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl MeaninglessNamingVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn is_meaningless_name(&self, name: &str) -> bool {
        let meaningless_names = [
            // 经典占位符
            "foo", "bar", "baz", "qux", "quux", "quuz",
            // 无意义的通用词
            "data", "info", "obj", "item", "thing", "stuff", "value",
            "temp", "tmp", "test", "example", "sample",
            // 管理器后缀滥用
            "manager", "handler", "processor", "controller",
            // 中文拼音（常见的）
            "yonghu", "mima", "denglu", "zhuce", "shuju",
        ];
        
        let name_lower = name.to_lowercase();
        meaningless_names.iter().any(|&bad_name| name_lower == bad_name)
    }

    fn create_issue(&self, name: &str, line: usize, column: usize) -> CodeIssue {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("变量名 '{}' 比我的网名还随意", name),
                format!("'{}' 这个名字，是从字典里随机选的吗？", name),
                format!("用 '{}' 做变量名？你是想让下一个维护代码的人猜谜吗？", name),
                format!("'{}' 这个名字毫无意义，就像我的人生一样", name),
                format!("看到 '{}' 这个变量名，我的智商受到了侮辱", name),
            ]
        } else {
            vec![
                format!("Variable name '{}' is more meaningless than my existence", name),
                format!("'{}' - did you pick this name with your eyes closed?", name),
                format!("Using '{}' as a variable name? Are you playing charades with future developers?", name),
                format!("'{}' tells me nothing about what this variable does", name),
                format!("The name '{}' is as helpful as a chocolate teapot", name),
            ]
        };

        let severity = if ["foo", "bar", "baz", "data", "temp"].contains(&name) {
            Severity::Spicy
        } else {
            Severity::Mild
        };

        CodeIssue {
            file_path: self.file_path.clone(),
            line,
            column,
            rule_name: "meaningless-naming".to_string(),
            message: messages[self.issues.len() % messages.len()].clone(),
            severity,
            roast_level: RoastLevel::Sarcastic,
        }
    }
}

impl<'ast> Visit<'ast> for MeaninglessNamingVisitor {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        let name = ident.to_string();
        if self.is_meaningless_name(&name) {
            let (line, column) = get_position(ident);
            self.issues.push(self.create_issue(&name, line, column));
        }
        syn::visit::visit_ident(self, ident);
    }
}

// ============================================================================
// 匈牙利命名法检测
// ============================================================================

struct HungarianNotationVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl HungarianNotationVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn is_hungarian_notation(&self, name: &str) -> bool {
        let hungarian_prefixes = [
            // 类型前缀
            "str", "int", "bool", "float", "double", "char",
            "arr", "vec", "list", "map", "set",
            // 作用域前缀
            "g_", "m_", "s_", "p_",
            // 其他常见前缀
            "b", "n", "sz", "lp", "dw",
        ];

        // 检查是否以匈牙利前缀开头
        for prefix in hungarian_prefixes {
            if name.starts_with(prefix) && name.len() > prefix.len() {
                // 检查前缀后是否跟着大写字母（驼峰命名）
                if let Some(next_char) = name.chars().nth(prefix.len()) {
                    if next_char.is_uppercase() {
                        return true;
                    }
                }
                // 检查下划线分隔的情况
                if name.starts_with(&format!("{}_", prefix)) {
                    return true;
                }
            }
        }

        false
    }

    fn create_issue(&self, name: &str, line: usize, column: usize) -> CodeIssue {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("'{}' 使用了匈牙利命名法？这不是1990年代了", name),
                format!("看到 '{}' 我仿佛回到了 C++ 的石器时代", name),
                format!("'{}' 这种命名方式已经过时了，就像我的发型一样", name),
                format!("匈牙利命名法 '{}'？Rust 编译器已经帮你检查类型了", name),
                format!("'{}' 让我想起了那些痛苦的 C++ 岁月", name),
            ]
        } else {
            vec![
                format!("'{}' uses Hungarian notation? This isn't the 1990s anymore", name),
                format!("Seeing '{}' makes me nostalgic for the dark ages of C++", name),
                format!("'{}' - Hungarian notation is as outdated as my haircut", name),
                format!("Hungarian notation '{}'? Rust's type system has got you covered", name),
                format!("'{}' reminds me of painful C++ memories", name),
            ]
        };

        CodeIssue {
            file_path: self.file_path.clone(),
            line,
            column,
            rule_name: "hungarian-notation".to_string(),
            message: messages[self.issues.len() % messages.len()].clone(),
            severity: Severity::Mild,
            roast_level: RoastLevel::Sarcastic,
        }
    }
}

impl<'ast> Visit<'ast> for HungarianNotationVisitor {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        let name = ident.to_string();
        if self.is_hungarian_notation(&name) {
            let (line, column) = get_position(ident);
            self.issues.push(self.create_issue(&name, line, column));
        }
        syn::visit::visit_ident(self, ident);
    }
}

// ============================================================================
// 过度缩写检测
// ============================================================================

struct AbbreviationAbuseVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    lang: String,
}

impl AbbreviationAbuseVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            lang: lang.to_string(),
        }
    }

    fn is_bad_abbreviation(&self, name: &str) -> Option<&'static str> {
        let bad_abbreviations = [
            // 管理相关
            ("mgr", "manager"),
            ("mngr", "manager"),
            ("ctrl", "controller"),
            ("proc", "processor"),
            ("hdlr", "handler"),
            
            // 用户相关
            ("usr", "user"),
            ("pwd", "password"),
            ("auth", "authentication"),
            ("cfg", "config"),
            ("prefs", "preferences"),
            
            // 界面相关
            ("btn", "button"),
            ("lbl", "label"),
            ("txt", "text"),
            ("img", "image"),
            ("pic", "picture"),
            
            // 数据相关
            ("db", "database"),
            ("tbl", "table"),
            ("col", "column"),
            ("idx", "index"),
            ("cnt", "count"),
            
            // 其他常见缩写
            ("calc", "calculate"),
            ("init", "initialize"),
            ("exec", "execute"),
            ("impl", "implementation"),
            ("util", "utility"),
        ];

        let name_lower = name.to_lowercase();
        for (abbrev, full) in bad_abbreviations {
            if name_lower == abbrev || name_lower.starts_with(&format!("{}_", abbrev)) {
                return Some(full);
            }
        }
        None
    }

    fn create_issue(&self, name: &str, suggestion: &str, line: usize, column: usize) -> CodeIssue {
        let messages = if self.lang == "zh-CN" {
            vec![
                format!("'{}' 缩写得太狠了，建议用 '{}'", name, suggestion),
                format!("看到 '{}' 我需要解密，不如直接用 '{}'", name, suggestion),
                format!("'{}' 这个缩写让我想起了发电报的年代，用 '{}' 吧", name, suggestion),
                format!("'{}' 省了几个字母，却让代码可读性大打折扣，试试 '{}'", name, suggestion),
                format!("缩写 '{}' 就像密码一样难懂，'{}'不香吗？", name, suggestion),
            ]
        } else {
            vec![
                format!("'{}' is too abbreviated, consider '{}'", name, suggestion),
                format!("Seeing '{}' makes me feel like I'm decoding, just use '{}'", name, suggestion),
                format!("'{}' reminds me of telegraph era, try '{}'", name, suggestion),
                format!("'{}' saves a few letters but kills readability, use '{}'", name, suggestion),
                format!("Abbreviation '{}' is cryptic, isn't '{}' better?", name, suggestion),
            ]
        };

        CodeIssue {
            file_path: self.file_path.clone(),
            line,
            column,
            rule_name: "abbreviation-abuse".to_string(),
            message: messages[self.issues.len() % messages.len()].clone(),
            severity: Severity::Mild,
            roast_level: RoastLevel::Gentle,
        }
    }
}

impl<'ast> Visit<'ast> for AbbreviationAbuseVisitor {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        let name = ident.to_string();
        if let Some(suggestion) = self.is_bad_abbreviation(&name) {
            let (line, column) = get_position(ident);
            self.issues.push(self.create_issue(&name, suggestion, line, column));
        }
        syn::visit::visit_ident(self, ident);
    }
}