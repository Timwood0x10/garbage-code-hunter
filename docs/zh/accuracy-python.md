# Python 准确度报告

> 生成: 2026-05-15 | 项目: 1

## 测试项目

| 项目 | 问题数 | 规则数 | 主要问题 |
|------|:------:|:------:|----------|
| ZK-bulletproofs | 111 | 3 | magic-number 97, cross-file-dup 11, commented-code 3 |

## 关键发现

- `wildcard-import`: 白名单正常工作（manim, numpy 等已豁免）
- `bare-except`: 100% TP
- 数学代码中单字母变量 FP 率较高（n, x, y, a, b 是标准符号）

## 估计 TP 率: ~80%
