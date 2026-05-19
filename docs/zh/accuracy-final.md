# 最终准确度报告：真实 TP/FP 数据与源码证据

> 生成：2026-05-15 | 项目数：5 | 方法：JSON 输出 + 精确文件行号源码验证

## 测试项目

| 语言 | 项目 | 范围 | 问题数 |
|---|---|---|---:|
| Go | Go 标准库 `net/http` | 生产级 HTTP 库 | 5,179 |
| Rust | Wasmtime | 生产级 WASM 运行时 | 3,918 |
| TypeScript | tRPC | 生产级 API 框架 | 1,799 |
| Ruby | Rails ActiveRecord | 生产级 ORM | 1,792 |
| Python | zkp | 密码学研究代码 | 21,127 |

## 关键工具问题

### `line:1` 行号错误

部分规则只能判断“文件里存在问题”，但没有定位到具体行，最终默认报告 `line: 1, column: 1`。

| 语言 | `line:1` 问题数 | 影响规则 |
|---|---:|---|
| Go | 77 | `panic-abuse`、`file-too-long`、`todo-comment`、`goroutine-abuse` |
| Rust | 301 | `unwrap-abuse`、`box-abuse`、`generic-abuse`、`lifetime-abuse` 等 |
| Python | 90 | 类似模式 |
| Ruby | 14 | 类似模式 |
| **合计** | **484** | - |

影响：12,006 个问题中约 4.0% 行号错误；Rust 中约 7.7% 受影响。

### `box-abuse` 虚假检测

Rust `box-abuse` 报告 72 个问题，其中多数文件实际没有 `Box::` 使用。
结论是该规则存在虚假检测，需要优先修复。

### 依赖目录未默认排除

Python 项目扫描 `.venv/` 时产生大量第三方库噪音。
应默认排除 `.venv/`、`node_modules/`、`vendor/`、`.build/` 等常见依赖目录。

## 总体结论

- 对真实复杂结构的检测相对可靠，例如 `deep-nesting`。
- 对上下文敏感规则误报较多，例如 `magic-number`、`single-letter-variable`、`println-debugging`。
- 对文档注释、测试代码、生成代码、依赖目录需要更强的上下文过滤。
- 行号准确性是影响可用性的关键问题，应优先修复。

## 优先级建议

1. 修复所有默认 `line:1` 的规则定位。
2. 修复或禁用 `box-abuse` 虚假检测。
3. 默认排除依赖目录和生成目录。
4. 对测试文件降低重复检测和魔法数字权重。
5. 让语言适配器识别各语言文档注释和惯用模式。

