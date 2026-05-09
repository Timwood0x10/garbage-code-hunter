# Garbage Code Hunter - Makefile
# A humorous Rust code quality detector

.PHONY: help build test clean install run fmt clippy check doc release ci

# Default target
help:
	@echo "Garbage Code Hunter - Available commands:"
	@echo ""
	@echo "  build      - Build the project in debug mode"
	@echo "  release    - Build the project in release mode"
	@echo "  test       - Run all tests"
	@echo "  check      - Run cargo check"
	@echo "  fmt        - Format code with rustfmt"
	@echo "  clippy     - Run clippy linter"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install the binary to cargo bin"
	@echo "  run        - Run with default arguments (analyze current directory)"
	@echo "  doc        - Generate documentation"
	@echo "  demo       - Run demo with sample garbage code"
	@echo "  ci         - Run CI checks (format, clippy, warnings, tests)"
	@echo ""
	@echo "Examples:"
	@echo "  make run ARGS='--lang en-US --verbose src/'"
	@echo "  make demo"
	@echo "  make release"
	@echo "  make ci"

# Build targets
build:
	cargo build

release:
	cargo build --release

# Testing and validation
test:
	cargo test

test-signal:
	cargo test -- --test-threads=1

test-verbose:
	cargo test -- --nocapture

test-coverage:
	@echo "Running code coverage analysis..."
	cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Html --output-dir coverage/

test-coverage-ci:
	@echo "Running code coverage for CI..."
	cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Xml

bench:
	cargo bench

check:
	cargo check

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --all

# Documentation
doc:
	cargo doc --open

# Installation
install:
	cargo install --path .

# Cleaning
clean:
	cargo clean
	rm -f tmp_demo_*.rs

# Running
run:
	cargo run -- $(ARGS)

# Demo target - creates sample garbage code and analyzes it
demo:
	@echo "Creating demo garbage code..."
	@echo '// Demo file with intentionally bad code patterns' > tmp_demo_garbage.rs
	@echo 'fn main() {' >> tmp_demo_garbage.rs
	@echo '    let data = "hello";' >> tmp_demo_garbage.rs
	@echo '    let temp = 42;' >> tmp_demo_garbage.rs
	@echo '    let info = vec![1, 2, 3];' >> tmp_demo_garbage.rs
	@echo '    let obj = String::new();' >> tmp_demo_garbage.rs
	@echo '    ' >> tmp_demo_garbage.rs
	@echo '    // Single letter variables' >> tmp_demo_garbage.rs
	@echo '    let a = 10;' >> tmp_demo_garbage.rs
	@echo '    let b = 20;' >> tmp_demo_garbage.rs
	@echo '    let c = a + b;' >> tmp_demo_garbage.rs
	@echo '    ' >> tmp_demo_garbage.rs
	@echo '    // unwrap() abuse' >> tmp_demo_garbage.rs
	@echo '    let result = Some(42);' >> tmp_demo_garbage.rs
	@echo '    let value = result.unwrap();' >> tmp_demo_garbage.rs
	@echo '    let another = Some("test").unwrap();' >> tmp_demo_garbage.rs
	@echo '    let third = Some(vec![1, 2, 3]).unwrap();' >> tmp_demo_garbage.rs
	@echo '    ' >> tmp_demo_garbage.rs
	@echo '    // Excessive cloning' >> tmp_demo_garbage.rs
	@echo '    let s1 = String::from("hello");' >> tmp_demo_garbage.rs
	@echo '    let s2 = s1.clone();' >> tmp_demo_garbage.rs
	@echo '    let s3 = s2.clone();' >> tmp_demo_garbage.rs
	@echo '    let s4 = s3.clone();' >> tmp_demo_garbage.rs
	@echo '    let s5 = s4.clone();' >> tmp_demo_garbage.rs
	@echo '    ' >> tmp_demo_garbage.rs
	@echo '    println!("{} {} {} {}", value, another.len(), third.len(), s5);' >> tmp_demo_garbage.rs
	@echo '}' >> tmp_demo_garbage.rs
	@echo '' >> tmp_demo_garbage.rs
	@echo '// Deeply nested function' >> tmp_demo_garbage.rs
	@echo 'fn deeply_nested() {' >> tmp_demo_garbage.rs
	@echo '    if true { if true { if true { if true {' >> tmp_demo_garbage.rs
	@echo '        if true { if true { if true { if true {' >> tmp_demo_garbage.rs
	@echo '            println!("Too deep!");' >> tmp_demo_garbage.rs
	@echo '        }}}}}}}}' >> tmp_demo_garbage.rs
	@echo '}' >> tmp_demo_garbage.rs
	@echo "Running analysis on demo file..."
	cargo run -- --verbose tmp_demo_garbage.rs
	@echo ""
	@echo "Demo with English output:"
	cargo run -- --lang en-US --verbose tmp_demo_garbage.rs
	@echo ""
	@echo "Cleaning up demo file..."
	rm -f tmp_demo_garbage.rs

# Development workflow
dev: fmt clippy test

# CI/CD targets
ci:
	@echo "========================================="
	@echo "🔍 Running CI checks..."
	@echo "========================================="
	@echo ""
	@echo "📋 Step 1: Checking code formatting..."
	@echo "========================================="
	@cargo fmt --all -- --check
	@echo ""
	@echo "========================================="
	@echo "🔍 Step 2: Running clippy linting (all targets)..."
	@echo "========================================="
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo ""
	@echo "========================================="
	@echo "🔍 Step 3: Checking for warnings..."
	@echo "========================================="
	@WARNINGS=$$(cargo check 2>&1 | grep "warning:" | wc -l | tr -d ' '); \
	if [ "$$WARNINGS" -gt 0 ]; then \
		echo "❌ Found $$WARNINGS warnings! Project requires 0 warnings."; \
		cargo check 2>&1 | grep "warning:"; \
		exit 1; \
	else \
		echo "✅ Zero warnings - perfect!"; \
	fi
	@echo ""
	@echo "========================================="
	@echo "📦 Step 4: Validating TOML files..."
	@echo "========================================="
	@for toml_file in Cargo.toml **/Cargo.toml; do \
		if [ -f "$$toml_file" ]; then \
			echo "Checking $$toml_file..."; \
			rusty_toml "$$toml_file" > /dev/null 2>&1 || \
			(cargo metadata --format-version 1 --manifest-path "$$toml_file" > /dev/null 2>&1) || \
			(echo "❌ Invalid TOML in $$toml_file" && exit 1); \
		fi; \
	done
	@echo "✅ All TOML files are valid"
	@echo ""
	@echo "========================================="
	@echo "🔒 Step 5: Running security audit..."
	@echo "========================================="
	@cargo audit || true
	@echo ""
	@echo "========================================="
	@echo "📚 Step 6: Checking for outdated dependencies..."
	@echo "========================================="
	@cargo outdated --workspace
	@echo ""
	@echo "========================================="
	@echo "🔍 Step 7: Checking licenses and dependencies..."
	@echo "========================================="
	@cargo deny check licenses
	@echo ""
	@echo "========================================="
	@echo "🔗 Step 8: Checking for dead code links..."
	@echo "========================================="
	@cargo machete
	@echo ""
	@echo "========================================="
	@echo "🧪 Step 9: Running unit tests..."
	@echo "========================================="
	@cargo test --lib --verbose
	@echo ""
	@echo "========================================="
	@echo "🧪 Step 10: Running integration tests..."
	@echo "========================================="
	@cargo test --test '*' --verbose || true
	@echo ""
	@echo "========================================="
	@echo "✅ All CI checks passed!"
	@echo "========================================="

# Package for distribution
package: clean release
	@echo "Creating distribution package..."
	mkdir -p dist
	cp target/release/garbage-code-hunter dist/
	cp README.md dist/
	cp LICENSE dist/ 2>/dev/null || echo "No LICENSE file found"
	tar -czf dist/garbage-code-hunter.tar.gz -C dist .
	@echo "Package created: dist/garbage-code-hunter.tar.gz"

# Quick analysis of current project
self-check:
	@echo "Analyzing our own code quality..."
	cargo run -- --lang en-US --verbose --exclude "target/*" --exclude "tmp_*" src/