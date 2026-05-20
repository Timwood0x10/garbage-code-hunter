# Garbage Code Hunter v0.2.2 — Bootstrap Test Report

**Date:** 2026-05-18
**Version:** 0.2.2
**Rust:** stable
**Platform:** macOS (Darwin 24.6.0)

---

## Test Summary

| Metric | Value |
|--------|-------|
| Total tests | 796 |
| Passed | 796 |
| Failed | 0 |
| Ignored | 0 |
| Total time | ~37s |

## Test Suites

| Suite | Tests | Time | Description |
|-------|-------|------|-------------|
| lib (unit tests) | 699 | 10.3s | Core library unit tests |
| cli_tests | 17 | 3.3s | CLI argument parsing and output |
| coverage_tests | 10 | 0.1s | Coverage metric tests |
| edge_cases | 12 | 0.1s | Edge case handling |
| edge_cases_comprehensive | 16 | 0.2s | Comprehensive edge cases |
| integration_automated | 13 | 21.8s | Automated integration tests |
| integration_tests | 11 | 0.2s | Integration tests |
| reporter_tests | 12 | 0.1s | Reporter output tests |
| unit_tests | 6 | 0.0s | i18n and roast message tests |

## Clippy

```
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Result:** 0 warnings, 0 errors.

## Key Fixes in This Session

### Critical / High
- Registered 5 missing tree-sitter rules (TerribleNamingRule, SingleLetterTsRule, DeepNestingRule, PrintlnDebuggingRule, MagicNumberRule)
- Registered unwrap-abuse rule for Rust
- Registered TooManyParamsRule, HungarianNotationTsRule, AbbreviationAbuseTsRule
- Fixed LongFunctionRule `{}` placeholder never substituted
- Fixed RustMustUseRule duplicate condition and multi-line signature handling
- Fixed RustErrorDisplayRule hardcoded line number
- Fixed NaN handling in QualityLevel::from_score
- Fixed UTF-8 string slicing panic in reporter
- Fixed 0-param functions counted as 1 across all 11 adapters

### Medium
- RustErrorDisplayRule now handles generic impls (`impl<T> Debug for Foo<T>`)
- RustDeriveOrderRule now handles multi-line `#[derive(...)]`
- Java has_annotation now checks the correct line (above method)
- TS prefer-interface now only flags object types (not unions/primitives)
- Python .format() check now skips comments and strings
- Magic number rule now handles Rust `1_000_000` underscored literals
- Added C/C++ support for PrintlnDebuggingRule
- Added Java constructor support for LongFunctionRule/GodFunctionRule
- Fixed LongFunctionRule message doubling line count

### Low / Cleanup
- Removed dead "true"/"false" checks in Rust magic number
- Removed dead "-1" check in Python magic number
- Removed dead Go skip list entries (multi-char in len==1 filter)
- Removed unused _prod_issues allocation in reporter
- Deduplicated BLOCK_PARENT_TYPES constant
- Deduplicated direct_signals() across all language arms
- Gated slow integration tests behind GCH_INTEGRATION env var

## Supported Languages (11)

Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, Ruby, Swift, Zig

## Analysis Rules (registered)

### Common (all languages)
- terrible-naming, single-letter-variable, deep-nesting, println-debugging, magic-number
- hungarian-notation, abbreviation-abuse
- long-function, god-function, complex-closure

### Rust-specific
- unnecessary-clone, unwrap-abuse, async-abuse, macro-abuse, lifetime-abuse
- trait-complexity, generic-abuse, pattern-matching-abuse, box-abuse
- reference-abuse, slice-abuse, module-complexity, too-many-params
- string-abuse, vec-abuse
- rust-doc-example, rust-derive-order, rust-error-display, rust-must-use
