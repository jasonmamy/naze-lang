#!/usr/bin/env bash
#
# Build script for Naze VS Code extension
# Run from the repository root or the editors directory
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Determine script and repo directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ "$(basename "$SCRIPT_DIR")" == "editors" ]]; then
    REPO_ROOT="$(dirname "$SCRIPT_DIR")"
else
    REPO_ROOT="$SCRIPT_DIR"
fi

VSCODE_DIR="$SCRIPT_DIR/vscode"
if [[ ! -d "$VSCODE_DIR" ]]; then
    VSCODE_DIR="$REPO_ROOT/editors/vscode"
fi

# Detect platform
detect_platform() {
    case "$(uname -s)" in
        Linux*)  echo "linux-x64" ;;
        Darwin*) echo "darwin-x64" ;;
        MINGW*|MSYS*|CYGWIN*) echo "win32-x64" ;;
        *) error "Unsupported platform: $(uname -s)" ;;
    esac
}

PLATFORM=$(detect_platform)
info "Detected platform: $PLATFORM"

# Check dependencies
check_dependencies() {
    info "Checking dependencies..."

    if ! command -v cargo &> /dev/null; then
        error "cargo not found. Please install Rust: https://rustup.rs"
    fi

    if ! command -v npm &> /dev/null; then
        error "npm not found. Please install Node.js: https://nodejs.org"
    fi

    info "Dependencies OK"
}

# Build LSP server
build_lsp() {
    info "Building LSP server..."
    cd "$REPO_ROOT"
    cargo build -p naze-lsp --release
    info "LSP server built successfully"
}

# Copy LSP binary to extension
copy_lsp_binary() {
    info "Copying LSP binary to extension..."

    mkdir -p "$VSCODE_DIR/bin"

    local src_binary="$REPO_ROOT/target/release/naze-lsp"
    local dst_binary="$VSCODE_DIR/bin/naze-lsp-$PLATFORM"

    # Windows uses .exe extension
    if [[ "$PLATFORM" == "win32-x64" ]]; then
        src_binary="${src_binary}.exe"
        dst_binary="${dst_binary}.exe"
    fi

    if [[ ! -f "$src_binary" ]]; then
        error "LSP binary not found at $src_binary"
    fi

    cp "$src_binary" "$dst_binary"
    chmod +x "$dst_binary"
    info "LSP binary copied to $dst_binary"
}

# Build VS Code extension
build_extension() {
    info "Building VS Code extension..."
    cd "$VSCODE_DIR"

    info "Installing extension dependencies..."
    npm install

    info "Compiling extension TypeScript..."
    npm run compile

    info "Extension built successfully"
}

# Build webview
build_webview() {
    info "Building webview..."
    cd "$VSCODE_DIR/webview"

    info "Installing webview dependencies..."
    npm install

    info "Building webview..."
    npm run build

    info "Webview built successfully"
}

# Package extension
package_extension() {
    info "Packaging extension..."
    cd "$VSCODE_DIR"
    npm run package

    local vsix_file=$(ls -t *.vsix 2>/dev/null | head -1)
    if [[ -n "$vsix_file" ]]; then
        info "Extension packaged: $VSCODE_DIR/$vsix_file"
    fi
}

# Main build process
main() {
    local skip_lsp=false
    local skip_package=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-lsp)
                skip_lsp=true
                shift
                ;;
            --skip-package)
                skip_package=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [options]"
                echo ""
                echo "Options:"
                echo "  --skip-lsp      Skip building the LSP server"
                echo "  --skip-package  Skip packaging the extension"
                echo "  -h, --help      Show this help message"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done

    check_dependencies

    if [[ "$skip_lsp" == false ]]; then
        build_lsp
        copy_lsp_binary
    else
        warn "Skipping LSP build"
    fi

    build_extension
    build_webview

    if [[ "$skip_package" == false ]]; then
        package_extension
    else
        warn "Skipping extension packaging"
    fi

    echo ""
    info "Build complete!"
    echo ""
    echo "To install the extension in VS Code:"
    echo "  1. Open VS Code"
    echo "  2. Press Ctrl+Shift+P (Cmd+Shift+P on macOS)"
    echo "  3. Run 'Extensions: Install from VSIX...'"
    echo "  4. Select: $VSCODE_DIR/naze-lang-*.vsix"
}

main "$@"
