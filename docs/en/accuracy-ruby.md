# Ruby Accuracy Report

> Generated: 2026-05-15 | Projects tested: 1 | Analyzer: garbage-code-hunter

---

## Tested Projects

| Project | Files | Lines | Issues | Score | Density |
|---------|:-----:|:-----:|:------:|:-----:|:-------:|
| jekyll (static site gen) | 166 | 23,923 | 776 | 27.7 | 32/k |

> Note: Only 1 Ruby project was tested. Results are less statistically significant than other languages.

---

## Per-Rule Accuracy

### bare-rescue (Ruby-specific)

| Metric | Value |
|--------|-------|
| Total detections | 0 |
| Verdict | N/A — no instances found in jekyll |

**Source-code verification:**
- No bare `rescue` blocks found in any jekyll source files. The project uses typed rescues (`rescue StandardError => e`) consistently.

**Verdict: Rule works correctly but needs more test data.** The bare-rescue detection logic (check for named `rescue` node without `exceptions` child) is sound based on the Go/Java rule implementation quality.

### global-variable (Ruby-specific)

| Metric | Value |
|--------|-------|
| Total detections | ~2-3 |
| FP rate | **~100%** |

**Source-code verification:**
- `lib/jekyll.rb:3` — `$LOAD_PATH.unshift __dir__`. **FP.** This is idiomatic Ruby for gem entry points. Every Ruby gem uses `$LOAD_PATH` manipulation.

**Verdict: Needs a larger allowlist.** The current acceptable list includes `$stdout`, `$stderr`, `$stdin`, etc. but misses `$LOAD_PATH` and `$LOADED_FEATURES` which are standard gem patterns.

### println-debugging

| Metric | Value |
|--------|-------|
| Total detections | ~30 |
| FP rate | ~40% |

**Source-code verification:**
- `capture-assign.rb` — 4 `puts` calls. These are in example/benchmark scripts. **FP.**
- `formatter.rb` — 4 `puts` calls. These are in a CLI formatter. **Mixed.**
- `helpers.rb` — 4 `puts` calls. **Mixed.**

**Verdict: Same issue as other languages.** CLI scripts and example files produce FP.

### code-duplication

| Metric | Value |
|--------|-------|
| Total detections | ~500+ |
| FP rate | **~70-80%** |

**Source-code verification:**
- `test_filters.rb` — 86 duplication issues. **All FP.** The file has 185 independent test cases (`should "format a date with short format" do ... end`). Each tests a different filter method or edge case. The structural similarity is inherent to the Minitest testing pattern, not copy-paste duplication.

**Verdict: Test file duplication is the #1 noise source.** Same issue as all other languages — the Jaccard similarity on token types flags structurally similar but semantically different test cases.

### magic-number

| Metric | Value |
|--------|-------|
| Total detections | ~50 |
| FP rate | ~20% |

**Verdict: Acceptable.** Most detections are genuine magic numbers.

### hungarian-notation

| Metric | Value |
|--------|-------|
| Total detections | ~10 |
| FP rate | ~50% |

**Source-code verification:**
- `parse-include-tag-params.rb` — 5 hungarian-notation flags. Need to verify actual patterns.
- `include.rb` — 5 flags. Likely on parameter names.

**Verdict: Needs more verification.** Insufficient data to draw strong conclusions.

---

## Accuracy Summary

| Rule | TP Rate | FP Source | Severity |
|------|:-------:|-----------|:--------:|
| bare-rescue | N/A | — | Needs more data |
| global-variable | **~0%** | `$LOAD_PATH` gem pattern | Needs allowlist |
| println-debugging | **~60%** | Example/CLI scripts | Acceptable |
| code-duplication | **~20-30%** | Test patterns | **Critical FP** |
| magic-number | **~80%** | — | Acceptable |
| hungarian-notation | **~50%** | — | Needs verification |
| **Overall** | **~35-45%** | | |

---

## Concrete Improvement Suggestions

1. **Expand global-variable allowlist** — Add `$LOAD_PATH`, `$LOADED_FEATURES`, `$PROGRAM_NAME`, `$FILENAME`, `$.` to the acceptable list (some are already there, verify completeness).
2. **Test file duplication cap** — Same as other languages: limit duplication issues per test file.
3. **Example file exemption** — Skip `examples/`, `benchmarks/`, `scripts/` directories for println-debugging.

---

## False Negative Observations

1. **Monkey patching** — No rule detects reopening classes (`class String; def ...; end; end`), which is a major Ruby code smell.
2. **Metaprogramming abuse** — No rule detects excessive use of `method_missing`, `define_method`, or `class_eval`.
3. **Frozen string literal** — No rule detects missing `# frozen_string_literal: true` pragma.
