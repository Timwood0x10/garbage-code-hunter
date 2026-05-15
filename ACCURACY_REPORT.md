# Garbage Code Hunter — Multi-Language Accuracy Report

> Generated: 2026-05-15
> Projects tested: **22** across **6 languages**

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

### Rust (5 projects)

| Project | Total | Prod | Test | Score | Key Issues |
|---------|:-----:|:----:|:----:|:-----:|------------|
| garbage-code-hunter | 1083 | 982 | 101 | 1.6 | self-analysis |
| Finance | 1260 | — | — | — | financial logic |
| ReChat-server | 172 | — | — | — | chat server |
| system_alert | 133 | — | — | — | system monitor |
| memscope-stress-test | 233 | — | — | — | mem stress test |

### Python (3 projects)

| Project | Total | Notes |
|---------|:-----:|-------|
| demo | 0 | small script |
| basis_math | 16 | math library |
| dataProcess | 1 | data processing |

### Zig (2 projects)

| Project | Total | Score | Key Issues |
|---------|:-----:|:-----:|------------|
| ziglings | 792 | 53.8 | learning exercises |
| zig/std (array_list.zig) | 394 | — | stdlib array list impl |

### C++ (3 projects)

| Project | Total | Score | Key Issues |
|---------|:-----:|:-----:|------------|
| stone-prover | 3936 | 41.2 | Starkware prover |
| wabt (binary-reader.cc) | 34 | — | WebAssembly toolkit |
| unixos_api | 5 | — | OS API |

### Java (1 project)

| Project | Total | Notes |
|---------|:-----:|-------|
| jdk micro benchmarks | 12562 | 100% test code |

---

## 2. Cross-Language Rule Coverage

| Rule | Go | Rust | Zig | C++ | Python | Java | JS/TS |
|------|:--:|:----:|:---:|:---:|:------:|:----:|:-----:|
| magic-number | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| single-letter-var | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| terrible-naming | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| deep-nesting | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| long-function | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| god-function | ✅ | ✅ | ✅ | ✅ | — | — | — |
| println-debugging | ✅ | ✅ | ✅ | — | ✅ | ✅ | ✅ |
| commented-code | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| dead-code | ✅ | ✅ | — | — | — | — | — |
| file-too-long | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| panic-abuse | ✅ | — | — | — | — | — | — |
| defer-in-loop | ✅ | — | — | — | — | — | — |
| unwrap/box/vec/macro | — | ✅ | — | — | — | — | — |

---

## 3. Production Code TP Rate (estimated)

| Rule | Est. TP | Main FP Source |
|------|:-------:|----------------|
| magic-number | 80-85% | common values (0,1,10,60,100) |
| single-letter-variable | 60-65% | loop counters not fully exempted |
| terrible-naming | 95% | — |
| println-debugging | 90% | — |
| deep-nesting | 95% | — |
| long-function | 85-90% | interface method signatures |
| god-function | 75-80% | large switch tables |
| dead-code (Go) | 75-80% | early return patterns |
| panic-abuse (Go) | 85% | main() panics acceptable |
| defer-in-loop (Go) | 100% | — |
| unwrap-abuse (Rust) | 90% | test unwraps |
| **Overall** | **~80%** | |

---

## 4. Key Observations

1. **Go dominates** — 8 projects, largest dataset. Production code 24-94% of total issues.
2. **Test code is noisy** — Go projects have 39-76% of issues in test files (duplication, println, single-letter).
3. **goagent** is the most "garbage" project — 35604 issues, mostly code-duplication + magic-number.
4. **gnark** (zk-SNARK) — 15592 issues, heavy magic-number usage (cryptographic constants).
5. **Python detection is weak** — small projects found few issues. Need larger Python codebase.
6. **Zig detection works** — ziglings and stdlib both produce meaningful results.
7. **Java works** — jdk benchmarks correctly identified as 100% test code.

---

## 5. Remaining Gaps

| Gap | Impact | Effort |
|-----|--------|--------|
| Go `for range` loop vars exempted | ~30% single-letter FP | Small |
| god-function threshold per-language | ~15% god-function FN | Medium |
| Python needs more rules (bare-except only now) | Python accuracy low | Medium |
| `spew.Dump`, `trace.Print` not detected | ~5% println FN | Small |
| No Ruby rules beyond global-variable/bare-rescue | Ruby accuracy low | Medium |
| `dead-code` only for Rust + Go | FN in other languages | Medium |
