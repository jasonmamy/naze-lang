# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Naze is a declarative, AI-native UI language designed to replace HTML/CSS/JS. It compiles `.naze` source files into a custom binary format (`app_data.bin`), which a WASM runtime deserializes and renders via Canvas2D — bypassing the DOM entirely.

**Current status:** Phase 2 substantially complete. See `docs/MVP.md` for Phase 1 summary, `docs/PHASE2.md` for Phase 2 progress, `docs/PROTOTYPE.md` for full architecture spec.

## Build Commands

```bash
# First-time setup (installs Rust, wasm-pack, WASM target)
bash setup.sh

# Build the nazec CLI
cargo build -p nazec

# Run all workspace tests
cargo test --workspace

# Run tests for a single crate
cargo test -p naze-parser
cargo test -p naze-compiler

# Run a single test by name
cargo test -p naze-parser -- test_name

# Check all crates compile
cargo check --workspace

# Format / lint
cargo fmt --all
cargo clippy --workspace -- -D warnings

# Rebuild WASM runtime (after changes to naze-runtime, naze-layout, or naze-renderer)
cd crates/naze-runtime && wasm-pack build --target web --release && cd ../..
```

There is also a `Makefile` with shortcuts: `make build`, `make test`, `make check`, `make fmt`, `make lint`, `make release`, `make clean`, `make setup`.

## Workspace Structure

Cargo workspace with 9 crates in `crates/`:

| Crate | Target | Purpose |
|-------|--------|---------|
| `nazec` | native | CLI binary (clap) — `new`, `build`, `run`, `check`, `dev`, `parse`, `gallery` |
| `naze-parser` | native | PEG parser (pest) — `.naze` → AST. Grammar in `src/naze.pest` |
| `naze-compiler` | native | Import resolution, type checking, codegen → `app_data.bin` |
| `naze-ir` | both | Shared IR types (`RenderTree`, `RenderNode`), custom binary serialization |
| `naze-runtime` | WASM | Deserializes app data, manages state/events, orchestrates layout + render |
| `naze-layout` | both | Custom layout engine (~200 LOC core) — row, column, stack, grid, flex |
| `naze-renderer` | WASM | Canvas2D renderer (web-sys) — draws to browser canvas |
| `naze-native` | native | Standalone desktop viewer — winit + tiny-skia software rasterizer |
| `naze-lsp` | native | Language Server Protocol implementation (tower-lsp) |

**Important:** `naze-runtime` is WASM-only and excluded from `default-members`. It must be built separately with `wasm-pack`. The pre-built WASM + JS wrapper live in `crates/naze-runtime/pkg/` and are embedded into the `nazec` binary via `include_bytes!()` (see `crates/nazec/src/build.rs`).

## Compile Pipeline

```
.naze source
  → naze-parser (pest PEG grammar → AST)
  → naze-compiler/resolve.rs (import resolution, component inlining)
  → naze-compiler/typecheck.rs (type checking)
  → naze-compiler/codegen.rs (AST → RenderTree)
  → naze-ir serialization (RenderTree → app_data.bin, custom binary format)
  → nazec/build.rs writes dist/
       ├── index.html (generated shell)
       ├── naze_runtime.js (embedded from pkg/)
       ├── naze_runtime_bg.wasm (embedded from pkg/)
       └── app_data.bin
```

## Custom Binary Serialization (naze-ir)

The IR uses a custom binary format instead of serde to keep WASM size small (~40KB savings). Key types:
- `RenderTree`: title, state declarations, data declarations, root nodes, page definitions
- `RenderNode`: kind (string), props (HashMap), children, event handlers, conditions, each-bindings
- `RenderValue`: Str, Num (with optional unit), Color (u32), Bool, InterpolatedStr, List, Object, Bind

All serialization is in `crates/naze-ir/src/lib.rs`. The `serde` feature is only enabled for native crates (compiler), not WASM.

## CLI (`nazec`)

```
nazec new <name>              # Scaffold project with naze.toml + app.naze
nazec build [--target web|native|android]  # Compile .naze → dist/
nazec run                     # Preview in native desktop window (hot reload)
nazec dev [--port 3000]       # Dev server with browser hot reload
nazec check                   # Type-check without building
nazec parse <file>            # Dump AST as JSON
nazec gallery [--build]       # Build interactive example gallery
```

## Key Design Decisions

- **No DOM** — rendering goes directly to Canvas2D via WASM.
- **Custom layout engine** replaced Taffy — saved ~160KB of WASM binary size.
- **Custom binary serialization** replaced serde in WASM — saved ~40KB.
- **Render tree serialization** (not code generation) — the compiler emits a data blob, the runtime interprets it.
- **WASM embedded in CLI** via `include_bytes!()` — single binary distribution, no external runtime files.
- **Compile-time component inlining** — all import resolution and component expansion happens at build time; the runtime is a thin interpreter.
- **AI-first language design** — one canonical way to express each concept, grammar designed for constrained LLM decoding.

## AI Efficiency — The Ψ Test

Naze achieves an **AI Efficiency Index (AEI) of 1x** — the lowest cost per AI interaction of any evaluated language. This must be maintained as features are added. When implementing or designing a new language feature, evaluate it against the four parameters of the unified Cost Complexity equation **Ψ(L, n) = n × λ × σ × (1 + r) × μ**:

- **σ (scatter):** Does this feature require reading files beyond the current component? If understanding or generating code for this feature requires the AI to read another file (shared state stores, external config, type definition files), it pushes σ > 1 and breaks Λ-Linear scaling. Design for σ = 1: all information the AI needs should be in the current file.
- **λ (verbosity):** Does this feature add boilerplate or verbose syntax? Minimize tokens per unit of intent. Prefer concise, declarative forms over ceremony.
- **r (retry rate):** Does this feature introduce multiple valid forms for the same concept, implicit behavior, or context-dependent semantics? These increase the probability of incorrect AI generation. Maintain one canonical form per concept.
- **μ (model cost):** Does this feature significantly increase grammar rule count beyond ~56 rules? Larger grammars may require larger (more expensive) models. Keep the grammar LL(1)-compatible and small.

**Concrete examples:**
- `shared state` — must be designed so the AI needs only the current file (σ = 1), not a separate state store
- `js` interop — must be a type-checked boundary call, not something requiring understanding of external JS files
- Pipeline operators `|` — acceptable: adds grammar rules but maintains σ = 1 (single-file expressions)

See `docs/TOKEN_EFFICIENCY.md` for the full framework, formula, and multi-language comparison. See `docs/PARITY.md` for feature-level competitive analysis.

## Key Files

- **PEG Grammar**: `crates/naze-parser/src/naze.pest` (~100 rules)
- **AST Types**: `crates/naze-parser/src/ast.rs`
- **Type Checker**: `crates/naze-compiler/src/typecheck.rs` (largest file, ~37KB)
- **Code Generator**: `crates/naze-compiler/src/codegen.rs` (~52KB)
- **IR + Serialization**: `crates/naze-ir/src/lib.rs`
- **WASM Runtime**: `crates/naze-runtime/src/lib.rs` (~3000 lines)
- **Build Pipeline**: `crates/nazec/src/build.rs` (WASM embedding + dist/ generation)
- **Examples**: `examples/` directory (18+ `.naze` files demonstrating features)

## Risk Areas

The highest-concern risks are: accessibility adequacy (high impact, high likelihood), language design not working for AI (fatal impact, medium likelihood), and lack of adoption (high impact, medium likelihood). See `docs/PROTOTYPE.md` risk register for details.
