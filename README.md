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

| Language | Status | TP Rate | Test Projects | Key Detectors |
|----------|--------|---------|---------------|---------------|
| Rust | Stable | ~90% | Finance, ReChat-server | unwrap/expect, panic, assert, debug, unsafe, naming, nesting, magic, dead_code, duplicate_import, `#[cfg(test)]` aware |
| Go | Stable | ~85% | interchange, gaia, loan, gosec | panic, debug, naming, nesting, goroutine, defer, conventions, unsafe, dead_code, duplicate_import |
| Python | Stable | ~80% | ZK-bulletproofs | except, print, naming, nesting, magic (int+float), wildcard-import, bool-compare, dead_code, duplicate_import |
| Ruby | Stable | ~85% | jekyll, Metric | raise, puts/p/warn, naming, nesting, magic, globals, bare-rescue, dead_code, duplicate_import |
| Java | Stable | ~80% | TestJava.java | throw, println/printStackTrace, naming, nesting, magic, empty-catch, logging frameworks, dead_code, duplicate_import |
| TypeScript | Beta | ~80% | zod, hono, trpc | throw, console/debugger, naming, nesting, magic, any/enum/alias, @ts-ignore, require, dead_code, duplicate_import |
| JavaScript | Beta | ~80% | self-tested | throw, console/debugger, naming, nesting, magic, eval/with/alert/var, dead_code, duplicate_import |
| C | Beta | ~85% | stone-prover | exit/abort/assert, printf, naming, nesting, magic, goto, sizeof, malloc, dead_code, duplicate_import |
| C++ | Beta | ~85% | stone-prover | exit/abort/terminate/throw, cout/cerr, naming, nesting, magic, goto/new, sizeof, malloc, dead_code, duplicate_import |
| Swift | Beta | ~80% | Alamofire, SnapKit, vapor | fatalError/assert, print/NSLog, naming, nesting, magic, try!/as!, dead_code, duplicate_import |
| Zig | Beta | ~90% | ziglings | @panic, @compileLog/warn, naming, nesting, magic, unreachable, dead_code, duplicate_import |

## Performance

Benchmarked on Apple Silicon (M-series), single-file analysis:

| Benchmark | Time |
|-----------|------|
| `create_analyzer` | 539 µs |
| `analyze_file/clean_rust` | 1.30 ms |
| `analyze_file/single_large_file` (100 functions) | 61.5 ms |
| `analyze_path/mixed_4_languages` | 102 ms |
| `analyze_path/10_garbage_files` | 7.50 ms |
| `analyze_path/50_garbage_files` | 37.4 ms |
| `scalability/20_files` | 15.2 ms |
| `scalability/50_files` | 37.4 ms |

Run `cargo bench` to reproduce. Logs are saved in `./benchs/`.

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

Create `.garbage-code-hunter.toml` in your project root to customize analysis. The tool auto-discovers this file by walking up from the working directory.

```bash
# Auto-discovered (no flag needed)
garbage-code-hunter analyze

# Explicit path
garbage-code-hunter analyze --project-config .garbage-code-hunter.toml
```

**Config file name:** `.garbage-code-hunter.toml` (canonical). No other names are supported.

**What you can customize:**

```toml
[whitelists]
magic-numbers = [800, 1000]
variable-names = ["ctx", "db"]
exclude-patterns = ["vendor/*", "third_party/*"]

[rules.magic-number]
enabled = true
allowed-numbers = [3000, 86400]

[rules.unwrap]
threshold = 3

[signals]
panic-addiction = true
naming-chaos = false
```

Full schema: [docs/config-reference.md](docs/config-reference.md)

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
