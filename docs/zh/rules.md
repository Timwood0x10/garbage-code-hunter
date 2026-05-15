# 分析规则参考手册

Garbage Code Hunter 使用 tree-sitter AST 解析，在 11 种语言中检测代码品味问题。
这不是 bug 检测器 —— 它找的是命名之罪、魔法数字、深层嵌套、上帝函数、print 调试、注释掉的代码、TODO 堆成山、复制粘贴等代码坏味道。

---

## 通用规则（所有语言）

这些规则通过语言特定的 tree-sitter 查询在每种支持的语言上运行。

### 命名类

| 规则 | 检测内容 | 严重度 |
|------|---------|--------|
| `terrible-naming` | 变量名为 `data`、`info`、`temp`、`tmp`、`val`、`value`、`thing`、`stuff`、`obj`、`object`、`manager`、`handler`、`helper`、`util`、`utils` | Spicy |
| `single-letter-variable` | 单字符变量名（通过 AST 分析排除循环计数器） | Mild |
| `meaningless-naming` | 占位名：`foo`、`bar`、`baz`、`qux`、`aaa`、`bbb`、`xxx` 等 | Mild/Spicy |
| `hungarian-notation` | 类型前缀（`strName`、`intCount`）和作用域前缀（`g_`、`m_`、`s_`、`p_`） | Mild |
| `abbreviation-abuse` | 缩写如 `mgr`、`ctrl`、`hdlr`、`usr`、`pwd`、`btn`、`lbl`、`tbl`、`col`、`cnt` | Mild |

### 复杂度类

| 规则 | 检测内容 | 阈值 | 严重度 |
|------|---------|------|--------|
| `deep-nesting` | 嵌套深度 > 5 层 | 5 层 | Mild → Nuclear |
| `long-function` | 函数超过 80 行（测试文件 150 行） | 80 行 | Mild → Nuclear |
| `god-function` | 行数 + 参数数 + 控制流的综合评分 | 15 分 | Mild/Spicy |

### 代码坏味道

| 规则 | 检测内容 | 严重度 |
|------|---------|--------|
| `magic-number` | 不在常见集合中的整数/浮点字面量（0、1、-1、2、100、10、60、24） | Mild |
| `println-debugging` | `println`、`print`、`console.log`、`fmt.Println`、`puts` 等 | Spicy |
| `commented-code` | 连续 3 行以上的注释掉的代码块 | Mild/Spicy |
| `todo-comment` | TODO/FIXME/BUG/HACK 注释，`todo!()`/`unimplemented!()` 宏 | Mild/Spicy |

### 重复类

| 规则 | 检测内容 | 严重度 |
|------|---------|--------|
| `code-duplication` | 同一文件内重复的 5 行代码块 | Mild |
| `cross-file-duplication` | 跨文件的完全相同函数 | Mild → Nuclear |
| `cross-file-near-duplicate` | 跨文件 token 相似度 >80% 的函数 | Mild |

### 结构类

| 规则 | 检测内容 | 严重度 |
|------|---------|--------|
| `file-too-long` | 文件超过 1000 行（测试文件 2000 行） | Mild → Nuclear |

---

## Rust 专属规则

| 规则 | 检测内容 | 阈值 |
|------|---------|------|
| `unwrap-abuse` | `.unwrap()` 调用 | 0（任何调用都触发） |
| `unnecessary-clone` | `.clone()` 调用 | 24 |
| `panic-abuse` | `panic!()` 宏 | 2 |
| `string-abuse` | `.to_string()` 调用 | 20 |
| `vec-abuse` | `vec!` 宏调用 | 15 |
| `async-abuse` | `async` 块 | 10 |
| `macro-abuse` | 宏调用 | 20 |
| `lifetime-abuse` | 生命周期标注 | 20 |
| `trait-complexity` | trait 体中的方法数 | 10 |
| `generic-abuse` | 类型参数数量 | 5 |
| `pattern-matching-abuse` | 元组模式 | 15 |
| `box-abuse` | `Box::new` 调用 | 8 |
| `reference-abuse` | 引用类型 | 50 |
| `slice-abuse` | 切片类型 | 29 |
| `module-complexity` | 嵌套 `mod` 项 | 0 |
| `complex-closure` | 嵌套闭包（深度 > 2）或参数 > 5 | — |
| `dead-code` | return/break/continue/panic 之后的不可达代码 | — |
| `duplicate-imports` | 重复的 `use` 语句 | — |

---

## Go 专属规则

| 规则 | 检测内容 | 阈值 | 严重度 |
|------|---------|------|--------|
| `panic-abuse` | `panic()` 调用 | 0 | Mild → Nuclear |
| `goroutine-abuse` | `go` 语句生成 | 8 | Spicy |
| `defer-in-loop` | `for` 循环体内的 `defer` | — | Spicy |

### 真实项目示例（interchange 项目）

```
📁 main.go
  ⚠️ cross file near duplicate: 1

📁 params.pb.go
  🔄 Code duplication issues: 2
  ⚠️ magic number: 20
  🏷️ Variable naming issues: 8 (n, n, i, l, b, ...)

📁 errors.go
  ⚠️ magic number: 4
```

---

## C/C++ 专属规则

| 规则 | 检测内容 | 阈值 |
|------|---------|------|
| `goto-abuse` | `goto` 语句 | 0 |
| `new-expression` | `new` 表达式（仅 C++） | 0 |
| `malloc-leak` | 堆分配（malloc、curlx_malloc、zmalloc 等） | 0 |

---

## 语言覆盖矩阵

| 规则 | Rust | Go | Python | JS/TS | Java | C/C++ | Ruby | Swift | Zig |
|------|------|-----|--------|-------|------|-------|------|-------|-----|
| terrible-naming | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| single-letter | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| deep-nesting | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| long-function | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| god-function | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| magic-number | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| println-debugging | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ | ✅ | — |
| commented-code | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| todo-comment | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| file-too-long | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| duplication | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 已知限制

1. **生成文件**：`.pb.go`、`.pulsar.go`、`_grpc.pb.go`、`*.gen.ts` 等尚未自动排除。使用 `--exclude` 标志或 `.garbage-code-hunter.toml` 手动过滤。

2. **跨文件重复**：近重复检测在大型代码库上可能产生较高的 issue 数量。正在改进中。

3. **评分**：非 Rust 项目可能显示偏高的分数，因为部分评分类别是 Rust 专属的。
