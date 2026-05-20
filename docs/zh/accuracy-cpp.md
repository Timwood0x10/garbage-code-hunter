# C++ 准确度报告

> 生成: 2026-05-15 | 项目: 1

## 测试项目

| 项目 | 问题数 | 规则数 | 主要问题 |
|------|:------:|:------:|----------|
| stone-prover | 878 | 16 | commented-code 250, magic-number 237, long-function 99 |

## 关键发现

- 所有通用规则正常工作: deep-nesting, long-function, god-function 等
- C/C++ 特有规则触发: abbreviation-abuse 63, file-too-long 8

## 估计 TP 率: ~85%
