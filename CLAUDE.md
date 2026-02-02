# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Naze is a new declarative, AI-native UI language designed to replace the HTML/CSS/JS paradigm. It compiles `.naze` source files to WebAssembly and renders via Canvas/WebGL, bypassing the DOM entirely. The project is under the Illuminaze umbrella.

**Current status:** Phase 1 complete (69KB WASM, 84 tests, 16 examples). See `docs/MVP.md` for summary, `docs/PHASE2.md` for next phase.

## Architecture

The system is composed of 14 decoupled components with explicit interface contracts (detailed in `docs/PROTOTYPE.md`):

- **C1: Language Spec** — Declarative syntax with brace-delimited blocks, `--` comments, pipeline operators (`|`), pattern matching, typed props. Three computation tiers: built-in declarative logic, WASM library imports, and server functions.
- **C2: Compiler** (Rust) — Parses `.naze` → AST → type-check → optimize → emit WASM/native.
- **C3: Runtime** (Rust→WASM) — App lifecycle, reactive state, event dispatch, routing, data fetching.
- **C4: Layout Engine** (Rust, builds on Taffy) — Named slot templates + spatial primitives (grid, row, column, stack).
- **C4a: Text Engine** (Rust + C FFI) — HarfBuzz, FreeType, ICU for text shaping/rendering.
- **C5: Renderer** (Rust, wgpu) — WebGL/WebGPU/Metal/Vulkan/DX12 backends.
- **C6: Accessibility Bridge** — Side DOM for screen readers, IME support.
- **C7: AI Integration** (Python + Rust) — Grammar-constrained LLM generation, validation loops.
- **C8: Meta-Index Generator** — HTML shell generation for SEO/crawlers.
- **C9: Dev Tooling** (Rust + TypeScript) — LSP, debugger, inspector, hot reload, playground.
- **C10: Package Manager** (Rust) — `nazec` CLI for project scaffolding, builds, dependency resolution.
- **C11: Testing Framework** — `.test.naze` files using same language.
- **C12: AI Prompting Runtime** — Provider abstraction for AI interactions within apps.
- **C14: Server Renderer & Runtime** — SSR/SSG/SPA rendering modes, server functions.

Data flows top-down: User/AI intent → C7 → C1 source → C2 compile → C3 runtime → C4/C4a/C5 render.

## Build Commands

```bash
# First-time setup (installs Rust, wasm-pack, WASM target)
bash setup.sh

# Build the nazec CLI
cargo build -p nazec

# Run all workspace tests
cargo test --workspace

# Check all crates compile
cargo check --workspace

# Format / lint
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

## Workspace Structure

Cargo workspace with 7 crates in `crates/`:

| Crate | Target | Purpose |
|-------|--------|---------|
| `nazec` | native | CLI binary (clap) — `new`, `build`, `check` commands |
| `naze-parser` | native | PEG parser (pest) — `.naze` → AST |
| `naze-compiler` | native | Type checker + binary serializer → `app_data.bin` |
| `naze-ir` | both | Shared IR types (RenderTree, custom binary serialization) |
| `naze-runtime` | WASM | Entry point — deserializes app data, orchestrates layout + render |
| `naze-layout` | both | Custom layout engine (~200 LOC) — computes positioned rectangles |
| `naze-renderer` | WASM | Canvas2D renderer (web-sys) — draws to browser canvas |

Native crates build with `cargo build`. WASM crates build with `wasm-pack build --target web`.
The runtime WASM is embedded in the `nazec` binary via `include_bytes!`.

## CLI (`nazec`)

```
nazec new <name>    # Scaffold project
nazec build         # Compile .naze → .wasm + index.html (output to dist/)
nazec dev           # Dev server with hot reload
nazec check         # Type-check without building
nazec test          # Run .test.naze files
nazec publish       # Publish to registry
nazec add <pkg>     # Add dependency
```

Project config will use `naze.toml` (TOML format).

## Development Phases

1. **Phase 1 — Proof of Life:** Minimal language spec, WASM compiler, basic runtime/layout/renderer, CLI scaffolding. Goal: colored rectangles + text in browser, <100KB binary.
2. **Phase 2 — Real Apps:** Full language (slots, events, data binding, responsive, animations, a11y), text engine, testing, SSR, server functions.
3. **Phase 3 — Developer Experience:** LSP, debugger, inspector, playground, AI integration layer.
4. **Phase 4 — Ecosystem:** Package registry, community, documentation.
5. **Phase 5 — Dedicated Browser:** Optional lightweight WASM runtime as standalone browser.

## Key Design Decisions

- **Rust** is the primary implementation language for all core components.
- **No DOM** — rendering goes directly to Canvas/WebGL via WASM.
- **AI-first** — the language is designed as a compilation target for LLMs, with grammar constraints enabling guaranteed syntactic validity.
- **Three computation tiers:** declarative built-ins (pure expressions, no statements), WASM imports for heavy logic, server functions for backend calls.
- **Accessibility is not optional** — a11y metadata is part of the component definition and compiled into the binary.
- Layout uses **named slot templates** (high-level) and **spatial primitives** (low-level), not CSS.

## Risk Areas (from docs/PROTOTYPE.md)

The highest-concern risks are: accessibility adequacy (high impact, high likelihood), language design not working for AI (fatal impact, medium likelihood), and lack of adoption (high impact, medium likelihood). Keep these in mind when making design decisions.
