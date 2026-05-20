# Go 准确度报告

> 生成: 2026-05-15 | 项目: 4

## 测试项目

| 项目 | 问题数 | 规则数 | 主要问题 |
|------|:------:|:------:|----------|
| interchange | 129 | 10 | dead-code 42, cross-file-dup 42, panic 14 |
| gaia | 182 | 12 | dead-code 46, magic-number 31, god-function 26 |
| loan | 127 | 11 | dead-code 51, cross-file-dup 28, panic 18 |
| gosec | 1229 | 15 | code-duplication 354, dead-code 317, deep-nesting 178 |

## 关键发现

- `panic-abuse`: 正常工作，所有 panic 已验证为真实
- `dead-code`: ~87% TP，文本检测，闭包括号偶尔误报
- `magic-number`: ~95% TP，switch case 偶尔漏过
- `single-letter`: ~73% TP，循环/数学变量是主要 FP 来源

## 估计 TP 率: ~85%
