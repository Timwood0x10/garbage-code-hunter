use garbage_code_hunter::i18n::I18n;

#[test]
fn test_i18n_chinese_messages() {
    let i18n = I18n::new("zh-CN");

    assert_eq!(i18n.get("title"), "🗑️  垃圾代码猎人 🗑️");
    assert_eq!(i18n.get("preparing"), "正在准备吐槽你的代码...");
    assert_eq!(
        i18n.get("clean_code"),
        "🎉 哇！你的代码居然没有明显的垃圾！"
    );
}

#[test]
fn test_i18n_english_messages() {
    let i18n = I18n::new("en-US");

    assert_eq!(i18n.get("title"), "🗑️  Garbage Code Hunter 🗑️");
    assert_eq!(i18n.get("preparing"), "Preparing to roast your code...");
    assert_eq!(
        i18n.get("clean_code"),
        "🎉 Wow! Your code doesn't have obvious garbage!"
    );
}

#[test]
fn test_i18n_fallback_to_english() {
    let i18n = I18n::new("fr-FR"); // Unsupported language

    // Should fallback to English
    assert_eq!(i18n.get("title"), "🗑️  Garbage Code Hunter 🗑️");
    assert_eq!(i18n.get("preparing"), "Preparing to roast your code...");
}

#[test]
fn test_i18n_missing_key() {
    let i18n = I18n::new("en-US");

    let result = i18n.get("nonexistent_key");
    assert!(result.contains("Missing translation"));
}

#[test]
fn test_roast_messages_chinese() {
    let i18n = I18n::new("zh-CN");

    let messages = i18n.get_roast_messages("terrible-naming");
    assert!(
        !messages.is_empty(),
        "Should have roast messages for terrible naming"
    );
    assert!(
        messages[0].contains("变量名"),
        "Chinese messages should contain Chinese text"
    );

    let unwrap_messages = i18n.get_roast_messages("unwrap-abuse");
    assert!(
        !unwrap_messages.is_empty(),
        "Should have roast messages for unwrap abuse"
    );
    assert!(
        unwrap_messages[0].contains("unwrap"),
        "Should mention unwrap"
    );
}

#[test]
fn test_roast_messages_english() {
    let i18n = I18n::new("en-US");

    let messages = i18n.get_roast_messages("terrible-naming");
    assert!(
        !messages.is_empty(),
        "Should have roast messages for terrible naming"
    );
    assert!(
        messages[0].contains("variable"),
        "English messages should contain English text"
    );

    let unwrap_messages = i18n.get_roast_messages("unwrap-abuse");
    assert!(
        !unwrap_messages.is_empty(),
        "Should have roast messages for unwrap abuse"
    );
    assert!(
        unwrap_messages[0].contains("unwrap"),
        "Should mention unwrap"
    );
}

#[test]
fn test_suggestions_chinese() {
    let i18n = I18n::new("zh-CN");

    let rule_names = vec!["terrible-naming".to_string(), "unwrap-abuse".to_string()];
    let suggestions = i18n.get_suggestions(&rule_names);

    // Suggestions are now handled by the --suggestions flag and hall_of_shame module
    assert!(
        suggestions.is_empty(),
        "Suggestions should be empty as they're now handled by --suggestions flag"
    );
    // assert!(
    //     suggestions.iter().any(|s| s.contains("variable name")),
    //     "Should have naming suggestions"
    // );
    // assert!(
    //     suggestions.iter().any(|s| s.contains("unwrap")),
    //     "Should have unwrap suggestions"
    // );
}

#[test]
fn test_suggestions_english() {
    let i18n = I18n::new("en-US");

    let rule_names = vec!["terrible-naming".to_string(), "deep-nesting".to_string()];
    let suggestions = i18n.get_suggestions(&rule_names);

    // Suggestions are now handled by the --suggestions flag and hall_of_shame module
    assert!(
        suggestions.is_empty(),
        "Suggestions should be empty as they're now handled by --suggestions flag"
    );
    // assert!(
    //     suggestions.iter().any(|s| s.contains("variable")),
    //     "Should have naming suggestions in English"
    // );
    // assert!(
    //     suggestions.iter().any(|s| s.contains("nesting")),
    //     "Should have nesting suggestions"
    // );
}

#[test]
fn test_empty_suggestions() {
    let i18n = I18n::new("en-US");

    let rule_names = vec![];
    let suggestions = i18n.get_suggestions(&rule_names);

    // Suggestions are now handled by the --suggestions flag and hall_of_shame module
    assert!(
        suggestions.is_empty(),
        "Suggestions should be empty as they're now handled by --suggestions flag"
    );
    // Since suggestions are now empty, remove this assertion
    // assert!(
    //     suggestions[0].contains("good"),
    //     "Default suggestion should be encouraging"
    // );
}

#[cfg(test)]
mod rule_tests {
    use garbage_code_hunter::rules::naming::{SingleLetterVariableRule, TerribleNamingRule};
    use garbage_code_hunter::rules::Rule;
    use std::path::Path;
    use syn::parse_file;

    #[test]
    fn test_terrible_naming_rule_name() {
        let rule = TerribleNamingRule;
        assert_eq!(rule.name(), "terrible-naming");
    }

    #[test]
    fn test_single_letter_rule_name() {
        let rule = SingleLetterVariableRule;
        assert_eq!(rule.name(), "single-letter-variable");
    }

    #[test]
    fn test_terrible_naming_detection() {
        let rule = TerribleNamingRule;
        let code = r#"
fn main() {
    let data = "hello";
    let temp = 42;
    let good_variable_name = "this is fine";
}
"#;

        let syntax_tree = parse_file(code).expect("Failed to parse code");
        let path = Path::new("test.rs");
        let issues = rule.check(path, &syntax_tree, code, "en-US", false);

        // Should detect 'data' and 'temp' but not 'good_variable_name'
        assert!(!issues.is_empty(), "Should detect terrible naming");
        assert!(issues.len() >= 2, "Should detect at least 2 issues");
    }

    #[test]
    fn test_single_letter_variable_detection() {
        let rule = SingleLetterVariableRule;
        let code = r#"
fn main() {
    let a = 10;  // Should be detected
    let b = 20;  // Should be detected
    let i = 0;   // Should NOT be detected (common loop variable)
    let j = 1;   // Should NOT be detected (common loop variable)
    let good_name = 42; // Should NOT be detected
}
"#;

        let syntax_tree = parse_file(code).expect("Failed to parse code");
        let path = Path::new("test.rs");
        let issues = rule.check(path, &syntax_tree, code, "en-US", false);

        // Should detect 'a' and 'b' but not 'i', 'j', or 'good_name'
        assert!(!issues.is_empty(), "Should detect single letter variables");
        // Note: The exact count might vary based on implementation details
    }
}
