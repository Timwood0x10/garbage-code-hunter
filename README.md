# 🗑️ Garbage Code Hunter

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/tests-71%20passing-brightgreen.svg)]()

A humorous Rust code quality detector that roasts your garbage code with style! 🔥

```
Inspiration from https://github.com/Done-0/fuck-u-code.git
```

Unlike traditional linters that give you dry, boring warnings, Garbage Code Hunter delivers **sarcastic, witty, and brutally honest** feedback about your code quality. It's like having a sassy code reviewer who isn't afraid to hurt your feelings (in a good way).

## ✨ Features

- 🎭 **Humorous Code Analysis**: Get roasted with style while learning better coding practices
- 🌍 **Multi-language Support**: Available in English and Chinese (more languages coming soon!)
- 🎯 **Smart Detection**: Identifies common code smells and anti-patterns
- 📊 **Professional Reports**: Generate detailed analysis reports in multiple formats
- 🔧 **Highly Configurable**: Customize output, filtering, and analysis depth
- 📝 **Markdown Export**: Perfect for documentation and CI/CD integration
- 🚀 **Fast & Lightweight**: Built with Rust for maximum performance

## 🎯 Scoring System

Garbage Code Hunter includes a comprehensive **scientific scoring system** that evaluates your Rust code quality on a scale of **0-100**, where:

- **Lower scores = Better code quality** 🏆
- **Higher scores = More problematic code** 💀

### 📊 Score Ranges & Quality Levels

| Score Range | Quality Level | Emoji | Description                                           |
| ----------- | ------------- | ----- | ----------------------------------------------------- |
| 0-20        | Excellent     | 🏆    | Outstanding code quality with minimal issues          |
| 21-40       | Good          | 👍    | Good code quality with minor improvements needed      |
| 41-60       | Average       | 😐    | Average code quality with room for improvement        |
| 61-80       | Poor          | 😟    | Poor code quality, refactoring recommended            |
| 81-100      | Terrible      | 💀    | Critical code quality issues, rewrite urgently needed |

### 🧮 Scoring Algorithm

The scoring system uses a **multi-factor algorithm** that considers:

#### 1. **Base Score Calculation**

Each detected issue contributes to the base score using:

```
Issue Score = Rule Weight × Severity Weight
```

#### 2. **Rule Weights** (Impact Factor)

Different types of issues have different weights based on their impact:

| Category                    | Rule                  | Weight | Rationale                        |
| --------------------------- | --------------------- | ------ | -------------------------------- |
| **Safety Critical**   | `unsafe-abuse`      | 5.0    | Memory safety violations         |
| **FFI Critical**      | `ffi-abuse`         | 4.5    | Foreign function interface risks |
| **Runtime Critical**  | `unwrap-abuse`      | 4.0    | Potential panic sources          |
| **Architecture**      | `lifetime-abuse`    | 3.5    | Complex lifetime management      |
| **Async/Concurrency** | `async-abuse`       | 3.5    | Async pattern misuse             |
| **Complexity**        | `deep-nesting`      | 3.0    | Code maintainability             |
| **Performance**       | `unnecessary-clone` | 2.0    | Runtime efficiency               |
| **Readability**       | `terrible-naming`   | 2.0    | Code comprehension               |

#### 3. **Severity Weights**

Issues are classified by severity with corresponding multipliers:

- **Nuclear** (💥): 10.0× - Critical issues that can cause crashes or security vulnerabilities
- **Spicy** (🌶️): 5.0× - Serious issues affecting maintainability or performance
- **Mild** (😐): 2.0× - Minor issues with style or best practices

#### 4. **Density Penalties**

Additional penalties based on issue concentration:

- **Issue Density**: Problems per 1000 lines of code

  - \>50 issues/1000 lines: +25 penalty
  - \>30 issues/1000 lines: +15 penalty
  - \>20 issues/1000 lines: +10 penalty
  - \>10 issues/1000 lines: +5 penalty
- **File Complexity**: Average issues per file

  - \>20 issues/file: +15 penalty
  - \>10 issues/file: +10 penalty
  - \>5 issues/file: +5 penalty

#### 5. **Severity Distribution Penalties**

Extra penalties for problematic patterns:

- **Nuclear Issues**: First nuclear issue +20, each additional +5
- **Spicy Issues**: After 5 spicy issues, each additional +2
- **Mild Issues**: After 20 mild issues, each additional +0.5

### 📈 Metrics Included

The scoring system provides detailed metrics:

- **Total Score**: Overall code quality score (0-100)
- **Category Scores**: Breakdown by issue categories
- **Issue Density**: Problems per 1000 lines of code
- **Severity Distribution**: Count of nuclear/spicy/mild issues
- **File Count**: Number of analyzed Rust files
- **Total Lines**: Total lines of code analyzed

### 🎯 Interpretation Guide

**For Excellent Code (0-20):**

- Minimal issues detected
- Strong adherence to Rust best practices
- Good architecture and safety patterns

**For Good Code (21-40):**

- Few minor issues
- Generally well-structured
- Minor optimizations possible

**For Average Code (41-60):**

- Moderate number of issues
- Some refactoring beneficial
- Focus on complexity reduction

**For Poor Code (61-80):**

- Significant issues present
- Refactoring strongly recommended
- Address safety and complexity concerns

**For Terrible Code (81-100):**

- Critical issues requiring immediate attention
- Consider rewriting problematic sections
- Focus on safety, correctness, and maintainability

### 🔬 Scientific Approach

The scoring system is designed to be:

- **Objective**: Based on measurable code metrics
- **Weighted**: Critical issues have higher impact
- **Contextual**: Considers code size and complexity
- **Actionable**: Provides specific improvement areas
- **Consistent**: Reproducible results across runs

## 🎪 What It Detects

### Naming Disasters

- Terrible variable names (`data`, `temp`, `info`, `obj`)
- Single-letter variables (except common loop counters)
- Generic meaningless identifiers

### Code Structure Issues

- Deep nesting (Russian doll syndrome)
- Overly long functions
- Complex conditional logic

### Rust-Specific Anti-patterns

- `unwrap()` abuse (panic bombs 💣)
- Unnecessary `clone()` calls (memory waste)
- Poor error handling patterns

## 🚀 Installation

### From Source

```bash
git clone https://github.com/yourusername/garbage-code-hunter.git
cd garbage-code-hunter
make install
```

### Using Cargo

```bash
cargo install garbage-code-hunter
```

## 📖 Usage

### Basic Usage

```bash
# Analyze current directory
cargo run

# Analyze specific file or directory
cargo run -- src/main.rs
cargo run -- src/

# Use make targets for convenience
make run ARGS="src/ --verbose"
make demo
```

### Language Options

```bash
# Chinese output (default)
garbage-code-hunter --lang zh-CN src/

# English output
garbage-code-hunter --lang en-US src/
```

### Advanced Options

```bash
# Verbose analysis with top 3 problematic files
garbage-code-hunter --verbose --top 3 --issues 5 src/

# Only show summary
garbage-code-hunter --summary src/

# Generate Markdown report
garbage-code-hunter --markdown src/ > code-quality-report.md

# Exclude files/directories
garbage-code-hunter --exclude "test_*" --exclude "target/*" src/

# Show only serious issues
garbage-code-hunter --harsh src/
```

## 🎨 Sample Output

### English Mode

```
🗑️  Garbage Code Hunter 🗑️
Preparing to roast your code...

📊 Code Quality Report
──────────────────────────────────────────────────
Found some areas for improvement:

📈 Issue Statistics:
   2 🔥 Nuclear Issues (fix immediately)
   5 🌶️  Spicy Issues (should fix)
   3 😐 Mild Issues (can ignore)
   10 📝 Total

📁 main.rs
  💥 Line 15:1 - This variable name is more abstract than my programming skills
  🌶️ Line 23:5 - Another unwrap()! Are you trying to make the program explode in production?
  😐 Line 8:9 - Single letter variable? Are you writing math formulas or torturing code readers?
```

### Chinese Mode

```
🗑️  Garbage Code Hunter 🗑️
Preparing to roast your code...

📊 Code Quality Report
──────────────────────────────────────────────────
Found some areas for improvement:

📈 Issue Statistics:
   1 🔥 Nuclear Issues (fix immediately)
   138 🌶️  Spicy Issues (should fix)
   34 😐 Mild Issues (can ignore)
   173 📝 Total

🏆 Code Quality Score
──────────────────────────────────────────────────
   📊 Score: 60.9/100 😐
   🎯 Level: Average
   📏 Lines of Code: 260
   📁 Files: 1
   🔍 Issue Density: 66 issues/1k lines

   🎭 Issue Distribution:
      💥 Nuclear: 1
      🌶️  Spicy: 138
      😐 Mild: 34

🏆 Files with Most Issues
──────────────────────────────────────────────────
   1. func.rs (173 issues)

📁 func.rs
  📦 Nesting depth issues: 20 (depth 4-9)
  🔄 Code duplication issues: 9 (6 instances)
  🏷️ Variable naming issues: 128 (a, b, c, d, e, ...)
  🏷️ Variable naming issues: 13 (a, b, c, d, e, ...)
  ⚠️ long function: 1


📊 Scoring Details
──────────────────────────────────────────────────
📋 Category Scores:
  ⚠ 🏷️ Naming 90分     Terrible, urgent fixes needed
    💬 Congrats! Variables harder to understand than comments 🏆
  ⚠ 🧩 Complexity 90分     Terrible, urgent fixes needed
    💬 Complexity off the charts! Even AI gave up 🤖
  ⚠ 🔄 Duplication 90分     Terrible, urgent fixes needed
    💬 Suggest renaming to ctrl-c-ctrl-v.rs 📋
  ✓✓ 🦀 Rust Basics 0分     Excellent, keep it up
  ✓✓ ⚡ Advanced Rust 0分     Excellent, keep it up
  • 🚀 Rust Features 69分     Poor, refactoring recommended
    💬 More macros than my excuses 🎭
  ✓✓ 🏗️ Code Structure 0分     Excellent, keep it up

🧮 Weighted Calculation:
  Score calculation: (90.0×0.25 + 90.0×0.20 + 90.0×0.15 + 0.0×0.15 + 0.0×0.10 + 69.4×0.10 + 0.0×0.05) ÷ 1.00 = 60.9

📏 Scoring Scale (higher score = worse code):
  💀 81-100: Terrible    🔥 61-80: Poor    ⚠️ 41-60: Average
  ✅ 21-40: Good         🌟 0-20: Excellent
📋 Summary
──────────────────────────────────────────────────
😐 Average code quality, Score: 60.9/100, room for improvement

Found some serious issues, suggest fixing nuclear problems first 🔥

💡 Suggestions
──────────────────────────────────────────────────
   💡 Use meaningful variable names that make code self-documenting (e.g., user_count instead of data)
   🎯 Variable names should describe what they store, not the data type
   🔧 Reduce nesting levels, consider extracting functions or using early returns (guard clauses)
   🏗️ Complex conditional logic can be split into multiple small functions
   ✂️ Split long functions into smaller ones, follow the single responsibility principle
   📏 A function should ideally not exceed 20-30 lines for better understanding and testing
   🔄 Extract common code into functions to follow the DRY principle
   🏗️ Consider creating utility functions or modules for repeated logic

Keep working hard to make your code better! 🚀
```

## 🛠️ Command Line Options

| Option                | Short          | Description                                    |
| --------------------- | -------------- | ---------------------------------------------- |
| `--help`            | `-h`         | Show help message                              |
| `--verbose`         | `-v`         | Show detailed analysis report                  |
| `--top N`           | `-t N`       | Show top N files with most issues (default: 5) |
| `--issues N`        | `-i N`       | Show N issues per file (default: 5)            |
| `--summary`         | `-s`         | Only show summary conclusion                   |
| `--markdown`        | `-m`         | Output Markdown format report                  |
| `--lang LANG`       | `-l LANG`    | Output language (zh-CN, en-US)                 |
| `--exclude PATTERN` | `-e PATTERN` | Exclude file/directory patterns                |
| `--harsh`           |                | Show only the worst offenders                  |

## 🔧 Development

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
# Debug build
make build

# Release build
make release

# Run tests
make test

# Format code
make fmt

# Run linter
make clippy
```

### Running Demo

```bash
make demo
```

This creates a sample file with intentionally bad code and runs the analyzer on it.

## 🎯 Examples

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Code Quality Check
  run: |
    cargo install garbage-code-hunter
    garbage-code-hunter --markdown --lang en-US src/ > quality-report.md
    # Upload report as artifact or comment on PR
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit
garbage-code-hunter --harsh --summary src/
if [ $? -ne 0 ]; then
    echo "Code quality issues detected. Please fix before committing."
    exit 1
fi
```

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **Add New Rules**: Implement additional code smell detection
2. **Language Support**: Add translations for more languages
3. **Improve Messages**: Make the roasts even funnier (but still helpful)
4. **Documentation**: Help improve docs and examples
5. **Bug Reports**: Found a bug? Let us know!

### Adding New Detection Rules

1. Create a new rule in `src/rules/`
2. Implement the `Rule` trait
3. Add humorous messages in `src/i18n.rs`
4. Add tests
5. Submit a PR!

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by the need for more entertaining code reviews
- Built with ❤️ and a lot of ☕
- Thanks to all the developers who write garbage code (we've all been there!)

## 🔗 Links

- [Documentation](https://docs.rs/garbage-code-hunter)
- [Crates.io](https://crates.io/crates/garbage-code-hunter)
- [GitHub Repository](https://github.com/TimWood0x10/garbage-code-hunter)
- [Issue Tracker](https://github.com/TimWood0x10/garbage-code-hunter/issues)

---

**Remember**: The goal isn't to shame developers, but to make code quality improvement fun and memorable. Happy coding! 🚀
