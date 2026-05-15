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

---

## Go-Specific Rules

| Rule | What it detects | Threshold | Severity |
|------|----------------|-----------|----------|
| `panic-abuse` | `panic()` calls | 0 | Mild → Nuclear |
| `goroutine-abuse` | `go` statement spawns | 8 | Spicy |
| `defer-in-loop` | `defer` inside `for` loop body | — | Spicy |

### Real-world example (interchange project)

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

## C/C++ Specific Rules

| Rule | What it detects | Threshold |
|------|----------------|-----------|
| `goto-abuse` | `goto` statements | 0 |
| `new-expression` | `new` expressions (C++ only) | 0 |
| `malloc-leak` | Heap allocations (malloc, curlx_malloc, zmalloc, etc.) | 0 |

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

---

## Known Limitations

1. **Generated files**: `.pb.go`, `.pulsar.go`, `_grpc.pb.go`, `*.gen.ts` etc. are not yet auto-excluded. Use `--exclude` flag or `.garbage-code-hunter.toml` to filter them manually.

2. **Cross-file duplication**: The near-duplicate detection can produce high issue counts on large codebases. This is being improved.

3. **Scoring**: Non-Rust projects may show inflated scores because some scoring categories are Rust-specific.
