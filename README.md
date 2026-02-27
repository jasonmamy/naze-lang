# Naze

A declarative, AI-native UI language that compiles to WebAssembly and renders via Canvas2D, bypassing the DOM entirely. Designed by AI, for AI -- this entire codebase (compiler, runtime, layout engine, tooling) was built through human-AI collaboration using Claude Code.

> **Note:** Naze is a research project and work in progress. It is not yet suitable for production applications. The language, compiler, and runtime are functional and tested (400+ tests, 109 examples), but APIs may change, features may be incomplete, and the ecosystem is early-stage. Contributions and feedback are welcome.

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

## The Vision

The web is shifting. Users increasingly interact with the internet through AI agents -- asking questions, delegating tasks, composing services -- rather than manually navigating pages and clicking links. As this transition accelerates, the assumptions underneath the web start to break. HTML was designed for documents that humans read. CSS was designed for layouts that humans see. JavaScript was designed for interactions that humans initiate. Search engines index text that humans scan.

None of this was designed for a world where the primary consumer of a web application is an AI agent acting on a user's behalf.

Naze is built for that world. It is a language and platform designed so that AI agents can **author**, **understand**, and **interact with** applications natively -- not by scraping HTML and guessing at CSS selectors, but through a structured, typed, semantic format that an agent can parse, reason about, and compose programmatically. Applications become machine-comprehensible artifacts: an agent can read a Naze binary and know every piece of state the app tracks, every action it supports, and every condition that changes its behavior. Discovery becomes structural ("find apps with a cart, checkout, and payment flow") rather than keyword-based.

This is solving problems of the future -- but it also solves problems of today. The modern web stack is bloated, fragmented, and expensive for AI to work with. Naze replaces that with a single language, a 395KB runtime, and a compile pipeline with no middle layers. The same design choices that make Naze agent-native also make it faster to build, smaller to ship, and cheaper to maintain right now.

## Why Naze Exists

AI is writing more and more code. But the languages it writes in -- React, Vue, Angular, the entire HTML/CSS/JS stack -- were designed for human developers. They scatter information across dozens of files, offer multiple valid ways to express the same concept, and rely on implicit framework behavior invisible in the source code. When an AI agent modifies one component in a React app, it reads the CSS module, the TypeScript interfaces, the Redux store, the custom hooks, the routing config. As the app grows, this cost grows superlinearly.

Naze asks: **what if a language were designed from scratch so that AI cost scales linearly with application size -- and stays there?**

### The framework: Token Complexity

We propose **Token Complexity** (&Lambda;(n)) as the Big O of AI development cost:

> **&Lambda;(L, n) = n &times; &lambda; &times; &sigma; &times; (1 + r)**

Where **&lambda;** is tokens per component (verbosity), **&sigma;** is files the AI must read per change (scatter), and **r** is the retry rate (how often the AI generates incorrect code). The critical parameter is **&sigma;** -- it determines the complexity class. If &sigma; = 1 (each component is self-contained), cost scales linearly. If &sigma; = log(n) (cross-file dependencies grow with app size), cost scales superlinearly. At 200 components, the gap is 10-20x. At 1,000, it's unbounded.

| Language / Framework | &sigma; | Class | Cost at 200 components |
|---------------------|---------|-------|------------------------|
| **Naze** | 1 | **&Lambda;-Linear** | 55K-110K tokens |
| **Svelte** | ~1.5 | **&Lambda;-Linear** (nearly) | 150K-280K tokens |
| **React + Tailwind + TS** | log(n) | **&Lambda;-LogLinear** | 700K-1.8M tokens |
| **Angular + TS** | ~n^0.3 | **&Lambda;-Quadratic** | 2M-8M tokens |

See [docs/TOKEN_EFFICIENCY.md](docs/TOKEN_EFFICIENCY.md) for the full framework, formula, and multi-language comparison.

## Design Principles

### AI-native, human-readable

The language is designed as a compilation target for AI, but `.naze` files read like a document describing what the UI should look like. A non-developer can open the source and understand the structure. AI-generated code is inspectable, not a black box.

### Kilobytes, not megabytes

The entire WASM runtime is 395KB. A typical app's render tree is hundreds of bytes. No `node_modules`, no megabytes of JavaScript, no CSS frameworks. The compiler is a single Rust binary. The attack surface is tiny.

### No middle layers

No bundler, no transpiler, no CSS preprocessor, no virtual DOM. Intent goes to pixels through the shortest path: parse, typecheck, serialize, deserialize, layout, render. `.naze` source compiles to four files: an HTML shell, a JS loader, a WASM runtime, and a binary data blob.

### Compile-time over runtime

Components are inlined, types are checked, imports are resolved, and dead code is eliminated at build time. The runtime is a thin interpreter that deserializes and renders -- no JIT, no eval, no parsing at runtime. No implicit framework behavior to reason about.

### One source, every platform

The same `.naze` file targets web (WASM + Canvas), desktop (native window via tiny-skia), and mobile -- without platform-specific code. Platform differences are handled by the renderer, not the language.

### Token-efficient by construction

Every language feature preserves &sigma; = 1. Structure, styling, state, events, data fetching -- all declared inline in one `.naze` file. One canonical form per concept: one way to express state, one way to bind events, one way to do conditional rendering. The AI can't pick the "wrong" pattern because there's only one pattern.

## What This Enables

### Local AI models that outperform cloud models

Today's coding LLMs are 70B-400B+ parameters because they cover 50+ languages and hundreds of frameworks. Naze has a constrained grammar (~157 PEG rules, LL(1)-compatible) -- small enough for grammar-constrained decoding (GBNF/CFG). A fine-tuned 3-7B model on `.naze` can match or outperform a general-purpose 70B model at Naze generation. That model runs locally on a laptop, offline, at zero cost. The language exports its own grammar for this purpose: `nazec grammar --format gbnf`.

### Instant app generation

The loop from "describe what you want" to "see it running" collapses. No `npm install`, no build config, no dependency resolution. `nazec build` produces a working app in milliseconds. The sub-second rebuild cycle makes voice-driven development practical: speak a change, a local LLM edits the `.naze` source, `nazec run` hot-reloads, and you see the result before you finish your next sentence.

### An agent-native application format

Naze compiles to `app_data.bin` -- a binary containing the complete application semantics: state schema, UI tree, actions, computed values, data bindings, conditions. An AI agent can deserialize this binary and understand the entire app without rendering it. No browser, no DOM, no JavaScript execution -- just structured data. Interaction becomes semantic (`execute action "append" on "tasks"`) instead of fragile (`click selector ".todo-form .submit-btn"`). See [docs/AGENT_RUNTIME.md](docs/AGENT_RUNTIME.md) for the full vision.

### Live app factory

The architecture enables a generative flywheel: describe an app in natural language, the agent queries a discovery registry for reusable packages, generates `.naze` source that imports them, the in-browser compiler produces a working app in seconds -- no server round-trip. Generated apps can be saved, forked, or published back to the registry, making the next app easier to build. Each published app enriches the registry; each richer registry improves generation quality. See [docs/ROADMAP.md](docs/ROADMAP.md) for the dedicated browser vision.

### Testing built in

Tests are `.test.naze` files in the same language as the app -- not Jest, not Playwright. Component tests render with props and assert output; flow tests simulate multi-page user journeys. `nazec test` runs everything. Because the language is declarative and the output is deterministic, tests are predictable and reproducible. AI generates tests alongside app code.

## Status

**Phases 1-5 complete (41 milestones). Phase 6 (Developer Experience & Adoption) in progress.**

11 crates, 400+ tests, ~157 grammar rules, 395KB WASM runtime, 109 examples.

- **Phase 1:** End-to-end pipeline -- `.naze` source to WASM + Canvas rendering
- **Phase 2:** State, events, routing, forms, animation, accessibility, dev server, native desktop builds
- **Phase 3:** Pipelines, pattern matching, responsive layout, testing framework, component events
- **Phase 4:** Server functions, SSR/SSG, package registry, AI grammar export, fine-tuning pipeline, prompt runtime
- **Phase 5:** Environment config, dynamic routing, error boundaries, auth, database integration, declarative queries
- **Phase 6 (current):** CI/CD (done), playground (done), docs site, VS Code extension, binary distribution

See [docs/ROADMAP.md](docs/ROADMAP.md) for the full roadmap and [docs/HISTORY.md](docs/HISTORY.md) for the consolidated Phase 1-5 record.

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
  done: runtime 395KB + app data 567B
```

This produces four files in `dist/`:

```
dist/
  index.html              HTML shell with canvas and bootstrap script
  naze_runtime.js         WASM loader (generated by wasm-pack)
  naze_runtime_bg.wasm    Runtime binary (395KB) — layout + Canvas2D renderer
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

If there are errors, you'll see diagnostics with error codes and fix suggestions:

```
error[E002]: type mismatch for prop 'width' on 'rect': expected number, got text
  --> app.naze:5:5
    |
  5 |     rect width: "oops", color: #ff0000
    |     ^
  = fix: Check the expected type and use the correct literal form
```

Use `--format json` for machine-readable output (includes `error_code` and `suggested_fix` fields):

```bash
nazec check --format json
# {"message":"type mismatch...","file":"app.naze","line":5,"column":5,
#  "severity":"Error","error_code":"E002","suggested_fix":"Check the expected type..."}
```

### 7. Export project context for AI agents

```bash
nazec context
```

This outputs a JSON summary of the project's structure — components, server functions, state variables, data sources, routes, guards, models, and prompts — without requiring AI agents to parse `.naze` source files.

```json
{
  "name": "my-app",
  "version": "0.1.0",
  "entry": "app.naze",
  "components": [{ "name": "card", "import_path": "components/card", "params": [...] }],
  "server_functions": [{ "name": "get_users", "params": [...] }],
  "state": [{ "name": "count", "shared": false }],
  "pages": [{ "path": "/dashboard", "params": [], "guard": "auth" }],
  "models": [{ "name": "User", "fields": [...] }],
  "prompts": [{ "name": "summarize", "provider": "openai" }]
}
```

AI tools and IDE extensions can use this to understand a project's API surface without reading individual files.

### 8. Add a reusable component

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

### 9. Browse all examples

The repository includes 109 example `.naze` files demonstrating various features. You can browse them interactively with the gallery command:

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

## Real-World Example Apps

Five apps in `examples/apps/` that exercise every major language feature:

| App | Features Tested |
|-----|-----------------|
| **todo** | append/remove, each, computed, pipelines, storage, validation |
| **dashboard** | routing (3 pages), themes, grid, select, pipelines, shared state, link/navigate |
| **tic-tac-toe** | complex state, match/wildcard, grid, conditional rendering |
| **chat** | append, each, textarea, shared state, scroll, session storage |
| **form-wizard** | input types, textarea, select, radio, checkbox, validation, match, storage |

### Run an app

```bash
# From the repo root — pick any app:
cd examples/apps/todo

# Dev server with hot reload (opens http://localhost:3000)
nazec dev

# Or build and serve manually
nazec build
python3 -m http.server -d dist 8080
```

Replace `todo` with `dashboard`, `tic-tac-toe`, `chat`, or `form-wizard`.

Each app directory has a `naze.toml` and `app.naze`. Edit `app.naze` and the dev server will hot-reload automatically.

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
    naze-lsp/         Language Server Protocol implementation
    naze-registry/    Package registry server
    naze-playground/  Compiler-as-WASM for browser playground
  examples/           109 example .naze files
  docs/
    ROADMAP.md        Long-term vision (Phases 1-6)
    HISTORY.md        Consolidated Phase 1-5 record
    PHASE6.md         Phase 6 milestone tracker
    LANGUAGE.md       Language reference
    TOKEN_EFFICIENCY.md  Token Complexity framework
    PROTOTYPE.md      Architecture spec
```

## CLI

```
nazec new <name>           Create a new project
nazec build [--target]     Compile to dist/ (web, native, or android)
nazec run                  Preview in a native desktop window (hot reload)
nazec dev [--port]         Dev server with browser hot reload
nazec serve [--port]       Production SSR server
nazec check                Type-check without building
nazec test [--format]      Run .test.naze test suites
nazec parse <file>         Dump AST as JSON
nazec context              Export project context as JSON for AI agents
nazec grammar [--format]   Export grammar for LLM constrained decoding
nazec gallery              Build and serve interactive example gallery
nazec ai generate          AI code generation from natural language
nazec playground           Start hosted playground server
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

## Playground

Try Naze in the browser without installing anything. The playground runs the full compiler and runtime as WebAssembly — your code compiles and renders live on a Canvas2D preview as you type.

Features: CodeMirror editor with Naze syntax highlighting, 500ms debounced compilation, example selector, shareable URLs, and mobile-responsive layout.

**Online:** Once deployed, available at your GitHub Pages URL.

**Local:**

```bash
make playground
cd playground && python3 -m http.server 4000
# open http://localhost:4000
```

`make playground` builds the compiler to WASM (`crates/naze-playground/`), copies it alongside the pre-built runtime WASM, and produces a self-contained `playground/` directory you can serve from any static file server.

## AI Model Training

Naze includes a pipeline for fine-tuning a local LLM to generate `.naze` code. The trained model runs via Ollama and integrates directly with the `nazec ai generate` command.

```bash
# Prerequisites: Python 3.10+, NVIDIA GPU (16GB+ VRAM), Ollama
python3 -m venv ai/.venv && source ai/.venv/bin/activate
pip install -r ai/requirements.txt

# Run the full pipeline (export data → prepare → train → register with Ollama)
bash ai/run_pipeline.sh

# Use the trained model
nazec ai generate --provider ollama --model naze-coder "Create a todo app"
```

See [ai/README.md](ai/README.md) for full setup instructions, VRAM requirements, and troubleshooting.

## Try the Toolkit

Test the end-user experience: package the compiler with docs and examples, then use it like a real user would.

```bash
make try
```

This builds the release binary, packages it with reference docs and examples, and extracts to `/tmp/naze-toolkit/`. From there:

```bash
cd /tmp/naze-toolkit/starter
../bin/nazec build
../bin/nazec dev
```

Or point an AI agent at `/tmp/naze-toolkit/README.md` and let it build apps.

Re-run `make try` after compiler changes — it rebuilds and re-extracts automatically.

## Language Features

Layout (`row`, `column`, `stack`, `grid`, `container`, `spacer`), elements (`rect`, `text`, `heading`, `image`, `input`, `select`, `textarea`), components with typed parameters, `use` imports, `state` and `computed`, event handlers, conditionals (`if`/`else`, `match`), iteration (`each`), routing (`page`), theming, animation with easing, drag & drop, scroll containers, data fetching (REST, WebSocket, SSE), server functions with SQL, storage (local/session), timers, pipeline operators, pattern matching, responsive layout, overlay system, form validation, accessibility (ARIA, focus management), declarative database queries (`model`/`find`/`insert`/`update`/`delete`), AI prompt blocks, JS interop, error boundaries, page guards, environment variables.

See [docs/LANGUAGE.md](docs/LANGUAGE.md) for the full language reference.

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

[MPL 2.0](LICENSE) (Mozilla Public License 2.0)

The Naze compiler, runtime, and tooling are open source and must remain so -- any modifications to the framework's source files must be shared under the same license. Applications you build with Naze (your `.naze` source files, compiled output, and everything in your `dist/` directory) are entirely yours and can be proprietary, commercial, or licensed however you choose.
