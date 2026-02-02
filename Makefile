.PHONY: build test check clean setup

# Build the nazec CLI (native)
build:
	cargo build -p nazec

# Build in release mode
release:
	cargo build -p nazec --release

# Run all workspace tests
test:
	cargo test --workspace

# Type-check without building
check:
	cargo check --workspace

# Format all code
fmt:
	cargo fmt --all

# Lint
lint:
	cargo clippy --workspace -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# First-time setup
setup:
	bash setup.sh
