# Static Analysis Review — garbage-code-hunter

**Date:** 2026-05-19  
**Scope:** Passive code reading only — no modifications made  
**Test baseline:** All 807 tests passing (699 lib + 17 signal + 10 reporter + 12 display + 16 rust_rules + 13 cli)

---

## 🔴 Critical

### 1. TypeScript parser uses TSX grammar instead of TypeScript grammar

**File:** `src/treesitter/parsers.rs:21`

```rust
Language::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
```

`LANGUAGE_TSX` is the grammar for `.tsx` files (TypeScript + JSX). Vanilla `.ts` files must use `LANGUAGE_TYPESCRIPT`. The two grammars differ in node types — TSX includes JSX node kinds (`jsx_element`, `jsx_identifier`, etc.) and produces a different AST structure for the same TypeScript source.

**Impact:** Every `.ts` file parsed by the engine gets an AST built from the wrong grammar. All TypeScript rules (naming, magic-number, terrible-naming, abbreviation-abuse, type issues, etc.) match against incorrect node types, so their counts are unreliable.

---

## 🟡 High

### 2. `summarize_style_ir_files` `code_smell_count` formula is truncated

**File:** `src/helpers.rs:509` vs `src/style_ir/mod.rs:181–192`

| Term | `StyleIr::code_smell_count()` | `helpers::summarize_style_ir_files()` |
|---|---|---|
| `unsafe_block_count * 2` | ✅ | ✅ |
| `magic_number_count` | ✅ | ✅ |
| `go_convention_count` | ✅ | ❌ |
| `python_issue_count` | ✅ | ❌ |
| `java_issue_count` | ✅ | ❌ |
| `ruby_issue_count` | ✅ | ❌ |
| `c_issue_count` | ✅ | ❌ |
| `ts_issue_count` | ✅ | ❌ |
| `dead_code_count` | ✅ | ❌ |
| `duplicate_import_count` | ✅ | ❌ |

`helpers.rs` keeps only `unsafe * 2 + magic_number`, discarding 8 language-specific terms.

**Impact:** The aggregated JSON summary output silently drops the vast majority of code-smell findings for Go, Python, Java, Ruby, C, and TypeScript. Those languages will report near-zero `code_smell_count` even when the per-file `StyleIr` detected many real issues.

---

## 🟡 Medium

### 3. `summarize_style_ir_files` `over_engineering_count` drops `goroutine_spawn_count`

**File:** `src/helpers.rs:508` vs `src/style_ir/mod.rs:176–178`

```rust
// helpers.rs (incomplete)
over_engineering_count: god_function_count + excessive_param_count,

// style_ir/mod.rs (canonical)
pub fn over_engineering_count(&self) -> usize {
    self.god_function_count() + self.excessive_param_count + self.goroutine_spawn_count
}
```

**Impact:** Go projects that use goroutines will have a lower `over_engineering_count` in the aggregated JSON output than the sum of their per-file IR summaries.

---

## 🔴 Dead Code / API Issues

### 4. `Reporter::savage_mode` field is stored but never read

**File:** `src/reporter/mod.rs:22, 53`

```rust
#[expect(dead_code)]
savage_mode: bool,
```

The field is accepted in `Reporter::new()` and stored, but no method ever reads it. The `#[expect(dead_code)]` annotation suppresses the compiler warning but does not fix the logic gap.

**Impact:** Callers that set `savage_mode = true` receive no different output — this is a silent API contract violation.

---

### 5. `CrossFileDupDetector::find_near_duplicates` is an empty stub

**File:** `src/treesitter/duplication.rs:215–217`

```rust
/// Find near-duplicate functions using Jaccard similarity on normalized tokens.
pub fn find_near_duplicates(&self) -> Vec<CodeIssue> {
    vec![]
}
```

The method unconditionally returns an empty vector. Jaccard-similarity near-duplicate detection is documented but never implemented. Callers at `src/analyzer.rs:239,357` always get zero results from this path.

**Impact:** Cross-file near-duplicate detection is a marketed "feature" that produces false-negative results for every project.

---

## 🟢 Low

### 6. `MacroRule` and `MethodCallRule` in `base_rules.rs` are dead code

**File:** `src/treesitter/rules/base_rules.rs:103–206`

`MacroRule` carries `#[allow(dead_code)]`; `MethodCallRule` has no attribute but is never instantiated or included in any rule registry. Both structs are exported helpers in a file whose purpose is "helper functions," but they implement `TreeSitterRule` as if they were active rules.

**Impact:** No runtime effect, but it is misleading — readers of the code will assume macro-count and method-call-count features are active when they are not.

---

### 7. `StyleProfile` has two construction paths with incompatible value scales

**Files:** `src/personality/profiles.rs:37` (→ `from_signal_counts`) vs `src/signals.rs:320` (→ `from_signal_scores`)

`StyleProfile::from_signal_counts(counts)` stores `score = count as f64` (raw integer counts).  
`StyleProfile::from_signal_scores(scores)` stores already-normalized scores in the range `0.0–25.0`.

`StyleProfile::infer_personality_type()` uses thresholds like `>= 12.0` (high) and `>= 6.0` (medium), written for the normalized 0–25 scale. If a caller ever passes `from_signal_counts` data into a path that runs `infer_personality_type`, the thresholds will fire at completely wrong counts (e.g., 12 raw issues would already qualify as "high duplication" rather than needing hundreds).

**Impact:** Currently the two call-paths are separate (`profiles::analyze` uses `from_signal_counts` directly and never calls `infer_personality_type`; tests use `from_signal_scores`), so the mismatch is latent. Any future merge of the two paths would produce wrong personality classifications.

---

## 📋 Untracked / Empty Files in Editor

| File | Status |
|---|---|
| `src/rules/struct_patterns.rs` | Does not exist on disk |
| `src/rules/comprehensive_rust.rs` | Does not exist on disk |

Both appear as open tabs in the editor with no corresponding committed content.

---

## Additional Findings (Round 2)

---

### 8. Glob-to-regex conversion on exclude patterns produces substring false positives

**File:** `src/analyzer.rs:99–103`  
**Severity:** Medium

```rust
let regex_pattern = pattern
    .replace(".", r"\.")
    .replace("*", ".*")
    .replace("?", ".");
```

A pattern like `"build"` is converted to `"build.*"` — a regex that matches any path containing "build" as a substring, e.g. `"mybuild/foo.o"` or `"re_build/output/"`. The intent is to match the `build/` path component. No path-boundary anchor is applied.

**Impact:** A user writing `--exclude "build"` to skip the standard `build/` directory will also silently skip any file whose path happens to contain the substring "build".

---

### 9. `count_commented_out_code` — `"match "` false-positive on Rust/Rust comments

**File:** `src/language/adapter/mod.rs:214–218`  
**Severity:** Low

`CODEC_PATTERNS` contains `"match "` to detect commented-out code. The word "match" is a legitimate Rust keyword. A doc comment like `/// match behaviour: ergonomics first` or a comment `// match user.name` will be falsely flagged as code-like content.

**Impact:** Partially mitigated by the `MIN_COMMENT_LINES` (3 consecutive lines) guard, so a single line triggers nothing. But a real 3-line doc-comment block that happens to discuss the word "match" is over-counted as commented-out code, contributing to inflated code-smell counts.

---

### 10. `collect_captures` capture-index lookup — `unwrap_or_else` masks logic errors

**File:** `src/treesitter/query.rs:108–114`  
**Severity:** Low

```rust
let name_idx = match capture_names_index.get(capture_name.as_str()) {
    Some(&idx) => idx,
    None => {
        // Fallback to unknown rather than panicking
        return Ok(Capture {
            node: cap.node,
            name: "unknown".to_string(),
        });
    }
};
```

When named-capture-index lookup fails, the code silently falls back to `"unknown"` instead of surfacing a real query-compilation error. This hides bugs in custom-tree-sitter queries.

**Impact:** Zero-capture rules and misconfigured `check_with_context` handlers will silently return empty results, making it difficult to debug why a rule stops producing output.

---

### 11. `count_todo_markers` — single-byte `#` prefix `find()` allows non-comment `#TODO` matches in Python/Ruby

**File:** `src/language/adapter/mod.rs:190–211`  
**Severity:** Low

```rust
let pos = trimmed.find(line_comment).unwrap_or(usize::MAX);
let content = trimmed[pos + line_comment.len()..].trim();
```

`find` returns the first occurrence of `#`. In a Python string literal or dict key like `tags = {"#TODO": "done"}`, `find("#TODO")` will extract `"TODO: \"done\"}"` and flag it as a marker, even though the `#` is not a comment. Only the *first* `#` matters here.

**Impact:** Non-comment Rust integer literals like `let x = 0x_DEAD_BEEF;` are not affected (that's handled by Rust adapter). But Python/Ruby string literals containing `#TODO` text will be under-counted, which is a *false negative* for the count.

---

### 12. `count_commented_out_code` — `starts_with("///")` guard silently skips `////` lines (extreme edge case)

**File:** `src/language/adapter/mod.rs:157–164`  
**Severity:** Low

The Rust doc-comment guard is `trimmed.starts_with("///")`. A line `//// some logic` (intentionally four slashes, used in some codebases to visually distinguish suppressed logic from real doc comments) matches `starts_with("///")` and is treated as a doc-comment, so it never enters the commented-out-code counter.

**Impact:** Real commented-out code lines starting with four slashes get silently dropped from `commented_out_lines`. Impact is cosmetic (one line missed) and occurs only in very specific stylistic edge cases.

---

### 13. `FriendFeedback::new_zh` embeds English `rule_name` inside Chinese `NextAction` strings

**File:** `src/friend/feedback.rs:267`  
**Severity:** Low

```rust
NextAction::from_issues_zh(&issues, score.total_score, &top_signals)
// …
fn from_issues_zh(issues: &[CodeIssue], score: f64, signals: &[StyleSignal]) -> Self {
    // …
    format!("修复 '{}'", issue.rule_name)
```

`NextAction::from_issues_zh` uses the Chinese verb "修复" but embeds `issue.rule_name` verbatim (always English, e.g. `"unwrap-abuse"`). The surrounding UI labels are Chinese; the rule names are English.

**Impact:** The Chinese-output path produces mixed-language strings, creating a UX inconsistency visible to `--lang zh-CN` users.

---

### 14. `top_files` field on `Reporter` — `#[expect(dead_code)]` is **correctly applied**

**File:** `src/reporter/mod.rs:22, 40, 51`  
**Severity:** N/A (annotation accurate)

Confirmed: `self.top_files` is set in `new()` but never read in any method body. The `#[expect(dead_code)]` annotation on line 22 correctly acknowledges intentional dead storage. No bug here.

---

### 15. `rule_display_name` in `translations.rs` — `#[expect(dead_code)]` is **correctly applied**

**File:** `src/reporter/translations.rs:85–114`  
**Severity:** N/A (annotation accurate)

Confirmed: `rule_display_name` is defined as `pub(super)fn` and contains an elaborate match block. A full codebase search (`grep -rn "rule_display_name"`) returns only the definition site. The `#[expect(dead_code)]` annotation on line 85 correctly classifies it as intentionally dead code.

---

### 16. `CODEC_PATTERNS` — `"match "` pattern also fires on commented-out Rust match-arms

**File:** `src/language/adapter/mod.rs:214–218`  
**Severity:** Info

This is a duplicate note (related to Finding 9) with a slightly different angle: commented-out `match` arms like `// match x {` will be counted in the code-like score **twice** — once by the `"match "` pattern and again by any `"{"` or `"}"` patterns in the same line. This does not cause a crash but marginally inflates `commented_out_lines`.

**Impact:** Over-counting of commented-out code lines for blocks containing match syntax; bounded by the `MIN_COMMENT_LINES` gate.

---

### Summary Table (All Findings)

| # | File | Lines | Issue | Severity |
|---|------|-------|-------|----------|
| 1 | `src/treesitter/parsers.rs` | 21 | TS mapped to TSX grammar | 🔴 Critical |
| 2 | `src/helpers.rs` | 509 | `code_smell_count` truncated (8 terms missing) | 🟡 High |
| 3 | `src/helpers.rs` | 508 | `over_engineering_count` drops `goroutine_spawn_count` | 🟡 Medium |
| 4 | `src/reporter/mod.rs` | 22, 53 | `savage_mode` stored but never read | 🔴 API Gap |
| 5 | `src/treesitter/duplication.rs` | 215–217 | `find_near_duplicates` always returns `[]` | 🟡 Stub |
| 6 | `src/treesitter/rules/base_rules.rs` | 103–206 | `MacroRule` / `MethodCallRule` dead code | 🟢 Low |
| 7 | `src/personality/profiles.rs` + `src/signals.rs` | 37–55, 320–374 | Two StyleProfile paths with incompatible value scales | 🟢 Latent |
| 8 | `src/analyzer.rs` | 99–103 | Glob-to-regex substring collision on exclude patterns | 🟡 Medium |
| 9 | `src/language/adapter/mod.rs` | 214–218 | `"match "` false-positive in commented-out code detection | 🟡 Low |
| 10 | `src/treesitter/query.rs` | 108–114 | `unwrap_or_else("unknown")` on capture index masks logic errors | 🟡 Low |
| 11 | `src/language/adapter/mod.rs` | 190–211 | `find("#")` allows non-comment `#TODO` matches in Python/Ruby | 🟢 Low |
| 12 | `src/language/adapter/mod.rs` | 157–164 | `starts_with("///")` silently skips `////` lines | 🟢 Low |
| 13 | `src/friend/feedback.rs` | 267 | `from_issues_zh` embeds English `rule_name` in Chinese strings | 🟢 Low |
| 14 | `src/reporter/mod.rs` | 22, 40, 51 | `top_files` `#[expect(dead_code)]` confirmed correct | — ✓ |
| 15 | `src/reporter/translations.rs` | 85–114 | `rule_display_name` `#[expect(dead_code)]` confirmed correct | — ✓ |
| 16 | `src/language/adapter/mod.rs` | 214–218 | Commented-out `match` arms over-counted by two patterns | ℹ Info |
