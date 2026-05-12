# Garbage 工具家族开发计划

> 一个月打造通用代码质量吐槽工具集

## 概述

构建一套通用的、语言无关的 CLI 工具家族，用于检测和吐槽代码质量问题。所有工具共享统一的输出风格、评分系统和吐槽逻辑。

### 工具清单

| 工具 | 定位 | 通用性 |
|---|---|---|
| `garbage-code-hunter` | 屎山指数计算器 | 多语言支持 |
| `commit-roaster` | commit 消息吐槽 | 天然通用 |
| `deps-shamer` | 依赖地狱分析 | 多包管理器支持 |
| `pr-title-hunter` | PR 标题检查 | 天然通用 |
| `garbage-cli` | 统一入口 | 整合以上工具 |

### 设计原则

```
1. 语言无关 — 不绑定任何特定语言生态
2. 单二进制 — cargo install xxx 直接用
3. CLI 第一 — 命行交互优先，可选输出格式
4. 可扩展 — 规则可配置，语言可插拔
5. 毒舌但有用 — 吐槽要有依据，不是纯搞笑
```

---

## 技术栈

### 核心依赖

```toml
[dependencies]
# CLI 框架
clap = { version = "4", features = ["derive"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Git 操作
git2 = "0.18"                    # 本地 git 仓库分析
reqwest = { version = "0.12", features = ["json"] }  # GitHub API

# 文件遍历
walkdir = "2"
glob = "0.3"

# 正则匹配
regex = "1"

# 输出美化
colored = "2"
tabled = "0.15"

# 异步
tokio = { version = "1", features = ["full"] }

# 错误处理
anyhow = "1"
thiserror = "1"
```

### 可选依赖

```toml
[features]
default = []
llm = ["reqwest", "serde_json"]  # --with-llm 支持
github = ["reqwest"]              # GitHub API 支持
```

### 开发工具

```
cargo-release    — 版本发布
cargo-cross      — 跨平台编译
cargo-dist       — 自动打包发布
```

---

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    CLI 层 (clap)                        │
│   commit-roaster | deps-shamer | code-hunter | ...     │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                   Core 层 (garbage-core)                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │  输出引擎    │  │  槽点引擎    │  │  评分系统    │    │
│  │  Formatter   │  │  RoastEngine │  │  Scorer      │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                  适配层 (Adapters)                       │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │
│  │  Rust   │ │  Node   │ │   Go    │ │ Python  │     │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘     │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                  数据源层 (Sources)                      │
│      git log | 依赖文件 | 源代码 | GitHub API           │
└─────────────────────────────────────────────────────────┘
```

### 模块划分

```
garbage-ecosystem/
├── Cargo.toml                 # workspace 根
├── crates/
│   ├── garbage-core/          # 共享核心库
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── output/        # 输出格式化
│   │   │   ├── roast/         # 槽点引擎
│   │   │   ├── score/         # 评分系统
│   │   │   ├── lang/          # 语言适配 trait
│   │   │   └── config/        # 配置加载
│   │   └── rules/             # 内置规则文件
│   │
│   ├── commit-roaster/        # commit 吐槽
│   ├── deps-shamer/           # 依赖吐槽
│   ├── code-hunter/           # 屎山指数（重构现有）
│   └── pr-title-hunter/       # PR 标题吐槽
│
└── garbage-cli/               # 统一入口（Phase 3）
```

---

## 数据结构设计

### 核心类型

```rust
// ========== 问题/槽点定义 ==========

/// 检测到的问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,                    // 问题 ID，如 "commit-empty-msg"
    pub category: Category,            // 分类
    pub severity: Severity,            // 严重程度
    pub message: String,               // 槽点文案
    pub location: Option<Location>,    // 位置信息
    pub context: HashMap<String, String>,  // 附加上下文
}

/// 问题分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Commit,        // commit 相关
    Dependency,    // 依赖相关
    Code,          // 代码质量
    Naming,        // 命名规范
    Structure,     // 结构问题
    Style,         // 风格问题
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,   // 致命，必须修
    High,       // 高，强烈建议修
    Medium,     // 中，最好修一下
    Low,        // 低，无所谓
    Info,       // 信息，仅供参考
}

/// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Location {
    File { path: String, line: Option<u32> },
    Commit { hash: String },
    Dependency { name: String, version: String },
}

// ========== 评分系统 ==========

/// 评分结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreReport {
    pub total_score: f64,              // 总分 0-100
    pub grade: Grade,                  // 等级
    pub breakdown: Vec<CategoryScore>, // 分类得分
    pub roasts: Vec<String>,           // 汇总吐槽
    pub stats: Stats,                  // 统计数据
}

/// 等级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Grade {
    S,      // 90-100: 神级代码
    A,      // 80-89:  优秀
    B,      // 70-79:  良好
    C,      // 60-69:  及格
    D,      // 50-59:  及格边缘
    F,      // 0-49:   屎山
}

/// 分类得分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub category: Category,
    pub score: f64,
    pub weight: f64,
    pub issue_count: usize,
}

/// 统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub files_scanned: usize,
    pub lines_of_code: usize,
    pub issues_found: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

// ========== 依赖分析 ==========

/// 依赖信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
    pub is_dev: bool,
    pub is_optional: bool,
}

/// 依赖来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    Registry,       // crates.io, npm 等
    Git { url: String },
    Path { path: String },
    Unknown,
}

/// 依赖分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepReport {
    pub total: usize,
    pub outdated: usize,
    pub unused: usize,
    pub duplicated: usize,
    pub issues: Vec<Issue>,
}

// ========== Commit 分析 ==========

/// Commit 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
    pub message: String,
    pub files_changed: Vec<String>,
    pub insertions: usize,
    pub deletions: usize,
}

/// Commit 分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitReport {
    pub total_commits: usize,
    pub issues: Vec<Issue>,
    pub worst_commits: Vec<(CommitInfo, Vec<Issue>)>,
    pub stats: CommitStats,
}

/// Commit 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitStats {
    pub avg_message_length: f64,
    pub empty_messages: usize,
    pub single_word_messages: usize,
    pub wip_commits: usize,
    pub fixup_commits: usize,
}
```

### 规则定义

```rust
/// 规则定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: Category,
    pub severity: Severity,
    pub pattern: RulePattern,
    pub message_template: String,
    pub enabled: bool,
}

/// 规则匹配模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RulePattern {
    Regex(String),                      // 正则匹配
    Contains(String),                   // 包含关键词
    StartsWith(String),                 // 以...开头
    EndsWith(String),                   // 以...结尾
    Length { min: Option<usize>, max: Option<usize> },  // 长度检查
    Custom(String),                     // 自定义函数名
}

/// 规则集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub name: String,
    pub version: String,
    pub rules: Vec<Rule>,
}
```

### 输出格式

```rust
/// 输出格式
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Table,      // 终端表格（默认）
    Json,       // JSON
    Markdown,   // Markdown
    Plain,      // 纯文本
}

/// 输出配置
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub color: bool,
    pub emoji: bool,
    pub verbose: bool,
    pub quiet: bool,
}
```

---

## 工具设计思路

### 1. commit-roaster

**定位：** 扫描 git log，疯狂吐槽 commit 消息

**核心逻辑：**
```
1. 打开 git 仓库（或指定 commit 范围）
2. 遍历 commit 历史
3. 对每条 commit 应用规则匹配
4. 生成吐槽报告
```

**内置规则：**
```toml
# commit 规则示例
[[rules]]
id = "empty-message"
pattern = { Regex = "^$" }
message = "commit 消息是空的，你是在梦游吗？"
severity = "Critical"

[[rules]]
id = "too-short"
pattern = { Length = { max = 10 } }
message = "这么短？连个主语都没有？"
severity = "High"

[[rules]]
id = "wip"
pattern = { Contains = "WIP" }
message = "WIP 提交？你确定这不是半成品？"
severity = "Medium"

[[rules]]
id = "fix-fix-fix"
pattern = { Regex = "^(fix|fix|fix)" }
message = "你到底修了什么？三次 fix 还没修好？"
severity = "High"

[[rules]]
id = "typo-commit"
pattern = { Regex = "^(typo|fix typo|typos)$" }
message = "又拼写错误？你的键盘是不是有问题？"
severity = "Low"

[[rules]]
id = "update-update"
pattern = { Regex = "^(update|update|update)" }
message = "update 了什么？更新了寂寞？"
severity = "Medium"

[[rules]]
id = "asdf-commit"
pattern = { Regex = "^(asdf|asdfg|qwer)" }
message = "键盘乱按也算 commit？"
severity = "Critical"
```

**使用方式：**
```bash
# 分析当前仓库最近 50 条 commit
commit-roaster

# 分析指定范围
commit-roaster --since "2024-01-01" --until "2024-12-31"

# 分析指定分支
commit-roaster --branch main

# 只看某个人的
commit-roaster --author "zhangsan"

# 输出 JSON
commit-roaster --format json

# 只看严重问题
commit-roaster --severity high
```

**输出示例：**
```
🔥 Commit Roast Report 🔥
━━━━━━━━━━━━━━━━━━━━━━━━

扫描了 127 条 commit，发现 43 个问题

💀 致命问题 (5)
  • abc1234: "" — commit 消息是空的，你是在梦游吗？
  • def5678: "fix" — 一个字就想打发我？
  • ghi9012: "asdf" — 键盘乱按也算 commit？

😡 高严重 (12)
  • jkl3456: "update" — update 了什么？更新了寂寞？
  • mno7890: "fix bug" — 哪个 bug？bug 叫什么名字？
  ...

📊 统计
  平均消息长度: 12 字符（你的作文是体育老师教的吗？）
  WIP 提交: 8 个（半成品仓库实锤）
  空消息: 3 个（沉默是金？）
  单词消息: 15 个（词汇量堪忧）

🏆 最差 Commit 奖
  获奖者: "fix" (出现 23 次)
  颁奖词: 你把 fix 当逗号用呢？
```

---

### 2. deps-shamer

**定位：** 分析依赖文件，吐槽依赖地狱

**核心逻辑：**
```
1. 扫描项目根目录
2. 识别依赖文件类型（Cargo.toml, package.json, go.mod...）
3. 解析依赖列表
4. 应用规则检查
5. 生成吐槽报告
```

**支持的包管理器：**
```
Rust    → Cargo.toml
Node    → package.json
Go      → go.mod
Python  → requirements.txt, pyproject.toml, setup.py
Java    → pom.xml, build.gradle
Ruby    → Gemfile
PHP     → composer.json
.NET    → *.csproj
```

**内置规则：**
```toml
[[rules]]
id = "too-many-deps"
message = "依赖比你朋友还多？{count} 个依赖，你是开超市的吗？"
severity = "Medium"
threshold = 50

[[rules]]
id = "unused-deps"
message = "依赖了 {name} 但代码里根本没用，是买来装饰的吗？"
severity = "High"

[[rules]]
id = "git-deps"
message = "直接引用 Git 仓库 {url}？你是不信任包管理器吗？"
severity = "Medium"

[[rules]]
id = "wildcard-version"
message = "版本号用 *？你是想体验每日 breaking change 吗？"
severity = "High"

[[rules]]
id = "pre-release"
message = "生产环境用预发布版本 {version}？你真勇敢"
severity = "Medium"

[[rules]]
id = "deprecated-dep"
message = "{name} 已经废弃了，你是考古学家吗？"
severity = "High"

[[rules]]
id = "duplicated-dep"
message = "同一个依赖出现两次？你的 Ctrl+C 和 Ctrl+V 很忙啊"
severity = "Medium"
```

**使用方式：**
```bash
# 分析当前项目
deps-shamer

# 分析指定文件
deps-shamer path/to/Cargo.toml
deps-shamer path/to/package.json

# 分析多个项目
deps-shamer project1/ project2/

# 检查是否过期（需要网络）
deps-shamer --check-outdated

# 输出 JSON
deps-shamer --format json
```

**输出示例：**
```
📦 Dependency Shame Report 📦
━━━━━━━━━━━━━━━━━━━━━━━━━━━

项目: my-awesome-app (Rust/Cargo)
依赖总数: 47

😱 震惊！你居然依赖了这些：
  • tokio — 你确定需要异步吗？一个 Hello World 也要异步？
  • serde — 全世界都在用，但你确定你序列化了什么？
  • rand — 随机数？你的代码是在赌博吗？

🗑️ 垃圾分类：
  • lazy_static — 2024 年了还用这个？用 once_cell 吧求你了
  • failure — 这个 crate 自己都 failure 了，换 anyhow 吧

⚠️ 版本问题：
  • regex = "*" — 版本号是薛定谔的猫吗？
  • some-lib = "0.1.0-alpha" — 生产环境用 alpha？勇气可嘉

📊 统计
  直接依赖: 23
  开发依赖: 12
  可选依赖: 8
  Git 依赖: 4（你是不是不信任 crates.io？）

🏆 最多依赖奖
  获奖者: tokio (被 15 个间接依赖引入)
  颁奖词: 你的项目是 tokio 的形状
```

---

### 3. code-hunter (重构)

**定位：** 屎山指数计算器，给整个项目打分

**重构方向：**
- 从 Rust 专用改为多语言支持
- 加入语言检测机制
- 规则可配置化

**语言检测：**
```rust
fn detect_languages(project_path: &Path) -> Vec<Language> {
    // 1. 检查依赖文件
    // 2. 统计文件扩展名
    // 3. 返回主要语言列表
}
```

**多语言规则：**
```
rules/
├── common.toml        # 通用规则
├── rust.toml          # Rust 特有规则
├── javascript.toml    # JS/TS 规则
├── python.toml        # Python 规则
├── go.toml            # Go 规则
└── java.toml          # Java 规则
```

**评分权重：**
```toml
[scoring]
# 各分类权重
commit_weight = 0.15
dependency_weight = 0.15
code_quality_weight = 0.40
naming_weight = 0.15
structure_weight = 0.15

# 严重程度扣分
[scoring.penalty]
critical = 10
high = 5
medium = 2
low = 0.5
info = 0
```

**使用方式：**
```bash
# 分析当前项目
code-hunter

# 分析指定目录
code-hunter path/to/project

# 指定语言（跳过自动检测）
code-hunter --lang rust

# 只分析特定方面
code-hunter --only code,naming

# 排除某些目录
code-hunter --exclude node_modules,target,.git

# 输出详细报告
code-hunter --verbose

# 输出 JSON
code-hunter --format json
```

**输出示例：**
```
🏔️ 屎山指数报告 🏔️
━━━━━━━━━━━━━━━━━━

项目: my-awesome-app
语言: Rust (95%), TOML (5%)

🏆 综合评分: 62/100 (C 级)

┌────────────┬───────┬──────┬────────┐
│ 分类       │ 得分  │ 权重 │ 问题数 │
├────────────┼───────┼──────┼────────┤
│ 代码质量   │ 55    │ 40%  │ 23     │
│ 命名规范   │ 70    │ 15%  │ 8      │
│ 依赖管理   │ 65    │ 15%  │ 5      │
│ Commit     │ 60    │ 15%  │ 12     │
│ 项目结构   │ 75    │ 15%  │ 3      │
└────────────┴───────┴──────┴────────┘

🔥 Top 5 致命问题：
  1. src/main.rs:142 — unwrap() 大会会员，这里 panic 了怎么办？
  2. src/lib.rs:89 — 300 行函数，你是写小说吗？
  3. Cargo.toml — 47 个依赖，你确定不是开超市？
  4. commit "fix" — 出现 23 次，fix 是你的口头禅吗？
  5. src/utils.rs:1 — 文件名叫 utils，里面什么都有，垃圾场吗？

💬 综合吐槽：
  这个项目就像一栋年久失修的房子：
  - 地基还行（项目结构尚可）
  - 墙壁有裂缝（代码质量堪忧）
  - 装修很随意（命名随心所欲）
  - 家具太多（依赖膨胀）
  
  建议：先从 unwrap() 开始修，它们就像定时炸弹

📊 统计
  文件数: 87
  代码行数: 12,450
  问题总数: 51
  致命: 5 | 高: 12 | 中: 20 | 低: 14
```

---

### 4. pr-title-hunter

**定位：** 检查 PR 标题质量

**核心逻辑：**
```
模式 1: 本地检查
  → 读取 git log 中的 merge commit
  → 提取 PR 标题

模式 2: GitHub API
  → 调用 GitHub API 获取 PR 列表
  → 批量检查标题
```

**内置规则：**
```toml
[[rules]]
id = "generic-title"
pattern = { Regex = "^(fix|update|change|modify|refactor)$" }
message = "PR 标题是 '{title}'？你是机器人吗？"
severity = "High"

[[rules]]
id = "too-short"
pattern = { Length = { max = 10 } }
message = "PR 标题这么短？你是发电报吗？"
severity = "Medium"

[[rules]]
id = "no-verb"
message = "PR 标题没有动词，你是名词收集器吗？"
severity = "Low"

[[rules]]
id = "jira-ticket-only"
pattern = { Regex = "^[A-Z]+-\\d+$" }
message = "只有 ticket 号？标题是给人看的，不是给 JIRA 看的"
severity = "Medium"

[[rules]]
id = "wip-pr"
pattern = { Regex = "(?i)^(WIP|draft|DO NOT MERGE)" }
message = "WIP PR？那为什么要开 PR？"
severity = "Info"

[[rules]]
id = "exclamation-marks"
pattern = { Regex = "!{2,}" }
message = "这么多感叹号？你是有多激动？"
severity = "Low"
```

**使用方式：**
```bash
# 检查本地仓库的 PR（通过 merge commit）
pr-title-hunter

# 检查 GitHub 仓库的 PR
pr-title-hunter --repo owner/repo

# 检查 open 的 PR
pr-title-hunter --repo owner/repo --state open

# 检查最近 N 个 PR
pr-title-hunter --repo owner/repo --limit 50

# 检查某个人的 PR
pr-title-hunter --repo owner/repo --author "zhangsan"

# 需要 token
pr-title-hunter --repo owner/repo --token $GITHUB_TOKEN
```

**输出示例：**
```
🎯 PR Title Roast Report 🎯
━━━━━━━━━━━━━━━━━━━━━━━━━━━

仓库: owner/repo
检查了 35 个 PR

💀 最差 PR 标题 Top 5：
  #123: "fix" — 修了什么？修了个寂寞？
  #124: "update" — 更新了空气？
  #125: "WIP" — 那你开 PR 干嘛？
  #126: "PROJ-123" — 你是人还是机器人？
  #127: "asdfgh" — 键盘跳舞呢？

📊 统计
  平均标题长度: 24 字符（勉强及格）
  无动词标题: 8 个（22%）
  只有 ticket 号: 5 个（14%）
  WIP PR: 3 个（8%）

🏆 最佳 PR 标题奖
  获奖者: "feat(auth): implement OAuth2 login flow with PKCE"
  颁奖词: 这才是人写的标题！
```

---

### 5. garbage-cli (Phase 3)

**定位：** 统一入口，子命令模式

**使用方式：**
```bash
# 子命令调用
garbage commit-roaster
garbage deps-shamer
garbage code-hunter
garbage pr-title-hunter

# 全面扫描
garbage scan

# 生成综合报告
garbage report --format html > report.html
```

**综合报告输出：**
```
🗑️ Garbage Report 🗑️
━━━━━━━━━━━━━━━━━━━

项目: my-awesome-app
扫描时间: 2024-01-15 14:30:00

┌─────────────────┬───────┬────────┐
│ 工具            │ 得分  │ 问题数 │
├─────────────────┼───────┼────────┤
│ code-hunter     │ 62    │ 51     │
│ commit-roaster  │ 45    │ 43     │
│ deps-shamer     │ 58    │ 12     │
│ pr-title-hunter │ 70    │ 8      │
└─────────────────┴───────┴────────┘

🏆 综合屎山指数: 59/100 (D 级)

💬 一句话总结：
  这个项目就像一个堆满杂物的仓库，
  代码还行，但 commit 和依赖管理是灾难
```

---

## 开发计划

### 周次安排

```
Week 1: 基础设施 + commit-roaster
Week 2: deps-shamer + 核心库完善
Week 3: code-hunter 重构 + pr-title-hunter
Week 4: 测试 + 文档 + 发布准备
```

### 详细计划

#### Week 1: 基础设施 + commit-roaster

**Day 1-2: 项目初始化**
- [ ] 创建 workspace 结构
- [ ] 搭建 garbage-core crate
- [ ] 定义核心数据结构
- [ ] 实现输出格式化（表格、JSON、Markdown）
- [ ] 实现基础的槽点引擎

**Day 3-4: commit-roaster 核心**
- [ ] 实现 git log 解析
- [ ] 实现规则匹配引擎
- [ ] 编写 commit 规则集
- [ ] 实现基础 CLI

**Day 5: commit-roaster 完善**
- [ ] 添加过滤选项（author、date、branch）
- [ ] 添加统计功能
- [ ] 编写测试
- [ ] 本地测试运行

#### Week 2: deps-shamer + 核心库完善

**Day 1-2: deps-shamer 基础**
- [ ] 实现依赖文件检测
- [ ] 实现 Cargo.toml 解析器
- [ ] 实现 package.json 解析器
- [ ] 实现 go.mod 解析器

**Day 3: deps-shamer 扩展**
- [ ] 实现更多语言支持（Python、Java）
- [ ] 编写依赖规则集
- [ ] 实现依赖统计

**Day 4-5: 核心库完善**
- [ ] 完善语言适配层
- [ ] 完善规则配置系统
- [ ] 添加自定义规则支持
- [ ] 编写测试

#### Week 3: code-hunter 重构 + pr-title-hunter

**Day 1-2: code-hunter 重构**
- [ ] 重构现有代码，使用 garbage-core
- [ ] 实现多语言支持
- [ ] 实现语言自动检测
- [ ] 配置化评分权重

**Day 3: code-hunter 完善**
- [ ] 添加更多语言规则
- [ ] 优化评分算法
- [ ] 完善输出格式

**Day 4-5: pr-title-hunter**
- [ ] 实现本地 PR 检测
- [ ] 实现 GitHub API 集成
- [ ] 编写 PR 规则集
- [ ] 实现 CLI

#### Week 4: 测试 + 文档 + 发布

**Day 1-2: 测试**
- [ ] 集成测试
- [ ] 跨平台测试
- [ ] 性能优化

**Day 3: 文档**
- [ ] 编写 README
- [ ] 编写使用文档
- [ ] 编写贡献指南

**Day 4-5: 发布准备**
- [ ] 配置 CI/CD
- [ ] 配置跨平台编译
- [ ] 准备 crates.io 发布
- [ ] 创建 GitHub Release

---

## 发布策略

### 版本号

```
0.1.0  — MVP，基本功能可用
0.2.0  — 多语言支持完善
0.3.0  — 自定义规则支持
1.0.0  — 稳定版本，API 不再变更
```

### 发布渠道

```
crates.io        — cargo install commit-roaster
GitHub Release   — 预编译二进制
Homebrew         — 后续考虑
```

### 跨平台

```
Linux x86_64
Linux ARM64
macOS x86_64
macOS ARM64
Windows x86_64
```

---

## 后续扩展

### Phase 4: 多语言代码分析

**目标：** code-hunter 支持 JS/TS、Python、Go，从 Rust 专用工具升级为多语言工具。

**实现策略：** 先用正则 + 简单 AST，不做完整解析。每种语言独立规则文件。

```
src/rules/
├── rust/           # 现有 Rust 规则（31 条）
├── javascript/     # JS/TS 规则
│   ├── mod.rs
│   └── rules/      # naming.rs, async.rs, callback_hell.rs, any_type.ts 等
├── python/         # Python 规则
│   ├── mod.rs
│   └── rules/      # naming.rs, bare_except.rs, global_vars.rs, type_hint.rs 等
└── go/             # Go 规则
    ├── mod.rs
    └── rules/      # naming.rs, error_ignored.rs, goroutine_leak.rs 等
```

**语言检测：**
```rust
fn detect_language(path: &Path) -> Vec<Language> {
    // 1. 检查依赖文件 (Cargo.toml → Rust, package.json → JS, go.mod → Go)
    // 2. 统计文件扩展名占比
    // 3. 返回主要语言列表，按比例排序
}
```

**JS/TS 规则（初始）：**
- `any-type` — 使用 `any` 类型（TypeScript）
- `callback-hell` — 回调嵌套过深
- `console-log` — 生产代码残留 `console.log`
- `var-usage` — 使用 `var` 而非 `let/const`
- `eqeqeq` — 使用 `==` 而非 `===`
- `promise-no-await` — Promise 没有 await
- `magic-number` — 魔法数字

**Python 规则（初始）：**
- `bare-except` — 裸 `except` 捕获所有异常
- `global-vars` — 全局变量滥用
- `no-type-hint` — 缺少类型注解
- `print-debug` — `print()` 调试残留
- `mutable-default` — 可变默认参数（`def f(a=[])`）
- `wildcard-import` — `from x import *`

**Go 规则（初始）：**
- `error-ignored` — 忽略 error 返回值
- `goroutine-leak` — goroutine 泄漏
- `naked-return` — 裸 return
- `panic-in-lib` — 库代码中使用 panic
- `shadowed-err` — err 变量遮蔽

**CLI 变更：**
```bash
garbage-code-hunter analyze --lang-rust --lang-js --lang-python --lang-go src/
garbage-code-hunter analyze --auto-detect src/    # 自动检测语言
```

**时间估算：** 2 周
- Week 1: 语言检测 + JS/TS 规则
- Week 2: Python 规则 + Go 规则 + 测试

---

### Phase 5: GitHub API 集成

**目标：** pr-title-hunter 支持远程仓库 PR 检查，不限于本地 merge commits。

**CLI 变更：**
```bash
# 本地模式（现有）
garbage-code-hunter pr

# 远程模式（新增）
garbage-code-hunter pr --repo owner/repo
garbage-code-hunter pr --repo owner/repo --state open
garbage-code-hunter pr --repo owner/repo --limit 100
garbage-code-hunter pr --repo owner/repo --author "zhangsan"
garbage-code-hunter pr --repo owner/repo --token $GITHUB_TOKEN
```

**实现：**
```
src/pr_title_hunter/
├── mod.rs          # 现有
├── types.rs        # 现有
├── rules.rs        # 现有
├── report.rs       # 现有
└── github.rs       # 新增：GitHub API 客户端
```

**核心逻辑：**
```rust
// github.rs
pub async fn fetch_prs(repo: &str, config: &GitHubConfig) -> Result<Vec<PrEntry>> {
    // GET /repos/{owner}/{repo}/pulls
    // 支持 state (open/closed/all), per_page, page
    // 支持 token 认证（避免限流）
}
```

**依赖：** 复用现有 `reqwest` 依赖，不需要新增。

**时间估算：** 3 天
- Day 1: GitHub API 客户端 + 认证
- Day 2: 集成到 CLI + 远程模式
- Day 3: 测试 + 错误处理

---

### Phase 6: Badge 生成

**目标：** 生成 SVG 分数徽章，嵌入 README 展示项目质量。

**CLI 变更：**
```bash
garbage-code-hunter badge                    # 生成 badge.svg（默认）
garbage-code-hunter badge -o quality.svg     # 指定输出文件
garbage-code-hunter badge --style flat       # 扁平风格
garbage-code-hunter badge --style plastic    # 塑料风格
```

**实现：**
```
src/badge/
├── mod.rs          # 入口
├── generator.rs    # SVG 生成器
└── templates/      # SVG 模板
    ├── flat.svg
    └── plastic.svg
```

**SVG 模板设计：**
```xml
<svg xmlns="http://www.w3.org/2000/svg" width="160" height="20">
  <linearGradient id="b" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <mask id="a"><rect width="160" height="20" rx="3" fill="#fff"/></mask>
  <g mask="url(#a)">
    <rect width="75" height="20" fill="#555"/>
    <rect x="75" width="85" height="20" fill="#4c1"/>
    <rect width="160" height="20" fill="url(#b)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="..." font-size="11">
    <text x="37.5" y="15" fill="#010101" fill-opacity=".3">garbage</text>
    <text x="37.5" y="14">garbage</text>
    <text x="117.5" y="15" fill="#010101" fill-opacity=".3">{score}</text>
    <text x="117.5" y="14">{score}</text>
  </g>
</svg>
```

**颜色映射：**
- 90-100: `#4c1` (绿色)
- 70-89: `#97CA00` (黄绿色)
- 50-69: `#dfb317` (黄色)
- 30-49: `#fe7d37` (橙色)
- 0-29: `#e05d44` (红色)

**时间估算：** 2 天
- Day 1: SVG 模板 + 生成器
- Day 2: CLI 集成 + 测试

---

### Phase 7: 历史趋势

**目标：** 对比多次扫描结果，显示代码质量变化趋势。

**CLI 变更：**
```bash
garbage-code-hunter trend                    # 显示最近 10 次扫描趋势
garbage-code-hunter trend --last 20          # 显示最近 20 次
garbage-code-hunter trend --since 2024-01-01 # 指定时间范围
garbage-code-hunter trend --format json      # JSON 输出
```

**数据存储：**
```
~/.garbage-code-hunter/
└── history/
    ├── 2024-01-15T14:30:00.json
    ├── 2024-01-16T09:00:00.json
    └── ...
```

**历史记录格式：**
```json
{
  "timestamp": "2024-01-15T14:30:00Z",
  "project_path": "/path/to/project",
  "overall_score": 72.5,
  "tools": {
    "code-hunter": { "score": 65, "issues": 45 },
    "commit-roaster": { "score": 80, "issues": 12 },
    "deps-shamer": { "score": 90, "issues": 3 },
    "pr-title-hunter": { "score": 75, "issues": 5 }
  }
}
```

**终端输出示例：**
```
📈 Quality Trend (last 10 scans)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Overall Score
  85 ┤                                    ╭── 82
  80 ┤                              ╭─────╯
  75 ┤                        ╭─────╯
  70 ┤                  ╭─────╯
  65 ┤            ╭─────╯
  60 ┤      ╭─────╯
  55 ┤──────╯
     └────┬────┬────┬────┬────┬────┬────┬────
       Jan1  Jan5  Jan9  Jan13 Jan17 Jan21 Jan25

  📊 Breakdown:
    code-hunter:     65 → 78 (+13) 📈
    commit-roaster:  80 → 82 (+2)  📈
    deps-shamer:     90 → 90 (=)   ➡️
    pr-title-hunter: 75 → 78 (+3)  📈
```

**实现：**
```
src/trend/
├── mod.rs          # 入口
├── history.rs      # 历史记录读写
└── display.rs      # 趋势图渲染（ASCII）
```

**时间估算：** 3 天
- Day 1: 历史记录存储 + 读取
- Day 2: 趋势图 ASCII 渲染
- Day 3: CLI 集成 + 测试

---

### Phase 8: 自动修复 (garbage fix)

**目标：** 自动修复简单代码问题，减少人工干预。

**CLI 变更：**
```bash
garbage-code-hunter fix                      # 修复当前目录所有可修复问题
garbage-code-hunter fix src/                 # 修复指定目录
garbage-code-hunter fix --dry-run            # 预览修复，不实际修改
garbage-code-hunter fix --rule unwrap-abuse  # 只修复特定规则
```

**可修复规则（第一阶段）：**
```
规则 ID               修复方式
─────────────────────────────────────────────────────
unwrap-abuse          .unwrap() → .expect("context")
single-letter-var     单字母变量 → 推断有意义名称
console-log (JS)      console.log → 注释或删除
var-usage (JS)        var → let/const
print-debug (Python)  print() → 注释或删除
bare-except (Python)  except: → except Exception:
```

**实现策略：**
```rust
// 使用 syn/proc-macro2 进行精确的 AST 级别修改
trait AutoFix: Rule {
    fn fix(&self, file: &str, issue: &Issue) -> Result<String>;
}

// unwrap → expect 示例
fn fix_unwrap(source: &str, line: usize) -> Result<String> {
    // 1. 找到 unwrap() 位置
    // 2. 分析上下文推断 expect 消息
    // 3. 替换 .unwrap() → .expect("descriptive message")
}
```

**安全策略：**
- 默认创建 `.bak` 备份文件
- `--dry-run` 模式预览变更
- 修复前自动运行 `cargo check` 验证
- 修复后自动运行 `make fmt`

**时间估算：** 1 周
- Day 1-2: AutoFix trait + unwrap 修复
- Day 3: JS/Python 简单修复
- Day 4-5: 测试 + 安全验证

---

### Phase 9: VS Code 集成增强

**目标：** 实时在编辑器中显示代码质量分数，无需离开 IDE。

**功能清单：**
```
功能                    描述
─────────────────────────────────────────────────────
实时诊断                保存文件时自动扫描，显示问题标记
状态栏分数              底部状态栏显示当前文件质量分数
CodeLens                函数上方显示复杂度评分
Quick Fix               右键快速修复（集成 garbage fix）
侧边栏面板              显示项目整体质量报告
配置界面                GUI 配置规则和阈值
```

**技术方案：**
- 基于现有 `vscode-extension/` TypeScript 扩展
- 使用 `Language Server Protocol (LSP)` 通信
- 后端调用 `garbage-code-hunter` CLI 或直接调用 Rust 库

**架构：**
```
VS Code Extension (TypeScript)
├── extension.ts        # 入口
├── client.ts           # LSP 客户端
├── statusBar.ts        # 状态栏
├── codeLens.ts         # CodeLens provider
└── quickFix.ts         # Quick Fix provider

Rust LSP Server (新增)
├── src/lsp/
│   ├── mod.rs
│   ├── server.rs       # LSP server 实现
│   ├── diagnostics.rs  # 诊断信息
│   └── handlers.rs     # 请求处理
```

**时间估算：** 2 周
- Week 1: LSP server + 诊断信息
- Week 2: VS Code 扩展增强 + CodeLens + Quick Fix

---

### Phase 10: CI/CD 集成 + 自定义规则

**目标：** `garbage check` 子命令用于 CI/CD 流水线，支持自定义规则配置。

**CLI 变更：**
```bash
# CI 模式：分数低于阈值返回非零 exit code
garbage-code-hunter check --threshold 70
garbage-code-hunter check --threshold 80 --format json

# 自定义规则
garbage-code-hunter check --config ./garbage.toml
```

**自定义规则配置 (`garbage.toml`)：**
```toml
[general]
threshold = 70
exclude = ["target/*", "node_modules/*", ".git/*"]

[code_hunter]
enabled = true
rules = { unwrap-abuse = "error", magic-number = "warn" }

[commit_roaster]
enabled = true
rules = { too-short = "error", wip-commit = "warn" }

[deps_shamer]
enabled = true
too_many_deps_threshold = 50
rules = { wildcard-version = "error", deprecated-dep = "warn" }

[pr_title_hunter]
enabled = true
rules = { generic-title = "error", ticket-only = "warn" }

[custom_rules]
# 用户自定义规则
[[custom_rules.regex]]
id = "no-todo-in-main"
pattern = "TODO"
severity = "warn"
message = "TODO found in main branch — time to fix it?"
apply_to = "src/main.rs"
```

**时间估算：** 1 周
- Day 1-2: `check` 子命令 + 阈值逻辑
- Day 3-4: 配置文件解析 + 规则覆盖
- Day 5: 测试 + 文档

---

### 总体时间线

```
Phase 4:  多语言代码分析          2 周
Phase 5:  GitHub API 集成         3 天
Phase 6:  Badge 生成              2 天
Phase 7:  历史趋势                3 天
Phase 8:  自动修复                1 周
Phase 9:  VS Code 集成增强        2 周
Phase 10: CI/CD + 自定义规则      1 周
────────────────────────────────────────
总计:                             ~7 周
```

**优先级排序：**
```
P0 (立即):  Phase 10 CI/CD + 自定义规则
P1 (高):    Phase 4 多语言分析
P2 (中):    Phase 5 GitHub API + Phase 6 Badge + Phase 7 趋势
P3 (低):    Phase 8 自动修复 + Phase 9 VS Code
```

---

## 风险与挑战

### 技术风险

```
1. 多语言 AST 解析复杂度高
   → 解决：先用正则，不做完整 AST

2. GitHub API 限流
   → 解决：支持 token，本地优先

3. 跨平台兼容性
   → 解决：用 cross 编译，CI 测试
```

### 项目风险

```
1. 范围蔓延
   → 解决：严格 MVP，先做核心功能

2. 维护负担
   → 解决：规则可配置，社区贡献

3. 用户接受度
   → 解决：毒舌但有用，不是纯搞笑
```

---

## 总结

这是一个雄心勃勃但可行的计划。核心思路：

1. **先做能用的** — MVP 优先
2. **再做好用的** — 完善功能
3. **最后做通用的** — 多语言支持

一个月时间紧凑，但只要坚持 MVP 原则，完全可以做出 4 个可用的工具。

关键成功因素：
- 坚持通用设计，不绑定特定语言
- 规则可配置，减少硬编码
- 输出风格统一，形成品牌
- 毒舌但有用，不是纯搞笑

开始吧！🚀
