# Phase 1: Proof of Life — Complete

Phase 1 established the end-to-end pipeline: `.naze` source → compile → WASM + HTML → colored rectangles and text in the browser.

## What Was Built

**7 Rust crates** in a Cargo workspace:

| Crate | Role | Target |
|-------|------|--------|
| `nazec` | CLI binary (`new`, `build`, `check`, `parse`) | native |
| `naze-parser` | PEG parser (pest) — `.naze` source to AST | native |
| `naze-compiler` | Import resolution, type checker, codegen | native |
| `naze-ir` | Shared IR types (RenderTree, serialization) | native + WASM |
| `naze-layout` | Layout engine (row, column, stack, grid) | native + WASM |
| `naze-renderer` | Canvas2D renderer (web-sys) | WASM |
| `naze-runtime` | WASM entry point — deserialize, layout, render | WASM |

**Key architecture decision:** The compiler emits a serialized render tree (`app_data.bin`), not WASM instructions. A pre-built runtime WASM (embedded in the `nazec` binary via `include_bytes!`) deserializes this data, computes layout, and renders to Canvas2D. This is simpler than code generation and sufficient for Phase 1's purely declarative scope.

## Numbers

- **Runtime WASM:** 69KB after `wasm-opt -Oz` (budget was 100KB)
- **Test suite:** 84 tests across all crates
- **Examples:** 16 `.naze` files covering layouts, components, typography, grids
- **Language features:** 9 built-in elements, 4 types, components with typed props and defaults, `use` imports

## Key Technical Decisions

- **Custom layout engine** (~200 LOC) instead of taffy — saved ~160KB of WASM size
- **Custom binary format** instead of MessagePack — eliminated serde from WASM, saving ~40KB
- **`naze-ir` crate** — breaks the dependency chain so pest/parser code stays out of WASM
- **Canvas2D** instead of WebGL — dramatically simpler, sufficient for rectangles and text
- **Custom diagnostic printer** instead of miette — full control over error formatting, supports `--format json`

## Deferred to Phase 2

- Source map generation (binary offset → `.naze` source location)
- Events, state, interaction (purely static/declarative in Phase 1)
- Images, gradients, shadows
- Responsive breakpoints, scroll containers
- Content slots for components
