# Phase 3: Language Completion & Developer Experience

**Status:** Complete. All milestones delivered except M21 (LSP polish, moved to Phase 6 as M45).

**Goal:** Complete the language with Tier 1 computation features (pipeline operators, pattern matching, pure functions), finalize developer tooling (testing, incremental compilation), and polish cross-platform builds. Target: sub-second hot reload, testing in CI, AI generates correct Naze >80% for common patterns.

**Architecture shift:** Phase 2's runtime handles state, events, and rendering. Phase 3 adds a computation layer (pipeline operators, pattern matching, pure functions) and an application logic layer (shared state, computed values, full HTTP, WebSocket streams, browser storage, timers, URL parameters). Together these let apps express data transformations and application behavior declaratively, without escaping to JavaScript. Key design: pipeline operators execute at runtime (10 built-in functions), while pure functions inline at compile time (AST-level substitution) and pattern matching desugars to if/else chains at compile time (no new IR/runtime constructs).

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

### M15: Pipeline Operators & Pure Functions ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Tier 1 computation: the missing piece that makes Naze more than a layout DSL. Pipeline operators let apps transform data declaratively without escaping to JavaScript or WASM imports.

- [x] Grammar: pipeline expression rule (`expression ("|" pipe_stage)*`)
- [x] Grammar: `function` definition rule (`function name(args) -> type { expression }`)
- [x] AST: `Expression::Pipeline`, `Node::Function`, `Expression::FunctionCall` variants
- [x] IR: pipeline stage nodes with function IDs and arguments
- [x] Built-in pipeline functions: `filter`, `map`, `sort-by`, `take`, `sum`, `count`
- [x] Built-in pipeline functions: `reduce`, `group-by`, `flatten`, `distinct`
- [ ] Built-in pipeline function: `zip` *(deferred — needs two source operands)*
- [x] Compiler: type-check pipeline stages (validate required arguments per function)
- [ ] Compiler: pipeline fusion optimization (`filter` → `map` → `take` compiles to single-pass iteration) *(deferred)*
- [x] Compiler: pure function inlining (AST-level parameter substitution at compile time)
- [ ] Compiler: constant folding for pure function calls with literal arguments *(deferred)*
- [x] Runtime: pipeline operator execution (WASM + native + gallery + run + native-build — all 5 eval sites)
- [x] Runtime: function calls resolved at compile time (inlined, no runtime dispatch needed)

**Example syntax:**
```naze
function area(w: number, h: number) -> number {
  w * h
}

state items = [
  { name: "Alice", score: 85 },
  { name: "Bob", score: 92 },
  { name: "Carol", score: 78 }
]

computed total = items | map score | reduce acc + it 0
computed top = items | filter score > 80 | sort-by score

each item in items | filter score > 80 | sort-by score {
  text "{item.name}: {item.score}"
}
```

### M16: Pattern Matching ✅
**Crates:** `naze-parser`, `naze-compiler`

Depends on M15 (shares expression infrastructure). Pattern matching desugars to nested if/else chains at compile time — no IR or runtime changes needed.

- [x] Grammar: `match` statement with pattern arms
- [x] AST: `Node::Match`, `MatchArm`, `MatchPattern` types
- [x] Compiler: exhaustiveness checking (warns if no wildcard `_` arm)
- [x] Compiler: wildcard `_` pattern support
- [x] Compiler: desugaring to nested `__if` RenderNodes (no new IR/runtime constructs)
- [x] Compiler: duplicate pattern detection (warning)
- [ ] Grammar: list comprehension syntax (`[expr for item in list if condition]`) *(deferred — syntactic sugar for pipelines)*
- [ ] Compiler: destructuring in match patterns (e.g., `{ name, score }`) *(deferred)*

**Example syntax:**
```naze
match status {
  "loading": text "Please wait..."
  "error": text "Something went wrong" color: #dc2626
  "success": text "Done!" color: #16a34a
  _: text "Unknown state"
}
```

### M17: Layout Templates & Responsive Design ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-layout`

Custom templates and responsive breakpoints for production layouts.

- [x] Grammar: `template` definition rule with named slots
- [x] AST: `Node::Template` variant
- [x] Compiler: template expansion to spatial primitives at compile time
- [x] Built-in template library: `app-shell(toolbar, sidebar, main, footer)`
- [x] Built-in template library: `dashboard(header, cards, detail-panel)`
- [x] Built-in template library: `sidebar-layout(nav, content)`
- [x] Built-in template library: `split-view(left, right)`, `centered(content)`
- [x] `responsive` property on layout containers (e.g., `responsive: 768px`)
- [x] `collapsible: Npx` for panels that hide at small viewport widths
- [x] Layout engine: breakpoint evaluation during layout pass (viewport width check)

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

### M18: Advanced Animation ✅
**Crates:** `naze-runtime`

Builds on M14 (basic transitions). Adds spring physics, keyframes, and layout-skip fast path. Entirely runtime-only — no grammar/AST/IR/compiler changes.

- [x] Spring physics easing: `transition: "color spring(180, 12)"`
- [x] Keyframe animations: `animate: "scale [1, 1.2, 0.95, 1] 400ms ease-in-out"`
- [x] `animate` prop syntax for multi-step keyframe sequences
- [x] Custom easing curves: `transition: "width 500ms cubic-bezier(0.34, 1.56, 0.64, 1)"`
- [x] Layout-skip fast path: transform/opacity/color changes skip re-layout, reuse cached layout
- [x] Runtime: spring physics solver (damped oscillation with stiffness + damping)
- [x] Runtime: keyframe interpolation between multiple values (numbers + colors)

### M19: Component Events & Theme Inheritance ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Components emit custom events to parents. Themes support inheritance and runtime switching.

- [x] Grammar: `emit` action syntax (`on click: emit toggle-sidebar`)
- [x] Compiler: validate emit events match parent `on` handlers
- [x] Runtime: event propagation from child component to parent (compile-time inlining)
- [x] Grammar: `extends` in theme definitions (inherit + override tokens)
- [x] Compiler: theme inheritance resolution (base → derived)
- [x] Runtime: theme switching without recompile (swap token values at runtime)
- [x] Compiler: warn on unknown theme token references (carried from M6)

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

### M19b: Overlay System & Interaction Primitives ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `naze-renderer`, `naze-layout`

The missing layer that enables all overlay-based components (dialogs, dropdowns, tooltips, popovers, toasts, etc.). This is the single highest-leverage addition — it unblocks 17 shadcn/ui-equivalent components. See [PARITY.md](PARITY.md) for the full gap analysis.

- [x] Grammar: `overlay` element rule (renders children above normal content flow)
- [x] Layout: overlay layer — separate render pass after main content, paints on top
- [x] Runtime: overlay z-ordering (most recent overlay on top; nested overlays stack correctly)
- [x] `focus-trap: true` prop — constrains Tab/Shift+Tab cycling to children of this subtree
- [x] `scroll-lock: true` prop — prevents background scroll while overlay is visible
- [x] `on click-outside: action` event — fires when user clicks anywhere outside the element subtree
- [x] `anchor: "element-id"` prop — positions overlay relative to a trigger element (below by default)
- [x] Anchor placement options: `anchor-placement: "bottom"`, `"top"`, `"left"`, `"right"` (auto-flip when near viewport edge)
- [x] `on context-menu: action` event — right-click handler
- [x] `on pointer-move: action` event — continuous pointer position tracking (for resize handles, custom drag)
- [x] Arrow key events: `on arrow-up`, `on arrow-down`, `on arrow-left`, `on arrow-right`
- [x] Runtime: dismiss overlay on Escape key (configurable via `dismiss-on-escape: false`)

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

### M19e: Remaining Gap Closures (Textarea, JS Interop, Browser APIs) ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `nazec`

Three additions that close the final parity gaps from ~97% to ~99%. Total grammar expansion: ~3 new rules. See [PARITY.md](PARITY.md) for the full analysis.

#### Textarea Element

- [x] Parser: `textarea` element keyword (same prop pattern as `input`)
- [x] IR: textarea node kind in `RenderNode`
- [x] Runtime: multi-line text input rendering with line breaks
- [x] Runtime: two-way binding via `bind` (same as `input`)
- [x] Runtime: `rows` prop for visible height, `max-length` for character limit
- [x] Runtime: validation rules (`required`, `min-length`, `max-length`) same as `input`

#### JS Interop

- [x] Grammar: `js_action` rule (`js "functionName"(args)` and `js "name"(args) -> target`)
- [x] Grammar: `data: js` variant (`data name: js "functionName"(args)`)
- [x] AST: `Action::JsCall` variant, `DataSource::JsCall` variant
- [x] IR: JS call action and data source types
- [ ] Compiler: validate `js` references against `naze.toml` `[scripts]` declarations (warn on undeclared) *(deferred — runtime responsibility)*
- [x] Build pipeline: embed `<script>` tags from `naze.toml` `[scripts]` into generated `index.html`
- [x] Runtime: sync JS calls via `js_sys::Function` or `wasm_bindgen` `eval`
- [x] Runtime: type marshalling (number↔f64, text↔string, bool↔boolean, list↔Array, object↔Object)
- [x] Runtime: `js "name"(args) -> target` — store return value in state, trigger re-render
- [x] Runtime: `data: js` async variant — `.loading`/`.error`/`.data` lifecycle for Promise-returning functions
- [x] Runtime: return value conversion — unconvertible values JSON-stringified to text

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

- [x] Grammar: `device` keyword as data source (`data name: device API_NAME`)
- [x] Grammar: `notify_action` rule (`notify "title" { body: "text", icon: "url" }`)
- [x] AST: `DataSource::Device` variant, `Action::Notify` variant
- [x] IR: device data source type, notify action type
- [x] Runtime: geolocation one-shot (`navigator.geolocation.getCurrentPosition`)
- [x] Runtime: geolocation watch (`navigator.geolocation.watchPosition`) with `{ watch: true }`
- [x] Runtime: camera access (`navigator.mediaDevices.getUserMedia`) — store stream in state
- [x] Runtime: permission handling — denied permissions surface as `.error` state
- [x] Runtime: `notify` action — request notification permission on first use, show `Notification`
- [x] Runtime: device data `.loading`/`.error`/`.data` lifecycle (same as fetch/stream)

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

### M20: Testing Framework ✅
**Crates:** integrated into `nazec`, `naze-parser`

First-class testing using the same Naze language. Essential for CI/CD and confidence in app correctness.

- [x] Grammar: `test` block syntax in `.test.naze` files
- [x] Grammar: `flow` block syntax for multi-page test flows
- [x] Grammar: `assert` statement variants (text visible/not visible, state, page)
- [x] Grammar: action steps (render, click, fill, navigate, wait)
- [x] Parser: `parse_test_file` for `.test.naze` files
- [x] `nazec test` CLI command: discover `.test.naze` files, run, report results
- [x] Headless renderer: software-only layout engine (`naze-layout`), no canvas/GPU — 1024x768 viewport
- [x] Component test runner: compile → render → simulate click/fill/navigate → assert text/state/page
- [x] Structured output: `nazec test --format json` with per-suite/per-test/per-assertion detail
- [ ] Screenshot comparison for visual regression *(stretch goal)*

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

### M22: Build Pipeline Polish ✅
**Crate:** `nazec`, `naze-native`

Carried from Phase 2 M10-M12. Dev server and cross-platform builds exist. This milestone addresses polish gaps.

- [x] Incremental compilation: only re-parse changed files, reuse cached AST/IR for unchanged files
- [x] Console output: "built in Xms" / "rebuilt in Xms" timing for `nazec build`, `nazec dev`, and `nazec run`
- [x] `nazec build --target native` produces fully standalone binary with size output
- [ ] `nazec build --target android` produces installable APK end-to-end *(deferred — requires Gradle/SDK toolchain)*
- [x] Runtime: touch scroll in scroll containers (carried from M8e)
- [x] Runtime: screen reader live region announcements (carried from M8d)
- [x] Runtime: `role: "status"` and `role: "alert"` set `aria-live` on a11y elements
- [ ] Virtual scrolling for large lists (carried from M8e; stretch goal) *(deferred)*
- [x] Source map generation: binary offset → `.naze` source location (`app_data.map.json`)

---

## Build Order

```
Track A (language):    M15 ✅ → M16 ✅
Track B (layout):      M17 (independent)
Track C (animation):   M18 ✅ (independent, builds on M14)
Track D (components):  M19 ✅ → M19b ✅ (overlay builds on M19 component events)  ── COMPLETE
Track E (visual):      M19c ✅
Track F (app logic):   M19d ✅ → M19e ✅ (M19e extends M19d's data/action infrastructure)
Track G (testing):     M20 (independent, can start immediately)
Track H (tooling):     M21, M22 (parallel with everything)
```

M19d's state/data extensions are independent of other milestones. M19e extends M19d's data source and action infrastructure with JS interop and device APIs.

**Suggested priority order:**
1. ~~M19b (overlay system)~~ — **Complete**
2. ~~M19d (app logic primitives)~~ — **Complete**
3. ~~M19c (visual properties)~~ — **Complete**
4. ~~M15 (pipeline operators + pure functions)~~ — **Complete**
5. ~~M16 (pattern matching)~~ — **Complete**
6. ~~M19e (remaining gap closures)~~ — **Complete**
7. ~~M20 (testing framework)~~ — **Complete**
8. ~~M17 (templates/responsive)~~ — **Complete**
9. ~~M19 (component events, theme inheritance)~~ — **Complete**
10. ~~M18 (advanced animation)~~ — **Complete**
11. ~~M22 (build pipeline polish)~~ — **Complete**
12. M21 (VS Code extension polish) — remaining tooling work

## WASM Size Budget

Phase 2 runtime: ~75KB. Phase 3 adds pipeline operators, pattern matching, templates, and application logic primitives. Pipeline/match are compile-time features that add minimal runtime code (the runtime already handles `each` iteration; pipelines extend this). Templates are fully compile-time (expanded to primitives). M19d's application logic primitives add runtime code for: WebSocket connections, localStorage/sessionStorage access, URL parameter sync, timer scheduling, clipboard API, and enhanced HTTP (methods, headers, caching, retry). These are browser API calls — small code, no heavy dependencies. Target: **< 175KB** for the runtime WASM.
