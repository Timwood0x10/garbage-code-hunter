# Garbage Code Hunter 文档

Garbage Code Hunter 是一个带毒舌风格的 CLI 工具集，用来分析代码品味。
它面向娱乐和代码可读性检查，不用于找 bug 或做安全审计。

> 灵感来源：[fuck-u-code](https://github.com/Done-0/fuck-u-code.git)

## 一句话概览

- 使用 tree-sitter 解析源码，尽量做语法级分析
- 使用 StyleIR 作为核心证据层，把不同语言转换成统一的风格事实
- 支持 11 种语言：Rust、Go、Python、JavaScript、TypeScript、Java、C、C++、Ruby、Swift、Zig
- 提供代码、commit、依赖、PR 标题、趋势等多种吐槽工具
- 同时支持本地固定文案和可选的 LLM 生成模式

## StyleIR

StyleIR 是这个项目最关键的架构改进。它不让每条检测规则直接面对每种语言的 AST，而是先由语言适配器抽取一组紧凑的风格事实：函数、行数、命名违规、嵌套、调试输出、panic 风险调用、魔法数字、TODO、重复 import、unsafe block 以及语言特定计数。

这样报告、评分、JSON 输出和后续检测器都能共享同一层证据，同时把语言差异隔离在 adapter 里。

## 它会检查什么

- 糟糕命名，比如 `data`、`tmp`、`foo`、`bar`
- 魔法数字和重复常量
- 过深嵌套和过长函数
- 什么都干的上帝函数
- 遗留的 `println`、`fmt.Println` 调试语句
- 被注释掉的代码和过期的 TODO/FIXME/HACK
- 跨文件重复实现

## 安装

```bash
cargo install garbage-code-hunter
```

## 快速开始

```bash
# 分析当前目录
garbage-code-hunter analyze

# 分析指定路径和语言
garbage-code-hunter analyze ./my-go-project --lang go

# 运行全量扫描
garbage-code-hunter scan ./my-project
```

## 主要命令

| 命令 | 别名 | 说明 |
|---|---|---|
| `analyze` | - | 核心代码品味分析 |
| `commit-roaster` | `cr` | 吐槽 git 历史里的 commit 消息 |
| `deps-shamer` | `ds` | 检查依赖管理习惯 |
| `pr-title-hunter` | `pr` | 吐槽本地或 GitHub PR 标题 |
| `scan` | - | 跑完整套工具并汇总分数 |
| `badge` | - | 生成 SVG 评分徽章 |
| `trend` | - | 查看分数趋势 |
| `last-words` | `lw` | 查找陈旧的 TODO/FIXME/HACK 注释 |
| `debt-invoice` | `debt` | 估算技术债成本 |
| `personality` | - | 根据代码模式推断开发者画像 |
| `decay` | - | 分析 git 历史中的质量衰减 |
| `autopsy` | - | 输出“尸检式”问题报告 |
| `radar` | - | 生成代码气味雷达图 |
| `ci-bot` | - | 生成 CI 风格审查评论 |
| `persona` | - | 使用指定人格模式吐槽代码 |
| `danger-zone` | `dz` | 找出最危险的文件 |
| `team-roast` | - | 按作者汇总问题 |

## 常用示例

```bash
# 输出 JSON
garbage-code-hunter analyze -f json

# 排除生成文件
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"

# 保存一次扫描结果
garbage-code-hunter scan --save

# 生成徽章
garbage-code-hunter badge --output badge.svg
```

## 配置

- 项目配置文件：`.garbage-code-hunter.toml`
- 搜索顺序：当前目录，然后向上级目录继续查找
- 详细配置：[`configuration.md`](configuration.md)
- 规则说明：[`rules.md`](rules.md)
- 工具说明：[`tools.md`](tools.md)

## 输出格式

大多数命令都支持 `terminal`、`text`、`json`、`markdown` 等输出格式，具体取决于工具本身。
部分分析命令还支持 `--lang`，默认值是 `en-US`，用于本地化吐槽文本。

## 备注

- `analyze` 支持 `--harsh`、`--educational`、`--hall-of-shame`、`--suggestions` 和与 LLM 相关的参数
- `scan` 支持 `--lang`、`--format`、`--save` 和 `--project-config`
- `badge` 可以直接给分数，也可以根据目标路径自动计算
- `persona` 支持 `linux-kernel`、`silicon-valley`、`japanese-enterprise`、`rust-fanatic` 等预设人格
