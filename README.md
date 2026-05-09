# 🗑️ Garbage Code Hunter

[English](README.md) | [中文](./README_zh.md)

A humorous Rust code quality detector that roasts your garbage code with style!

> **Inspiration**: https://github.com/Done-0/fuck-u-code.git

## What is this?

Garbage Code Hunter is a Rust static analysis tool. Unlike traditional linters that give you dry warnings, we tell you how bad your code is in a **sarcastic, witty, and brutally honest** way.

Think of it as a code reviewer who isn't afraid to hurt your feelings (but it's for your own good).

## What can it do?

- 🔍 **Detect code smells**: Bad naming, deep nesting, long functions, unwrap abuse...
- 🔗 **Cross-file duplication detection** (NEW!): Find copy-pasted code across files
- 🎯 **Context-aware analysis**: Automatically adjusts sensitivity for tests/examples/UI code
- 🗣️ **Savage roasts**: Every warning comes with a humorous roast to make you laugh while fixing
- 📊 **Quality scoring**: 0-100 scale, higher score = worse code
- 🌍 **Bilingual**: Supports both English and Chinese roasts
- 🤖 **LLM powered**: Connect to Ollama for even more creative roasts
- 🔌 **VSCode extension**: Real-time roasting in your editor

## ✨ New in v0.2.0

### 🚀 Cross-File Duplication Detection
Automatically detects duplicated functions across your entire codebase:
```bash
garbage-code-hunter --verbose src/
# Output includes: 📌 cross-file-duplication: 33 issues
```

### 🎯 Context-Aware Analysis
Reduces false positives by understanding code context:
- **Example/Demo code**: Reduces sensitivity by 70%
- **Test code**: Reduces sensitivity by 80%
- **Benchmark/Documentation**: Further reductions
- **Business code**: Full detection (default)

### ⚡ Performance Improvements
- **5x faster**: Large projects (200+ files) analyzed in <1 second
- **Low memory usage**: <50MB for 200+ file projects
- **Zero warnings**: Clean compilation with 57 passing tests

## Quick Start

### Install

```bash
cargo install garbage-code-hunter
```

### Usage

```bash
# Analyze current directory
garbage-code-hunter

# Analyze specific file
garbage-code-hunter src/main.rs

# Chinese roasts
garbage-code-hunter --lang zh-CN src/

# Generate Markdown report
garbage-code-hunter --markdown src/ > report.md
```

## Example Output

```
🗑️  Garbage Code Hunter 🗑️

📊 Code Quality Report
────────────────────────────────────
📈 Issue Statistics:
   8 🔥 Nuclear Issues (fix immediately)
   15 🌶️  Spicy Issues (should fix)
   12 😐 Mild Issues (can ignore)

🏆 Code Quality Score
────────────────────────────────────
   📊 Total: 63.0/100 😞
   🎯 Level: Poor

📁 src/main.rs
  🏷️ Naming issue: "This variable name is more random than my passwords"
  📦 Nesting issue: "Nesting so deep, trying to dig to the Earth's core?"
  ⚠️ Unwrap abuse: "unwrap() is more unstable than my emotions"
```

## VSCode Extension

Get real-time roasting in VSCode:

1. Install the `garbage-code-hunter` CLI
2. Search "Garbage Code Hunter" in VSCode marketplace
3. Analysis triggers automatically when you save Rust files

## License

Apache License 2.0

---

**Remember**: We roast the code, not you. Let's make code reviews a bit more fun! 🚀
