# Todolist — is_test_file 漏报修复

## 问题确认

### Rust（已确认，严重）
- `is_test_file()` 的 `content.contains("#[cfg(test)]")` 导致 **71/89 个 .rs 文件**被误判为测试文件
- 6 个 detector 继承默认 `skips_test_files() = true`，加上 `skip_tests_config = true`，三重 AND → 全部静默跳过
- 受影响 detector: PanicAddiction, NamingChaos, NestedHell, HotfixCulture, OverEngineering, CodeSmells

### 其他语言（验证结果）
- **不存在内容检测误伤** — `is_test_file()` 只有一个内容检测：`#[cfg(test)]`，仅影响 Rust
- 其他语言仅靠路径匹配，无内容误伤
- 但路径匹配**有遗漏**（假阴性，测试文件没被识别出来）：

| 语言 | 缺失的测试文件模式 | 说明 |
|------|-------------------|------|
| Go | `_test.go` 后缀 | Go 标准测试文件命名 |
| Java | `*Test.java`, `*Tests.java` | JUnit 惯例 |
| JS | `.test.js`, `.spec.js` | Jest/Mocha 惯例 |
| TS | `.test.ts`, `.spec.ts` | 同上 |
| Python | 路径 OK（`test_` 前缀已覆盖） | — |
| Ruby | `_test.rb`, `_spec.rb` | RSpec/Minitest 惯例 |
| Swift | `*Tests.swift`, `*Test.swift` | XCTest 惯例 |
| Zig | `*_test.zig` | Zig 惯例 |

### 死代码
- `has_test_nodes()` trait 方法只在 Rust adapter 实现，且**从未被调用**

## 修复计划

- [x] 1. 删除 `content.contains("#[cfg(test)]")`，改为纯路径匹配
- [x] 2. 补全其他语言的路径模式（Go/Java/JS/TS/Ruby/Swift/Zig）
- [x] 3. 删除 `has_test_nodes()` 死代码 + `at_attr` pattern
- [x] 4. 更新相关测试用例
- [x] 5. `make ci` 验证

## 待确认

- [x] 其他子命令 — `scan` 和 `analyze` 共用 `CodeAnalyzer`，已一并修复
- [ ] `skip_tests_config` 默认 true 是否合理？
- [ ] `file_context.rs:149` 有独立的 `is_test_file()`，`path_str.contains("test_")` 会误匹配 `contest_handler.rs` 等（影响规则权重，非 detector 跳过）
- [x] `at_attr` 删除无影响 — 无任何 `_from_batch` 方法引用它

## 第二轮修复

- [x] `skip_tests` 默认改为 `false` — 测试文件不再被跳过
- [x] `file_context.rs` `is_test_file()` 修复 `test_` 误匹配（`contest_handler.rs` 等）
- [x] `FileContext::Test` 已有 0.2 权重 + `should_skip_rule` 跳过 unwrap/panic/todo/naming — 测试风格照查，debug 宽松

## 第三轮修复

- [x] Rust adapter: `unwrap` 才算违规，`expect` 是推荐替换方式，不再计数
- [x] tree-sitter pattern 加 `#eq? @pc_method "unwrap"` 精确匹配
- [x] 更新 4 个测试（unit + integration）适配新规则
- [x] panic 计数 1372 → 1310（去掉 62 个 expect）

## 第四轮修复

- [x] `#[cfg(test)]` 模块内排除 unwrap/assert/panic — 生产代码误报消除
- [x] `cfg_test_ranges()` 函数：用 `find()` + char 迭代，支持多字节 UTF-8
- [x] findings pipeline 加 0.2x 权重 — test file findings 降权
- [x] panic 计数 1301 → 14（仅统计生产代码中的真实 unwrap）
- [x] 评分 60.6 → 58.0，恐慌成瘾 100% → 39%
