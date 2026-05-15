# Release v0.2.1

## 🎉 Major Feature Update

This release brings significant architectural improvements and exciting new features to Garbage Code Hunter.

## 🚀 Key Changes

### Core Architecture
- **Migrated from syn to tree-sitter** for multi-language AST parsing
  - Now supports 11 languages: Rust, C, C++, Python, JavaScript, TypeScript, Go, Java, Ruby, Swift, Zig
  - More accurate and faster parsing
  - Better cross-language support

### New Analysis Tools (8 added)
1. **Badge Generator** - Generate SVG score badges for README
2. **PR Title Hunter** - Analyze PR titles (local git + GitHub API support)
3. **Trend Tracking** - Track quality score history over time
4. **Last Words** - Find TODO/FIXME/HACK comments and their age
5. **Debt Invoice** - Generate technical debt cost estimates
6. **Personality Profiler** - Analyze developer coding style
7. **Radar Chart** - Generate code smell radar visualization (SVG)
8. **Team Roast** - Per-developer analysis and roasting

### Enhancements
- **GitHub Action** - Automatic PR review with roast comments
- **VSCode Extension** - Hover provider and inline decorations
- **LLM Integration** - Enhanced prompts with 9-language personas
- **Jaccard Similarity** - Cross-file near-duplicate detection
- **C++ Allocation Detection** - Framework-specific alloc patterns (curlx_malloc, zmalloc, ngx_alloc)
- **Context-Aware Analysis** - Reduced false positives for test/example code

### Refactoring
- Extracted shared modules (Severity, OutputFormat, scoring helpers)
- Migrated duplication detection to tree-sitter
- Removed legacy rule files
- Updated CI/CD workflows with better testing

### Bug Fixes
- Fixed i18n display issues (proper Chinese/English separation)
- Fixed CI bootstrap command to use correct subcommand
- Improved score formatting and alignment

## 📊 Stats
- **95 files changed**
- **17,025 insertions**, 8,928 deletions
- **252 tests passing**
- **12 languages supported**

## 🔧 Installation

```bash
cargo install garbage-code-hunter
```

## 📝 Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for detailed changes.

---

**Previous Release**: v0.2.0
**Release Date**: 2026-05-15
