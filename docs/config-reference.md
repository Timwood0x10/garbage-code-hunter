# Project Configuration Reference

Place `.garbage-code-hunter.toml` in your project root to customize analysis.

## Full Schema

```toml
# ── Global whitelists ──────────────────────────────────────────
[whitelists]
# Numbers to ignore globally (e.g., "magic-number" rule)
magic-numbers = [800, 1000, 2000]

# Variable names to allow globally (e.g., naming rules)
variable-names = ["data", "info", "ctx"]

# Directory patterns with reduced sensitivity
directories = ["generated", "vendor"]

# Completely excluded path patterns
exclude-patterns = ["third_party/*"]

# ── Rule overrides ─────────────────────────────────────────────
[rules.naming]
enabled = true                    # Enable/disable all naming rules
severity = "mild"                 # Override severity: mild | spicy | nuclear
allowed-names = ["e", "ctx"]      # Additional names allowed by naming rules

[rules.magic-number]
enabled = true                    # Enable/disable magic-number rule
allowed-numbers = [3000, 86400]   # Numbers to allow (in addition to per-language defaults)
ui-layout-numbers = [20, 25, 30]  # Common UI layout values (merged with defaults)

[rules.unwrap]
enabled = true                    # Enable/disable unwrap-abuse rule
threshold = 3                     # Min unwraps before reporting (default: 1)
nuclear-threshold = 15            # Count for nuclear severity (default: 15)

[rules.println]
enabled = true                    # Enable/disable println-debugging rule
allow-in-main-files = true        # Allow println in main entry files
threshold = 3                     # Min occurrences before reporting (default: 1)

# ── Signal detectors ──────────────────────────────────────────
[signals]
# Each signal can be enabled/disabled individually
panic-addiction = true
naming-chaos = true
nested-hell = true
hotfix-culture = true
over-engineering = true
code-smells = true
duplication = true

# ── Per-file/per-directory overrides ──────────────────────────
[[overrides]]
paths = ["tests/*", "examples/*"]
relax-sensitivity = true           # Reduce strictness for non-production code

[[overrides]]
paths = ["src/legacy/**"]
disable-rules = ["naming", "unwrap"]
