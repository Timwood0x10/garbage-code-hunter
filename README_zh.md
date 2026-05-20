# Garbage Code Hunter

[![CI/CD](https://github.com/TimWood0x10/garbage-code-hunter/actions/workflows/ci.yml/badge.svg)](https://github.com/TimWood0x10/garbage-code-hunter/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/garbage-code-hunter.svg)](https://crates.io/crates/garbage-code-hunter)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-yellow.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust Version](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [中文](README_zh.md)

一个用毒舌风格吐槽代码质量、commit、依赖、PR 标题和技术债的 CLI 工具集。

最关键的改进是 **StyleIR**：一个语言中立的风格中间表示层。它先从解析后的源码里提取客观风格事实，再让检测器、评分、报告和 JSON 输出共享同一层证据。

> 灵感来源：[fuck-u-code](https://github.com/Done-0/fuck-u-code.git)

## 注意

Garbage Code Hunter 是娱乐向的代码品味工具。它检查可读性、风格、可维护性信号和一些有趣的项目健康指标。它不是 bug 检测器、安全扫描器，也不能替代 Clippy、ESLint、Pylint 等正式 linter 或静态分析工具。

## 安装

```bash
cargo install garbage-code-hunter
```

## 快速开始

```bash
# 分析当前目录
garbage-code-hunter analyze

# 使用中文输出分析项目
garbage-code-hunter analyze ./my-project --lang zh-CN

# 运行完整工具集
garbage-code-hunter scan ./my-project

# 输出 JSON
garbage-code-hunter analyze -f json
```

## 功能清单

### StyleIR 核心

- 把解析后的源码转换成稳定的风格事实，避免每条规则都直接绑定不同语言 AST
- 统计函数数量、上帝函数、panic 风险调用、命名违规、嵌套、调试输出、魔法数字、TODO、重复 import、unsafe block 和语言特定问题计数
- 输出稳定的 JSON summary，方便报告、自动化和后续规则迁移
- 通过语言适配器兼容 Rust、Go、Python、Java、Ruby、C/C++、TypeScript、Swift、Zig 和 JavaScript，同时让跨语言评分更一致

### 工具

| 功能 | 命令 | 别名 | 说明 |
|---|---|---|---|
| Code Hunter | `analyze` | - | 核心源码分析：命名、复杂度、重复、调试残留、风格气味 |
| Commit Roaster | `commit-roaster` | `cr` | 吐槽 git 历史里的弱 commit 信息 |
| Deps Shamer | `deps-shamer` | `ds` | 检查常见生态的依赖管理习惯 |
| PR Title Hunter | `pr-title-hunter` | `pr` | 吐槽本地或 GitHub PR 标题 |
| Full Scan | `scan` | - | 运行完整工具集并输出综合分数 |
| Badge | `badge` | - | 生成 SVG 质量徽章 |
| Trend | `trend` | - | 查看已保存的质量分数趋势 |
| Last Words | `last-words` | `lw` | 查找陈旧的 TODO/FIXME/HACK 注释 |
| Debt Invoice | `debt-invoice` | `debt` | 估算技术债成本 |
| Personality | `personality` | - | 根据代码模式推断开发者画像 |
| Decay | `decay` | - | 分析 git 历史中的质量衰减 |
| Autopsy | `autopsy` | - | 输出根因风格的代码尸检报告 |
| Radar | `radar` | - | 生成代码气味雷达图或 SVG |
| CI Bot | `ci-bot` | - | 生成 CI 风格审查评论 |
| Persona | `persona` | - | 使用指定人格模式吐槽代码 |
| Danger Zone | `danger-zone` | `dz` | 找出仓库中最危险的文件 |
| Team Roast | `team-roast` | - | 按贡献者汇总代码质量和技术债 |

## 支持语言

Rust、Go、Python、JavaScript、TypeScript、Java、C、C++、Ruby、Swift、Zig。

| 语言 | 状态 | TP 率 | 测试项目 | 主要检测能力 |
|------|------|-------|----------|-------------|
| Rust | 稳定 | ~90% | Finance, ReChat-server | unwrap/expect, panic, assert, debug, unsafe, naming, nesting, magic, dead_code, duplicate_import, `#[cfg(test)]` 感知 |
| Go | 稳定 | ~85% | interchange, gaia, loan, gosec | panic, debug, naming, nesting, goroutine, defer, conventions, unsafe, dead_code, duplicate_import |
| Python | 稳定 | ~80% | ZK-bulletproofs | except, print, naming, nesting, magic (int+float), 通配符导入, 布尔比较, dead_code, duplicate_import |
| Ruby | 稳定 | ~85% | jekyll, Metric | raise, puts/p/warn, naming, nesting, magic, 全局变量, bare-rescue, dead_code, duplicate_import |
| Java | 稳定 | ~80% | TestJava.java | throw, println/printStackTrace, naming, nesting, magic, 空catch, 日志框架, dead_code, duplicate_import |
| TypeScript | Beta | ~80% | zod, hono, trpc | throw, console/debugger, naming, nesting, magic, any/enum/alias, @ts-ignore, require, dead_code, duplicate_import |
| JavaScript | Beta | ~80% | 自测 | throw, console/debugger, naming, nesting, magic, eval/with/alert/var, dead_code, duplicate_import |
| C | Beta | ~85% | stone-prover | exit/abort/assert, printf, naming, nesting, magic, goto, sizeof, malloc, dead_code, duplicate_import |
| C++ | Beta | ~85% | stone-prover | exit/abort/terminate/throw, cout/cerr, naming, nesting, magic, goto/new, sizeof, malloc, dead_code, duplicate_import |
| Swift | Beta | ~80% | Alamofire, SnapKit, vapor | fatalError/assert, print/NSLog, naming, nesting, magic, try!/as!, dead_code, duplicate_import |
| Zig | Beta | ~90% | ziglings | @panic, @compileLog/warn, naming, nesting, magic, unreachable, dead_code, duplicate_import |

## 性能基准

Apple Silicon (M 系列) 单文件分析性能：

| 基准测试 | 耗时 |
|----------|------|
| `create_analyzer` | 539 µs |
| `analyze_file/clean_rust` | 1.30 ms |
| `analyze_file/single_large_file` (100 函数) | 61.5 ms |
| `analyze_path/mixed_4_languages` | 102 ms |
| `analyze_path/10_garbage_files` | 7.50 ms |
| `analyze_path/50_garbage_files` | 37.4 ms |
| `scalability/20_files` | 15.2 ms |
| `scalability/50_files` | 37.4 ms |

运行 `cargo bench` 可复现。日志保存在 `./benchs/` 目录。

## 常用命令

```bash
# 排除噪音文件
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"

# 保存扫描历史并查看趋势
garbage-code-hunter scan --save
garbage-code-hunter trend

# 生成图片资产
garbage-code-hunter badge --output badge.svg
garbage-code-hunter radar --output radar.svg

# 分析 GitHub PR 标题
garbage-code-hunter pr --repo owner/repo --state open --token $GITHUB_TOKEN
```

## 配置

在项目根目录创建 `.garbage-code-hunter.toml` 即可自定义分析规则。工具会自动从当前目录向上查找此文件。

```bash
# 自动发现（无需指定路径）
garbage-code-hunter analyze

# 手动指定路径
garbage-code-hunter analyze --project-config .garbage-code-hunter.toml
```

**配置文件名：** `.garbage-code-hunter.toml`（唯一支持的文件名，不支持 `.garbage-hunter.toml` 等其他名称）

**可配置内容：**

```toml
[whitelists]
magic-numbers = [800, 1000]
variable-names = ["ctx", "db"]
exclude-patterns = ["vendor/*", "third_party/*"]

[rules.magic-number]
enabled = true
allowed-numbers = [3000, 86400]

[rules.unwrap]
threshold = 3

[signals]
panic-addiction = true
naming-chaos = false
```

完整配置说明：[docs/config-reference.md](docs/config-reference.md)

## 文档

- 中文文档：[docs/zh/index.md](docs/zh/index.md)
- 英文文档：[docs/en/index.md](docs/en/index.md)
- StyleIR 实现：[src/style_ir/mod.rs](src/style_ir/mod.rs)
- 工具指南：[docs/zh/tools.md](docs/zh/tools.md)
- 配置说明：[docs/zh/configuration.md](docs/zh/configuration.md)
- 规则参考：[docs/zh/rules.md](docs/zh/rules.md)

## VSCode 插件

VSCode 插件位于 [vscode-extension](vscode-extension/README.md)。

## License

Apache-2.0
