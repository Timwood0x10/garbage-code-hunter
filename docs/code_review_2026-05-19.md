# Code Review — garbage-code-hunter

Date: 2026-05-19  
Reviewer: Code review pass on `src/`  
Total findings: 13 (8 Medium, 5 Low)  
Excluded: false/phantom path reports, folder/content mismatches reported without line location

---

## MEDIUM (8 findings)

### M-1 · `src/language/adapter/mod.rs:151–224`
**`count_commented_out_code` / `count_todo_markers` — line-scan instead of AST scan**  
Both methods loop over `file.content.lines()`, so string literals containing `//` are counted as commented-out code and TODOs. Example: `let s = "// this is a string"` inflates the commented-lines counter. `CODEC_PATTERNS` additionally contains bare tokens like `"="`, `"::"`, `"->"` which cause false matches against comment content such as `// ======` or `/// -> callback`. No AST-level comment node traversal is performed.

---

### M-2 · `src/analyzer.rs:200–229 / 240–288`
**`analyze_to_findings` Phase 1/2/3 — test files not fully skipped**  
`is_test_file` is computed and stored in `effective_test`, but only passed to `ts_rule_engine.check_file_with_context`. Phases 2 (cross-file dup) and 3 (intra-file dup) iterate over `parsed_files` without any `is_test_file` guard, so test data still participates in fingerprinting and deduplication scoring. `analyze_full` (called by `main.rs:628`) has the same gap. `IntraFileDupDetector::check(parsed)` in `duplication.rs:396` calls `is_skippable` by path suffix only — it may miss tests recognized by path but through `#[cfg(test)]`-only parsing.

---

### M-3 · `src/language/adapter/c.rs:20–28` — also `cpp.rs:20`  
**`count_panic_calls` — `exit()` / `abort()` classified as panics**  
Tree-sitter query `(call_expression function: (identifier) @f (#match? @f "^(exit|abort)$"))` fires on every `exit(EXIT_FAILURE)` and `abort()` call in C/C++. These are normal idioms in `main()` and error handlers, not anti-patterns. The false positives flow into `StyleIrSummary.panic_call_count`, inflate `PanicAddiction` signal scores, and bias personality inference toward "The YOLO Engineer" for virtually every non-trivial C/C++ codebase. The autopsy `contributing_factors` (via `quick_fix_count`) are also inflated.

---

### M-4 · `src/llm/provider.rs:128–161`
**`extract_code_contexts` — silently drops file read errors**  
`std::fs::read_to_string(&path).ok()?` swallows all `io::Error` variants (permission denied, file deleted mid-run, encoding error) without logging. The caller proceeds to send the prompt with `(context unavailable)` placeholder text. LLM quality degrades silently and users have no indication that source file reads failed.

---

### M-5 · `src/autopsy/mod.rs:114–124`
**`density = total` — autopsy lifespan threshold mislabeled**  
`let density = total as f64;` holds the raw issue count, not a density-per-1k-LOC figure. The threshold chain `> 100 → 3 months`, `> 50 → 6 months`, `> 20 → 12 months` all operate on raw count. Any real codebase finding 20+ issues (extremely common) always emits "3 months — code is on life support." The variable name is misleading and the badge/lifespan message is nearly useless.

---

### M-6 · `src/analyze.rs:251 / 283–287 / src/analyze.rs:368–403`
**`for_signal` + CodeIssue findings double-count signals in `analyze_to_findings` vs `analyze_full`**  
Both pipelines (1) convert `CodeIssue` → `StyleFinding` via `From::from` (maps exact per-rule issues), then (2) push aggregate `StyleFinding::for_signal(signal, count, file_path)` from Phase 4 signal detectors. For files with `unwrap` issues (and other rules), `compute_signal_scores_from_findings` in `finding.rs:443` counts both raw `PanicAddiction` StyleFindings and aggregate `PanicAddiction` StyleFindings, roughly doubling the signal score for that file. `CodeScorer::calculate_score` in `scoring.rs:174` also separately calls `compute_signal_scores` on original `CodeIssue` list — three independent PanicAddiction score contributions exist for each pass.

---

### M-7 · `src/llm/prompt.rs:107`
**Prompt template hardcodes JSON brace with single braces**  
`format!` page uses `{{"0": "..."}}` — the outer braces are escaped as `{{`/`}}` for Rust's `format!`. If the format string is ever ported to a macro or template engine that doesn't double-brace-escape, the JSON placeholder becomes invalid. Current code path is fine; the maintenance risk is real.

---

### M-8 · `src/treesitter/duplication.rs:93–96`
**`normalize_node` Rust-keyword guard leaks into cross-language ASTs**  
The `_ => { return vec![] }` in the keyword match covers `"let" | "const" | "mut" | ... | "fn"`, all Rust tokens. For Go/JS/Python ASTs parsed by the same `normalize_node` (which is language-agnostic), Go's `const` declarations and JS's `function` keywords also match and get `return vec![]`, meaning the AST nodes below those keywords are never fingerprinted. In practice Go's `const` block content still gets fingerprinted because tree-sitter Go uses `const_declaration` not `const` as the direct child, and JS `function_declaration` similarly isn't a bare `function` node. The slipthrough is benign for typical use, but the guard boundary is misleading and should be language-qualified.

---

## LOW (5 findings)

### L-1 · `src/reporter/mod.rs:22–23`
**`top_files: usize` — dead field**  
Accepted in `Reporter::new()`, stored in the struct, annotated `#[expect(dead_code)]`, and registered in `i18n.rs:314` — but never read in `report_with_spread`, `report_with_metrics`, or `print_boss_file`. A planned "clip to top-N files" feature was never wired.

---

### L-2 · `src/reporter/translations.rs:85–86`
**`rule_display_name` — dead code**  
`#[expect(dead_code)]` on a 25-entry Chinese→English rule name mapping that has zero callers in the entire codebase. All lookup targets are also reachable via `i18n.get`. The mapping can go stale without any test or compile-time signal.

---

### L-3 · `src/llm/client.rs:56–63`
**`LlmConfig::from_args` — `unwrap_or` on user slices**  
`endpoint.unwrap_or(default_endpoint)` and `model.unwrap_or(default_model)` unwrap the `Option<&str>` from CLI args — this is safe because the `Option` is always `Some` or `None`, not in-band error. Via `new(LlmConfig{endpoint:"",model:"",...})` raw construction, no guard catches blank strings. The CLI path (`from_args`) has defaults so no runtime crash, only library-consumer misconstruction.

---

### L-4 · `src/autopsy/mod.rs:115`
**`density` variable name misrepresents its value (raw count)**  
`let density = total as f64;` is named `density` but holds `total issue count`. Not a logic error, but the misleading name combined with the M-5 threshold behavior means the three-branch lifespan logic is effectively a 2-branch raw-count broken by the "24+ months" fallback.

---

### L-5 · `src/language/adapter/{c.rs,cpp.rs}:184` — also `go.rs:245`, `js.rs:170`, `py.rs:401`, `java.rs:208`, `ruby.rs:210`, `ts.rs:170`, `zig.rs:178`, `swift.rs:187`
**`count_magic_numbers` — `false`/`true`/`NULL`/`nullptr` not whitelisted**  
`if text != "0" && text != "1" && text != "-1"` misses language-specific constant booleans and null pointers. C/C++'s `false`/`true`/`NULL`, Go's `nil`, Python's `True`/`False`, JS's `true`/`false`, Rust's `true`/`false` — all 9 adapters share the same 3-whitelist pattern, but only C/C++, Go, JS, Python, Ruby, TS, Zig, Swift, Rust are affected (all non-Rust adapters carry the narrower whitelist). Impact is Low because `is_common_safe_number` provides broad coverage and magic-number is not a primary signal scorer.

---
