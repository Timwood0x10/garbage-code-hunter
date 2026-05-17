//! Direct signal detectors for the StyleSignal system.
//!
//! Each detector implements `SignalDetector` and produces scores
//! directly from parsed AST files, bypassing the Rule → Issue pipeline.

use crate::language::adapter::adapter_for;
use crate::language::Language;
use crate::signals::{SignalDetector, StyleSignal};
use crate::treesitter::duplication::IntraFileDupDetector;
use crate::treesitter::engine::ParsedFile;

/// All languages that have a LanguageAdapter implementation.
const ADAPTER_LANGUAGES: &[Language] = &[
    Language::Rust,
    Language::Python,
    Language::JavaScript,
    Language::TypeScript,
    Language::Go,
    Language::Java,
    Language::Ruby,
    Language::Swift,
    Language::Zig,
    Language::C,
    Language::Cpp,
];

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
        ADAPTER_LANGUAGES
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        adapter_for(file.language)
            .map(|a| a.count_panic_calls(file))
            .unwrap_or(0)
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
        ADAPTER_LANGUAGES
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        adapter_for(file.language)
            .map(|a| a.count_naming_violations(file))
            .unwrap_or(0)
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
        ADAPTER_LANGUAGES
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        adapter_for(file.language)
            .map(|a| a.count_deeply_nested_blocks(file))
            .unwrap_or(0)
    }
}

// ── HotfixCulture Detector ────────────────────────────────────────

/// Detects HotfixCulture signal: println!, dbg!, todo!, unimplemented! calls.
pub struct HotfixCultureDetector;

impl HotfixCultureDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HotfixCultureDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for HotfixCultureDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::HotfixCulture
    }

    fn supported_languages(&self) -> &'static [Language] {
        ADAPTER_LANGUAGES
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        adapter_for(file.language)
            .map(|a| a.count_debug_calls(file))
            .unwrap_or(0)
    }
}

// ── OverEngineering Detector ─────────────────────────────────────

/// Detects OverEngineering signal: god functions (>50 lines) and excessive params (>5).
pub struct OverEngineeringDetector;

impl OverEngineeringDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OverEngineeringDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for OverEngineeringDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::OverEngineering
    }

    fn supported_languages(&self) -> &'static [Language] {
        ADAPTER_LANGUAGES
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        let Some(adapter) = adapter_for(file.language) else {
            return 0;
        };
        let param_threshold = 5;
        let god_threshold = 50;
        let mut count = 0;
        let functions = adapter.extract_functions(file);
        for f in &functions {
            if f.end_line - f.start_line > god_threshold {
                count += 1;
            }
        }
        count += adapter.count_excessive_params(file, param_threshold);
        count
    }
}

// ── CodeSmells Detector ────────────────────────────────────────────

/// Detects CodeSmells signal: unsafe blocks, magic numbers, unnecessary clone, etc.
pub struct CodeSmellsDetector;

impl CodeSmellsDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CodeSmellsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for CodeSmellsDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::CodeSmells
    }

    fn supported_languages(&self) -> &'static [Language] {
        ADAPTER_LANGUAGES
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        let Some(adapter) = adapter_for(file.language) else {
            return 0;
        };
        let unsafe_count = adapter.count_unsafe_blocks(file);
        let magic_count = adapter.count_magic_numbers(file);
        unsafe_count * 2 + magic_count
    }
}

// ── Duplication Detector ───────────────────────────────────────────

/// Detects Duplication signal: intra-file duplicated code blocks.
///
/// Cross-file duplication detection is stateful (accumulates fingerprints
/// across all files) and is handled separately in the analysis pipeline.
pub struct DuplicationDetector;

impl DuplicationDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DuplicationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for DuplicationDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::Duplication
    }

    fn supported_languages(&self) -> &'static [Language] {
        ADAPTER_LANGUAGES
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        IntraFileDupDetector::check(file).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // ── SignalDetector — HotfixCultureDetector ─────────────────────

    /// Objective: Verify HotfixCultureDetector counts println! calls.
    #[test]
    fn test_detector_hotfix_println() {
        let file = parse_rust(
            r#"
fn main() {
    println!("hello");
    println!("world");
}
"#,
        );
        let detector = HotfixCultureDetector::new();
        assert_eq!(detector.count_violations(&file), 2, "2 println! calls");
    }

    /// Objective: Verify HotfixCultureDetector counts todo! and unimplemented!.
    #[test]
    fn test_detector_hotfix_todo() {
        let file = parse_rust(
            r#"
fn main() {
    todo!("implement this");
    unimplemented!();
}
"#,
        );
        let detector = HotfixCultureDetector::new();
        assert_eq!(
            detector.count_violations(&file),
            2,
            "todo! + unimplemented! = 2"
        );
    }

    /// Objective: Verify HotfixCultureDetector returns 0 for clean code.
    #[test]
    fn test_detector_hotfix_clean() {
        let file = parse_rust(
            r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        );
        let detector = HotfixCultureDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "no debug calls");
    }

    /// Objective: Verify HotfixCultureDetector counts dbg! and eprintln! too.
    #[test]
    fn test_detector_hotfix_dbg_eprintln() {
        let file = parse_rust(
            r#"
fn main() {
    dbg!(42);
    eprintln!("error!");
    eprint!("warning!");
}
"#,
        );
        let detector = HotfixCultureDetector::new();
        assert_eq!(
            detector.count_violations(&file),
            3,
            "dbg! + eprintln! + eprint! = 3"
        );
    }

    // ── SignalDetector — OverEngineeringDetector ──────────────────

    /// Objective: Verify OverEngineeringDetector counts god functions (>50 lines).
    #[test]
    fn test_detector_overengineering_god_function() {
        let file = parse_rust(
            r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
}
"#,
        );
        let detector = OverEngineeringDetector::new();
        // The main function is short (< 50 lines), no god functions
        assert_eq!(
            detector.count_violations(&file),
            0,
            "short function should not count as overengineered"
        );
    }

    /// Objective: Verify OverEngineeringDetector counts excessive params (>5).
    #[test]
    fn test_detector_overengineering_excessive_params() {
        let file = parse_rust(
            r#"
fn process(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    a + b + c + d + e + f
}
"#,
        );
        let detector = OverEngineeringDetector::new();
        assert_eq!(
            detector.count_violations(&file),
            1,
            "function with 6 params should count as violation"
        );
    }

    /// Objective: Verify OverEngineeringDetector is 0 for clean functions.
    #[test]
    fn test_detector_overengineering_clean() {
        let file = parse_rust(
            r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        );
        let detector = OverEngineeringDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "clean function");
    }

    // ── SignalDetector — CodeSmellsDetector ──────────────────────

    /// Objective: Verify CodeSmellsDetector finds unsafe blocks.
    #[test]
    fn test_detector_code_smells_unsafe() {
        let file = parse_rust(
            r#"
fn main() {
    unsafe {
        let p = 42 as *const i32;
        let _ = *p;
    }
}
"#,
        );
        let detector = CodeSmellsDetector::new();
        let count = detector.count_violations(&file);
        assert!(
            count >= 2,
            "unsafe block (2 points) should be >= 2, got {count}"
        );
    }

    /// Objective: Verify CodeSmellsDetector counts magic numbers in expressions.
    #[test]
    fn test_detector_code_smells_magic() {
        let file = parse_rust(
            r#"
fn main() {
    let x = 1;
    foo(42);
    bar(100);
}
"#,
        );
        let detector = CodeSmellsDetector::new();
        assert_eq!(detector.count_violations(&file), 2, "two magic numbers = 2");
    }

    /// Objective: Verify CodeSmellsDetector skips numbers in const/let declarations.
    #[test]
    fn test_detector_code_smells_const_ok() {
        let file = parse_rust(
            r#"
const MAX: i32 = 100;
fn main() {
    let x = MAX;
}
"#,
        );
        let detector = CodeSmellsDetector::new();
        assert_eq!(
            detector.count_violations(&file),
            0,
            "const value and no-magic should be 0"
        );
    }

    /// Objective: Verify CodeSmellsDetector skips 0 and 1 in trivial expressions.
    #[test]
    fn test_detector_code_smells_trivial_numbers_ok() {
        let file = parse_rust(
            r#"
fn main() {
    let x = 0;
    let y = x + 1;
}
"#,
        );
        let detector = CodeSmellsDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "0 and 1 not magic");
    }

    /// Objective: Verify CodeSmellsDetector returns 0 for clean code.
    #[test]
    fn test_detector_code_smells_clean() {
        let file = parse_rust(
            r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        );
        let detector = CodeSmellsDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "clean code = 0");
    }

    // ── SignalDetector — DuplicationDetector ─────────────────────

    /// Objective: Verify DuplicationDetector finds intra-file duplication.
    #[test]
    fn test_detector_duplication_intra_file() {
        let file = parse_rust(
            r#"
fn setup_a() {
    let x = 1;
    let y = 2;
    let z = 3;
    let w = 4;
    let v = 5;
}
fn setup_b() {
    let x = 1;
    let y = 2;
    let z = 3;
    let w = 4;
    let v = 5;
}
"#,
        );
        let detector = DuplicationDetector::new();
        let count = detector.count_violations(&file);
        assert!(count >= 1, "duplicated blocks should be >= 1, got {count}");
    }

    /// Objective: Verify DuplicationDetector returns 0 for clean code.
    #[test]
    fn test_detector_duplication_clean() {
        let file = parse_rust(
            r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn sub(a: i32, b: i32) -> i32 { a - b }
"#,
        );
        let detector = DuplicationDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "no duplication");
    }

    /// Objective: Verify DuplicationDetector returns 0 for short files (<10 lines).
    #[test]
    fn test_detector_duplication_short_file() {
        let file = parse_rust("fn main() { let x = 1; }");
        let detector = DuplicationDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "short file = 0");
    }
}
