# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.2.2] - 2026-05-19

### Core Architecture Overhaul

#### Style IR — Language-Neutral Style Facts (NEW)

Introduced a language-independent intermediate representation (`src/style_ir/mod.rs`) that decouples AST parsing from signal detection:

- **`StyleIr`** — Unified struct holding all extracted style facts (panic calls, naming violations, deep nesting, debug calls, excessive params, unsafe blocks, magic numbers, commented-out lines, TODO markers, plus language-specific issues for Go/Python/Java/Ruby/C/TS)
- **`StyleIrSummary`** — Stable JSON-serializable summary exposing 25+ metrics including `god_function_count`, `over_engineering_count`, `code_smell_count`, and `is_clean_signal_baseline`
- **`StyleIrThresholdSummary`** — Exposes detection thresholds (`excessive_param_threshold: 5`, `god_function_line_threshold: 50`)
- Integrated into all output formats (terminal text, JSON, Markdown, GitHub Actions) for consistent downstream consumption

#### StyleFinding — Unified Finding Model (NEW)

Standardized observation model (`src/finding.rs`, 816 lines) replacing ad-hoc issue handling:

- **`StyleFinding`** — Structured finding with `FindingId`, `CodeLocation` (file/line/column/span/symbol), `RuleMeta` (name/category/intent), `StyleSignal`, `Severity`, `Confidence` (Low/Medium/High), `Evidence` (snippet/metric/context), and optional `StyleSuggestion`
- **`StyleCategory`** — 8 categories: Naming, Complexity, Duplication, Comments, DebuggingLeftovers, Structure, Consistency, DependencyStyle
- **`RuleIntent`** — 5 intents: Readability, Maintainability, TeamConvention, NoiseReduction, CognitiveLoad
- Bidirectional conversion between `StyleFinding ↔ CodeIssue` for backward compatibility
- `compute_signal_scores_from_findings()` — Language-agnostic signal scoring via `.signal` field (no `classify_rule()` dependency)
- 40+ unit tests verifying ID determinism, category mapping, confidence scoring, and signal aggregation

#### SignalDetector — Direct Signal Detection (NEW)

Signal-based analysis layer (`src/signals.rs`, `src/detectors.rs`) aggregating 10 behavioral dimensions:

- **`StyleSignal`** enum — 10 signals: Duplication, PanicAddiction, NamingChaos, NestedHell, HotfixCulture, OverEngineering, CodeSmells, LegacyCode, TodoMountain, LineCountSmell
- **`SignalDetector` trait** — `count_violations()`, `count_violations_with_ir()` (avoids redundant parsing), `detect_findings()`, `detect_findings_with_ir()`
- **10 detectors**: `PanicAddictionDetector`, `NamingChaosDetector`, `NestedHellDetector`, `HotfixCultureDetector`, `OverEngineeringDetector`, `CodeSmellsDetector`, `DuplicationDetector`, `LegacyCodeDetector`, `TodoMountainDetector`, `LineCountSmellDetector`
- **`LanguageCapabilityMatrix`** — Tracks which signals are detectable per language (all 11 grammar languages support all 10 signals)
- **`aggregate_detector_scores()`** — Aggregates scores across files with density normalization
- **`violations_to_score()`** — log2 density formula capped at 25.0
- **`classify_rule()`** — Maps 50+ rule names to their behavioral signal
- **Test-file awareness** — Per-detector `skips_test_files()` + config-level `[signals] skip-tests` = double-layer filtering
- 100+ unit tests across all detectors

#### LanguageAdapter — Unified Multi-Language API (NEW)

Replaced per-language rule files with a uniform adapter trait (`src/language/adapter/`, 3000+ lines):

- **`LanguageAdapter` trait** — `extract_functions()`, `count_panic_calls()`, `count_naming_violations()`, `count_deeply_nested_blocks()`, `count_debug_calls()`, `count_excessive_params()`, `count_unsafe_blocks()`, `count_magic_numbers()`, `count_commented_out_code()`, `count_todo_markers()`, `count_goroutine_spawns()`, `count_defer_in_loop()`, `count_go_convention_violations()`, `count_python_issues()`, `count_java_issues()`, `count_ruby_issues()`, `count_c_issues()`, `count_ts_issues()`, `count_dead_code()`, `count_duplicate_imports()`, `has_test_nodes()`
- **11 adapters**: Rust, Python, JavaScript, TypeScript, Go, Java, Ruby, C, C++, Swift, Zig
- **Batch query optimization** — `query_patterns()` + `batch_captures()` + `compute_all()` runs all tree-sitter queries in a single cursor traversal per file, reducing AST traversals from 15 to 1
- **Language-specific rules**: Hungarian notation, abbreviation abuse, naming conventions per language
- Every adapter has 10+ unit tests

#### StyleProfile — Unified Personality Inference (NEW)

Replaced ad-hoc personality inference with data-driven profiling:

- **`StyleProfile::from_signal_counts()`** — Normalizes raw signal counts into 0-25 scores
- **`StyleProfile::from_signal_scores()`** — Builds from direct detector scores
- **`infer_personality_type()`** — 7 personality archetypes depending on dominant signal combination:
  - The Copy-Paste Artist (high Duplication)
  - The YOLO Engineer (high PanicAddiction)
  - The Trait Wizard (high NestedHell)
  - The Legacy Necromancer (high NamingChaos)
  - The Hotfix Mercenary (high HotfixCulture)
  - The Startup Survivor (med Duplication + med PanicAddiction)
  - The Academic Wizard (high OverEngineering / med NamingChaos + med NestedHell)
  - The Enterprise Bureaucrat (default — all signals low)
- 14 unit tests covering all personality branches

### New Modules

#### Friend Feedback Layer (`src/friend/`)

"Friend's Perspective" output system that transforms raw metrics into human-readable insights:

- **`FriendFeedback`** — Combines `FriendMood` (Proud/Concerned/Sarcastic/Alarmed/Exhausted), `BehaviorPattern` (top 3 signals), `NextAction` (top 3 quick wins)
- **`FriendMood::from_score()`** — Maps total score (0-100) to emotional response with emoji
- **`BehaviorPattern::from_signals()`** — Ranks signals by score, provides English + Chinese descriptions and actionable suggestions
- **`NextAction::from_issues()`** — Sorts issues by severity, shows top 3 with file:line locations
- **Bilingual output** — `print()` (English) + `print_zh()` (Chinese)
- Integrated into terminal output and Markdown report
- 8 unit tests covering mood thresholds, pattern ranking, and construction

#### Roast Generator (`src/treesitter/roast.rs`)

- **`RoastGenerator`** — Produces sarcastic "friend-like" messages for each `StyleCategory`
- Per-category roast templates with randomized selection

#### Reporter Translations (`src/reporter/translations.rs`)

- Extracted 871 lines of bilingual (en-US / zh-CN) translation tables from `display.rs`
- Covers personality types, threat levels, diagnostic messages, and code metric labels

#### Reporter Autopsy (`src/reporter/autopsy.rs`)

- Standalone autopsy module for per-file deep-dive analysis
- Utilizes `StyleProfile` for personality inference from signal scores

#### Helpers & CLI Args (`src/helpers.rs`, `src/args.rs`)

- **`args.rs`** (304 lines) — Dedicated CLI argument definitions moved from `main.rs`
- **`helpers.rs`** (548 lines) — Shared utility functions: `derive_count_score()`, `derive_radar_score()`, `derive_danger_zone_score()`, `quick_scan_score()`, `overall_score()`, `parse_date_to_timestamp()`, `calculate_metrics()`

#### Detectors Registry (`src/detectors.rs`)

- 10 `SignalDetector` implementations all operating on `StyleIr` for language-agnostic detection
- LegacyCodeDetector and TodoMountainDetector explicitly set `skips_test_files: false` to always scan test code
- LineCountSmellDetector applies per-context thresholds (1000 lines normal, 2000 lines test)

### Multi-Language Expansion

#### Swift & Zig Support (NEW)

- **SwiftAdapter** — `extract_functions`, `count_panic_calls` (force-unwrap `!`, `fatalError`, `preconditionFailure`), `count_naming_violations`, `count_deeply_nested_blocks`, `count_debug_calls` (print, debugPrint, dump)
- **ZigAdapter** — `extract_functions`, `count_panic_calls` (unreachable, @panic, catch unreachable), `count_naming_violations`, `count_deeply_nested_blocks`, `count_debug_calls` (@import("std").debug.print, std.log)
- All detectors extended to support 11 languages (Rust/Python/JS/TS/Go/Java/Ruby/C/C++/Swift/Zig)

### Analysis Pipeline

#### Analyzer (`src/analyzer.rs`)

- **3-Phase pipeline**: Phase 1 = parse & cache `ParsedFile`, Phase 2 = SignalDetector + StyleIr, Phase 3 = duplication detection
- **`FullAnalysisResult`** — Contains `issues`, `file_infos` (per-file StyleIr summaries), `direct_scores` (signal scores), `duplication_findings`
- **`StyleIrFileInfo`** — Per-file metadata with `style_ir_summary` for JSON output
- **`is_generated_file()`** — Comprehensive regex-based filtering (protobuf, node_modules, vendor, .pb., .generated.)
- **`collect_source_files()`** — Unified file gathering with path exclusion integration
- **Optimization**: Shared `ParsedFile` cache avoids redundant re-parsing across detector phases

#### Scoring (`src/scoring.rs`)

- **Two-tier scoring**: `n_score` (nuclear severity, log-scaled capped at 40) + `d_score` (issue density per 1k lines, log-scaled capped at 60) = total 0-100
- **`calculate_score_with_direct()`** — Integrates direct signal scores into composite quality score
- **`CodeQualityScore.signal_scores`** — New field storing per-signal scores for downstream consumption
- Fixed: Score was 100.0 (Terrible) when no issues → now correctly 0.0 (Excellent)
- Extended test suite for monotonicity, threshold boundaries, zero-division safety

### Output & Reporting

#### Terminal Output

- Integrated `FriendFeedback` display after score summary
- `StyleIrSummary` shown in default/brief/summary modes (function count, god functions, code smells, over-engineering)
- Personality type shown in summary
- Bilingual mode: `LANG=zh_CN.UTF-8` triggers Chinese roast + translations

#### JSON Output (`--format json`)

- New schema with `schema_version`, `issues[]`, `files[]` (per-file `style_ir_summary`), `summary.file_count/issue_count/style_ir_summary`
- `StyleIrSummary` includes thresholds for downstream tooling
- VSCode extension updated to parse new JSON schema (`normalizeAnalyzeJson()`)

#### Markdown Output (`--format markdown`)

- Complete report with score, personality, signal distribution table, per-file metrics, FriendFeedback section
- Bilingual Markdown header/footer

#### GitHub Actions (`--format github-actions`)

- Native GitHub Annotation output for PR comments
- Integrated into CI workflow

### Performance

- **18x analysis speedup** for large projects — by consolidating multiple tree-sitter traversals into single LanguageAdapter calls
- **10x test execution speedup** — via `shared_engine` (OnceLock) caching tree-sitter grammars across all tests
- **Tree-sitter `QUERY_CACHE`** — Thread-local query caching avoids redundant compilation of repeated patterns
- **Batch captures** — `collect_captures_multi()` executes merged multi-pattern queries in a single cursor pass
- **Parallel analysis** — `rayon`-based parallel file processing with shared engine
- Benchmark suite (`benches/performance_tests.rs`) with 15 benchmarks across 8 groups:
  - Near-linear scalability: 1 file = 0.81ms, 50 files = 39.79ms (~0.80ms/file constant)
  - Garbage code ~1.6× slower than clean code (stable ratio)
  - Cross-language projects: 4 languages × 4 files = 107ms
  - Full pipeline overhead: `analyze_full` 61% slower than `analyze_file`

### Accuracy & Quality

- **25 real-world projects tested** across 7 languages, ~5.5M lines of code
- **Structural rules (deep-nesting, god-function, long-function, code-duplication)**: **100% TP rate** — production-ready for CI gates
- **Language-specific rules (must-use, unwrap, abbrev)**: **~100% TP**
- **Overall accuracy report** published: `ACCURACY_REPORT.md`
- **Per-language accuracy docs**: `docs/en/` and `docs/zh/` directories with full TP/FP sampling (2-3 samples per rule per language)
- **Style rules (magic-number, naming, println)**: ~15-25% TP — identified as advisory-only, needs per-project config tuning
- Context-aware analysis: FileContext system adjusts rule sensitivity for Business/Example/Test/Benchmark/Documentation code
- Magic-number per-language default allowlists (0, 1, 2, -1 excluded globally)

### Documentation

- **Full bilingual documentation site**: `docs/en/` and `docs/zh/` covering configuration, rules, tools, and accuracy for all 11 languages
- **Configuration reference**: `docs/config-reference.md` with example config at `docs/examples/basic-config.toml`
- **Architecture report**: `OPTIMIZATION_ANALYSIS_UPDATED.md` with 5-phase architecture roadmap
- **Benchmark report**: `BENCHMARK_REPORT.md` with detailed performance profiles
- **Comprehensive audit**: `todolist.md` tracking all findings (H1-H10, M1-M12, L1-L8)
- **Code review findings**: `REVIEW_FINDINGS.md` and `docs/code_review_2026-05-19.md`
- **Bootstrap test report**: `BOOTSTRAP_TEST_REPORT_v0.2.2.md` (796 tests, 0 failures)
- **README updated** with architecture diagram and tool listing

### CI/CD & Dev Experience

- **GitHub Actions CI/CD**: Automated pipeline with `-D warnings` clippy policy, multi-platform builds (ubuntu/macos/windows), and parallel testing
- **GitHub Actions PR Review**: Automatic PR comment with roast analysis
- **VSCode Extension**: Updated to parse new JSON schema + 30-second timeout to prevent hanging
- **Makefile**: `make ci` = clippy + test + bench, `make doc` for docs site, `make format`
- **Pre-push hook**: Runs clippy + tests before push via `hooks/pre-push`
- **Deny.toml**: License + advisory checks for dependency audit (`cargo deny`)

### Bug Fixes

**🔴 Critical**
- **UTF-8 panic in `truncate()`** — 3 copies in `decay/mod.rs`, `team_roast/mod.rs`, `commit_roaster/report.rs` would panic on multi-byte chars (Chinese, emoji). Extracted shared UTF-8-safe implementation into `utils.rs`
- **Scoring logic bug** — Score was 100.0 (Terrible) when no issues → now correctly 0.0 (Excellent)
- **VSCode extension hanging** — Added 30-second timeout to `exec()` calls
- **LongFunctionRule `{}` placeholder** — Never substituted, always output raw template
- **RustMustUseRule duplicate condition** — Copy-paste error causing duplicate detection
- **RustMustUseRule multi-line signatures** — Missed multi-line `fn` declarations

**🔴 High**
- **Missing Swift/Zig language detection** — `Language::from_extension()` never matched `"swift"` or `"zig"`, silently classifying as Unknown
- **CrossFileAnalyzer error handling** — Replaced `let _ =` with proper error logging
- **Example file detection logic** — Fixed `/messages/` check placement to prevent misclassification
- **5 tree-sitter rules never registered** — TerribleNamingRule, SingleLetterTsRule, DeepNestingRule, PrintlnDebuggingRule, MagicNumberRule
- **Unwrap-abuse rule never registered** for Rust
- **9 language-specific rules never registered**: TooManyParamsRule, HungarianNotationTsRule, AbbreviationAbuseTsRule, RustDeriveOrderRule, RustErrorDisplayRule, RustMustUseRule, RustDocExampleRule, CrossFileDuplication, FeatureEnvyRule
- **Signal test assertions stale** — `sigs.len() == 7` should be 10 (missing LegacyCode, TodoMountain, LineCountSmell)
- **`direct_signals()` missing** LegacyCode, TodoMountain, LineCountSmell per language

**🟡 Medium**
- **Unclamped score functions** — `derive_radar_score()` and `derive_danger_zone_score()` did not clamp to `[0.0, 100.0]`
- **Hardcoded Chinese in English mode** — `get_category_roast()` always used Chinese names ("命名规范")
- **CLI short flag conflict** — `team-roast` `-l` used by both `--limit` and `--lang`. Changed `--lang` to `-L`
- **CLI default value ambiguity** — `llm_timeout` changed from `u64` to `Option<u64>`
- **0-param functions counted as 1** — All 11 adapters `count_excessive_params()` bug
- **RustErrorDisplayRule hardcoded line 1** — Now handles generic impls (`impl<T> Debug for Foo<T>`)
- **RustDeriveOrderRule misses multi-line derives** — Now handles `#[derive(\n  Debug, Clone\n)]`
- **Java has_annotation wrong line** — Now checks the line above the method
- **TypeScript prefer-interface** — Now only flags object types, not unions/primitives
- **Python `.format()` matches strings/comments** — Now skips string literals and comments
- **Go debug_calls double-counts panic()** — Filtered out panic from debug count
- **Magic number `1_000_000` parse failure** — Handles Rust underscored literals
- **`is_clean_signal_baseline()`** — Was missing `commented_out_lines` and `todo_count` checks
- **Go skip list dead code** — Multi-char entries in len==1 filter
- **`classify_rule()` implicit fallback to CodeSmells** — magic-number, unnecessary-clone, etc. all fall through
- **Silent config parse failure** — No user feedback on malformed config

**🟢 Low**
- **Dead code cleanup** — Removed unused functions, variables, modules across 15 files
- **Module declaration deduplication** — `main.rs` now imports from `lib.rs` instead of redeclaring modules (faster builds)
- **Removed 10 `#[allow(dead_code)]` annotations** — Zero dead code policy enforced
- **Deduplicated `BLOCK_PARENT_TYPES`** — Was duplicated across multiple rule files
- **Deduplicated `direct_signals()`** — All language arms now share a single array
- **Gated slow integration tests** behind `GCH_INTEGRATION` env var
- **Go skip list cleanup** — Removed dead entries
- **Regex recompilation** — Switched to `LazyLock` for per-file regex patterns

## [0.2.1]

### Added
- **Cross-file duplication detection**: New ability to detect duplicated code across multiple files using function fingerprinting and similarity matching
- **Context-aware analysis**: FileContext system that adjusts rule sensitivity based on code type (Business, Example, Test, Benchmark, Documentation)
- **GitHub Actions CI/CD**: Automated pipeline with 0 warning policy, testing, and multi-platform builds
- **Phase 1.3 rule adaptation**: Core rules now use `check_with_context()` method for context-aware analysis:
  - MeaninglessNamingRule: Skips/reduces sensitivity in Test/Example contexts
  - MagicNumberRule: Filters UI-related values in Business context
  - PrintlnDebuggingRule: Completely skips Test/Example files
  - UnwrapAbuseRule: Higher threshold in Test contexts
  - TerribleNamingRule: Skips Example/Demo files
- **Shared `truncate()` utility**: UTF-8 safe string truncation in `utils.rs`, replacing 3 duplicate buggy implementations

### Fixed
- **🔴 Critical: UTF-8 panic in `truncate()`** — 3 copies of the same bug in `decay/mod.rs`, `team_roast/mod.rs`, `commit_roaster/report.rs` would panic on multi-byte characters (Chinese, emoji). Extracted shared UTF-8-safe implementation into `utils.rs`
- **🔴 High: Missing Swift/Zig language detection** — `Language::from_extension()` never matched `"swift"` or `"zig"`, silently classifying these files as `Unknown` despite full tree-sitter grammar support
- **🟡 Medium: Unclamped score functions** — `derive_radar_score()` and `derive_danger_zone_score()` in `main.rs` did not clamp return values to `[0.0, 100.0]`, unlike `derive_count_score()`
- **🟡 Medium: Hardcoded Chinese in English mode** — `reporter/display.rs` `get_category_roast()` always used Chinese category names ("命名规范") regardless of locale setting
- **🟡 Medium: CLI short flag conflict** — `team-roast` subcommand had `-l` used by both `--limit` and `--lang`, causing panic on any invocation. Changed `--lang` to `-L`
- **🟢 Low: Dead code cleanup**
  - Removed unused `all_rules()`, `single_letter_variable()`, `deep_nesting()` from `treesitter/rules/mod.rs`
  - Removed unused `_max_per_rule` variable from `reporter/mod.rs`
  - Removed unused `CommitInfo::hash` field from `decay/mod.rs`
  - Removed dead `parse_python()` test helper from `treesitter/duplication.rs`
  - Removed empty `src/evolution_chat/` directory
  - Deleted `RELEASE_NOTE.md` (content merged into CHANGELOG)

### Fixed
- **🔴 Critical: Scoring logic bug** ([scoring.rs:91](src/scoring.rs#L91))
  - Fixed: Score was 100.0 (Terrible) when no issues, should be 0.0 (Excellent)
  - Impact: All clean projects now correctly show score of 0.0

- **🔴 Critical: VSCode extension hanging** ([extension.ts:462](vscode-extension/src/extension.ts#L462))
  - Added: 30-second timeout to exec() calls to prevent extension freeze when CLI hangs

- **🔴 High: CrossFileAnalyzer error handling** ([analyzer.rs:110](src/analyzer.rs#L110))
  - Changed: Replaced `let _ =` with proper error logging for failed file processing

- **🔴 High: Example file detection logic** ([file_context.rs:108-112](src/context/file_context.rs#L108-L112))
  - Fixed: Moved `/messages/` check inside condition block to prevent misclassification

- **🟡 Medium: CLI default value ambiguity** ([config.rs:137](src/config.rs#L137))
  - Changed: `llm_timeout` parameter from `u64` to `Option<u64>` to distinguish "not set" from "explicitly set to 30"

- **🟡 Medium: Dead code removal**
  - Deleted: `i18n::get_suggestions()` method that always returned empty vec
  - Removed: Unused `_rule_count` variable in main.rs
  - Cleaned up: Empty inline decoration handlers in VSCode extension
  - Removed: Empty `onDidChangeActiveTextEditor` event handler

- **🟢 Low: Module declaration deduplication** ([main.rs](src/main.rs), [lib.rs](src/lib.rs))
  - Changed: main.rs now imports from lib.rs instead of redeclaring all modules
  - Impact: Eliminates duplicate compilation, faster build times

### Changed
- **Removed 10 `#[allow(dead_code)]` annotations**: Project now enforces zero dead code policy
- **Improved error messages**: Better user feedback for cross-file analysis failures
- **Code quality**: All public APIs in cross_file module are now properly used or marked internal


## [0.1.3] - 2026-05-09

### Added
- Educational advice system with bilingual support (English/Chinese)
- Hall of Shame feature for tracking worst offenders
- LLM integration support (Ollama, OpenAI-compatible)
- Markdown output format option
- Verbose mode with detailed diagnostics

### Fixed
- Improved println! detection accuracy (reduced false positives)
- Enhanced magic number detection patterns
- Better unwrap abuse warnings

## [0.1.2] - 2026-05-08

### Added
- Initial VSCode extension support
- Basic rule engine with 25+ rules
- File exclusion patterns
- Multi-language roast messages

## [0.1.1] - 2026-05-07

### Added
- First working version
- Core analyzer implementation
- Basic scoring system
- CLI interface with clap


