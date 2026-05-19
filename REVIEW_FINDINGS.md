---

## Additional Findings (Round 3)

---

### 17. `days_from_ymd` era computation — missing parens causes `/400` to apply only to else-branch

**File:** `src/helpers.rs:196`  
**Severity:** **Critical**

```rust
// Current (buggy):
let era = if y >= 0 { y } else { y - 399 } / 400;
```

Due to Rust's operator precedence, this evaluates as:

```rust
let era = (if y >= 0 { y } else { y - 399 }) / 400;
```

`/ 400` is applied only to the **else-branch result**, not the whole conditional. For any negative year (BC dates), the era is divided by 400 a second time, producing a value offset by multiples of 146097 days (one full Gregorian cycle). For example, `y = -44` (44 BC): `y - 399 = -443; -443 / 400 = -2` (integer division floors toward negative infinity), so `era = -2`, but the correct Gregorian era for -44 is `era = 0`.

**Impact:** `parse_date_to_timestamp` yields epoch day counts shifted by `±146097 × n` for BC-era dates. Who calls this? The `decay` and `commit_roaster` subsystems only — but they silently accept the wrong number.

---

### 18. `days_from_ymd` — `y = 0` (astronomical year zero) boundary error

**File:** `src/helpers.rs:196`  
**Severity:** **Critical** (related to Finding 17; same root cause)

The if-branch `y >= 0 { y }` gives `era = 0 / 400 = 0`. With `yoe = 0`, the `doy` calculation goes: `(153 × (m - 3) + 2) / 5 + d - 1`. When `y = 0` and `m ≤ 2` the year is decremented to `y = -1` shifting into the negative/else path entirely. All months ≥ January but ≤ February force the year into the else branch, causing the wrong era calculation.

Unlike the pure-ancient scenario in Finding 17, here the wrong era means `days_since_epoch` could itself return a negative number far outside the Unix epoch range. Any code downstream formatting that as a `SystemTime` will panic (Unix-like systems reject negative durations before 1970 in some places).

**Impact:** `parse_date_to_timestamp` can return wildly incorrect (even negative) timestamps for astronomical year-zero dates in Jan/Feb.

---

### 19. `count_duplicate_imports` for Rust — whole-line `HashSet` under-counts

**File:** `src/language/adapter/rust.rs:282–289`  
**Severity:** **High**

```rust
if trimmed.starts_with("use ") && !seen.insert(trimmed.to_string()) {
    count += 1;
}
```

Two import forms are under-counted:

**a) Prefix-import pairs** — distinct lines sharing a prefix are treated as unrelated:
```rust
use std::fmt;           // line A
use std::fmt::Display;  // line B — "different key", not flagged
```
`HashSet` considers these unique keys — the `Display` sub-import is never flagged as a follow-on to `fmt`.

**b) Grouped imports** — a single `use std::{fmt, io};` occupies one key, but splitting that line into individual statements later won't flag anything. Meanwhile if a *lone* `use std::fmt;` already seen is grouped, the group won't match the line-level key. The core problem: dead code is accumulated in `seen`, but the **HashKey pair** is an exact line-string, not a semantic prefix/child relationship.

**Impact:** Duplicate-import smell is under-reported for any Rust project with path-prefix imports (very common) or `use` groups.

---

### 20. `"return;"` arm in Rust `count_dead_code` is dead — shadowed by prefix-guard

**File:** `src/language/adapter/rust.rs:251–254`  
**Severity:** **Low** (code-smell only)

```rust
if matches!(
    trimmed,
    "return;" | "break;" | "continue;" | "unreachable!()" | "unreachable!();"
) || (trimmed.starts_with("return ") && trimmed.ends_with(';'))
    || (trimmed.starts_with("panic!(") && trimmed.ends_with(';'))
    || (trimmed.starts_with("unreachable!(") && trimmed.ends_with(')'))
```

`"return;"` is a live arm in the `matches!` branch, but it's listed **first** alongside `"break;"` and `"continue;"`. If a user wrote `return;` it **would** match the first arm and the function wouldn't need the prefix check at all — so detection is not broken.

However, `"return;"` is immediately followed by `trimmed.starts_with("return ")` (note the space vs semicolon). Which makes the `matches!` arm `"return;"` redundant — there's no semantic difference. The `"break;"` and `"continue;"` arms *are* still needed since there's no equivalent prefix form for them. But the `"return;"` pattern overlaps with itself in both directions.

Same pattern exists in `src/language/adapter/go.rs:366–373`.

**Impact:** No functional defect in detection; but future readers will remove the `"return;"` arm and accidentally break `return;` detection if they also remove the `matches!` block entirely.

---

### 21. `count_panic_calls` in Rust misses `assert!`, `assert_eq!`, `assert_ne!`

**File:** `src/language/adapter/rust.rs:20–46`  
**Severity:** **Medium**

```rust
// Only detects field expressions and macro_invocation with name "panic":
let Ok(groups) = collect_captures(file, "(field_expression ...)") else { ... };
let Ok(groups) = collect_captures(file, "(macro_invocation macro: (identifier) @m ...)") else { ... };
```

`assert!`, `assert_eq!`, `assert_ne!` fire `panic!` macro internally, but they do **not** match either of these query patterns: they are `macro_invocation` nodes named `"assert"` / `"assert_eq"` / `"assert_ne"`, not `"panic"`. They are caught separately by `count_debug_calls` but routed to the CodeSmells / HotfixCulture signal, **not** to PanicAddiction.

**Impact:** Production uses of `assert!`, `assert_eq!`, `assert_ne!` are invisible to PanicAddiction risk scoring, producing an understated panic-risk assessment for codebases that use assertions for input validation.

---

### 22. `"panic!"` string comparison in `count_panic_calls` is dead code

**File:** `src/language/adapter/rust.rs:38`  
**Severity:** **Low**

```rust
if cap.text == "panic" || cap.text == "panic!" {
```

Tree-sitter `(macro_invocation macro: (identifier) @m)` captures only the identifier token — `"panic"`, never `"panic!"`. The `"!"` is an `!` token node, not part of the identifier. The `|| cap.text == "panic!"` branch is dead code.

**Impact:** None at runtime. Mislead maintenance — a future query expansion that captures a different node kind could start matching `"panic!"` but would need to be kept in sync.

---

### 23. `count_dead_code` in Rust misses `panic!("…")` with trailing semicolon

**File:** `src/language/adapter/rust.rs:252`  
**Severity:** **Medium**

```rust
|| (trimmed.starts_with("panic!(") && trimmed.ends_with(')'))
```

This guard matches `panic!("...")` **without** a trailing semicolon, but NOT the far more common `panic!("...");` statement form. After `rustfmt`, every bare `panic!()` in statement position gets a trailing semicolon, making the guard a no-op for the idiomatic usage pattern.

The same pattern fixed-by-omission: `"return;"` is covered by the earlier `matches!` arm, but the `"panic!("...` guard was intentionally written without `;` to avoid matching expression contexts — however the function operates on whole comment lines, not token-level, so tracing whether a macro is at statement or expression scope is already lost. The guard needs to match `panic!("...");` as well.

**Impact:** Any dead-code comment containing `panic!("...");` is silently skipped, under-counting dead-code smell for Rust projects that use `panic!` as assertion/dead-code markers rather than leaving them as bare expressions.

---

### Summary Table (All Findings)

| # | File | Lines | Issue | Severity |
| **17** | **`src/helpers.rs`** | **196** | **`days_from_ymd` missing parens — `/400` only on else-branch** | **🔴 Critical** |
| **18** | **`src/helpers.rs`** | **196–200** | **`days_from_ymd` `y=0` boundary can produce wrong/negative epoch days** | **🔴 Critical** |
| **19** | **`src/language/adapter/rust.rs`** | **282–289** | **`count_duplicate_imports` whole-line `HashSet` under-counts path-prefix pairs** | **🟠 High** |
| **20** | **`src/language/adapter/rust.rs`** | **251–254** | **`"return;"` arm shadowed by `"return"` prefix-guard (but detection still works)** | **🟡 Low** |
| **21** | **`src/language/adapter/rust.rs`** | **20–46** | **`count_panic_calls` skips `assert!` / `assert_eq!` / `assert_ne!`** | **🟡 Medium** |
| **22** | **`src/language/adapter/rust.rs`** | **38** | **`"panic!"` comparison is dead — tree-sitter macro identifier is always `"panic"`** | **🟢 Low** |
| **23** | **`src/treesitter/query.rs`** | **92** | **`unwrap()` on cache.get after just-insert — safe but masks logic errors** | **🟡 Low** |
