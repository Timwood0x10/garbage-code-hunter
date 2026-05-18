# Garbage Code Hunter — 最终测试报告

## 1. 魔法数字允许列表（按语言默认值 + 配置集成）

### 问题
`magic-number` 规则原先使用硬编码允许列表（`0, 1, -1, 2, 100, 0.0, 1.0, 10, 60, 24`），所有语言共享。领域相关的常见数字（如 JS/TS 的 HTTP 状态码 200/404、C/Rust 的缓冲区大小 1024/4096、Python 的字节范围 255）被错误标记。

### 解决方案
- **按语言默认值**：在 `check_inner()` 中添加 — JS/TS（HTTP 状态码）、Python（字节范围）、C/C++/Go/Rust（缓冲区大小、2 的幂）
- **配置集成**：`check_with_context()` 从 `.garbage-code-hunter.toml` 读取 `MagicNumberRuleConfig.allowed_numbers` + `.ui_layout_numbers`，与内置默认值合并
- **Serde 修复**：在 `RulesConfig.magic_number` 字段上添加 `#[serde(rename = "magic_number")]`，解决 `#[serde(rename_all = "kebab-case")]` 引起的 kebab-case/snake_case 冲突

### 测试结果

| 测试 | 修改前 | 修改后 |
|------|--------|--------|
| Python: `x = 1024 + 255` | 2 个违规 | 1 个违规（仅 1024，255 已允许） |
| Rust: `let x = 1024 + 4096;` | 2 个违规 | 0 个违规 |
| JS: `const x = 200 + 404;` | 2 个违规 | 0 个违规 |
| C: `int x = 1024 + 65535;` | 2 个违规 | 0 个违规 |
| Config 白名单 (JS): `3000 + 3600 + 21`（TOML 配置） | 3 个违规 | 0 个违规 |
| Config 白名单 (JS) — 无配置 | 3 个违规 | 3 个违规（不变） |

**单元测试**：779/779 通过（新增 `test_magic_number_config_parse`）

---

## 2. 信号层 Line=0 分离

### 问题
信号发现（如 "命名混乱"、"嵌套地狱"）与逐行规则发现混在同一个 `issues[]` 数组中，`line: 0` 产生无意义的位置信息。

### 解决方案
- **JSON 输出**（`helpers.rs` 中的 `output_json`）：信号发现过滤到独立的 `signals[]` 数组；仅 `line > 0` 的发现保留在 `issues[]` 中。
- 新增 `AnalyzeJsonSignal` 结构体，summary 中添加 `signal_count` 字段。

### 测试结果

| 指标 | 修改前 | 修改后 |
|------|--------|--------|
| `issues[]` 含 `line=0` | 是（混合） | 0（干净） |
| `signals[]` | 无 | 29（system_alert 项目） |
| `summary.signal_count` | 无 | 29 |
| `summary.issue_count` | 包含信号 | 112（仅问题） |

JSON 示例结构：
```json
{
  "schema_version": "1.0",
  "issues": [ /* 仅逐行规则发现 */ ],
  "signals": [ { "signal": "Naming Chaos", "file_path": "...", "severity": "Mild", "violation_count": 11 } ],
  "summary": { "issue_count": 112, "signal_count": 29, "total_score": 46.72 }
}
```

---

## 3. 完整流水线验证

| 项目 | 问题数 | 信号数 | 分数 | 状态 |
|------|--------|--------|------|------|
| `system_alert` (Rust) | 112 | 29 | 46.72 | ✅ 干净分离 |
| 单 JS 文件（无配置） | 4 | 1 | — | ✅ 按语言默认值 |
| 单 JS 文件（有配置） | 1 | 1 | — | ✅ 配置白名单生效 |

---

## 4. 潜在改进（未包含）

- 终端输出：信号发现（`line=0`）仍会出现在朋友反馈的"快速胜利"中。后续可以在此处也过滤。
- `MagicNumberRule` 仅在 `register_rust_rules()` 中注册，尽管支持所有语言——应移至共享注册表中。
