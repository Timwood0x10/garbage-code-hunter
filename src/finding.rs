//! StyleFinding — the structured representation of a code style observation.
//!
//! Evolves from the raw `CodeIssue` to include confidence, evidence,
//! category context, and actionable suggestions. This is the standard
//! data model for all outputs (terminal, JSON, Markdown, CI, etc.).

use crate::analyzer::{CodeIssue, Severity};
use crate::signals::{classify_rule, StyleProfile, StyleSignal};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

// ── Identity ─────────────────────────────────────────────────────

/// Unique identifier for a finding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FindingId(String);

impl FindingId {
    pub fn new(seed: &str) -> Self {
        let mut hasher = std::hash::DefaultHasher::new();
        seed.hash(&mut hasher);
        Self(format!("{:016x}", hasher.finish())[..12].to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ── Location ─────────────────────────────────────────────────────

/// Location in source code with optional span and symbol name.
#[derive(Debug, Clone)]
pub struct CodeLocation {
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub span: Option<TextSpan>,
    pub symbol_name: Option<String>,
}

/// A contiguous range of text in source code.
#[derive(Debug, Clone)]
pub struct TextSpan {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

// ── Rule Metadata ────────────────────────────────────────────────

/// Meta information about the rule that triggered the finding.
#[derive(Debug, Clone)]
pub struct RuleMeta {
    pub name: String,
    pub category: StyleCategory,
    pub intent: RuleIntent,
}

/// Code style categories — what aspect of the code is being evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleCategory {
    Naming,
    Complexity,
    Duplication,
    Comments,
    DebuggingLeftovers,
    Structure,
    Consistency,
    DependencyStyle,
}

impl StyleCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            StyleCategory::Naming => "Naming",
            StyleCategory::Complexity => "Complexity",
            StyleCategory::Duplication => "Duplication",
            StyleCategory::Comments => "Comments",
            StyleCategory::DebuggingLeftovers => "Debugging Leftovers",
            StyleCategory::Structure => "Structure",
            StyleCategory::Consistency => "Consistency",
            StyleCategory::DependencyStyle => "Dependency Style",
        }
    }
}

/// Intent behind a rule — why this rule exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleIntent {
    Readability,
    Maintainability,
    TeamConvention,
    NoiseReduction,
    CognitiveLoad,
}

impl RuleIntent {
    pub fn display_name(&self) -> &'static str {
        match self {
            RuleIntent::Readability => "Readability",
            RuleIntent::Maintainability => "Maintainability",
            RuleIntent::TeamConvention => "Team Convention",
            RuleIntent::NoiseReduction => "Noise Reduction",
            RuleIntent::CognitiveLoad => "Cognitive Load",
        }
    }
}

// ── Confidence ──────────────────────────────────────────────────

/// How confident we are that this is a real issue vs. a false positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    pub fn display_name(&self) -> &'static str {
        match self {
            Confidence::Low => "Low",
            Confidence::Medium => "Medium",
            Confidence::High => "High",
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            Confidence::Low => 0.3,
            Confidence::Medium => 0.6,
            Confidence::High => 1.0,
        }
    }
}

// ── Evidence ────────────────────────────────────────────────────

/// Evidence supporting the finding.
#[derive(Debug, Clone)]
pub struct Evidence {
    pub snippet: Option<String>,
    pub metric: Option<EvidenceMetric>,
    pub nearby_context: Vec<String>,
}

/// A numeric metric with threshold comparison.
#[derive(Debug, Clone)]
pub struct EvidenceMetric {
    pub name: String,
    pub value: f64,
    pub threshold: f64,
    pub unit: String,
}

// ── Suggestion ──────────────────────────────────────────────────

/// A suggested fix or improvement for the finding.
#[derive(Debug, Clone)]
pub struct StyleSuggestion {
    pub title: String,
    pub explanation: String,
    pub quick_fix_hint: Option<String>,
    pub safer_alternative: Option<String>,
}

// ── StyleFinding ────────────────────────────────────────────────

/// The core finding model — a structured observation about code style.
///
/// Each finding represents one detected issue with full metadata
/// for presentation, filtering, and educational feedback.
#[derive(Debug, Clone)]
pub struct StyleFinding {
    pub id: FindingId,
    pub location: CodeLocation,
    pub rule: RuleMeta,
    pub signal: StyleSignal,
    pub severity: Severity,
    pub confidence: Confidence,
    pub evidence: Evidence,
    pub suggestion: Option<StyleSuggestion>,
}

impl StyleFinding {
    /// Create a finding from a signal-level detection (no single-line location).
    pub fn for_signal(signal: StyleSignal, violation_count: usize, file_path: PathBuf) -> Self {
        let id = FindingId::new(&format!(
            "signal:{:?}:{violation_count}:{}",
            signal,
            file_path.display()
        ));
        let severity = if violation_count > 10 {
            Severity::Nuclear
        } else if violation_count > 3 {
            Severity::Spicy
        } else {
            Severity::Mild
        };
        StyleFinding {
            id,
            location: CodeLocation {
                file_path,
                line: 0,
                column: 0,
                span: None,
                symbol_name: None,
            },
            rule: RuleMeta {
                name: signal.display_name().to_string(),
                category: StyleCategory::Consistency,
                intent: RuleIntent::Maintainability,
            },
            signal,
            severity,
            confidence: Confidence::High,
            evidence: Evidence {
                snippet: Some(format!(
                    "{violation_count} {} violations",
                    signal.display_name()
                )),
                metric: Some(EvidenceMetric {
                    name: "violations".to_string(),
                    value: violation_count as f64,
                    threshold: 0.0,
                    unit: "count".to_string(),
                }),
                nearby_context: Vec::new(),
            },
            suggestion: None,
        }
    }

    /// Convert back to a `CodeIssue` for downstream consumers that
    /// still depend on the legacy issue model.
    pub fn to_code_issue(&self) -> CodeIssue {
        CodeIssue {
            file_path: self.location.file_path.clone(),
            line: self.location.line,
            column: self.location.column,
            rule_name: self.rule.name.clone(),
            message: self
                .evidence
                .snippet
                .clone()
                .unwrap_or_else(|| format!("{:?}", self.signal)),
            severity: self.severity.clone(),
        }
    }
}

impl From<&CodeIssue> for StyleFinding {
    fn from(issue: &CodeIssue) -> Self {
        let id = FindingId::new(&format!(
            "{}:{}:{}:{}",
            issue.file_path.display(),
            issue.line,
            issue.rule_name,
            issue.message,
        ));
        let signal = classify_rule(&issue.rule_name);
        let location = CodeLocation {
            file_path: issue.file_path.clone(),
            line: issue.line,
            column: issue.column,
            span: None,
            symbol_name: None,
        };
        let rule = RuleMeta {
            name: issue.rule_name.clone(),
            category: rule_to_category(&issue.rule_name),
            intent: rule_to_intent(&issue.rule_name),
        };
        let confidence = rule_to_confidence(&issue.rule_name);
        let evidence = Evidence {
            snippet: Some(issue.message.clone()),
            metric: None,
            nearby_context: Vec::new(),
        };

        StyleFinding {
            id,
            location,
            rule,
            signal,
            severity: issue.severity.clone(),
            confidence,
            evidence,
            suggestion: None,
        }
    }
}

// ── Category / Intent / Confidence mapping ──────────────────────

fn rule_to_category(rule_name: &str) -> StyleCategory {
    match rule_name {
        n if n.contains("naming")
            || n.contains("letter")
            || n.contains("hungarian")
            || n.contains("abbreviation")
            || n.contains("name")
            || n.contains("predicate") =>
        {
            StyleCategory::Naming
        }
        n if n.contains("nest")
            || n.contains("complex")
            || n.contains("function_length")
            || n.contains("long-function")
            || n.contains("god-function")
            || n.contains("too-many-params")
            || n.contains("module-complexity")
            || n.contains("trait-complexity") =>
        {
            StyleCategory::Complexity
        }
        n if n.contains("duplicat") => StyleCategory::Duplication,
        n if n.contains("todo")
            || n.contains("fixme")
            || n.contains("commented")
            || n.contains("dead-code") =>
        {
            StyleCategory::Comments
        }
        n if n.contains("println")
            || n.contains("unwrap")
            || n.contains("panic")
            || n.contains("except")
            || n.contains("rescue") =>
        {
            StyleCategory::DebuggingLeftovers
        }
        n if n.contains("file-too-long")
            || n.contains("module-nesting")
            || n.contains("import") =>
        {
            StyleCategory::Structure
        }
        n if n.contains("generic") || n.contains("magic") || n.contains("constant-name") => {
            StyleCategory::Consistency
        }
        _ => StyleCategory::Consistency,
    }
}

fn rule_to_intent(rule_name: &str) -> RuleIntent {
    match rule_name {
        n if n.contains("naming")
            || n.contains("letter")
            || n.contains("name")
            || n.contains("hungarian")
            || n.contains("abbreviation") =>
        {
            RuleIntent::Readability
        }
        n if n.contains("nest")
            || n.contains("complex")
            || n.contains("long-function")
            || n.contains("god-function")
            || n.contains("too-many-params") =>
        {
            RuleIntent::CognitiveLoad
        }
        n if n.contains("duplicat") => RuleIntent::Maintainability,
        n if n.contains("todo")
            || n.contains("fixme")
            || n.contains("commented")
            || n.contains("dead-code") =>
        {
            RuleIntent::NoiseReduction
        }
        n if n.contains("println")
            || n.contains("unwrap")
            || n.contains("panic")
            || n.contains("except")
            || n.contains("rescue") =>
        {
            RuleIntent::NoiseReduction
        }
        n if n.contains("magic") || n.contains("constant-name") || n.contains("generic") => {
            RuleIntent::Maintainability
        }
        _ => RuleIntent::Readability,
    }
}

fn rule_to_confidence(rule_name: &str) -> Confidence {
    // High-confidence: objectively measurable
    if matches!(
        rule_name,
        "magic-number"
            | "code-duplication"
            | "cross-file-duplication"
            | "unwrap-abuse"
            | "panic-abuse"
            | "empty-catch"
            | "bare-except"
            | "bare-rescue"
            | "println-debugging"
            | "dead-code"
            | "commented-code"
            | "file-too-long"
    ) {
        return Confidence::High;
    }

    // Medium-confidence: measurable but context-dependent
    if matches!(
        rule_name,
        "deep-nesting"
            | "cyclomatic-complexity"
            | "complex-closure"
            | "long-function"
            | "god-function"
            | "too-many-params"
            | "module-complexity"
            | "trait-complexity"
            | "todo-comment"
            | "todo-fixme"
            | "todo-bug"
            | "todo-hack"
    ) {
        return Confidence::Medium;
    }

    // Low-confidence: subjective / style preference
    Confidence::Low
}

// ── Universal Style IR ──────────────────────────────────────────
//
// These functions build StyleProfile directly from StyleFinding[*].signal,
// without needing classify_rule(). This is the language-agnostic Style IR
// path — any language that produces StyleFindings gets signal scoring and
// personality inference for free.

/// Compute signal scores from findings using each finding's `.signal` field
/// (language-agnostic, no classify_rule() needed).
pub fn compute_signal_scores_from_findings(
    findings: &[StyleFinding],
    total_lines: usize,
) -> HashMap<StyleSignal, f64> {
    let k_lines = (total_lines as f64 / 1000.0).max(0.001);
    let mut counts: HashMap<StyleSignal, usize> = HashMap::new();

    for finding in findings {
        *counts.entry(finding.signal).or_insert(0) += 1;
    }

    let mut scores = HashMap::new();
    for signal in StyleSignal::all() {
        let count = counts.get(signal).copied().unwrap_or(0);
        let density = count as f64 / k_lines;
        let score = ((density + 1.0).log2() * 6.0).min(25.0);
        scores.insert(*signal, score);
    }

    scores
}

/// Build a StyleProfile directly from findings (language-agnostic).
pub fn build_profile_from_findings(findings: &[StyleFinding], total_lines: usize) -> StyleProfile {
    let signal_scores = compute_signal_scores_from_findings(findings, total_lines);
    StyleProfile::from_signal_scores(signal_scores)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_issue(rule: &str) -> CodeIssue {
        CodeIssue {
            file_path: PathBuf::from("src/main.rs"),
            line: 42,
            column: 5,
            rule_name: rule.to_string(),
            message: format!("{rule} detected"),
            severity: Severity::Spicy,
        }
    }

    // ── FindingId ─────────────────────────────────────────────────

    /// Objective: Verify FindingId is 12-char hex from blake3 hash.
    #[test]
    fn test_finding_id_format() {
        let id = FindingId::new("test-seed");
        assert_eq!(id.as_str().len(), 12, "id should be 12 hex chars");
        assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Objective: Verify same seed produces same id.
    #[test]
    fn test_finding_id_deterministic() {
        let a = FindingId::new("hello");
        let b = FindingId::new("hello");
        assert_eq!(a, b);
    }

    /// Objective: Verify different seeds produce different ids.
    #[test]
    fn test_finding_id_distinct() {
        let a = FindingId::new("hello");
        let b = FindingId::new("world");
        assert_ne!(a, b);
    }

    // ── Category mapping ─────────────────────────────────────────

    /// Objective: Verify naming-related rules map to StyleCategory::Naming.
    #[test]
    fn test_category_naming() {
        for name in &[
            "terrible-naming",
            "single-letter-variable",
            "meaningless-naming",
            "hungarian-notation",
            "abbreviation-abuse",
            "go-receiver-name",
            "ruby-predicate-method",
        ] {
            assert_eq!(
                rule_to_category(name),
                StyleCategory::Naming,
                "{name} should be Naming"
            );
        }
    }

    /// Objective: Verify complexity rules map to Complexity.
    #[test]
    fn test_category_complexity() {
        for name in &[
            "deep-nesting",
            "cyclomatic-complexity",
            "complex-closure",
            "long-function",
            "god-function",
            "too-many-params",
            "module-complexity",
        ] {
            assert_eq!(
                rule_to_category(name),
                StyleCategory::Complexity,
                "{name} should be Complexity"
            );
        }
    }

    /// Objective: Verify duplication rules map to Duplication.
    #[test]
    fn test_category_duplication() {
        assert_eq!(
            rule_to_category("code-duplication"),
            StyleCategory::Duplication
        );
        assert_eq!(
            rule_to_category("cross-file-duplication"),
            StyleCategory::Duplication
        );
    }

    // ── Confidence mapping ───────────────────────────────────────

    /// Objective: Verify objectively measurable rules return High confidence.
    #[test]
    fn test_confidence_high() {
        for name in &[
            "magic-number",
            "code-duplication",
            "unwrap-abuse",
            "empty-catch",
            "dead-code",
        ] {
            assert_eq!(
                rule_to_confidence(name),
                Confidence::High,
                "{name} should be High confidence"
            );
        }
    }

    /// Objective: Verify context-dependent rules return Medium confidence.
    #[test]
    fn test_confidence_medium() {
        for name in &[
            "deep-nesting",
            "long-function",
            "todo-comment",
            "too-many-params",
        ] {
            assert_eq!(
                rule_to_confidence(name),
                Confidence::Medium,
                "{name} should be Medium confidence"
            );
        }
    }

    /// Objective: Verify subjective rules return Low confidence.
    #[test]
    fn test_confidence_low() {
        for name in &[
            "terrible-naming",
            "single-letter-variable",
            "abbreviation-abuse",
            "go-receiver-name",
        ] {
            assert_eq!(
                rule_to_confidence(name),
                Confidence::Low,
                "{name} should be Low confidence"
            );
        }
    }

    // ── StyleFinding conversion ──────────────────────────────────

    /// Objective: Verify From<&CodeIssue> produces correct fields.
    #[test]
    fn test_finding_from_issue_basic() {
        let issue = make_issue("unwrap-abuse");
        let finding = StyleFinding::from(&issue);
        assert_eq!(finding.location.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(finding.location.line, 42);
        assert_eq!(finding.rule.name, "unwrap-abuse");
        assert_eq!(finding.signal, StyleSignal::PanicAddiction);
        assert_eq!(finding.severity, Severity::Spicy);
        assert_eq!(finding.confidence, Confidence::High);
    }

    /// Objective: Verify finding ID is deterministic for same input.
    #[test]
    fn test_finding_id_from_issue_deterministic() {
        let issue = make_issue("deep-nesting");
        let a = StyleFinding::from(&issue).id;
        let b = StyleFinding::from(&issue).id;
        assert_eq!(a, b, "same issue should produce same finding id");
    }

    /// Objective: Verify rule_to_category falls back to Consistency for unmatched rules.
    #[test]
    fn test_category_fallback_consistency() {
        let cat = rule_to_category("zzz-unmatched-rule");
        assert_eq!(cat, StyleCategory::Consistency);
    }

    /// Objective: Verify rule_to_intent handles known and unknown rules.
    #[test]
    fn test_intent_mapping() {
        assert_eq!(rule_to_intent("deep-nesting"), RuleIntent::CognitiveLoad);
        assert_eq!(rule_to_intent("unwrap-abuse"), RuleIntent::NoiseReduction);
        assert_eq!(
            rule_to_intent("code-duplication"),
            RuleIntent::Maintainability
        );
        assert_eq!(rule_to_intent("terrible-naming"), RuleIntent::Readability);
    }

    /// Objective: Verify Confidence score values.
    #[test]
    fn test_confidence_score_values() {
        assert!((Confidence::High.score() - 1.0).abs() < f64::EPSILON);
        assert!((Confidence::Medium.score() - 0.6).abs() < f64::EPSILON);
        assert!((Confidence::Low.score() - 0.3).abs() < f64::EPSILON);
    }

    /// Objective: Verify display_name methods return non-empty strings.
    #[test]
    fn test_display_names_non_empty() {
        for cat in &[
            StyleCategory::Naming,
            StyleCategory::Complexity,
            StyleCategory::Duplication,
            StyleCategory::Comments,
            StyleCategory::DebuggingLeftovers,
            StyleCategory::Structure,
            StyleCategory::Consistency,
            StyleCategory::DependencyStyle,
        ] {
            assert!(
                !cat.display_name().is_empty(),
                "{:?} display_name empty",
                cat
            );
        }
        for intent in &[
            RuleIntent::Readability,
            RuleIntent::Maintainability,
            RuleIntent::TeamConvention,
            RuleIntent::NoiseReduction,
            RuleIntent::CognitiveLoad,
        ] {
            assert!(
                !intent.display_name().is_empty(),
                "{:?} display_name empty",
                intent
            );
        }
        for conf in &[Confidence::Low, Confidence::Medium, Confidence::High] {
            assert!(
                !conf.display_name().is_empty(),
                "{:?} display_name empty",
                conf
            );
        }
    }

    /// Objective: Verify evidence snippet comes from issue message.
    #[test]
    fn test_evidence_snippet() {
        let issue = make_issue("deep-nesting");
        let finding = StyleFinding::from(&issue);
        assert_eq!(
            finding.evidence.snippet.as_deref(),
            Some("deep-nesting detected")
        );
    }

    /// Objective: Verify suggestion is None by default for From<CodeIssue>.
    #[test]
    fn test_suggestion_default_none() {
        let issue = make_issue("long-function");
        let finding = StyleFinding::from(&issue);
        assert!(finding.suggestion.is_none());
    }

    // ── Universal Style IR ────────────────────────────────────────

    fn finding(signal: StyleSignal, severity: Severity) -> StyleFinding {
        let issue = CodeIssue {
            file_path: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            rule_name: format!("{:?}", signal).to_lowercase(),
            message: "test".to_string(),
            severity,
        };
        // Override signal since rule_name won't map correctly
        let mut f = StyleFinding::from(&issue);
        f.signal = signal;
        f
    }

    /// Objective: Verify compute_signal_scores_from_findings counts signals correctly.
    #[test]
    fn test_signal_scores_from_findings() {
        let findings = vec![
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Spicy),
            finding(StyleSignal::PanicAddiction, Severity::Mild),
        ];
        let scores = compute_signal_scores_from_findings(&findings, 1000);
        assert!(
            *scores.get(&StyleSignal::Duplication).unwrap_or(&0.0)
                > *scores.get(&StyleSignal::PanicAddiction).unwrap_or(&0.0),
            "Duplication (2 count) should score higher than PanicAddiction (1 count)"
        );
    }

    /// Objective: Verify compute_signal_scores_from_findings returns 0 for empty input.
    #[test]
    fn test_signal_scores_from_findings_empty() {
        let scores = compute_signal_scores_from_findings(&[], 1000);
        for signal in StyleSignal::all() {
            assert_eq!(
                scores.get(signal).copied().unwrap_or(0.0),
                0.0,
                "empty findings => all signal scores 0"
            );
        }
    }

    /// Objective: Verify build_profile_from_findings produces correct dominant signal.
    #[test]
    fn test_build_profile_from_findings_dominant() {
        let findings = vec![
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::NestedHell, Severity::Mild),
        ];
        let profile = build_profile_from_findings(&findings, 1000);
        assert_eq!(
            profile.dominant_signal,
            Some(StyleSignal::Duplication),
            "3 duplicates vs 1 nested => Duplication should be dominant"
        );
    }

    /// Objective: Verify build_profile_from_findings allows personality inference.
    #[test]
    fn test_build_profile_from_findings_personality() {
        let findings = vec![
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
            finding(StyleSignal::Duplication, Severity::Nuclear),
        ];
        let profile = build_profile_from_findings(&findings, 100);
        let personality = profile.infer_personality_type();
        assert_eq!(
            personality, "The Copy-Paste Artist",
            "massive duplication => Copy-Paste Artist, got {personality}"
        );
    }
}
