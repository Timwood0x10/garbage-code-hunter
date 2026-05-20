# Todolist — 语言适配器置信度提升

## 目标
将各语言检测置信度从当前水平提升到可用水平，重点修复返回 0 的硬编码方法。

## 优先级排序

### P0 — C/C++ panic 检测（影响最大）
- [x] `c.rs`: tree-sitter pattern 检测 `exit/abort/assert/_Exit/quick_exit/longjmp`
- [x] `cpp.rs`: tree-sitter pattern 检测 `exit/abort/assert/terminate/_Exit/quick_exit` + `throw_statement`
- [x] `count_panic_from_batch` 实现，测试更新

### P1 — C++ debug 检测
- [x] `cpp.rs`: `count_debug_from_batch` 扩展，文本扫描 `cout/cerr/clog`
- [x] 测试: `test_cpp_debug_cout`

### P2 — JS 语言特性检测
- [x] `count_js_issues` trait 方法 + `AdapterCounts.js_issues` + `StyleIr.js_issue_count`
- [x] `js.rs`: 文本扫描 `eval()/with/alert()/var`
- [x] 4 个测试用例

### P3 — Swift 语言特性检测
- [x] `count_swift_issues` trait 方法 + `AdapterCounts.swift_issues` + `StyleIr.swift_issue_count`
- [x] `swift.rs`: panic pattern 扩展 `assert/assertionFailure/precondition`，debug 扩展 `NSLog`
- [x] `count_swift_issues`: 文本扫描 `try!/as!`
- [x] 4 个测试用例

### P4 — Zig 语言特性检测（Beta）
- [x] `zig.rs`: panic 扩展 — 文本扫描 `unreachable`
- [x] `zig.rs`: debug 扩展 — tree-sitter `@compileLog` + `std.debug.warn`
- [x] 2 个测试用例
- [x] 文档标注 Zig 为 Beta（README + README_zh + accuracy docs）

### P5 — TS 语言特性补充
- [x] `ts.rs`: 扩展 `count_ts_issues`
  - 补充: `@ts-ignore`, `@ts-expect-error`, `require()` vs import
  - 注: non-null assertion (`!`) 和 `as` 类型转换误报率高，暂不实现

### P6 — dead_code 通用化
- [x] 为 Python/JS/TS/Java/Ruby/C/C++/Swift/Zig 实现 `count_dead_code`
  - 共享 `count_dead_code_with` helper
  - Python: `return`/`raise`/`sys.exit()` 后的代码
  - JS/TS: `return`/`throw` 后的代码
  - Java: `return`/`throw` 后的代码
  - Ruby: `return`/`raise`/`exit` 后的代码
  - C/C++: `return`/`throw`/`exit()` 后的代码
  - Swift/Zig: `return`/`throw`/`@panic()` 后的代码

### P7 — duplicate_imports 通用化
- [x] 为所有语言实现 `count_duplicate_imports`
  - 共享 `count_duplicate_imports_with` helper
  - Rust: `use `
  - Go: `import ` / `import (`
  - Python: `import ` / `from `
  - JS/TS/Java/Swift: `import `
  - Ruby: `require ` / `require_relative `
  - C/C++: `#include`
  - Zig: `@import(`

### P8 — Java debug 检测补充
- [x] `java.rs`: 扩展 debug 检测
  - 补充: `logger.info/debug/warn/error`, `.fine()/.finest()/.severe()` 日志框架模式
  - 注: `System.err.println` 已被现有 tree-sitter pattern 覆盖

### P9 — 其他小修补
- [ ] `go.rs`: 检测 `unsafe` 包导入和 unsafe pointer 操作
- [x] `python.rs`: magic number 检测补充 float literal
- [x] `ruby.rs`: debug 检测补充 `warn`, `STDERR.puts`
- [ ] `c.rs`/`cpp.rs`: `count_c_issues` 中的文本扫描改用 tree-sitter（sizeof/malloc-null-check）

## 已知限制
- Zig 语法多变，tree-sitter grammar 可能滞后，标注为 Beta
- Swift/Zig 的 tree-sitter grammar 质量不如 Rust/Go/Python
- 文本扫描（.contains()）在字符串/注释中可能误报

## 验证
每个 P0-P4 完成后:
1. `make ci` 通过
2. 用对应语言的真实项目运行 `analyze` 验证检测结果
3. 更新对应语言的 accuracy doc
