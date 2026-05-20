# Rules Reference

This page summarizes the rule families used by the analyzer.
The exact rule list evolves over time, but the categories below are stable.

## Rule families

- Naming quality: terrible names, abbreviations, single-letter variables
- Structure quality: deep nesting, long functions, god functions
- Maintainability: duplication, commented-out code, stale TODO comments
- Debug leftovers: println and similar print-based debugging
- Language-specific hygiene: Go, Python, Rust, Java, C, C++, Ruby, Swift, and Zig rules

## Commonly reported issues

- `magic-number`
- `deep-nesting`
- `long-function`
- `god-function`
- `println-debugging`
- `commented-code`
- `todo-comment`
- `duplication`

## Language coverage

- Rust, Go, Python, JavaScript, TypeScript, Java, C, C++, Ruby, Swift, and Zig are supported
- Tree-sitter grammars are available for the full supported set used by source parsing
- File discovery recognizes the corresponding extensions listed in `src/language/mod.rs`

## Tuning tips

- Use `.garbage-code-hunter.toml` to whitelist accepted names and numbers
- Use `exclude-patterns` for generated code
- Use `directories` to reduce sensitivity in noisy folders
- Use CLI `--exclude` for one-off skips

## Notes

- Rule names and severities are designed to be humorous rather than authoritative
- The tool does not claim to detect bugs, security issues, or performance regressions
