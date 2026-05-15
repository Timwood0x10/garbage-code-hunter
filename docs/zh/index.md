# Garbage Code Hunter 文档

一个幽默的代码质量检测工具集，用最毒舌的方式吐槽你的垃圾代码。

> **灵感来源**: [fuck-u-code](https://github.com/Done-0/fuck-u-code.git)

## 这是什么？

Garbage Code Hunter 是一个 CLI 工具集，用于代码品味分析。不同于传统 linter 给你干巴巴的警告，我们用**毒舌、机智、毫不留情**的方式告诉你代码有多烂。

这不是静态 bug 检测器。它找的是：
- 烂命名（`data`、`info`、`tmp`、`foo`、`bar`）
- 到处散落的魔法数字
- 深层嵌套的代码迷宫
- 什么都干的上帝函数
- 遗留的 `println`/`fmt.Println` 调试
- 当宝贝一样留着的注释代码
- 永远不会完成的 TODO 注释
- 跨文件复制粘贴的函数

## 支持语言

Rust、Go、Python、JavaScript、TypeScript、Java、C、C++、Ruby、Swift、Zig（共 11 种）

## 快速开始

```bash
# 安装
cargo install garbage-code-hunter

# 分析当前目录
garbage-code-hunter analyze

# 分析特定语言项目
garbage-code-hunter analyze ./my-go-project --lang go

# 全量扫描
garbage-code-hunter scan ./my-project
```

## 工具全家桶

| 工具 | 命令 | 功能 |
|------|------|------|
| **Code Hunter** | `analyze` | 静态分析：命名、嵌套、重复代码 |
| **Commit Roaster** | `commit-roaster` | 吐槽烂 commit 消息 |
| **Deps Shamer** | `deps-shamer` | 羞耻依赖管理 |
| **PR Title Hunter** | `pr-title-hunter` | 吐槽低质量 PR 标题 |
| **Full Scan** | `scan` | 跑所有工具，输出综合评分 |
| **Last Words** | `last-words` | 发现 TODO/FIXME/HACK 注释 |
| **Debt Invoice** | `debt-invoice` | 生成技术债账单 |
| **Personality** | `personality` | 分析开发者人格画像 |
| **Danger Zone** | `danger-zone` | 找出最危险的文件 |
| **Team Roast** | `team-roast` | 按开发者分析，团队吐槽 |
| **Radar** | `radar` | 代码气味雷达图（SVG） |
| **Autopsy** | `autopsy` | 代码尸检报告 |
| **Decay** | `decay` | 项目质量衰减曲线 |
| **CI Bot** | `ci-bot` | CI 风格 PR 审查评论 |
| **Persona** | `persona` | 用特定人格模式吐槽 |

## 文档目录

- [规则参考手册](rules.md) — 所有检测规则及语言覆盖
- [工具指南](tools.md) — 各工具详细说明
- [配置说明](configuration.md) — 配置文件选项

## 真实项目测试结果

### Go 项目（interchange，约 47K 行）

```
Issue 统计:
  46 Nuclear | 396 Spicy | 12,332 Mild | 12,774 Total

主要问题: magic-number、single-letter、code-duplication
问题最多的文件: tx.pulsar.go (1250)、tx.pb.go (1129) ← 生成文件
```

### Rust 项目（ReChat-server）

```
Issue 统计:
  0 Nuclear | 34 Spicy | 2,103 Mild | 2,137 Total

评分: 1.1/100 — Excellent
主要问题: cross-file-near-duplicate、println-debugging
```

### Zig 项目（ziglings）

```
Issue 统计:
  0 Nuclear | 18 Spicy | 6,101 Mild | 6,119 Total

主要问题: magic-number (358)、commented-code (102)、single-letter (80)
```
