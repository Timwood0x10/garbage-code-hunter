# Garbage Code Hunter 🗑️ - VS Code Extension

A humorous Rust code quality detector that roasts your garbage code with style! This VS Code extension provides real-time analysis of your Rust code, highlighting potential issues with witty and educational feedback.

## ✨ Features

- 🔍 **Real-time Analysis**: Automatically analyze your Rust files on save
- 🎭 **Humorous Feedback**: Get roasted with style while learning better coding practices
- 📊 **Visual Diagnostics**: Issues highlighted directly in your editor with inline messages
- 🌍 **Multi-language Support**: Available in English and Chinese
- ⚡ **ErrorLens-style Display**: Inline messages show issues right next to your code
- 🎯 **Contextual Actions**: Right-click to analyze specific files
- 📈 **Workspace Analysis**: Analyze your entire project at once
- 🤖 **LLM-Powered Roasts**: Generate creative roasts using local LLMs (Ollama/OpenAI-compatible)
- 📊 **Quality Score**: View code quality score with detailed category breakdown
- 🎓 **Educational Mode**: Get explanations and fix suggestions for detected issues
- 🔧 **Highly Configurable**: Customize language, display options, and LLM settings

## 🚀 Quick Start

### Prerequisites

1. Install the `garbage-code-hunter` CLI tool:
   ```bash
   cargo install garbage-code-hunter
   ```

2. Make sure the CLI tool is available in your PATH:
   ```bash
   garbage-code-hunter --version
   ```

### Installation

1. Install this extension from the VS Code marketplace
2. Open a Rust project
3. Save a `.rs` file to trigger automatic analysis
4. Watch the magic happen! 🎭

## 📖 Usage

### Automatic Analysis
- **On Save**: Files are automatically analyzed when you save them
- **Real-time Feedback**: Issues appear in the Problems panel and inline in your editor
- **Visual Indicators**: Different colors for different severity levels

### Manual Analysis
- **Current File**: Right-click → "🗑️ Roast This File"
- **Entire Workspace**: Command Palette → "🔥 Roast Entire Workspace"
- **Quality Score**: Command Palette → "📊 Show Quality Score"
- **Educational Advice**: Command Palette → "🎓 Show Educational Advice"
- **Clear Results**: Command Palette → "🧹 Clear All Roasts"

### Inline Messages (ErrorLens-style)
Issues are displayed inline next to your code with:
- 🔴 **Nuclear** issues (errors) - Red text
- 🟠 **Spicy** issues (warnings) - Orange text
- 🔵 **Mild** issues (info) - Blue text

## ⚙️ Configuration

Access settings via: File → Preferences → Settings → Extensions → Garbage Code Hunter

```json
{
  // Enable/disable real-time analysis
  "garbageHunter.enableRealTimeAnalysis": true,

  // Language for roast messages
  "garbageHunter.language": "en-US", // en-US, zh-CN, auto

  // Show inline messages like ErrorLens
  "garbageHunter.showInlineMessages": true,

  // Maximum length of inline messages
  "garbageHunter.maxInlineMessageLength": 100,

  // File patterns to exclude from analysis
  "garbageHunter.excludePatterns": [
    "**/target/**",
    "**/node_modules/**",
    "**/.git/**"
  ],

  // LLM-powered roasts (requires Ollama or OpenAI-compatible API)
  "garbageHunter.llm.enabled": false,
  "garbageHunter.llm.provider": "ollama",
  "garbageHunter.llm.model": "gemma4:e2b",
  "garbageHunter.llm.endpoint": "",
  "garbageHunter.llm.apiKey": ""
}
```

## 🤖 LLM Integration

Enable LLM-powered roast generation for creative, context-aware feedback:

1. Install and start [Ollama](https://ollama.com):
   ```bash
   ollama pull gemma4:e2b
   ```

2. Enable LLM in VS Code settings:
   ```json
   {
     "garbageHunter.llm.enabled": true,
     "garbageHunter.llm.model": "gemma4:e2b"
   }
   ```

3. Use `--markdown` mode in the CLI for best LLM roast display

**Supported providers:**
- Ollama (default: `http://localhost:11434`)
- Any OpenAI-compatible API (LM Studio, vLLM, etc.)

## 🎯 Detection Features

### 📝 Naming Issues
- Meaningless variable names (`foo`, `bar`, `data`, `temp`)
- Hungarian notation (`strName`, `intCount`)
- Excessive abbreviations (`mgr`, `ctrl`, `usr`)

### 🔧 Code Complexity
- Deep nesting (>5 levels)
- Long functions
- God functions doing too much

### 🦀 Rust-Specific Issues
- Unwrap abuse
- Unnecessary clones
- String vs &str misuse

### 🎓 Student Code Patterns
- Printf debugging (`println!` everywhere)
- Panic abuse
- TODO comment overload

## 🔧 Commands

| Command | Description |
|---------|-------------|
| `garbageHunter.analyzeFile` | 🗑️ Roast This File |
| `garbageHunter.analyzeWorkspace` | 🔥 Roast Entire Workspace |
| `garbageHunter.clearDiagnostics` | 🧹 Clear All Roasts |
| `garbageHunter.showScore` | 📊 Show Quality Score |
| `garbageHunter.showEducational` | 🎓 Show Educational Advice |

## 🎭 Example Roasts

```rust
// Your code:
let data = get_user_info();

// Garbage Hunter says:
🗑️ Variable name 'data' is more meaningless than my existence
```

```rust
// Your code:
user.unwrap().name.unwrap()

// Garbage Hunter says:
🗑️ This unwrap() chain is more dangerous than a toddler with scissors
```

### LLM-Generated Roasts (with Ollama)

```
🗑️ Nesting so deep, trying to dig to the Earth's core? That's not structure, that's a geological fault line.
🗑️ Variable 'value' - congrats on inventing the most meaningless identifier
🗑️ Copy-paste ninja detected! 23 identical lines found. You've duplicated logic instead of abstracting it.
```

## 🐛 Known Issues

- Large workspaces may take some time to analyze
- Requires the CLI tool to be installed separately
- Analysis is currently limited to Rust files only
- LLM roasts require a running Ollama instance or OpenAI-compatible API

## 🔄 Release Notes

### 0.2.0
- Added LLM-powered roast generation (Ollama / OpenAI-compatible)
- Added quality score display command
- Added educational advice command
- Updated all comments to English
- Updated license to Apache 2.0
- Updated dependencies to latest versions

### 0.1.0
- Initial release
- Real-time analysis on file save
- Inline message display (ErrorLens-style)
- Workspace analysis
- Configurable language and display options
- Problems panel integration

## 📄 License

Apache License 2.0 - see [LICENSE](https://github.com/TimWood0x10/garbage-code-hunter/blob/main/LICENSE) for details.

---

**Enjoy roasting your code!** 🗑️🔥

*Remember: We roast your code, not you. It's all about learning and having fun while writing better code!*
