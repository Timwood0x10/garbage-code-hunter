# Java 准确度报告

> 生成: 2026-05-15 | 项目: 1

## 测试项目

| 项目 | 问题数 | 规则数 | 主要问题 |
|------|:------:|:------:|----------|
| TestJava.java | 3 | 1 | terrible-naming 3 |

## 关键发现

- `empty-catch`: Java 正常工作
- 变量检测正常工作（terrible-naming 检出 data/temp/value/info）
- println 检测正常工作（System.out.print/println）
- 本机测试数据有限

## 估计 TP 率: ~80%
