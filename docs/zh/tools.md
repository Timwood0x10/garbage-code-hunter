# 工具指南

Garbage Code Hunter 内置 15+ 个分析工具。以下是每个工具的说明。

---

## `analyze` — 代码品味分析

核心工具。使用 tree-sitter AST 解析扫描源文件，报告代码品味问题。

```bash
# 分析当前目录
garbage-code-hunter analyze

# 分析指定路径并过滤语言
garbage-code-hunter analyze ./src --lang rust

# 多个排除模式
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"
```

**输出内容：**
- Issue 统计（Nuclear/Spicy/Mild）
- 质量评分（0-100）
- 每个文件的 issue 明细
- 分类评分（命名、复杂度、重复等）

---

## `commit-roaster` — Commit 消息吐槽

分析 git 提交历史，吐槽烂 commit 消息。

```bash
garbage-code-hunter commit-roaster
garbage-code-hunter commit-roaster --limit 50  # 最近 50 条
```

**检测内容：**
- 空消息、太短（< 5 字符）
- "WIP"、"fix"、"update" 没有上下文
- 键盘乱敲（"asdfgh"、"test test test"）
- 全大写或全小写
- merge commit 泛滥

---

## `deps-shamer` — 依赖羞耻

分析项目依赖，羞耻不良实践。

```bash
garbage-code-hunter deps-shamer
```

**支持：** Cargo (Rust)、npm (JS/TS)、pip (Python)、Go modules、Maven (Java)

**检测内容：**
- 依赖过多
- 生产环境使用预发布版本
- git 依赖
- 过时或废弃的包

---

## `pr-title-hunter` — PR 标题质量

吐槽来自本地分支或 GitHub 的低质量 PR 标题。

```bash
garbage-code-hunter pr-title-hunter
garbage-code-hunter pr-title-hunter --repo owner/repo  # GitHub PR
```

---

## `scan` — 全量扫描

并行运行所有工具，输出综合评分。

```bash
garbage-code-hunter scan ./my-project
```

---

## `last-words` — TODO/FIXME 扫描

发现遗留的 TODO/FIXME/HACK/BUG 注释，报告它们存活了多久。

```bash
garbage-code-hunter last-words
```

---

## `debt-invoice` — 技术债账单

生成"技术债账单"，估算维护成本。

```bash
garbage-code-hunter debt-invoice
```

---

## `personality` — 开发者人格

分析代码模式，判断开发者人格类型。

```bash
garbage-code-hunter personality
```

**人格类型：** 复制粘贴艺术家、unwrap 狂热者、TODO 梦想家等。

---

## `danger-zone` — 危险文件

找出代码库中最危险的文件（issue 密度最高）。

```bash
garbage-code-hunter danger-zone
```

---

## `team-roast` — 团队吐槽

基于 git blame 的按开发者分析。

```bash
garbage-code-hunter team-roast
```

---

## `radar` — 雷达图

生成 SVG 雷达图，展示代码气味分布。

```bash
garbage-code-hunter radar
```

---

## `autopsy` — 代码尸检

生成代码尸检报告，含根因分析。

```bash
garbage-code-hunter autopsy
```

---

## `decay` — 质量衰减

展示项目质量随 git 历史的变化。

```bash
garbage-code-hunter decay
```

---

## `ci-bot` — CI 机器人

生成 CI 风格的 PR 审查评论（用于 GitHub Actions 集成）。

```bash
garbage-code-hunter ci-bot
```

---

## `persona` — 人格分析

用特定人格模式分析代码。

```bash
garbage-code-hunter persona --style senior
garbage-code-hunter persona --style intern
```
