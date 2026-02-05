# Naze Editor Extensions

Editor integrations for the Naze declarative UI language.

## VS Code Extension

The VS Code extension provides a complete development environment for Naze, featuring:

### Features

#### Language Support
- **Syntax Highlighting** — Full TextMate grammar with support for elements, properties, keywords, strings, colors, and comments
- **Bracket Matching** — Auto-closing and matching for `{}`, `()`, `[]`, and `""`
- **Code Folding** — Collapse blocks for easier navigation
- **Comment Toggle** — Use `Ctrl+/` to toggle `--` comments

#### IntelliSense
- **Autocomplete** — Context-aware suggestions for:
  - Element types (`column`, `row`, `rect`, `text`, etc.)
  - Properties based on element type
  - Property values (colors, alignments, units)
  - Keywords (`state`, `if`, `each`, `component`, etc.)
- **Hover Documentation** — Detailed docs with examples on hover
- **Go to Definition** — `Ctrl+Click` or `F12` to jump to component/state definitions
- **Find All References** — `Shift+F12` to find all usages
- **Rename Symbol** — `F2` to rename across the file
- **Document Outline** — Navigate via the Outline panel

#### Diagnostics
- **Real-time Error Detection** — Parse errors shown as you type
- **Error Highlighting** — Red squiggles with descriptive messages

#### Code Actions
- **Wrap in Column/Row** — Quickly wrap selected elements
- **Extract to Component** — Refactor selection into a reusable component

#### Visual Editor (Experimental)
- **Block-Based View** — Notion-style visual representation without `{}`
- **Properties Panel** — Edit properties with color pickers and dropdowns
- **Add Elements** — Dropdown menu to insert new elements
- **Live Preview** — Side panel showing rendered output
- **AI Assistant** — Natural language interface for code generation

---

## Building from Source

### Prerequisites

- **Rust** (1.70+) with `cargo`
- **Node.js** (18+) with `npm`
- **VS Code** (1.85+)

### Quick Build (Recommended)

Use the build script to automate all steps:

```bash
# From repository root
./editors/build.sh

# Or from the editors directory
cd editors
./build.sh
```

The script automatically detects your platform (Linux/macOS/Windows) and:
1. Builds the LSP server
2. Copies the binary to the correct location
3. Installs dependencies and compiles the extension
4. Builds the webview
5. Packages the `.vsix` file

**Options:**
- `--skip-lsp` — Skip building the LSP server (if already built)
- `--skip-package` — Skip creating the `.vsix` package

### Manual Build Steps

If you prefer to run steps individually:

```bash
# 1. Build the LSP server
cargo build -p naze-lsp --release

# 2. Copy LSP binary to extension (adjust for your platform)
mkdir -p editors/vscode/bin
cp target/release/naze-lsp editors/vscode/bin/naze-lsp-linux-x64
# On macOS: cp target/release/naze-lsp editors/vscode/bin/naze-lsp-darwin-x64
# On Windows: cp target/release/naze-lsp.exe editors/vscode/bin/naze-lsp-win32-x64.exe

# 3. Build the VS Code extension
cd editors/vscode
npm install
npm run compile

# 4. Build the visual editor webview
cd webview
npm install
npm run build
cd ..

# 5. Package the extension
npm run package
```

This creates `naze-lang-0.1.0.vsix` in `editors/vscode/`.

---

## Installation

### From VSIX (Local Build)

1. Open VS Code
2. Press `Ctrl+Shift+P` → "Extensions: Install from VSIX..."
3. Select the `.vsix` file
4. Reload VS Code

### Development Mode

For active development, use VS Code's extension development host:

```bash
cd editors/vscode
npm install
npm run compile
```

Then press `F5` in VS Code to launch a new window with the extension loaded.

---

## Configuration

The extension can be configured in VS Code settings (`Ctrl+,`):

| Setting | Default | Description |
|---------|---------|-------------|
| `naze.lsp.path` | `""` | Custom path to `naze-lsp` binary. If empty, uses bundled binary. |
| `naze.ai.enabled` | `true` | Enable AI assistant features in visual editor. |
| `naze.ai.apiKey` | `""` | Anthropic API key for AI features. Can also use `ANTHROPIC_API_KEY` env var. |

---

## Usage

### Opening Files

- Create or open a `.naze` file
- Syntax highlighting and IntelliSense activate automatically

### Visual Editor

1. Open a `.naze` file
2. Click the "Open Visual Editor" button in the editor title bar, or:
   - Press `Ctrl+Shift+P` → "Naze: Open Visual Editor"
3. Edit visually — changes sync to source automatically

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Space` | Trigger autocomplete |
| `Ctrl+.` | Show code actions |
| `F12` | Go to definition |
| `Shift+F12` | Find all references |
| `F2` | Rename symbol |
| `Ctrl+/` | Toggle comment |

---

## Troubleshooting

### LSP Not Starting

If you see "Failed to start Naze language server":

1. Check that the LSP binary exists in `bin/` or configure `naze.lsp.path`
2. Ensure the binary has execute permissions: `chmod +x bin/naze-lsp-*`
3. Check Output panel (`Ctrl+Shift+U`) → "Naze Language Server" for errors

### Restart Language Server

If IntelliSense stops working:
- Press `Ctrl+Shift+P` → "Naze: Restart Language Server"

### Visual Editor Not Loading

If the visual editor shows "Loading...":
1. Ensure webview is built: `cd editors/vscode/webview && npm run build`
2. Check Developer Tools (`Ctrl+Shift+I`) for errors

---

## Project Structure

```
editors/
├── README.md                 # This file
└── vscode/
    ├── package.json          # Extension manifest
    ├── tsconfig.json         # TypeScript config
    ├── language-configuration.json
    ├── syntaxes/
    │   └── naze.tmLanguage.json    # TextMate grammar
    ├── src/
    │   ├── extension.ts            # Extension entry point
    │   └── visualEditor/
    │       ├── provider.ts         # Custom editor provider
    │       └── webview.ts          # Webview HTML generator
    ├── webview/                    # Visual editor React app
    │   ├── package.json
    │   ├── vite.config.ts
    │   └── src/
    │       ├── App.tsx
    │       ├── parser.ts           # Client-side Naze parser
    │       └── components/
    │           ├── BlockEditor.tsx
    │           ├── Block.tsx
    │           ├── PropertiesPanel.tsx
    │           ├── AddElementDropdown.tsx
    │           ├── AICommandLine.tsx
    │           └── LivePreview.tsx
    └── bin/                        # LSP binaries (platform-specific)
```

---

## Contributing

See the main [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

### Running Tests

```bash
# LSP tests
cargo test -p naze-lsp

# Extension tests (requires VS Code)
cd editors/vscode
npm test
```

---

## License

See [LICENSE](../LICENSE) in the repository root.
