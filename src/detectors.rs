//! Direct signal detectors for the StyleSignal system.
//!
//! Each detector implements `SignalDetector` and produces scores
//! directly from parsed AST files, bypassing the Rule → Issue pipeline.

use crate::language::adapter::{LanguageAdapter, RustAdapter};
use crate::language::Language;
use crate::signals::{SignalDetector, StyleSignal};
use crate::treesitter::engine::ParsedFile;

// ── PanicAddiction Detector ───────────────────────────────────────

/// Detects PanicAddiction signal: .unwrap(), .expect(), panic!() calls.
pub struct PanicAddictionDetector;

impl PanicAddictionDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PanicAddictionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for PanicAddictionDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::PanicAddiction
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        RustAdapter.count_panic_calls(file)
    }
}

// ── NamingChaos Detector ─────────────────────────────────────────

/// Detects NamingChaos signal: single-letter vars, terrible/meaningless names,
/// Hungarian notation, and abbreviation abuse.
pub struct NamingChaosDetector;

impl NamingChaosDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NamingChaosDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for NamingChaosDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::NamingChaos
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        RustAdapter.count_naming_violations(file)
    }
}

// ── NestedHell Detector ──────────────────────────────────────────

/// Detects NestedHell signal: deeply-nested block scopes (≥5 levels).
pub struct NestedHellDetector;

impl NestedHellDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NestedHellDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for NestedHellDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::NestedHell
    }

    fn supported_languages(&self) -> &'static [Language] {
        &[Language::Rust]
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        RustAdapter.count_deeply_nested_blocks(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use crate::treesitter::engine::{ParsedFile, TreeSitterEngine};

    fn parse_rust(source: &str) -> ParsedFile {
        let engine = TreeSitterEngine::new();
        engine
            .parse_file(std::path::Path::new("test.rs"), source)
            .expect("Rust parse should succeed")
    }

    // ── SignalDetector — PanicAddictionDetector ────────────────────

    /// Objective: Verify PanicAddictionDetector finds .unwrap() calls.
    #[test]
    fn test_detector_panic_unwrap() {
        let file = parse_rust("fn main() { let x = val.unwrap(); let y = other.unwrap(); }");
        let detector = PanicAddictionDetector::new();
        let count = detector.count_violations(&file);
        assert_eq!(count, 2, "should find 2 unwrap calls, got {count}");
    }

    /// Objective: Verify PanicAddictionDetector finds .expect() calls.
    #[test]
    fn test_detector_panic_expect() {
        let file = parse_rust("fn main() { let x = val.expect(\"msg\"); }");
        let detector = PanicAddictionDetector::new();
        let count = detector.count_violations(&file);
        assert_eq!(count, 1, "should find 1 expect call, got {count}");
    }

    /// Objective: Verify PanicAddictionDetector finds panic!() macro calls.
    #[test]
    fn test_detector_panic_macro() {
        let file = parse_rust(
            r#"
fn main() {
    panic!("something went wrong");
    panic!("another panic");
}
"#,
        );
        let detector = PanicAddictionDetector::new();
        let count = detector.count_violations(&file);
        assert_eq!(count, 2, "should find 2 panic!() calls, got {count}");
    }

    /// Objective: Verify detector finds mixed unwrap + expect + panic calls.
    #[test]
    fn test_detector_panic_mixed() {
        let file = parse_rust(
            r#"
fn main() {
    let a = x.unwrap();
    let b = y.expect("msg");
    panic!("boom");
}
"#,
        );
        let detector = PanicAddictionDetector::new();
        let count = detector.count_violations(&file);
        assert_eq!(count, 3, "should find 3 total violations, got {count}");
    }

    // ── SignalDetector — NamingChaosDetector ─────────────────────

    /// Objective: Verify NamingChaosDetector catches single-letter var.
    #[test]
    fn test_detector_naming_single_letter() {
        let file = parse_rust("fn main() { let x = 1; }");
        let detector = NamingChaosDetector::new();
        assert_eq!(detector.count_violations(&file), 1, "single-letter x");
    }

    /// Objective: Verify NamingChaosDetector catches terrible naming.
    #[test]
    fn test_detector_naming_terrible() {
        let file = parse_rust("fn main() { let data = 1; }");
        let detector = NamingChaosDetector::new();
        assert_eq!(detector.count_violations(&file), 1, "terrible name 'data'");
    }

    /// Objective: Verify NamingChaosDetector returns 0 for clean naming.
    #[test]
    fn test_detector_naming_clean() {
        let file = parse_rust("fn main() { let user_name = \"alice\"; }");
        let detector = NamingChaosDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "clean naming");
    }

    // ── SignalDetector — NestedHellDetector ──────────────────────

    /// Objective: Verify NestedHellDetector finds deeply-nested blocks.
    #[test]
    fn test_detector_nested_hell_deep() {
        let file = parse_rust(
            r#"
fn main() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            let x = 1;
                        }
                    }
                }
            }
        }
    }
}
"#,
        );
        let detector = NestedHellDetector::new();
        let count = detector.count_violations(&file);
        assert!(
            count >= 1,
            "6-level deep nesting should find at least 1 deeply-nested block, got {count}"
        );
    }

    /// Objective: Verify NestedHellDetector returns 0 for flat code.
    #[test]
    fn test_detector_nested_hell_flat() {
        let file = parse_rust(
            r#"
fn main() {
    let x = 1;
    let y = 2;
}
"#,
        );
        let detector = NestedHellDetector::new();
        assert_eq!(
            detector.count_violations(&file),
            0,
            "flat code should have 0 violations"
        );
    }

    /// Objective: Verify NestedHellDetector counts only blocks at depth >= 5.
    #[test]
    fn test_detector_nested_hell_just_under_threshold() {
        let file = parse_rust(
            r#"
fn main() {
    if true {
        if true {
            if true {
                if true {
                    let x = 1;
                }
            }
        }
    }
}
"#,
        );
        let detector = NestedHellDetector::new();
        assert_eq!(
            detector.count_violations(&file),
            0,
            "4-level nesting should be under threshold (5)"
        );
    }
}
