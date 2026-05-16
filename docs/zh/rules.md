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
| `rust-doc-example` | 文档注释（`///`）没有示例代码块（&#96 &#96 &#96） | — |
| `rust-derive-order` | `#[derive(..)]` 不按标准顺序（Debug, Clone, Copy, PartialEq...） | — |
| `rust-error-display` | 实现了 Debug 但没有实现 Display 的类型 | — |
| `rust-must-use` | 返回 Result/Option 的 `pub fn` 缺少 `#[must_use]` | — |
| `too-many-params` | 函数参数超过 6 个 | — |

---

## Go 专属规则

| 规则 | 检测内容 | 阈值 | 严重度 |
|------|---------|------|--------|
| `panic-abuse` | `panic()` 调用 | 0 | Mild → Nuclear |
| `goroutine-abuse` | `go` 语句生成 | 8 | Spicy |
| `defer-in-loop` | `for` 循环体内的 `defer` | — | Spicy |
| `go-receiver-name` | 方法接收者超过 2 个字符 | — | Mild |
| `go-error-string` | 错误字符串首字母大写 | — | Mild |
| `go-context-first` | `context.Context` 不是函数的第一个参数 | — | Mild |
| `go-else-return` | `if-else` 中 `if` 块有 return（应使用 early return） | — | Mild |

---

## Python 专属规则

| 规则 | 检测内容 | 严重度 |
|------|---------|--------|
| `bare-except` | `except:` 未指定异常类型 | Spicy |
| `wildcard-import` | `from module import *`（排除已知白名单模块） | Mild |
| `python-naming` | 函数不是 snake_case / 类不是 PascalCase | Mild |
| `compared-to-bool` | `if x == True` 而非 `if x` | Mild |
| `not-is-none` | `x == None` 而非 `x is None` | Mild |
| `python-type-ignore` | `# type: ignore` 注释 | Mild |
| `python-fstring` | `.format()` 或 `%` 格式化（应使用 f-string） | Mild |
| `python-magic-method` | 非标准 `__dunder__` 方法定义 | Mild |

---

## Java 专属规则

| 规则 | 检测内容 | 严重度 |
|------|---------|--------|
| `empty-catch` | 空的 `catch (Exception e) {}` 块 | Spicy |
| `constant-name` | `static final` 字段不是 UPPER_SNAKE_CASE | Mild |
| `java-javadoc-missing` | 公共/受保护方法缺少 Javadoc 注释 | Mild |
| `java-try-resource` | `try-finally` 中的 `.close()`（应使用 try-with-resources） | Mild |
| `java-string-concat` | 循环内的字符串拼接（`+=`） | Mild |

---

## Ruby 专属规则

| 规则 | 检测内容 | 严重度 |
|------|---------|--------|
| `global-variable` | 非内置全局变量（`$xxx`） | Mild |
| `bare-rescue` | `rescue` 未指定异常类 | Mild |
| `frozen-string` | 缺少 `# frozen_string_literal: true` 魔法注释 | Mild |
| `negated-if` | `if !x` 而非 `unless x` | Mild |
| `ruby-predicate-method` | 谓词方法（`is_xxx`、`has_xxx`）不以 `?` 结尾 | Mild |

---

## C/C++ 专属规则

| 规则 | 检测内容 | 阈值 |
|------|---------|------|
| `c-goto-abuse` | `goto` 语句 | 0 |
| `c-new-expression` | `new` 表达式（仅 C++） | 0 |
| `c-malloc-leak` | 堆分配（malloc、curlx_malloc、zmalloc 等） | 0 |
| `c-malloc-check` | `malloc` 返回值未检查是否为 NULL | — |
| `c-sizeof-type` | `sizeof(类型名)` 而非 `sizeof(表达式)` | — |

---

## TypeScript 专属规则

| 规则 | 检测内容 | 严重度 |
|------|---------|--------|
| `any-type` | `any` 类型标注 / `as any` 强制转换 | Mild |
| `prefer-interface` | `type Foo = { ... }` 当可以使用 `interface` 时 | Mild |

---

## 语言专属豁免列表

以下惯用写法从通用规则中豁免，以减少误报：

### single-letter-variable 豁免

| 语言 | 豁免的标识符 |
|------|-------------|
| Go | `err`、`ok`、`ctx`、`mu`、`wg`、`ch`、`fn` |
| Python | `_`（丢弃变量） |
| C/C++ | `i`、`j`、`k`、`n`、`p`、`s` |

### abbreviation-abuse 豁免

| 语言 | 豁免的缩写 |
|------|-----------|
| Go | `ctx`、`req`、`resp`、`srv`、`cfg`、`buf`、`ch`、`wg`、`mu`、`fn`、`fmt`、`err`、`ok`、`http`、`json`、`tls`、`ssh` |
| Python | `cls`、`idx`、`fmt`、`msg`、`btn`、`img` |
| Java | `str`、`num`、`obj`、`arr`、`idx` |

### 其他豁免

| 规则 | 语言 | 豁免模式 |
|------|------|---------|
| `god-function` | Go | `func main()`、`func init()` |
| `any-type` | TypeScript | `*.d.ts` 文件 |
| `hungarian-notation` | 所有 | `c`、`t`、`ctx`、`req`、`res`、`err`、`db`、`kv`、`fs`、`io` |

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
| go-receiver-name | — | ✅ | — | — | — | — | — | — | — |
| go-error-string | — | ✅ | — | — | — | — | — | — | — |
| go-context-first | — | ✅ | — | — | — | — | — | — | — |
| go-else-return | — | ✅ | — | — | — | — | — | — | — |
| python-naming | — | — | ✅ | — | — | — | — | — | — |
| compared-to-bool | — | — | ✅ | — | — | — | — | — | — |
| not-is-none | — | — | ✅ | — | — | — | — | — | — |
| python-type-ignore | — | — | ✅ | — | — | — | — | — | — |
| python-fstring | — | — | ✅ | — | — | — | — | — | — |
| python-magic-method | — | — | ✅ | — | — | — | — | — | — |
| rust-doc-example | ✅ | — | — | — | — | — | — | — | — |
| rust-derive-order | ✅ | — | — | — | — | — | — | — | — |
| rust-error-display | ✅ | — | — | — | — | — | — | — | — |
| rust-must-use | ✅ | — | — | — | — | — | — | — | — |
| java-javadoc-missing | — | — | — | — | ✅ | — | — | — | — |
| java-try-resource | — | — | — | — | ✅ | — | — | — | — |
| java-string-concat | — | — | — | — | ✅ | — | — | — | — |
| ruby-predicate-method | — | — | — | — | — | — | ✅ | — | — |
| c-malloc-check | — | — | — | — | — | ✅ | — | — | — |
| c-sizeof-type | — | — | — | — | — | ✅ | — | — | — |
| prefer-interface | — | — | — | ✅ | — | — | — | — | — |

---

## 已知限制

1. **生成文件**：`.pb.go`、`.pulsar.go`、`_grpc.pb.go`、`*.gen.ts` 等尚未自动排除。使用 `--exclude` 标志或 `.garbage-code-hunter.toml` 手动过滤。

2. **跨文件重复**：近重复检测在大型代码库上可能产生较高的 issue 数量。正在改进中。

3. **评分**：非 Rust 项目可能显示偏高的分数，因为部分评分类别是 Rust 专属的。

4. **Java Javadoc 检测**：`java-javadoc-missing` 规则基于行的检测，可能无法识别跨多行的多行 Javadoc 注释。
