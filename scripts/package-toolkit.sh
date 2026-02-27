#!/usr/bin/env bash
set -euo pipefail

# Naze AI Developer Toolkit Packager
# Assembles a self-contained distribution: binary + docs + examples + starter project

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TOOLKIT_NAME="naze-toolkit"
ARCH="linux-x86_64"
PACKAGE_DIR="$REPO_ROOT/target/package/$TOOLKIT_NAME"
ARCHIVE_NAME="$TOOLKIT_NAME-$ARCH.tar.gz"

echo "=== Naze AI Developer Toolkit Packager ==="
echo ""

# Step 1: Build release binary if not present
BINARY="$REPO_ROOT/target/release/nazec"
if [ ! -f "$BINARY" ]; then
    echo "Building release binary..."
    cd "$REPO_ROOT"
    cargo build -p nazec --release
else
    echo "Using existing release binary"
fi
ls -lh "$BINARY"
echo ""

# Step 2: Create package directory structure
rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"/{bin,reference,examples/components,starter/components,.claude/skills}

# Step 3: Copy and strip binary
echo "Copying nazec binary..."
cp "$BINARY" "$PACKAGE_DIR/bin/nazec"
chmod +x "$PACKAGE_DIR/bin/nazec"
if command -v strip &>/dev/null; then
    echo "Stripping binary..."
    strip "$PACKAGE_DIR/bin/nazec" 2>/dev/null || true
fi

# Step 4: Copy reference docs
echo "Copying reference documentation..."
cp "$REPO_ROOT/docs/AGENTS.md" "$PACKAGE_DIR/reference/"
cp "$REPO_ROOT/docs/LANGUAGE.md" "$PACKAGE_DIR/reference/"

# Step 5: Copy curated examples
echo "Copying curated examples..."
for f in \
    hello.naze \
    counter.naze \
    conditional.naze \
    input.naze \
    validation.naze \
    computed.naze \
    pipeline.naze \
    functions.naze \
    match.naze \
    navigation.naze \
    dashboard-static.naze \
    component-basic.naze \
    slots.naze \
    overlay-dialog.naze \
    data-fetch.naze
do
    cp "$REPO_ROOT/examples/$f" "$PACKAGE_DIR/examples/"
done

for f in color-box.naze slot-card.naze panel.naze page-layout.naze; do
    cp "$REPO_ROOT/examples/components/$f" "$PACKAGE_DIR/examples/components/"
done

# Step 6: Create starter project
echo "Creating starter project..."
cat > "$PACKAGE_DIR/starter/naze.toml" << 'TOML'
[app]
name = "my-app"
version = "0.1.0"

[build]
entry = "app.naze"
output = "dist/"
TOML

cat > "$PACKAGE_DIR/starter/app.naze" << 'NAZE'
-- my-app
-- Created with Naze AI Developer Toolkit

app "my-app" {
  state count = 0
  state items = ["Naze", "is", "declarative"]

  column padding: 20px, gap: 16px {
    heading "Welcome to my-app!"
    text "Count: {count}"

    row gap: 12px {
      rect width: 120px, height: 50px, color: #2563eb, radius: 8px, role: "button", label: "Increment counter" {
        text "Increment"
        on click: set count = count + 1
      }
      rect width: 120px, height: 50px, color: #dc2626, radius: 8px, role: "button", label: "Reset counter" {
        text "Reset"
        on click: set count = 0
      }
    }

    if count > 0 {
      text "You clicked {count} times!"
    } else {
      text "Click Increment to get started."
    }

    heading "Items:"
    each item in items {
      text "- {item}"
    }

    text "Edit app.naze and rebuild to see changes."
  }
}
NAZE

# Step 6b: Install Claude Code skill for auto-discovery
echo "Installing Claude Code skill..."
cp -r "$REPO_ROOT/naze-dev" "$PACKAGE_DIR/.claude/skills/naze-dev"

# Step 6c: Generate bootstrap CLAUDE.md
echo "Generating CLAUDE.md..."
cat > "$PACKAGE_DIR/CLAUDE.md" << 'CLAUDEMD'
# Naze AI Developer Toolkit

You are working with **Naze** — a declarative UI language that compiles
.naze files to Canvas2D via WASM. No DOM, no CSS, no JavaScript.

## Detailed References

- Full syntax: @.claude/skills/naze-dev/references/language.md
- Code examples: @.claude/skills/naze-dev/references/examples.md
- CLI commands: @.claude/skills/naze-dev/references/cli.md

## Essential Rules

1. Every entry file: `app "Title" { ... }`
2. Layout: `row` (horizontal), `column` (vertical), `container` (styled box)
3. State: `state name = value` — mutated with `on click: set name = expr`
4. Props use colon: `width: 200px` — colors unquoted: `color: #2563eb`
5. Identifiers are kebab-case: `font-size`, `my-component`
6. Inputs require `bind:` — `input bind: name`
7. Components: one per file, `use components/name` to import
8. No semicolons. No HTML/CSS/JS concepts. No DOM.

## Commands

    bin/nazec new my-app    # Scaffold project
    bin/nazec build         # Compile to dist/
    bin/nazec dev           # Dev server with hot reload
    bin/nazec check         # Type-check only
    bin/nazec serve         # Production SSR server

Build output goes to dist/ — serve over HTTP (WASM requires it).

## Directory Layout

    bin/nazec                     compiler binary
    reference/                    full language docs
    examples/                     15 curated .naze examples
    starter/                      ready-to-build project
    .claude/skills/naze-dev/      skill (auto-discovered)

## Global Skill Install

To use the naze-dev skill in all projects:

    mkdir -p ~/.claude/skills
    cp -r .claude/skills/naze-dev ~/.claude/skills/naze-dev
CLAUDEMD

# Step 7: Generate toolkit README
echo "Generating README.md..."
cat > "$PACKAGE_DIR/README.md" << 'README'
# Naze AI Developer Toolkit

Naze is a declarative UI language that compiles `.naze` files to Canvas2D via
WASM. No DOM, no CSS, no JavaScript. One canonical form per concept.

This toolkit contains everything needed to build Naze applications.

## Quick Setup

1. Add the compiler to your PATH (optional):
   ```
   export PATH="$PWD/bin:$PATH"
   ```

2. Build the starter project:
   ```
   cd starter
   ../bin/nazec build
   ```

3. Serve it (WASM requires HTTP, not file://):
   ```
   python3 -m http.server -d dist 8080
   ```

4. Open http://localhost:8080

To have an AI agent build apps for you, tell it to read this file and then
read `reference/AGENTS.md` for the full language reference.

---

## For AI Agents

You are working with the **Naze programming language**. This toolkit has
everything you need to write and build Naze applications.

### Directory Layout

```
bin/nazec              -- compiler (self-contained, embeds WASM runtime)
reference/AGENTS.md    -- LANGUAGE REFERENCE — read this for all syntax and rules
reference/LANGUAGE.md  -- extended reference with additional detail
examples/              -- 15 curated examples covering all major features
starter/               -- ready-to-build project (naze.toml + app.naze)
.claude/skills/naze-dev/ -- Claude Code skill (auto-discovered by Claude Code)
CLAUDE.md              -- Bootstrap context for Claude Code
```

### How to Build

```bash
# Build a project (run from its directory, where naze.toml lives):
/absolute/path/to/bin/nazec build

# Output goes to dist/ — serve with any HTTP server:
python3 -m http.server -d dist 8080

# Create a new project from scratch:
/absolute/path/to/bin/nazec new my-project

# Type-check without building:
/absolute/path/to/bin/nazec check

# Dev server with hot reload:
/absolute/path/to/bin/nazec dev
```

### Essential Rules

1. Every entry file has one `app "Title" { ... }` block
2. State: `state name = value` — mutated with `on click: set name = expr`
3. Layout: `row` (horizontal), `column` (vertical), `container` (styled box)
4. Props use colon syntax: `width: 200px` not `width=200px`
5. Colors are unquoted hex: `color: #2563eb` not `color: "#2563eb"`
6. Identifiers are kebab-case: `font-size`, `my-component`
7. Components are one-per-file, imported with `use components/name`
8. No semicolons. No HTML/CSS/JS concepts. No DOM.

### What to Read Next

**Read `reference/AGENTS.md`** for the complete language reference — all
elements, props, state, events, components, routing, data fetching, pipelines,
pattern matching, animation, theming, and more.

### Examples

| File | Demonstrates |
|------|-------------|
| hello.naze | Minimal app |
| counter.naze | State and events |
| conditional.naze | if/else and iteration |
| input.naze | Text input with two-way binding |
| validation.naze | Form validation with error display |
| computed.naze | Derived values that auto-update |
| pipeline.naze | Data transforms: filter, sort, map, sum |
| functions.naze | Pure functions with types |
| match.naze | Pattern matching |
| navigation.naze | Multi-page app with routing |
| dashboard-static.naze | Complex layout composition |
| component-basic.naze | Component import and reuse |
| slots.naze | Component content slots |
| overlay-dialog.naze | Modal dialog with backdrop |
| data-fetch.naze | API data with loading states |

## Claude Code (AI-Assisted Development)

Start Claude Code from this directory — CLAUDE.md and the naze-dev skill
are auto-discovered. No setup required.

To install the skill globally (all projects):
\`\`\`
mkdir -p ~/.claude/skills
cp -r .claude/skills/naze-dev ~/.claude/skills/naze-dev
\`\`\`
README

# Step 8: Create archive
echo ""
echo "Creating archive..."
cd "$REPO_ROOT/target/package"
tar czf "$ARCHIVE_NAME" "$TOOLKIT_NAME"

# Step 9: Report
ARCHIVE_PATH="$REPO_ROOT/target/package/$ARCHIVE_NAME"
echo ""
echo "=== Package Complete ==="
echo "Archive: $ARCHIVE_PATH"
ls -lh "$ARCHIVE_PATH"
echo ""
BINARY_SIZE=$(ls -lh "$PACKAGE_DIR/bin/nazec" | awk '{print $5}')
FILE_COUNT=$(find "$PACKAGE_DIR" -type f | wc -l)
echo "Contents: $FILE_COUNT files, binary $BINARY_SIZE"
echo ""
echo "To test:"
echo "  cd target/package && tar xzf $ARCHIVE_NAME"
echo "  cd $TOOLKIT_NAME/starter && ../bin/nazec build"
