# 规则参考

本页概述分析器使用的规则族。
具体规则会持续演进，但下面这些分类是稳定的。

## 规则族

- 命名质量：糟糕命名、缩写滥用、单字母变量
- 结构质量：深层嵌套、超长函数、上帝函数
- 可维护性：重复代码、注释掉的代码、陈旧 TODO 注释
- 调试残留：`println` 一类的打印调试
- 语言特定规范：Go、Python、Rust、Java、C、C++、Ruby、Swift、Zig 规则

## 常见问题

- `magic-number`
- `deep-nesting`
- `long-function`
- `god-function`
- `println-debugging`
- `commented-code`
- `todo-comment`
- `duplication`

## 语言覆盖

- 支持 Rust、Go、Python、JavaScript、TypeScript、Java、C、C++、Ruby、Swift、Zig
- 源码解析使用的 tree-sitter 语法覆盖了完整支持列表
- 文件发现逻辑会识别 `src/language/mod.rs` 中列出的对应扩展名

## 调参建议

- 用 `.garbage-code-hunter.toml` 白名单化可接受的名称和数字
- 用 `exclude-patterns` 过滤生成代码
- 用 `directories` 降低噪音目录的敏感度
- 用 CLI `--exclude` 做一次性的排除

## 备注

- 规则名和严重度是偏娱乐化的，不是权威审计结论
- 这个工具不承诺发现 bug、安全问题或性能退化
