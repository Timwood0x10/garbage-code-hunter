# Ruby 准确度报告

> 生成：2026-05-15 | 测试项目：1 | 分析器：garbage-code-hunter

## 测试项目

| 项目 | 文件数 | 行数 | 问题数 | 分数 | 密度 |
|---|---:|---:|---:|---:|---:|
| jekyll（静态站点生成器） | 166 | 23,923 | 776 | 27.7 | 32/k |

> 说明：Ruby 只测试了 1 个项目，统计显著性低于其他语言。

## 主要结论

- `bare-rescue` 在样本中未发现实例，规则逻辑需要更多项目验证。
- `global-variable` 对 `$LOAD_PATH` 等 Ruby gem 常见模式误报较多，需要扩充白名单。
- `println-debugging` 会把示例、benchmark、CLI 输出中的 `puts` 当作调试输出。
- `code-duplication` 在测试文件中噪音较大，Minitest 风格用例结构相似导致大量误报。
- `magic-number` 整体可用，大多数检测是真实魔法数字。

## 估计准确率

| 规则 | TP 率 | 主要误报来源 | 结论 |
|---|---:|---|---|
| `bare-rescue` | N/A | 样本不足 | 需要更多数据 |
| `global-variable` | ~0% | `$LOAD_PATH` gem 模式 | 需要白名单 |
| `println-debugging` | ~60% | 示例和 CLI 输出 | 可接受 |
| `code-duplication` | ~20-30% | 测试模式 | 高噪音 |
| `magic-number` | ~80% | 少量上下文数字 | 可接受 |
| `hungarian-notation` | ~50% | 样本不足 | 需要验证 |
| **整体** | **~35-45%** | - | 需改进 |

## 改进建议

- 扩充 Ruby 全局变量白名单，例如 `$LOAD_PATH`、`$LOADED_FEATURES`、`$PROGRAM_NAME`。
- 对测试文件的重复代码检测设置上限或降低权重。
- 对 `examples/`、`benchmarks/`、`scripts/` 中的输出语句降低敏感度。

