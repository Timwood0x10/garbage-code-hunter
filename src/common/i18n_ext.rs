//! Extended i18n helpers for entertainment tools.

/// Check if the language is Chinese.
pub fn is_chinese(lang: &str) -> bool {
    matches!(
        lang.to_lowercase().replace('_', "-").as_str(),
        "zh" | "zh-cn" | "chinese"
    )
}

/// Select a string based on language.
pub fn t<'a>(lang: &'a str, zh: &'a str, en: &'a str) -> &'a str {
    if is_chinese(lang) {
        zh
    } else {
        en
    }
}

/// Select an owned String based on language.
pub fn t_owned(lang: &str, zh: &str, en: &str) -> String {
    if is_chinese(lang) {
        zh.to_string()
    } else {
        en.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chinese() {
        assert!(is_chinese("zh-CN"));
        assert!(is_chinese("zh"));
        assert!(is_chinese("chinese"));
        assert!(!is_chinese("en-US"));
        assert!(!is_chinese("en"));
    }

    #[test]
    fn test_t() {
        assert_eq!(t("zh-CN", "中文", "English"), "中文");
        assert_eq!(t("en-US", "中文", "English"), "English");
    }
}
