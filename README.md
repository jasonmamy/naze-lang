# Naze

A declarative UI language that compiles to WebAssembly and renders via Canvas2D, bypassing the DOM entirely.

```naze
app "Hello" {
  column padding: 20px, gap: 16px {
    heading "Hello, Naze!"
    row gap: 12px {
      rect width: 80px, height: 80px, color: #2563eb, radius: 8px
      rect width: 80px, height: 80px, color: #dc2626, radius: 8px
      rect width: 80px, height: 80px, color: #16a34a, radius: 8px
    }
    text "Edit app.naze and run nazec build to see changes."
  }
}
```

## Why Naze?

The modern web stack is a 9-step pipeline built for humans: TypeScript transpiled to JavaScript, bundled by Webpack/Vite, CSS processed through PostCSS/Tailwind, minified, tree-shaken, code-split, downloaded (often 1-5MB of JavaScript), parsed back into an AST, built into a DOM, CSS cascade resolved, layout computed, and finally pixels appear on screen. Most of this exists to manage complexity that humans created for humans. The frameworks, bundlers, and transpilers are developer ergonomics layers that end users never benefit from.

AI is writing more and more of this code. AI doesn't need developer ergonomics. It doesn't need readable class names or semantic HTML. It could target something far more direct.

**The key insight: WASM already runs in every major browser.** Chrome, Firefox, Safari, Edge all have WASM runtimes today. Projects like Flutter/web, Figma, and game engines already prove you can bypass the DOM entirely by rendering through WASM + Canvas. The runtime isn't the missing piece. The infrastructure exists. What's missing is the **language layer** -- a declarative, intent-based language designed for AI to author (and humans to read) that compiles down to WASM.

Naze is that language. A purpose-built, AI-native UI format where:

- **Intent goes straight to pixels.** No bundler. No framework selection. No "React or Vue or Svelte." No Webpack config. The compiler emits a compact binary render tree; the runtime renders it directly.
- **Apps are kilobytes, not megabytes.** The Naze runtime is 69KB. A typical app's render tree is hundreds of bytes. Compare that to the megabytes of JavaScript, CSS frameworks (95% unused), and polyfills that a typical SPA ships.
- **The syntax is readable but compilable.** `.naze` files read like a document describing what the UI should look like. A non-developer can understand them. An AI can generate them reliably. No JSX, no template literals, no CSS-in-JS.
- **Compiled, not interpreted.** Naze is not another runtime-interpreted language. The compiler does the heavy lifting ahead of time -- parsing, type checking, import resolution, component inlining, dead code elimination -- and emits a compact binary. The runtime is a thin executor that deserializes and renders. No JIT, no eval, no parsing at runtime. This is closer to how C or Rust works than how JavaScript or Python works.
- **One source, every platform.** The same `.naze` file targets web (WASM + Canvas), desktop (native renderer), and mobile -- without platform-specific code.

### What this enables

- **True cross-platform from a single source.** One `.naze` file compiles to web (WASM + Canvas), desktop (native window), and mobile (Android/iOS). No React Native, no Electron, no Flutter -- one language, every platform, identical output.
- **Purpose-built AI models.** Today's coding LLMs are 70B-400B+ parameters because they're trained on 50+ languages, hundreds of frameworks, and millions of patterns. Naze is one language with a constrained grammar -- one way to express layout, data binding, events. A fine-tuned 3-7B model on `.naze` could match or outperform a general-purpose 70B model at Naze generation specifically. That's a model that runs locally on a laptop, offline, at zero cost -- not a cloud compromise, but potentially the *better* experience. The language is the constraint that makes this tractable.
- **Instant app generation.** Because the language is declarative and the compiler is fast, the loop from "describe what you want" to "see it running" collapses. No `npm install`, no build config, no dependency resolution. `nazec build` produces a working app in milliseconds.
- **Auditable by anyone.** `.naze` files are plain text, readable by non-developers. A product manager, designer, or client can open the source and understand the structure. AI-generated code becomes inspectable, not a black box.
- **Tiny attack surface.** No `node_modules`. No supply chain of thousands of transitive dependencies. The compiler is a single Rust binary. The runtime is 69KB of WASM. There is very little to exploit.
- **Testing built in from the ground up.** Tests are `.test.naze` files written in the same language as the app -- not a separate framework, not Jest, not Playwright. Component tests render with props and assert output; flow tests simulate multi-page user journeys. `nazec test` runs everything. Because the language is declarative and the output is deterministic (same input always produces the same render tree), tests are predictable and reproducible. AI generates tests alongside app code, and the compiler validates both.
- **Conversational development.** The sub-second rebuild cycle makes voice-driven development practical: speak a change, a local LLM edits the `.naze` source, `nazec run` hot-reloads, and you see the result before you finish your next sentence. The constrained grammar means a small local model generates correctly, and the deterministic output means the model can predict exactly what you'll see.
- **Offline-first development.** With a local Naze-trained LLM and the `nazec` compiler, you can generate and build apps with no internet connection. No CDN, no package registry, no cloud build service required.

The competitive moat is the language, not the runtime. Whoever designs the right AI-native language wins, because the execution layer is commoditized. Think about what happened with JavaScript: the language was the innovation, not the browser.

See [docs/BRAINSTORM.md](docs/BRAINSTORM.md) for the full design rationale and [docs/ROADMAP.md](docs/ROADMAP.md) for the long-term vision.

## Status

**Phase 1 (Proof of Life) is complete.** `nazec new hello && cd hello && nazec build` produces a `dist/` directory with WASM + HTML that renders colored rectangles and text in the browser. Runtime binary is 69KB.

See [docs/MVP.md](docs/MVP.md) for Phase 1 summary and [docs/PHASE2.md](docs/PHASE2.md) for Phase 2 planning.

## Quick Start

```bash
# First-time setup (installs Rust toolchain + wasm-pack)
bash setup.sh

# Build the compiler
cargo build -p nazec

# Create and build a project
nazec new hello
cd hello
nazec build

# Open dist/index.html in a browser (requires a local server for WASM)
python3 -m http.server -d dist 8080
# Then visit http://localhost:8080
```

## Tutorial: Hello World from Scratch

This walks through creating a project, writing a `.naze` file, building it, and viewing it in the browser.

### 1. Install

```bash
# Clone the repo and set up the toolchain
git clone https://github.com/anthropics/naze-lang.git
cd naze-lang
bash setup.sh

# Build the compiler
cargo build -p nazec
```

After this, the `nazec` binary is at `target/debug/nazec`. You can add it to your PATH or use the full path.

### 2. Create a project

```bash
nazec new hello
```

This creates a `hello/` directory with two files:

```
hello/
  naze.toml       # Project manifest
  app.naze        # Entry file
  components/     # Directory for reusable components
```

`naze.toml` defines the project:

```toml
[app]
name = "hello"
version = "0.1.0"

[build]
entry = "app.naze"
output = "dist/"
```

`app.naze` is the generated starter file:

```naze
-- hello
-- Created with nazec

app "hello" {
  column padding: 20px, gap: 16px {
    heading "Hello, hello!"

    row gap: 12px {
      rect width: 80px, height: 80px, color: #2563eb, radius: 8px
      rect width: 80px, height: 80px, color: #dc2626, radius: 8px
      rect width: 80px, height: 80px, color: #16a34a, radius: 8px
    }

    text "Edit app.naze and run nazec build to see changes."
  }
}
```

The language is declarative — `column` lays children out vertically, `row` horizontally, `rect` draws a colored rectangle, `heading` and `text` render text. Properties like `padding`, `gap`, `color`, and `radius` are passed inline.

### 3. Build

```bash
cd hello
nazec build
```

Output:

```
building hello v0.1.0
  resolving...
  type checking...
  compiling...
  writing dist/...
  done: runtime 75KB + app data 567B
```

This produces four files in `dist/`:

```
dist/
  index.html              HTML shell with canvas and bootstrap script
  naze_runtime.js         WASM loader (generated by wasm-pack)
  naze_runtime_bg.wasm    Runtime binary (75KB) — layout + Canvas2D renderer
  app_data.bin            Your app's serialized render tree (567 bytes)
```

### 4. View in the browser

WASM files must be served over HTTP (they won't load via `file://`). Start any local server:

```bash
# Python
python3 -m http.server -d dist 8080

# Node
npx serve dist

# Or any static file server
```

Open `http://localhost:8080` in your browser. You'll see a heading, three colored rounded rectangles, and a line of text rendered on a full-page canvas.

### 4b. Preview locally without a browser (Linux)

Instead of serving files and opening a browser, you can preview directly in a native desktop window:

```bash
nazec run
```

This opens a window rendering your app using the same `app_data.bin` that the browser would use. No HTTP server, no WASM, no browser required -- just `nazec build` then `nazec run`.

Currently Linux only. The native renderer uses a software rasterizer (tiny-skia) rather than Canvas2D, so there may be minor visual differences in font metrics or anti-aliasing compared to the browser. The layout and structure are identical -- both use the same layout engine and the same serialized render tree.

### 5. Edit and rebuild

Open `hello/app.naze` in your editor and make changes. For example, replace the content with a dashboard layout:

```naze
app "My Dashboard" {
  column gap: 0px {
    container padding: 16px, color: #1e293b {
      heading "Dashboard" color: #ffffff
    }
    column padding: 20px, gap: 16px {
      heading "Overview"
      row gap: 16px {
        container padding: 16px, color: #eff6ff, radius: 8px, width: 180px {
          column gap: 4px {
            text "Revenue"
            heading "$12,345" font-size: 24px
          }
        }
        container padding: 16px, color: #f0fdf4, radius: 8px, width: 180px {
          column gap: 4px {
            text "Users"
            heading "1,234" font-size: 24px
          }
        }
      }
    }
  }
}
```

Rebuild and refresh:

```bash
nazec build
# Refresh the browser
```

### 6. Type-check without building

```bash
nazec check
```

If there are errors, you'll see diagnostics with source snippets:

```
error: type mismatch for prop 'width' on 'rect': expected number, got text
  --> app.naze:5:5
    |
  5 |     rect width: "oops", color: #ff0000
    |     ^
```

Use `--format json` for machine-readable output:

```bash
nazec check --format json
```

### 7. Add a reusable component

Create `components/pill.naze`:

```naze
component pill(color: color, size: number = 60px) {
  rect width: size, height: 32px, color: color, radius: 16px
}
```

Use it in `app.naze`:

```naze
use components/pill

app "Pills" {
  column padding: 20px, gap: 12px {
    heading "Status Indicators"
    row gap: 8px {
      pill color: #22c55e
      pill color: #eab308
      pill color: #ef4444, size: 80px
    }
  }
}
```

Build and refresh — the component is inlined at compile time with prop values substituted. The three `pill` invocations become three `rect` elements with the colors and sizes filled in.

### 8. Browse all examples

The repository includes 18 example `.naze` files demonstrating various features. You can browse them interactively with the gallery command:

```bash
# From the repository root
nazec gallery
```

This builds all examples to `examples/dist/`, starts a local server, and opens your browser to an interactive gallery. Click any example name in the sidebar to instantly switch between demos — no page reload required.

To just build the gallery without serving:

```bash
nazec gallery --build
# Then serve manually: cd examples/dist && python3 -m http.server 8000
```

Examples include:
- **Layout**: `rows`, `columns`, `grid`, `nested`, `padding`
- **Styling**: `colors`, `rounded`, `typography`
- **Components**: `component-basic`, `component-props`, `multi-component`, `slots`
- **State**: `counter`, `conditional`
- **Apps**: `hello`, `boxes`, `app-shell`, `dashboard-static`

## Project Structure

```
naze-lang/
  crates/
    nazec/            CLI binary — new, build, check, run, parse commands
    naze-parser/      PEG parser (pest) — .naze source to AST
    naze-compiler/    Type checker + binary serializer
    naze-ir/          Shared IR types (minimal deps, used by both native + WASM)
    naze-runtime/     WASM entry point — deserializes, lays out, renders
    naze-layout/      Layout engine — row, column, stack, grid
    naze-renderer/    Canvas2D renderer (web-sys)
    naze-native/      Standalone native viewer for app_data.bin
  examples/           18 example .naze files
  docs/
    ROADMAP.md        Long-term vision (Phase 1-5)
    PHASE2.md         Phase 2 milestone tracker
    MVP.md            Phase 1 summary
    LANGUAGE.md       Language reference
    PROTOTYPE.md      Component architecture spec
    BRAINSTORM.md     Original design brainstorm
    WISH_LIST.md      Speculative ideas (voice-driven dev, etc.)
    LLM.md            Local LLM fine-tuning plan
```

## CLI

```
nazec new <name>        Create a new project
nazec build             Compile to dist/ (WASM + HTML)
nazec run               Preview in a native desktop window (Linux)
nazec check             Type-check without building
nazec parse <file>      Dump AST as JSON
nazec gallery           Build and serve interactive example gallery
nazec gallery --build   Build gallery only (no server)
nazec build --format json   Machine-readable error output
```

## Build Commands

```bash
cargo build -p nazec                  # Build the CLI
cargo test --workspace                # Run all tests
cargo check --workspace               # Type-check all crates
cargo fmt --all                       # Format
cargo clippy --workspace              # Lint
```

The runtime WASM crate is pre-built and embedded in the `nazec` binary via `include_bytes!`. To rebuild it after changes to `naze-runtime`, `naze-layout`, or `naze-renderer`:

```bash
cd crates/naze-runtime
wasm-pack build --target web --release
```

## Language Features (Phase 1)

- `app` blocks with title
- Layout: `row`, `column`, `stack`, `grid`, `container`, `spacer`
- Elements: `rect`, `text`, `heading`
- Properties: dimensions (`width`, `height`, `padding`, `gap`), colors (`#hex`), `radius`
- Components with typed parameters and defaults
- `use` imports for component reuse
- `--` line comments

See [docs/LANGUAGE.md](docs/LANGUAGE.md) for the full language reference with property tables and layout semantics.

## Architecture

The compiler (native Rust) parses `.naze` files, resolves imports, type-checks, and serializes a compact render tree to binary. The pre-built runtime WASM deserializes this data, computes layout, and renders to a Canvas2D element. The runtime is embedded in the `nazec` binary so builds require no external tools.

```
.naze source → parse → resolve → typecheck → serialize → app_data.bin
                                                           ↓
                               ┌───────────────────────────┤
                               ↓                           ↓
                 nazec run (Linux)                 browser (all platforms)
                 deserialize → layout → tiny-skia   runtime.wasm → layout → Canvas2D
```

## License

MIT
