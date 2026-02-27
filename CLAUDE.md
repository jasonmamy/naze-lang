# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Naze is a declarative, AI-native UI language designed to replace HTML/CSS/JS. It compiles `.naze` source files into a custom binary format (`app_data.bin`), which a WASM runtime deserializes and renders via Canvas2D — bypassing the DOM entirely.

**Current status:** Phase 5 complete (M1-M41). Phase 6 (Developer Experience & Adoption) in progress — M42 (CI/CD) and M44 (Playground) complete. See `docs/ROADMAP.md` for the full roadmap, `docs/HISTORY.md` for Phases 1-5 consolidated record, `docs/PHASE6.md` for Phase 6, and `docs/PROTOTYPE.md` for the architecture spec.

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

Cargo workspace with 11 crates in `crates/`:

| Crate | Target | Purpose |
|-------|--------|---------|
| `nazec` | native | CLI binary (clap) — 18 commands (see CLI section below) |
| `naze-parser` | native | PEG parser (pest) — `.naze` → AST. Grammar in `src/naze.pest` |
| `naze-compiler` | native | Import resolution, type checking, codegen → `app_data.bin` |
| `naze-ir` | both | Shared IR types (`RenderTree`, `RenderNode`), custom binary serialization |
| `naze-runtime` | WASM | Deserializes app data, manages state/events, orchestrates layout + render |
| `naze-layout` | both | Custom layout engine (~200 LOC core) — row, column, stack, grid, flex |
| `naze-renderer` | WASM | Canvas2D renderer (web-sys) — draws to browser canvas |
| `naze-native` | native | Standalone desktop viewer — winit + tiny-skia software rasterizer |
| `naze-lsp` | native | Language Server Protocol implementation (tower-lsp) |
| `naze-registry` | native | Package registry server (Axum + SQLite + filesystem storage) |
| `naze-playground` | WASM | Compiler-as-WASM for browser-based playground |

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
- `RenderTree`: title, root, state, computed, data, storage, timers, params, pages, themes, imports, server_functions, server_calls, guards, prompts
- `RenderNode`: kind (string), props (HashMap), children, event handlers, conditions, each-bindings
- `RenderValue`: Str, Num (with optional unit), Color (u32), Bool, InterpolatedStr, List, Object, Bind

All serialization is in `crates/naze-ir/src/lib.rs`. The `serde` feature is only enabled for native crates (compiler), not WASM.

## CLI (`nazec`)

```
nazec new <name>              # Scaffold project with naze.toml + app.naze
nazec build [--target web|native|android] [--static]  # Compile .naze → dist/
nazec run                     # Preview in native desktop window (hot reload)
nazec dev [--port 3000]       # Dev server with browser hot reload
nazec serve [--port 8080]     # Production SSR server
nazec check                   # Type-check without building
nazec test [--format json]    # Run .test.naze test suites
nazec parse <file>            # Dump AST as JSON
nazec grammar [--format gbnf|ebnf]  # Export grammar for LLM constrained decoding
nazec gallery [--build]       # Build interactive example gallery
nazec analyze                 # WASM binary size analyzer
nazec add <package>           # Add dependency to naze.toml
nazec remove <package>        # Remove dependency
nazec update                  # Update dependencies
nazec publish                 # Publish package to registry
nazec search <query>          # Search package registry
nazec playground [--port]     # Start hosted playground server
nazec ai <generate|fix|dataset>  # AI code generation and training tools
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
- **μ (model cost):** Does this feature significantly increase grammar complexity? The grammar is currently ~157 rules (grown from ~56 in Phase 1) and remains LL(1)-compatible. New rules are acceptable when they follow existing patterns (e.g., `storage` mirrors `state`), but novel syntax forms should be justified.

**Concrete examples:**
- `shared state` — must be designed so the AI needs only the current file (σ = 1), not a separate state store
- `js` interop — must be a type-checked boundary call, not something requiring understanding of external JS files
- Pipeline operators `|` — acceptable: adds grammar rules but maintains σ = 1 (single-file expressions)

See `docs/TOKEN_EFFICIENCY.md` for the full framework, formula, and multi-language comparison. See `docs/PARITY.md` for feature-level competitive analysis.

## Grammar Tiers

The PEG grammar is partitioned into tiers for future modular GBNF export and targeted LLM training. When adding new grammar rules, assign them to the appropriate tier. Lower tiers must never depend on higher tier syntax.

| Tier | Name | Scope | Example rules |
|------|------|-------|---------------|
| 0 | Core UI | Layout, elements, state, events, conditionals, themes, components | `app_block`, `element`, `state_stmt`, `if_stmt`, `each_stmt` |
| 1 | Data | Fetch, streams, server functions, storage, timers | `data_stmt`, `server_function_def`, `storage_stmt` |
| 2 | Database | Models, declarative queries | `model_def`, `server_find_expr`, `server_insert_expr` |
| 3 | AI | Prompt blocks, provider config | `prompt_stmt`, `prompt_block` |
| 4 | Systems | (future) Concurrency, file IO, networking | — |

Tier annotations are marked in `naze.pest` as `// [Tier N: Name]` comments. A Tier 0 GBNF can train a 3B model for UI-only generation; Tier 0-2 trains a 7B fullstack model.

## Key Files

- **PEG Grammar**: `crates/naze-parser/src/naze.pest` (~157 rules)
- **AST Types**: `crates/naze-parser/src/ast.rs`
- **Type Checker**: `crates/naze-compiler/src/typecheck.rs` (largest file, ~37KB)
- **Code Generator**: `crates/naze-compiler/src/codegen.rs` (~52KB)
- **IR + Serialization**: `crates/naze-ir/src/lib.rs`
- **WASM Runtime**: `crates/naze-runtime/src/lib.rs` (~5000+ lines)
- **Build Pipeline**: `crates/nazec/src/build.rs` (WASM embedding + dist/ generation)
- **Server Functions**: `crates/nazec/src/server_fns.rs` (shared JSON↔RenderValue, server fn evaluation)
- **SSR Server**: `crates/nazec/src/serve.rs` (production Axum server)
- **HTML Renderer**: `crates/nazec/src/html_renderer.rs` (SSG output)
- **AI Tools**: `crates/nazec/src/ai.rs` (generate, fix, dataset commands)
- **Examples**: `examples/` directory (109 `.naze` files demonstrating features)

## Risk Areas

The highest-concern risks are: accessibility adequacy (high impact, high likelihood), language design not working for AI (fatal impact, medium likelihood), and lack of adoption (high impact, medium likelihood). See `docs/PROTOTYPE.md` risk register for details.
