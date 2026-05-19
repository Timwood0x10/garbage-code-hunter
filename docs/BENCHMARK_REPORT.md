# Garbage Code Hunter — 基准性能报告

**报告时间**: 2026-05-18  
**测试版本**: v0.2.3+  
**平台**: macOS (Darwin 24.6.0)  
**Rust**: stable  
**CPU**: Apple Silicon (M系列)  

---

## 测试概况

| 指标 | 值 |
|------|-----|
| 基准测试总数 | 15（含 8 个分组） |
| 单元测试总数 | 796+ |
| 支持语言 | 11 种 (Rust/Python/JS/TS/Go/Java/C/C++/Ruby/Swift/Zig) |

---

## 基准测试套件结构

```
benches/performance_tests.rs  ← 完全自包含，无需外部下载
├── Fixture generators         ← 4 种 + 2 种项目管理器
│   ├── create_large_garbage_file()     — Rust 100函数巨型垃圾文件
│   ├── create_clean_file()             — Rust 高质量代码文件
│   ├── create_python_garbage_file()    — Python 50函数垃圾文件
│   ├── create_js_garbage_file()        — JavaScript 50函数垃圾文件
│   ├── create_go_garbage_file()        — Go 50函数垃圾文件
│   ├── create_multi_file_project(N)    — N文件的Rust项目 (garbage/clean模式)
│   └── create_cross_file_project()     — 5个高度相似文件
└── 8个基准测试组
```

---

## 完整测试结果

### 创建开销

| 基准测试 | Mean (ns) | Median (ns) | StdDev (ns) |
|----------|-----------|-------------|-------------|
| `create_analyzer/default` | 708,739 | 708,739 | 24,023 |
| `create_analyzer/with_exclusions` | 1,302,141 | 1,302,141 | 3,333 |

### 单文件分析

| 基准测试 | Mean (ms) | Median (ms) |
|----------|-----------|-------------|
| `analyze_file/clean_rust` | 1.38 | 1.38 |
| `analyze_file/example_func_rs` | 2.00 | 2.00 |
| `analyze_file/example_ultimate_garbage` | 0.74 | 0.74 |
| `analyze_file/large_garbage_rust` (100 functions) | **62.66** | **62.66** |
| `analyze_file/single_large_file` | 65.90 | 65.90 |

### 管道对比（同一大文件）

| 基准测试 | Mean (ms) | Median (ms) | 相对 analyze_file |
|----------|-----------|-------------|-------------------|
| `analyze_file/single_large_file` | 65.90 | 65.90 | 1.0× (基线) |
| `analyze_to_findings/single_large_file` | 72.52 | 72.52 | 1.10× |
| `analyze_full/single_large_file` | 106.29 | 106.29 | 1.61× |

`analyze_full` 比 `analyze_file` 慢 61%，因为它做了更多聚合和后期处理。

### 全目录分析

| 基准测试 | Mean (ms) | Median (ms) |
|----------|-----------|-------------|
| `analyze_path/dir_10_garbage_files` (10 files) | 7.67 | 7.67 |
| `analyze_path/example_directory` (2 files) | 3.91 | 3.91 |
| `analyze_path/mixed_4_languages` (4 files) | **107.88** | **107.88** |
| `analyze_path/cross_file_5_similar` (5 files) | 3.16 | 3.16 |

> 混合语言项目（4文件×4语言）耗时最长（107ms）因为需要为每种语言初始化和解析。

### 多语言解析对比

| 基准测试 | Mean (ms) | Median (ms) |
|----------|-----------|-------------|
| `parse_file/rust_large` / 100 functions | 64.53 | 64.53 |
| `parse_file/python` / 50 functions | 9.41 | 9.41 |
| `parse_file/javascript` / 50 functions | 10.56 | 10.56 |
| `parse_file/go` / 50 functions | 10.42 | 10.42 |
| `parse_file/rust_clean` / well-written | 2.11 | 2.11 |

Rust 解析器生成的文件大（100函数），所以耗时最长。其他语言50函数都在 ~10ms 左右。

### 可扩展性（文件数量）

| 文件数 | Mean (ms) | 归一化 (per-file) | 比例 |
|--------|-----------|-------------------|------|
| 1 | 0.81 | 0.81 | 1.0× |
| 5 | 3.87 | 0.77 | 4.8× |
| 20 | 15.88 | 0.79 | 19.7× |
| 50 | **39.79** | 0.80 | **49.3×** |

**结论：近线性扩展**。每个文件的平均开销稳定在 ~0.79ms，表明代码分析是可扩展的。50个文件时总时间 ~40ms。

### 垃圾代码 vs 干净代码

| 场景 | 文件数 | Mean (ms) | 垃圾/干净 比例 |
|------|--------|-----------|----------------|
| `garbage/10` | 10 | 8.05 | 1.63× |
| `clean/10` | 10 | 4.95 | — |
| `garbage/50` | 50 | 38.87 | 1.64× |
| `clean/50` | 50 | 23.67 | — |
| `project/garbage/20` | 20 | 15.46 | 1.65× |
| `project/clean/20` | 20 | 9.39 | — |

**结论**：垃圾代码检测大约比干净代码慢 1.6×。这个比例在不同文件数量下高度一致，验证了基准测试的可靠性。

---

## 性能特性总结

1. **分析器创建** ~0.7ms（无排除）/ ~1.3ms（有排除）
2. **单文件分析** — 小文件 ~1-2ms，大文件（100函数）~63-65ms
3. **管道开销** — `analyze_full` 比 `analyze_file` 慢 61%
4. **可扩展性** — 近乎完美的线性缩放（每个文件 ~0.79ms 恒定开销）
5. **垃圾 vs 干净** — 垃圾代码分析比干净代码慢 ~1.6×，比例稳定
6. **跨语言** — 混合语言项目需要额外初始化开销，但解析速度相近

---

## 运行方式

```bash
# 完整运行（约 5 分钟）
cargo bench --bench performance_tests

# 快速模式（仅达到置信水平就停止）
cargo bench --bench performance_tests -- --quick

# 保存基线
cargo bench --bench performance_tests -- --save-baseline my_baseline

# 对比基线
cargo bench --bench performance_tests -- --baseline my_baseline

# 列出所有基准测试
cargo bench --bench performance_tests -- --list
```
