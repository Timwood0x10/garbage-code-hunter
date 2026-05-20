# Garbage Code Hunter Documentation

Garbage Code Hunter is a CLI toolkit that analyzes code quality with a sarcastic tone.
It is built for entertainment and code-taste inspection, not for bug finding or security analysis.

> Inspiration: [fuck-u-code](https://github.com/Done-0/fuck-u-code.git)

## At a glance

- Scans source trees with tree-sitter where available
- Uses StyleIR, a language-neutral style facts layer, as the core evidence model
- Supports 11 languages: Rust, Go, Python, JavaScript, TypeScript, Java, C, C++, Ruby, Swift, and Zig
- Ships multiple roast-style tools for code, commits, dependencies, PR titles, trends, and more
- Supports local roast messages and optional LLM-powered output

## StyleIR

StyleIR is the main architectural improvement. Instead of making every detector reason directly over every language AST, adapters extract a compact set of style facts first: functions, line counts, naming violations, nesting, debug calls, panic-prone calls, magic numbers, TODOs, duplicate imports, unsafe blocks, and language-specific counters.

This gives reports, scoring, JSON output, and future detectors a shared evidence layer while keeping language-specific parsing isolated in adapters.

## What it detects

- Weak naming such as `data`, `tmp`, `foo`, and `bar`
- Magic numbers and repeated constants
- Deep nesting and overly long functions
- God functions that do too much
- Leftover debug prints such as `println` and `fmt.Println`
- Commented-out code and stale TODO/FIXME/HACK comments
- Duplication across files

## Installation

```bash
cargo install garbage-code-hunter
```

## Quick start

```bash
# Analyze the current directory
garbage-code-hunter analyze

# Analyze a specific path and language
garbage-code-hunter analyze ./my-go-project --lang go

# Run a full scan across all tools
garbage-code-hunter scan ./my-project
```

## Main commands

| Command | Alias | Description |
|---|---|---|
| `analyze` | - | Core code taste analysis |
| `commit-roaster` | `cr` | Roast commit messages from git history |
| `deps-shamer` | `ds` | Inspect dependency hygiene |
| `pr-title-hunter` | `pr` | Roast PR titles from GitHub or local data |
| `scan` | - | Run all tools and combine their scores |
| `badge` | - | Generate an SVG score badge |
| `trend` | - | Show score history over time |
| `last-words` | `lw` | Find stale TODO/FIXME/HACK comments |
| `debt-invoice` | `debt` | Estimate technical debt cost |
| `personality` | - | Infer developer personality from code patterns |
| `decay` | - | Analyze quality decay over git history |
| `autopsy` | - | Produce a root-cause style report |
| `radar` | - | Generate a code-smell radar view |
| `ci-bot` | - | Generate CI-style review comments |
| `persona` | - | Roast code with a chosen persona |
| `danger-zone` | `dz` | Find the riskiest files |
| `team-roast` | - | Summarize quality by author |

## Common examples

```bash
# Output JSON
garbage-code-hunter analyze -f json

# Restrict scan results
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"

# Save a scan result
garbage-code-hunter scan --save

# Generate a badge
garbage-code-hunter badge --output badge.svg
```

## Configuration

- Project config: `.garbage-code-hunter.toml`
- Search order: current directory, then parent directories
- Main config docs: [`configuration.md`](configuration.md)
- Rule reference: [`rules.md`](rules.md)
- Tool reference: [`tools.md`](tools.md)

## Output formats

Most commands support `terminal`, `text`, `json`, or `markdown` output depending on the tool.
Several analysis commands also accept a `--lang` flag, defaulting to `en-US` for localized roast text.

## Notes

- `analyze` supports `--harsh`, `--educational`, `--hall-of-shame`, `--suggestions`, and LLM-related flags.
- `scan` accepts `--lang`, `--format`, `--save`, and `--project-config`.
- `badge` can use a manually supplied score or compute one from the target path.
- `persona` supports preset personas such as `linux-kernel`, `silicon-valley`, `japanese-enterprise`, and `rust-fanatic`.
