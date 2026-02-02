#!/usr/bin/env bash
set -euo pipefail

echo "=== Naze development environment setup ==="

# Check for C compiler (required by Rust linker)
if ! command -v cc &> /dev/null && ! command -v gcc &> /dev/null; then
    echo "ERROR: No C compiler found. Install build-essential first:"
    echo "  sudo apt-get install -y build-essential   # Debian/Ubuntu"
    echo "  sudo dnf install -y gcc                   # Fedora"
    echo "  xcode-select --install                    # macOS"
    exit 1
fi

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    . "$HOME/.cargo/env"
else
    echo "Rust already installed: $(rustc --version)"
fi

# Add WASM target (needed for naze-runtime in M9+)
echo "Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

# Install wasm-pack (needed for M9+)
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
else
    echo "wasm-pack already installed: $(wasm-pack --version)"
fi

echo ""
echo "=== Setup complete ==="
echo ""
echo "Build the CLI:    cargo build -p nazec"
echo "Run the CLI:      cargo run -p nazec -- new hello"
echo "Run all tests:    cargo test --workspace"
