# Naze Development History

Consolidated record of Phases 1-5 (M1-M41). All milestones complete. For the active roadmap, see [ROADMAP.md](ROADMAP.md). For Phase 6 (in progress), see [PHASE6.md](PHASE6.md).

---

## Phase 1: Proof of Life

Established the end-to-end pipeline: `.naze` source compiles to WASM + HTML, renders colored rectangles and text in the browser via Canvas2D.

- 7 Rust crates in a Cargo workspace
- Custom layout engine (~200 LOC) instead of Taffy -- saved ~160KB WASM
- Custom binary serialization instead of serde -- saved ~40KB WASM
- Components with typed parameters and defaults, `use` imports
- 9 built-in elements, 4 types, 16 example files
- **Stats at completion:** 84 tests, 69KB WASM runtime

See [MVP.md](MVP.md) for the original design document.

---

## Phase 2: Real Apps + Developer Experience (M1-M14)

Made apps dynamic and interactive. Added developer tooling. Prototyped cross-platform rendering.

**Phase 2a -- Dynamic Apps (M1-M8e):**
- **M1** State & Reactivity -- `state`, `let`, render loop with change detection
- **M2** Event System -- `on click/hover/keypress`, hit testing, focus management, tab navigation
- **M3** Conditional Rendering -- `if`/`else`, `each` iteration
- **M4** Content Slots -- `slot`, named slots, `fill`, default slot content
- **M5** Images & Richer Rendering -- `image` element, `opacity`, `border`, viewport resize
- **M6** Theming -- `theme` blocks, design tokens, `theme.colors.primary` references
- **M7** Improved Layout -- `flex-grow/shrink`, percentages, `align`/`justify`, `wrap`
- **M8** Navigation & Routing -- `page` blocks, `link`, History API, `navigate` action
- **M8b** Form Inputs -- `input`, `checkbox`, `radio`, `select`, two-way `bind`, validation
- **M8c** Drag & Drop -- `draggable`, `drop-target`, `on drag-start/drag-over/drop`
- **M8d** Accessibility -- `role`, `label`, hidden DOM overlay for screen readers, ARIA inference
- **M8e** Scroll & Overflow -- `scroll` container, scrollbar rendering, `on scroll`, `scroll-to`

**Phase 2b -- Tooling & Platforms (M9-M14):**
- **M9** VS Code Extension -- TextMate grammar, LSP, autocomplete, hover, go-to-def, visual editor
- **M10** Dev Server & Hot Reload -- `nazec dev` with Axum + WebSocket hot reload
- **M11** Native Desktop -- `nazec run` with winit + tiny-skia software renderer
- **M12** Android Prototype -- `nazec build --target android` with WebView
- **M13** Data Fetching -- `data` keyword, `.loading`/`.error`/`.data` lifecycle
- **M14** Animation -- `transition` prop, easing functions, property interpolation

**Stats at completion:** 11 crates, ~200 tests, ~75KB WASM runtime

---

## Phase 3: Language Completion & Developer Experience (M15-M22)

Completed the language with computation features, finalized tooling, added testing framework.

- **M15** Pipeline Operators & Pure Functions -- `filter`, `map`, `sort-by`, `take`, `sum`, `count`, `reduce`, `group-by`, `flatten`, `distinct`; `function` definitions with compile-time inlining
- **M16** Pattern Matching -- `match` with string/number/bool/wildcard arms, compile-time desugaring
- **M17** Layout Templates & Responsive Design -- `template` definitions, `responsive` breakpoints, `collapsible` panels
- **M18** Advanced Animation -- spring physics, keyframe `animate` prop, custom cubic-bezier, layout-skip fast path
- **M19** Component Events & Theme Inheritance -- `emit` action, `extends` for themes, runtime theme switching
- **M19b** Overlay System -- `overlay` element, `focus-trap`, `scroll-lock`, `on click-outside`, `anchor` positioning
- **M19c** Visual Properties -- `shadow`, `text-align`, `text-overflow`, `gradient`, `transform`, `cursor`, `overflow`
- **M19d** Application Logic Primitives -- `computed`, `shared state`, `storage`, enhanced `data` (HTTP config, cache, retry, trigger), WebSocket/SSE streams, `param`, `timer`, debounce/throttle, `copy`, `send`, file uploads
- **M19e** Remaining Gap Closures -- `textarea` element, JS interop (`js` action + `data: js`), device APIs (geolocation, camera), browser notifications
- **M20** Testing Framework -- `.test.naze` files, `test`/`flow` blocks, `assert` statements, headless renderer
- **M22** Build Pipeline Polish -- incremental compilation, build timing, source maps, touch scroll, live regions

M21 (VS Code Extension Polish) deferred to Phase 6 as M45.

**Stats at completion:** ~300 tests, ~130 grammar rules, ~165KB WASM runtime

---

## Phase 4: Ecosystem & External Integration (M23-M30)

Enabled production deployment with server rendering, package ecosystem, SEO, and AI integration.

- **M23** WASM Module Imports -- `import` statement, qualified `module.func()` calls, JS bridge dispatch, `wasmparser` type checking
- **M24** Server Functions -- `server function` definitions, auto-generated RPC stubs, Axum `/api/{name}` endpoints
- **M25a** Static Site Generation -- `nazec build --static`, HTML renderer with CSS flexbox/grid mapping
- **M25b** Server-Side Rendering -- `nazec serve`, per-request HTML rendering, hydration, server function pre-evaluation
- **M26a** Local Package Dependencies -- `[dependencies]` in naze.toml, git-based deps, `nazec add/remove/update`, lockfile
- **M26b** Package Registry -- `naze-registry` crate (Axum + SQLite), `nazec publish/search`, semver resolution
- **M27** SEO / Meta-Index -- Open Graph, Twitter Cards, JSON-LD, `<noscript>` fallback, per-route HTML
- **M28a** AI Grammar Export -- `nazec grammar --format gbnf|ebnf`, structured error JSON
- **M28b** AI Validation & Fine-Tuning -- `nazec ai generate/fix/dataset`, validation feedback loop
- **M29** AI Prompting Runtime -- `prompt` keyword, OpenAI/Anthropic/Ollama/generic provider adapters
- **M30** Advanced Developer Tools -- inspector (element bounds, component tree, a11y viewer), event/network logs, playground, size analyzer

**Stats at completion:** ~350 tests, ~140 grammar rules, ~355KB WASM runtime

---

## Phase 5: Production-Ready Application Platform (M31-M41)

Hardened Naze for production deployment with environment config, dynamic routing, error handling, authentication, database integration, browser API parity, and WASM size optimization.

- **M31** Environment Variables -- `[env]` in naze.toml, compile-time `env.*` substitution, `.env` file loading
- **M32** Dynamic Routing -- `:param` path segments, catch-all `*` routes, `params.*` namespace
- **M33** Enhanced Server Functions -- multi-statement bodies, `let` bindings, `fetch` expressions, sequential evaluation
- **M34** Page Metadata -- `meta title/description/image` in page blocks, SSR override, client-side `document.title`
- **M35** Error Boundaries -- `boundary { ... } catch { ... }`, compile-time desugaring to `__if` nodes
- **M36** Guards and Middleware -- `guard` definitions, `page guard:` references, SSR 302 redirects
- **M37** Authentication Patterns -- header interpolation in `data` blocks, server-side auth forwarding, auth project template
- **M38** Database Integration -- `sql` expression in server functions, parameterized queries, PostgreSQL + SQLite via feature flags
- **M39** Declarative Database Queries -- `model` definitions, `find`/`insert`/`update`/`delete` expressions, compile-time SQL generation
- **M40** Browser API Parity -- textarea rendering, real Notification API, JS interop implementation, geolocation + accelerometer device APIs
- **M41** WASM Size Optimization -- wasm-opt enabled, 14 unused web-sys features removed, `state_key()` helper for format!() reduction

**Stats at completion:** 389 workspace tests, ~157 grammar rules, 395KB WASM runtime (budget: 405KB), 109 example files
