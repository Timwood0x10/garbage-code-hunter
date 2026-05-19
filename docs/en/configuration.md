# Configuration

Garbage Code Hunter reads project-level settings from `.garbage-code-hunter.toml`.
The file is discovered by walking upward from the target path.

## Project config example

```toml
[project-type]
web-service = {}

[whitelists]
magic-numbers = [8080, 443, 80, 3000, 5000]
variable-names = ["ctx", "req", "res", "err", "db", "wg", "mu"]
directories = ["vendor/", "generated/", "testdata/"]
exclude-patterns = [
  "*.pb.go",
  "*.pulsar.go",
  "*_grpc.pb.go",
  "*.gen.ts",
  "*.generated.*",
  "node_modules/",
  "venv/",
  "__pycache__/",
  "vendor/",
]

[rules.naming]
enabled = true
severity = "mild"
allowed-names = ["id", "ok", "tx", "rx", "fs"]

[rules.unwrap]
enabled = true
threshold = 1
nuclear-threshold = 15

[rules.magic-number]
enabled = true
allowed-numbers = [200, 201, 204, 400, 401, 403, 404, 500]

[rules.println]
enabled = true

[[overrides]]
pattern = "generated/"
```

## Configuration sections

- `project-type`: optional project hint for analysis context
- `whitelists.magic-numbers`: numbers that should not be flagged
- `whitelists.variable-names`: variable names that are allowed
- `whitelists.directories`: directories that reduce sensitivity
- `whitelists.exclude-patterns`: files or paths to skip
- `rules.*`: per-rule switches and thresholds
- `signals.*`: detector configuration for special signal checks
- `overrides`: path-based overrides for generated or special folders

## Severity levels

| Level | Weight | Meaning |
|---|---:|---|
| `mild` | 0.5 | Low-priority issue |
| `spicy` | 1.5 | Should be fixed |
| `nuclear` | 3.0 | Fix immediately |

## CLI flags

```bash
# Analyze a path
garbage-code-hunter analyze [PATH]

# Filter by language
garbage-code-hunter analyze --lang go

# Exclude paths
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"

# Save a scan result
garbage-code-hunter scan --save

# Output format
garbage-code-hunter analyze --format json
garbage-code-hunter analyze --format markdown
```

## LLM mode

The application can switch to LLM-backed roast generation.
Use `--llm` together with `--llm-provider`, `--llm-endpoint`, `--llm-model`, `--llm-api-key`, and `--llm-timeout`.
The supported providers currently include `ollama` and generic OpenAI-style endpoints.

Example:

```bash
garbage-code-hunter analyze --llm --llm-provider ollama
```
