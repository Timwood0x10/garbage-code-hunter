# Garbage Code Hunter — Multi-Language Accuracy Report

> Generated: 2026-05-15
> Projects tested: **35+** across **11 languages**
> Detailed per-language reports: `docs/en/accuracy-*.md`

---

## 1. Tested Projects

### Go (8 projects)

| Project | Total | Prod | Test | Score | Key Issues |
|---------|:-----:|:----:|:----:|:-----:|------------|
| interchange | 602 | 203 | 399 | 20.1 | Cosmos SDK app chain |
| gaia | 1039 | 429 | 610 | — | Cosmos SDK hub chain |
| loan | 366 | 208 | 158 | — | Cosmos SDK app chain |
| gnark | 15592 | 10058 | 5534 | — | zk-SNARK library |
| goagent | 35604 | 28618 | 6986 | — | AI agent framework |
| gosec | 4602 | 2821 | 1781 | 56.7 | Go security scanner |
| go-stdlib-http | 7379 | 1750 | 5629 | 58.2 | Go stdlib HTTP pkg |
| train | 34 | 34 | 0 | — | ML training |

### Rust (11 projects)

| Project | Total | .rs files | Key Rules Firing |
|---------|:-----:|:---------:|------------------|
| coq-of-rust | 469 | 282 | unwrap 13, box 11, deep-nesting 15 |
| Finance | 1260 | 66 | println 388, box 30, macro 20 |
| memscope-rs | 3820 | 470 | deep-nesting 90, long-function 79 |
| memscope-stress-test | 233 | 8 | println 154, magic 58, box 4 |
| ReChat-server | 172 | 48 | box 8, long-function 7 |
| system_alert | 133 | 22 | magic 64, deep-nesting 14, box 6 |
| AlgoGpuRust | 70 | 44 | unwrap 1, box 1, macro 1 |
| gpu-code | 69 | 6 | magic 39, unwrap 3 |
| garbage-code-hunter | 1083 | 80 | all 26 Rust rules firing |
| lifeRestart | 41609 | — | (large web project) |
| algo | 0 | 1 | clean code |

### Python (10 projects)

| Project | Total | Prod | Test | Key Issues |
|---------|:-----:|:----:|:----:|------------|
| predict | 145647 | — | — | massive ML prediction framework |
| ecc | 114159 | — | — | elliptic curve crypto lib |
| learn-claude-code | 24610 | — | — | AI training code |
| ZK-bulletproofs | 20012 | 20012 | 0 | zero-knowledge proofs lib |
| audiolm | timeout | — | — | large audio ML project (timeout) |
| ds | 393 | 393 | 0 | deepseek scripts |
| multi-agent | 758 | 702 | 56 | multi-agent system |
| vision | 4915 | 4915 | 0 | manim math visualization |
| basis_math | 16 | — | — | basic math |
| dataProcess | 1 | — | — | data processing |

### TypeScript (3 projects)

| Project | Files | Lines | Issues | Score | Key Issues |
|---------|:-----:|:-----:|:------:|:-----:|------------|
| zod | 409 | 74K | 3,633 | 54.2 | any-type 410, duplication 190 |
| hono | 393 | 78K | 7,785 | 58.0 | any-type 310, duplication 3300 |
| trpc | 987 | 119K | 5,610 | 36.6 | any-type 250, duplication 2800 |

### Java (3 projects)

| Project | Files | Lines | Issues | Score | Key Issues |
|---------|:-----:|:-----:|:------:|:-----:|------------|
| okhttp | 71 | 4.4K | 158 | 37.3 | println 50, empty-catch 1 |
| junit5 | 1,724 | 222K | 6,723 | 14.3 | duplication 5000+, empty-catch 2 |
| jdk benchmarks | — | — | 12,562 | — | 100% test code |

### Swift (3 projects)

| Project | Files | Lines | Issues | Score | Key Issues |
|---------|:-----:|:-----:|:------:|:-----:|------------|
| Alamofire | 108 | 47K | 5,843 | 54.2 | **JS files in docs/ inflated by 2300+** |
| SnapKit | 44 | 4.5K | 380 | 42.8 | magic-number 260 (bitmask) |
| vapor | 298 | 38K | 2,880 | 39.7 | duplication 1500+, commented-code FP |

### Ruby (1 project)

| Project | Files | Lines | Issues | Score | Key Issues |
|---------|:-----:|:-----:|:------:|:-----:|------------|
| jekyll | 166 | 24K | 776 | 27.7 | duplication 500+, global-var FP |

### Zig (4 projects)

| Project | Total | .zig files | Key Issues |
|---------|:-----:|:----------:|------------|
| OmniScope | 11714 | 173 | large Zig project |
| PolyScope | 2312 | 75 | Zig tools |
| ziglings | 792 | 100 | learning exercises |
| zig/std | 394 | — | stdlib |

### C++ (3 projects)

| Project | Total | Score | Key Issues |
|---------|:-----:|:-----:|------------|
| stone-prover | 3936 | 41.2 | Starkware prover |
| wabt | 34 | — | WebAssembly toolkit |
| unixos_api | 5 | — | OS API |

---

## 2. Source-Code-Verified Accuracy (NEW)

> Based on sampling 5+ issues per language against actual source code.

| Language | Overall TP Rate | #1 FP Source | Critical Bug |
|----------|:--------------:|--------------|:------------:|
| **Go** | ~75% | code-duplication noise | — |
| **Rust** | ~85% | test code println | — |
| **Python** | ~55% | wildcard-import (manim), println in CLI | — |
| **TypeScript** | ~40-50% | `any-type` in type libs | **any-type counting bug** |
| **Java** | ~60% | code-duplication in tests | — |
| **Swift** | ~30-40% | commented-code matches `///` docs | **JS files scanned** |
| **Ruby** | ~35-45% | code-duplication in tests | — |
| **Zig** | ~70% | — | — |
| **C++** | ~70% | — | — |

### Per-Rule TP Rate (verified against source code)

| Rule | TP Rate | Main FP Source | Languages Verified |
|------|:-------:|----------------|:------------------:|
| bare-except | **~100%** | — | Python |
| empty-catch | **~100%** | — | Java |
| bare-rescue | N/A | — | Ruby (no instances found) |
| global-variable | **~0%** | `$LOAD_PATH` gem pattern | Ruby |
| any-type | **~10-20%** | Type library internals, framework generics | TypeScript |
| wildcard-import | **~20%** | Library idioms (manim) | Python |
| println-debugging | **~30-70%** | CLI scripts, example files | All |
| commented-code | **~0-10%** | `///` doc comments (Swift), directives | Swift, TypeScript |
| code-duplication | **~20-30%** | Test file structural similarity | All |
| magic-number | **~60-80%** | Bitmasks, HTTP codes, common values | All |
| single-letter-var | **~50-60%** | Math notation, loop counters | All |
| hungarian-notation | **~40-70%** | Framework conventions (c, t, ctx) | TS, Java |
| long-function | **~50-85%** | Doc comments inflate count | Swift |
| god-function | **~50-80%** | Facade classes, switch tables | All |
| deep-nesting | **~90%** | — | All |
| terrible-naming | **~95%** | — | All |
| **Overall (verified)** | **~55-60%** | | |

---

## 3. Cross-Language Rule Coverage

| Rule | Go | Rust | Zig | C++ | Python | Java | JS/TS | Ruby | Swift |
|------|:--:|:----:|:---:|:---:|:------:|:----:|:-----:|:----:|:-----:|
| magic-number | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| single-letter-var | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| terrible-naming | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| deep-nesting | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| long-function | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| god-function | ✅ | ✅ | ✅ | ✅ | — | — | — | — | — |
| println-debugging | ✅ | ✅ | ✅ | — | ✅ | ✅ | ✅ | ✅ | ✅ |
| commented-code | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| dead-code | ✅ | ✅ | — | — | — | — | — | — | — |
| file-too-long | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| panic-abuse | ✅ | — | — | — | — | — | — | — | — |
| defer-in-loop | ✅ | — | — | — | — | — | — | — | — |
| unwrap/box/vec/macro | — | ✅ | — | — | — | — | — | — | — |
| bare-except | — | — | — | — | ✅ | — | — | — | — |
| wildcard-import | — | — | — | — | ✅ | — | — | — | — |
| empty-catch | — | — | — | — | — | ✅ | — | — | — |
| any-type | — | — | — | — | — | — | ✅ | — | — |
| global-variable | — | — | — | — | — | — | — | ✅ | — |
| bare-rescue | — | — | — | — | — | — | — | ✅ | — |

---

## 4. Critical Bugs Found

### 4.1 Non-target-language files scanned (Swift)

The analyzer scanned `.js` files inside `docs/` when analyzing Swift projects, producing 2,300+ false issues from `jquery.min.js`, `lunr.min.js`, etc.

**Fix:** Filter files by target language extension. Skip `docs/`, `vendor/`, `.build/`, `node_modules/` directories.

### 4.2 `any-type` counting bug (TypeScript)

`hono/src/index.ts` reported 144 `any` types but the file has 0. The analyzer likely aggregates counts from other files or has a tree-sitter query matching issue.

**Fix:** Investigate the `any-type` query — `(predefined_type) @t` may match nodes outside the target file, or the file path resolution is wrong.

### 4.3 `any-type` does not distinguish context (TypeScript)

Type-validation libraries (zod) and web frameworks (hono) use `any` for fundamentally different reasons than application code. The rule flags ~700+ issues across these 3 projects, with ~80-90% being legitimate library code.

**Fix:** Add per-file threshold — skip files with >20 `any` types (likely type library internals). Or detect files where >50% of lines are type/interface declarations.

### 4.4 `commented-code` matches `///` doc comments (Swift)

Swift's `///` doc-comment format triggers the commented-code rule with near-100% FP rate. The rule matches doc comments containing keywords like `return`, `true`, `credential`.

**Fix:** Skip `///` lines entirely. For `//` comments, only flag lines containing actual code patterns (assignment, function calls, control flow keywords in code-like syntax).

---

## 5. Key Observations

1. **Rust remains the gold standard** — ~85% TP rate. All 26 rules are well-calibrated.
2. **Go is solid** — ~75% TP rate. Main noise is code-duplication in test files.
3. **TypeScript `any-type` is broken** — ~10-20% TP rate. This is the single biggest accuracy problem across all languages.
4. **Swift `commented-code` is broken** — ~0-10% TP rate. `///` doc comments are not commented-out code.
5. **Code-duplication is universally noisy** — ~20-30% TP rate across all languages. Test files with similar patterns produce thousands of false positives.
6. **Language-specific rules are highly accurate** — `bare-except` (Python) and `empty-catch` (Java) both have ~100% TP rate.
7. **println-debugging needs context** — CLI scripts, example files, and benchmark code legitimately use print/println for user output.
8. **Math/scientific code exemptions needed** — Single-letter variables in numerical computing (a, b, x, y, z, h, n) follow mathematical conventions.

---

## 6. Improvement Roadmap (Priority Order)

| # | Fix | Impact | Effort | Languages |
|---|-----|--------|--------|-----------|
| 1 | **Filter by target file extension** | Eliminates 2300+ FP in Swift alone | Small | All |
| 2 | **Fix `any-type` counting bug** | Eliminates fabricated counts | Small | TS |
| 3 | **`any-type` context filter** (skip files with >20 `any`) | Eliminates ~80% of TS FP | Small | TS |
| 4 | **Skip `///` in commented-code** | Eliminates ~90% of Swift FP | Small | Swift |
| 5 | **Exclude `docs/`, `vendor/`, `node_modules/`** | Eliminates non-project files | Small | All |
| 6 | **Test file duplication cap** (max 20 per file) | Reduces duplication noise by 60-70% | Medium | All |
| 7 | **CLI script exemption for println** | Reduces println FP by 30-40% | Medium | All |
| 8 | **Wildcard import allowlist** (manim, numpy, etc.) | Reduces Python FP | Small | Python |
| 9 | **Framework convention exemption** (c, t, ctx, req, res) | Reduces hungarian FP | Small | TS, Java |
| 10 | **Math variable exemption** | Reduces single-letter FP in scientific code | Medium | All |
| 11 | **Exclude doc comments from function length** | Fixes long-function FP | Medium | Swift, TS |
| 12 | **OptionSet bitmask exemption** | Fixes magic-number FP in Swift | Small | Swift |

---

## 7. Remaining Gaps

| Gap | Impact | Effort |
|-----|--------|--------|
| Go `for range` loop vars exempted | ~30% single-letter FP | Small |
| god-function threshold per-language | ~15% god-function FN | Medium |
| `spew.Dump`, `trace.Print` not detected | ~5% println FN | Small |
| `dead-code` only for Rust + Go | FN in other languages | Medium |
| No Swift force-unwrap detection | Major Swift code smell FN | Medium |
| No Ruby monkey-patching detection | Major Ruby code smell FN | Medium |
| No Java resource-leak detection | Common Java code smell FN | Medium |
