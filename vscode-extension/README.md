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
- 🔧 **Highly Configurable**: Customize roast intensity, language, and display options

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
  
  // Roast intensity level
  "garbageHunter.roastIntensity": "sarcastic", // gentle, sarcastic, savage
  
  // Language for roast messages
  "garbageHunter.language": "en-US", // en-US, zh-CN
  
  // Show inline messages like ErrorLens
  "garbageHunter.showInlineMessages": true,
  
  // Maximum length of inline messages
  "garbageHunter.maxInlineMessageLength": 100,
  
  // File patterns to exclude from analysis
  "garbageHunter.excludePatterns": [
    "**/target/**",
    "**/node_modules/**",
    "**/.git/**"
  ]
}
```

## 🎯 Detection Features

The extension detects various "garbage code" patterns:

### 📝 Naming Issues
- Meaningless variable names (`foo`, `bar`, `data`, `temp`)
- Hungarian notation (`strName`, `intCount`)
- Excessive abbreviations (`mgr`, `ctrl`, `usr`)

### 🔧 Code Complexity
- Deep nesting (>3 levels)
- Long functions
- God functions doing too much

### 🦀 Rust-Specific Issues
- Unwrap abuse
- Unnecessary clones
- String vs &str misuse
- Iterator pattern violations

### 🎓 Student Code Patterns
- Printf debugging (`println!` everywhere)
- Panic abuse
- TODO comment overload

## 🎨 Screenshots

### Real-time Analysis
![Real-time analysis](images/realtime-analysis.png)

### Inline Messages
![Inline messages](images/inline-messages.png)

### Problems Panel
![Problems panel](images/problems-panel.png)

### Workspace Analysis
![Workspace analysis](images/workspace-analysis.png)

## 🔧 Commands

| Command | Description |
|---------|-------------|
| `garbageHunter.analyzeFile` | 🗑️ Roast This File |
| `garbageHunter.analyzeWorkspace` | 🔥 Roast Entire Workspace |
| `garbageHunter.clearDiagnostics` | 🧹 Clear All Roasts |

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

## 🐛 Known Issues

- Large workspaces may take some time to analyze
- Requires the CLI tool to be installed separately
- Analysis is currently limited to Rust files only

## 🔄 Release Notes

### 0.1.0
- Initial release
- Real-time analysis on file save
- Inline message display (ErrorLens-style)
- Workspace analysis
- Configurable roast intensity and language
- Problems panel integration

## 🤝 Contributing

Found a bug or have a feature request? 

- 🐛 [Report Issues](https://github.com/TimWood0x10/garbage-code-hunter/issues)
- 💡 [Request Features](https://github.com/TimWood0x10/garbage-code-hunter/issues)
- 🔧 [Contribute Code](https://github.com/TimWood0x10/garbage-code-hunter/pulls)

## 📄 License

MIT License - see [LICENSE](https://github.com/TimWood0x10/garbage-code-hunter/blob/main/LICENSE) for details.

---

**Enjoy roasting your code!** 🗑️🔥

*Remember: We roast your code, not you. It's all about learning and having fun while writing better code!*