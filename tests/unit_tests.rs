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
