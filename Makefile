# Garbage Code Hunter - Makefile
# A humorous multi-language code quality detector

CARGO := cargo
PROJECT := garbage-code-hunter
VERSION := $(shell grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

# Colors
BLUE   := \033[34m
GREEN  := \033[32m
YELLOW := \033[33m
RED    := \033[31m
NC     := \033[0m

# Directories
COVERAGE_DIR := target/coverage

.PHONY: help

.DEFAULT_GOAL := help

help:
	@echo "$(BLUE)$(PROJECT) v$(VERSION)$(NC)"
	@echo ""
	@echo "  $(GREEN)Build & Check$(NC)"
	@echo "    make build          - Debug build"
	@echo "    make release        - Release build"
	@echo "    make check          - Full check (format + clippy + compile)"
	@echo "    make fmt            - Format code"
	@echo "    make fmt-check      - Check formatting"
	@echo "    make clippy         - Run clippy linter"
	@echo "    make clean          - Clean artifacts"
	@echo ""
	@echo "  $(GREEN)Test$(NC)"
	@echo "    make test           - Run all tests (serial)"
	@echo "    make test-unit      - Unit tests only"
	@echo "    make test-integration - Integration tests only"
	@echo "    make test-verbose   - Tests with output"
	@echo ""
	@echo "  $(GREEN)Analyze$(NC)"
	@echo "    make run ARGS=...   - Run with arguments"
	@echo "    make demo           - Demo with sample code"
	@echo "    make self-check     - Analyze own code"
	@echo ""
	@echo "  $(GREEN)Coverage & Docs$(NC)"
	@echo "    make coverage       - LLVM coverage summary"
	@echo "    make coverage-html  - LLVM coverage HTML report"
	@echo "    make doc            - Build docs"
	@echo ""
	@echo "  $(GREEN)CI & Package$(NC)"
	@echo "    make ci             - Full CI pipeline"
	@echo "    make install        - Install binary"
	@echo "    make package        - Create distribution tarball"

# ─── Build & Check ────────────────────────────────────────────────────────────

.PHONY: build
build:
	@echo "$(BLUE)Building debug...$(NC)"
	$(CARGO) build
	@echo "$(GREEN)Done$(NC)"

.PHONY: release
release:
	@echo "$(BLUE)Building release...$(NC)"
	$(CARGO) build --release
	@echo "$(GREEN)Done$(NC)"

.PHONY: check
check:
	@echo "$(BLUE)Checking formatting...$(NC)"
	$(CARGO) fmt --all -- --check
	@echo "$(BLUE)Checking compilation...$(NC)"
	$(CARGO) check --workspace
	@echo "$(GREEN)All checks passed$(NC)"

.PHONY: fmt
fmt:
	@echo "$(BLUE)Formatting...$(NC)"
	$(CARGO) fmt --all
	@echo "$(GREEN)Formatted$(NC)"

.PHONY: fmt-check
fmt-check:
	@echo "$(BLUE)Checking formatting...$(NC)"
	$(CARGO) fmt --all -- --check
	@echo "$(GREEN)Formatting OK$(NC)"

.PHONY: clippy
clippy:
	@echo "$(BLUE)Running clippy...$(NC)"
	$(CARGO) clippy --workspace --all-targets --all-features -- \
		-D warnings \
		-W clippy::all \
		-A clippy::too_many_arguments \
		-A clippy::type_complexity
	@echo "$(GREEN)Clippy OK$(NC)"

.PHONY: clean
clean:
	@echo "$(YELLOW)Cleaning...$(NC)"
	$(CARGO) clean
	rm -rf $(COVERAGE_DIR) target/tarpaulin
	rm -f tmp_demo_*.rs
	@echo "$(GREEN)Cleaned$(NC)"

# ─── Test ──────────────────────────────────────────────────────────────────────

.PHONY: test
test:
	@echo "$(BLUE)Running all tests (serial)...$(NC)"
	$(CARGO) test --workspace -- --test-threads=1
	@echo "$(GREEN)All tests passed$(NC)"

.PHONY: test-unit
test-unit:
	@echo "$(BLUE)Running unit tests...$(NC)"
	$(CARGO) test --lib --workspace -- --test-threads=1
	@echo "$(GREEN)Unit tests passed$(NC)"

.PHONY: test-integration
test-integration:
	@echo "$(BLUE)Running integration tests...$(NC)"
	$(CARGO) test --test '*' --workspace -- --test-threads=1
	@echo "$(GREEN)Integration tests passed$(NC)"

.PHONY: test-verbose
test-verbose:
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	$(CARGO) test --tests -- --test-threads=1 --nocapture

# ─── Analyze ───────────────────────────────────────────────────────────────────

.PHONY: run
run:
	@echo "$(BLUE)Running $(PROJECT)...$(NC)"
	$(CARGO) run -- $(ARGS)

.PHONY: demo
demo:
	@echo "$(BLUE)Creating demo garbage code...$(NC)"
	@echo 'fn main() {' > tmp_demo_garbage.rs
	@echo '    let data = "hello";' >> tmp_demo_garbage.rs
	@echo '    let temp = 42;' >> tmp_demo_garbage.rs
	@echo '    let a = 10; let b = 20; let c = a + b;' >> tmp_demo_garbage.rs
	@echo '    let _ = Some(42).unwrap();' >> tmp_demo_garbage.rs
	@echo '    let _ = Some("test").unwrap();' >> tmp_demo_garbage.rs
	@echo '    let _ = Some(vec![1,2,3]).unwrap();' >> tmp_demo_garbage.rs
	@echo '    println!("{}", c);' >> tmp_demo_garbage.rs
	@echo '}' >> tmp_demo_garbage.rs
	@echo 'fn deeply_nested() {' >> tmp_demo_garbage.rs
	@echo '    if true { if true { if true { if true {' >> tmp_demo_garbage.rs
	@echo '        if true { if true { if true { if true {' >> tmp_demo_garbage.rs
	@echo '            println!("deep");' >> tmp_demo_garbage.rs
	@echo '        }}}}}}}}' >> tmp_demo_garbage.rs
	@echo '}' >> tmp_demo_garbage.rs
	$(CARGO) run -- analyze --verbose tmp_demo_garbage.rs
	@echo "$(GREEN)Demo complete$(NC)"
	rm -f tmp_demo_garbage.rs

.PHONY: self-check
self-check:
	@echo "$(BLUE)Analyzing our own code...$(NC)"
	$(CARGO) run -- analyze --lang en-US --verbose \
		--exclude "target/*" --exclude "tmp_*" src/
	@echo "$(GREEN)Self-check complete$(NC)"

# ─── Coverage & Docs ───────────────────────────────────────────────────────────

.PHONY: coverage
coverage:
	@echo "$(BLUE)Generating coverage summary...$(NC)"
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		mkdir -p $(COVERAGE_DIR); \
		$(CARGO) llvm-cov --workspace \
			--ignore-filename-regex="(tests/|_test\\.rs$$)" \
			--summary-only; \
	else \
		echo "$(YELLOW)Install: cargo install cargo-llvm-cov$(NC)"; \
	fi

.PHONY: coverage-html
coverage-html:
	@echo "$(BLUE)Generating coverage HTML...$(NC)"
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		mkdir -p $(COVERAGE_DIR); \
		$(CARGO) llvm-cov --workspace \
			--ignore-filename-regex="(tests/|_test\\.rs$$)" \
			--html --output-dir $(COVERAGE_DIR); \
		echo "$(GREEN)Report: $(COVERAGE_DIR)/index.html$(NC)"; \
	else \
		echo "$(YELLOW)Install: cargo install cargo-llvm-cov$(NC)"; \
	fi

.PHONY: doc
doc:
	$(CARGO) doc --open

# ─── CI & Package ──────────────────────────────────────────────────────────────

.PHONY: ci
ci:
	@echo "$(BLUE)=== CI Pipeline ===$(NC)"
	@echo ""
	@echo "$(BLUE)[1/5] Formatting...$(NC)"
	$(CARGO) fmt --all -- --check
	@echo "$(GREEN)OK$(NC)"
	@echo ""
	@echo "$(BLUE)[2/5] Compilation...$(NC)"
	$(CARGO) check --workspace
	@echo "$(GREEN)OK$(NC)"
	@echo ""
	@echo "$(BLUE)[3/5] Clippy...$(NC)"
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings
	@echo "$(GREEN)OK$(NC)"
	@echo ""
	@echo "$(BLUE)[4/5] Unit tests...$(NC)"
	$(CARGO) test --lib --workspace -- --test-threads=1
	@echo "$(GREEN)OK$(NC)"
	@echo ""
	@echo "$(BLUE)[5/5] Integration tests...$(NC)"
	$(CARGO) test --test '*' --workspace -- --test-threads=1
	@echo ""
	@echo "$(GREEN)✅ CI passed$(NC)"

.PHONY: install
install:
	@echo "$(BLUE)Installing...$(NC)"
	$(CARGO) install --path .
	@echo "$(GREEN)Installed$(NC)"

.PHONY: package
package: release
	@echo "$(BLUE)Packaging...$(NC)"
	mkdir -p dist
	cp target/release/$(PROJECT) dist/
	cp README.md dist/
	cp LICENSE dist/ 2>/dev/null || true
	cp -r example dist/ 2>/dev/null || true
	tar -czf dist/$(PROJECT)-$(VERSION).tar.gz -C dist .
	@echo "$(GREEN)Package: dist/$(PROJECT)-$(VERSION).tar.gz$(NC)"
