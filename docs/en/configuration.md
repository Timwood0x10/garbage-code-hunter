# Configuration

## Project Config File

Create `.garbage-code-hunter.toml` in your project root. The tool searches upward from the target directory.

### Example

```toml
[project-type]
web-service = {}

[whitelists]
# Numbers that are NOT magic numbers
magic-numbers = [8080, 443, 80, 3000, 5000]

# Variable names that are acceptable
variable-names = ["ctx", "req", "res", "err", "db", "wg", "mu"]

# Directories to reduce sensitivity (still analyzed, but lower weight)
directories = ["vendor/", "generated/", "testdata/"]

# Patterns to completely exclude from analysis
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
# Override settings for generated code directories
```

## Severity Levels

| Level | Weight | Description |
|-------|--------|-------------|
| `mild` | 0.5 | Minor issues, can ignore |
| `spicy` | 1.5 | Should fix |
| `nuclear` | 3.0 | Fix immediately |

## CLI Options

```bash
# Basic usage
garbage-code-hunter analyze [PATH]

# Language filter
garbage-code-hunter analyze --lang go

# Exclude patterns
garbage-code-hunter analyze --exclude "vendor/*" --exclude "*.pb.go"

# LLM mode (requires API key)
garbage-code-hunter analyze --mode llm

# Output format
garbage-code-hunter analyze --format json
garbage-code-hunter analyze --format markdown
```
