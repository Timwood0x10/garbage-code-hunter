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

    // ── is_chinese ────────────────────────────────────────────────

    /// Objective: Verify all standard Chinese locale variants are recognized.
    /// Invariants: Case insensitive; underscores normalized to hyphens.
    #[test]
    fn test_is_chinese_standard_variants() {
        assert!(is_chinese("zh-CN"), "zh-CN should be Chinese");
        assert!(is_chinese("zh"), "zh should be Chinese");
        assert!(is_chinese("chinese"), "chinese string should be Chinese");
        assert!(
            is_chinese("zh_CN"),
            "zh_CN (underscore) should be normalized to zh-CN"
        );
    }

    /// Objective: Verify non-Chinese locales are correctly rejected.
    /// Invariants: en, ja, fr, de, etc. are not Chinese.
    #[test]
    fn test_is_chinese_non_chinese() {
        assert!(!is_chinese("en-US"), "en-US is not Chinese");
        assert!(!is_chinese("en"), "en is not Chinese");
        assert!(!is_chinese("ja"), "ja is not Chinese");
        assert!(!is_chinese("fr"), "fr is not Chinese");
        assert!(!is_chinese("de"), "de is not Chinese");
    }

    /// Objective: Verify case insensitivity in is_chinese.
    #[test]
    fn test_is_chinese_case_insensitive() {
        assert!(is_chinese("ZH"), "ZH uppercase should match");
        assert!(is_chinese("Zh-cn"), "Zh-cn mixed case should match");
        assert!(is_chinese("Chinese"), "Chinese capitalized should match");
    }

    /// Objective: Verify empty string is not considered Chinese.
    #[test]
    fn test_is_chinese_empty() {
        assert!(!is_chinese(""), "empty string is not Chinese");
    }

    // ── t ─────────────────────────────────────────────────────────

    /// Objective: Verify t selects zh text for Chinese lang, en text otherwise.
    /// Invariants: Returns reference to one of the two input strings (no allocation).
    #[test]
    fn test_t_basic() {
        assert_eq!(
            t("zh-CN", "中文", "English"),
            "中文",
            "zh-CN => Chinese text"
        );
        assert_eq!(
            t("en-US", "中文", "English"),
            "English",
            "en-US => English text"
        );
    }

    /// Objective: Verify t falls back to English for unknown locales.
    #[test]
    fn test_t_fallback_for_unknown() {
        assert_eq!(
            t("klingon", "中文", "English"),
            "English",
            "unknown locale => English"
        );
        assert_eq!(
            t("", "中文", "English"),
            "English",
            "empty locale => English"
        );
    }

    // ── t_owned ───────────────────────────────────────────────────

    /// Objective: Verify t_owned returns owned String with correct value.
    #[test]
    fn test_t_owned_basic() {
        assert_eq!(
            t_owned("zh", "中文", "English"),
            "中文",
            "zh => Chinese String"
        );
        assert_eq!(
            t_owned("en", "中文", "English"),
            "English",
            "en => English String"
        );
    }

    /// Objective: Verify t_owned handles identical zh/en strings without duplication.
    #[test]
    fn test_t_owned_identical_strings() {
        let result = t_owned("en", "same", "same");
        assert_eq!(
            result, "same",
            "identical zh/en still returns correct value"
        );
    }
}
