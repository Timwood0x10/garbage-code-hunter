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

### Phase 4: 生态扩展

```
- VSCode 插件
- GitHub Action 模板
- GitLab CI 集成
- 自定义规则分享平台
```

### Phase 5: LLM 集成

```
- --with-llm 可选调用 Ollama
- 生成更智能的吐槽文案
- 提供修复建议
```

### Phase 6: 社区

```
- 开源治理
- 贡献者指南
- 插件系统
- 规则市场
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
