# Garbage Code Hunter — Final Test Report

## 1. Magic Number Allowlist (Per-Language Defaults + Config Integration)

### Problem
The `magic-number` rule had a hardcoded allowlist (`0, 1, -1, 2, 100, 0.0, 1.0, 10, 60, 24`) shared across all languages. Common domain-specific numbers like HTTP status codes (JS/TS: 200, 404), buffer sizes (C/Rust: 1024, 4096), and byte limits (Python: 255) were incorrectly flagged.

### Solution
- **Per-language defaults** added via `check_inner()`: JS/TS (HTTP codes), Python (byte ranges), C/C++/Go/Rust (buffer sizes, powers of 2)
- **Config integration**: `check_with_context()` reads `MagicNumberRuleConfig.allowed_numbers` + `.ui_layout_numbers` from `.garbage-code-hunter.toml`, merges with built-in defaults
- **Serde fix**: Added `#[serde(rename = "magic_number")]` on `RulesConfig.magic_number` to resolve kebab-case/snake_case conflict caused by `#[serde(rename_all = "kebab-case")]` on `RulesConfig`

### Test Results

| Test | Before | After |
|------|--------|-------|
| Python: `x = 1024 + 255` | 2 issues | 1 issue (only 1024, 255 allowed) |
| Rust: `let x = 1024 + 4096;` | 2 issues | 0 issues |
| JS: `const x = 200 + 404;` | 2 issues | 0 issues |
| C: `int x = 1024 + 65535;` | 2 issues | 0 issues |
| Config allowlist (JS): `3000 + 3600 + 21` with all in TOML | 3 issues | 0 issues |
| Config allowlist (JS) — no config | 3 issues | 3 issues (unchanged) |

**Unit tests**: 779/779 passed (added `test_magic_number_config_parse`)

---

## 2. Signal Layer Line=0 Separation

### Problem
Signal findings (e.g., "Naming Chaos", "Nested Hell") were serialized into the same `issues[]` array as per-line rule findings, with `line: 0` producing meaningless locations.

### Solution
- **JSON output** (`output_json` in `helpers.rs`): Filtered into separate `signals[]` array; only `line > 0` findings remain in `issues[]`.
- Added `AnalyzeJsonSignal` struct and `signal_count` field in summary.

### Test Results

| Metric | Before | After |
|--------|--------|-------|
| `issues[]` with `line=0` | Yes (mixed) | 0 (clean) |
| `signals[]` | N/A | Present (29 for system_alert) |
| `summary.signal_count` | N/A | 29 |
| `summary.issue_count` | Included signals | 112 (issues only) |

Sample JSON structure:
```json
{
  "schema_version": "1.0",
  "issues": [ /* per-line rule findings only */ ],
  "signals": [ { "signal": "Naming Chaos", "file_path": "...", "severity": "Mild", "violation_count": 11 } ],
  "summary": { "issue_count": 112, "signal_count": 29, "total_score": 46.72 }
}
```

---

## 3. Full Pipeline Verification

| Project | Issues | Signals | Score | Status |
|---------|--------|---------|-------|--------|
| `system_alert` (Rust) | 112 | 29 | 46.72 | ✅ Clean separation |
| Single JS file (no config) | 4 | 1 | — | ✅ Per-language defaults |
| Single JS file (with config) | 1 | 1 | — | ✅ Config allowlist works |

---

## 4. Potential Improvements (Not In Scope)

- Terminal output: signal findings (`line=0`) still appear in friend feedback "Quick wins" as line=0 entries. A future change could filter them there too.
- `MagicNumberRule` is only registered in `register_rust_rules()` despite supporting all languages — should be in a shared registry.
