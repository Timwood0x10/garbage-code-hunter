use super::*;
use crate::scoring::{QualityLevel, SeverityDistribution};

fn en_reporter() -> Reporter {
    Reporter::new(
        false,
        false,
        10,
        5,
        false,
        false,
        false,
        "en",
        Box::new(crate::llm::LocalRoastProvider),
    )
}

fn zh_reporter() -> Reporter {
    Reporter::new(
        false,
        false,
        10,
        5,
        false,
        false,
        false,
        "zh-CN",
        Box::new(crate::llm::LocalRoastProvider),
    )
}

fn dummy_score(total: f64) -> CodeQualityScore {
    CodeQualityScore {
        total_score: total,
        n_score: 0.0,
        d_score: 0.0,
        category_scores: std::collections::HashMap::new(),
        signal_scores: std::collections::HashMap::new(),
        file_count: 0,
        total_lines: 0,
        issue_density: 0.0,
        severity_distribution: SeverityDistribution {
            nuclear: 0,
            spicy: 0,
            mild: 0,
        },
        quality_level: QualityLevel::from_score(total),
    }
}

// ── is_zh ─────────────────────────────────────────────────────

#[test]
fn test_is_zh_english() {
    let r = en_reporter();
    assert!(!r.is_zh(), "EN reporter => is_zh = false");
}

#[test]
fn test_is_zh_chinese() {
    let r = zh_reporter();
    assert!(r.is_zh(), "ZH reporter => is_zh = true");
}

// ── diagnose_personality — all types × score bands ─────────────

#[test]
fn test_diagnose_copy_paste_artist_severe() {
    let r = en_reporter();
    let d = r.diagnose_personality("The Copy-Paste Artist", 85.0);
    assert!(d.contains("severe"), "score 85 => 'severe', got {d}");
    assert!(
        d.contains("duplication"),
        "Copy-Paste => duplication syndrome"
    );
}

#[test]
fn test_diagnose_copy_paste_artist_moderate() {
    let r = en_reporter();
    let d = r.diagnose_personality("The Copy-Paste Artist", 65.0);
    assert!(d.contains("moderate"), "score 65 => 'moderate', got {d}");
}

#[test]
fn test_diagnose_copy_paste_artist_mild() {
    let r = en_reporter();
    let d = r.diagnose_personality("The Copy-Paste Artist", 45.0);
    assert!(d.contains("mild"), "score 45 => 'mild', got {d}");
}

#[test]
fn test_diagnose_yolo_engineer() {
    let r = en_reporter();
    let d = r.diagnose_personality("The YOLO Engineer", 70.0);
    assert!(d.contains("panic"), "YOLO => panic syndrome");
}

#[test]
fn test_diagnose_trait_wizard() {
    let r = en_reporter();
    let d = r.diagnose_personality("The Trait Wizard", 70.0);
    assert!(
        d.contains("complexity"),
        "Trait Wizard => complexity syndrome"
    );
}

#[test]
fn test_diagnose_legacy_necromancer() {
    let r = en_reporter();
    let d = r.diagnose_personality("The Legacy Necromancer", 70.0);
    assert!(
        d.contains("semantic decay"),
        "Legacy Necromancer => semantic decay"
    );
}

#[test]
fn test_diagnose_hotfix_mercenary() {
    let r = en_reporter();
    let d = r.diagnose_personality("The Hotfix Mercenary", 70.0);
    assert!(d.contains("debt"), "Hotfix Mercenary => debt syndrome");
}

#[test]
fn test_diagnose_startup_survivor() {
    let r = en_reporter();
    let d = r.diagnose_personality("The Startup Survivor", 70.0);
    assert!(d.contains("growth"), "Startup Survivor => growth syndrome");
}

#[test]
fn test_diagnose_academic_wizard() {
    let r = en_reporter();
    let d = r.diagnose_personality("The Academic Wizard", 70.0);
    assert!(
        d.contains("abstraction"),
        "Academic Wizard => abstraction syndrome"
    );
}

#[test]
fn test_diagnose_unknown_fallback() {
    let r = en_reporter();
    let d = r.diagnose_personality("Unknown Type", 70.0);
    assert!(
        d.contains("smell"),
        "unknown => code smell syndrome, got {d}"
    );
}

#[test]
fn test_diagnose_chinese_output() {
    let r = zh_reporter();
    let d_en = en_reporter().diagnose_personality("The Copy-Paste Artist", 85.0);
    let d_zh = r.diagnose_personality("The Copy-Paste Artist", 85.0);
    assert_ne!(d_en, d_zh, "ZH diagnosis should differ from EN");
    assert!(
        d_zh.contains("严重") || d_zh.contains("中度") || d_zh.contains("轻度"),
        "ZH diagnosis should contain Chinese severity, got {d_zh}"
    );
}

// ── translate_personality_type ────────────────────────────────

#[test]
fn test_translate_personality_type_english() {
    let r = en_reporter();
    assert_eq!(
        r.translate_personality_type("The Copy-Paste Artist"),
        "The Copy-Paste Artist"
    );
    assert_eq!(
        r.translate_personality_type("The YOLO Engineer"),
        "The YOLO Engineer"
    );
}

#[test]
fn test_translate_personality_type_chinese() {
    let r = zh_reporter();
    assert_eq!(
        r.translate_personality_type("The Copy-Paste Artist"),
        "复制粘贴艺术家"
    );
    assert_eq!(
        r.translate_personality_type("The YOLO Engineer"),
        "YOLO工程师"
    );
    assert_eq!(
        r.translate_personality_type("The Enterprise Bureaucrat"),
        "企业官僚"
    );
    assert_eq!(r.translate_personality_type("Unknown"), "Unknown");
}

// ── translate_threat ──────────────────────────────────────────

#[test]
fn test_translate_threat_english() {
    let r = en_reporter();
    assert_eq!(r.translate_threat("⚠ HIGH"), "⚠ HIGH");
    assert_eq!(r.translate_threat("☢ CRITICAL"), "☢ CRITICAL");
}

#[test]
fn test_translate_threat_chinese() {
    let r = zh_reporter();
    assert_eq!(r.translate_threat("⚠ HIGH"), "⚠ 高危");
    assert_eq!(r.translate_threat("☢ CRITICAL"), "☢ 危险");
    assert_eq!(r.translate_threat("🟢 MODERATE"), "🟢 中等");
}

// ── translate_trait ───────────────────────────────────────────

#[test]
fn test_translate_trait_english_passthrough() {
    let r = en_reporter();
    assert_eq!(
        r.translate_trait("Ctrl+C, Ctrl+V is your IDE's most used shortcut"),
        "Ctrl+C, Ctrl+V is your IDE's most used shortcut"
    );
}

#[test]
fn test_translate_trait_chinese_known() {
    let r = zh_reporter();
    assert_eq!(
        r.translate_trait("Ctrl+C, Ctrl+V is your IDE's most used shortcut"),
        "Ctrl+C、Ctrl+V是你IDE最常用的快捷键"
    );
    assert_eq!(
        r.translate_trait("Why abstract when you can duplicate"),
        "能复制何必抽象"
    );
    assert_eq!(
        r.translate_trait("Jack of all trades, master of technical debt"),
        "样样通，技术债专精"
    );
}

#[test]
fn test_translate_trait_unknown_fallback() {
    let r = zh_reporter();
    assert_eq!(
        r.translate_trait("some unknown trait"),
        "some unknown trait"
    );
}

// ── translate_emotion ─────────────────────────────────────────

#[test]
fn test_translate_emotion_all() {
    let r = zh_reporter();
    let pairs = [
        ("numb from repetitive work", "重复劳动导致麻木"),
        ("denial", "否认"),
        ("proud of the complexity", "为复杂度自豪"),
        ("confident in their abbreviations", "对自己的缩写充满信心"),
        ("overwhelmed but optimistic", "不堪重负但保持乐观"),
        ("battle-scarred but still shipping", "伤痕累累但仍在交付"),
        (
            "intellectually satisfied, practically lost",
            "理论上满足，实践中迷失",
        ),
        ("professional detachment", "职业性超脱"),
    ];
    for (en, zh) in &pairs {
        assert_eq!(r.translate_emotion(en), *zh, "emotion '{en}'");
    }
}

#[test]
fn test_translate_emotion_unknown() {
    let r = zh_reporter();
    assert_eq!(r.translate_emotion("unknown emotion"), "unknown emotion");
}

// ── translate_philosophy ──────────────────────────────────────

#[test]
fn test_translate_philosophy_all() {
    let r = zh_reporter();
    let pairs = [
        ("it worked once, it'll work again", "成功过一次，就会再成功"),
        ("it won't crash in production", "生产环境不会崩溃的"),
        (
            "if it's not nested, it's not sophisticated",
            "不够嵌套就不够精妙",
        ),
        (
            "comments are for people who can't read code",
            "注释是给读不懂代码的人看的",
        ),
        ("future me will deal with it", "未来的我会处理的"),
        (
            "move fast and definitely break things",
            "快速行动，必然出错",
        ),
        (
            "the theory is beautiful, the practice is secondary",
            "理论很美，实践其次",
        ),
        ("it compiles, it ships", "能编译就能上线"),
    ];
    for (en, zh) in &pairs {
        assert_eq!(r.translate_philosophy(en), *zh, "philosophy '{en}'");
    }
}

// ── translate_cause_of_death ──────────────────────────────────

#[test]
fn test_translate_cause_of_death_all() {
    let r = zh_reporter();
    let pairs = [
        ("uncontrolled duplication metastasis", "失控的重复代码转移"),
        ("panic-driven development", "恐慌驱动开发"),
        (
            "complexity collapse — nesting level exceeded event horizon",
            "复杂度坍塌 — 嵌套层级超过事件视界",
        ),
        (
            "semantic starvation — variable names lost all meaning",
            "语义饥饿 — 变量名完全丧失含义",
        ),
        (
            "TODO accumulation — promises exceeded delivery capacity",
            "TODO堆积 — 承诺超出交付能力",
        ),
        ("growth-at-all-costs syndrome", "不惜一切代价增长综合症"),
        (
            "abstraction overdose — complexity exceeded comprehension",
            "抽象过度 — 复杂度超出理解范围",
        ),
        ("death by a thousand paper cuts", "千刀万剐"),
    ];
    for (en, zh) in &pairs {
        assert_eq!(r.translate_cause_of_death(en), *zh, "cause '{en}'");
    }
}

#[test]
fn test_translate_cause_of_death_unknown() {
    let r = zh_reporter();
    assert_eq!(r.translate_cause_of_death("unknown cause"), "unknown cause");
}

// ── translate_condition ───────────────────────────────────────

#[test]
fn test_translate_condition_all() {
    let r = zh_reporter();
    let pairs = [
        (
            "terminal — palliative care recommended",
            "晚期 — 建议姑息治疗",
        ),
        ("critical but treatable", "危重但可治疗"),
        (
            "critical — needs immediate Result therapy",
            "危重 — 需要立即Result治疗",
        ),
        ("stable but concerning", "稳定但令人担忧"),
        (
            "severe — god functions threatening readability",
            "严重 — 上帝函数威胁可读性",
        ),
        (
            "moderate — some functions need splitting",
            "中度 — 部分函数需要拆分",
        ),
        ("chronic but manageable", "慢性但可控"),
        (
            "stable with high technical debt interest",
            "稳定但技术债利息很高",
        ),
        (
            "critical — needs stabilization sprint",
            "危重 — 需要稳定化冲刺",
        ),
        (
            "chronic — needs simplification therapy",
            "慢性 — 需要简化治疗",
        ),
        (
            "fair — routine maintenance recommended",
            "尚可 — 建议常规维护",
        ),
    ];
    for (en, zh) in &pairs {
        assert_eq!(r.translate_condition(en), *zh, "condition '{en}'");
    }
}

// ── translate_final_words ─────────────────────────────────────

#[test]
fn test_translate_final_words_all() {
    let r = zh_reporter();
    let pairs = [
        ("just one more hotfix", "再修一个热修复就好"),
        ("i'll add error handling later", "以后再加错误处理"),
        ("i can still understand it", "我还是能看懂的"),
        ("temp2 should work", "temp2应该能用"),
        ("i'll fix it in the next sprint", "下个冲刺再修"),
        (
            "we'll fix it after the next funding round",
            "下一轮融资后就修",
        ),
        (
            "it's actually quite simple once you understand the theory",
            "理解了理论其实很简单",
        ),
        ("we'll refactor next quarter", "下个季度重构"),
    ];
    for (en, zh) in &pairs {
        assert_eq!(r.translate_final_words(en), *zh, "final words '{en}'");
    }
}

// ── print_behavior_distribution ───────────────────────────────

/// Objective: Smoke test — print_behavior_distribution does not panic with empty scores.
#[test]
fn test_print_behavior_distribution_empty_scores() {
    let r = en_reporter();
    let score = dummy_score(50.0);
    r.print_behavior_distribution(&score);
}

/// Objective: Smoke test — print_behavior_distribution does not panic with realistic scores.
#[test]
fn test_print_behavior_distribution_with_scores() {
    let r = en_reporter();
    let mut score = dummy_score(50.0);
    score.signal_scores.insert(StyleSignal::Duplication, 25.0);
    score.signal_scores.insert(StyleSignal::NamingChaos, 10.0);
    score.signal_scores.insert(StyleSignal::PanicAddiction, 5.0);
    r.print_behavior_distribution(&score);
}

// ── final_summary ─────────────────────────────────────────────

/// Objective: Smoke test — print_final_summary does not panic.
#[test]
fn test_print_final_summary_no_panic() {
    let r = en_reporter();
    let score = dummy_score(75.0);
    r.print_final_summary(&score, 10, Some("The Copy-Paste Artist"));
}

#[test]
fn test_print_final_summary_none_personality() {
    let r = en_reporter();
    let score = dummy_score(30.0);
    r.print_final_summary(&score, 0, None);
}
