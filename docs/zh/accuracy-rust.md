# Rust 准确度报告

> 生成: 2026-05-15 | 项目: 2

## 测试项目

| 项目 | 问题数 | 规则数 | 主要问题 |
|------|:------:|:------:|----------|
| Finance | 184 | 15 | code-duplication 43, magic-number 32, god-function 22 |
| ReChat-server | 34 | 10 | long-function 7, magic-number 6, terrible-naming 6 |

## 关键发现

- 所有 Rust 特有规则正常触发: macro-abuse 20, box-abuse 4, unwrap-abuse 3
- 未发现路径解析 bug
- 单字母变量全部为真实坏命名

## 估计 TP 率: ~90%
