# Garbage Code Hunter — TODO List

## ✅ 已完成

- [x] **scan 命令集成所有分析工具** — 将 10 个缺失工具（last-words, debt-invoice, personality, decay, autopsy, radar, ci-bot, persona, danger-zone, team-roast）接入 `run_scan`，加权评分，共享 `quick_scan_score`
- [x] **Rule 系统清理** — 消除 33 处 `is_test_file` 样板代码，添加 `skips_test_files()` trait 方法，修复 6 个 clippy 警告
- [x] **严重性加权评分** — scoring.rs 支持 Nuclear=3x, Spicy=1.5x, Mild=0.5x 权重，覆盖全部 37 条规则
- [x] **规则单元测试** — naming, rust_specific, complexity, code_smells, student_code, garbage_naming 共 21 个测试
- [x] **文档注释** — Rule trait, RuleEngine, scoring 结构体, 37 个 rule struct 添加 `///` 文档

## 📋 待完成

### #31 — rayon 并行分析优化 ★★☆☆☆ ✅

**目标**: 大型代码库分析提速

- [x] `Cargo.toml` 添加 `rayon` 依赖
- [x] `src/analyzer.rs` — `analyze_path()` 中文件迭代改为 `par_iter()` 并行分析（两阶段：Phase 1 并行单文件分析，Phase 2 顺序跨文件去重）
- [x] `src/main.rs` — `run_scan` 中 14 个独立工具通过 `std::thread::scope` 并行执行
- [x] `src/main.rs` — `quick_scan_score` 中 7 个工具并行执行
- [x] `Rule` trait 添加 `Send + Sync` bounds 支持并行

### #30 — 接入 ProjectConfig 配置系统 ★★★☆☆ ✅

**目标**: 支持 `.garbage-code-hunter.toml` 项目级配置

- [x] `src/main.rs` — `run_scan` 和 `run_analyze` 加载 `.garbage-code-hunter.toml`（`--project-config` 或自动发现），传递给 `CodeAnalyzer::with_config()`
- [x] `src/analyzer.rs` — `CodeAnalyzer::with_config()` 接受 `ProjectConfig`，传递给 `RuleEngine::with_config()`，合并 config 中的 exclude patterns
- [x] `src/rules/mod.rs` — `RuleEngine::is_rule_disabled_by_config()` 读取 config 规则开关；`check_file_with_context()` 支持文件级 override 的 `disabled_rules`
- [x] `src/rules/rust_specific.rs` — `unwrap-abuse` 读取 `config.rules.unwrap.threshold` / `nuclear_threshold`
- [x] `src/rules/code_smells.rs` — `magic-number` 读取 `config.whitelists.magic_numbers` + `config.rules.magic_number.allowed_numbers`
- [x] `src/rules/student_code.rs` — `println-debugging` 读取 `config.rules.println.threshold` / `allow_in_main_files`
- [x] `src/rules/naming.rs` — `terrible-naming` 读取 `config.rules.naming.allowed_names` + `config.whitelists.variable_names`

### #29 — 多语言支持（JS/TS、Python、Go、Java）★★★★★

**目标**: 从 Rust-only 扩展到多语言分析

#### 架构设计
- [ ] 设计语言无关的规则抽象层（当前 `Rule` trait 参数为 `syn::File`，深度耦合 Rust AST）
- [ ] 方案选择：regex/行分析 vs tree-sitter 解析器 vs 混合方案

#### 语言检测
- [ ] `src/analyzer.rs` — 移除硬编码 `ext == "rs"`，改为语言→扩展名映射表
- [ ] 添加语言检测函数：`.rs`→Rust, `.js/.ts`→JS/TS, `.py`→Python, `.go`→Go, `.java`→Java

#### 可移植规则（基于正则/行分析，不依赖 AST）
- [ ] `naming` — 变量名检测适配各语言的变量声明语法
- [ ] `complexity` — 嵌套深度用缩进/大括号计数实现
- [ ] `code-smells` — magic number, commented code, dead code 通用实现
- [ ] `student-code` — `println!` → `console.log`(JS), `print`(Python), `fmt.Println`(Go), `System.out.println`(Java)

#### 测试文件检测
- [ ] `src/analyzer.rs` — `is_test_file()` 扩展：`_test.go`, `.test.js`, `test_*.py`, `*Test.java`

#### 项目类型检测
- [ ] `src/context/file_context.rs` — `detect_project_type` 扩展：读取 `package.json`, `requirements.txt`, `go.mod`, `pom.xml`

#### 评分
- [ ] `src/scoring.rs` — 添加通用语言类别，移除/重命名 `rust-basics`, `advanced-rust`, `rust-features` 等 Rust 专属类别
