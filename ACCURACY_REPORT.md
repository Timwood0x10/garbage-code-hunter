# Garbage Code Hunter — Accuracy Review

> Updated: 2026-05-19 | Version: v0.2.2 | Status: evidence-corrected

---

## Goal

Provide a conservative, code-backed accuracy report that separates verified facts from unverified benchmark claims.

---

## Executive Summary

The previous version of this report overstated confidence in overall accuracy. The false-positive patterns described here are real, but the headline TP rates are not fully supported by the visible sample data.

Key corrections:

- The listed matrix contains **14 projects**, not 23.
- The visible Rust sample is **24 TP / 38 sampled = 63% raw TP**, not 87%.
- The visible Go sample is **16 TP / 40 sampled = 40% raw TP**, not 54%.
- Weighted TP rates require per-rule volume and sampling weights; those inputs are not included here, so weighted claims are marked as **unverified**.
- Performance improvements are plausible from the current implementation, but exact timings require a reproducible benchmark command and dataset.

Current assessment: the analyzer has useful high-confidence structural rules, but noisy style rules still create substantial false positives. It should be treated as **CI-advisory**, not a definitive production quality gate without project-specific configuration.

---

## Impact Analysis

| Area | Status | Risk |
|------|--------|------|
| Documentation only | This report updates claims and recommendations; no analyzer behavior changes | Low |
| User expectations | Accuracy claims become more conservative and easier to verify | Low |
| Release messaging | “Production-ready” wording is removed until backed by reproducible evidence | Medium |

---

## Evidence Status

| Claim | Previous Wording | Current Status | Reason |
|-------|------------------|----------------|--------|
| Project count | 23 projects | **Incorrect** | The table lists 14 projects |
| Rust TP rate | ~87% | **Unverified / overstated** | Visible sample is 63% raw TP |
| Go TP rate | ~54% | **Unverified / overstated** | Visible sample is 40% raw TP |
| FP patterns | Known FP patterns | **Supported** | Current rules still lack enough context for several cases |
| Performance 18x | 25.4s → 1.4s | **Plausible but unverified here** | Code contains caching/reuse, but this report lacks reproduction steps |
| Generated/dependency filtering | Needs generated-code handling | **Partially fixed** | Some generated/vendor paths are skipped; `.venv`, `venv`, `*.min.js`, `*.generated.*` need clearer default handling |

---

## Test Matrix

The following table is preserved as historical sample data. Counts and timings were not re-run as part of this documentation correction.

### Rust Projects (9 listed)

| Project | Lines | Functions | Issues | Signals | Time |
|---------|------:|----------:|-------:|--------:|-----:|
| algo | 413 | 30 | 0 | 4 | 0.5s |
| AlgoGpuRust | 3,668 | 190 | 49 | 34 | 0.8s |
| gpu-code | 394 | 9 | 42 | 14 | 0.5s |
| system_alert | 2,278 | 78 | 112 | 29 | 0.6s |
| ReChat-server | 3,972 | 173 | 167 | 43 | 0.7s |
| Finance | 26,467 | 760 | 1,769 | 216 | 1.6s |
| memscope-rs | 118,569 | 5,056 | 3,989 | 727 | 4.4s |
| memscope-stress-test | 864 | 4 | 64 | 16 | 0.5s |
| coq-of-rust | 20,343 | 1,032 | 560 | 496 | 1.3s |
| **Subtotal** | **176,968** | **7,332** | **6,752** | **1,579** | |

### Go Projects (4 listed)

| Project | Lines | Functions | Issues | Signals | Time |
|---------|------:|----------:|-------:|--------:|-----:|
| CodeTribunal | 7,133 | 320 | 476 | 78 | 0.7s |
| gnark | 153,424 | 5,617 | 11,646 | 2,125 | 6.7s |
| gaia | 23,068 | 616 | 423 | 288 | 1.3s |
| goagent | 743,199 | 27,145 | 29,885 | 4,109 | 28.0s |
| **Subtotal** | **926,824** | **33,698** | **42,430** | **6,700** | |

### Mixed / Other (1 listed)

| Project | Lines | Functions | Issues | Signals | Time | Lang |
|---------|------:|----------:|-------:|--------:|-----:|------|
| myblog | 51,111 | 180 | 3,408 | 230 | 7.3s | Mixed |

---

## TP/FP Analysis

### Methodology

Sampled issues were classified as:

- **TP (True Positive)**: genuine code quality issue.
- **FP (False Positive)**: valid code or accepted convention incorrectly flagged.
- **Debatable**: project-context dependent.

The sample sizes are small. Treat these as directional indicators, not statistically complete accuracy measurements.

### Rust — Raw Sample

| Rule | Sample | TP | FP | Debatable | Notes |
|------|-------:|---:|---:|----------:|-------|
| magic-number | 10 | 5 | 3 | 2 | Small ints and domain constants can be noisy |
| hungarian-notation | 10 | 2 | 6 | 2 | Domain prefixes can be mistaken for Hungarian notation |
| deep-nesting | 5 | 5 | 0 | 0 | Strong signal in sampled cases |
| god-function | 5 | 5 | 0 | 0 | Strong signal in sampled cases |
| code-duplication | 5 | 4 | 1 | 0 | Mostly useful, occasional similar-but-distinct code |
| unwrap-abuse | 3 | 3 | 0 | 0 | Useful in sampled cases |
| **Total** | **38** | **24** | **10** | **4** | **Raw TP: 63%, FP: 26%, Debatable: 11%** |

**Verdict:** Rust structural rules look useful, but the prior **~87% TP** headline is not justified by the visible sample alone.

### Go — Raw Sample

| Rule | Sample | TP | FP | Debatable | Notes |
|------|-------:|---:|---:|----------:|-------|
| magic-number | 10 | 6 | 3 | 1 | Buffer sizes, HTTP codes, and constants need context |
| println-debugging | 10 | 2 | 7 | 1 | `fmt.Println` in CLI code is often valid output |
| single-letter-variable | 10 | 3 | 5 | 2 | Tests and idiomatic short names are noisy |
| code-duplication | 5 | 4 | 1 | 0 | Generated/template output can inflate counts |
| hungarian-notation | 5 | 1 | 3 | 1 | Domain prefixes can be misclassified |
| **Total** | **40** | **16** | **19** | **5** | **Raw TP: 40%, FP: 48%, Debatable: 13%** |

**Verdict:** Go accuracy needs more tuning before it should be used as a strict quality gate.

---

## Current Code Cross-Check

### Supported by Current Implementation

- Parsed files are reused across phases, reducing repeated I/O and parsing.
- Tree-sitter queries use a thread-local cache.
- Several language adapters use cached regex via `LazyLock`.
- Test-file detection exists for common test, example, bench, fixture, and mock paths.
- Some generated/dependency files are skipped, including protobuf outputs, `node_modules`, `vendor`, and `swagger-ui` paths.
- Magic-number detection has a basic allowlist and project-config overrides.

### Still Incomplete or Noisy

- `println-debugging` still needs context for CLI output and Go `func main()`.
- `single-letter-variable` still needs language-specific exceptions for idioms such as Go `g/e`, Rust formatter `f`, and math notation.
- `hungarian-notation` still needs stronger semantic prefix filtering.
- Generated-code filtering should explicitly cover `*.gen.*`, `*.generated.*`, `*.min.js`, `*.bundle.js`, `.venv`, and `venv` in the analyzer path collection layer.
- README and analyzer defaults should be aligned for built-in exclude patterns.

---

## Performance

### Optimization Evidence in Code

| Optimization | Status | Evidence |
|--------------|--------|----------|
| Reuse parsed files | Present | Analysis phases reuse `parsed_files` |
| StyleIr reuse | Present | Direct detectors can consume precomputed IR |
| Query cache | Present | Thread-local `HashMap` for compiled tree-sitter queries |
| Regex caching | Present | `LazyLock` regexes in language adapters |

### Historical Timing Claims

| Version | Time (37K lines) | Status |
|---------|----------------:|--------|
| Before optimization | 25.4s | Historical, not reproduced here |
| Phase 1: Eliminate redundant I/O | 5.7s | Historical, not reproduced here |
| Phase 2: StyleIr pre-computation | 2.1s | Historical, not reproduced here |
| Phase 3: Query cache | 1.4s | Historical, not reproduced here |
| Phase 4: Regex caching | 1.3s | Historical, not reproduced here |

**Verdict:** The optimization mechanisms exist, but this report should not claim exact speedups without a reproducible benchmark command, fixture path, machine profile, and output artifact.

---

## Issue Distribution by Rule

These distributions are preserved as historical observations and should be re-run before release notes or marketing copy use them.

### system_alert (Rust, 112 issues)

```text
37  magic-number
26  hungarian-notation
14  deep-nesting
13  code-duplication
 8  god-function
 8  rust-must-use
 3  long-function
 1  unwrap-abuse
```

### gnark (Go, 11,646 issues)

```text
5382  magic-number
2356  code-duplication
1908  single-letter-variable
1176  cross-file-duplication
 259  terrible-naming
 245  println-debugging
 134  god-function
  76  long-function
```

### CodeTribunal (Go, 476 issues)

```text
157  println-debugging
130  single-letter-variable
103  magic-number
 51  code-duplication
 12  cross-file-duplication
  7  deep-nesting
```

---

## Recommendations

### P0 — Documentation and Measurement

1. Add a reproducible benchmark command and fixture list before publishing timing claims.
2. Report raw TP/FP rates separately from weighted estimates.
3. Include rule-level issue volumes when presenting weighted TP rates.
4. Align README exclude defaults with `CodeAnalyzer` defaults.

### P1 — Accuracy Improvements

1. Improve magic-number context awareness for test tables, crypto/math constants, HTTP codes, and power-of-2 values.
2. Exclude `fmt.Print*` in Go `func main()` and known CLI output paths from `println-debugging`.
3. Add single-letter variable exceptions for tests and language idioms.
4. Refine Hungarian notation detection to avoid domain prefixes and framework conventions.

### P2 — Generated and Dependency Filtering

1. Skip `*.gen.*`, `*.generated.*`, `*.min.js`, and `*.bundle.js` by default.
2. Skip `.venv` and `venv` in analyzer path collection, not only in metrics helpers.
3. Add regression tests for generated/template duplication false positives.

---

## Verification

Validated during this review:

```bash
cargo test -q analyzer::tests --lib
```

Result: 18 analyzer tests passed.

---

## Conclusion

The original report correctly identified several important false-positive patterns, but its headline accuracy and readiness claims were too strong for the evidence shown. The safer conclusion is:

- Rust structural rules are promising and often useful.
- Go style rules need more context before strict CI use.
- Current performance architecture is improved, but exact speedup claims need reproducible evidence.
- The next priority should be measurement reproducibility, README/default-exclude alignment, and targeted FP reduction for noisy rules.
