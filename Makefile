.PHONY: build test check clean setup ci fmt-check

# Build the nazec CLI (native)
build:
	cargo build -p nazec

# Build in release mode
release:
	cargo build -p nazec --release

# WASM-only crates excluded from native builds
WASM_EXCLUDE = --exclude naze-runtime --exclude naze-renderer --exclude naze-playground

# Run all workspace tests
test:
	cargo test --workspace $(WASM_EXCLUDE)

# Type-check without building
check:
	cargo check --workspace

# Format all code
fmt:
	cargo fmt --all

# Lint
lint:
	cargo clippy --workspace $(WASM_EXCLUDE) -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Run CI checks locally (mirrors GitHub Actions)
ci: fmt-check lint test

# Check formatting (without modifying)
fmt-check:
	cargo fmt --all -- --check

# First-time setup
setup:
	bash setup.sh
