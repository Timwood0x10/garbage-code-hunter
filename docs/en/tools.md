# Tools Guide

This project ships a broad set of roast-style tools. Most commands accept a `-f/--format` flag and a `--lang` flag where localized text is relevant.

## `analyze`

Core source analysis command.

```bash
garbage-code-hunter analyze
```

Useful flags:
- `--harsh`
- `--verbose`
- `--top`
- `--issues`
- `--summary`
- `--brief`
- `--markdown`
- `--lang`
- `--exclude`
- `--educational`
- `--project-config`
- `--hall-of-shame`
- `--suggestions`
- `--format`
- `--llm`
- `--llm-provider`
- `--llm-endpoint`
- `--llm-model`
- `--llm-api-key`
- `--llm-timeout`
- `--config`

## `commit-roaster` / `cr`

Analyzes git commit history and roasts weak commit messages.

```bash
garbage-code-hunter commit-roaster --limit 50
```

Flags: `--limit`, `--author`, `--since`, `--until`, `--branch`, `--format`

## `deps-shamer` / `ds`

Inspects dependency files for risky or messy dependency choices.

Flags: `--format`

## `pr-title-hunter` / `pr`

Roasts PR titles from local data or GitHub.

Flags: `--limit`, `--repo`, `--state`, `--token`, `--author`, `--format`

## `scan`

Runs the main tool suite and combines the results.

Flags: `--lang`, `--format`, `--save`, `--project-config`

## `badge`

Generates an SVG badge.

Flags: `--output`, `--style`, `--score`, `PATH`

## `trend`

Shows score history.

Flags: `--last`, `--format`

## `last-words` / `lw`

Finds stale TODO/FIXME/HACK comments.

Flags: `--age`, `--format`, `--lang`

## `debt-invoice` / `debt`

Estimates technical debt cost.

Flags: `--format`, `--lang`

## `personality`

Infers developer personality from code patterns.

Flags: `--format`, `--lang`

## `decay`

Analyzes quality decay over git history.

Flags: `--format`, `--lang`

## `autopsy`

Produces a root-cause style quality report.

Flags: `--format`, `--lang`

## `radar`

Generates a smell radar chart or terminal view.

Flags: `--format`, `--lang`, `--output`

## `ci-bot`

Produces CI-style review output.

Flags: `--format`, `--lang`

## `persona`

Applies a chosen roast persona.

Flags: `--persona`, `--format`, `--lang`

Common personas include `linux-kernel`, `silicon-valley`, `japanese-enterprise`, and `rust-fanatic`.

## `danger-zone` / `dz`

Highlights the highest-risk files.

Flags: `--format`, `--lang`

## `team-roast`

Groups findings by contributor.

Flags: `--limit`, `--format`, `--lang`
