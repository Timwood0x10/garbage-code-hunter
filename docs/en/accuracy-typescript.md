# TypeScript Accuracy Report

> Generated: 2026-05-15 | Projects tested: 3 | Analyzer: garbage-code-hunter

---

## Tested Projects

| Project | Files | Lines | Issues | Score | Density |
|---------|:-----:|:-----:|:------:|:-----:|:-------:|
| zod (schema validation) | 409 | 74,562 | 3,633 | 54.2 | 48/k |
| hono (web framework) | 393 | 77,864 | 7,785 | 58.0 | 99/k |
| trpc (RPC framework) | 987 | 118,699 | 5,610 | 36.6 | 47/k |

---

## Per-Rule Accuracy

### any-type (TypeScript-specific)

| Metric | Value |
|--------|-------|
| Total detections | ~700+ |
| Verified TP rate | **~10-20%** |
| Major FP source | Type libraries, framework generics |

**Source-code verification:**

| File | Reported | Actual | Verdict |
|------|:--------:|:------:|:-------:|
| zod schemas.ts | 297 | 283 | FP — type library internals |
| zod util.ts | 78 | 63 | Mixed — utility code |
| hono types.ts | 141 | 74 | FP — framework generic defaults |
| hono index.ts | **144** | **0** | **Bug — counting error** |
| trpc procedureBuilder.ts | 36 | 21 | FP — library internals |

**Key patterns flagged as FP:**
- Generic type parameter defaults: `Handler<E extends Env = any, P extends string = any>`
- Type-level programming: `T extends any ? core.output<T> : never`
- Internal type assertions: `payload as any`, `null as any`
- Utility type definitions: `AnyFunc = (...args: any[]) => any`

**Verdict: The most problematic rule for TypeScript.** The `any-type` rule does not distinguish between:
1. **Type library internals** (zod, io-ts) — `any` is unavoidable for type-level programming
2. **Framework generic defaults** (hono, express) — `any` as default type parameter is idiomatic
3. **Genuine laziness** — `let x: any` in application code (the actual target)

**The hono index.ts bug (144 reported, 0 actual) suggests a counting/aggregation error in the analyzer.**

### println-debugging

| Metric | Value |
|--------|-------|
| Total detections | ~100+ |
| FP rate | ~30% |

**Verdict: Mostly reliable.** Some FP from benchmark scripts and CLI tools. Test files should be excluded (they already are for some rules, but not consistently).

### hungarian-notation

| Metric | Value |
|--------|-------|
| Total detections | ~80+ |
| FP rate | ~60% |

**Source-code verification:**
- Hono's `c` (context) parameter is flagged as hungarian notation. **FP.** `c` is the standard Hono context variable, similar to `req`/`res` in Express.
- tRPC's `t` (router builder) is flagged. **FP.** `t` is the standard tRPC initialization variable.

**Verdict: High FP rate for framework conventions.** Common framework abbreviations (`c` for context, `t` for tRPC builder, `ctx` for context) should be exempted.

### code-duplication

| Metric | Value |
|--------|-------|
| Total detections | ~3000+ |
| FP rate | **~70-80%** |

**Verdict: Dominant noise source.** Test files produce massive duplication counts (hono: 1,070 duplication issues in `index.test.ts` alone). The Jaccard similarity on token types is too coarse — any two test functions that use identifier + type + string will be flagged as similar.

### magic-number

| Metric | Value |
|--------|-------|
| Total detections | ~500+ |
| FP rate | ~25% |

**Verdict: Acceptable.** HTTP status codes, common constants (0, 1, 100) produce some FP.

### commented-code

| Metric | Value |
|--------|-------|
| Total detections | ~30+ |
| FP rate | ~40% |

**Source-code verification:**
- `// eslint-disable` lines flagged as commented code. **FP.**
- `// @ts-expect-error` lines flagged. **FP.**

**Verdict: Needs to exempt tool directives.** Lines starting with `// eslint`, `// @ts-`, `// prettier`, `// noinspection` should be excluded.

---

## Accuracy Summary

| Rule | TP Rate | FP Source | Severity |
|------|:-------:|-----------|:--------:|
| any-type | **~10-20%** | Type libraries, framework generics | **Critical FP** |
| println-debugging | **~70%** | Benchmarks, CLI tools | Acceptable |
| hungarian-notation | **~40%** | Framework conventions (c, t, ctx) | High FP |
| code-duplication | **~20-30%** | Test files, structural similarity | **Critical FP** |
| magic-number | **~75%** | HTTP codes, common values | Acceptable |
| commented-code | **~60%** | eslint/ts-expect directives | Needs filter |
| long-function | **~80%** | Doc comments counted | Acceptable |
| god-function | **~70%** | Facade classes with overloads | Acceptable |
| single-letter-var | **~60%** | Loop vars (i, j, k) not fully exempt | Acceptable |
| **Overall** | **~40-50%** | | |

---

## Critical Issues

1. **`any-type` counting bug** — hono index.ts reported 144 `any` types but the file has 0. The analyzer likely aggregates counts from other files or has a query matching bug.
2. **`any-type` does not distinguish context** — A type-validation library (zod) and a web framework (hono) use `any` for fundamentally different reasons than application code. The rule needs per-project or per-file-type calibration.
3. **Duplication explosion in test files** — Test files with many similar test cases produce thousands of false duplication issues. This inflates issue counts by 3-5x.

---

## Concrete Improvement Suggestions

1. **`any-type` context filter** — Skip `any` in files that are primarily type definitions (files where >50% of lines are `type`/`interface` declarations). Or add a per-file threshold (e.g., only flag if <10 `any` types in a file — a file with 100+ is clearly a type library).
2. **Framework convention exemption** — Add `c`, `t`, `ctx`, `req`, `res`, `err` to the hungarian-notation allowlist.
3. **Directive exemption for commented-code** — Skip lines matching `// eslint`, `// @ts-`, `// prettier`, `// noinspection`, `// istanbul`.
4. **Test file duplication cap** — Limit duplication issues per test file to 10-20, since test repetition is inherent to the testing pattern.
5. **Fix the counting bug** — Investigate why hono index.ts shows 144 `any` types when the file has 0. Likely a tree-sitter query matching issue or file path resolution bug.
