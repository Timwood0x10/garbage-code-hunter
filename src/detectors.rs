//! Direct signal detectors for the StyleSignal system.
//!
//! Each detector implements `SignalDetector` and produces scores
//! directly from parsed AST files, bypassing the Rule → Issue pipeline.

use crate::language::Language;
use crate::signals::{SignalDetector, StyleSignal};
use crate::style_ir::StyleIr;
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
        StyleIr::from_parsed(file)
            .map(|ir| ir.panic_call_count)
            .unwrap_or(0)
    }

    fn count_violations_with_ir(&self, ir: &StyleIr, _file: &ParsedFile) -> usize {
        ir.panic_call_count
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
        StyleIr::from_parsed(file)
            .map(|ir| ir.naming_violation_count)
            .unwrap_or(0)
    }

    fn count_violations_with_ir(&self, ir: &StyleIr, _file: &ParsedFile) -> usize {
        ir.naming_violation_count
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
        StyleIr::from_parsed(file)
            .map(|ir| ir.deeply_nested_block_count + ir.defer_in_loop_count)
            .unwrap_or(0)
    }

    fn count_violations_with_ir(&self, ir: &StyleIr, _file: &ParsedFile) -> usize {
        ir.deeply_nested_block_count + ir.defer_in_loop_count
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
        StyleIr::from_parsed(file)
            .map(|ir| ir.debug_call_count)
            .unwrap_or(0)
    }

    fn count_violations_with_ir(&self, ir: &StyleIr, _file: &ParsedFile) -> usize {
        ir.debug_call_count
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
        StyleIr::from_parsed(file)
            .map(|ir| ir.over_engineering_count())
            .unwrap_or(0)
    }

    fn count_violations_with_ir(&self, ir: &StyleIr, _file: &ParsedFile) -> usize {
        ir.over_engineering_count()
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
        StyleIr::from_parsed(file)
            .map(|ir| ir.code_smell_count())
            .unwrap_or(0)
    }

    fn count_violations_with_ir(&self, ir: &StyleIr, _file: &ParsedFile) -> usize {
        ir.code_smell_count()
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

// ── LegacyCode Detector ─────────────────────────────────────────────

/// Detects LegacyCode signal: blocks of commented-out code left in source.
pub struct LegacyCodeDetector;

impl LegacyCodeDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LegacyCodeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for LegacyCodeDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::LegacyCode
    }

    fn supported_languages(&self) -> &'static [Language] {
        ADAPTER_LANGUAGES
    }

    fn skips_test_files(&self) -> bool {
        false
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        StyleIr::from_parsed(file)
            .map(|ir| ir.commented_out_lines)
            .unwrap_or(0)
    }

    fn count_violations_with_ir(&self, ir: &StyleIr, _file: &ParsedFile) -> usize {
        ir.commented_out_lines
    }
}

// ── TodoMountain Detector ────────────────────────────────────────────

/// Detects TodoMountain signal: TODO/FIXME/BUG/HACK markers in comments.
pub struct TodoMountainDetector;

impl TodoMountainDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TodoMountainDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for TodoMountainDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::TodoMountain
    }

    fn supported_languages(&self) -> &'static [Language] {
        ADAPTER_LANGUAGES
    }

    fn skips_test_files(&self) -> bool {
        false
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        StyleIr::from_parsed(file)
            .map(|ir| ir.todo_count)
            .unwrap_or(0)
    }

    fn count_violations_with_ir(&self, ir: &StyleIr, _file: &ParsedFile) -> usize {
        ir.todo_count
    }
}

// ── LineCountSmell Detector ──────────────────────────────────────────

/// Detects LineCountSmell signal: files exceeding reasonable line thresholds.
pub struct LineCountSmellDetector;

impl LineCountSmellDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LineCountSmellDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDetector for LineCountSmellDetector {
    fn signal(&self) -> StyleSignal {
        StyleSignal::LineCountSmell
    }

    fn supported_languages(&self) -> &'static [Language] {
        ADAPTER_LANGUAGES
    }

    fn skips_test_files(&self) -> bool {
        false
    }

    fn count_violations(&self, file: &ParsedFile) -> usize {
        let line_count = file.content.lines().count();
        let is_test = file.path.to_string_lossy().contains("test");
        let threshold = if is_test { 2000 } else { 1000 };
        if line_count > threshold {
            line_count
        } else {
            0
        }
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
        let file = parse_rust("fn main() { let a = 1; }");
        let detector = NamingChaosDetector::new();
        assert_eq!(detector.count_violations(&file), 1, "single-letter a");
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

    // ── SignalDetector — LegacyCodeDetector ─────────────────────

    /// Objective: Verify LegacyCodeDetector finds consecutive commented-out code lines.
    #[test]
    fn test_detector_legacy_code_block() {
        let file = parse_rust(
            r#"
fn main() {
    // let x = 1;
    // let y = 2;
    // let z = 3;
    // let w = x + y;
    // foo(w);
    bar();
}
"#,
        );
        let detector = LegacyCodeDetector::new();
        assert!(
            detector.count_violations(&file) >= 5,
            "5 consecutive commented-out lines should be >= 5"
        );
    }

    /// Objective: Verify LegacyCodeDetector ignores doc comments.
    #[test]
    fn test_detector_legacy_code_doc_ok() {
        let file = parse_rust(
            r#"
/// Documented function
fn documented() -> i32 {
    // normal comment
    42
}
"#,
        );
        let detector = LegacyCodeDetector::new();
        assert_eq!(
            detector.count_violations(&file),
            0,
            "doc comment + 1 normal = 0"
        );
    }

    /// Objective: Verify LegacyCodeDetector returns 0 for short comments.
    #[test]
    fn test_detector_legacy_code_short_ok() {
        let file = parse_rust(
            r#"
// short
// comments
// are fine
fn main() {}
"#,
        );
        let detector = LegacyCodeDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "short comments = 0");
    }

    /// Objective: Verify LegacyCodeDetector is 0 for empty code.
    #[test]
    fn test_detector_legacy_code_empty() {
        let file = parse_rust("// just a single comment");
        let detector = LegacyCodeDetector::new();
        assert_eq!(detector.count_violations(&file), 0);
    }

    /// Objective: Verify LegacyCodeDetector catches 4+ consecutive commented lines.
    #[test]
    fn test_detector_legacy_code_four_lines() {
        let file = parse_rust(
            r#"
// fn old() {
//     do_thing();
//     let x = 1;
//     x
// }
fn new() {}
"#,
        );
        let detector = LegacyCodeDetector::new();
        assert!(
            detector.count_violations(&file) >= 4,
            "4 consecutive commented lines"
        );
    }

    // ── SignalDetector — TodoMountainDetector ────────────────────

    /// Objective: Verify TodoMountainDetector counts TODO markers.
    #[test]
    fn test_detector_todo_basic() {
        let file = parse_rust(
            r#"
// TODO: refactor
// FIXME: fix this
fn main() {}
"#,
        );
        let detector = TodoMountainDetector::new();
        assert_eq!(detector.count_violations(&file), 2);
    }

    /// Objective: Verify TodoMountainDetector counts BUG and HACK too.
    #[test]
    fn test_detector_todo_bug_hack() {
        let file = parse_rust(
            r#"
// BUG: critical issue here
// HACK: workaround
fn main() {}
"#,
        );
        let detector = TodoMountainDetector::new();
        assert_eq!(detector.count_violations(&file), 2);
    }

    /// Objective: Verify TodoMountainDetector returns 0 for clean code.
    #[test]
    fn test_detector_todo_clean() {
        let file = parse_rust(
            r#"
fn main() {
    let x = 1;
}
"#,
        );
        let detector = TodoMountainDetector::new();
        assert_eq!(detector.count_violations(&file), 0);
    }

    /// Objective: Verify TodoMountainDetector is case-insensitive.
    #[test]
    fn test_detector_todo_case_insensitive() {
        let file = parse_rust(
            r#"
// todo: lowercase
// fixme: lowercase
fn main() {}
"#,
        );
        let detector = TodoMountainDetector::new();
        assert!(
            detector.count_violations(&file) >= 2,
            "case-insensitive TODO + FIXME"
        );
    }

    /// Objective: Verify TodoMountainDetector counts inline markers.
    #[test]
    fn test_detector_todo_inline() {
        let file = parse_rust(
            r#"
fn main() {
    let x = 1; // TODO: use constant
    let y = 2; // FIXME: off by one
}
"#,
        );
        let detector = TodoMountainDetector::new();
        assert_eq!(detector.count_violations(&file), 2);
    }

    // ── SignalDetector — LineCountSmellDetector ──────────────────

    fn create_rust_file(lines: usize) -> String {
        let mut s = String::from("fn main() {\n");
        for i in 0..lines.saturating_sub(2) {
            s.push_str(&format!("    let x_{} = {};\n", i, i));
        }
        s.push_str("}\n");
        s
    }

    /// Objective: Verify LineCountSmellDetector signals for large files.
    #[test]
    fn test_detector_linecount_over_threshold() {
        let code = create_rust_file(1100);
        let engine = TreeSitterEngine::new();
        let file = engine
            .parse_file(std::path::Path::new("lib.rs"), &code)
            .expect("parse should work");
        let detector = LineCountSmellDetector::new();
        assert!(
            detector.count_violations(&file) > 0,
            "1100-line file should trigger smell"
        );
    }

    /// Objective: Verify LineCountSmellDetector does NOT signal small files.
    #[test]
    fn test_detector_linecount_under_threshold() {
        let code = create_rust_file(100);
        let file = parse_rust(&code);
        let detector = LineCountSmellDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "100-line file = 0");
    }

    /// Objective: Verify LineCountSmellDetector uses higher threshold for test files.
    #[test]
    fn test_detector_linecount_test_threshold() {
        use std::path::Path;
        let code = create_rust_file(1100);
        let engine = TreeSitterEngine::new();
        let file = engine
            .parse_file(Path::new("test_suite.rs"), &code)
            .expect("parse should work");
        let detector = LineCountSmellDetector::new();
        assert_eq!(
            detector.count_violations(&file),
            0,
            "1100-line test file = 0 (test threshold = 2000)"
        );
    }

    /// Objective: Verify LineCountSmellDetector crosses test threshold.
    #[test]
    fn test_detector_linecount_test_over_threshold() {
        use std::path::Path;
        let code = create_rust_file(2100);
        let engine = TreeSitterEngine::new();
        let file = engine
            .parse_file(Path::new("test_suite.rs"), &code)
            .expect("parse should work");
        let detector = LineCountSmellDetector::new();
        assert!(
            detector.count_violations(&file) > 0,
            "2100-line test file should trigger smell"
        );
    }

    // ── SignalDetector — NamingChaosDetector additional ──────────

    /// Objective: Verify NamingChaosDetector catches Hungarian notation.
    #[test]
    fn test_detector_naming_hungarian() {
        let file = parse_rust(
            r#"
fn main() {
    let strName = String::new();
    let intCount = 42;
}
"#,
        );
        let detector = NamingChaosDetector::new();
        assert!(
            detector.count_violations(&file) >= 2,
            "Hungarian notation vars"
        );
    }

    /// Objective: Verify NamingChaosDetector catches non-idiomatic single-letter.
    #[test]
    fn test_detector_naming_non_idiomatic_single() {
        let file = parse_rust(
            r#"
fn main() {
    let z = 1;
    let q = 2;
}
"#,
        );
        let detector = NamingChaosDetector::new();
        assert_eq!(detector.count_violations(&file), 2, "z + q = 2");
    }

    /// Objective: Verify NamingChaosDetector exempts idiomatic single-letter vars.
    #[test]
    fn test_detector_naming_idiomatic_single_ok() {
        let file = parse_rust(
            r#"
fn main() {
    let i = 0;
    let j = 1;
    let n = 100;
}
"#,
        );
        let detector = NamingChaosDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "i, j, n are idiomatic");
    }

    // ── SignalDetector — PanicAddictionDetector additional ───────

    /// Objective: Verify PanicAddictionDetector returns 0 for safe code.
    #[test]
    fn test_detector_panic_clean() {
        let file = parse_rust(
            r#"
fn safe() -> Result<i32, String> {
    Ok(42)
}
"#,
        );
        let detector = PanicAddictionDetector::new();
        assert_eq!(detector.count_violations(&file), 0, "safe code = 0");
    }

    /// Objective: Verify PanicAddictionDetector finds .unwrap() in closures.
    #[test]
    fn test_detector_panic_unwrap_in_closure() {
        let file = parse_rust(
            r#"
fn main() {
    let result = (0..10).filter(|x| x.unwrap() > 0);
}
"#,
        );
        let detector = PanicAddictionDetector::new();
        assert_eq!(detector.count_violations(&file), 1, "unwrap in closure");
    }

    // ── SignalDetector — OverEngineeringDetector additional ──────

    /// Objective: Verify OverEngineeringDetector counts god function with >50 lines.
    #[test]
    fn test_detector_overengineering_god_function_55_lines() {
        let mut code = String::from("fn god() {\n");
        for i in 0..53 {
            code.push_str(&format!("    let var_{} = {};\n", i, i));
        }
        code.push_str("}\n");
        let file = parse_rust(&code);
        let detector = OverEngineeringDetector::new();
        assert!(
            detector.count_violations(&file) >= 1,
            "55-line god function should count"
        );
    }

    // ── SignalDetector — CodeSmellsDetector additional ───────────

    /// Objective: Verify CodeSmellsDetector handles empty functions.
    #[test]
    fn test_detector_code_smells_empty_fn() {
        let file = parse_rust("fn empty() {}");
        let detector = CodeSmellsDetector::new();
        assert_eq!(detector.count_violations(&file), 0);
    }

    /// Objective: Verify CodeSmellsDetector counts multiply-imported items.
    #[test]
    fn test_detector_code_smells_duplicate_import() {
        let file = parse_rust(
            r#"
use std::collections::HashMap;
use std::collections::HashMap;
fn main() {}
"#,
        );
        let detector = CodeSmellsDetector::new();
        assert!(
            detector.count_violations(&file) >= 1,
            "duplicate import should be > 0"
        );
    }

    /// Objective: Verify signal() returns the correct StyleSignal variant.
    #[test]
    fn test_detector_panic_signal_type() {
        assert_eq!(
            PanicAddictionDetector::new().signal(),
            StyleSignal::PanicAddiction
        );
    }

    #[test]
    fn test_detector_naming_signal_type() {
        assert_eq!(
            NamingChaosDetector::new().signal(),
            StyleSignal::NamingChaos
        );
    }

    #[test]
    fn test_detector_nested_signal_type() {
        assert_eq!(NestedHellDetector::new().signal(), StyleSignal::NestedHell);
    }

    #[test]
    fn test_detector_hotfix_signal_type() {
        assert_eq!(
            HotfixCultureDetector::new().signal(),
            StyleSignal::HotfixCulture
        );
    }

    #[test]
    fn test_detector_overeng_signal_type() {
        assert_eq!(
            OverEngineeringDetector::new().signal(),
            StyleSignal::OverEngineering
        );
    }

    #[test]
    fn test_detector_code_smells_signal_type() {
        assert_eq!(CodeSmellsDetector::new().signal(), StyleSignal::CodeSmells);
    }

    #[test]
    fn test_detector_legacy_signal_type() {
        assert_eq!(LegacyCodeDetector::new().signal(), StyleSignal::LegacyCode);
    }

    #[test]
    fn test_detector_todo_signal_type() {
        assert_eq!(
            TodoMountainDetector::new().signal(),
            StyleSignal::TodoMountain
        );
    }

    #[test]
    fn test_detector_linecount_signal_type() {
        assert_eq!(
            LineCountSmellDetector::new().signal(),
            StyleSignal::LineCountSmell
        );
    }

    #[test]
    fn test_detector_duplication_signal_type() {
        assert_eq!(
            DuplicationDetector::new().signal(),
            StyleSignal::Duplication
        );
    }
}
