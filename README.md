# Garbage Code Hunter

[![CI/CD](https://github.com/TimWood0x10/garbage-code-hunter/actions/workflows/ci.yml/badge.svg)](https://github.com/TimWood0x10/garbage-code-hunter/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/garbage-code-hunter.svg)](https://crates.io/crates/garbage-code-hunter)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-yellow.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust Version](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | [中文](README_zh.md)

A humorous CLI toolkit that roasts code quality, commit messages, dependencies, PR titles, and technical debt.

The key improvement is **StyleIR**: a language-neutral style intermediate representation that extracts objective style facts from parsed source, then lets detectors, scoring, reports, and JSON output share the same evidence layer.

> Inspiration: [fuck-u-code](https://github.com/Done-0/fuck-u-code.git)

## Important

Garbage Code Hunter is an entertainment and code-taste tool. It checks readability, style, maintainability signals, and amusing project health metrics. It is not a bug finder, security scanner, or replacement for linters such as Clippy, ESLint, Pylint, or static analyzers.

## Install

```bash
cargo install garbage-code-hunter
```

## Quick Start

```bash
# Analyze current directory
garbage-code-hunter analyze

# Analyze a project with localized output
garbage-code-hunter analyze ./my-project --lang zh-CN

# Run the full toolkit
garbage-code-hunter scan ./my-project

# JSON output
garbage-code-hunter analyze -f json
```

## Features

### StyleIR core

- Converts parsed source into stable style facts instead of coupling every rule directly to every language AST
- Tracks function counts, god functions, panic-prone calls, naming violations, nesting, debug calls, magic numbers, TODOs, duplicate imports, unsafe blocks, and language-specific issue counters
- Produces JSON-ready summaries for reports, automation, and future rule migration
- Makes cross-language scoring more consistent while preserving language adapters for Rust, Go, Python, Java, Ruby, C/C++, TypeScript, Swift, Zig, and JavaScript

### Tools

| Feature | Command | Alias | Description |
|---|---|---|---|
| Code Hunter | `analyze` | - | Core source analysis for naming, complexity, duplication, debug leftovers, and style smells |
| Commit Roaster | `commit-roaster` | `cr` | Roasts weak commit messages from git history |
| Deps Shamer | `deps-shamer` | `ds` | Checks dependency hygiene across common ecosystems |
| PR Title Hunter | `pr-title-hunter` | `pr` | Roasts low-quality PR titles locally or from GitHub |
| Full Scan | `scan` | - | Runs the tool suite and produces a combined score |
| Badge | `badge` | - | Generates an SVG quality badge |
| Trend | `trend` | - | Shows saved quality score history |
| Last Words | `last-words` | `lw` | Finds stale TODO/FIXME/HACK comments |
| Debt Invoice | `debt-invoice` | `debt` | Estimates technical debt cost |
| Personality | `personality` | - | Infers developer personality from code patterns |
| Decay | `decay` | - | Analyzes project quality decay over git history |
| Autopsy | `autopsy` | - | Produces a root-cause style code autopsy report |
| Radar | `radar` | - | Generates a code-smell radar view or SVG |
| CI Bot | `ci-bot` | - | Produces CI-style review comments |
| Persona | `persona` | - | Roasts code with a selected persona |
| Danger Zone | `danger-zone` | `dz` | Finds the riskiest files in the repository |
| Team Roast | `team-roast` | - | Summarizes quality and debt by contributor |

## Language Support

Rust, Go, Python, JavaScript, TypeScript, Java, C, C++, Ruby, Swift, and Zig.

## Common Usage

```bash
# Exclude noisy files
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"

# Save scan history and view trend
garbage-code-hunter scan --save
garbage-code-hunter trend

# Generate assets
garbage-code-hunter badge --output badge.svg
garbage-code-hunter radar --output radar.svg

# GitHub PR titles
garbage-code-hunter pr --repo owner/repo --state open --token $GITHUB_TOKEN
```

## Configuration

Create `.garbage-code-hunter.toml` in your project root to whitelist names, allowed magic numbers, excluded paths, and rule thresholds.

```bash
garbage-code-hunter analyze --project-config .garbage-code-hunter.toml
```

## Documentation

- English docs: [docs/en/index.md](docs/en/index.md)
- Chinese docs: [docs/zh/index.md](docs/zh/index.md)
- StyleIR details: [src/style_ir/mod.rs](src/style_ir/mod.rs)
- Tools guide: [docs/en/tools.md](docs/en/tools.md)
- Configuration: [docs/en/configuration.md](docs/en/configuration.md)
- Rules reference: [docs/en/rules.md](docs/en/rules.md)

## VSCode Extension

A VSCode extension is available under [vscode-extension](vscode-extension/README.md).

## License

Apache-2.0
