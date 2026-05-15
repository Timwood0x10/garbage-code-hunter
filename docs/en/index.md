# Garbage Code Hunter Documentation

A humorous code quality detector that roasts your garbage code with style.

> **Inspiration**: [fuck-u-code](https://github.com/Done-0/fuck-u-code.git)

## What is this?

Garbage Code Hunter is a CLI toolkit for code taste analysis. Unlike traditional linters that give you dry warnings, we tell you how bad your code is in a **sarcastic, witty, and brutally honest** way.

This is NOT a static bug detector. It finds:
- Bad naming (`data`, `info`, `tmp`, `foo`, `bar`)
- Magic numbers scattered everywhere
- Deeply nested code mazes
- God functions that do everything
- `println`/`fmt.Println` debugging left behind
- Commented-out code hoarded like ex's photos
- TODO comments that will never be done
- Copy-pasted functions across files

## Supported Languages

Rust, Go, Python, JavaScript, TypeScript, Java, C, C++, Ruby, Swift, Zig (11 total)

## Quick Start

```bash
# Install
cargo install garbage-code-hunter

# Analyze current directory
garbage-code-hunter analyze

# Analyze a specific language project
garbage-code-hunter analyze ./my-go-project --lang go

# Full scan with all tools
garbage-code-hunter scan ./my-project
```

## Tool Collection

| Tool | Command | What it does |
|------|---------|-------------|
| **Code Hunter** | `analyze` | Static analysis: naming, nesting, duplication |
| **Commit Roaster** | `commit-roaster` | Roast bad commit messages |
| **Deps Shamer** | `deps-shamer` | Shame bad dependency practices |
| **PR Title Hunter** | `pr-title-hunter` | Roast low-quality PR titles |
| **Full Scan** | `scan` | Run all tools, get combined score |
| **Last Words** | `last-words` | Find TODO/FIXME/HACK comments |
| **Debt Invoice** | `debt-invoice` | Generate technical debt invoice |
| **Personality** | `personality` | Analyze developer personality |
| **Danger Zone** | `danger-zone` | Identify most dangerous files |
| **Team Roast** | `team-roast` | Per-developer analysis |
| **Radar** | `radar` | Code smell radar chart (SVG) |
| **Autopsy** | `autopsy` | Code autopsy report |
| **Decay** | `decay` | Quality decay over git history |
| **CI Bot** | `ci-bot` | CI-style PR review comment |
| **Persona** | `persona` | Analyze with specific personality |

## Documentation

- [Rules Reference](rules.md) — All detection rules with language coverage
- [Tools Guide](tools.md) — Detailed tool documentation
- [Configuration](configuration.md) — Config file options

## Real-World Results

### Go project (interchange, ~47K lines)

```
Issue Statistics:
  46 Nuclear | 396 Spicy | 12,332 Mild | 12,774 Total

Top issues: magic-number, single-letter, code-duplication
Top files: tx.pulsar.go (1250), tx.pb.go (1129) ← generated files
```

### Rust project (ReChat-server)

```
Issue Statistics:
  0 Nuclear | 34 Spicy | 2,103 Mild | 2,137 Total

Score: 1.1/100 — Excellent
Top issues: cross-file-near-duplicate, println-debugging
```

### Zig project (ziglings)

```
Issue Statistics:
  0 Nuclear | 18 Spicy | 6,101 Mild | 6,119 Total

Top issues: magic-number (358), commented-code (102), single-letter (80)
```
