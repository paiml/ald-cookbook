# ALD Cookbook Makefile
# Extreme TDD automation with Toyota Production System principles

.PHONY: help build test test-fast test-all test-property coverage lint fmt check \
        validate quick-validate examples book book-serve clean mutants audit

# Default target
help:
	@echo "ALD Cookbook - Extreme TDD Makefile"
	@echo ""
	@echo "Testing Targets:"
	@echo "  test          Run all tests"
	@echo "  test-fast     Run fast tests only (50 proptest cases)"
	@echo "  test-all      Run comprehensive tests (500 proptest cases)"
	@echo "  test-property Run property-based tests"
	@echo "  coverage      Generate coverage report (95% minimum)"
	@echo "  mutants       Run mutation testing (80% minimum)"
	@echo ""
	@echo "Quality Targets:"
	@echo "  lint          Run clippy linter"
	@echo "  fmt           Format code"
	@echo "  check         Cargo check"
	@echo "  audit         Security audit"
	@echo "  validate      Full validation pipeline"
	@echo "  quick-validate Quick validation (no mutants)"
	@echo ""
	@echo "Build Targets:"
	@echo "  build         Build library"
	@echo "  build-release Build release"
	@echo "  examples      Build all examples"
	@echo ""
	@echo "Documentation:"
	@echo "  book          Build mdBook documentation"
	@echo "  book-serve    Serve book locally"
	@echo ""
	@echo "Utility:"
	@echo "  clean         Clean build artifacts"

# ============================================================================
# Testing Targets
# ============================================================================

# Standard test run
test:
	cargo test --all-features

# Fast tests for development (50 proptest cases)
test-fast:
	PROPTEST_CASES=50 cargo test --lib

# Comprehensive tests (500 proptest cases)
test-all:
	PROPTEST_CASES=500 cargo test --all-features
	cargo test --test proptest_format
	cargo test --test proptest_transforms
	cargo test --test proptest_drift

# Property-based tests only
test-property:
	cargo test --test proptest_format -- --nocapture
	cargo test --test proptest_transforms -- --nocapture
	cargo test --test proptest_drift -- --nocapture

# Documentation tests
test-doc:
	cargo test --doc

# Coverage with threshold enforcement
coverage:
	@echo "Running coverage analysis (95% minimum)..."
	cargo llvm-cov --all-features --fail-under 95
	cargo llvm-cov --html --all-features
	@echo "Coverage report: target/llvm-cov/html/index.html"

# Coverage report only (no threshold)
coverage-report:
	cargo llvm-cov --all-features --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

# Mutation testing
mutants:
	@echo "Running mutation testing (80% minimum)..."
	cargo mutants --timeout 120 -- --lib

# ============================================================================
# Quality Targets
# ============================================================================

# Clippy linter
lint:
	cargo clippy -- -D warnings
	cargo clippy --examples -- -D warnings
	cargo clippy --tests -- -D warnings

# Format code
fmt:
	cargo fmt

# Format check
fmt-check:
	cargo fmt --check

# Cargo check
check:
	cargo check --all-features --all-targets

# Security audit
audit:
	cargo audit

# Full validation pipeline (Toyota Way: Jidoka)
validate: fmt-check lint check test test-property
	@echo ""
	@echo "============================================"
	@echo "Full validation passed!"
	@echo "============================================"

# Quick validation for development
quick-validate: fmt-check lint check test-fast
	@echo ""
	@echo "============================================"
	@echo "Quick validation passed!"
	@echo "============================================"

# ============================================================================
# Build Targets
# ============================================================================

# Debug build
build:
	cargo build

# Release build
build-release:
	cargo build --release

# Build with all features
build-full:
	cargo build --all-features

# Build all examples
examples:
	cargo build --examples

# Run specific example
run-%:
	cargo run --example $*

# ============================================================================
# Documentation Targets
# ============================================================================

# Build mdBook
book:
	@echo "Building mdBook documentation..."
	cd book && mdbook build
	@echo "Book built: book/target/book/index.html"

# Serve book locally
book-serve:
	cd book && mdbook serve --open

# Build and check book
book-check:
	cd book && mdbook build
	@test -f book/target/book/index.html || (echo "Book index.html not found" && exit 1)
	@test -f book/target/book/print.html || (echo "Book print.html not found" && exit 1)
	@echo "Book structure verified"

# Generate API docs
docs:
	cargo doc --all-features --no-deps
	@echo "API docs: target/doc/ald_cookbook/index.html"

# ============================================================================
# Example Execution
# ============================================================================

# Run all core examples
examples-run:
	@echo "Running core examples..."
	cargo run --example create_ald_from_arrow
	cargo run --example load_ald_basic
	cargo run --example convert_csv_to_ald
	cargo run --example transform_filter
	cargo run --example drift_ks_test
	cargo run --example federated_iid_split
	cargo run --example registry_publish
	@echo "All core examples passed"

# Idempotency check
idempotency:
	@echo "Checking idempotency..."
	@cargo run --example create_ald_from_arrow 2>/dev/null | grep -E "Rows|File size" > /tmp/run1.txt
	@cargo run --example create_ald_from_arrow 2>/dev/null | grep -E "Rows|File size" > /tmp/run2.txt
	@diff /tmp/run1.txt /tmp/run2.txt && echo "Idempotency check passed" || (echo "Idempotency check FAILED" && exit 1)

# ============================================================================
# Utility Targets
# ============================================================================

# Clean build artifacts
clean:
	cargo clean
	rm -rf book/target/book

# Clean and rebuild
rebuild: clean build

# Full CI simulation
ci: validate coverage book-check idempotency
	@echo ""
	@echo "============================================"
	@echo "Full CI simulation passed!"
	@echo "============================================"

# Pre-commit hook
pre-commit: quick-validate
	@echo "Pre-commit checks passed"

# ============================================================================
# WASM Targets
# ============================================================================

# Build for WASM
wasm:
	cargo build --target wasm32-unknown-unknown --features browser

# ============================================================================
# Toyota Way Quality Gates
# ============================================================================

# Quality gate: test count (Kaizen metric)
quality-gate:
	@echo "Checking quality gates..."
	@TEST_COUNT=$$(cargo test --lib 2>&1 | grep -E "^test result" | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+"); \
	if [ "$$TEST_COUNT" -lt 50 ]; then \
		echo "FAILED: Test count $$TEST_COUNT < 50 minimum"; \
		exit 1; \
	else \
		echo "PASSED: $$TEST_COUNT tests (≥50 minimum)"; \
	fi

# Full quality verification (Jidoka)
verify-all: validate coverage mutants quality-gate
	@echo ""
	@echo "============================================"
	@echo "All quality gates passed!"
	@echo "  - Format: OK"
	@echo "  - Lint: OK"
	@echo "  - Tests: OK"
	@echo "  - Coverage: ≥95%"
	@echo "  - Mutations: ≥80%"
	@echo "  - Test count: ≥50"
	@echo "============================================"
