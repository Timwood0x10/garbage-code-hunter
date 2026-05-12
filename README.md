# 🗑️ Garbage Code Hunter

[![CI/CD](https://github.com/yourusername/garbage-code-hunter/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/garbage-code-hunter/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/garbage-code-hunter.svg)](https://crates.io/crates/garbage-code-hunter)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [中文](./README_zh.md)

A humorous Rust code quality detector that roasts your garbage code with style!

> **Inspiration**: https://github.com/Done-0/fuck-u-code.git

## What is this?

Garbage Code Hunter is a Rust static analysis tool. Unlike traditional linters that give you dry warnings, we tell you how bad your code is in a **sarcastic, witty, and brutally honest** way.

Think of it as a code reviewer who isn't afraid to hurt your feelings (but it's for your own good).

## What can it do?

- 🔍 **Code analysis**: Bad naming, deep nesting, long functions, unwrap abuse...
- 🔗 **Cross-file duplication detection**: Find copy-pasted code across files
- 🔥 **Commit Roaster** (`commit-roaster` / `cr`): Roast bad commit messages from git history
- 📦 **Deps Shamer** (`deps-shamer` / `ds`): Shame bad dependency practices (Rust, Node, Python, Go)
- 🏷️ **PR Title Hunter** (`pr-title-hunter` / `pr`): Roast low-quality PR titles from merge commits
- 🔎 **Full Scan** (`scan`): Run all tools and get a combined garbage score
- 🎯 **Context-aware analysis**: Automatically adjusts sensitivity for tests/examples/UI code
- 🗣️ **Savage roasts**: Every warning comes with a humorous roast to make you laugh while fixing
- 📊 **Quality scoring**: 0-100 scale per tool, with severity-weighted penalties
- 🌍 **Bilingual**: Supports both English and Chinese roasts
- 🤖 **LLM powered**: Connect to Ollama for even more creative roasts
- 🔌 **VSCode extension**: Real-time roasting in your editor

## Quick Start

### Install

```bash
cargo install garbage-code-hunter
```

### Subcommands

#### Code Analysis (default)
```bash
garbage-code-hunter                    # Analyze current directory
garbage-code-hunter src/main.rs        # Analyze specific file
garbage-code-hunter --lang zh-CN src/  # Chinese roasts
garbage-code-hunter --markdown src/    # Markdown report for AI tools
```

#### Commit Roaster
Scan git history and roast bad commit messages:
```bash
garbage-code-hunter commit-roaster              # Last 50 commits
garbage-code-hunter cr --limit 100              # Last 100 commits
garbage-code-hunter cr --author "john" --since 2024-01-01
garbage-code-hunter cr -f json                  # JSON output
```

#### Deps Shamer
Analyze project dependencies and shame bad practices:
```bash
garbage-code-hunter deps-shamer          # Current directory
garbage-code-hunter ds /path/to/project  # Specific project
garbage-code-hunter ds -f json           # JSON output
```

Supported ecosystems: Cargo.toml, package.json, go.mod, requirements.txt, pyproject.toml.

#### PR Title Hunter
Scan merge commits and roast bad PR titles:
```bash
garbage-code-hunter pr-title-hunter       # Last 50 merge commits
garbage-code-hunter pr --limit 200        # Last 200 merge commits
garbage-code-hunter pr -f json            # JSON output
```

#### Full Scan
Run all tools and get a combined garbage score:
```bash
garbage-code-hunter scan                  # Run everything
garbage-code-hunter scan /path/to/project # Specific project
garbage-code-hunter scan -f json          # JSON output
```

### Output Formats

All subcommands support `terminal` (default, colored) and `json` output:
```bash
garbage-code-hunter cr -f json | jq '.score'
garbage-code-hunter ds -f json | jq '.issues | length'
```

## Example Output

### Commit Roaster
```
🔥 Commit Roast Report 🔥
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Scanned 50 commits, found 12 issues

💀 Critical (2)
  • abc1234 "" — The commit message is empty. Were you sleepwalking?
  • def5678 "asdf" — Keyboard mashing is not a commit strategy.

🔴 High (5)
  • ghi9012 "fix" — Fix WHAT? 'fix' is not a description, it's a cry for help.
  ...

📊 Score: 76/100 (B 👍)
```

### Deps Shamer
```
📦 Dependency Shame Report 📦
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  📂 Rust/Cargo: 45 dependencies

🔴 High (2)
  • Version '*' for 'tokio' — enjoy your daily breaking changes? [tokio]
  • 'failure' is deprecated — are you an archaeologist? [failure]

📊 Dependency Health Score: 88/100
```

### Full Scan
```
🔎 Running Full Garbage Scan...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✅ code-hunter: 72/100 (23 issues in 15 files)
  ✅ commit-roaster: 85/100 (50 commits analyzed)
  ✅ deps-shamer: 90/100 (45 dependencies)
  ✅ pr-title-hunter: 95/100 (30 PRs checked)

📦 Garbage Report 📦
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Tool Summary
  ────────────────────────────────
  code-hunter          72/100  (23 items)
  commit-roaster       85/100  (50 items)
  deps-shamer          90/100  (45 items)
  pr-title-hunter      95/100  (30 items)

  🏆 Overall Garbage Score: 86/100
```

## Tools Detail

### Commit Roaster Rules
Detects: empty messages, single-word commits, WIP commits, "fix/update/change" only, keyboard mashing, ALL CAPS, excessive exclamation marks, and more. Rules are configurable via TOML.

### Deps Shamer Rules
Detects: too many dependencies (>50), git-based deps, wildcard versions, pre-release in production, deprecated packages (per-ecosystem), duplicate deps, too many dev deps, too many optional deps.

### PR Title Hunter Rules
Detects: empty titles, too short (<5 chars), generic titles ("fix", "update"), ticket-only titles ("PROJ-123"), WIP/Draft, excessive exclamation marks, ALL CAPS, keyboard mashing, lowercase start (skips conventional commits).

## VSCode Extension

Get real-time roasting in VSCode:

1. Install the `garbage-code-hunter` CLI
2. Search "Garbage Code Hunter" in VSCode marketplace
3. Analysis triggers automatically when you save Rust files

## License

Apache License 2.0

---

**Remember**: We roast the code, not you. Let's make code reviews a bit more fun! 🚀
