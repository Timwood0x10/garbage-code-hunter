# Garbage Code Hunter — Audit TODO

## CRITICAL — Compilation Failures

- [x] **C1.** `src/autopsy/mod.rs:158` — `categorize()` match missing LegacyCode, TodoMountain, LineCountSmell
- [x] **C2.** `src/friend/feedback.rs:76` — `BehaviorPattern::from_signals()` match missing 3 variants
- [x] **C3.** `src/scoring.rs:222` — `legacy_category_name()` match missing 3 variants
- [x] **C4.** `src/signals.rs:130` — `display_name()` missing 3 variants
- [x] **C5.** `src/signals.rs:144` — `display_name_zh()` missing 3 variants

## HIGH — Panic / Serious Logic Errors

- [x] **H1.** `src/reporter/display.rs:699` — UTF-8 string slicing panic on multi-byte chars
- [x] **H2.** All 11 adapters `count_excessive_params()` — 0-param functions counted as 1
- [ ] **H3.** `src/treesitter/engine.rs:64` — Mutex held during entire parse, blocks parallelism
- [x] **H4.** `src/treesitter/rules/complex_rules.rs:151` — LongFunctionRule `{}` placeholder never substituted
- [x] **H5.** `src/treesitter/rules/rust_rules.rs:505` — RustMustUseRule duplicate condition (copy-paste)
- [x] **H6.** `src/treesitter/rules/rust_rules.rs:499` — RustMustUseRule misses multi-line signatures
- [ ] **H7.** `src/language/adapter/js.rs:26` + `ts.rs:26` — extract_functions misses arrow functions
- [ ] **H8.** `src/language/adapter/cpp.rs:29` — extract_functions misses class methods; debug_calls misses cout
- [x] **H9.** `src/treesitter/rules/common_rules.rs` — TerribleNamingRule, SingleLetterTsRule, DeepNestingRule, PrintlnDebuggingRule, MagicNumberRule never registered
- [x] **H10.** `src/treesitter/rules/rust_rules.rs` — unwrap-abuse rule never registered

## MEDIUM — Logic Errors / False Positives

- [x] **M1.** `src/signals.rs:647,658,704,720` — Test assertions `sigs.len() == 7` stale (should be 10)
- [x] **M2.** `src/signals.rs:188` — `direct_signals()` missing LineCountSmell per language
- [ ] **M3.** `src/signals.rs:315` — `classify_rule()` implicit fallback for magic-number, unnecessary-clone, etc.
- [x] **M4.** `src/style_ir/mod.rs:159` — `is_clean_signal_baseline()` unchecked commented_out_lines/todo_count
- [x] **M5.** `src/scoring.rs:43` — `QualityLevel::from_score` maps NaN to Excellent
- [ ] **M6.** `src/context/project_config.rs:38` — Silent config parse failure, no user feedback
- [x] **M7.** `src/treesitter/rules/rust_rules.rs:469` — RustErrorDisplayRule hardcoded line 1
- [x] **M8.** `src/treesitter/rules/rust_rules.rs:450` — RustErrorDisplayRule misses generic impls
- [x] **M9.** `src/treesitter/rules/rust_rules.rs:384` — RustDeriveOrderRule misses multi-line derives
- [x] **M10.** `src/language/adapter/go.rs:170` — count_debug_calls double-counts panic()
- [x] **M11.** `src/language/adapter/java.rs:236` — has_annotation checks wrong line
- [x] **M12.** `src/language/adapter/ts.rs:188` — type_alias flagged as "should be interface" for all types
- [x] **M13.** `src/language/adapter/python.rs:442` — .format() check matches strings/comments

## LOW — Performance / Code Quality

- [x] **L1.** `src/treesitter/rules/complex_rules.rs:413` — Regex recompiled per file, use LazyLock
- [x] **L2.** `src/treesitter/rules/complex_rules.rs:1075` — Rust `1_000_000` parse::<f64>() fails
- [x] **L3.** `src/treesitter/rules/complex_rules.rs:612` — Go skip list dead code (multi-char in len==1 filter)
- [x] **L4.** `src/language/adapter/rust.rs:237` — dead "true"/"false" checks in magic number
- [x] **L5.** `src/language/adapter/python.rs:392` — dead "-1" check in magic number
- [x] **L8.** `src/language/adapter/ruby.rs:192` — missing -1 exemption (now consistent — -1 is dead in all adapters)
- [ ] **L6.** `src/treesitter/duplication.rs:215` — find_near_duplicates() unimplemented stub
- [x] **L7.** `src/reporter/mod.rs:118` — unused _prod_issues allocation
- [ ] **L8.** `src/language/adapter/ruby.rs:192` — missing -1 exemption (inconsistent with other adapters)
