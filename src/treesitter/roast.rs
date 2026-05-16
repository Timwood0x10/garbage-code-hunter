/// Roast Buddy Message Generator
///
/// Generates sarcastic yet helpful messages in a "roast buddy" tone.
/// Like a friend who roasts your code but actually wants you to succeed.
///
/// # Design Philosophy
/// - **Tone**: Casual, conversational, slightly sarcastic but constructive
/// - **Structure**: Roast first → Explain why → Suggest improvement
/// - **Style**: Uses emojis, internet slang, and developer humor
/// - **Goal**: Make code review fun while still being useful
///
/// Category of code style issue
#[derive(Debug, Clone, Copy)]
pub enum StyleCategory {
    /// Naming issues (terrible names, single letters, abbreviations)
    Naming,
    /// Structure issues (deep nesting, long functions, god functions)
    Structure,
    /// Duplication issues (copy-paste, boilerplate)
    Duplication,
    /// Comment issues (missing docs, TODO/FIXME, commented-out code)
    Comments,
    /// Static analysis (magic numbers, dead code) — lower priority
    StaticAnalysis,
}

impl StyleCategory {
    /// Returns the emoji prefix for this category
    pub fn emoji(&self) -> &'static str {
        match self {
            StyleCategory::Naming => "🏷️",
            StyleCategory::Structure => "🏗️",
            StyleCategory::Duplication => "📋",
            StyleCategory::Comments => "💬",
            StyleCategory::StaticAnalysis => "🔍",
        }
    }

    /// Returns the friendly name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            StyleCategory::Naming => "命名风格",
            StyleCategory::Structure => "代码结构",
            StyleCategory::Duplication => "重复代码",
            StyleCategory::Comments => "注释文档",
            StyleCategory::StaticAnalysis => "静态检测",
        }
    }
}

/// Simple counter for rotating through message variations
static mut ROAST_COUNTER: usize = 0;

fn next_index(len: usize) -> usize {
    unsafe {
        ROAST_COUNTER = (ROAST_COUNTER + 1) % 10000;
        ROAST_COUNTER % len
    }
}

/// The main roast message generator
pub struct RoastGenerator;

impl Default for RoastGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RoastGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a roast message for a naming issue
    pub fn naming_roast(&self, bad_name: &str, context: &str) -> String {
        let roasts = [
            format!(
                "{} 嘿兄弟，'{}' 这个名字... 说真的，{}。建议你：{}",
                StyleCategory::Naming.emoji(),
                bad_name,
                context,
                self.naming_suggestion(bad_name)
            ),
            format!(
                "{} '{}' 是什么鬼？{}。换个名字吧，比如：{}",
                StyleCategory::Naming.emoji(),
                bad_name,
                context,
                self.naming_suggestion(bad_name)
            ),
            format!(
                "{} 我看到 '{}' 的时候，我的眼睛疼。{}。来，试试：{}",
                StyleCategory::Naming.emoji(),
                bad_name,
                context,
                self.naming_suggestion(bad_name)
            ),
        ];
        roasts[next_index(roasts.len())].clone()
    }

    /// Generate a roast message for a structure issue
    pub fn structure_roast(&self, metric: &str, value: usize, context: &str) -> String {
        let roasts = [
            format!(
                "{} {} 达到了 {}？{}。说真的，{}",
                StyleCategory::Structure.emoji(),
                metric,
                value,
                context,
                self.structure_suggestion(metric)
            ),
            format!(
                "{} 兄弟，{}={} 这数字有点离谱啊... {}。我的建议：{}",
                StyleCategory::Structure.emoji(),
                metric,
                value,
                context,
                self.structure_suggestion(metric)
            ),
            format!(
                "{} {}:{} — 这是代码还是迷宫？{}。听我一句劝：{}",
                StyleCategory::Structure.emoji(),
                metric,
                value,
                context,
                self.structure_suggestion(metric)
            ),
        ];
        roasts[next_index(roasts.len())].clone()
    }

    /// Generate a roast message for duplication
    pub fn duplication_roast(&self, dup_type: &str, count: usize, location_hint: &str) -> String {
        let roasts = [
            format!(
                "{} 发现 {} 处{}重复！{}。Ctrl+C Ctrl+V 虽然爽，但维护的时候你会哭的",
                StyleCategory::Duplication.emoji(),
                count,
                dup_type,
                location_hint
            ),
            format!(
                "{} 又是复制粘贴？{} 已经出现 {} 次了。提取成函数吧，求你了",
                StyleCategory::Duplication.emoji(),
                dup_type,
                count
            ),
            format!(
                "{} DRY (Don't Repeat Yourself) 原则听过没？这里 {} 重复了 {} 次。{}",
                StyleCategory::Duplication.emoji(),
                dup_type,
                count,
                location_hint
            ),
        ];
        roasts[next_index(roasts.len())].clone()
    }

    /// Generate a roast message for comment issues
    pub fn comment_roast(&self, issue_type: &str, detail: &str) -> String {
        let roasts = [
            format!(
                "{} {} — {}。代码写得好不好另说，但至少让别人看得懂啊",
                StyleCategory::Comments.emoji(),
                issue_type,
                detail
            ),
            format!(
                "{} 未来的你（或者你的同事）看到这段{}会想打人的：{}",
                StyleCategory::Comments.emoji(),
                issue_type,
                detail
            ),
            format!(
                "{} 注释不是用来道歉的（{}）。写清楚为什么，而不是写了什么",
                StyleCategory::Comments.emoji(),
                detail
            ),
        ];
        roasts[next_index(roasts.len())].clone()
    }

    /// Generate a mild static analysis message (lower severity)
    pub fn static_analysis_note(&self, rule_name: &str, detail: &str) -> String {
        format!(
            "{} [{}] {} — 这个不急，但有空可以看看",
            StyleCategory::StaticAnalysis.emoji(),
            rule_name,
            detail
        )
    }

    // --- Private helpers ---

    fn naming_suggestion(&self, _bad_name: &str) -> &'static str {
        let suggestions = [
            "用有意义的名词或动词组合",
            "想想6个月后的自己还能看懂这个名字吗？",
            "遵循项目命名规范（camelCase/snake_case/kebab-case）",
            "问自己：这个变量到底代表什么？把答案写成名字",
            "避免缩写，除非它是团队公认的（如 id, url, api）",
        ];
        suggestions[next_index(suggestions.len())]
    }

    fn structure_suggestion(&self, metric: &str) -> &'static str {
        match metric {
            "nesting depth" | "嵌套深度" => {
                "用早返回(early return)、提取方法、或策略模式来减少嵌套"
            }
            "function length" | "函数长度" | "lines" => "拆分成多个小函数，每个函数只做一件事",
            "complexity" | "复杂度" => "简化逻辑、提取辅助函数、使用表驱动替代 if-else 链",
            _ => "重构一下，让代码更易读、更易维护",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naming_roast_generates_message() {
        let gen = RoastGenerator::new();
        let msg = gen.naming_roast("data", "太泛了，不知道存的是什么");
        assert!(msg.contains("data"));
        assert!(msg.contains(StyleCategory::Naming.emoji()));
    }

    #[test]
    fn test_structure_roast_generates_message() {
        let gen = RoastGenerator::new();
        let msg = gen.structure_roast("嵌套深度", 8, "这代码嵌套得比俄罗斯套娃还深");
        assert!(msg.contains("8"));
        assert!(msg.contains(StyleCategory::Structure.emoji()));
    }

    #[test]
    fn test_duplication_roast_generates_message() {
        let gen = RoastGenerator::new();
        let msg = gen.duplication_roast("代码块", 5, "出现在第10行和第50行");
        assert!(msg.contains("5"));
        assert!(msg.contains(StyleCategory::Duplication.emoji()));
    }

    #[test]
    fn test_comment_roast_generates_message() {
        let gen = RoastGenerator::new();
        let msg = gen.comment_roast("TODO", "TODO: fix this later");
        assert!(msg.contains("TODO"));
        assert!(msg.contains(StyleCategory::Comments.emoji()));
    }

    #[test]
    fn test_static_analysis_note_is_mild() {
        let gen = RoastGenerator::new();
        let msg = gen.static_analysis_note("magic-number", "Found magic number 42");
        assert!(msg.contains(StyleCategory::StaticAnalysis.emoji()));
        assert!(msg.contains("不急"));
    }

    #[test]
    fn test_category_emoji_and_names() {
        assert_eq!(StyleCategory::Naming.emoji(), "🏷️");
        assert_eq!(StyleCategory::Naming.display_name(), "命名风格");
        assert_eq!(StyleCategory::Structure.emoji(), "🏗️");
        assert_eq!(StyleCategory::Duplication.emoji(), "📋");
        assert_eq!(StyleCategory::Comments.emoji(), "💬");
        assert_eq!(StyleCategory::StaticAnalysis.emoji(), "🔍");
    }
}
