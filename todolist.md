# Style Guide Rules — Todolist

基于各语言官方编码风格指南 + 标准库编码规范，为每种语言添加专属规则。
现有通用规则保留不变，新增规则补充语言特定的风格检测。

## 状态说明

- [ ] 未开始
- [~] 进行中
- [x] 已完成

---

## Step 1: Go — Effective Go + Code Review Comments

### 新增规则

- [ ] `go-error-string` — error 字符串不应以大写字母开头或以标点结尾
  - 来源: [Go Code Review Comments #errors](https://github.com/golang/go/wiki/CodeReviewComments#error-strings)
  - 检测: `fmt.Errorf("Something went wrong")` → 应为 `fmt.Errorf("something went wrong")`
  - 严重性: Mild

- [ ] `go-context-first` — `context.Context` 参数必须是函数第一个参数
  - 来源: [Effective Go](https://go.dev/doc/effective_go)
  - 检测: `func Foo(x int, ctx context.Context)` → 应为 `func Foo(ctx context.Context, x int)`
  - 严重性: Mild

- [ ] `go-receiver-name` — 方法接收者名称应简短（1-2 字符），不应超过 2 字符
  - 来源: [Go Code Review Comments #receiver-names](https://github.com/golang/go/wiki/CodeReviewComments#receiver-names)
  - 检测: `func (s MyServer) Handle()` → 应为 `func (s *MyServer) Handle()` 且接收者名应如 `m`
  - 严重性: Mild

- [ ] `go-else-return` — `if err != nil { ... } else { ... }` 应翻转为 early return
  - 来源: [Go Code Review Comments](https://github.com/golang/go/wiki/CodeReviewComments)
  - 检测: if-else 块中 err 检查后跟 else 分支
  - 严重性: Mild

### 修改现有规则

- [ ] `single-letter-variable` — Go 语言豁免: `err`, `ok`, `ctx`, `_`, `mu`, `wg`, `ch`, `fn`
- [ ] `abbreviation-abuse` — Go 惯用缩写加入白名单: `ctx`, `req`, `resp`, `srv`, `cfg`, `buf`, `ch`, `wg`, `mu`, `fn`

---

## Step 2: Python — PEP 8 + Google Python Style

### 新增规则

- [ ] `python-type-ignore` — 检测 `# type: ignore` 注释
  - 来源: [Google Python Style Guide](https://google.github.io/styleguide/pyguide.html)
  - 检测: `x = get_value()  # type: ignore`
  - 严重性: Mild

- [ ] `python-fstring` — 应使用 f-string 而非 `.format()` 或 `%` 格式化
  - 来源: PEP 498, 现代 Python 惯例
  - 检测: `"hello {}".format(name)` 或 `"hello %s" % name`
  - 严重性: Mild

- [ ] `python-magic-method` — 用户自定义的 `__dunder__` 方法（非标准 dunder）
  - 来源: PEP 8 — "avoid inventing new magic methods"
  - 检测: `def __my_custom__(self):` — 不在标准 dunder 列表中
  - 严重性: Mild

### 修改现有规则

- [ ] `single-letter-variable` — Python 语言豁免: `self`, `cls`, `_` (throwaway), 推导式中的 `i`/`j`/`k`

---

## Step 3: Rust — API Guidelines + Clippy Conventions

### 新增规则

- [ ] `rust-doc-example` — 公共项有 `///` 文档注释但缺少 ```` ```` ``` 示例代码块
  - 来源: [Rust API Guidelines C-EXAMPLE](https://rust-lang.github.io/api-guidelines/documentation.html#c-example)
  - 检测: `/// Does something` 后没有 `` ``` `` 代码块
  - 严重性: Mild

- [ ] `rust-derive-order` — `#[derive(...)]` 未按规范顺序排列
  - 来源: Clippy 惯例, Rust 社区规范
  - 规范顺序: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default`, `Serialize`, `Deserialize`
  - 检测: `#[derive(Clone, Debug)]` → 应为 `#[derive(Debug, Clone)]`
  - 严重性: Mild

- [ ] `rust-error-display` — Error 类型实现了 `Debug` 但未实现 `Display`
  - 来源: [Rust API Guidelines C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#error-types-are-meaningful-and-well-behaved-c-good-err)
  - 检测: `impl fmt::Debug for MyError` 存在但无 `impl fmt::Display for MyError`
  - 严重性: Mild

- [ ] `rust-must-use` — 返回 `Result` 或 `Option` 的函数/方法缺少 `#[must_use]`
  - 来源: [Rust API Guidelines C-MUST-USE](https://rust-lang.github.io/api-guidelines/interoperability.html?highlight=must_use#c-must-use)
  - 检测: `pub fn foo() -> Result<T, E>` 没有 `#[must_use]`
  - 严重性: Mild

---

## Step 4: Java — Google Java Style Guide

### 新增规则

- [ ] `java-javadoc-missing` — 公共方法缺少 Javadoc 注释
  - 来源: [Google Java Style Guide §7](https://google.github.io/styleguide/javaguide.html#s7-javadoc)
  - 检测: `public void foo()` 前没有 `/** ... */`
  - 严重性: Mild

- [ ] `java-try-resource` — `try` + `finally` 中调用 `.close()` 应使用 try-with-resources
  - 来源: [Google Java Style Guide §6.1.2](https://google.github.io/styleguide/javaguide.html#s6.1.2-blocks)
  - 检测: `finally { resource.close(); }` 模式
  - 严重性: Mild

- [ ] `java-string-concat` — 循环中使用字符串拼接（应使用 StringBuilder）
  - 来源: JDK 编码惯例
  - 检测: for/while 循环内 `str += "..."` 或 `str = str + "..."`
  - 严重性: Mild

---

## Step 5: Ruby — Ruby Style Guide (RuboCop)

### 新增规则

- [ ] `ruby-frozen-string` — 文件缺少 `# frozen_string_literal: true` 魔法注释
  - 来源: [Ruby Style Guide #frozen-string-literal](https://rubystyleguide.gitbook.io/)
  - 检测: 文件第一行不是 `# frozen_string_literal: true`
  - 严重性: Mild

- [ ] `ruby-negated-if` — 使用 `if !condition` 而非 `unless`
  - 来源: [Ruby Style Guide #unless-for-negatives](https://rubystyleguide.gitbook.io/)
  - 检测: `if !foo` → 应为 `unless foo`（仅适用于简单条件）
  - 严重性: Mild

- [ ] `ruby-predicate-method` — 返回布尔值的方法未以 `?` 结尾
  - 来源: [Ruby Style Guide #predicate-methods](https://rubystyleguide.gitbook.io/)
  - 检测: `def is_valid` → 应为 `def valid?`
  - 严重性: Mild

---

## Step 6: C — Linux Kernel Style + CERT C

### 重构

- [ ] 从 `rust_rules.rs` 提取 C/C++ 规则到独立的 `c_rules.rs`
  - 迁移: `goto-abuse`, `malloc-leak`, `new-expression`
  - 更新 `mod.rs` 注册

### 新增规则

- [ ] `c-malloc-check` — `malloc` 返回值未检查 NULL
  - 来源: [CERT C MEM32-C](https://wiki.sei.cmu.edu/confluence/display/c/MEM32-C.+Detect+and+handle+memory+allocation+errors)
  - 检测: `p = malloc(size)` 后没有 `if (p == NULL)` 检查
  - 严重性: Spicy

- [ ] `c-sizeof-type` — 使用 `sizeof(type)` 而非 `sizeof(expr)`
  - 来源: [CERT C EXP42-C](https://wiki.sei.cmu.edu/confluence/display/c/EXP42-C.+Do+not+compare+sizeof+to+a+constant)
  - 检测: `sizeof(int)` → 应为 `sizeof(*ptr)` 或 `sizeof(variable)`
  - 严重性: Mild

- [ ] `c-const-correct` — 可以声明为 `const` 的函数参数未声明
  - 来源: [Linux Kernel Coding Style](https://www.kernel.org/doc/html/latest/process/coding-style.html)
  - 检测: 指针参数在函数体内未被修改但未标记 `const`
  - 严重性: Mild

---

## Step 7: 通用规则语言特化

修改 `complex_rules.rs` 中的跨语言规则，添加语言特定例外:

- [ ] `single-letter-variable` — 语言特化豁免
  - Go: `err`, `ok`, `ctx`, `_`, `mu`, `wg`, `ch`, `fn`
  - Python: `self`, `cls`, `_`
  - C: `i`, `j`, `k`, `n`, `p`, `s` (在函数作用域内)

- [ ] `god-function` — 语言特化豁免
  - Go: `main()`, `init()` 是惯用入口点
  - Python: `__init__`, `__str__`, `__repr__` 不应计入复杂度

- [ ] `abbreviation-abuse` — 语言特化白名单
  - Go: `ctx`, `req`, `resp`, `srv`, `cfg`, `buf`, `ch`, `wg`, `mu`, `fn`
  - Python: `cls`, `idx`, `fmt`, `msg`, `btn`, `img`
  - Java: `str`, `num`, `obj`, `arr`, `idx`

---

## Step 8: 更新评分分类

修改 `src/scoring.rs` 的 `build_categories()`:

- [ ] **naming** 新增: `go-receiver-name`, `ruby-predicate-method`
- [ ] **complexity** 新增: `go-else-return`, `ruby-negated-if`
- [ ] **code-smells** 新增: `go-error-string`, `go-context-first`, `python-type-ignore`, `python-fstring`, `python-magic-method`, `rust-doc-example`, `rust-derive-order`, `rust-error-display`, `rust-must-use`, `java-javadoc-missing`, `java-try-resource`, `java-string-concat`, `ruby-frozen-string`, `c-malloc-check`, `c-sizeof-type`, `c-const-correct`

---

## Step 9: 测试验证

- [ ] `make fmt && make check` — 0 errors
- [ ] `cargo test` — 全部通过
- [ ] 自举测试: `cargo run -- analyze .` — 验证新规则正确触发
- [ ] 多语言测试: 创建各语言违规代码，验证检测准确性
- [ ] 误报验证: 确认惯用写法不被误报（Go `if err != nil`、Python `self`、Rust 测试中的 `unwrap()`）

---

## 统计

| 语言 | 新增规则 | 修改规则 | 合计 |
|------|:--------:|:--------:|:----:|
| Go | 4 | 2 | 6 |
| Python | 3 | 1 | 4 |
| Rust | 4 | 0 | 4 |
| Java | 3 | 0 | 3 |
| Ruby | 3 | 0 | 3 |
| C | 3 | 0 | 3 |
| 通用 | 0 | 3 | 3 |
| **合计** | **20** | **6** | **26** |
