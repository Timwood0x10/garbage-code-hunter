# 🗑️ Garbage Code Hunter v0.2.0

## What's New

### 🔥 Cross-File Analysis (New!)
- **Function fingerprinting**: Detect code clones across multiple files
- **Normalized token matching**: Variable names don't matter, structure does
- **Exact + Near duplicate detection**: Find both copy-paste and refactored clones
- **Configurable similarity threshold**: Control how strict the matching is

### 🎯 Context-Aware Analysis
- **File type detection**: Automatically detect test files, examples, benchmarks
- **Smart filtering**: Reduce false positives in example/test code
- **Project-level configuration**: `.garbage-code-hunter.toml` support
- **Per-directory overrides**: Different rules for different parts of your project

### 🛠️ Code Quality Improvements
- **Performance**: Regex compilation optimized with `OnceLock`
- **Bug fixes**: Clone detection threshold logic fixed
- **Dead code removal**: Cleaned up unused variables and functions
- **Better error handling**: Improved CLI stability

### 🔌 VSCode Extension
- **Real-time analysis**: Analyze on save with smart debouncing
- **ErrorLens-style inline messages**: See roasts directly in your code
- **Multi-language support**: Auto-detect Chinese/English from comments
- **LLM integration**: Optional Ollama/OpenAI for creative roasts
- **Educational mode**: Get suggestions on how to fix issues

## Installation

### CLI
```bash
cargo install garbage-code-hunter
```

### VSCode Extension
Search "Garbage Code Hunter" in the VSCode marketplace.

## Quick Start

```bash
# Analyze current directory
garbage-code-hunter

# Analyze with Chinese roasts
garbage-code-hunter --lang zh-CN src/

# Generate Markdown report
garbage-code-hunter --markdown src/ > report.md

# Enable LLM roasts
garbage-code-hunter --llm --llm-model gemma4:e2b src/
```

## Assets

| Platform | Download |
|----------|----------|
| Linux AMD64 | `garbage-code-hunter-linux-amd64.tar.gz` |
| macOS AMD64 | `garbage-code-hunter-macos-amd64.tar.gz` |
| Windows AMD64 | `garbage-code-hunter-windows-amd64.zip` |
| VSCode Extension | `garbage-code-hunter-vscode.vsix` |

## Breaking Changes

None - fully backward compatible with v0.1.x

## Known Issues

- Cross-file analysis may use significant memory on very large projects (configurable via `max_memory_mb`)
- LLM roasts require local Ollama or OpenAI-compatible API

## Thanks

Special thanks to the inspiration from [fuck-u-code](https://github.com/Done-0/fuck-u-code.git)!

---

**Full Changelog**: https://github.com/yourusername/garbage-code-hunter/compare/v0.1.0...v0.2.0
