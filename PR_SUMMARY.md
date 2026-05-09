# PR Summary / PR 摘要

---

## 🇺🇸 English

### Title
`v0.2.0: Cross-file analysis, context-aware detection, VSCode extension`

### Summary
This PR introduces major new features and code quality improvements for Garbage Code Hunter v0.2.0.

### What's New

#### 🔥 Cross-File Analysis
- **Function fingerprinting**: Detect code clones across multiple files using AST-based normalization
- **Exact duplicate detection**: Hash-based matching for copy-paste detection
- **Near duplicate detection**: Jaccard similarity for refactored clones (configurable threshold)
- **Memory management**: Configurable memory limits with automatic eviction

#### 🎯 Context-Aware Analysis
- **File type detection**: Automatically identify test files, examples, benchmarks, documentation
- **Smart filtering**: Reduce false positives in example/test code with weight multipliers
- **Project configuration**: Support for `.garbage-code-hunter.toml` configuration files
- **Per-directory overrides**: Different rules for different parts of your project

#### 🤖 LLM Integration
- **Ollama support**: Local LLM for creative roasts
- **OpenAI-compatible APIs**: Support for custom endpoints
- **Educational mode**: Get suggestions on how to fix issues

#### 🔌 VSCode Extension
- **Real-time analysis**: Analyze on save with smart debouncing (800ms)
- **ErrorLens-style messages**: Inline diagnostics with humorous roasts
- **Auto language detection**: Detect Chinese/English from file comments
- **Command palette integration**: 5 commands + context menus

### Code Quality Fixes (This Session)

| Issue | File | Fix |
|-------|------|-----|
| Regex recompilation | `duplication.rs` | Use `OnceLock` for static regex |
| Clone threshold bug | `rust_specific.rs` | `== 15` → `>= 15 && issues.is_empty()` |
| Operator precedence | `code_smells.rs` | Split into clear variables |
| Unused variables | `display.rs` | Remove `_score_color`, `_nuclear_count` |
| Redundant logic | `display.rs` | Simplify `get_category_roast` |
| Dead code | `hall_of_shame.rs` | Remove `_worst_offenses`, `_shame_categories`, `most_common_patterns` |

### Breaking Changes
None - fully backward compatible.

### Testing
- All existing tests pass
- New cross-file analysis tests added
- VSCode extension tested locally

### Checklist
- [x] Code follows project style guidelines
- [x] All tests pass
- [x] Documentation updated (README, README_zh)
- [x] Release workflow added (release.yml)
- [x] Release notes prepared (RELEASE_NOTE.md)

---

## 🇨🇳 中文

### 标题
`v0.2.0: 跨文件分析、上下文感知检测、VSCode 扩展`

### 摘要
本 PR 为 Garbage Code Hunter v0.2.0 引入了主要新功能和代码质量改进。

### 新功能

#### 🔥 跨文件分析
- **函数指纹**：使用 AST 归一化检测跨文件代码克隆
- **精确重复检测**：基于哈希的复制粘贴检测
- **近似重复检测**：Jaccard 相似度检测重构后的克隆（可配置阈值）
- **内存管理**：可配置内存限制，自动淘汰旧数据

#### 🎯 上下文感知分析
- **文件类型检测**：自动识别测试文件、示例、基准测试、文档
- **智能过滤**：通过权重乘数减少示例/测试代码中的误报
- **项目配置**：支持 `.garbage-code-hunter.toml` 配置文件
- **目录级覆盖**：为项目不同部分设置不同规则

#### 🤖 LLM 集成
- **Ollama 支持**：本地 LLM 生成创意吐槽
- **OpenAI 兼容 API**：支持自定义端点
- **教育模式**：获取修复建议

#### 🔌 VSCode 扩展
- **实时分析**：保存时智能防抖分析（800ms）
- **ErrorLens 风格消息**：行内诊断 + 幽默吐槽
- **自动语言检测**：从文件注释检测中文/英文
- **命令面板集成**：5 个命令 + 右键菜单

### 代码质量修复（本次会话）

| 问题 | 文件 | 修复 |
|------|------|------|
| 正则重复编译 | `duplication.rs` | 使用 `OnceLock` 静态正则 |
| 克隆阈值 Bug | `rust_specific.rs` | `== 15` → `>= 15 && issues.is_empty()` |
| 运算符优先级 | `code_smells.rs` | 拆分为清晰变量 |
| 未使用变量 | `display.rs` | 删除 `_score_color`, `_nuclear_count` |
| 冗余逻辑 | `display.rs` | 简化 `get_category_roast` |
| 死代码 | `hall_of_shame.rs` | 删除 `_worst_offenses`, `_shame_categories`, `most_common_patterns` |

### 破坏性变更
无 - 完全向后兼容。

### 测试
- 所有现有测试通过
- 新增跨文件分析测试
- VSCode 扩展本地测试

### 检查清单
- [x] 代码遵循项目风格规范
- [x] 所有测试通过
- [x] 文档已更新（README, README_zh）
- [x] 发布工作流已添加（release.yml）
- [x] 发布说明已准备（RELEASE_NOTE.md）

---

## 📊 Stats / 统计

```
67 files changed
+18,548 lines
-5,566 lines
```

## 🔗 Related / 相关

- Inspired by: https://github.com/Done-0/fuck-u-code.git
