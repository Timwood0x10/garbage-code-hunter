# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

---

## Versioning Guide

- **Major (X.0.0)**: Breaking changes, API redesigns
- **Minor (0.X.0)**: New features, significant improvements
- **Patch (0.0.X)**: Bug fixes, minor improvements

### Current Version: 0.2.0-dev

**Next release target**: v0.2.0
**Status**: Feature complete, awaiting final integration testing
**Blocking items**:
- [ ] Complete cross_file module API integration (remove remaining warnings)
- [ ] Finance project validation (target: <350 issues, currently ~798)
- [ ] Performance optimization for >1000 file projects
