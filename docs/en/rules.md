# Analysis Rules Reference

Garbage Code Hunter uses tree-sitter AST parsing to detect code taste issues across 11 languages.
This is NOT a bug finder — it finds naming sins, magic numbers, deep nesting, god functions, print debugging, commented-out code, TODO mountains, copy-paste, and other code smells.

---

## Universal Rules (all languages)

These rules run on every supported language using language-specific tree-sitter queries.

### Naming

| Rule | What it detects | Severity |
|------|----------------|----------|
| `terrible-naming` | Variables named `data`, `info`, `temp`, `tmp`, `val`, `value`, `thing`, `stuff`, `obj`, `object`, `manager`, `handler`, `helper`, `util`, `utils` | Spicy |
| `single-letter-variable` | Single-character variable names (excludes loop counters via AST analysis) | Mild |
| `meaningless-naming` | Placeholder names: `foo`, `bar`, `baz`, `qux`, `aaa`, `bbb`, `xxx`, etc. | Mild/Spicy |
| `hungarian-notation` | Type prefixes (`strName`, `intCount`) and scope prefixes (`g_`, `m_`, `s_`, `p_`) | Mild |
| `abbreviation-abuse` | Abbreviations like `mgr`, `ctrl`, `hdlr`, `usr`, `pwd`, `btn`, `lbl`, `tbl`, `col`, `cnt` | Mild |

### Complexity

| Rule | What it detects | Threshold | Severity |
|------|----------------|-----------|----------|
| `deep-nesting` | Nesting depth > 5 levels | depth 5 | Mild → Nuclear |
| `long-function` | Functions longer than 80 lines (150 for test files) | 80 lines | Mild → Nuclear |
| `god-function` | Composite score from line count + parameters + control flow | score > 15 | Mild/Spicy |

### Code Smells

| Rule | What it detects | Severity |
|------|----------------|----------|
| `magic-number` | Integer/float literals not in common set (0, 1, -1, 2, 100, 10, 60, 24) | Mild |
| `println-debugging` | `println`, `print`, `console.log`, `fmt.Println`, `puts`, etc. | Spicy |
| `commented-code` | Blocks of 3+ consecutive commented-out code lines | Mild/Spicy |
| `todo-comment` | TODO/FIXME/BUG/HACK comments, `todo!()`/`unimplemented!()` macros | Mild/Spicy |

### Duplication

| Rule | What it detects | Severity |
|------|----------------|----------|
| `code-duplication` | Repeated 5-line chunks within the same file | Mild |
| `cross-file-duplication` | Identical functions across different files | Mild → Nuclear |
| `cross-file-near-duplicate` | Functions with >80% token similarity across files | Mild |

### Structure

| Rule | What it detects | Severity |
|------|----------------|----------|
| `file-too-long` | Files over 1000 lines (2000 for test files) | Mild → Nuclear |

---

## Rust-Specific Rules

| Rule | What it detects | Threshold |
|------|----------------|-----------|
| `unwrap-abuse` | `.unwrap()` calls | 0 (any call triggers) |
| `unnecessary-clone` | `.clone()` calls | 24 |
| `panic-abuse` | `panic!()` macros | 2 |
| `string-abuse` | `.to_string()` calls | 20 |
| `vec-abuse` | `vec!` macro calls | 15 |
| `async-abuse` | `async` blocks | 10 |
| `macro-abuse` | Macro invocations | 20 |
| `lifetime-abuse` | Lifetime annotations | 20 |
| `trait-complexity` | Methods in trait bodies | 10 |
| `generic-abuse` | Type parameters | 5 |
| `pattern-matching-abuse` | Tuple patterns | 15 |
| `box-abuse` | `Box::new` calls | 8 |
| `reference-abuse` | Reference types | 50 |
| `slice-abuse` | Slice types | 29 |
| `module-complexity` | Nested `mod` items | 0 |
| `complex-closure` | Nested closures (depth > 2) or > 5 parameters | — |
| `dead-code` | Unreachable code after return/break/continue/panic | — |
| `duplicate-imports` | Duplicate `use` statements | — |
| `rust-doc-example` | Doc comments (`///`) without example code block (&#96 &#96 &#96) | — |
| `rust-derive-order` | `#[derive(..)]` not in standard order (Debug, Clone, Copy, PartialEq...) | — |
| `rust-error-display` | Types implementing Debug but not Display | — |
| `rust-must-use` | `pub fn` returning Result/Option without `#[must_use]` | — |

---

## Go-Specific Rules

| Rule | What it detects | Threshold | Severity |
|------|----------------|-----------|----------|
| `panic-abuse` | `panic()` calls | 0 | Mild → Nuclear |
| `goroutine-abuse` | `go` statement spawns | 8 | Spicy |
| `defer-in-loop` | `defer` inside `for` loop body | — | Spicy |
| `go-receiver-name` | Method receivers longer than 2 characters | — | Mild |
| `go-error-string` | Error strings starting with uppercase letter | — | Mild |
| `go-context-first` | `context.Context` not as the first function parameter | — | Mild |
| `go-else-return` | `if-else` where `if` block has a return (should use early return) | — | Mild |

---

## Python-Specific Rules

| Rule | What it detects | Severity |
|------|----------------|----------|
| `bare-except` | `except:` without specifying exception type | Spicy |
| `wildcard-import` | `from module import *` (excludes known-FP modules) | Mild |
| `python-naming` | Functions not snake_case / classes not PascalCase | Mild |
| `compared-to-bool` | `if x == True` instead of `if x` | Mild |
| `not-is-none` | `x == None` instead of `x is None` | Mild |
| `python-type-ignore` | `# type: ignore` comments | Mild |
| `python-fstring` | `.format()` or `%` formatting instead of f-strings | Mild |
| `python-magic-method` | Non-standard `__dunder__` method definitions | Mild |

---

## Java-Specific Rules

| Rule | What it detects | Severity |
|------|----------------|----------|
| `empty-catch` | Empty `catch (Exception e) {}` blocks | Spicy |
| `constant-name` | `static final` fields not in UPPER_SNAKE_CASE | Mild |
| `java-javadoc-missing` | Public/protected methods missing Javadoc comments | Mild |
| `java-try-resource` | `try-finally` with `.close()` instead of try-with-resources | Mild |
| `java-string-concat` | String concatenation (`+=`) inside loops | Mild |

---

## Ruby-Specific Rules

| Rule | What it detects | Severity |
|------|----------------|----------|
| `global-variable` | Non-builtin global variables (`$xxx`) | Mild |
| `bare-rescue` | `rescue` without specifying exception class | Mild |
| `frozen-string` | Missing `# frozen_string_literal: true` magic comment | Mild |
| `negated-if` | `if !x` instead of `unless x` | Mild |
| `ruby-predicate-method` | Predicate methods (`is_xxx`, `has_xxx`) not ending with `?` | Mild |

---

## C/C++ Specific Rules

| Rule | What it detects | Threshold |
|------|----------------|-----------|
| `c-goto-abuse` | `goto` statements | 0 |
| `c-new-expression` | `new` expressions (C++ only) | 0 |
| `c-malloc-leak` | Heap allocations (malloc, curlx_malloc, zmalloc, etc.) | 0 |
| `c-malloc-check` | `malloc` return value not checked for NULL | — |
| `c-sizeof-type` | `sizeof(type)` instead of `sizeof(expr)` | — |

---

## TypeScript-Specific Rules

| Rule | What it detects | Severity |
|------|----------------|----------|
| `any-type` | `any` type annotations / `as any` casts | Mild |
| `prefer-interface` | `type Foo = { ... }` when `interface` could be used | Mild |

---

## Language-Specific Allowlists

The following idioms are exempted from universal rules to reduce false positives:

### single-letter-variable exemptions

| Language | Exempted identifiers |
|----------|---------------------|
| Go | `err`, `ok`, `ctx`, `mu`, `wg`, `ch`, `fn` |
| Python | `_` (throwaway) |
| C/C++ | `i`, `j`, `k`, `n`, `p`, `s` |

### abbreviation-abuse exemptions

| Language | Exempted abbreviations |
|----------|----------------------|
| Go | `ctx`, `req`, `resp`, `srv`, `cfg`, `buf`, `ch`, `wg`, `mu`, `fn`, `fmt`, `err`, `ok`, `http`, `json`, `tls`, `ssh` |
| Python | `cls`, `idx`, `fmt`, `msg`, `btn`, `img` |
| Java | `str`, `num`, `obj`, `arr`, `idx` |

### Other exemptions

| Rule | Language | Exempted pattern |
|------|----------|-----------------|
| `god-function` | Go | `func main()`, `func init()` |
| `any-type` | TypeScript | `*.d.ts` files |
| `hungarian-notation` | All | `c`, `t`, `ctx`, `req`, `res`, `err`, `db`, `kv`, `fs`, `io` |

---

## Language Coverage Matrix

| Rule | Rust | Go | Python | JS/TS | Java | C/C++ | Ruby | Swift | Zig |
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

## Known Limitations

1. **Generated files**: `.pb.go`, `.pulsar.go`, `_grpc.pb.go`, `*.gen.ts` etc. are not yet auto-excluded. Use `--exclude` flag or `.garbage-code-hunter.toml` to filter them manually.

2. **Cross-file duplication**: The near-duplicate detection can produce high issue counts on large codebases. This is being improved.

3. **Scoring**: Non-Rust projects may show inflated scores because some scoring categories are Rust-specific.

4. **Java Javadoc detection**: The `java-javadoc-missing` rule is line-based and may miss multi-line Javadoc comments that span several lines before the method declaration.
