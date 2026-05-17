# Garbage Code Hunter - 优化分析报告

**更新时间**: 2026-05-17  
**分析版本**: v0.2.2  
**当前状态**: 已进入 Style IR 增量迁移阶段

---

## 项目定位

**Garbage Code Hunter** 是一个幽默的代码风格分析工具。它不是传统静态错误检测器，而是一个"会吐槽代码风格的朋友"。

核心价值不是"发现多少 bug"，而是：

- 像朋友一样指出代码读起来哪里难受
- 用幽默但可信的方式降低代码评审压力
- 帮用户逐步改善代码风格习惯
- 通过历史趋势形成"陪伴感"和"记忆感"

---

## 当前真实架构状态

### 已经存在的能力

项目已经完成了早期优化报告中提到的部分基础设施：

- `SignalDetector` 已存在，定义在 `src/signals.rs`
- `PanicAddictionDetector`、`NamingChaosDetector`、`NestedHellDetector` 等直接信号检测器已存在，定义在 `src/detectors.rs`
- `StyleFinding` 已存在，定义在 `src/finding.rs`
- `CodeIssue -> StyleFinding` 兼容转换已存在
- `LanguageAdapter` 已存在，定义在 `src/language/adapter/mod.rs`
- `CodeAnalyzer` 已经能运行 direct signal detection，并产生 signal-level findings
- `StyleIr` 已作为增量迁移骨架加入，定义在 `src/style_ir/mod.rs`
- `PanicAddictionDetector`、`NamingChaosDetector`、`NestedHellDetector`、`HotfixCultureDetector`、`OverEngineeringDetector`、`CodeSmellsDetector` 已改为消费 `StyleIr`

因此，后续优化重点不再是"创建 SignalDetector / StyleFinding / LanguageAdapter"，而是：

> 把已有的 LanguageAdapter、SignalDetector、StyleFinding 正式收敛到稳定的 Style IR 管线。

---

## 目标架构

```text
Source Code
  ↓
Tree-sitter ParsedFile
  ↓
LanguageAdapter / StyleIrExtractor
  ↓
Style IR
  ↓
SignalDetector / Rule Compatibility Adapter
  ↓
StyleFinding
  ↓
StyleProfile
  ↓
FriendFeedback / Reporter / JSON / CI / VSCode
```

设计原则：

- 语言相关复杂度截止在 `LanguageAdapter` / `StyleIr` 之前
- `StyleIr` 不是通用 AST，只保存吐槽代码风格需要的事实
- `SignalDetector` 应逐步消费 `StyleIr`，减少直接依赖 tree-sitter 查询
- 旧的 rule pipeline 暂时保留，通过 adapter 与 `StyleFinding` 兼容
- 每次迁移一个 detector，确保行为不打折

---

## 优化空间分析

### P0：稳定 Style IR Schema

**状态**：进行中。

当前新增的 `StyleIr` 包含以下语言无关事实：

```rust
pub struct StyleIr {
    pub language: Language,
    pub line_count: usize,
    pub functions: Vec<FunctionNode>,
    pub panic_call_count: usize,
    pub naming_violation_count: usize,
    pub deeply_nested_block_count: usize,
    pub debug_call_count: usize,
    pub excessive_param_count: usize,
    pub unsafe_block_count: usize,
    pub magic_number_count: usize,
}
```

**已完成**：

- `panic_call_count` 已被 `PanicAddictionDetector` 消费
- `naming_violation_count` 已被 `NamingChaosDetector` 消费
- `deeply_nested_block_count` 已被 `NestedHellDetector` 消费
- `debug_call_count` 已被 `HotfixCultureDetector` 消费
- `functions` / `excessive_param_count` 已被 `OverEngineeringDetector` 消费
- `unsafe_block_count` / `magic_number_count` 已被 `CodeSmellsDetector` 消费
- `excessive_param_count` 使用旧 detector 的 `>5` 参数阈值，避免行为漂移
- `StyleIr` 字段已补充语义文档：字段只记录 adapter 提取的事实，解释与权重留给 detector 方法

**下一步**：

- 持续保持 `StyleIr` 字段为事实层，避免把解释逻辑下沉到字段本身
- 避免把 `StyleIr` 做成通用 AST
- 保持字段稳定，供 JSON、VSCode、CodeTribunal 消费

**验收标准**：

- 至少 3 个 detector 可以基于 `StyleIr` 运行（已完成）
- 原有 detector 测试保持通过
- `make check` 0 errors
- 单个文件不超过 1000 行

---

### P1：迁移 Direct Signal Detectors 到 Style IR

**状态**：已完成主要 direct detector 迁移。

已完成：

- `PanicAddictionDetector` 已从直接调用 `LanguageAdapter` 改为读取 `StyleIr::panic_call_count`
- `NamingChaosDetector` 已从直接调用 `LanguageAdapter` 改为读取 `StyleIr::naming_violation_count`
- `NestedHellDetector` 已从直接调用 `LanguageAdapter` 改为读取 `StyleIr::deeply_nested_block_count`
- `HotfixCultureDetector` 已从直接调用 `LanguageAdapter` 改为读取 `StyleIr::debug_call_count`
- `OverEngineeringDetector` 已从直接调用 `LanguageAdapter` 改为读取 `StyleIr::over_engineering_count()`
- `CodeSmellsDetector` 已从直接调用 `LanguageAdapter` 改为读取 `StyleIr::code_smell_count()`

待迁移：

- `DuplicationDetector` 暂不迁移。它依赖 `IntraFileDupDetector::check(file)`，属于结构匹配，不适合塞进当前最小 Style IR。

**原则**：

- 一次只迁移一个 detector
- 每次迁移后跑对应单元测试
- 行为保持一致，不能因为迁移丢功能
- 避免大爆炸式重构

---

### P2：拆分超大规则文件

**状态**：待开始。

当前违反单文件 1000 行限制的文件包括：

- `src/treesitter/rules/rust_rules.rs`
- `src/treesitter/rules/complex_rules.rs`
- `src/reporter/display.rs`

**建议做法**：

- 不改变行为，只做模块拆分
- 先拆 `rust_rules.rs`，因为它最大且语言边界清晰
- 再拆 `complex_rules.rs`，按 complexity / naming / debug / magic number 分组
- 最后拆 `reporter/display.rs`，按 terminal section 分组

**禁止事项**：

- 禁止顺手重写规则逻辑
- 禁止删除文件
- 禁止用大范围 find-and-replace 改符号名

---

### P3：统一人格与风格画像

**状态**：待开始。

当前存在多套风格解释逻辑：

- issue count based personality
- signal based autopsy
- reporter output 中的展示逻辑

目标是统一到：

```text
StyleFinding[]
  ↓
StyleProfile
  ↓
Personality / Autopsy / FriendFeedback
```

这样可以保证 CLI、Markdown、CI Bot、VSCode extension 对同一份事实给出一致解释。

---

### P4：朋友式反馈层

**状态**：待开始。

建议新增：

```text
src/friend/
  mod.rs
  feedback.rs
  persona.rs
  roast.rs
  templates.rs
```

目标输出：

1. 今日整体观感
2. 最明显的代码习惯
3. 最值得先改的 3 个位置
4. 一句吐槽
5. 下一步行动建议

---

## 正确的实施路线图

### Phase 1：Style IR 试点

- [x] 新增 `src/style_ir/mod.rs`
- [x] 定义最小 `StyleIr`
- [x] `PanicAddictionDetector` 消费 `StyleIr`
- [x] 迁移 `NamingChaosDetector`
- [x] 迁移 `NestedHellDetector`
- [x] 为 `StyleIr` 增加 panic / naming / nested 字段测试
- [x] 为 `StyleIr` 字段补充事实层语义文档

### Phase 2：Detector 迁移

- [x] 迁移 `HotfixCultureDetector`
- [x] 迁移 `OverEngineeringDetector`
- [x] 迁移 `CodeSmellsDetector`
- [x] 确认迁移 detector 的单测与 Style IR 字段测试通过
- [ ] `DuplicationDetector` 保留在旧路径，等待未来单独设计 duplication IR

### Phase 3：规则文件拆分

- [ ] 拆分 `rust_rules.rs`
- [ ] 拆分 `complex_rules.rs`
- [ ] 拆分 `reporter/display.rs`
- [ ] 每个模块拆完后运行 `make fmt` 和 `make check`

### Phase 4：结构化输出稳定

- [ ] 稳定 `StyleFinding` JSON schema
- [ ] 输出 `StyleIr` 摘要信息
- [ ] 给 VSCode / CI / CodeTribunal 提供稳定 report schema

### Phase 5：朋友式反馈

- [ ] 新增 `FriendFeedback`
- [ ] 新增 persona-based roast 模板
- [ ] 将 reporter 输出迁移到 feedback model

---

## 风险评估

### 当前迁移风险

**风险等级**：低到中。

原因：

- `StyleIr` 是新增模块，不破坏旧 rule pipeline
- 当前迁移了六个 detector，且都只读取 `StyleIr` 字段或方法
- `PanicAddictionDetector` 的上游影响为低风险
- `NamingChaosDetector` 和 `NestedHellDetector` 的上游影响均为低风险
- `HotfixCultureDetector`、`OverEngineeringDetector`、`CodeSmellsDetector` 的上游影响均为低风险
- `SignalDetector` trait 本身影响为中风险，但本次没有修改 trait 签名

### 最大风险

- `StyleIr` 字段语义不稳定，导致后续 JSON / VSCode / CodeTribunal 消费端频繁变更
- 一次性迁移太多 detector，导致行为回归难以定位
- 过早删除旧 rule pipeline，导致功能打折

### 控制策略

- 每次只迁移一个 detector
- 保留旧管线直到 Style IR 覆盖主要信号
- 每次迁移都保留原有测试
- 使用 adapter 兼容旧数据结构

---

## 编码规范约束

后续所有修改必须遵守 `plan/rules/rules.md`：

- 禁止删除项目文件
- 禁止执行 `rm` 命令
- 单个文件不超过 1000 行
- 修改后执行 `make fmt`
- `make check` 必须 0 errors
- warning 可以暂时不处理
- 禁止使用 `#[allow(dead_code)]` 修 warning
- 禁止执行任何 git 命令
- 禁止覆盖率测试
- 注释必须是英文
- 测试应验证真实模块行为，不能滥竽充数

---

## 一句话总结

优化方向已经从"补基础设施"升级为"收敛架构"：项目现在应该把已有的 `LanguageAdapter`、`SignalDetector`、`StyleFinding` 汇聚到稳定的 `StyleIr`，再基于它做跨语言吐槽、朋友式反馈和 CodeTribunal 评审团集成。
