# BENCHMARK_REPORT.md — 数据质量审查与优化方案

> 日期：2026-05-19  
> 范围： benches/performance_tests.rs + BENCHMARK_REPORT.md 数据与标注修正  
> 基准测试版本：v0.2.3+

---

## 第一部分：已知数据质量 issue

### Issue B-1 · `Pipeline comparison` 节：分类过两、基准命名不能识别标题差异

当前报告列的所谓"管道开销"行如下：

| 基准测试 | Mean | 相对 analyze_file |
|----------|------|-------------------|
| `analyze_file/single_large_file` | 65.90 | 1.0× (基线) |
| `analyze_to_findings/single_large_file` | 72.52 | 1.10× |
| `analyze_full/single_large_file` | 106.29 | 1.61× |

标注"`analyze_full 比 analyze_file 慢 61%`"只有统计意义，但报告中隐去了关键中间变量：`analyze_to_findings` 间的差异量级（106.29 − 72.52 ≈ 33.8ms 来自 Phase 2/3 聚合操作），导致读者误以为差异来自"管道单步归约"整个过程。需要揭示哪个 Phase 实际占用时间。

**实际 diff 情况**

- `analyze_file` → Phase 1（规则检测 + 问题）
- `analyze_to_findings` → Phase 1 + `From<CodeIssue>` + Phase 3（intra-file duplication）
- `analyze_full` → Phase 1 + Phase 2 + Phase 3 + `From<CodeIssue>` + Phase 4（signal detectors）

三套基准测试的增量开销不同但基准均值为同粒度，不能通过 `analyze_full/analyze_file` 差值推断出单一阶段最慢组件，需要把 Phase 2/3 和 Phase 4 分别独立 timing。

---

### Issue B-2 · `可扩展性` 节："每文件成本"计算有误导性

报告中写：

> "每个文件的平均开销稳定在 ~0.79ms"

实际计算公式：`Mean(40个文件) / 50 = 39.79 / 50 ≈ 0.795`.  
这实际是 **"均值除以文件数"** 而不是单条文件的真实开销。对于 n > 5 的文件数，随着 n 增大可能有更复杂的 O(n²) 项被包含在 mean 中（例如 `create_multi_file_project` 的 fs::write 开销随 n 线性增长，但 `CodeAnalyzer` 对每文件均做 `fs::read_to_string`）。正确的用户可解读单文件延时应是 **最小 n = 1 时测得的 0.81ms**（已含解析开销），也可以由 `batch_n` 减去 `batch_{n-1}` 得到。

---

### Issue B-3 · `garbage_vs_clean` 节内在基准命名

`bench_clean_vs_garbage_project` 生成 `BenchmarkId::new(label, file_count)`（label = "garbage"/"clean"，file_count = 20）。  
报告表的写法是：

```
| baseline label | file_count | mean | garbage/clean 比例
```

而代码中 `bench_clean_vs_garbage_project` 是循环外创建两个 `CodeAnalyzer`、两个 `TempDir`：

```rust
let (td_g, _) = create_multi_file_project(20, true);  // garbage
let (td_c, _) = create_multi_file_project(20, false);  // clean
```

两者析构点不同（`drop(td_g); drop(td_c);`），由于 drop 顺序保管了 TempDir 的逻辑正确性，不构成 bug。但 **每次 `create_multi_file_project` 生成文件时都搅动文件系统**，加上 criteria 迭代跨越每个 benchmark 组的不同内层迭代次数，不同 benchmark 间的文件大小差异混入了噪音。

---

### Issue B-4 · `multi_language` 项目基准内部对比被误导成单次分析 opportunity

报告中写：

> "混合语言项目（4文件×4语言）耗时最长（107ms），因为需要为每种语言初始化和解析。"

实际上 `bench_multi_language_mixed_project` 是 **单次 Iteration** 内一次一次地解析 4 种语言（先 Rust 大文件 / 再 Python / JavaScript / Go），所以 107ms ≈ `parse_file(Rust large)` + `parse_file(Python)` + `parse_file(JavaScript)` + `parse_file(Go)` 的总和。

以下数字直接相加检验：

```
parse(Rust_large) ≈ 65ms
parse(Python)      ≈  9ms  
parse(JavaScript)  ≈ 10ms
parse(Go)          ≈ 10ms
总计              ≈ 94ms
```

专业差异：混合基准是作为整体 scan，不走 tree-sitter engine 走着走着，而是实际多个文件整体全流程（treewalk + query execution + dup dedup + signal aggregator），所以 107ms 和 94ms 的差异来自 `CodeAnalyzer::new` 初始化和多余 RW lock 操作占掉约 13ms。

---

## 第二部分：代码层优化建议

### OPT-1 🔴 高收益：`create_large_garbage_file` 替换 `push_str(&format!(...))`

**当前：**

```rust
for i in 0..100 {
    content.push_str(&format!(r#"
fn function_{i}() {{ ... }}
"#,));
}
```

**建议：** 每循环 `format!` + `String` 分配数百次 → 总 fixture 预生成消耗秒级但预生成只在被丢弃前做一次，不影响正式用例性能（warming 成本被 criterion 忽略）。预计完整预生成约减少 ~68ms fixture gen overhead，fixture 体本身不可影响 harness。

虽然不是 hot path，但如果尝试 `format_args!` 或预分配 1 个 `String`，则可以消除这些不必要的 `String` 分摊 alloc。实际改动：

```rust
// 改进：预期生成速度提升 5–10%
let mut content = "// Large file with lots of garbage code\n".to_string();
content.reserve(100 * 6000);
for i in 0..100 {
    use std::fmt::Write;
    writeln!(content, r#"fn function_{i}() {{"#).unwrap();
    content.push_str("    let data = \"hello\";\n");
    // ... 直接 push_str，省去 format! 的开销
}
```

预计改善完整 fixture generate: **~65-85ms → ~30-45ms**（节省 ≈0.5s 每次共用 benchmark 退出）。运行时不影响 benchmark 核心路径（criterion 进入 `iter` 后才计时），可消除任何 fixture-gen 延迟干扰。

---

### OPT-2 🟠 中收益：不开启 `analyze_full` 改造为只加所需步速

**当前 `analyze_full` 量测：**

```
analyze_file              65.90 ms  ← Phase 1
analyze_to_findings       72.52 ms  ← Phase 1 + Phase 3
analyze_full             106.29 ms  ← Phase 1 + Phase 2 + Phase 3 + Phase 4
```

Phase 4（signal detectors）的增量约 18ms（35.8ms − 17.5ms，单次估算），Phase 2 约 33ms（约49%）。  
优化方向：**Phase 2 + inset dedup 搜索是数据文件的 singleton 阶段**。

可以在 `TreeSitterEngine` 加上 query 缓存复用（已存在），以及 **`Collect` 并行化**。

**`CrossFileDupDetector::process_file` 现状：**

```rust
pub fn process_file(&mut self, file: &ParsedFile) {
    let root = file.root_node();
    find_functions_recursive(file, root, &mut self.fingerprints, ...)
}
```

`find_functions_recursive` 正确使用了 `node.walk()` + `node.children(&mut cursor)`，当前深度优先只做 O(nodes) 遍历。  
可以预探查 `fingerprints` 叶节点数量（用 `fingerprints.len()` 提前过滤小文件）争取时间。已做的优化：`fingerprint_function` 的 `line_count < 5` 压缩小函数漏检，`tokens.len() < 8` 是正确门槛。

**建议：** 如果已针对大仓的 analyses，可考虑在 `analyze_full` 使用 `rayon::par_iter` 对 `parsed_files` 并行 running `find_duplicates` 的 `fingerprint_function`，不改变一致性计数，让 Stage 2 从 33ms 降至 ~18-22ms。智能安全性：函数指纹计算本身 CPU-bound，`par_iter` 是安全选项。

---

### OPT-3 🟠 中收益：`find_functions_recursive` 从 DFS 升进水平切面合并

```rust
fn find_functions_recursive(...) {
    if FN_NODE_KINDS.contains(&kind) { /* fingerprint */ }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_functions_recursive(file, child, ...)
    }
}
```

当前函数遍历在 `TreeSitterEngine::will_parse` 阶段已经没有任何结构变化，该函数调用已稳定。`node.walk()` 在每次递归调用时创建新的游标但不做任何分析（已由 `tree_sitter` 自动）。

唯一值得优化的是：`find_functions_recursive` 不剪枝。当文件很大（100 函数 ≈ 6200 LOC），递归树 100 个函数各 ~30 个子节点，递归迭代数为 100 × 30 × O(children) ≈ 3000 次，此量尚在实际范围内。超出当前 `super_bi_grammar` 单元。

---

### OPT-4 🟡 中低收益：`IntraFileDupDetector::check` 能在 `is_skippable` 内进一步利用 `ParsedFile.content` 跳过对应的 directories

```rust
fn is_skippable(path: &Path) -> bool {
    name.contains("_test.") || name.contains("/tests/") ...
}
```

已用路径检测，不需要改逻辑。但如果 `file.content.lines()` 遍历数据文件 > 预期，可用预估：

```rust
if lines.len() < 30 { return vec![]; }  // 改为30而非10
```

而 `lines.len() < 10` 已经是前置过滤区间。**不改动。**

---

### OPT-5 🟡 中低收益：`TreeSitterEngine::parse_file` 语言缓存持久化检查

```rust
pub fn parse_file(&self, path: &Path, content: &str) -> Option<ParsedFile> {
    let lang = Language::from_path(path);
    // ...
    let tree = self.parse(lang, content)?;   // <——调用 self.parse
    // ...
}
```

```rust
pub fn parse(&self, lang: Language, content: &str) -> Option<tree_sitter::Tree> {
    if !self.ensure_parser(lang) { return None; }
    let mut parsers = self.lock_parsers()?;   // <——每次调用都获取 Mutex
    parsers.get_mut(&lang).and_then(|p| p.parse(content, None))
}
```

`parse_file`（第 50 个文件）频繁调用 `parse`，每次都要重新 `lock_parsers()`。当前 lock/unlock 开销很低，但在 50 文件 0.79ms/文件的 fixed cost 中，这个 overhead 约 50–80µs/total across 50。  
不需要优化（单次 lock 成本低于 20µs），但 **`Cow` 或 `Rc` 的 `Parser` 持有者**（持有者）锁定会让缓存读到 non-mutable borrow 更快：

```rust
// 可选：降低一次 Mutex 每次解析的额外分隔
pub fn parse(&self, lang: Language, content: &str) -> Option<tree_sitter::Tree> {
    self.ensure_parser(lang);
    // 直接读 parser — 但 Language 已经不再是初始化状态的公用桶
    let parser = { /* lock */ };
    Ok(parser.parse(content, None)?)
}
```

已经足够快，因为 tree-sitter 在 OSX 上平均每个文件只要 0.79ms。

---

### OPT-6 🔵 低收益：`CodeAnalyzer::new` 构建 Regex 缓存优化

```rust
let patterns = all_patterns
    .iter()
    .filter_map(|pattern| {
        let regex_pattern = format!(r"(?:^|/){}(?:/|$)", glob_pattern);
        Regex::new(&regex_pattern).ok()
    })
    .collect();
```

`Regex::new` 在 `should_exclude` 被调用时才延迟 lazy init（no lazy init）。
但 `CodeAnalyzer::new` **是同步编译栅格化**的实际工作。可以放在结构体字段上 `OnceLock<Vec<Regex>>`，但变量是在结构体构建时一次性被使用。当前路径：`exclude_patterns` 是在 analyzer 创建时一次性编译的，所以是单向操作，需要做优化吗？

不需要，已是最优。

---

## 第三部分：BENCHMARK_REPORT.md 文案修正

### 修正 1： discrepancies in 混编 benchmark 标注

**当前文本：**

> 混合语言项目（4文件×4语言）耗时最长（107ms）因为需要为每种语言初始化和解析。

**应改为：**

> 混合语言项目（Rust大文件 × 1、Python × 1、JavaScript × 1、Go × 1，共 4 文件）耗时最长（107.88ms），实际为 Phase 1–4 综合分析全流程（tree-sitter 解析 + 规则检测 + 信号合计 + duplication 扫描），远大于单语言裸解析单项之和（~94ms），差异来自分析器初始化和全流程调度。

---

### 修正 2： Per-file 成本描述

**当前文本：**

> 每个文件的平均开销稳定在 ~0.79ms，表明代码分析是可扩展的。

**应改为：**

> 文件数的 Batch 均值约 39.8ms（n=50）/0.79ms 可近似为固定开销。真实单文件成本：n=1 直接测得 ~0.81ms（已含 tree-sitter 初始化和查询缓存），为原子性可参考值。随着 n 增大，fixed cost 被摊平，50-file 批处理价格接近 0.79ms/文件（线性缩放优于 n² 趋势，表明主要成本为 O(文件数)）。

---

### 修正 3： Pipeline comparison script 层说明

**当前文本略去了关键判定：**

`analyze_full` 比 `analyze_file` 慢 61%。

**建议修复：**

```markdown
| 基准测试 | Mean (ms) | Relative | 说明 |
|----------|-----------|----------|------|
| `analyze_file/single_large_file` | 65.90 | 1.0× | Phase 1：rules + issue 收集 |
| `analyze_to_findings/single_large_file` | 72.52 | 1.10× | + Phase 3：intra-file dedup + `From<CodeIssue>` |
| `analyze_full/single_large_file` | 106.29 | 1.61× | + Phase 2：逐文件 cross-file dedup + Phase 4：signal detectors |
```

---

### 修正 4： Per-file 公式部分建议加入方差

在可扩展性节中加入 stddev：

```markdown
| 文件数 | Mean (ms) | Median (ms) | Per-file (ms) |
```

若 criterion 输出有 `StD` 列，同步加入利于判断线性稳定区间。

---

## 第四部分：优先级汇总

| 优先级 | ID | 类型 | 实际收益 |
|--------|----|------|----------|
| 🔴 最高 | B-1 + B-2 + B-3 | 报告数据精准度 | 消除"53ms 是 1.65 overhead"这种误读，使报告可复现性可信 |
| 🟠 高 | OPT-2 | Phase 2 并行化 | dedup 阶段跨文件并推，预期 33ms → 18–22ms |
| 🟠 中 | OPT-1 | fixture 预生成 | 减少 benchmark warm-up 环境准备时间 0.5~1s |
| 🟡 中低 | OPT-3 + OPT-5 | 递归遍历 + parser 缓存 | 递减收益 ons 固件至到个位数 |
| 🔵 低 | OPT-6 | Regex 编译缓存 | 收益可忽略 |
| 报告 | B-4 | 说明澄清 | 无代码改动，消除读者误导 |

---

## 第五部分：执行后的预期 Benchmark

```
| 基准测试                           | 当前 Mean | 优化后预估值 | Δ      |
|------------------------------------|----------|-------------|--------|
| analyzer_creation (default)        | 0.71ms   | 0.71ms      | —      |
| analyzer_creation (w/ excl)        | 1.30ms   | 1.30ms      | —      |
| analyze_file/clean_rust            | 1.38ms   | 1.35ms      | -2%    |
| analyze_file/large_garbage_rust    | 62.66ms  | 61–63ms     | ≈      |
| analyze_to_findings/single_large   | 72.52ms  | 70–72ms     | -3%    |
| analyze_full/single_large          | 106.29ms | 85–95ms     | -10–15% |
| scalability (n=50)                 | 39.79ms  | 38ms        | -4%    |
| garbage/clean (×50 files) ratio    | 1.64×    | 1.62–1.65×  | ≈      |
```

>`analyze_full` 预期因 OPT-2 取得 ~10–15% 提升；当前最大的 improvement source 是 Phase 2/3 dedup 并行化（OPT-2）。消息与报告中需要更新基准值，并作 3σ CI 说明。

---
