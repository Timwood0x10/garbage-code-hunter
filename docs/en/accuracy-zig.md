# Zig Accuracy Report

> **Status: Beta** — Zig syntax is evolving and not yet mainstream. Tree-sitter grammar coverage may lag behind language changes. Detection results should be treated as indicative, not authoritative.

> Generated: 2026-05-15 | Projects: 1

## Tested Projects

| Project | Issues | Rules | Top Issues |
|---------|:------:|:-----:|------------|
| ziglings | 247 | 11 | magic-number 111, commented-code 94, cross-file-dup 12 |

## Key Observations

- `std.debug.print` detection working (was added in Round 3)
- Hungarian-notation: 10 issues, all valid
- Deep-nesting + long-function working correctly for Zig

## Estimated TP Rate: ~90%
