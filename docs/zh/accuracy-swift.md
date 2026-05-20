# Swift 准确度报告

> 生成：2026-05-15 | 测试项目：3 | 分析器：garbage-code-hunter

## 测试范围

Swift 样本覆盖应用框架、网络库和服务端 Swift 项目，用于验证 Swift 注释、OptionSet、测试代码和函数长度相关规则。

## 关键问题

- 非 Swift 文件曾被纳入扫描，导致 `docs/`、`vendor/`、`.build/` 等目录产生大量噪音。
- `commented-code` 对 Swift 的 `///` 文档注释误报严重。
- `magic-number` 对 `OptionSet` 位掩码、HTTP 状态码和测试值过于敏感。
- `god-function` 和 `long-function` 会把 `///` 文档注释计入函数长度。
- `code-duplication` 在测试结构和 API overload 上误报明显。

## 估计准确率

| 规则 | TP 率 | 主要误报来源 | 结论 |
|---|---:|---|---|
| `commented-code` | ~0-10% | `///` 文档注释、`MARK` 分隔 | 需要修复 |
| `magic-number` | ~60-70% | OptionSet 位掩码、测试值 | 需要上下文 |
| `code-duplication` | ~30% | 测试模式、overload | 高噪音 |
| `god-function` | ~50% | 文档注释计入长度 | 需要修复 |
| `long-function` | ~50% | 文档注释计入长度 | 需要修复 |
| `single-letter-var` | ~60% | 循环变量 | 可接受 |
| `file-too-long` | ~70% | - | 可接受 |
| **整体** | **~30-40%** | - | 需改进 |

## 改进建议

- Swift 项目只扫描 `.swift` 文件，并默认跳过 `docs/`、`vendor/`、`.build/`、`Pods/`。
- `commented-code` 应直接跳过 `///` 文档注释。
- 计算函数长度时排除 `///` 和 `/** */` 文档块。
- 对 `UInt(1) << N` 这类 `OptionSet` 位掩码豁免魔法数字检测。

