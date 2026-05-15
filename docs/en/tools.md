# Tools Guide

Garbage Code Hunter ships 15+ analysis tools. Here's what each one does.

---

## `analyze` — Code Taste Analysis

The core tool. Scans source files using tree-sitter AST parsing and reports code taste issues.

```bash
# Analyze current directory
garbage-code-hunter analyze

# Analyze specific path with language filter
garbage-code-hunter analyze ./src --lang rust

# Multiple exclude patterns
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"
```

**Output includes:**
- Issue statistics (Nuclear/Spicy/Mild)
- Quality score (0-100)
- Per-file issue breakdown
- Category scores (naming, complexity, duplication, etc.)

---

## `commit-roaster` — Commit Message Roast

Analyzes git commit history and roasts bad commit messages.

```bash
garbage-code-hunter commit-roaster
garbage-code-hunter commit-roaster --limit 50  # Last 50 commits
```

**Detects:**
- Empty messages, too short (< 5 chars)
- "WIP", "fix", "update" with no context
- Keyboard mashing ("asdfgh", "test test test")
- All caps or all lowercase
- Merge commit spam

---

## `deps-shamer` — Dependency Shame

Analyzes project dependencies and shames bad practices.

```bash
garbage-code-hunter deps-shamer
```

**Supports:** Cargo (Rust), npm (JS/TS), pip (Python), Go modules, Maven (Java)

**Detects:**
- Too many dependencies
- Pre-release versions in production
- Git dependencies
- Outdated or deprecated packages

---

## `pr-title-hunter` — PR Title Quality

Roasts low-quality PR titles from local branches or GitHub.

```bash
garbage-code-hunter pr-title-hunter
garbage-code-hunter pr-title-hunter --repo owner/repo  # GitHub PRs
```

---

## `scan` — Full Scan

Runs ALL tools in parallel and produces a combined score.

```bash
garbage-code-hunter scan ./my-project
```

---

## `last-words` — TODO/FIXME Scanner

Finds legacy TODO/FIXME/HACK/BUG comments and reports how long they've been sitting there.

```bash
garbage-code-hunter last-words
```

---

## `debt-invoice` — Technical Debt Invoice

Generates a "technical debt invoice" with estimated maintenance costs.

```bash
garbage-code-hunter debt-invoice
```

---

## `personality` — Developer Personality

Analyzes code patterns to determine developer personality type.

```bash
garbage-code-hunter personality
```

**Personalities:** Copy-Paste Artist, Unwrap Enthusiast, TODO Dreamer, etc.

---

## `danger-zone` — Dangerous Files

Identifies the most dangerous files in the codebase (highest issue density).

```bash
garbage-code-hunter danger-zone
```

---

## `team-roast` — Team Analysis

Per-developer analysis based on git blame.

```bash
garbage-code-hunter team-roast
```

---

## `radar` — Radar Chart

Generates an SVG radar chart showing code smell distribution.

```bash
garbage-code-hunter radar
```

---

## `autopsy` — Code Autopsy

Generates a code autopsy report with root cause analysis.

```bash
garbage-code-hunter autopsy
```

---

## `decay` — Quality Decay

Shows how project quality has changed over git history.

```bash
garbage-code-hunter decay
```

---

## `ci-bot` — CI Bot

Generates a CI-style PR review comment (for GitHub Actions integration).

```bash
garbage-code-hunter ci-bot
```

---

## `persona` — Persona Analysis

Analyzes code with a specific roast personality.

```bash
garbage-code-hunter persona --style senior
garbage-code-hunter persona --style intern
```
