# Go Accuracy Report

> Generated: 2026-05-15 | Projects: 4

## Tested Projects

| Project | Issues | Rules | Top Issues |
|---------|:------:|:-----:|------------|
| interchange | 129 | 10 | dead-code 42, cross-file-dup 42, panic 14 |
| gaia | 182 | 12 | dead-code 46, magic-number 31, god-function 26 |
| loan | 127 | 11 | dead-code 51, cross-file-dup 28, panic 18 |
| gosec | 1229 | 15 | code-duplication 354, dead-code 317, deep-nesting 178 |

## Key Observations

- `panic-abuse`: Works correctly. All panics verified as genuine.
- `dead-code`: ~87% TP. Text-based, some FP from closing braces in closures.
- `magic-number`: ~95% TP. Switch case labels occasionally slip through.
- `single-letter`: ~73% TP. Loop/math variables main FP source.

## Estimated TP Rate: ~85%
