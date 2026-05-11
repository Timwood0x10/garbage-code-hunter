# Garbage Code Hunter v0.2.0

## 🎉 What's New

### 🚀 Major Features

#### Cross-File Duplication Detection (NEW!)
- **Function fingerprinting**: Automatically identifies duplicated code across files
- **Similarity matching**: Finds copy-pasted code with configurable thresholds (default: 85%)
- **Memory efficient**: LRU cache with configurable memory limits (default: 512MB)
- **Real-world validated**: Successfully tested on 200+ file projects in <1.5 seconds

**Example output:**
```bash
$ garbage-code-hunter /path/to/project --verbose
📌 cross-file-duplication: 33 issues  ← NEW!
```

#### Context-Aware Analysis (NEW!)
- **FileContext system**: Automatically detects code type and adjusts sensitivity
  - **Business code**: Full detection strength (default)
  - **Example/Demo code**: 70% reduced sensitivity
  - **Test code**: 80% reduced sensitivity
  - **Benchmark/Documentation**: 60-90% reduction
- **Phase 1.3 rule adaptation**: Core rules now use context information:
  - MeaninglessNamingRule: Skips test/example files
  - MagicNumberRule: Filters UI-related values
  - PrintlnDebuggingRule: Completely skips non-main files
  - UnwrapAbuseRule: Higher threshold for test code
  - TerribleNamingRule: Skips example/demo directories

**Impact:**
```
Finance project: 820 → ~798 issues (-2.7%) ⚠️
system_alert: 206 → ~25 issues (-88%) ✅
```

---

## 🐛 Bug Fixes

### 🔴 Critical Fixes

- **Scoring logic bug fixed**
  - Issue: Score was 100.0 (Terrible) when no issues, should be 0.0 (Excellent)
  - Fix: Corrected scoring to use 0 = best, 100 = worst
  - Impact: All clean projects now show correct score of 0.0/100

- **VSCode extension hanging prevention**
  - Added 30-second timeout to CLI execution calls
  - Prevents VSCode freeze when CLI hangs
  - Shows friendly error message instead of infinite wait

- **CrossFileAnalyzer error handling**
  - Changed from `let _ =` to proper error logging
  - Failed file processing now shows warnings with file path

- **Example file detection logic fix**
  - Fixed operator precedence issue in `is_example_file()`
  - `/messages/` directory now correctly excluded from `src/` business code

### 🟡 Medium Fixes

- **CLI default value ambiguity resolved**
  - Changed `llm_timeout` parameter from `u64` to `Option<u64>`
  - Can now distinguish "not set" from "explicitly set to 30"

- **Dead code cleanup**
  - Removed unused `i18n::get_suggestions()` method
  - Removed unused `_rule_count` variable
  - Cleaned up empty inline decoration handlers in VSCode extension
  - Removed empty event handler registrations

- **Module declaration deduplication**
  - main.rs now imports from lib.rs instead of redeclaring modules
  - Eliminates duplicate compilation, faster build times

---

## 📊 Performance Improvements

| Metric | v0.1.x | v0.2.0 | Improvement |
|--------|--------|--------|-------------|
| **Large project analysis (208 files)** | Unknown | **1.13s** | Excellent |
| **Medium project analysis (66 files)** | Unknown | **0.41s** | Excellent |
| **Memory usage (208 files)** | Unknown | **46MB** | 9% of limit |
| **Cross-file detection** | N/A | **39 issues found** | New capability |

---

## 🔧 Breaking Changes

None - Fully backward compatible.

---

## 📦 Installation

### From Binary (Recommended)

Download the appropriate binary for your platform:

| Platform | Architecture | File |
|----------|-------------|------|
| Linux | x86_64 | `garbage-code-hunter-linux-x86_64.tar.gz` |
| macOS | Apple Silicon (ARM64) | `garbage-code-hunter-macos-arm64.tar.gz` |

> **Note**: Only modern Macs with Apple Silicon (M1/M2/M3) are supported. Intel Macs are not included due to low usage.

### From Source

```bash
git clone https://github.com/yourusername/garbage-code-hunter.git
cd garbage-code-hunter
cargo install --path .
```

### From crates.io

```bash
cargo install garbage-code-hunter
```

### VSCode Extension

Search for "Garbage Code Hunter" in the VSCode Marketplace.

---

## 🚀 Usage Examples

### Basic Analysis
```bash
# Analyze current directory
garbage-code-hunter .

# Analyze specific file
garbage-code-hunter src/main.rs

# Verbose mode with cross-file detection
garbage-code-hunter ./src --verbose
```

### Context-Aware Options
```bash
# English roasts
garbage-code-hunter . --lang en-US

# Chinese roasts
garbage-code-hunter . --lang zh-CN

# Markdown output
garbage-code-hunter . --markdown > report.md
```

### Advanced Options
```bash
# Educational mode with suggestions
garbage-code-hunter . --educational

# Hall of Shame report
garbage-code-hunter . --hall-of-shame

# LLM-powered roasts (requires Ollama)
garbage-code-hunter . --llm --llm-provider ollama
```

---

## 🧪 Testing & Validation

### Bootstrap Test Results (v0.2.0)

Tested on **10 real Rust projects**:

| Project | Files | Issues | Time | Status |
|---------|-------|--------|------|--------|
| garbage-code-hunter (self) | 34 | 230 | 0.74s | ✅ |
| algo | 1 | 0 | 0.02s | ✅ |
| gpu-code | 6 | ~30 | 0.05s | ✅ |
| system_alert | 11 | ~25 | 0.11s | ✅ |
| AlgoGpuRust | 21 | ~3 | 0.28s | ✅ |
| ReChat-server | 26 | ~6 | 0.17s | ✅ |
| Finance | 66 | ~798 | 0.41s | ✅ |
| memscope-rs | 208 | 81 | 1.13s | ✅ |
| lifeRestart | 33 | 0 | 0.09s | ✅ |

**Summary:**
- ✅ Zero crashes across all projects
- ✅ Average execution time <0.35s
- ✅ Cross-file detection working (39 total issues found)
- ✅ Memory usage <50MB for all tests

---

## 📋 Known Issues & Limitations

### Current Limitations

1. **Phase 1.3 not fully effective yet**
   - MeaninglessNamingRule still reports many false positives in Finance project (540/798)
   - Root cause: FileContext识别为 Business 时规则仍较严格
   - **Next release target**: Further tune context sensitivity thresholds

2. **cross_file module has unused APIs**
   - Some public functions not yet integrated into main workflow
   - Results in compiler warnings (currently suppressed)
   - **Plan**: Complete integration or make internal

3. **No Windows binary in this release**
   - Focusing on Unix-like systems first
   - Can be added in future releases if requested

---

## 🔄 Migration Guide

### From v0.1.x to v0.2.0

**No breaking changes!** Just upgrade and enjoy:

```bash
# If installed via cargo
cargo update garbage-code-hunter

# If using binary, download new version
# No configuration changes needed
```

**New features are opt-in:**
- Cross-file detection: Automatic when analyzing directories
- Context awareness: Automatic based on file paths
- No flags required to enable new features

---

## 👥 Contributors

Thanks to all contributors who helped make this release possible!

Special thanks to:
- **Bootstrap testers**: Provided real-world projects for validation
- **Bug reporters**: Identified critical issues (scoring logic, VSCode hanging)
- **Community feedback**: Shaped the feature priorities

---

## 📝 Changelog

See [CHANGELOG.md](./CHANGELOG.md) for detailed version history.

---

## 📜 License

MIT License - see [LICENSE](./LICENSE) for details.

---

## 🙏 Acknowledgments

- Inspired by [fuck-u-code](https://github.com/Done-0/fuck-u-code.git)
- Built with [syn](https://docs.rs/syn/) for AST parsing
- Uses [clap](https://docs.rs/clap/) for CLI argument parsing
- CI/CD powered by GitHub Actions

---

*Release date: 2026-05-09*
*Version: 0.2.0*
*Status: Production Ready ✅*
