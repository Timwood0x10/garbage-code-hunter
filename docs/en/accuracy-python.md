# Python Accuracy Report

> Generated: 2026-05-15 | Projects: 1

## Tested Projects

| Project | Issues | Rules | Top Issues |
|---------|:------:|:-----:|------------|
| ZK-bulletproofs | 111 | 3 | magic-number 97, cross-file-dup 11, commented-code 3 |

## Key Observations

- `wildcard-import`: allowlist working (manim, numpy, etc. exempted)
- `bare-except`: 100% TP
- Single-letter in math code: high FP rate (n, x, y, a, b are standard notation)

## Estimated TP Rate: ~80%
