# Garbage Code Hunter — Accuracy Report

> Generated: 2026-05-15 from this machine
> All data fresh, not from external reports

---

## Go (5 projects)

| Project | Total | Rule Types | Top Issues |
|---------|:-----:|:----------:|------------|
| gnark | 13633 | 20 | magic-number 5622, code-duplication 2356, single-letter 2170 |
| gosec | 3433 | 16 | code-duplication 1764, magic-number 359, dead-code 335 |
| interchange | 462 | 12 | magic-number 221, code-duplication 64, dead-code 56 |
| gaia | 460 | 14 | code-duplication 143, magic-number 116, dead-code 48 |
| loan | 261 | 12 | code-duplication 67, dead-code 53, cross-file-dup 41 |

### Key observations
- **panic-abuse**: gnark 169, loan 18, interchange 14 — rule working correctly
- **println-debugging**: gnark 0 (after Sprintf fix), gosec 60, loan 10 — no comment-matching bug
- **Sprintf fix**: removed from print detection (was FP — formats string, doesn't print)

---

## Rust (2 projects)

| Project | Total | Rule Types | Top Issues |
|---------|:-----:|:----------:|------------|
| garbage-code-hunter | 983 | 25 | code-duplication 419, println 164, magic-number 90 |
| Finance | 1225 | 17 | code-duplication 462, println 388, magic-number 199 |

### Key observations
- All 25 Rust-specific rules firing (unwrap, box, macro, vec, lifetime, etc.)
- No mod.rs path resolution bug found on this machine

---

## Zig (1 project)

| Project | Total | Rule Types | Top Issues |
|---------|:-----:|:----------:|------------|
| ziglings | 769 | 14 | magic-number 358, println 161, commented-code 102 |

### Key observations
- `std.debug.print` detection: 161 hits ✅
- Single-letter detection: 80 hits ✅
- Hungarian-notation: 10 — all valid (Zig Hungarian)
- No broken rules found

---

## Python (2 projects)

| Project | Total | Rule Types | Top Issues |
|---------|:-----:|:----------:|------------|
| ZK-bulletproofs | 18458 | 21 | magic-number 7568, code-duplication 3871, single-letter 1697 |
| ds | 395 | 10 | println 226, magic-number 74, cross-file-dup 35 |

### Key observations
- wildcard-import: allowlist working (manim, numpy, etc.)
- 21 different rule types firing — comprehensive coverage

---

## Active Issues Log

| Issue | Status | Fix |
|-------|--------|-----|
| Sprintf in print detection | ✅ FIXED | Removed Sprint/Sprintln/Sprintf from Go pattern |
| Go println in comments | ❌ NOT REPRODUCIBLE | Old report bug, not on this machine |
| Go panic undercounting | ❌ NOT REPRODUCIBLE | This machine detects 49 vs report's 1 |
| Rust mod.rs path bug | ❌ NOT REPRODUCIBLE | 9-line mod.rs has 0 issues here |
| Zig hungarian broken | ❌ NOT REPRODUCIBLE | 10 detections, all valid |
| Python wildcard allowlist | ✅ FIXED | manim, numpy, etc. added to allowlist |
| Python/TS comment directive skip | ✅ FIXED | /// and /** skipped in commented-code |
| Ruby global-variable allowlist | ✅ FIXED | $0, $*, $_ etc. added |
| Hungarian framework exemptions | ✅ FIXED | c, t, ctx, req, res, err exempted |
| Test file duplication skip | ✅ FIXED | IntraFileDupDetector skips _test.go |
| Cross-language scoring | ✅ FIXED | 5 universal categories, no Rust-specific |
| calculate_metrics multi-lang | ✅ FIXED | Counts all SUPPORTED_EXTENSIONS, not just .rs |
| God-function threshold | ✅ FIXED | 15→10, catches more complex functions |
| Production/test split in report | ✅ FIXED | Shows 📦 prod and 🧪 test separately |
