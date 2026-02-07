# Naze Roadmap

The long-term vision for Naze in five phases, from proof of concept to a full platform.

---

## Phase 1: Proof of Life — Complete

Establish the end-to-end pipeline: `.naze` source compiles to WASM + HTML, renders colored rectangles and text in the browser via Canvas2D.

**Delivered:** 7 Rust crates, 84 tests, 69KB WASM runtime, 16 example files, custom layout engine (~200 LOC), custom binary serialization format. Components with typed parameters and defaults. `use` imports. `nazec new`, `build`, `check`, `parse` CLI commands.

See [MVP.md](MVP.md) for the full summary.

---

## Phase 2: Real Apps + Developer Experience — Mostly Complete

Make apps dynamic and interactive. Add developer tooling. Prototype cross-platform rendering.

See [PHASE2.md](PHASE2.md) for the detailed milestone tracker.

**Phase 2a (M1-M14): Complete** — state, events, conditionals, slots, images, theming, layout (flex-grow/shrink, percentages, align/justify), routing, form inputs with validation, drag & drop, accessibility (ARIA, screen reader DOM, focus management), scroll containers, data fetching, animation with easing.

**Phase 2b (M9-M12): Substantially Complete** — VS Code extension with LSP, TextMate grammar, and visual editor. Dev server with WebSocket hot reload. Native desktop builds via `nazec run`. Android build prototype via `nazec build --target android`. Remaining polish items (incremental compilation, Marketplace publish, standalone native binary) deferred to Phase 3.

### Core Language (M1-M8)

- **State & reactivity** -- `let` bindings, `state` keyword, re-render on state change
- **Events** -- `on click`, `on hover`, `on keypress`, hit testing, focus management
- **Conditional rendering** -- `if`/`else`, `each` iteration
- **Content slots** -- component composition with named slots
- **Images & richer rendering** -- `image` element, borders, opacity, viewport resize
- **Theming** -- `theme.naze` with named design tokens, compile-time resolution
- **Improved layout** -- flex-grow/shrink, min/max constraints, align/justify, scroll, wrap
- **Navigation & routing** -- multi-page apps with `page` blocks, History API

### Interactive Features (M8b-M8e, M13-M14)

- **Form inputs** -- `input`, `checkbox`, `radio`, `select` with two-way binding and validation
- **Drag & drop** -- `draggable`, `drop-target`, drag events with visual feedback
- **Accessibility** -- `role`, `label`, `tab-index`, screen reader DOM overlay, compiler warnings
- **Scroll containers** -- mouse wheel scrolling, scrollbars, `on scroll`, `scroll-to`
- **Data fetching** -- `data` keyword, async fetch, loading/error states
- **Animation** -- `transition` props, easing functions, requestAnimationFrame scheduler

### Tooling & Platforms (M9-M12)

- **VS Code extension** -- syntax highlighting, LSP (diagnostics, autocomplete, hover, go-to-definition), visual editor
- **Dev server** -- `nazec dev` with file watching, auto-rebuild, browser hot reload via WebSocket
- **Desktop** -- same `.naze` source renders in a native window via `winit` + tiny-skia
- **Android** -- `nazec build --target android` generates WebView-based Android project

---

## Phase 3: Language Completion & Developer Experience — Next

Complete the language with advanced computation features, finalize tooling, add testing framework.

See [PHASE3.md](PHASE3.md) for the detailed milestone tracker (M15-M22).

### Language Completion (M15-M19e)

- **Pipeline operators** -- `|` chains for `filter`, `map`, `sort-by`, `take`, `sum`, `reduce`, etc.
- **Pure functions** -- expression-body functions with no side effects, inlining and constant folding
- **Pattern matching** -- `match` with exhaustive checking, destructuring, wildcard `_`
- **List comprehensions** -- `[expr for item in list if condition]`
- **Layout templates** -- `template` keyword, built-in template library (app-shell, dashboard, sidebar-layout)
- **Responsive breakpoints** -- `responsive: stack below 768px`, `collapsible`
- **Advanced animation** -- spring physics, keyframes, GPU fast path, custom `cubic-bezier()`
- **Component events** -- `emit` from child, `on event` in parent
- **Theme inheritance** -- `extends`, runtime switching
- **Overlay system** -- `overlay` element, focus trapping, outside-click, scroll-lock, anchor positioning
- **Visual properties** -- shadows, text-align, text-overflow, gradients, transforms, cursor styles
- **Application logic primitives** -- `computed` state, `shared state`, `storage` (localStorage), enhanced `data` (full HTTP), `data: stream` (WebSocket/SSE), `param` (URL query params), `timer`, `debounce`/`throttle`, `copy` (clipboard), file input
- **Remaining gap closures** -- `textarea` element, `js` interop (third-party JS SDK calls), browser device APIs (`device geolocation`, `device camera`, `notify`) (see [PARITY.md](PARITY.md))

### Testing & Tooling (M20-M22)

- **Testing framework** -- `.test.naze` files, `test`/`flow` blocks, headless renderer, `nazec test`, CI JSON output
- **VS Code extension polish** -- full type-checking LSP, cross-file go-to-def, Marketplace publish
- **Build pipeline polish** -- incremental compilation, standalone native binary, Android APK end-to-end

**Parity targets:**
- **Component UI parity:** M19b (overlay) + M19c (visual properties) → ~92% of shadcn/ui-equivalent components buildable.
- **Application logic parity:** M19d (app logic primitives) + M19e (JS interop, device APIs, textarea) → ~85%. Combined with M15 (pipelines) + M23 (WASM imports) + M24 (server functions), reaches **~99%**.
- See [PARITY.md](PARITY.md) for the full analysis and design rationale.

**Target:** Sub-second hot reload cycle. Testing in CI. AI generates correct Naze >80% for common patterns.

---

## Phase 4: Ecosystem & External Integration

Server rendering, package ecosystem, AI integration, SEO.

See [PHASE4.md](PHASE4.md) for the detailed milestone tracker (M23-M30).

### External Integration (M23-M25)

- **WASM module imports** -- `import` keyword, `wasm-merge`/`wasm-opt` pipeline, type-checked signatures
- **Server functions** -- `server` keyword, auto-generated RPC stubs, `server(edge)` for edge deployment
- **Server rendering** -- SSR/SSG modes, layout tree → HTML serialization, embedded HTTP server

### Ecosystem (M26-M27)

- **Package manager** -- `nazec add/publish`, `naze.lock`, semver resolution, registry
- **SEO / meta-index** -- Open Graph tags, JSON-LD, per-route HTML pages

### AI & Advanced Tools (M28-M30)

- **AI integration** -- grammar export (GBNF/CFG), validation feedback loop, fine-tuning dataset, prompt library
- **AI prompting runtime** -- `prompt` keyword, provider adapters (OpenAI, Anthropic, Ollama)
- **Advanced dev tools** -- inspector, debugger, playground (compiler-as-WASM), binary size analyzer

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

6. **Token-efficient by construction.** Naze targets Λ-Linear token complexity — AI cost scales linearly with application size because components are self-contained, styling is inline, and each concept has one canonical form. See [TOKEN_EFFICIENCY.md](TOKEN_EFFICIENCY.md) for the full framework.
