# Garbage Code Hunter

[English](./README.md) | [中文](README_zh.md)

一个幽默的代码质量检测工具集，用最毒舌的方式吐槽你的垃圾代码！

> **灵感来源**: https://github.com/Done-0/fuck-u-code.git

## 这是什么？

Garbage Code Hunter 是一个 CLI 工具集，用于代码质量分析。不同于传统 linter 给你干巴巴的警告，我们用**毒舌、机智、毫不留情**的方式告诉你代码有多烂。

## 工具全家桶

| 工具 | 命令 | 别名 | 功能 |
|---|---|---|---|
| **Code Hunter** | `analyze`（默认） | - | 静态分析：命名、嵌套、unwrap 滥用、重复代码 |
| **Commit Roaster** | `commit-roaster` | `cr` | 吐槽 git 历史中的烂 commit 消息 |
| **Deps Shamer** | `deps-shamer` | `ds` | 羞耻依赖管理：烂版本号、过时包、git 依赖 |
| **PR Title Hunter** | `pr-title-hunter` | `pr` | 吐槽低质量 PR 标题（本地 + GitHub） |
| **Full Scan** | `scan` | - | 跑所有工具，输出综合评分 |
| **Badge** | `badge` | - | 生成 SVG 评分徽章 |
| **Trend** | `trend` | - | 查看质量评分历史趋势 |

## 架构图

```mermaid
graph TB
    CLI["garbage-code-hunter<br/>CLI 入口 (clap)"]

    subgraph Tools["分析工具层"]
        CH["Code Hunter<br/>Rust 静态分析"]
        CR["Commit Roaster<br/>Commit 消息审查"]
        DS["Deps Shamer<br/>依赖分析"]
        PR["PR Title Hunter<br/>PR 标题审查"]
    end

    subgraph Extensions["扩展功能"]
        SCAN["Scan<br/>综合扫描"]
        BADGE["Badge<br/>SVG 徽章"]
        TREND["Trend<br/>历史趋势"]
    end

    subgraph Shared["共享模块 (common)"]
        SEV["Severity<br/>严重级别"]
        OF["OutputFormat<br/>输出格式"]
        SCORE["score_to_grade<br/>评分等级"]
    end

    subgraph Output["输出层"]
        TERM["Terminal<br/>彩色终端"]
        JSON["JSON<br/>机器可读"]
        SVG["SVG<br/>徽章图片"]
    end

    CLI --> CH
    CLI --> CR
    CLI --> DS
    CLI --> PR
    CLI --> SCAN
    CLI --> BADGE
    CLI --> TREND

    SCAN --> CH
    SCAN --> CR
    SCAN --> DS
    SCAN --> PR

    CH --> SEV
    CR --> SEV
    DS --> SEV
    PR --> SEV

    CH --> OF
    CR --> OF
    DS --> OF
    PR --> OF

    CH --> TERM
    CH --> JSON
    CR --> TERM
    CR --> JSON
    DS --> TERM
    DS --> JSON
    PR --> TERM
    PR --> JSON
    BADGE --> SVG
    TREND --> TERM
    TREND --> JSON
```

```mermaid
graph LR
    subgraph DepsShamer["Deps Shamer - 多生态支持"]
        direction TB
        CARGO["Cargo.toml<br/>Rust"]
        NPM["package.json<br/>Node.js"]
        GOMOD["go.mod<br/>Go"]
        PIP["requirements.txt<br/>Python"]
        PYPROJ["pyproject.toml<br/>Python"]
    end

    subgraph Rules["规则引擎"]
        direction TB
        TRAIT["DepRule / PrRule / Rule<br/>trait 接口"]
        DEFAULT["default_rules()<br/>内置规则"]
        CUSTOM["TOML 配置<br/>自定义规则"]
    end

    subgraph PRMode["PR Title Hunter 模式"]
        direction TB
        LOCAL["本地模式<br/>git2 merge commits"]
        REMOTE["远程模式<br/>GitHub API"]
    end

    DepsShamer --> TRAIT
    TRAIT --> DEFAULT
    TRAIT --> CUSTOM
    PRMode --> LOCAL
    PRMode --> REMOTE
```

## 特性一览

- **多工具分析**：4 个独立工具覆盖代码、commit、依赖、PR
- **多生态依赖**：Cargo.toml、package.json、go.mod、requirements.txt、pyproject.toml
- **GitHub API**：PR Title Hunter 支持远程仓库（`--repo owner/repo`）
- **历史趋势**：用 ASCII 图表追踪质量变化
- **SVG 徽章**：生成 shields.io 风格徽章嵌入 README
- **上下文感知**：对测试/示例/UI 代码自动降低检测灵敏度
- **严重级别加权评分**：每个工具 0-100 分，按惩罚权重扣分
- **双输出格式**：彩色终端或 JSON
- **中英双语**：支持中文和英文吐槽
- **LLM 增强**：可选接入 Ollama 生成创意吐槽
- **VSCode 插件**：编辑器内实时分析

## 快速开始

### 安装

```bash
cargo install garbage-code-hunter
```

### 子命令

#### 代码分析（默认）
```bash
garbage-code-hunter                    # 分析当前目录
garbage-code-hunter src/main.rs        # 分析指定文件
garbage-code-hunter --lang zh-CN src/  # 中文吐槽
garbage-code-hunter --markdown src/    # Markdown 报告（给 AI 用）
garbage-code-hunter --educational      # 显示修复建议
garbage-code-hunter --hall-of-shame    # 显示最烂文件排名
```

#### Commit Roaster
```bash
garbage-code-hunter commit-roaster              # 最近 50 条 commit
garbage-code-hunter cr --limit 100              # 最近 100 条
garbage-code-hunter cr --author "john" --since 2024-01-01
garbage-code-hunter cr -f json                  # JSON 输出
```

#### Deps Shamer
```bash
garbage-code-hunter deps-shamer          # 当前目录
garbage-code-hunter ds /path/to/project  # 指定项目
garbage-code-hunter ds -f json           # JSON 输出
```

#### PR Title Hunter
```bash
# 本地模式（从 merge commits 提取）
garbage-code-hunter pr --limit 100

# 远程模式（GitHub API）
garbage-code-hunter pr --repo owner/repo
garbage-code-hunter pr --repo owner/repo --state open --limit 50
garbage-code-hunter pr --repo owner/repo --token $GITHUB_TOKEN
garbage-code-hunter pr --repo owner/repo --author "username"
```

#### 综合扫描
```bash
garbage-code-hunter scan              # 跑所有工具
garbage-code-hunter scan --save       # 跑完保存到历史
garbage-code-hunter scan -f json      # JSON 输出
```

#### 徽章
```bash
garbage-code-hunter badge                         # 自动评分 + 生成 badge.svg
garbage-code-hunter badge --score 72              # 指定分数
garbage-code-hunter badge -o quality.svg          # 自定义输出路径
garbage-code-hunter badge --style plastic         # 塑料风格
```

#### 历史趋势
```bash
garbage-code-hunter trend              # 显示最近 10 次扫描
garbage-code-hunter trend --last 20    # 显示最近 20 次
garbage-code-hunter trend -f json      # JSON 输出
```

### 输出格式

所有子命令支持 `terminal`（默认彩色）和 `json` 输出：
```bash
garbage-code-hunter cr -f json | jq '.score'
garbage-code-hunter ds -f json | jq '.issues | length'
garbage-code-hunter trend -f json | jq '.records[-1].overall_score'
```

## 示例输出

### Commit Roaster
```
Commit Roast Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
扫描了 50 条 commit，发现 12 个问题

Critical (2)
  * abc1234 "" — commit 消息为空，你是在梦游吗？
  * def5678 "asdf" — 键盘乱拍不是 commit 策略。

High (5)
  * ghi9012 "fix" — 修了啥？"fix" 不是描述，是求救信号。

Score: 76/100 (B)
```

### 历史趋势
```
Quality Trend
  (显示最近 5 次扫描)

  Score
    85 |   ●
       |   |
    80 | --+
       |
        05-01  05-08  05-13

Breakdown
  Overall              75 -> 85 (+10) UP
  code-hunter          65 -> 78 (+13) UP
  commit-roaster       80 -> 82 (+2)  RIGHT

Recent Scans
  2026-05-13T10:00:00  85  .
  2026-05-08T14:30:00  80  .
  2026-05-01T09:00:00  75  .
```

### 综合扫描
```
Running Full Garbage Scan...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  code-hunter: 72/100 (23 个问题，15 个文件)
  commit-roaster: 85/100 (分析了 50 条 commit)
  deps-shamer: 90/100 (45 个依赖)
  pr-title-hunter: 95/100 (检查了 30 个 PR)

Garbage Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Tool Summary
  code-hunter          72/100  (23 items)
  commit-roaster       85/100  (50 items)
  deps-shamer          90/100  (45 items)
  pr-title-hunter      95/100  (30 items)

  Overall Garbage Score: 86/100
```

## 工具详情

### Code Hunter 规则（Rust）
- 单字母变量名
- 无意义命名（data、temp、foo、bar）
- 深嵌套（>4 层）
- 超长函数（>50 行）
- `unwrap()` 滥用
- 魔法数字
- 重复代码块
- 跨文件重复检测
- 上下文感知：测试/示例代码自动降低灵敏度

### Commit Roaster 规则
- 空消息、单词 commit
- WIP commit 推送到共享分支
- 通用消息："fix"、"update"、"change"
- 键盘乱拍（asdf、qwer）
- 全大写、过多感叹号
- 仅版本号变更、默认 merge 消息
- 支持 TOML 规则文件自定义

### Deps Shamer 规则
- 依赖过多（>50）
- Git 依赖
- 通配符版本
- 生产环境用预发布版本
- 过时包（按生态维护列表）
- 重复依赖
- 开发/可选依赖过多

### PR Title Hunter 规则
- 空标题或过短（<5 字符）
- 通用标题（"fix"、"update"、"WIP"）
- 仅 ticket 号（"PROJ-123"、"#456"）
- 全大写、过多感叹号
- 键盘乱拍
- 小写开头（自动跳过 conventional commits）

## VSCode 插件

在 VSCode 中实时吐槽你的代码：

1. 安装 `garbage-code-hunter` CLI
2. 在 VSCode 扩展市场搜索 "Garbage Code Hunter"
3. 保存 Rust 文件时自动触发分析

## 许可证

Apache License 2.0

---

**记住**：我们吐槽的是代码，不是你。让代码审查变得有趣一点！
