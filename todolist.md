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

- [ ] 1. 删除 `content.contains("#[cfg(test)]")`，改为 `false`（仅路径匹配）
- [ ] 2. 补全其他语言的路径模式（Go/Java/JS/TS/Ruby/Swift/Zig）
- [ ] 3. 删除或整合 `has_test_nodes()` 死代码
- [ ] 4. 更新相关测试用例
- [ ] 5. `make ci` 验证
