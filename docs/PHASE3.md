# Phase 3: Language Completion & Developer Experience

**Goal:** Complete the language with Tier 1 computation features (pipeline operators, pattern matching, pure functions), finalize developer tooling (testing, incremental compilation), and polish cross-platform builds. Target: sub-second hot reload, testing in CI, AI generates correct Naze >80% for common patterns.

**Architecture shift:** Phase 2's runtime handles state, events, and rendering. Phase 3 adds a computation layer (pipeline operators, pattern matching) and an application logic layer (shared state, computed values, full HTTP, WebSocket streams, browser storage, timers, URL parameters). Together these let apps express data transformations and application behavior declaratively, without escaping to JavaScript. The compiler gains fusion optimizations (multi-stage pipelines compiled to single-pass iteration).

**Prerequisite:** Phase 2 milestones M1-M14 complete. See [PHASE2.md](PHASE2.md).

---

## Deferred from Phase 2

These items were tracked in Phase 2 but deferred to Phase 3 milestones:

| Item | Origin | Target |
|------|--------|--------|
| Warn on unknown theme token references | M6 | M19 |
| Screen reader live region announcements | M8d | M22 |
| Touch scroll | M8e | M22 |
| Virtual scrolling for large lists | M8e | M22 |
| Project-aware type-checking in LSP | M9 | M21 |
| VS Code Marketplace publish | M9 | M21 |
| Incremental compilation | M10 | M22 |
| Build timing output | M10 | M22 |
| Standalone native binary | M11 | M22 |
| GPU renderer option (wgpu) | M11 | M22 |
| End-to-end Android APK build | M12 | M22 |
| `animate` blocks (keyframe animations) | M14 | M18 |

---

## Phase 3a: Language Completion

### M15: Pipeline Operators & Pure Functions
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Tier 1 computation: the missing piece that makes Naze more than a layout DSL. Pipeline operators let apps transform data declaratively without escaping to JavaScript or WASM imports.

- [ ] Grammar: pipeline expression rule (`expression ("|" pipe_stage)*`)
- [ ] Grammar: `function` definition rule (`function name(args) -> type { expression }`)
- [ ] AST: `Node::PipeExpr`, `Node::Function` variants
- [ ] IR: pipeline operation nodes, function call nodes
- [ ] Built-in pipeline functions: `filter`, `map`, `sort-by`, `take`, `sum`
- [ ] Built-in pipeline functions: `reduce`, `group-by`, `flatten`, `distinct`, `zip`
- [ ] Compiler: type-check pipeline chains (input/output type compatibility)
- [ ] Compiler: pipeline fusion optimization (`filter` → `map` → `take` compiles to single-pass iteration)
- [ ] Compiler: pure function inlining for small expression bodies
- [ ] Compiler: constant folding for pure function calls with literal arguments
- [ ] Runtime: pipeline operator execution (WASM)
- [ ] Runtime: function call dispatch

**Example syntax:**
```naze
function full-name(first: text, last: text) -> text {
  "{first} {last}"
}

state items = [
  { name: "Alice", score: 85 },
  { name: "Bob", score: 92 },
  { name: "Carol", score: 78 }
]

each item in items | filter score > 80 | sort-by score {
  text "{item.name}: {item.score}"
}
```

### M16: Pattern Matching & List Comprehensions
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Depends on M15 (shares expression infrastructure).

- [ ] Grammar: `match` expression with pattern arms
- [ ] Grammar: list comprehension syntax (`[expr for item in list if condition]`)
- [ ] AST: `Node::Match`, `Node::ListComprehension` variants
- [ ] IR: match nodes with pattern arms, list comprehension nodes
- [ ] Compiler: exhaustiveness checking for match arms (all cases covered)
- [ ] Compiler: wildcard `_` pattern support
- [ ] Compiler: destructuring in match patterns (e.g., `{ name, score }`)
- [ ] Runtime: match evaluation
- [ ] Runtime: list comprehension execution

**Example syntax:**
```naze
match status {
  "loading": text "Please wait..."
  "error": text "Something went wrong" color: #dc2626
  "success": text "Done!" color: #16a34a
  _: text "Unknown state"
}

let high-scores = [item.name for item in items if item.score > 80]
```

### M17: Layout Templates & Responsive Design
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-layout`

Custom templates and responsive breakpoints for production layouts.

- [ ] Grammar: `template` definition rule with named slots
- [ ] AST: `Node::Template` variant
- [ ] Compiler: template expansion to spatial primitives at compile time
- [ ] Built-in template library: `app-shell(toolbar, sidebar, main, footer)`
- [ ] Built-in template library: `dashboard(header, cards, detail-panel)`
- [ ] Built-in template library: `sidebar-layout(nav, content)`
- [ ] Built-in template library: `split-view(left, right)`, `centered(content)`
- [ ] `responsive` property on layout containers (e.g., `responsive: stack below 768px`)
- [ ] `collapsible: below Npx` for panels that hide at small viewport widths
- [ ] Layout engine: breakpoint evaluation during layout pass (viewport width check)

**Example syntax:**
```naze
template "my-dashboard"(top-bar, filters, card-grid, detail) {
  grid {
    row height: 64px {
      slot "top-bar"
    }
    row flex-grow: 1 {
      column width: 240px {
        slot "filters"
      }
      column flex-grow: 1 {
        scroll { slot "card-grid" }
      }
      column width: 400px, collapsible: below 1200px {
        slot "detail"
      }
    }
  }
}
```

### M18: Advanced Animation
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Builds on M14 (basic transitions). Adds spring physics, keyframes, and GPU fast path.

- [ ] Spring physics easing: `physics: spring(stiffness: 300, damping: 20)`
- [ ] Keyframe animations: `animate scale: [1, 1.2, 0.95, 1] over 400ms`
- [ ] `animate` block syntax for explicit multi-step animations
- [ ] Custom easing curves: `cubic-bezier(x1, y1, x2, y2)`
- [ ] GPU fast path: transform/opacity changes skip re-layout, update renderer directly
- [ ] Runtime: spring physics solver (stiffness + damping parameters)
- [ ] Runtime: keyframe interpolation between multiple values

### M19: Component Events & Theme Inheritance
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Components emit custom events to parents. Themes support inheritance and runtime switching.

- [ ] Grammar: `emit` action syntax (`on click: emit toggle-sidebar`)
- [ ] Compiler: validate emit events match parent `on` handlers
- [ ] Runtime: event propagation from child component to parent
- [ ] Grammar: `extends` in theme definitions (inherit + override tokens)
- [ ] Compiler: theme inheritance resolution (base → derived)
- [ ] Runtime: theme switching without recompile (swap token values at runtime)
- [ ] Compiler: warn on unknown theme token references (carried from M6)

**Example syntax:**
```naze
-- Component emits event
component sidebar-toggle {
  rect width: 40px, height: 40px, color: #333333, role: "button", label: "Toggle sidebar" {
    on click: emit toggle-sidebar
  }
}

-- Parent handles event
sidebar-toggle {
  on toggle-sidebar: set sidebar-open = !sidebar-open
}

-- Theme inheritance
theme dark extends default {
  colors {
    background: #1a1a2e
    text: #e0e0e0
  }
}
```

### M19b: Overlay System & Interaction Primitives
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `naze-renderer`, `naze-layout`

The missing layer that enables all overlay-based components (dialogs, dropdowns, tooltips, popovers, toasts, etc.). This is the single highest-leverage addition — it unblocks 17 shadcn/ui-equivalent components. See [PARITY.md](PARITY.md) for the full gap analysis.

- [ ] Grammar: `overlay` element rule (renders children above normal content flow)
- [ ] Layout: overlay layer — separate render pass after main content, paints on top
- [ ] Runtime: overlay z-ordering (most recent overlay on top; nested overlays stack correctly)
- [ ] `focus-trap: true` prop — constrains Tab/Shift+Tab cycling to children of this subtree
- [ ] `scroll-lock: true` prop — prevents background scroll while overlay is visible
- [ ] `on click-outside: action` event — fires when user clicks anywhere outside the element subtree
- [ ] `anchor: "element-id"` prop — positions overlay relative to a trigger element (below by default)
- [ ] Anchor placement options: `anchor-placement: "bottom"`, `"top"`, `"left"`, `"right"` (auto-flip when near viewport edge)
- [ ] `on context-menu: action` event — right-click handler
- [ ] `on pointer-move: action` event — continuous pointer position tracking (for resize handles, custom drag)
- [ ] Arrow key events: `on arrow-up`, `on arrow-down`, `on arrow-left`, `on arrow-right`
- [ ] Runtime: dismiss overlay on Escape key (configurable)

**Example syntax:**
```naze
-- Dialog with overlay
state dialog-open = false

rect role: "button" { text "Open"  on click: set dialog-open = true }

if dialog-open {
  overlay focus-trap: true, scroll-lock: true, on click-outside: set dialog-open = false {
    rect width: 100%, height: 100%, color: #00000080
    container width: 480px, padding: 24px, color: #ffffff, radius: 12px, shadow: lg {
      heading "Confirm Action"
      text "This cannot be undone."
      row gap: 12px, justify: end {
        rect role: "button" { text "Cancel"  on click: set dialog-open = false }
        rect role: "button", color: #dc2626 { text "Confirm" color: #fff  on click: set dialog-open = false }
      }
    }
  }
}

-- Dropdown anchored to trigger
state menu-open = false
rect id: "menu-btn", role: "button" { text "Options"  on click: set menu-open = !menu-open }
if menu-open {
  overlay anchor: "menu-btn", on click-outside: set menu-open = false {
    column color: #ffffff, shadow: md, radius: 8px, padding: 4px {
      rect role: "menuitem" { text "Edit"  on click: set menu-open = false }
      rect role: "menuitem" { text "Delete" color: #dc2626  on click: set menu-open = false }
    }
  }
}
```

### M19c: Visual Properties Expansion ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `naze-renderer`

Styling properties needed for production visual fidelity. Without these, components work but lack polish (no shadows, no centered text, no truncation). See [PARITY.md](PARITY.md).

- [x] `shadow` prop: named presets (`shadow: sm`, `shadow: md`, `shadow: lg`, `shadow: xl`) and custom values (`shadow: "0 4px 6px rgba(0,0,0,0.1)"`)
- [x] `text-align` prop: `start`, `center`, `end`, `justify`
- [x] `text-overflow` prop: `clip` (default), `ellipsis` (truncate with "...")
- [x] `text-decoration` prop: `none`, `underline`, `line-through`
- [x] `line-height` prop: numeric multiplier (e.g., `1.5`) or unit value (`24px`)
- [x] `letter-spacing` prop: unit value (`0.5px`, `1px`)
- [x] `gradient` prop: `gradient: "linear(to-right, #3b82f6, #8b5cf6)"`, `gradient: "radial(#fff, #000)"`
- [x] `transform` prop: `transform: "rotate(45deg)"`, `transform: "scale(1.2)"`, `transform: "translate(10px, 5px)"`
- [x] `cursor` prop: `pointer`, `grab`, `grabbing`, `text`, `not-allowed`, `crosshair`, `move`, `resize`
- [x] `overflow` prop on non-scroll containers: `visible` (default), `hidden`, `clip`
- [x] Renderer: shadow rendering via Canvas2D `shadowBlur`/`shadowColor`/`shadowOffset`
- [x] Renderer: gradient fills via Canvas2D `createLinearGradient`/`createRadialGradient`
- [x] Renderer: transform matrix via Canvas2D `setTransform`/`rotate`/`scale`
- [x] Layout: text measurement with alignment, overflow truncation, line-height

**Example syntax:**
```naze
-- Card with shadow and centered heading
container shadow: lg, padding: 24px, radius: 12px, color: #ffffff {
  heading "Dashboard" text-align: center
  text "Long content that might overflow..." text-overflow: ellipsis
  text "Learn more" text-decoration: underline, color: #2563eb, cursor: pointer
}

-- Gradient background
rect width: 300px, height: 100px, gradient: "linear(to-right, #3b82f6, #8b5cf6)", radius: 8px

-- Loading spinner
rect width: 24px, height: 24px, border: 3px, border-color: #3b82f6, radius: 12px,
    transform: "rotate(0deg)", transition: "transform 1000ms linear"
```

### M19d: Application Logic Primitives ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

The missing application-level features that close the gap from ~40% to ~82% application logic parity. These are all tied to the reactivity/rendering loop — WASM imports can't handle them because they must observe state changes, trigger re-renders, or access browser APIs. See [PARITY.md](PARITY.md) for the full gap analysis and design rationale.

**Design principle:** Only add primitives for features tied to reactivity. Pure computation stays in WASM imports (Tier 2). Server logic stays in server functions (Tier 3). Each new keyword has one canonical form — no aliases, no options. Total grammar expansion: ~6 new rules (~40% of Phase 2's grammar scope).

#### Tier A — State Extensions

- [x] Grammar: `computed_stmt` rule (`computed name = expression`)
- [x] AST: `Node::Computed` variant
- [x] IR: `ComputedDecl { name, expression }` added to `RenderTree`
- [x] Compiler: dependency analysis — scan expression for state/computed refs at compile time
- [x] Compiler: cycle detection — error if computed values form a dependency cycle
- [x] Runtime: evaluate computed values after state change, before render
- [x] Runtime: skip re-evaluation when dependencies haven't changed

- [x] Grammar: `shared` modifier on `state_stmt` (`shared state name = value`)
- [ ] Grammar: optional grouping block (`shared state auth { user = null, token = "" }`) *(deferred — simple form works)*
- [x] AST: `SharedState` flag on state nodes
- [x] Compiler: validate shared state names are unique across all files
- [x] Runtime: shared state persists across `navigate` actions (not scoped to page)
- [x] Runtime: changes to shared state trigger re-render on any page that references it

- [x] Grammar: `storage_stmt` rule (`storage name: local "key" default: value`)
- [x] AST: `Node::Storage` variant
- [x] IR: `StorageDecl { name, storage_type, key, default }`
- [x] Runtime: initialize from localStorage/sessionStorage on load, fall back to default
- [x] Runtime: auto-sync to storage on `set` (JSON serialization for non-string values)
- [x] Runtime: reactive — changes trigger re-render like normal state

#### Tier A — Data Extensions

- [x] Grammar: optional block body on `data_stmt` (`data name: fetch "url" { ... }`)
- [x] Grammar: data block properties: `method`, `params`, `headers`, `body`, `cache`, `retry`, `trigger`, `content-type`
- [x] AST: `DataConfig` struct on `Node::Data` (method, params, headers, body, cache, retry, trigger)
- [x] IR: extended `DataDecl` with HTTP configuration fields
- [x] Runtime: HTTP method support (GET, POST, PUT, DELETE)
- [x] Runtime: request params (append to URL as query string)
- [x] Runtime: custom headers (with string interpolation for auth tokens)
- [x] Runtime: request body serialization (JSON default, multipart for file uploads)
- [x] Runtime: response caching by URL+params with TTL expiry
- [x] Runtime: retry with exponential backoff on network failure
- [x] Runtime: `trigger: manual` — suppress auto-fetch; activated by `trigger name` action
- [x] Runtime: reactive URL interpolation — re-fetch when interpolated state values change (GET only)

- [x] Grammar: `stream` variant in data_stmt (`data name: stream "wss://..."`)
- [x] Grammar: `type: sse` property in stream data block
- [x] AST: `DataSource::Stream` variant (vs `DataSource::Fetch`)
- [x] IR: stream data source type in `DataDecl`
- [x] Runtime: WebSocket connection management (connect, auto-reconnect with backoff)
- [x] Runtime: Server-Sent Events connection management
- [x] Runtime: append incoming messages to `.data` reactive list
- [x] Runtime: reactive URL interpolation — close old connection, open new on URL change
- [x] `send` action — push message to a WebSocket stream

#### Tier B — Scheduling & Browser APIs

- [x] Grammar: `param_stmt` rule (`param name: type default: value`)
- [x] AST: `Node::Param` variant
- [x] IR: `ParamDecl { name, param_type, default }`
- [x] Runtime: read URL query string on init, populate state with parsed values
- [x] Runtime: on `set`, update both state and URL via `replaceState`
- [x] Runtime: on `popstate`, sync from URL back to state and re-render

- [x] Grammar: `timer_stmt` rule (`timer name: after/every duration { action }`)
- [x] Grammar: duration literal (`number ("ms" | "s" | "min")`)
- [x] AST: `Node::Timer` variant with `TimerKind::After` | `TimerKind::Every`
- [x] IR: `TimerDecl { name, kind, duration_ms, action }`
- [x] Runtime: `after` — single `setTimeout`, execute action, done
- [x] Runtime: `every` — `setInterval`, execute action each tick
- [x] Runtime: automatic cleanup when page/component unmounts

- [x] Grammar: `debounce` and `throttle` modifiers on `on_handler`
- [x] AST: optional `EventModifier { kind, duration_ms }` on event handlers
- [x] Runtime: debounce — delay action until N ms of inactivity
- [x] Runtime: throttle — execute at most once per N ms

- [x] Grammar: `copy` action variant (`copy expression`)
- [x] Runtime: evaluate expression, write to clipboard via `navigator.clipboard.writeText()`

- [x] Grammar: `send` action variant (`send stream-name expression`)
- [x] Runtime: send message on named WebSocket stream

- [x] Grammar: `trigger` action variant (`trigger data-name`)
- [x] Runtime: execute manual-trigger data fetch

- [x] `input type: "file"` variant with `accept` and `max-size` props
- [x] Runtime: file input binding — store selected File in state
- [x] Runtime: multipart form data encoding for file upload via enhanced `data` POST

**Example syntax:**
```naze
-- Computed state (read-only, auto-updates)
computed filtered = items | filter status == "active"
computed total = cart | map price * quantity | sum

-- Shared state (persists across pages)
shared state auth {
  user = null
  token = ""
}

-- Persistent storage (survives browser close)
storage theme: local "theme-preference" default: "light"
storage cart: local "cart-items" default: []

-- Full HTTP with enhanced data
data users: fetch "/api/users" {
  method: get
  params: { page: current-page }
  headers: { "Authorization": "Bearer {auth.token}" }
  cache: 5min
  retry: 3
}

data save-result: fetch "/api/users" {
  method: post
  body: { name: name-input, email: email-input }
  trigger: manual
}
on click: trigger save-result

-- WebSocket stream
data chat: stream "wss://api.example.com/chat/{room-id}"
on click: send chat "{message-input}"

-- URL parameters
param page: number default: 1
param search: text default: ""

-- Timers
timer toast-dismiss: after 5s { set show-toast = false }
timer auto-save: every 30s { trigger save-result }

-- Debounce/throttle on events
input bind: search, on change debounce 300ms: trigger search-results

-- Clipboard
on click: copy share-url

-- File upload
input type: "file", bind: avatar, accept: "image/*", max-size: 5mb
data upload: fetch "/api/upload" { method: post, body: { file: avatar }, content-type: multipart, trigger: manual }
on click: trigger upload
```

### M19e: Remaining Gap Closures (Textarea, JS Interop, Browser APIs)
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `nazec`

Three additions that close the final parity gaps from ~97% to ~99%. Total grammar expansion: ~3 new rules. See [PARITY.md](PARITY.md) for the full analysis.

#### Textarea Element

- [ ] Parser: `textarea` element keyword (same prop pattern as `input`)
- [ ] IR: textarea node kind in `RenderNode`
- [ ] Runtime: multi-line text input rendering with line breaks
- [ ] Runtime: two-way binding via `bind` (same as `input`)
- [ ] Runtime: `rows` prop for visible height, `max-length` for character limit
- [ ] Runtime: validation rules (`required`, `min-length`, `max-length`) same as `input`

#### JS Interop

- [ ] Grammar: `js_action` rule (`js "functionName"(args)` and `js "name"(args) -> target`)
- [ ] Grammar: `data: js` variant (`data name: js "functionName"(args)`)
- [ ] AST: `Action::JsCall` variant, `DataSource::JsCall` variant
- [ ] IR: JS call action and data source types
- [ ] Compiler: validate `js` references against `naze.toml` `[scripts]` declarations (warn on undeclared)
- [ ] Build pipeline: embed `<script>` tags from `naze.toml` `[scripts]` into generated `index.html`
- [ ] Runtime: sync JS calls via `js_sys::Function` or `wasm_bindgen` `eval`
- [ ] Runtime: type marshalling (number↔f64, text↔string, bool↔boolean, list↔Array, object↔Object)
- [ ] Runtime: `js "name"(args) -> target` — store return value in state, trigger re-render
- [ ] Runtime: `data: js` async variant — `.loading`/`.error`/`.data` lifecycle for Promise-returning functions
- [ ] Runtime: return value conversion — unconvertible values JSON-stringified to text

**Example syntax:**
```naze
-- naze.toml:
-- [scripts]
-- stripe = "https://js.stripe.com/v3/"

-- Sync call
on click: js "analytics.track"("purchase", { amount: total })

-- Async call with data lifecycle
data checkout: js "createCheckoutSession"(cart-items) { trigger: manual }
on click: trigger checkout
if checkout.loading { text "Processing payment..." }
```

#### Browser Device APIs

- [ ] Grammar: `device` keyword as data source (`data name: device API_NAME`)
- [ ] Grammar: `notify_action` rule (`notify "title" { body: "text", icon: "url" }`)
- [ ] AST: `DataSource::Device` variant, `Action::Notify` variant
- [ ] IR: device data source type, notify action type
- [ ] Runtime: geolocation one-shot (`navigator.geolocation.getCurrentPosition`)
- [ ] Runtime: geolocation watch (`navigator.geolocation.watchPosition`) with `{ watch: true }`
- [ ] Runtime: camera access (`navigator.mediaDevices.getUserMedia`) — store stream in state
- [ ] Runtime: permission handling — denied permissions surface as `.error` state
- [ ] Runtime: `notify` action — request notification permission on first use, show `Notification`
- [ ] Runtime: device data `.loading`/`.error`/`.data` lifecycle (same as fetch/stream)

**Example syntax:**
```naze
-- Geolocation
data location: device geolocation
if location.loading { text "Acquiring location..." }
if location.data { text "Lat: {location.data.latitude}, Lng: {location.data.longitude}" }

-- Notification
on click: notify "Order Shipped!" { body: "Your order is on its way." }
```

---

## Phase 3b: Testing & Tooling

### M20: Testing Framework
**Crates:** new `naze-test` (or integrated into `nazec`), `naze-layout`, `naze-ir`

First-class testing using the same Naze language. Essential for CI/CD and confidence in app correctness.

- [ ] Grammar: `test` block syntax in `.test.naze` files
- [ ] Grammar: `flow` block syntax for multi-page test flows
- [ ] Grammar: `assert` statement variants:
  - `assert text "X" is visible`
  - `assert emitted event-name`
  - `assert no accessibility violations`
  - `assert not visible`
- [ ] Headless renderer: software-only (reuses `naze-layout` + `naze-native` renderer), no canvas/GPU
- [ ] Component test runner: render with props → simulate events → assert output/state
- [ ] Flow test runner: navigate pages, simulate user journey across routes
- [ ] `nazec test` CLI command: discover `.test.naze` files, run, report results
- [ ] Structured output: JSON results for CI integration (pass/fail + assertion details + timing)
- [ ] Screenshot comparison for visual regression (stretch goal)

**Example syntax:**
```naze
use components/counter

test "counter increments on click" {
  render counter count: 0
  assert text "0" is visible
  click "Increment"
  assert text "1" is visible
}

flow "login and navigate to dashboard" {
  navigate "/"
  assert text "Login" is visible
  fill "email" with "user@example.com"
  fill "password" with "secret"
  click "Sign In"
  assert page is "/dashboard"
  assert text "Welcome" is visible
}
```

### M21: VS Code Extension Polish
**Crate:** `naze-lsp`, **Dir:** `editors/vscode/`

Carried from Phase 2 M9. The extension exists with syntax highlighting, LSP, visual editor, and code actions. This milestone finishes the remaining gaps.

- [ ] Full project-aware type-checking in LSP (currently parse-only; add multi-file resolution + type errors)
- [ ] Cross-file go-to-definition (jump from `use` import to component source)
- [ ] Format document support (integrate `nazec fmt` if available, or LSP formatter)
- [ ] Publish to VS Code Marketplace (or provide `.vsix` download link)
- [ ] Visual editor stability testing and feature parity with text editor

### M22: Build Pipeline Polish
**Crate:** `nazec`, `naze-native`

Carried from Phase 2 M10-M12. Dev server and cross-platform builds exist. This milestone addresses polish gaps.

- [ ] Incremental compilation: only re-parse changed files, reuse cached AST/IR for unchanged files
- [ ] Console output: "rebuilt in Xms" timing for `nazec dev` and `nazec run`
- [ ] `nazec build --target native` produces fully standalone binary (no runtime dependencies)
- [ ] `nazec build --target android` produces installable APK end-to-end
- [ ] Runtime: touch scroll in scroll containers (carried from M8e)
- [ ] Runtime: screen reader live region announcements (carried from M8d)
- [ ] Virtual scrolling for large lists (carried from M8e; stretch goal)
- [ ] Source map generation: binary offset → `.naze` source location (needed for M20 test output and future debugger)

---

## Build Order

```
Track A (language):    M15 → M16 (M16 depends on M15 expression infrastructure)
Track B (layout):      M17 (independent, parallel with Track A)
Track C (animation):   M18 (independent, builds on M14)
Track D (components):  M19 → M19b ✅ (overlay builds on M19 component events)
Track E (visual):      M19c ✅
Track F (app logic):   M19d ✅ → M19e (M19e extends M19d's data/action infrastructure)
Track G (testing):     M20 (independent, can start immediately)
Track H (tooling):     M21, M22 (parallel with everything)
```

All tracks can run in parallel except M15 → M16, M19 → M19b, and M19d → M19e. M19d's state/data extensions are independent of other milestones. M19e extends M19d's data source and action infrastructure with JS interop and device APIs. The `computed` feature benefits from M15 pipeline syntax but can ship with simple expressions first.

**Suggested priority order:**
1. ~~M19b (overlay system)~~ — **Complete**
2. ~~M19d (app logic primitives)~~ — **Complete**
3. ~~M19c (visual properties)~~ — **Complete**
4. M15 (pipeline operators) — highest-impact language feature; unlocks full `computed` expressions
5. M19e (remaining gap closures) — textarea, JS interop, browser device APIs
6. M20 (testing framework) — enables CI/CD, validates other milestones
7. M16 (pattern matching) — completes Tier 1 computation
8. M17 (templates/responsive) — production layout quality
9. M19 (component events, theme inheritance) — component model completion
10. M18 (advanced animation) — UI polish
11. M21, M22 (tooling polish) — can be interleaved throughout

## WASM Size Budget

Phase 2 runtime: ~75KB. Phase 3 adds pipeline operators, pattern matching, templates, and application logic primitives. Pipeline/match are compile-time features that add minimal runtime code (the runtime already handles `each` iteration; pipelines extend this). Templates are fully compile-time (expanded to primitives). M19d's application logic primitives add runtime code for: WebSocket connections, localStorage/sessionStorage access, URL parameter sync, timer scheduling, clipboard API, and enhanced HTTP (methods, headers, caching, retry). These are browser API calls — small code, no heavy dependencies. Target: **< 175KB** for the runtime WASM.
