.PHONY: build test check clean setup ci fmt-check package try

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

# Package toolkit for distribution
package: release
	bash scripts/package-toolkit.sh

TOOLKIT_DIR = /tmp/naze-toolkit

# Extract toolkit to /tmp for end-user experience testing
try: package
	rm -rf $(TOOLKIT_DIR)
	mkdir -p $(TOOLKIT_DIR)
	tar xzf target/package/naze-toolkit-linux-x86_64.tar.gz -C /tmp
	@echo ""
	@echo "Toolkit extracted to $(TOOLKIT_DIR)"
	@echo ""
	@echo "Test it like an end user:"
	@echo "  cd $(TOOLKIT_DIR)/starter"
	@echo "  ../bin/nazec build"
	@echo "  ../bin/nazec dev"
	@echo ""
	@echo "Create a new app:"
	@echo "  cd $(TOOLKIT_DIR)"
	@echo "  bin/nazec new my-app"
	@echo "  cd my-app && ../bin/nazec dev"
	@echo ""
	@echo "Point an AI agent at: $(TOOLKIT_DIR)/README.md"

# First-time setup
setup:
	bash setup.sh
