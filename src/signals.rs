// StyleSignal — maps rule issues to behavioral style signals.

use crate::analyzer::CodeIssue;
use crate::language::Language;
use crate::treesitter::engine::ParsedFile;
use std::collections::HashMap;

// ── Signal Detector trait ─────────────────────────────────────────

/// Direct signal detector that produces StyleSignal scores from parsed code.
///
/// Unlike rules that produce CodeIssues, a SignalDetector directly scores
/// a file for a specific style signal without going through Rule → Issue pipeline.
/// This enables the "few rules + strong aggregation" approach by replacing many
/// small rules with one signal detector per behavioral dimension.
pub trait SignalDetector: Send + Sync {
    fn signal(&self) -> StyleSignal;
    fn supported_languages(&self) -> &'static [Language];
    /// Run detection on a parsed file. Returns the number of signal violations found.
    fn count_violations(&self, file: &ParsedFile) -> usize;

    /// Produce per-file signal findings with violation counts.
    /// Default implementation wraps `count_violations` into a single `(signal, count)` pair.
    /// Override for more granular findings (e.g., one per call-site).
    fn detect_findings(&self, file: &ParsedFile) -> Vec<(StyleSignal, usize)> {
        let count = self.count_violations(file);
        if count > 0 {
            vec![(self.signal(), count)]
        } else {
            vec![]
        }
    }
}

/// Helpers for converting raw violations to density-normalized signal scores.
pub fn violations_to_score(count: usize, total_lines: usize) -> f64 {
    let k_lines = (total_lines as f64 / 1000.0).max(0.001);
    let density = count as f64 / k_lines;
    ((density + 1.0).log2() * 6.0).min(25.0)
}

/// Aggregate scores from all detectors across all parsed files.
pub fn aggregate_detector_scores(
    detectors: &[Box<dyn SignalDetector>],
    files: &[ParsedFile],
) -> HashMap<StyleSignal, f64> {
    let mut total_counts: HashMap<StyleSignal, usize> = HashMap::new();
    let mut total_lines: HashMap<StyleSignal, usize> = HashMap::new();

    for file in files {
        let lang = file.language;
        for detector in detectors {
            if !detector.supported_languages().contains(&lang) {
                continue;
            }
            let signal = detector.signal();
            let count = detector.count_violations(file);
            *total_counts.entry(signal).or_insert(0) += count;
            *total_lines.entry(signal).or_insert(0) += file.content.lines().count();
        }
    }

    let mut scores = HashMap::new();
    for (signal, count) in total_counts {
        let lines = total_lines.get(&signal).copied().unwrap_or(1);
        scores.insert(signal, violations_to_score(count, lines));
    }
    scores
}

pub use crate::detectors::{
    CodeSmellsDetector, DuplicationDetector, NamingChaosDetector, NestedHellDetector,
    PanicAddictionDetector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleSignal {
    Duplication,
    PanicAddiction,
    NamingChaos,
    NestedHell,
    HotfixCulture,
    OverEngineering,
    CodeSmells,
}

impl StyleSignal {
    pub fn all() -> &'static [StyleSignal] {
        &[
            StyleSignal::Duplication,
            StyleSignal::PanicAddiction,
            StyleSignal::NamingChaos,
            StyleSignal::NestedHell,
            StyleSignal::HotfixCulture,
            StyleSignal::OverEngineering,
            StyleSignal::CodeSmells,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            StyleSignal::Duplication => "Duplication",
            StyleSignal::PanicAddiction => "Panic Addiction",
            StyleSignal::NamingChaos => "Naming Chaos",
            StyleSignal::NestedHell => "Nested Hell",
            StyleSignal::HotfixCulture => "Hotfix Culture",
            StyleSignal::OverEngineering => "Over-Engineering",
            StyleSignal::CodeSmells => "Code Smells",
        }
    }

    pub fn display_name_zh(&self) -> String {
        match self {
            StyleSignal::Duplication => "重复代码",
            StyleSignal::PanicAddiction => "恐慌成瘾",
            StyleSignal::NamingChaos => "命名混乱",
            StyleSignal::NestedHell => "嵌套地狱",
            StyleSignal::HotfixCulture => "热修复文化",
            StyleSignal::OverEngineering => "过度工程",
            StyleSignal::CodeSmells => "代码异味",
        }
        .to_string()
    }
}

// ── Language Capability Matrix ────────────────────────────────────

/// Tracks which StyleSignals are detectable per language.
///
/// This matrix answers "Can we detect signal X in language Y?"
/// through two paths:
///   - `supported_signals()` — via classify_rule on issue rule names
///   - `direct_signals()` — via SignalDetector (direct AST analysis)
///
/// All languages with a tree-sitter grammar currently support all 7
/// signals through the cross-language rules (deep-nesting, long-function,
/// god-function, file-too-long, todo-comment, commented-code, dead-code,
/// terrible-naming, single-letter-variable, meaningless-naming,
/// hungarian-notation, abbreviation-abuse, magic-number, println-debugging).
/// Language-specific rules add precision but don't expand the signal set.
pub struct LanguageCapabilityMatrix;

impl LanguageCapabilityMatrix {
    /// Returns StyleSignals that have at least one rule for this language.
    pub fn supported_signals(lang: Language) -> &'static [StyleSignal] {
        if lang.has_tree_sitter_grammar() {
            StyleSignal::all()
        } else {
            &[]
        }
    }

    /// Returns signals that have a direct SignalDetector implementation.
    pub fn direct_signals(lang: Language) -> &'static [StyleSignal] {
        match lang {
            Language::Rust => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::Python => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::Go => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::JavaScript => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::TypeScript => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::Java => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::Ruby => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::C => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::Cpp => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            Language::Swift | Language::Zig => &[
                StyleSignal::Duplication,
                StyleSignal::CodeSmells,
                StyleSignal::PanicAddiction,
                StyleSignal::NamingChaos,
                StyleSignal::NestedHell,
                StyleSignal::HotfixCulture,
                StyleSignal::OverEngineering,
            ],
            _ => &[],
        }
    }

    /// Returns true if the given signal is detectable for this language.
    pub fn supports_signal(lang: Language, signal: StyleSignal) -> bool {
        Self::supported_signals(lang).contains(&signal)
    }

    /// Returns true if the given signal has a direct detector for this language.
    pub fn has_direct_detector(lang: Language, signal: StyleSignal) -> bool {
        Self::direct_signals(lang).contains(&signal)
    }
}

pub fn classify_rule(rule_name: &str) -> StyleSignal {
    match rule_name {
        "code-duplication" | "cross-file-duplication" => StyleSignal::Duplication,
        "unwrap-abuse" | "panic-abuse" | "bare-except" | "bare-rescue" | "empty-catch"
        | "println-debugging" => StyleSignal::PanicAddiction,
        "terrible-naming"
        | "single-letter-variable"
        | "meaningless-naming"
        | "hungarian-notation"
        | "abbreviation-abuse"
        | "c-naming"
        | "go-receiver-name"
        | "go-mixed-caps"
        | "ruby-predicate-method"
        | "python-naming"
        | "constant-name" => StyleSignal::NamingChaos,
        "deep-nesting"
        | "cyclomatic-complexity"
        | "c-nesting"
        | "complex-closure"
        | "go-else-return"
        | "negated-if" => StyleSignal::NestedHell,
        "todo-comment" | "todo-fixme" | "todo-bug" | "todo-hack" | "commented-code"
        | "dead-code" | "c-commented-code" | "c-dead-code" => StyleSignal::HotfixCulture,
        "too-many-params" | "god-function" | "long-function" | "c-long-function"
        | "c-god-function" | "file-too-long" | "module-complexity" | "trait-complexity"
        | "generic-abuse" => StyleSignal::OverEngineering,
        _ => StyleSignal::CodeSmells,
    }
}

// ── Style IR: StyleProfile ────────────────────────────────────────

/// StyleProfile is the intermediate representation between signal scores and
/// personality inference. It holds the normalized signal vector and derives
/// the dominant personality type from it.
#[derive(Debug, Clone)]
pub struct StyleProfile {
    pub signal_scores: HashMap<StyleSignal, f64>,
    pub dominant_signal: Option<StyleSignal>,
}

impl StyleProfile {
    /// Build a StyleProfile from a signal-scores map.
    pub fn from_signal_scores(signal_scores: HashMap<StyleSignal, f64>) -> Self {
        let dominant_signal = StyleSignal::all()
            .iter()
            .max_by(|a, b| {
                let sa = signal_scores.get(a).copied().unwrap_or(0.0);
                let sb = signal_scores.get(b).copied().unwrap_or(0.0);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied();
        Self {
            signal_scores,
            dominant_signal,
        }
    }

    pub fn score(&self, signal: StyleSignal) -> f64 {
        self.signal_scores.get(&signal).copied().unwrap_or(0.0)
    }

    /// Infer personality type from the signal vector.
    ///
    /// Mapping (scores range 0-25):
    ///   - ≥ 12 → high (dominant)
    ///   - ≥ 6  → medium (present)
    ///   - < 6  → low
    pub fn infer_personality_type(&self) -> &'static str {
        let dup = self.score(StyleSignal::Duplication);
        let panic = self.score(StyleSignal::PanicAddiction);
        let naming = self.score(StyleSignal::NamingChaos);
        let nested = self.score(StyleSignal::NestedHell);
        let hotfix = self.score(StyleSignal::HotfixCulture);
        let over_eng = self.score(StyleSignal::OverEngineering);

        // Single high-dominance signals
        if dup >= 12.0 && dup >= panic && dup >= naming && dup >= nested {
            return "The Copy-Paste Artist";
        }
        if panic >= 12.0 && panic >= dup && panic >= naming && panic >= nested {
            return "The YOLO Engineer";
        }
        if nested >= 12.0 && nested >= naming && nested >= hotfix {
            return "The Trait Wizard";
        }
        if naming >= 12.0 && naming >= nested {
            return "The Legacy Necromancer";
        }
        if hotfix >= 12.0 {
            return "The Hotfix Mercenary";
        }

        // Compound personalities (medium signals)
        if dup >= 6.0 && panic >= 6.0 {
            return "The Startup Survivor";
        }
        if (naming >= 6.0 && nested >= 6.0) || over_eng >= 12.0 {
            return "The Academic Wizard";
        }
        if over_eng >= 6.0 {
            return "The Academic Wizard";
        }

        "The Enterprise Bureaucrat"
    }
}

pub fn compute_signal_scores(
    issues: &[CodeIssue],
    total_lines: usize,
) -> HashMap<StyleSignal, f64> {
    let k_lines = total_lines as f64 / 1000.0;
    let mut counts: HashMap<StyleSignal, usize> = HashMap::new();

    for issue in issues {
        let signal = classify_rule(&issue.rule_name);
        *counts.entry(signal).or_insert(0) += 1;
    }

    let mut scores = HashMap::new();
    for signal in StyleSignal::all() {
        let count = counts.get(signal).copied().unwrap_or(0);
        let density = if k_lines > 0.0 {
            count as f64 / k_lines
        } else {
            0.0
        };
        let score = ((density + 1.0).log2() * 6.0).min(25.0);
        scores.insert(*signal, score);
    }

    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_issue(rule_name: &str) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: 0,
            rule_name: rule_name.to_string(),
            message: String::new(),
            severity: crate::analyzer::Severity::Spicy,
        }
    }

    // ── display_name ──────────────────────────────────────────────

    /// Objective: Verify all signals have unique non-empty English display names.
    #[test]
    fn test_display_name_all_variants() {
        let mut names = std::collections::HashSet::new();
        for s in StyleSignal::all() {
            let name = s.display_name();
            assert!(!name.is_empty(), "{:?}.display_name should not be empty", s);
            assert!(
                names.insert(name),
                "{:?}.display_name '{}' is not unique",
                s,
                name
            );
        }
    }

    /// Objective: Verify all signals have unique non-empty Chinese display names.
    #[test]
    fn test_display_name_zh_all_variants() {
        let mut names = std::collections::HashSet::new();
        for s in StyleSignal::all() {
            let name = s.display_name_zh();
            assert!(
                !name.is_empty(),
                "{:?}.display_name_zh should not be empty",
                s
            );
            assert!(
                names.insert(name.clone()),
                "{:?}.display_name_zh '{}' is not unique",
                s,
                name
            );
        }
    }

    // ── classify_rule: all branches ───────────────────────────────

    /// Objective: Verify all Duplication rule names map correctly.
    #[test]
    fn test_classify_duplication_all() {
        assert_eq!(
            classify_rule("code-duplication"),
            StyleSignal::Duplication,
            "code-duplication"
        );
        assert_eq!(
            classify_rule("cross-file-duplication"),
            StyleSignal::Duplication,
            "cross-file-duplication"
        );
    }

    /// Objective: Verify all PanicAddiction rule names map correctly.
    #[test]
    fn test_classify_panic_all() {
        for name in &[
            "unwrap-abuse",
            "panic-abuse",
            "bare-except",
            "bare-rescue",
            "empty-catch",
            "println-debugging",
        ] {
            assert_eq!(
                classify_rule(name),
                StyleSignal::PanicAddiction,
                "{name} should map to PanicAddiction"
            );
        }
    }

    /// Objective: Verify all NamingChaos rule names map correctly.
    #[test]
    fn test_classify_naming_all() {
        for name in &[
            "terrible-naming",
            "single-letter-variable",
            "meaningless-naming",
            "hungarian-notation",
            "abbreviation-abuse",
            "c-naming",
            "go-receiver-name",
            "go-mixed-caps",
            "ruby-predicate-method",
            "python-naming",
            "constant-name",
        ] {
            assert_eq!(
                classify_rule(name),
                StyleSignal::NamingChaos,
                "{name} should map to NamingChaos"
            );
        }
    }

    /// Objective: Verify all NestedHell rule names map correctly.
    #[test]
    fn test_classify_nested_all() {
        for name in &[
            "deep-nesting",
            "cyclomatic-complexity",
            "c-nesting",
            "complex-closure",
            "go-else-return",
            "negated-if",
        ] {
            assert_eq!(
                classify_rule(name),
                StyleSignal::NestedHell,
                "{name} should map to NestedHell"
            );
        }
    }

    /// Objective: Verify all HotfixCulture rule names map correctly.
    #[test]
    fn test_classify_hotfix_all() {
        for name in &[
            "todo-comment",
            "todo-fixme",
            "todo-bug",
            "todo-hack",
            "commented-code",
            "dead-code",
            "c-commented-code",
            "c-dead-code",
        ] {
            assert_eq!(
                classify_rule(name),
                StyleSignal::HotfixCulture,
                "{name} should map to HotfixCulture"
            );
        }
    }

    /// Objective: Verify all OverEngineering rule names map correctly.
    #[test]
    fn test_classify_over_engineering_all() {
        for name in &[
            "too-many-params",
            "god-function",
            "long-function",
            "c-long-function",
            "c-god-function",
            "file-too-long",
            "module-complexity",
            "trait-complexity",
            "generic-abuse",
        ] {
            assert_eq!(
                classify_rule(name),
                StyleSignal::OverEngineering,
                "{name} should map to OverEngineering"
            );
        }
    }

    /// Objective: Verify CodeSmells fallback for unknown and known non-mapped rules.
    #[test]
    fn test_classify_code_smells_fallback() {
        assert_eq!(classify_rule("magic-number"), StyleSignal::CodeSmells);
        assert_eq!(classify_rule("unknown-rule"), StyleSignal::CodeSmells);
        assert_eq!(classify_rule(""), StyleSignal::CodeSmells);
        assert_eq!(classify_rule("rust-doc-example"), StyleSignal::CodeSmells);
    }

    // ── LanguageCapabilityMatrix ──────────────────────────────────

    /// Objective: Verify all grammar languages support all 7 signals.
    /// Invariants: supported_signals returns StyleSignal::all() for grammar languages.
    #[test]
    fn test_matrix_supported_all_grammar_languages() {
        for lang in crate::language::LANGUAGES_WITH_GRAMMAR {
            let sigs = LanguageCapabilityMatrix::supported_signals(*lang);
            assert_eq!(
                sigs.len(),
                7,
                "{} should support 7 signals",
                lang.display_name()
            );
        }
    }

    /// Objective: Verify Unknown language returns empty supported signals.
    #[test]
    fn test_matrix_supported_unknown() {
        let sigs = LanguageCapabilityMatrix::supported_signals(Language::Unknown);
        assert!(sigs.is_empty(), "Unknown should have no supported signals");
    }

    /// Objective: Verify supports_signal returns true for a known pairing.
    #[test]
    fn test_matrix_supports_signal_rust_panic() {
        assert!(LanguageCapabilityMatrix::supports_signal(
            Language::Rust,
            StyleSignal::PanicAddiction
        ));
    }

    /// Objective: Verify supports_signal returns false for Unknown.
    #[test]
    fn test_matrix_supports_signal_unknown() {
        assert!(!LanguageCapabilityMatrix::supports_signal(
            Language::Unknown,
            StyleSignal::PanicAddiction
        ));
    }

    /// Objective: Verify direct_signals returns all 7 signals for Rust.
    #[test]
    fn test_matrix_direct_signals_rust() {
        let sigs = LanguageCapabilityMatrix::direct_signals(Language::Rust);
        for signal in StyleSignal::all() {
            assert!(
                sigs.contains(signal),
                "Rust should have direct {}",
                signal.display_name()
            );
        }
        assert_eq!(sigs.len(), 7, "Rust has all 7 direct signals");
    }

    /// Objective: Verify direct_signals returns all 7 signals for Go.
    #[test]
    fn test_matrix_direct_signals_go() {
        let sigs = LanguageCapabilityMatrix::direct_signals(Language::Go);
        for signal in StyleSignal::all() {
            assert!(
                sigs.contains(signal),
                "Go should have direct {}",
                signal.display_name()
            );
        }
        assert_eq!(sigs.len(), 7, "Go has all 7 direct signals");
    }

    /// Objective: Verify direct_signals returns all 7 signals for Python.
    #[test]
    fn test_matrix_direct_signals_python() {
        let sigs = LanguageCapabilityMatrix::direct_signals(Language::Python);
        for signal in StyleSignal::all() {
            assert!(
                sigs.contains(signal),
                "Python should have direct {}",
                signal.display_name()
            );
        }
        assert_eq!(sigs.len(), 7, "Python has all 7 direct signals");
    }

    /// Objective: Verify has_direct_detector matches direct_signals.
    #[test]
    fn test_matrix_has_direct_detector_rust() {
        assert!(LanguageCapabilityMatrix::has_direct_detector(
            Language::Rust,
            StyleSignal::PanicAddiction
        ));
        assert!(LanguageCapabilityMatrix::has_direct_detector(
            Language::Swift,
            StyleSignal::PanicAddiction
        ));
        assert!(LanguageCapabilityMatrix::has_direct_detector(
            Language::Zig,
            StyleSignal::PanicAddiction
        ));
        assert!(!LanguageCapabilityMatrix::has_direct_detector(
            Language::Unknown,
            StyleSignal::PanicAddiction
        ));
    }

    // ── compute_signal_scores ─────────────────────────────────────

    /// Objective: Verify empty issues produce all zeros.
    /// Invariants: All 7 signals present, each exactly 0.0.
    #[test]
    fn test_compute_signal_scores_empty() {
        let scores = compute_signal_scores(&[], 1000);
        assert_eq!(scores.len(), 7, "all 7 signals present");
        for s in StyleSignal::all() {
            assert!(
                (scores[s] - 0.0).abs() < f64::EPSILON,
                "empty issues => {s:?} = {}",
                scores[s]
            );
        }
    }

    /// Objective: Verify mixed issues produce expected ordering.
    /// Invariants: More issues in a category => higher score for that category.
    #[test]
    fn test_compute_signal_scores_mixed() {
        let issues = vec![
            make_issue("unwrap-abuse"),
            make_issue("unwrap-abuse"),
            make_issue("deep-nesting"),
            make_issue("terrible-naming"),
        ];
        let scores = compute_signal_scores(&issues, 1000);
        assert!(
            scores[&StyleSignal::PanicAddiction] > scores[&StyleSignal::NamingChaos],
            "2 panics should score higher than 1 naming"
        );
        assert!(
            scores[&StyleSignal::PanicAddiction] > scores[&StyleSignal::NestedHell],
            "2 panics should score higher than 1 nesting"
        );
    }

    /// Objective: Verify compute_signal_scores handles zero total_lines without division by zero.
    /// Invariants: k_lines = 0, density = 0, score = log2(0+1)*6 = 0.
    #[test]
    fn test_compute_signal_scores_zero_lines() {
        let issues = vec![make_issue("unwrap-abuse")];
        let scores = compute_signal_scores(&issues, 0);
        // With 0 lines, k_lines = 0, density = 0/0 = 0 (the code branches to 0.0)
        // score = (0+1).log2() * 6 = 0, clamped to min(25) = 0... wait log2(1) = 0, so 0 * 6 = 0
        assert!(
            scores.values().all(|&s| s >= 0.0),
            "zero lines should not produce NaN or negative"
        );
        assert!(
            scores[&StyleSignal::PanicAddiction] > 0.0
                || (scores[&StyleSignal::PanicAddiction] - 0.0).abs() < f64::EPSILON,
            "score with zero lines should be >= 0"
        );
    }

    /// Objective: Verify score cap at 25.0 for high-density signals.
    #[test]
    fn test_compute_signal_scores_capped() {
        // 1000 panics in 1 line → extremely high density → capped
        let issues: Vec<_> = (0..1000).map(|_| make_issue("unwrap-abuse")).collect();
        let scores = compute_signal_scores(&issues, 1);
        assert!(
            scores[&StyleSignal::PanicAddiction] <= 25.0,
            "score should be capped at 25, got {}",
            scores[&StyleSignal::PanicAddiction]
        );
    }

    /// Objective: Verify each category contributes independently.
    /// Invariants: Only the category with issues should have a non-zero score.
    #[test]
    fn test_compute_signal_scores_category_independence() {
        let issues = vec![make_issue("deep-nesting")];
        let scores = compute_signal_scores(&issues, 1000);
        assert!(
            scores[&StyleSignal::NestedHell] > 0.0,
            "NestedHell should be non-zero"
        );
        for s in StyleSignal::all() {
            if *s != StyleSignal::NestedHell {
                assert!(
                    (scores[s] - 0.0).abs() < f64::EPSILON,
                    "only NestedHell should be non-zero, but {s:?} = {}",
                    scores[s]
                );
            }
        }
    }

    /// Objective: Verify the density formula: higher density with same count = higher score.
    /// Invariants: Same issues in fewer lines => higher score.
    #[test]
    fn test_compute_signal_scores_density_scaling() {
        let issues = vec![make_issue("unwrap-abuse")];
        let sparse = compute_signal_scores(&issues, 100_000); // low density
        let dense = compute_signal_scores(&issues, 10); // high density
        assert!(
            dense[&StyleSignal::PanicAddiction] > sparse[&StyleSignal::PanicAddiction],
            "dense (10 lines) should score higher than sparse (100k lines)"
        );
    }

    // ── StyleProfile ─────────────────────────────────────────────

    fn make_profile(scores: &[(StyleSignal, f64)]) -> StyleProfile {
        let map: HashMap<StyleSignal, f64> = scores.iter().cloned().collect();
        StyleProfile::from_signal_scores(map)
    }

    /// Objective: Verify empty signal_scores produce default-0 scores.
    /// Invariants: When all scores are equal (all 0), max_by returns the last variant (CodeSmells).
    #[test]
    fn test_style_profile_empty() {
        let p = StyleProfile::from_signal_scores(HashMap::new());
        assert_eq!(p.dominant_signal, Some(StyleSignal::CodeSmells));
        assert_eq!(p.score(StyleSignal::Duplication), 0.0);
        assert_eq!(p.infer_personality_type(), "The Enterprise Bureaucrat");
    }

    /// Objective: Verify high Duplication => Copy-Paste Artist.
    #[test]
    fn test_style_profile_copy_paste() {
        let p = make_profile(&[
            (StyleSignal::Duplication, 15.0),
            (StyleSignal::PanicAddiction, 3.0),
        ]);
        assert_eq!(p.dominant_signal, Some(StyleSignal::Duplication));
        assert_eq!(p.infer_personality_type(), "The Copy-Paste Artist");
    }

    /// Objective: Verify high PanicAddiction => YOLO Engineer.
    #[test]
    fn test_style_profile_yolo() {
        let p = make_profile(&[
            (StyleSignal::PanicAddiction, 15.0),
            (StyleSignal::NamingChaos, 3.0),
        ]);
        assert_eq!(p.infer_personality_type(), "The YOLO Engineer");
    }

    /// Objective: Verify high NestedHell => Trait Wizard.
    #[test]
    fn test_style_profile_trait_wizard() {
        let p = make_profile(&[
            (StyleSignal::NestedHell, 15.0),
            (StyleSignal::NamingChaos, 2.0),
        ]);
        assert_eq!(p.infer_personality_type(), "The Trait Wizard");
    }

    /// Objective: Verify high NamingChaos => Legacy Necromancer.
    #[test]
    fn test_style_profile_legacy_necromancer() {
        let p = make_profile(&[
            (StyleSignal::NamingChaos, 15.0),
            (StyleSignal::NestedHell, 3.0),
        ]);
        assert_eq!(p.infer_personality_type(), "The Legacy Necromancer");
    }

    /// Objective: Verify high HotfixCulture => Hotfix Mercenary.
    #[test]
    fn test_style_profile_hotfix() {
        let p = make_profile(&[(StyleSignal::HotfixCulture, 15.0)]);
        assert_eq!(p.infer_personality_type(), "The Hotfix Mercenary");
    }

    /// Objective: Verify medium Duplication + medium PanicAddiction => Startup Survivor.
    #[test]
    fn test_style_profile_startup() {
        let p = make_profile(&[
            (StyleSignal::Duplication, 8.0),
            (StyleSignal::PanicAddiction, 7.0),
        ]);
        assert_eq!(p.infer_personality_type(), "The Startup Survivor");
    }

    /// Objective: Verify medium NamingChaos + medium NestedHell => Academic Wizard.
    /// Invariants: compound pattern takes precedence over individual medium scores.
    #[test]
    fn test_style_profile_academic_compound() {
        let p = make_profile(&[
            (StyleSignal::NamingChaos, 8.0),
            (StyleSignal::NestedHell, 7.0),
        ]);
        assert_eq!(p.infer_personality_type(), "The Academic Wizard");
    }

    /// Objective: Verify high OverEngineering => Academic Wizard.
    #[test]
    fn test_style_profile_academic_over_eng() {
        let p = make_profile(&[(StyleSignal::OverEngineering, 15.0)]);
        assert_eq!(p.infer_personality_type(), "The Academic Wizard");
    }

    /// Objective: Verify all scores < 6 => Enterprise Bureaucrat.
    #[test]
    fn test_style_profile_enterprise() {
        let p = make_profile(&[
            (StyleSignal::Duplication, 4.0),
            (StyleSignal::HotfixCulture, 3.0),
        ]);
        assert_eq!(p.infer_personality_type(), "The Enterprise Bureaucrat");
    }

    /// Objective: Verify all scores 0 => Enterprise Bureaucrat.
    #[test]
    fn test_style_profile_all_zero() {
        let p = make_profile(&[
            (StyleSignal::Duplication, 0.0),
            (StyleSignal::PanicAddiction, 0.0),
            (StyleSignal::NamingChaos, 0.0),
            (StyleSignal::NestedHell, 0.0),
            (StyleSignal::HotfixCulture, 0.0),
            (StyleSignal::OverEngineering, 0.0),
            (StyleSignal::CodeSmells, 0.0),
        ]);
        // All equal at 0, the last in `all()` (CodeSmells) wins as dominant
        assert_eq!(p.dominant_signal, Some(StyleSignal::CodeSmells));
        assert_eq!(p.infer_personality_type(), "The Enterprise Bureaucrat");
    }

    // ── SignalDetector — PanicAddictionDetector ────────────────────

    use crate::treesitter::engine::TreeSitterEngine;

    fn parse_rust(code: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(std::path::Path::new("test.rs"), code)
            .expect("Rust parse should succeed")
    }

    /// Objective: Verify violations_to_score returns 0 for 0 violations.
    #[test]
    fn test_violations_to_score_zero() {
        let score = violations_to_score(0, 1000);
        assert!(
            (score - 0.0).abs() < f64::EPSILON,
            "0 violations => score 0, got {score}"
        );
    }

    /// Objective: Verify violations_to_score increases with more violations.
    #[test]
    fn test_violations_to_score_increasing() {
        let low = violations_to_score(1, 1000);
        let high = violations_to_score(10, 1000);
        assert!(
            high > low,
            "more violations => higher score, {high} <= {low}"
        );
    }

    /// Objective: Verify violations_to_score caps at 25.0.
    #[test]
    fn test_violations_to_score_capped() {
        let score = violations_to_score(1_000_000, 1);
        assert!(score <= 25.0, "score should be capped at 25, got {score}");
    }

    /// Objective: Verify aggregate_detector_scores works across multiple files.
    #[test]
    fn test_aggregate_detector_scores() {
        let files = vec![parse_rust("fn a() { let x = v.unwrap(); }")];
        let detectors: Vec<Box<dyn SignalDetector>> = vec![Box::new(PanicAddictionDetector::new())];
        let scores = aggregate_detector_scores(&detectors, &files);
        let panic_score = scores
            .get(&StyleSignal::PanicAddiction)
            .copied()
            .unwrap_or(0.0);
        assert!(
            panic_score > 0.0,
            "PanicAddiction score should be > 0, got {panic_score}"
        );
        assert!(
            panic_score <= 25.0,
            "PanicAddiction score should be <= 25, got {panic_score}"
        );
    }
}
