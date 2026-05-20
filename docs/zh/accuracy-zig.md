# Zig 准确度报告

> **状态: Beta** — Zig 语法仍在演进，尚未成为主流语言。tree-sitter grammar 覆盖可能滞后于语言变化。检测结果应视为参考，而非权威。

> 生成: 2026-05-15 | 项目: 1

## 测试项目

| 项目 | 问题数 | 规则数 | 主要问题 |
|------|:------:|:------:|----------|
| ziglings | 247 | 11 | magic-number 111, commented-code 94, cross-file-dup 12 |

## 关键发现

- `std.debug.print` 检测正常工作
- Hungarian-notation: 10 个，全部有效
- deep-nesting + long-function 对 Zig 正常生效

## 估计 TP 率: ~90%
