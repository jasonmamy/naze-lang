# Naze Roadmap

The long-term vision for Naze in five phases, from proof of concept to a full platform.

---

## Phase 1: Proof of Life -- Complete

Establish the end-to-end pipeline: `.naze` source compiles to WASM + HTML, renders colored rectangles and text in the browser via Canvas2D.

**Delivered:** 7 Rust crates, 84 tests, 69KB WASM runtime, 16 example files, custom layout engine (~200 LOC), custom binary serialization format. Components with typed parameters and defaults. `use` imports. `nazec new`, `build`, `check`, `parse` CLI commands.

See [MVP.md](MVP.md) for the full summary.

---

## Phase 2: Real Apps + Developer Experience -- In Progress

Make apps dynamic and interactive. Add developer tooling. Prototype cross-platform rendering.

See [PHASE2.md](PHASE2.md) for the detailed milestone tracker.

### Core Language (M1-M8)

- **State & reactivity** -- `let` bindings, `state` keyword, re-render on state change
- **Events** -- `on click`, `on hover`, `on keypress`, hit testing, focus management
- **Conditional rendering** -- `if`/`else`, `each` iteration
- **Content slots** -- component composition with named slots
- **Images & richer rendering** -- `image` element, borders, opacity, viewport resize
- **Theming** -- `theme.naze` with named design tokens, compile-time resolution
- **Improved layout** -- flex-grow/shrink, min/max constraints, align/justify, scroll, wrap
- **Navigation & routing** -- multi-page apps with `page` blocks, History API

### Tooling (M9-M10)

- **VS Code extension** -- syntax highlighting, LSP (diagnostics, autocomplete, hover, go-to-definition)
- **Dev server** -- `nazec dev` with file watching, auto-rebuild, browser hot reload

### Cross-Platform Prototypes (M11-M12)

- **Desktop** -- same `.naze` source renders in a native window via `winit` + software/GPU renderer
- **Android** -- same `.naze` source renders on Android (embedded WASM or native renderer)

### Late Features (M13-M14)

- **Data fetching** -- `data` keyword, async fetch, loading/error states
- **Animation** -- `transition` props, easing functions, requestAnimationFrame scheduler

---

## Phase 3: Developer Experience

Make it pleasant to build Naze apps. Fast iteration, AI integration, rich tooling.

- **AI integration layer** -- intent-to-Naze pipeline, validation feedback loop, fine-tuning dataset, constrained decoding
- **Inspector** -- visual element inspector, component tree, property panel (like browser DevTools)
- **Debugger** -- WASM source-mapped debugging, state inspection, event logging
- **Hot reload** -- file watcher, incremental recompile, state-preserving hot-swap
- **Playground** -- browser-based editor with live preview, shareable URLs, example gallery (compiler itself compiled to WASM)
- **Binary size analyzer** -- WASM section treemap, before/after comparison

**Target:** AI generates correct Naze on first attempt >80% of the time for common UI patterns. Sub-second hot reload cycle.

---

## Phase 4: Ecosystem & Adoption

Package registry, community, production deployments, SEO.

- **Package manager** -- `nazec add`, `nazec publish`, `naze.lock`, semver resolution, git-based and registry-based dependencies
- **HTML meta-index** -- SEO-friendly HTML shell generation (Open Graph, JSON-LD, structured metadata) from `.naze` source
- **Server rendering** -- SSG and SSR modes, server functions (`server` keyword), edge deployment
- **Testing framework** -- `.test.naze` files, component tests, flow tests, headless renderer, `nazec test`
- **Documentation** -- language reference, tutorials, migration guides, AI prompt cookbook

**Target:** Naze apps discoverable by search engines. Community contributing shared components. At least 3 non-trivial public Naze apps.

---

## Phase 5: Dedicated Browser

Optional lightweight runtime that runs Naze natively without browser overhead. Only justified once there's significant Naze content to consume.

- Standalone WASM runtime (no browser engine)
- Native window with GPU rendering (no Canvas indirection)
- URL bar, navigation, tabs, bookmarks
- HTML fallback (embedded lightweight webview for legacy content)
- Content-type detection: route `application/naze` to native pipeline, HTML to webview

This is the "new browser" -- but it starts as an optimization for existing Naze apps, not as a prerequisite. Everything in Phases 1-4 runs in standard browsers today.

---

## Design Principles

These hold across all phases:

1. **AI-native, human-readable.** The language is designed as a compilation target for AI, but `.naze` files should be readable by anyone. No framework magic, no hidden state, no implicit behavior.

2. **Kilobytes, not megabytes.** The runtime stays small. Phase 1 delivered 69KB. Phase 2 targets <150KB. Every dependency is justified by the bytes it costs.

3. **One source, every platform.** The same `.naze` file should render identically on web, desktop, and mobile. Platform differences are handled by the renderer, not the language.

4. **Compile-time over runtime.** Components are inlined, theme tokens are resolved, types are checked, and dead code is eliminated at compile time. The runtime does as little work as possible.

5. **No middle layers.** No bundler, no transpiler, no CSS preprocessor, no virtual DOM. Intent goes to pixels through the shortest path: parse, typecheck, serialize, deserialize, layout, render.
