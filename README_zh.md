# 🗑️ Garbage Code Hunter

[English](./README.md) | [中文](README_zh.md)


一个幽默的 Rust 代码质量检测工具，用最毒舌的方式吐槽你的垃圾代码！

> **灵感来源**: https://github.com/Done-0/fuck-u-code.git

## 这是什么？

Garbage Code Hunter 是一个 Rust 静态分析工具，不同于传统 linter 给你干巴巴的警告，我们用**毒舌、机智、毫不留情**的方式告诉你代码有多烂。

就像一位不怕伤你自尊的代码审查员（当然是为你好）。

## 能做什么？

- 🔍 **检测代码异味**：命名糟糕、嵌套太深、函数太长、unwrap 滥用...
- 🗣️ **毒舌吐槽**：每条警告都附带幽默的吐槽，让你笑着改代码
- 📊 **质量评分**：0-100 分，分数越高代码越烂
- 🌍 **中英双语**：支持中文和英文吐槽
- 🤖 **LLM 增强**：可接入 Ollama 生成更创意的吐槽
- 🔌 **VSCode 插件**：实时在编辑器里吐槽你的代码

## 快速开始

### 安装

```bash
cargo install garbage-code-hunter
```

### 使用

```bash
# 分析当前目录
garbage-code-hunter

# 分析指定文件
garbage-code-hunter src/main.rs

# 中文吐槽
garbage-code-hunter --lang zh-CN src/

# 生成 Markdown 报告
garbage-code-hunter --markdown src/ > report.md
```

## 示例输出

```
🗑️  垃圾代码猎人 🗑️

📊 垃圾代码检测报告
────────────────────────────────────
📈 问题统计:
   8 🔥 核弹级问题 (需要立即修复)
   15 🌶️  辣眼睛问题 (建议修复)
   12 😐 轻微问题 (可以忽略)

🏆 代码质量评分
────────────────────────────────────
   📊 总分: 63.0/100 😞
   🎯 等级: 较差

📁 src/main.rs
  🏷️ 变量命名问题: "这个变量名比我的密码还随意"
  📦 嵌套深度问题: "嵌套这么深，是想挖到地核吗？"
  ⚠️ unwrap 滥用: "unwrap() 比我的情绪还不稳定"
```

## VSCode 插件

在 VSCode 中实时吐槽你的代码：

1. 安装 `garbage-code-hunter` CLI
2. 在 VSCode 扩展市场搜索 "Garbage Code Hunter"
3. 保存 Rust 文件时自动触发分析

## 许可证

Apache License 2.0

---

**记住**：我们吐槽的是代码，不是你。让代码审查变得有趣一点！🚀
