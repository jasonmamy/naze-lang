# Naze Prototype Plan

> **Note:** This project was originally brainstormed as "WUI" (Web UI) and renamed to **Naze**. See the README discussion entry 26 for rationale.

A detailed build plan with fully decoupled components. Each component has its own interface contract, versioning, and can evolve independently.

---

## Architecture: Component Map

```
┌─────────────────────────────────────────────────────────────────┐
│                        USER / AI AGENT                          │
│                    (natural language intent)                     │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│  C7: AI Integration Layer                                        │
│  intent → Naze source → validate → iterate                       │
└──────────────────────────┬───────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│  C1: Naze Language                                                │
│  (.naze source files — declarative, AI-native syntax)             │
└──────────────────────────┬───────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│  C2: Compiler                                                    │
│  parse → typecheck → optimize → emit                             │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐             │
│  │ WASM backend │  │ Native ARM  │  │ Native x86   │             │
│  │ (web)        │  │ (iOS/Andr.) │  │ (desktop)    │             │
│  └─────────────┘  └─────────────┘  └──────────────┘             │
└──────────────────────────┬───────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│  C3: Runtime                                                     │
│  loads compiled binary, manages app lifecycle, state, events     │
│  ┌──────────┐  ┌───────────┐  ┌───────────┐  ┌──────────────┐  │
│  │ Event    │  │ State     │  │ Navigation│  │ Network /    │  │
│  │ System   │  │ Manager   │  │ / Routing │  │ Data Fetch   │  │
│  └──────────┘  └───────────┘  └───────────┘  └──────────────┘  │
└──────────────────────────┬───────────────────────────────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ C4: Layout   │ │ C5: Renderer │ │ C4a: Text    │
│ Engine       │ │              │ │ Engine       │
│              │ │ ┌──────────┐ │ │              │
│ flexbox-like │ │ │ WebGL    │ │ │ HarfBuzz +   │
│ constraints  │ │ │ WebGPU   │ │ │ FreeType +   │
│ responsive   │ │ │ Metal    │ │ │ ICU          │
│              │ │ │ Vulkan   │ │ │              │
│              │ │ │ DX12     │ │ │              │
│              │ │ └──────────┘ │ │              │
└──────────────┘ └──────────────┘ └──────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ C6: A11y     │ │ C8: Meta-    │ │ C9: Dev      │ │ C14: Server  │
│ Bridge       │ │ Index Gen    │ │ Tooling      │ │ Renderer &   │
│              │ │              │ │              │ │ Runtime      │
│ side DOM     │ │ HTML shell   │ │ LSP, debug,  │ │ SSR, server  │
│ → AOM later  │ │ SEO, OG tags │ │ hot reload   │ │ functions    │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
```

---

## Component Contracts (Interfaces)

Every component communicates through a defined interface. Components can be replaced, upgraded, or forked independently as long as the interface contract is honored.

### C1: Naze Language Spec

**Owns:** syntax definition, grammar, semantics, type system, layout model, component system
**Produces:** `.naze` source files
**Consumed by:** C2 (Compiler), C9 (Dev Tooling / LSP), C7 (AI Integration)
**Interface contract:**
- Formal grammar specification (BNF/PEG)
- Type system rules
- Standard library type definitions (including built-in layout templates and primitives)
- Semantic validation rules
- Component interface contract (props, slots, accessibility requirements)
**Computation model (three tiers):**
- *Tier 1 — Built-in declarative logic:* pipeline operators (`|` for chaining `filter`, `sort-by`, `map`, `take`, `sum`, etc.), pure functions with expression bodies (no side effects, no mutation), pattern matching (`match`), list comprehensions, local `let` bindings. All expressions — no statements, no imperative control flow.
- *Tier 2 — WASM library imports:* `import` keyword references pre-compiled `.wasm` modules (from packages or local paths). The compiler merges imported WASM into the app binary via `wasm-merge` + `wasm-opt`. After merging, imported functions are normal intra-module calls — zero overhead. Packages can ship as `.naze` (source) or `.wasm` (pre-compiled Rust/C/Go).
- *Tier 3 — Server functions:* `server function` runs on the server via RPC (already documented).
**Versioning:** semver on the language spec. Breaking syntax changes = major version.

#### Design Pillar 1: Human-Readable Syntax

The Naze syntax is its own purpose-built format. Not markdown, not an existing markup language. Design goal: a non-developer should be able to read a `.naze` file and roughly understand what the app does.

**Why not markdown (or Djot, AsciiDoc, MyST, etc.):**
- Markdown was designed for text documents, not UI applications
- Adopting an existing format means inheriting its limitations and ambiguities
- We'd inevitably create extensions to handle layout, interaction, and data binding — defeating the clean-slate purpose
- Naze needs constructs (slots, reactive data, interaction handlers, accessibility) that no document format supports natively

**Design principles for the syntax:**
- Brace-delimited blocks (`{}`) for structure (not indentation-based — LLMs struggle with significant whitespace)
- Keywords read like English: `use`, `data`, `on click`, `each`, `if`, `navigate to`
- Comments use `--` (short, unambiguous, low token count)
- Properties are `key: value` pairs inline or on the next indented line
- The syntax can be rendered to a formatted "document view" for human review — similar to how markdown renders to HTML, a `.naze` file renders to a readable summary of the app's structure

**Example — readable at a glance:**
```
-- A settings page. Even without knowing Naze, you can read this.

app "Settings" {

  use app-shell(toolbar-area, main)

  toolbar-area: { toolbar title: "Settings" }

  main: {
    heading "Preferences"

    section "Notifications" {
      toggle "Email notifications", bind: prefs.email-notify
      toggle "Push notifications",  bind: prefs.push-notify
    }

    section "Appearance" {
      select "Theme", options: ["Light", "Dark", "System"], bind: prefs.theme
      slider "Font size", range: 12..24, bind: prefs.font-size
    }

    row(align: end, padding-top: 24px) {
      button "Cancel", style: secondary, on click: navigate back
      button "Save",   style: primary,   on click: save prefs
    }
  }
}
```

#### Design Pillar 2: Layout Model — Named Slots on Spatial Primitives

Layout has two layers. Most users work with the high-level layer (named slots). Power users and component authors work with the low-level layer (spatial primitives).

**Layer 1: Named slot templates (high-level)**

Built-in templates that ship with the standard library:
```
use app-shell(toolbar, sidebar, main, footer)
use dashboard(header, cards, detail-panel)
use sidebar-layout(nav, content)
use split-view(left, right)
use centered(content)
```

You place content into named slots:
```
use app-shell(toolbar, sidebar, main)

toolbar: { -- toolbar content here }

sidebar: { -- sidebar content here }

main: { -- main content here }
```

**Layer 2: Spatial primitives (low-level)**

The building blocks that slot templates are made of:
```
grid { ... }     -- 2D grid layout
  row { ... }    -- horizontal container
  column { ... } -- vertical container
stack { ... }    -- layered (z-axis) container
spacer           -- flexible empty space
anchor { ... }   -- position relative to parent edge (top-right, bottom-left, etc.)
scroll { ... }   -- scrollable region
```

Properties on spatial primitives:
```
grid(columns: 3, gap: 16px, responsive: stack below 768px)
row(height: 56px, align: center, justify: space-between, padding: 0 16px)
column(width: 250px, fill: remaining)
anchor(position: top-right, offset: 16px)
scroll(direction: vertical, virtualize: true)
```

**Defining custom templates:**
```
template "my-dashboard"(top-bar, filters, card-grid, detail) {
  grid {
    row(height: 64px, span: full) { slot top-bar }
    row(fill: remaining) {
      column(width: 240px) { slot filters }
      column(fill: remaining) { scroll { slot card-grid } }
      column(width: 400px, collapsible: below 1200px) { slot detail }
    }
  }
}
```

Then used like any built-in template:
```
use my-dashboard(top-bar, filters, card-grid, detail)

top-bar: { toolbar title: "Analytics" }

filters: { -- filter controls }

card-grid: { -- cards here }

detail: { -- detail panel }
```

#### Design Pillar 3: Reusable Components

Components are defined in `.naze` files with declared inputs (props) and named content slots. They're the unit of reuse and sharing.

**Defining a component:**
```
-- components/toolbar.naze

component toolbar(title: text, show-search: bool = true, show-avatar: bool = true) {

  row(height: 56px, align: center, padding: 0 16px) {
    icon "menu", on click: emit toggle-sidebar
    heading title, size: medium
    spacer
    if show-search { search-input placeholder: "Search..." }
    if show-avatar {
      avatar current-user.image {
        on click: emit open-user-menu
      }
    }
  }

  accessibility {
    role: navigation
    label: "Main toolbar"
    keyboard: tab cycles through interactive elements
  }
}
```

**Using a component:**
```
use components/toolbar

toolbar title: "Home"
toolbar title: "Settings", show-search: false
```

**Components with content slots:**
```
-- components/card.naze

component card(variant: "default" | "highlighted" = "default") {
  container(padding: 16px, radius: 8px, shadow: small, style: variant) {
    slot header
    slot content
    slot footer?    -- optional slot (? = may be empty)
  }
}
```

```
-- Using the card with slots
card variant: "highlighted" {
  header: {
    heading "Revenue"
    badge "Live"
  }
  content: {
    stat-number value: revenue.current, format: currency
    trend-line data: revenue.history
  }
  footer: {
    link "View details", to: /revenue
  }
}
```

**Component composition — components using other components:**
```
-- components/metric-card.naze

use components/card
use components/trend-line

component metric-card(title: text, value: number, format: format-type, history: list) {
  card {
    header: { heading title }
    content: {
      stat-number value: value, format: format
      trend-line data: history
    }
  }
}
```

**Key design decisions:**
- Components are files. One component per `.naze` file. The filename is the component name.
- Props are typed and can have defaults. The compiler checks prop types at compile time.
- Content slots allow composition (putting content inside a component). Optional slots use `?`.
- Components emit events (e.g., `emit toggle-sidebar`) rather than accepting callback props. The parent handles events.
- Accessibility metadata is part of the component definition, not an afterthought.
- A component library is a directory of `.naze` files, distributable as a package.

### C2: Compiler

**Owns:** parsing, type checking, optimization, code generation
**Consumes:** `.naze` source (from C1 spec)
**Produces:** WASM binary, native binaries, source maps, structured error output
**Interface contracts:**
- *Input:* `.naze` files conforming to C1 spec version X
- *Output (WASM):* standard `.wasm` file + Naze metadata section (component tree, accessibility hints, data binding map, layer hints for compositing)
- *Output (native):* platform binary linking against C3 runtime ABI
- *Output (errors):* structured JSON error format (for both humans and C7 AI layer)
- *Output (source maps):* mapping from binary offsets to source locations (for C9 tooling)
- *Output (HTML/SSR):* server-rendered HTML representation of the app for SSR/SSG. Uses C4 Layout Engine to compute positions, serializes to HTML/CSS instead of draw commands.
- *Output (server binary):* compiled server functions + SSR renderer as WASM (for edge/serverless) or native binary (for containers)
- *Client↔server stubs:* auto-generated RPC stubs for `server` functions. Client stub (HTTP POST, JSON-serialized args) and server handler share the same type signature.
**WASM module merging pipeline:** When `.naze` source imports external `.wasm` modules (Tier 2 computation), the compiler resolves imports → locates `.wasm` files (from packages or local paths) → merges all modules into a single binary via `wasm-merge` → runs `wasm-opt` for tree-shaking, dead code elimination, and optimization (LTO, opt-level tuning). After merging, cross-module calls become intra-module calls — eligible for inlining and further optimization. Lazy loading opt-in: large libraries can be split into separate `.wasm` chunks loaded on demand.
**Pure function optimization:** Tier 1 pure functions (no side effects) are candidates for inlining, constant folding, and common subexpression elimination. Pipeline operator chains (`|`) are fused — `filter` → `map` → `take` compiles to a single pass over the data, not three separate iterations.
**Pluggable backends:** WASM (Phase 1), ARM/AArch64 (Phase 2), x86-64 (Phase 2)
**Build modes:**
- `nazec build` — SSG: pre-render all routes to HTML + client WASM
- `nazec build --client-only` — SPA: HTML shell + client WASM only
- `nazec build --server` — fullstack: server binary (WASM or native) + client WASM bundle
**Versioning:** independent of language spec. Compiler version tracks which spec versions it supports.

### C3: Runtime

**Owns:** app lifecycle, state management, event dispatch, navigation, network/data
**Consumes:** compiled WASM binary (from C2), platform events (from host environment)
**Produces:** layout tree (to C4), render commands (to C5), accessibility tree (to C6)
**Interface contracts:**
- *App binary ABI:* defines how compiled Naze code calls runtime functions (init, update, event handlers)
- *Layout tree format:* structured tree passed to C4 Layout Engine
- *Render command buffer:* draw list passed to C5 Renderer
- *Accessibility tree format:* semantic tree passed to C6 A11y Bridge
- *Platform adapter interface:* abstraction for host environment (browser APIs, native OS APIs)
**Versioning:** ABI version separate from implementation version. ABI breaks = major version.

#### C3 Sub-component: Event System & Input Handling

**Owns:** platform event integration, hit testing, event dispatch, focus management, gesture recognition

**Event flow:**
```
Platform (browser canvas listeners / native OS events)
  │
  ▼
C3 Event Dispatcher
  │
  ├─→ Hit Tester → spatial query on C4 positioned layout tree
  │     "which element's rectangle contains point (x, y)?"
  │
  ├─→ Focus Manager → tracks focusedElement, computes tab order
  │     keyboard events route to focused element
  │
  └─→ Element Handlers → executes `on click`, `on keypress`, etc.
        events bubble up the tree unless stopped
```

**Hit testing:** C4 Layout Engine already computes positioned rectangles for every element. Hit testing is a reverse-order tree walk (topmost layer first) checking point containment. Respects clipping regions and z-index. O(log n) with spatial partitioning (quadtree for dense UIs), O(n) worst case. Fast enough — game engines do this at 60fps with thousands of elements.

**Focus management:** Maintains single `focusedElement` reference. Tab order computed from layout tree in visual order (top-to-bottom, left-to-right). Components render their own focus indicators via theme tokens. When a text field receives focus, C3 notifies C6 to show/position the hidden IME input element.

**Text input integration:** Browser keyboard events arrive via JS interop on the canvas element. For simple keyboard input (ASCII, shortcuts), events dispatch directly to the focused element. For complex input (IME — Chinese, Japanese, Korean, emoji), events flow through C6's hidden input element → C6 forwards to C3 → C3 updates text field state → C5 re-renders.

**Gesture recognition:** Combines low-level events into high-level gestures: tap, double-tap, long-press, swipe, pinch, rotate. Platform-specific (touch events on mobile, mouse events on desktop).

**Clipboard integration:** Ctrl+C/Ctrl+V intercepted by the event system. C3 reads selected text from the focused element's selection state → writes to browser Clipboard API (`navigator.clipboard`) via JS interop. Paste reads from clipboard → inserts at cursor. Right-click triggers a Naze-rendered context menu.

**Find in page:** C3 intercepts Ctrl+F → shows a built-in find overlay (rendered by C5). The overlay searches all text content in the layout tree, highlights matches in-place, and navigates between them. This replaces the browser's native DOM-based Ctrl+F, which can't see canvas content. Same approach as Google Docs canvas mode.

**Interface with C6 (for IME and password managers):**
- When a Naze text field receives focus → C3 calls `C6.showIMEInput(textField, cursorPosition)`
- Input events from hidden element → C6 forwards to C3 → C3 updates text field state
- When text field loses focus → C3 calls `C6.hideIMEInput()`

### C4: Layout Engine

**Owns:** box model, flexbox-like layout, constraint solving, responsive breakpoints, scrolling
**Consumes:** layout tree (from C3)
**Produces:** positioned element tree with computed bounds (to C5 Renderer)
**Interface contract:**
- *Input:* layout tree nodes with style properties (width, height, padding, margin, flex direction, alignment, constraints, breakpoints)
- *Output:* positioned rectangles with absolute coordinates, scroll regions, overflow info
**Resize handling:** Layout is computed for whatever dimensions C3 provides — no pre-determined screen sizes. On window resize: C3 passes new dimensions → C4 recalculates all positions in a single top-down pass → outputs new positioned tree to C5. Responsive breakpoints (`responsive: stack below 768px`) are evaluated during this pass — they're constraints, not separate layout modes.
**Dirty tracking (optimization):** When only a subtree changes (e.g., text input updates one component), only that subtree needs re-layout. Unchanged branches reuse cached positions. Full re-layout only on resize or structural changes.
**Versioning:** independent. Layout algorithm changes that don't alter the input/output format are minor versions.

### C4a: Text Engine

**Owns:** text shaping, font loading, line breaking, bidi, glyph rasterization
**Consumes:** text runs with style (font, size, weight, language, direction) from C4
**Produces:** shaped glyph runs with positions (to C5 Renderer), line metrics (to C4 Layout)
**Interface contract:**
- *Input:* text content + style parameters + available width
- *Output:* glyph IDs, positions, line breaks, measured dimensions
**Built on:** HarfBuzz (shaping), FreeType (rasterization), ICU (bidi, line break, locale)
**Versioning:** independent. Can upgrade HarfBuzz/FreeType/ICU without touching other components.

### C5: Renderer

**Owns:** GPU rendering, compositing, animation frame scheduling
**Consumes:** positioned element tree (from C4), glyph runs (from C4a), render commands (from C3)
**Produces:** pixels on screen
**Interface contract:**
- *Input:* draw list — rectangles, rounded rects, images, glyph runs, clip regions, opacity, transforms, blend modes
- *Platform backend interface:* each backend implements the same rendering API
  - `WebGLBackend` — browsers (Phase 1)
  - `WebGPUBackend` — browsers (Phase 1-2)
  - `MetalBackend` — iOS/macOS (Phase 2)
  - `VulkanBackend` — Android/Linux (Phase 2)
  - `DX12Backend` — Windows (Phase 2)
**DPI / device pixel ratio:** Canvas element is sized at CSS pixels; backing buffer renders at `devicePixelRatio` scale (2x, 3x for retina). All draw commands use logical coordinates; the renderer scales to physical pixels. Text and UI render crisp at native resolution. `devicePixelRatio` changes (e.g., dragging window between displays) trigger re-render at new scale.
**GPU animation fast path:** Transform (position, rotation, scale) and opacity changes from the C3 animation scheduler are applied directly as GPU uniform updates — no C4 re-layout needed. The renderer updates the transform matrix or blend parameter and re-draws. This makes transform/opacity animations essentially free (same optimization browsers use for CSS `transform` and `opacity`). Size/padding/margin animations go through C4 re-layout per frame (more expensive, but still fast for typical UIs).
**Layer compositing:** The UI is separated into independent GPU texture layers. Each layer renders to its own offscreen framebuffer. When compositing the final frame, the GPU overlays textures — unchanged layers are not re-rendered. The C2 compiler outputs layer hints in the compiled binary: static analysis of the component tree determines which subtrees should be separate compositing layers (e.g., toolbar, sidebar, main content, modals). This avoids the over/under-compositing problems browsers face with runtime heuristics.
**Sub-components:**
- `LayerCompositor` — manages GPU texture layers, determines which layers need repaint based on dirty state from C3, composites final frame by overlaying layer textures
- `DirtyRectTracker` — computes bounding boxes of changed elements within a dirty layer, clips the repaint region so only changed areas are redrawn
- `TextureCache` — renders expensive components (charts, complex shapes, filtered images) to GPU textures, reuses cached texture until the component's data changes (invalidation driven by C3 dirty tracking)
- `DrawCallBatcher` — groups similar draw commands (same shader, same texture atlas) into single batched GPU calls, reducing CPU→GPU overhead
**Culling:** Frustum culling skips elements entirely outside the visible viewport (important for scrollable content). Occlusion culling skips elements hidden behind opaque elements (e.g., content behind a modal backdrop).
**Versioning:** renderer version independent of backends. Backends version independently of each other.

### C6: Accessibility Bridge

**Owns:** mapping Naze semantic tree to platform accessibility APIs
**Consumes:** accessibility tree (from C3)
**Produces:** platform-specific accessibility output
**Interface contract:**
- *Input:* semantic tree — nodes with role, label, value, state, relationships, keyboard behavior
- *Output (Phase 1):* hidden DOM elements (side DOM) for browser accessibility APIs
- *Output (Phase 2+):* native accessibility APIs (NSAccessibility on macOS, UIAccessibility on iOS, ATK on Linux, UIA on Windows)
- *Output (future):* W3C Accessibility Object Model (AOM) when browser support lands
**Versioning:** independent. Platform adapters version independently.

#### C6 Sub-component: IME Integration (Hidden Input Elements)

**Key insight: the side DOM already exists for accessibility. Reuse it for IME support.**

The C6 Accessibility Bridge creates a hidden HTML tree (the "side DOM") to expose semantic information to screen readers. This same DOM hosts hidden `<input>` and `<textarea>` elements for IME support. One hidden DOM serves two purposes.

**Responsibilities:**
- Create/manage hidden input elements (pooled — don't create/destroy on every focus change)
- Position them at Naze text cursor coordinates (so IME popups appear in the right place)
- Set input `type` attribute to trigger platform-specific keyboards (`email`, `number`, `tel`, `url`)
- Set `autocomplete` attributes for password manager compatibility (`current-password`, `username`, `email`, `one-time-code`)
- Forward `input`, `compositionstart`, `compositionupdate`, `compositionend` events to C3 Runtime

**Hidden input lifecycle:**
1. Naze text field receives focus (via hit test + click, or tab navigation)
2. C3 Runtime calls `C6.showIMEInput(textField, cursorPosition)`
3. C6 shows/repositions hidden input at cursor coordinates, sets type, calls `hiddenInput.focus()`
4. Browser's IME activates on the hidden element
5. User types → hidden input fires events → C6 forwards to C3 → text field updates → C5 re-renders
6. Text field loses focus → C6 hides input element (keeps in DOM for reuse)

**Mobile considerations:**
- Hidden input must have realistic dimensions — mobile keyboards position relative to input bounds
- Set width/height to match the Naze text field's computed dimensions from C4
- Use `<input>` for single-line fields, `<textarea>` for multi-line

**Native platforms (Phase 2+):** On iOS, Android, and desktop, use platform IME APIs directly (UITextInput, InputConnection, native IME). No hidden input needed. The C6 side DOM is browser-specific; native platforms have their own accessibility APIs.

### C7: AI Integration Layer

**Owns:** LLM-to-Naze pipeline — prompt engineering, constrained generation, validation loop
**Consumes:** C1 language spec (for grammar constraints), C2 compiler error output (for feedback loops)
**Produces:** valid `.naze` source files
**Interface contract:**
- *Input:* natural language intent (from user or agent)
- *Output:* syntactically and semantically valid `.naze` source
- *Feedback loop:* compiler errors (structured JSON from C2) → LLM self-correction → recompile
- *Grammar constraint file:* formal grammar exportable from C1 spec in GBNF format (for llama.cpp) and CFG format (for XGrammar/SGLang). Enables any base model to generate syntactically valid `.naze` via grammar-constrained decoding with zero fine-tuning. Ships with `nazec` CLI.
- *Training data pipeline:* C2 compiler serves as automatic quality filter — generated `.naze` examples are parsed, type-checked, and validated before inclusion in training datasets. Enables execution-verified synthetic data generation for C13 model creation.
**Versioning:** independent. Tracks which C1 spec version it targets.

### C8: HTML Meta-Index Generator

**Owns:** generating the HTML compatibility shell for SEO/crawlers (the simplest form of server rendering — metadata only)
**Consumes:** `.naze` source or compiled metadata (from C2)
**Produces:** HTML file with metadata + link to Naze binary
**Note:** C8 is a subset of C14 (Server Renderer). C8 generates metadata-only HTML shells for client-only/SPA mode. C14 generates full content HTML for SSR/SSG. In fullstack builds (`nazec build --server`), C14 subsumes C8's functionality entirely.
**Interface contract:**
- *Input:* Naze app metadata — title, description, content structure, route map
- *Output:* `index.html` with `<title>`, `<meta>`, Open Graph, JSON-LD, `<link rel="alternate" type="application/naze">`
**Versioning:** independent. Trivial component, low velocity.

### C9: Developer Tooling

**Owns:** editor integration, debugging, profiling, playground
**Consumes:** C1 spec (for LSP), C2 source maps (for debugger), C3 runtime state (for inspector)
**Produces:** developer-facing tools
**Sub-components (each independently versioned):**
- **C9a: Language Server (LSP)** — autocomplete, diagnostics, hover, go-to-definition
- **C9b: Inspector** — visual tree, layout bounds, data binding state, accessibility tree
- **C9c: Debugger** — breakpoints, step through, variable inspection (via WASM debug info)
- **C9d: Hot Reload** — file watcher → incremental recompile → runtime hot-swap
- **C9e: Playground** — browser-based editor + live preview (embeds C2 compiler as WASM)
- **C9f: Binary Size Analyzer** — treemap visualization of WASM binary contents

### C10: Package Manager (part of `nazec` CLI)

**Owns:** dependency resolution, fetching, caching, publishing, project scaffolding
**Consumes:** `naze.toml` manifest, package sources (from registries or git or local paths)
**Produces:** resolved dependency tree, cached source files ready for C2 compilation
**Interface contracts:**
- *Input:* `naze.toml` with dependency declarations (name, version constraint, source)
- *Output:* resolved, fetched `.naze` source files in `.nazec/registry/`
- *Cache:* compiled artifacts in `.nazec/cache/` keyed by source hash + compiler version
- *Manifest format:* `naze.toml` schema (TOML-based, versioned)
- *Registry protocol:* HTTP-based API for search, fetch, publish (registry-agnostic — works with git, npm-as-transport, or dedicated Naze registry)
**Versioning:** part of `nazec` CLI binary. Manifest schema versioned independently.

**Key design decisions:**
- Packages are **source-distributed** — `.naze` files + `naze.toml`, not pre-compiled WASM
- **No node_modules** — dependencies cached in `.nazec/registry/`, compiled to `.nazec/cache/`
- **Build cache** — unchanged dependencies are compiled once and cached. Cache key: source hash + compiler version + target platform
- **Lock file** — `naze.lock` records exact resolved versions for reproducible builds
- **Registry-agnostic** — `nazec` resolves from git URLs, local paths, or HTTP registries. The registry is a pluggable backend.

**CLI commands (owned by C10):**
```
nazec new my-app          # scaffold project with naze.toml + app.naze
nazec add @org/lib        # add dependency to naze.toml, fetch source
nazec remove @org/lib     # remove dependency
nazec update              # update dependencies to latest matching versions
nazec publish             # publish package to registry
nazec search <query>      # search registry for packages
```

**CLI commands (owned by C2 but orchestrated through `nazec`):**
```
nazec build               # resolve deps → compile → emit .wasm + index.html
nazec dev                 # build + dev server + hot reload + inspector
nazec check               # type-check and validate without emitting
nazec test                # run component tests
nazec size                # binary size analysis
```

### C11: Testing Framework

**Owns:** test runner, assertion engine, headless rendering for tests, flow simulation
**Consumes:** `.test.naze` files (same C1 syntax), C3 runtime (for rendering components under test), C5 renderer (headless mode)
**Produces:** test results (pass/fail + diagnostics), coverage data
**Interface contracts:**
- *Input:* `.test.naze` files with `test` blocks (component tests) and `flow` blocks (e2e tests)
- *Output:* structured test results (JSON) — pass/fail, assertion details, timing, screenshots (for flow tests)
- *Test renderer:* headless C5 backend that renders without a display, for fast component testing
- *Flow runner:* simulates navigation, clicks, typing across rendered pages
**Sub-components:**
- **C11a: Component Test Runner** — render component with props, simulate events, assert output/state
- **C11b: Flow Test Runner** — multi-page user journey simulation
- **C11c: Assertion Engine** — `assert text "X" is visible`, `assert emitted event-name`, `assert no accessibility violations`
- **C11d: Headless Renderer** — C5 renderer backend for testing (no GPU, software rendering)
**Versioning:** independent. Test syntax is part of C1 spec. Runner versions independently.

### C12: Native AI Prompting Runtime

**Owns:** `prompt` keyword execution, AI provider abstraction, streaming, caching, model routing
**Consumes:** `ai.naze` config (provider definitions), component `prompt` declarations (from C1 spec), environment variables (credentials)
**Produces:** AI response data (text, structured output, streams) consumed by components via reactive data binding (C3)
**Interface contracts:**
- *AI provider config format:* `ai.naze` with named provider definitions (type, model, endpoint, credentials via `env.VAR`)
- *Component prompt syntax:* `prompt X: from ai-provider` with system/user messages, streaming, max-tokens
- *Provider adapter interface:* each provider implements: connect, prompt, stream, cancel
- *Compile-time validation:* prompt templates checked for valid variable references and provider existence
- *Caching:* identical prompts with same inputs return cached responses (configurable TTL)
**Provider adapters (each independently versioned):**
- **OpenAI adapter** — GPT models, vision, structured output
- **Anthropic adapter** — Claude models, streaming
- **Ollama/local adapter** — local models, no network required
- **Generic HTTP adapter** — any OpenAI-compatible API endpoint
**Versioning:** independent. Provider adapters version independently. Prompt syntax is part of C1 spec.

### C13: Embedded AI Authoring Layer (Experimental)

**Owns:** local model that understands the app's source, generates/modifies `.naze` files on behalf of users
**Key principle:** The LLM is the *authoring layer*, not the runtime. It outputs `.naze` source files. Those files compile to WASM and render through the normal deterministic pipeline (C2 → C3 → C4 → C5). The runtime doesn't change. The LLM is a sophisticated code generator that lives inside the app.
**Consumes:** all `.naze` source files in project (indexed), `sources.naze` schema, test files, user natural language input
**Produces:** generated/modified `.naze` source files → fed to C2 compiler → WASM → rendered app
**Interface contracts:**
- *Config:* `ai assistant: local` block in `ai.naze` with `learn-from` glob patterns, `can-modify` / `read-only` permission scoping
- *Index build:* triggered by `nazec build` or `nazec dev` — generates embedding index from `.naze` sources
- *Generation interface:* user natural language → LLM generates `.naze` file → C2 compiles → C3 renders. Same compile pipeline as normal development.
- *Permission model:* `can-modify` lists file globs the LLM is allowed to generate/edit. `read-only` lists files it can reference but not change (e.g., `sources.naze`, `theme.naze`).
- *Validation:* all generated `.naze` output passes through C2 compiler. Syntax errors caught before rendering. Tests (C11) can gate changes.
- *Update protocol:* incremental re-indexing on file change (dev mode), full rebuild on `nazec build`
- *Version control:* auto-commit each LLM modification for rollback capability
**The small model thesis:**
Naze's constrained, single-language design means a fine-tuned small model (3-7B parameters) could match or outperform general-purpose large models (70B+) at `.naze` generation. Why:
- One language, not 50 — only `.naze` syntax, no CSS/JS/framework knowledge needed
- Constrained grammar — one way to express layout, data binding, events
- Declarative, not imperative — smaller output space
- Predictable patterns — components, slots, data bindings follow consistent structures
This makes local-first AI the primary mode, not a compromise.

**Ecosystem tiers (provider-agnostic):**
1. *Existing AI dev tools* — Claude Code, Cursor, Copilot, etc. work with `.naze` files today, no special integration. This is the baseline.
2. *Local Naze-specialized models (the sweet spot)* — fine-tuned 3-7B models via Ollama/llama.cpp. CPU-only, no cloud, no cost. Naze's constrained grammar makes small models viable.
3. *Cloud AI services* — third-party companies offer Naze-optimized models for complex/novel generation. Business opportunity for them, not for Naze.

**Business model:** Naze is fully free and open-source. AI services are ecosystem opportunities for third parties. Naze provides the provider-agnostic plumbing (`ai.naze` config). Naze grows the pie; others build businesses on slices of it.

**Sub-components:**
- **C13a: Source Indexer** — parses `.naze` files into structured chunks (components, props, data bindings, test expectations), generates embeddings, maintains incremental index
- **C13b: Local Inference Engine** — small fine-tuned model runtime (ONNX, llama.cpp, or similar) for CPU-only inference. Positioned as the *primary* mode, not a fallback. Naze's constrained grammar makes 3-7B models viable for most tasks. Cloud routing available for complex generation.
- **C13c: RAG Pipeline** — retrieval-augmented generation combining embedding search with model for app-aware code generation
- **C13d: Permission Enforcer** — validates generated `.naze` output against `can-modify` / `read-only` scoping before writing to disk
**Model creation pipeline (research-backed, Feb 2026):**

The Naze model can be bootstrapped from zero using a two-layer approach: grammar-constrained decoding (GCD) for guaranteed syntax + QLoRA fine-tuning for semantic quality.

*Phase A — Grammar + GCD baseline (Week 1):*
- [ ] Export C1 grammar as GBNF format (for llama.cpp) and CFG (for XGrammar)
- [ ] Test GCD with Qwen2.5-Coder-7B base model → any output is syntactically valid Naze
- [ ] Hand-write 100-200 seed `.naze` examples covering full grammar range
- [ ] Use frontier model (Claude/GPT-4) with Self-Instruct to expand seeds to 5,000 examples
- [ ] Filter all generated examples through C2 compiler (execution-verified generation)

*Phase B — First fine-tune (Week 2):*
- [ ] QLoRA fine-tune Qwen2.5-Coder-3B or 7B on the 5K filtered dataset
- [ ] Hardware: single RTX 4090 (QLoRA, 24GB VRAM) or cloud A100. Training: 2-8 hours.
- [ ] Evaluate: parse rate, semantic correctness, comparison vs GCD-only baseline
- [ ] Use fine-tuned model to generate more examples → filter with C2 → expand to 10K-20K
- [ ] Cost estimate: $5-50 GPU, $50-200 API for data generation

*Phase C — Self-improvement iteration (Week 3):*
- [ ] Fine-tune on 20K dataset with GCD as safety layer
- [ ] Evaluate, debug failure modes, add targeted examples for weak areas
- [ ] Optional: UICoder-style visual evaluation loop (render generated `.naze` → vision model scores visual output → filter → retrain)
- [ ] Target: 80-90%+ semantic correctness for common patterns

*Phase D — Distribution (ongoing):*
- [ ] Publish fine-tuned model on HuggingFace / Ollama registry
- [ ] Ship GBNF grammar file with `nazec` CLI (enables GCD for any base model)
- [ ] `nazec` integrates with Ollama for local model management

**Key precedent — Apple UICoder (VL/HCC 2024):** Fine-tuned a model on SwiftUI from near-zero Swift training data. Self-improvement loop (generate → compile → GPT-4V evaluate → filter → retrain) took compilation rate from 3% to 82%, matching GPT-4. Researchers confirmed this generalizes to Flutter/Dart and React Native — directly applicable to Naze.

**Total estimated cost:** $100-300, 2-3 weeks, one engineer. C2 compiler is the secret weapon — provides automatic quality filtering for training data at every stage.

**Status:** Research phase. Requires C1 grammar spec and C2 compiler to exist first (grammar for GCD, compiler for training data filtering). Can begin as soon as Phase 1 compiler parses `.naze` files. Not blocked on Phase 2+ features.
**Open questions:**
- Compile cycle latency: generate + compile + render needs to feel interactive (<3-5 seconds)
- Hallucination guardrails: compiler catches syntax errors, but semantic correctness (wrong data binding, broken layout) is harder. UICoder-style visual evaluation may help.
- Dev-time tool (helps developers) vs. runtime feature (helps end-users customize the app) vs. both
- Version control integration for LLM-generated changes (auto-commit, diff, rollback)

---

### C14: Server Renderer & Runtime

**Owns:** server-side HTML rendering from Naze source, server function execution, HTTP request handling, RPC protocol
**Key principle:** separate from C5 (GPU Renderer). C5 renders pixels via GPU. C14 renders HTML via C4 Layout Engine — no GPU required.
**Consumes:** `.naze` source (from C1), layout tree (from C4), `sources.naze` config (from C3 Data Source layer), `server` function definitions (from C2 compiler output)
**Produces:** HTML response (for SSR/SSG), RPC responses (for server functions), static assets (serves client `.wasm`, images, fonts)
**Interface contracts:**
- *SSR/SSG output:* semantic HTML generated from Naze layout tree. Uses C4 Layout Engine to compute positions, serializes to HTML/CSS instead of GPU draw commands. Includes headings, text, images, links, structured data — enough for instant first paint and SEO indexing.
- *Server function RPC:* HTTP POST endpoints with JSON-serialized arguments. Auto-generated by C2 compiler from `server` function definitions. Type-safe — client stub and server handler share the same type signature.
- *Edge function support:* `server(edge)` functions compiled to WASM for edge runtimes (Cloudflare Workers, Fermyon Spin, Fastly Compute).
- *HTTP listener:* embedded HTTP server (Hyper/Axum-based for native binary, WASI HTTP handler for WASM target). No external web server dependency.
- *Router:* maps URL paths to routes (SSR pages) and RPC endpoints (server functions).
- *Data Source Manager:* manages database connection pools, API clients from `sources.naze`. Shared between SSR rendering and server function execution.
- *Static asset server:* serves client WASM binary, images, fonts from `dist/` directory.
**Two compilation targets:**
- **WASM binary** (for edge/serverless) — runs on WASI-compatible runtimes: Wasmtime (Fermyon Spin, Fastly), V8 isolates (Cloudflare Workers), WasmEdge (AWS Lambda)
- **Native binary** (for containers/VPS) — x86-64 or ARM via Cranelift/LLVM. Standalone executable (~5-20MB). Docker-deployable with `FROM scratch`.
**Sub-components:**
- **C14a: Server Renderer** — renders Naze layout tree to HTML/CSS. Uses C4 for layout computation. No GPU, no C5 dependency. Handles SSG (build-time) and SSR (request-time) modes.
- **C14b: Server Function Runtime** — executes `server` functions, manages RPC protocol, handles request/response serialization.
- **C14c: HTTP Server** — embedded listener, router, static asset serving. Hyper/Axum for native, WASI HTTP for WASM.
- **C14d: Edge Runtime** — optimized subset for edge deployment. `server(edge)` functions with sub-millisecond cold starts.
**Versioning:** independent. Server renderer version tracks C4 Layout Engine compatibility. Server function ABI tracks C2 compiler output format.

### C1 Extension: Styling & Theming

Part of the C1 Language Spec. Documented separately for clarity.

**Owns:** `theme.naze` format, design tokens, token references in components, theme inheritance
**Interface contract:**
- *Theme file format:* `theme.naze` with named sections (colors, fonts, spacing, radii, shadows)
- *Token reference syntax:* `theme.colors.primary`, `theme.spacing.md`, etc.
- *Theme inheritance:* `extends` keyword to override specific tokens while inheriting the rest
- *Compiler behavior:* resolves token references at compile time. Warns on raw values when token equivalents exist.

### C3 Extension: Data Source Layer

Part of the C3 Runtime. Documented separately for clarity.

**Owns:** data source abstraction, named source registry, connection management, caching, reactive data updates
**Interface contracts:**
- *Source definition format:* `sources.naze` with typed source declarations (rest, graphql, postgres, websocket, static)
- *Component data binding:* `data X: from source-name "query/path"` syntax
- *Credential handling:* `env.VAR_NAME` references resolved from environment at runtime, never embedded in source
- *Source adapter interface:* each source type implements: connect, query, subscribe, disconnect
**Source adapters (each independently versioned):**
- **REST adapter** — HTTP client with auth, retry, caching
- **GraphQL adapter** — query/mutation/subscription support
- **Database adapters** — PostgreSQL, MySQL, SQLite (for server/native contexts)
- **WebSocket adapter** — persistent connections, reconnection, channel subscriptions
- **Static adapter** — local file loading (JSON, TOML, CSV)

---

## Phase Breakdown

### Phase 1: Proof of Life

**Goal:** a "hello world" Naze app running in a browser via WASM + Canvas.

**Components active:** C1 (minimal), C2 (WASM backend only), C3 (minimal), C4 (minimal), C5 (WebGL backend only), C10 (minimal CLI)

#### C1: Language Spec v0.1
- [ ] Define minimal grammar — enough for: static text, rectangles, colors, basic layout (stack, row, grid)
- [ ] Formal grammar in PEG notation
- [ ] Brace-delimited blocks (`{}`), `key: value` properties, `--` comments
- [ ] Spatial primitives: `row`, `column`, `grid`, `stack`, `spacer`
- [ ] Minimal component support: `component` definition with typed props, `use` for import
- [ ] One built-in layout template: `app-shell(main)` (single slot)
- [ ] 10-15 example `.naze` files covering: hello world, basic layout, simple component reuse
- [ ] No data binding, no interaction, no animation, no responsive breakpoints yet
- [ ] No content slots yet (just props)

**Effort:** Medium. Language design + grammar formalization + example files. The syntax decisions (brace-delimited blocks, keyword choices, property syntax) have lasting consequences and need care.

#### C2: Compiler v0.1
- [ ] PEG parser for C1 v0.1 grammar
- [ ] AST representation
- [ ] Type checker (basic — static types on primitives)
- [ ] Component resolution — resolve `use` imports, check prop types
- [ ] WASM code emitter — generates WASM that calls C3 runtime functions
- [ ] Structured error output (JSON format)
- [ ] Source map generation

**Build pipeline (end-to-end):**
```
nazec build app.naze
  1. Read naze.toml → resolve dependencies
  2. Fetch/cache dependency sources (C10)
  3. Parse all .naze files (app + deps) → ASTs
  4. Type-check across component boundaries
  5. Tree-shake unused components
  6. Emit WASM binary (app.wasm)
  7. Generate meta-index HTML (index.html)
  8. Write to dist/
```

**Effort:** Medium. Parser + type checker + WASM emission. Can use existing parser generator libraries. Cranelift or raw WASM bytecode emission.

**Language choice for compiler itself:** Rust (for WASM ecosystem alignment, performance, and eventual self-hosting potential).

#### C10: Package Manager v0.1
- [ ] `nazec new` — scaffold project directory with `naze.toml` + `app.naze`
- [ ] `naze.toml` parser (TOML format)
- [ ] Local path dependencies (`source = "../shared"`)
- [ ] `nazec build` orchestration — read manifest, resolve deps, invoke compiler
- [ ] `.nazec/cache/` directory for compiled artifacts
- [ ] No registry support yet — local paths and git URLs only

**Effort:** Small-Medium. TOML parsing is trivial (Rust `toml` crate). The orchestration is the main work.

**`naze.toml` minimal schema (Phase 1):**
```toml
[app]
name = "hello"
version = "0.1.0"

[build]
entry = "app.naze"
output = "dist/"
```

#### C3: Runtime v0.1
- [ ] WASM module loader (runs in browser via JS bootstrap)
- [ ] App lifecycle: init → first render
- [ ] Builds layout tree from compiled binary's declarations
- [ ] Passes layout tree to C4
- [ ] Passes render commands to C5
- [ ] No state management, no events, no navigation, no networking

**Effort:** Medium. The bootstrap JS is minimal. Runtime is a Rust library compiled to WASM.

#### C4: Layout Engine v0.1
- [ ] Stack layout (vertical/horizontal)
- [ ] Fixed widths/heights
- [ ] Padding, margin
- [ ] No flex, no responsive, no scrolling

**Effort:** Small. A simple box-model layout solver. Can reference Yoga (Facebook's flexbox implementation) or Taffy (Rust flexbox library) for design, but implement minimal subset.

#### C5: Renderer v0.1 (WebGL backend only)
- [ ] Canvas element creation
- [ ] Filled rectangles with color
- [ ] Rounded rectangles
- [ ] Text rendering (basic — single font, single size, no shaping)
- [ ] No images, no gradients, no shadows, no clipping

**Effort:** Medium. WebGL setup, shader programs, basic draw calls. Text via canvas 2D fallback initially (real text engine comes in Phase 2).

#### Phase 1 deliverable
- `nazec new hello && cd hello && nazec build` produces `dist/app.wasm` + `dist/index.html`
- Open `dist/index.html` in Chrome → see colored rectangles and text on a canvas
- Project structure: `naze.toml` + `app.naze` + `components/*.naze` + `dist/`
- Total WASM binary size target: < 100KB

#### Phase 1 milestones
1. `nazec new` scaffolds a project with `naze.toml` + `app.naze`
2. Parser parses `.naze` files into ASTs
3. Component imports (`use components/box`) resolve correctly
4. Compiler emits a `.wasm` file from the AST
5. Runtime loads the `.wasm` in a browser
6. Layout engine positions boxes
7. Renderer draws colored rectangles on a canvas
8. Text appears on screen
9. End-to-end: edit `.naze` source → `nazec build` → see change in browser

---

### Phase 2: Real Apps

**Goal:** build a non-trivial app (dashboard, form, content page) with interaction, data, and accessibility.

**Components active:** C1 (expanded), C2 (expanded), C3 (expanded), C4 (full), C4a (new), C5 (expanded), C6 (new), C14 (new)

#### C1: Language Spec v0.2 → v0.5
- [ ] **Content slots** — `slot header`, `slot content`, `slot footer?` (optional). Enable composition (putting content inside components)
- [ ] **Component events** — `emit toggle-sidebar`, parent handles with `on toggle-sidebar: ...`
- [ ] **Data binding** — reactive connections to data sources (`data: fetch(...)`, `bind: state.count`)
- [ ] **Interaction** — `on click`, `on hover`, `on keypress`, `on drag`
- [ ] **Conditional rendering** — `if`, `each` (iteration), `match`
- [ ] **Full layout template system** — `template` keyword for defining custom slot templates using spatial primitives
- [ ] **Built-in layout template library** — `app-shell`, `dashboard`, `sidebar-layout`, `split-view`, `centered`
- [ ] **Responsive breakpoints** — `responsive: stack below 768px`, `collapsible: below 1200px`
- [ ] **Animation declarations** — four types:
  - [ ] Property animation: `animate opacity from 0 to 1 over 300ms`
  - [ ] Transition: `transition background: 150ms ease` (auto-animate on value change)
  - [ ] Spring physics: `animate position-y: target, physics: spring(stiffness: 300, damping: 20)`
  - [ ] Keyframe: `animate scale: [1, 1.2, 0.95, 1] over 400ms`
  - [ ] Easing curves: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`, `cubic-bezier(x1, y1, x2, y2)`, `spring(stiffness, damping)`
- [ ] **Accessibility attributes** — `role: button`, `label: "Submit form"`, `keyboard: enter → activate`
- [ ] **Style system** — colors, fonts, spacing as named tokens / themes
- [ ] **Document view renderer** — tool that renders `.naze` source to a formatted readable summary (the "document view" of the app's structure, for human review)
- [ ] **Package system** — component directories distributable as versioned packages
- [ ] **AI prompt primitive** — `prompt` keyword for declaring AI interactions in components. `ai.naze` config for provider abstraction. Compile-time validation of prompt templates and provider references.
- [ ] **Server functions** — `server` keyword for functions that run on the server, never ship to client WASM. `server(edge)` variant for edge deployment. Compiler auto-generates client RPC stubs.
- [ ] **Pipeline operators** — `|` chains for `filter`, `sort-by`, `map`, `take`, `sum`, `reduce`, `group-by`, `flatten`, `distinct`, `zip`
- [ ] **Pure functions** — `function name(args) -> type` with expression body, no side effects, no mutation
- [ ] **Pattern matching** — `match` with exhaustive checking, wildcard `_`, destructuring
- [ ] **List comprehensions** — `[expr for item in list if condition]`
- [ ] **Local `let` bindings** — `let name = expr` within component scope, immutable
- [ ] **WASM module imports** — `import name from "package"` or `import name from "./path"`. Resolve `.wasm` modules from package registry or local filesystem. Type-check imported function signatures against Naze's type system.

**Effort:** Large. This is the bulk of language design. Multiple iterations, lots of user testing (with both AI and human authors). The slot/composition model and layout template system are the most design-intensive pieces.

#### C2: Compiler v0.2 → v0.5
- [ ] Full type checker for expanded language features
- [ ] **Component compilation** — resolve imports, check prop types across component boundaries, compile slot contents
- [ ] **Layout template resolution** — expand slot templates into spatial primitive trees at compile time
- [ ] Data binding compilation — reactive dependency tracking in emitted code
- [ ] Event system compilation — wire `emit` in child to `on` handler in parent
- [ ] Tree shaking of unused components
- [ ] Dead code elimination
- [ ] Binary size optimization passes
- [ ] Incremental compilation (only recompile changed components)
- [ ] Watch mode (`nazec watch`) for development
- [ ] Native ARM backend (Phase 2b) via Cranelift or LLVM
- [ ] **Compile-time accessibility checks** — warn on missing `role`, `label`, `keyboard` on interactive elements
- [ ] **WASM module merging** — resolve `import` declarations, locate `.wasm` files, merge via `wasm-merge`, optimize via `wasm-opt` (tree-shake, DCE, LTO)
- [ ] **Pipeline fusion** — compile `filter` → `map` → `take` chains into single-pass iteration
- [ ] **Pure function inlining** — inline small pure functions at call sites, constant fold where possible
- [ ] **Lazy WASM splitting** — opt-in code splitting for large imported `.wasm` modules (load on demand)

**Effort:** Large. Reactive data binding compilation and cross-component type checking are the hardest sub-problems. WASM module merging leverages existing Binaryen tooling (wasm-merge, wasm-opt) — integration effort rather than net-new. Incremental compilation is important for developer experience.

#### C3: Runtime v0.2 → v0.5
- [ ] State management — reactive state with fine-grained change tracking
- [ ] Event system — browser events → Naze event primitives → dispatch to handlers
- [ ] **Hit testing** — tree walk on C4 positioned layout tree, point-in-rectangle containment, z-index aware, clipping region aware
- [ ] **Focus management** — `focusedElement` tracking, tab order computation from layout tree, focus indicator rendering via theme tokens
- [ ] **Text input integration** — keyboard event dispatch to focused element, cursor/selection state per text field, IME event forwarding from C6
- [ ] **Gesture recognition** — tap, double-tap, long-press, swipe, pinch from low-level events
- [ ] **Clipboard integration** — Ctrl+C/V via browser Clipboard API, right-click context menu
- [ ] **Find in page** — built-in Ctrl+F overlay, search all text in layout tree, highlight matches
- [ ] Navigation / routing — URL-based navigation, deep linking, history API
- [ ] Network / data fetch — async data loading, caching, error states
- [ ] **Animation scheduler** — 60fps `requestAnimationFrame` loop, property interpolation, easing curves, spring physics
  - [ ] Register/deregister active animations on property value changes
  - [ ] Per-frame interpolation: elapsed time + easing function → current value
  - [ ] GPU-only fast path: transform (position, rotation, scale) and opacity changes skip C4 re-layout, update GPU uniforms directly
  - [ ] Layout-triggering path: size/padding/margin changes go through C4 re-layout per frame
  - [ ] Spring physics solver (stiffness, damping parameters)
  - [ ] Keyframe animations (multi-step value sequences)
- [ ] Platform adapter: browser (Phase 2a), iOS/Android (Phase 2b)

**Effort:** Large. State management and event systems are the core of any UI runtime. This is where most of the runtime complexity lives.

#### C4: Layout Engine v0.2 → v0.5
- [ ] Full flexbox-like layout (align, justify, flex-grow, flex-shrink, wrap)
- [ ] Constraint-based sizing (min/max width/height, aspect ratio)
- [ ] Responsive breakpoints (layout recalculates when container size changes)
- [ ] Scroll containers (overflow scrolling, virtualized lists)
- [ ] Text flow integration (inline text alongside block elements)
- [ ] Absolute/relative positioning
- [ ] Z-index / layer ordering

**Effort:** Medium-Large. Could build on **Taffy** (Rust flexbox library, used by Dioxus and others) to avoid starting from zero. Scroll virtualization is its own sub-problem.

#### C4a: Text Engine v0.1 → v0.3
- [ ] Integrate HarfBuzz (text shaping — ligatures, kerning, complex scripts)
- [ ] Integrate FreeType (glyph rasterization)
- [ ] Integrate ICU (bidirectional text, line breaking, locale-aware behavior)
- [ ] Font loading (web fonts, system fonts, font fallback chains)
- [ ] Line breaking and paragraph layout
- [ ] Text selection (hit testing, range selection, cursor positioning)
- [ ] Compile all three C libraries to WASM (they're C/C++, this is known to work)

**Effort:** Medium. Integration effort — the libraries exist and work. The challenge is compiling them to WASM with acceptable binary size and wiring them into the layout/render pipeline. HarfBuzz + FreeType + ICU add ~1-2MB to WASM binary; may need subsetting.

#### C5: Renderer v0.2 → v0.5
- [ ] Image rendering (PNG, JPEG, WebP decode + GPU texture upload)
- [ ] Gradients (linear, radial)
- [ ] Shadows (box shadow, drop shadow)
- [ ] Clipping and masking
- [ ] Border rendering (solid, rounded)
- [ ] Opacity and blending
- [ ] Glyph atlas management (cache rendered glyphs as GPU textures)
- [ ] **DPI / device pixel ratio** — canvas backing buffer at `devicePixelRatio` scale, logical-to-physical coordinate mapping, re-render on DPI change (multi-monitor)
- [ ] **Viewport / resize handling** — `window.resize` listener, re-layout + re-render cycle, orientation change on mobile
- [ ] WebGPU backend (alongside WebGL)
- [ ] Metal backend (Phase 2b — iOS/macOS)
- [ ] Vulkan backend (Phase 2b — Android/Linux)
- [ ] **Layer compositing** — GPU texture layers per subtree, automatic layer assignment from compiler hints, composite final frame by overlaying layer textures
- [ ] **Dirty rectangle tracking** — compute bounding boxes of changed elements within a layer, clip repaint to changed regions only
- [ ] **Texture caching** — cache expensive components (charts, complex shapes) as GPU textures, invalidate on data change
- [ ] **Draw call batching** — group same-shader/same-texture draw commands into single batched GPU calls
- [ ] **Frustum culling** — skip elements outside visible viewport (scrollable content optimization)
- [ ] **Occlusion culling** — skip elements hidden behind opaque elements (modal backdrops, stacked views)

**Effort:** Large. Each rendering feature is a shader program + integration. Backend abstraction layer (trait/interface that all backends implement) must be designed carefully. Could build on **wgpu** (Rust GPU abstraction) which already handles WebGL/WebGPU/Metal/Vulkan/DX12.

#### C6: Accessibility Bridge v0.1 → v0.2
- [ ] Consume accessibility tree from C3 Runtime
- [ ] Generate hidden side DOM (browser platform)
  - [ ] Map Naze roles to ARIA roles
  - [ ] Map Naze labels to aria-label / aria-labelledby
  - [ ] Map Naze states to aria-expanded, aria-checked, etc.
  - [ ] Keyboard focus management (tab order, focus trapping)
  - [ ] Live region announcements (for dynamic content updates)
- [ ] **IME integration** — hidden input element pool in the side DOM
  - [ ] Create/manage pooled `<input>` and `<textarea>` elements
  - [ ] Position at Naze text cursor coordinates (for IME popup placement)
  - [ ] Set input `type` for platform-specific keyboards (email, number, tel, url)
  - [ ] Forward `input`, `compositionstart/update/end` events to C3 Runtime
  - [ ] Handle mobile keyboard positioning (match text field dimensions)
  - [ ] Set `autocomplete` attributes for password manager compatibility (`current-password`, `username`, `email`)
- [ ] Synchronize side DOM with Naze tree on every state change
- [ ] Test with screen readers (NVDA on Windows, VoiceOver on macOS, TalkBack on Android)

**Effort:** Medium-Large. The side DOM approach is well-understood (Flutter does it), but getting it right across screen readers is detail-intensive work. Keyboard navigation and focus management are particularly tricky.

#### C1 Extension: Theming v0.1 → v0.3
- [ ] `theme.naze` file format — named sections: colors, fonts, spacing, radii, shadows
- [ ] Token reference syntax — `theme.colors.primary`, `theme.spacing.md` resolvable in any component
- [ ] Theme inheritance — `extends` keyword for dark mode / brand variants
- [ ] Compiler resolves tokens at compile time, warns on raw values where tokens exist
- [ ] Built-in default theme — sensible defaults so apps look decent without a custom `theme.naze`
- [ ] Runtime theme switching — swap theme without recompile (for light/dark toggle)

**Effort:** Medium. Token resolution at compile time is straightforward. Runtime switching requires the renderer to support re-theming without full re-render.

#### C3 Extension: Data Sources v0.1 → v0.3
- [ ] `sources.naze` file format — typed source declarations
- [ ] `env.VAR_NAME` reference resolution from environment variables
- [ ] REST adapter — GET/POST/PUT/DELETE, auth headers, retry, response caching
- [ ] `data X: from source-name "path"` syntax in components
- [ ] Reactive data updates — component re-renders when data changes
- [ ] Loading/error states — `data.loading`, `data.error` available in components
- [ ] GraphQL adapter (Phase 2b)
- [ ] WebSocket adapter (Phase 2b) — persistent connection, real-time updates
- [ ] Database adapters (Phase 2b) — PostgreSQL, SQLite (for native/server contexts)

**Effort:** Medium-Large. REST adapter is straightforward. Reactive data binding integration with C3 state management is the harder part. Database adapters require server-side rendering or native context.

#### C11: Testing Framework v0.1 → v0.3
- [ ] `.test.naze` file format — `test` blocks for component tests, `flow` blocks for e2e
- [ ] C11a: Component test runner — render with props, simulate events, assert output
- [ ] C11c: Assertion engine — `assert text "X" is visible`, `assert emitted event`, `assert not visible`
- [ ] C11d: Headless renderer — software rendering backend for C5 (no GPU required)
- [ ] `nazec test` CLI integration — discover and run all `.test.naze` files
- [ ] Structured test output (JSON) — for CI integration and AI feedback loops
- [ ] C11b: Flow test runner (Phase 2b) — multi-page navigation, full user journey simulation
- [ ] Accessibility assertions — `assert no accessibility violations`
- [ ] Screenshot comparison (Phase 2b) — visual regression testing

**Effort:** Medium-Large. The headless renderer (C11d) is the biggest piece — a software-only C5 backend. The test syntax and runner are comparatively simple.

#### C14: Server Renderer & Runtime v0.1 → v0.3
- [ ] C14a: Server Renderer — render Naze layout tree to HTML/CSS using C4 (no GPU)
- [ ] SSG mode: pre-render all routes at build time via `nazec build`
- [ ] SSR mode: render on each request via `nazec build --server`
- [ ] C14b: Server Function Runtime — execute `server` functions, handle RPC from client
- [ ] Request/response serialization (JSON)
- [ ] Data source connection management (reuse C3 Data Source adapters)
- [ ] C14c: HTTP Server — embedded Hyper/Axum listener (native), WASI HTTP handler (WASM)
- [ ] URL router (pages + server function RPC endpoints)
- [ ] Static asset serving (client WASM, images, fonts)
- [ ] C14d: Edge Runtime (Phase 2b) — `server(edge)` functions for Cloudflare Workers, Fermyon Spin
- [ ] Two compilation targets: WASM (edge/serverless) and native binary (containers)
- [ ] Loading skeleton generation for client-only mode (from `.naze` layout structure)

**Effort:** Medium-Large. The server renderer (C14a) reuses C4 Layout Engine, which reduces net-new work. The HTTP server (C14c) uses existing Rust ecosystem (Hyper/Axum). Server function compilation (C14b) depends on C2 compiler generating the RPC stubs.

#### Phase 2 deliverables
- Build a real dashboard app: data fetching from REST API, interactive charts, responsive layout, keyboard-navigable
- Build a real form: validation, submission, error states, accessible labels
- Theme system: light and dark themes, runtime switching
- Data layer: components fetch from named sources, credentials in environment
- Test suite: component tests + flow tests for the dashboard app, all passing
- App works in Chrome, Firefox, Safari
- Screen reader announces content correctly
- `nazec test` runs full test suite in CI
- Native iOS/Android builds from same source (Phase 2b)
- WASM binary size target: < 500KB for a medium app (excluding images/fonts)

---

### Phase 3: Developer Experience

**Goal:** make it pleasant to build Naze apps. Tools, AI integration, fast iteration.

**Components active:** C7 (new), C9 (new), all prior components in maintenance mode

#### C7: AI Integration Layer v0.1 → v0.3
- [ ] Export C1 grammar as formal constraint file for LLM structured output
- [ ] Prompt template library — curated few-shot examples for common UI patterns
- [ ] Validation feedback loop:
  - [ ] LLM generates `.naze` source
  - [ ] C2 compiler validates, returns structured errors
  - [ ] Errors fed back to LLM for self-correction
  - [ ] Repeat until valid (with max retry limit)
- [ ] Intent-to-Naze pipeline:
  - [ ] Natural language → Naze source → compile → render preview → user feedback → iterate
- [ ] Fine-tuning dataset:
  - [ ] 500+ Naze examples covering common patterns (nav bars, cards, tables, forms, modals, dashboards, e-commerce layouts)
  - [ ] Paired with natural language descriptions
- [ ] Benchmark: measure token efficiency vs generating equivalent React/HTML
- [ ] Constrained decoding integration (grammar-guided generation for compatible LLM APIs)

**Effort:** Medium-Large. The prompt engineering and validation loop are the core work. Fine-tuning dataset curation is labor-intensive. Constrained decoding depends on LLM provider support.

#### C9a: Language Server (LSP) v0.1
- [ ] Diagnostics (real-time errors/warnings as you type)
- [ ] Autocomplete (component names, property names, enum values)
- [ ] Hover (type information, documentation)
- [ ] Go-to-definition (jump to component source)
- [ ] Rename symbol
- [ ] Format document
- [ ] VS Code extension packaging

**Effort:** Medium. LSP is well-defined. Reuses C2 parser and type checker. The Rust `tower-lsp` crate provides the LSP server framework.

#### C9b: Inspector v0.1
- [ ] Visual overlay showing element bounds (like browser DevTools element inspector)
- [ ] Tree view of Naze component hierarchy
- [ ] Property panel showing computed layout values, styles, data bindings
- [ ] Accessibility tree viewer
- [ ] Click-to-select element in the rendered output
- [ ] Runs as a panel within the Naze app itself (or as a browser extension)

**Effort:** Medium. Needs runtime hooks (C3) to expose the component tree and layout state.

#### C9c: Debugger v0.1
- [ ] WASM debugging via Chrome DevTools Protocol (source-mapped back to `.naze` source)
- [ ] Breakpoints in `.naze` source files
- [ ] State inspection (view current reactive state values)
- [ ] Event log (what events fired, what handlers ran)

**Effort:** Medium. Depends on WASM DWARF debug info support in browsers (Chrome has this). Source maps from C2 are critical.

#### C9d: Hot Reload v0.1
- [ ] File watcher on `.naze` source files
- [ ] Incremental recompile (C2)
- [ ] Runtime hot-swap: replace component tree without losing state
- [ ] Sub-second reload cycle target

**Effort:** Medium. Incremental compilation (C2) is a prerequisite. Hot-swap protocol between C2 and C3 needs careful design.

#### C9e: Playground v0.1
- [ ] Browser-based editor (Monaco or CodeMirror)
- [ ] C2 compiler running as WASM in the browser (compile `.naze` entirely client-side)
- [ ] Live split-pane preview (edit left, render right)
- [ ] Shareable URLs (encode source in URL hash or short-link)
- [ ] Example gallery

**Effort:** Medium. The key enabler is C2 compiler compiled to WASM itself (so it runs in browser). Rust → WASM compilation of the compiler is feasible.

#### C9f: Binary Size Analyzer v0.1
- [ ] Parse WASM binary sections
- [ ] Treemap visualization (which components/functions contribute to size)
- [ ] Comparison mode (before/after optimization)
- [ ] CLI + web UI

**Effort:** Small. WASM binary format is well-documented. Visualization is standard.

#### Phase 3 deliverables
- Developer can: write Naze in VS Code with autocomplete/errors, hot reload, inspect/debug
- AI agent can: take natural language → generate valid Naze → compile → show preview
- Playground live at a public URL for anyone to try
- Measure: AI generates correct Naze on first attempt >80% of the time for common patterns

---

### Phase 4: Ecosystem & Adoption

**Goal:** package registry, community, meta-index for SEO, production deployments.

**Components active:** C8 (new), all prior components in maintenance/evolution mode

#### C8: HTML Meta-Index Generator v0.1
- [ ] Extract metadata from `.naze` source (title, description, content structure, routes)
- [ ] Generate `index.html` with:
  - [ ] `<title>` and `<meta description>`
  - [ ] Open Graph tags (og:title, og:description, og:image)
  - [ ] JSON-LD structured data
  - [ ] `<link rel="alternate" type="application/naze" href="app.naze">`
  - [ ] Optional: basic HTML text content for accessibility/no-JS fallback
- [ ] Route-aware: generates per-route HTML pages for multi-page Naze apps
- [ ] CLI: `nazec meta app.naze → index.html`
- [ ] Integrate into `nazec build` pipeline as automatic step

**Effort:** Small. Template-based HTML generation. The metadata extraction from Naze source is the only interesting part.

#### C10: Package Manager v0.2 → v0.5
- [ ] **Git-based dependencies** — `source = "git:github.com/org/repo"` with tag/branch/commit pinning
- [ ] **Lock file** — `naze.lock` for reproducible builds (records exact resolved versions + source hashes)
- [ ] **Dependency resolution** — semver constraint solving (version = "^1.0" resolves to latest compatible)
- [ ] **`nazec add`** — add dependency to `naze.toml`, fetch source, update lock file
- [ ] **`nazec update`** — update deps to latest matching versions
- [ ] **`nazec publish`** — publish package (source .naze files + naze.toml) to registry
- [ ] **Registry protocol** — HTTP API: search, fetch, publish. Registry-agnostic (can point at git, npm-as-transport, or dedicated registry)
- [ ] **Dedicated Naze registry** (optional, later) — web UI for search/discovery, package pages with docs/examples
- [ ] **Namespace/scoping** — `@org/package` namespacing to prevent name conflicts

**Package format (what gets published):**
```
@org/charts/
  naze.toml              # manifest: name, version, dependencies, description
  line-chart.naze        # source component
  bar-chart.naze         # source component
  pie-chart.naze         # source component
  shared/
    axis.naze            # internal shared component
    legend.naze
```

Source-distributed. The `.naze` files are the package. No compiled artifacts, no minification, no obfuscation.

**Effort:** Medium-Large. Dependency resolution is the hard part (semver solving). Git-based fetching is straightforward. A dedicated registry is a separate infrastructure project that can come later.

#### Documentation & Learning
- [ ] Language reference documentation
- [ ] Tutorial series (build your first Naze app)
- [ ] API reference for C3 Runtime and C4 Layout primitives
- [ ] Migration guides (from React, Flutter, etc.)
- [ ] AI prompt cookbook (how to get best results from LLMs generating Naze)

**Effort:** Medium. Ongoing. Critical for adoption but often underinvested.

#### Phase 4 deliverables
- Naze apps discoverable by Google via HTML meta-index
- Community contributing shared components
- Production deployment guide (hosting, CDN, caching)
- At least 3 non-trivial public Naze apps demonstrating the platform

---

### Phase 5: Dedicated Browser (Optimization)

**Goal:** optional lightweight runtime that runs Naze natively without browser overhead.

#### Dedicated Runtime
- [ ] Standalone WASM runtime (Wasmtime or custom) — no browser engine
- [ ] Native window creation (winit or platform-specific)
- [ ] Direct GPU rendering (no Canvas indirection)
- [ ] Networking stack (reqwest/hyper or platform-native)
- [ ] Embedded HTML fallback (embed a lightweight webview — webview2 on Windows, WKWebView on macOS, etc.)
- [ ] URL bar, navigation, tabs, bookmarks (basic browser chrome)
- [ ] Dual-branch detection: check for `application/naze` content type, route accordingly

**Effort:** Very Large. This is essentially building a browser shell. The rendering/layout/text components (C4, C4a, C5) are reused. The new work is the shell, navigation, HTML fallback, and platform integration.

This phase is only justified once there's significant Naze content to consume.

---

## Component Dependency Graph

Shows which components block which. Components within the same tier can be built in parallel.

```
Tier 0 (no dependencies):
  C1 Language Spec

Tier 1 (depends on C1):
  C2 Compiler
  C10 Package Manager (naze.toml parsing, dependency resolution)

Tier 2 (depends on C2):
  C3 Runtime
  C4 Layout Engine
  C4a Text Engine
  C5 Renderer

Tier 3 (depends on C3 + C5):
  C6 Accessibility Bridge
  C11 Testing Framework (headless renderer depends on C5)
  C9b Inspector
  C9c Debugger

Tier 4 (depends on C1 + C2):
  C7 AI Integration Layer
  C8 Meta-Index Generator
  C9a LSP
  C9d Hot Reload
  C9e Playground
  C9f Binary Size Analyzer

Tier 5 (depends on C2 + C4):
  C14 Server Renderer & Runtime (depends on C2 compiler + C4 layout engine + C3 data sources)

Tier 6 (depends on C3 + C12):
  C12 AI Prompting Runtime (depends on C3 runtime + C1 prompt syntax)
  C13 Embedded Intelligence (experimental — depends on C12 + all source files)
```

Within each tier, components can be developed in parallel by separate teams/contributors.

---

## Effort Summary

| Component | Phase | Build-on or Net-new | Effort | Can parallelize with |
|-----------|-------|---------------------|--------|---------------------|
| C1: Language Spec | 1-2 | Net-new | Medium | Nothing (blocks everything) |
| C2: Compiler | 1-2 | Net-new (uses Cranelift/LLVM) | Large | C4, C5 (after C1 done) |
| C3: Runtime | 1-2 | Net-new | Large | C4, C5 |
| C4: Layout Engine | 1-2 | Build on Taffy | Medium | C4a, C5, C6 |
| C4a: Text Engine | 2 | Build on HarfBuzz/FreeType/ICU | Medium | C4, C5, C6 |
| C5: Renderer | 1-2 | Build on wgpu | Large | C4, C4a, C6 |
| C6: A11y Bridge | 2 | Net-new | Medium-Large | C4a, C5 |
| C7: AI Integration | 3 | Net-new | Medium-Large | C9 (all sub-components) |
| C8: Meta-Index Gen | 4 | Net-new | Small | Everything |
| C9a: LSP | 3 | Net-new (uses tower-lsp) | Medium | C9b-f, C7 |
| C9b: Inspector | 3 | Net-new | Medium | C9a, C9c-f, C7 |
| C9c: Debugger | 3 | Net-new | Medium | C9a-b, C9d-f, C7 |
| C9d: Hot Reload | 3 | Net-new | Medium | C9a-c, C9e-f, C7 |
| C9e: Playground | 3 | Net-new | Medium | C9a-d, C9f, C7 |
| C9f: Size Analyzer | 3 | Net-new | Small | Everything |
| C10: Package Manager | 1-4 | Net-new (uses toml crate) | Medium | C2 (after C1 done) |
| C11: Testing Framework | 2-3 | Net-new | Medium-Large | C7 (after C3+C5 done) |
| C1 ext: Theming | 2 | Net-new (part of C1) | Medium | C3 data, C11 testing |
| C3 ext: Data Sources | 2 | Net-new (part of C3) | Medium-Large | C1 theming, C11 testing |
| C12: AI Prompting Runtime | 2-3 | Net-new | Medium | C7, C11 (after C3 done) |
| C13: Embedded Intelligence | Research | Net-new | Large (research) | Independent (experimental) |
| C14: Server Renderer & Runtime | 2-3 | Net-new (uses Hyper/Axum) | Medium-Large | C11 (after C2+C4 done) |

**Total effort estimate (Phase 1-4):**
- Phase 1 (Proof of Life): ~3-5 components at minimal scope. Smallest viable team: 1-2 people.
- Phase 2 (Real Apps): all core components at full scope. Needs 3-5 people or significant calendar time with fewer.
- Phase 3 (Dev Experience): 7+ sub-components, highly parallelizable. Can distribute across many contributors.
- Phase 4 (Ecosystem): operational infrastructure + community building. Different skillset than Phase 1-3.
- Phase 5 (Dedicated Browser): optional. Only if ecosystem justifies it.

---

## Implementation Language Choices

| Component | Recommended | Rationale |
|-----------|-------------|-----------|
| C1: Language Spec | Markdown + PEG | Spec is a document, not code |
| C2: Compiler | Rust | WASM ecosystem, performance, compiles to WASM itself (for playground) |
| C3: Runtime | Rust → WASM | Must run as WASM in browsers; Rust has best WASM tooling |
| C4: Layout Engine | Rust | Taffy is Rust. Performance-critical. |
| C4a: Text Engine | Rust + C (via FFI) | HarfBuzz/FreeType/ICU are C/C++; Rust bindings exist |
| C5: Renderer | Rust | wgpu is Rust. GPU abstraction layer. |
| C6: A11y Bridge | Rust + JS (for side DOM) | Side DOM requires JS interop in browsers |
| C7: AI Integration | Python + Rust | Python for LLM API integration; Rust for grammar constraint export |
| C8: Meta-Index Gen | Rust | Small, part of compiler toolchain |
| C10: Package Manager | Rust | Part of `nazec` binary. TOML parsing via `toml` crate. |
| C11: Testing Framework | Rust | Test runner + headless renderer. Software rasterizer for C11d. |
| C3 ext: Data Sources | Rust | Source adapters. REST via reqwest/hyper. DB via sqlx. |
| C9a: LSP | Rust | Reuses compiler internals. tower-lsp crate. |
| C9b-f: Dev Tools | Rust + TypeScript | Rust for core; TypeScript for VS Code extension and web UI |
| C12: AI Prompting Runtime | Rust | Provider adapters. Streaming via async. Part of C3 runtime. |
| C13: Embedded Intelligence | Rust + Python | Rust for indexer/runtime; Python for model training/fine-tuning |
| C14: Server Renderer & Runtime | Rust | HTML serializer, Hyper/Axum HTTP server, WASI HTTP handler. Compiles to WASM (edge) or native (containers). |

---

## Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Language design doesn't work for AI generation | Fatal | Medium | Prototype language with LLM testing from week 1. Iterate grammar based on AI success rate. |
| WASM binary size too large (>1MB for simple apps) | High | Medium | Aggressive tree-shaking. Subset text libraries. Stream-load renderer separately from app. |
| Accessibility is inadequate | High | High | Budget significant effort for C6. Test with real screen reader users early. Make a11y attributes compile-time required. |
| WebGPU adoption stalls | Medium | Low | WebGL backend is fallback. WebGPU is nice-to-have, not required. |
| No adoption — "just another framework" | High | Medium | The AI-native angle is the differentiator. If AI can build Naze apps 10x faster than React apps, adoption follows. |
| Text rendering quality doesn't match browsers | Medium | Medium | Use the same libraries browsers use (HarfBuzz, FreeType). Flutter/Makepad prove this works. |
| Existing frameworks (Flutter, Compose) add AI-native features first | High | Medium | Speed. The language design is the moat. Existing frameworks carry legacy design decisions. |
| Embedded app intelligence is impractical at small model sizes | Medium | High | Treat C13 as research. Use RAG/embeddings first (proven), defer local fine-tuning. Fall back to remote AI (C12) if local doesn't work. |
| AI prompt primitive adds runtime complexity and cost | Medium | Medium | Provider abstraction keeps it optional. Apps without `prompt` blocks have zero AI runtime overhead. Caching reduces redundant API calls. |
| Small model thesis doesn't hold — fine-tuned 7B can't match generation quality | Medium | Medium | Cloud fallback always available. Grammar-constrained decoding helps. Iterate on training data. Existing dev tools (Claude Code, Cursor) work regardless. |

---

*This plan is version 0.1. Components and phases will evolve as prototyping reveals what works and what doesn't.*
