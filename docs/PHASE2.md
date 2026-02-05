# Phase 2: Real Apps + Developer Experience

**Goal:** Build non-trivial interactive apps with state, events, and real UI features. VS Code support. Desktop and Android prototypes.

**Architecture shift:** Phase 1's runtime is a one-shot renderer (deserialize → layout → draw). Phase 2 adds a render loop: state changes trigger re-layout and re-draw. The IR format (`naze-ir`) grows to include event handlers, conditionals, and state bindings.

---

## Phase 2a: Dynamic Apps

### M1: State & Reactivity ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Make apps dynamic. This is the foundation for everything else in Phase 2.

- [x] `let` bindings: `let label = "Hello"` (immutable, compile-time)
- [x] `state` keyword: `state count = 0` (mutable, triggers re-render on change)
- [x] Grammar: add `let_stmt` and `state_stmt` rules to pest
- [x] AST: new `Node::Let` and `Node::State` variants
- [x] IR: state variable declarations, initial values
- [x] Compiler: track state variables, emit state info in render tree
- [x] Runtime: render loop — state change → re-layout → re-draw (requestAnimationFrame)
- [x] Runtime: state store with change detection

### M2: Event System & Interaction ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `naze-renderer`

Make elements clickable and interactive.

- [x] `on click: <action>` syntax on any element
- [x] `on hover`, `on keypress` event handlers
- [x] Actions: `set count = count + 1`, `set visible = true`, `navigate "/page"`
- [x] Grammar: `on_handler` rule, action expressions
- [x] Hit testing: canvas click → point-in-rectangle walk on positioned layout tree
- [x] Event dispatch: find target → bubble up tree → run handler
- [x] Cursor changes: pointer cursor on clickable elements
- [x] Focus management: tab order from layout tree, focusedElement tracking
- [x] Tab navigation: Tab/Shift+Tab cycles through focusable elements
- [x] Focus ring: visual indicator for focused elements
- [x] Enter key activates focused elements

### M3: Conditional Rendering ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`

Show/hide UI based on state.

- [x] `if condition { ... }` / `if condition { ... } else { ... }`
- [x] `each item in list { ... }` for iteration
- [x] Grammar: `if_stmt`, `each_stmt` rules
- [x] IR: conditional nodes, list nodes
- [x] Compiler: validate condition expressions reference valid state
- [x] Runtime: evaluate conditions during render, include/exclude subtrees

### M4: Content Slots ✅
**Crates:** `naze-parser`, `naze-compiler`

Enable component composition — putting content inside components.

- [x] `slot` declaration in component bodies (default slot)
- [x] `slot "name"` for named slots
- [x] `fill "name" { ... }` syntax at call sites to fill named slots
- [x] Children without `fill` go to default slot
- [x] Grammar: `slot_stmt`, `fill_stmt` rules
- [x] Compiler: validate slot names match declarations, inline slot content
- [x] Default slot content (fallback when caller doesn't provide)

### M5: Images & Richer Rendering ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-renderer`, `naze-runtime`

- [x] `image` element: `image src: "photo.jpg", width: 200px, height: 150px`
- [x] Runtime: fetch image, decode, draw to canvas (`drawImage`)
- [x] Image caching (don't re-fetch on re-render)
- [x] `opacity` prop on any element
- [x] `border` prop: `border: 2px, border-color: #cccccc`
- [x] Viewport resize: `window.resize` → re-layout → re-draw
- [x] Background color on `row`/`column` (already working)

### M6: Theming ✅
**Crates:** `naze-parser`, `naze-compiler`

Design tokens for consistent styling.

- [x] `theme.naze` file format:
  ```naze
  theme {
    colors {
      primary: #2563eb
      danger: #dc2626
      background: #ffffff
    }
    spacing {
      sm: 8px
      md: 16px
      lg: 24px
    }
  }
  ```
- [x] Token references in components: `color: theme.colors.primary`
- [x] Compiler resolves tokens at compile time (inlines values)
- [x] Built-in default theme
- [ ] Warn on unknown token references

### M7: Improved Layout ✅
**Crates:** `naze-layout`

Make the layout engine production-quality.

- [x] `flex-grow` on children (distributes remaining space proportionally)
- [x] `min-width`, `max-width`, `min-height`, `max-height` constraints
- [x] `align` and `justify` computed: start, center, end, stretch, space-between, space-around, space-evenly
- [x] `wrap` prop on `row` (wrapping flex layout)
- [x] `flex-shrink` on children (shrinks items proportionally when overflow)
- [x] `scroll` container (overflow scrolling with scroll position state) - see M8e
- [x] Percentage-based widths/heights (relative to parent): `width: 50%`

### M8: Navigation & Routing ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Multi-page apps.

- [x] `page "/path" { ... }` blocks (multiple per app)
- [x] `link "text" to: "/path"` element
- [x] Runtime: History API integration, popstate handling
- [x] Compiler: multi-page IR (multiple render trees per app)
- [x] Runtime: page switching without full reload
- [x] `navigate "/path"` action in event handlers

### M8b: Form Inputs
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `naze-renderer`

Standard form elements for interactive apps. Depends on M1 (state) and M2 (events).

- [x] `input` element: `input bind: username, placeholder: "Enter name", type: "text"`
- [x] Input types: `"text"`, `"number"`, `"email"`, `"password"`
- [x] `checkbox` element: `checkbox bind: agreed, label: "I agree"`
- [x] `radio` element: `radio bind: choice, value: "option-a", label: "Option A"`
- [x] `select` / `option` elements for dropdowns
- [x] `on change` event for form elements
- [x] Two-way binding: `bind: stateVar` reads and writes state
- [x] Runtime: text input rendering on canvas (cursor, blinking caret done; selection TODO)
- [x] Runtime: keyboard input handling (typing, backspace, Enter/Escape; arrow keys/clipboard TODO)
- [x] Input validation: `validate: { required: true, min-length: 3, pattern: "..." }`
- [x] Validation states: `{field}_valid`, `{field}_error` for conditional rendering

### M8c: Drag & Drop ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Depends on M2 (events).

- [x] `draggable: true` prop on any element
- [x] `drop-target: true` prop on containers
- [x] Events: `on drag-start`, `on drag-over`, `on drop`
- [x] Drag data: `drag-data: expression` to attach data to drag operations
- [x] Runtime: hit testing extension for drag regions
- [x] Runtime: visual feedback during drag (ghost element, drop zone highlighting)

### M8d: Accessibility ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Depends on M2 (events). Required for production apps.

- [x] `role` prop: `button`, `link`, `navigation`, `main`, `heading`, `list`, `listitem`, etc.
- [x] `label` prop: accessible name (equivalent to `aria-label`)
- [x] `tab-index` prop for keyboard navigation order
- [x] Focus ring rendering on focused elements
- [x] Keyboard-only operation for all interactive elements (Enter to activate, Escape to dismiss)
- [x] Screen reader integration: hidden DOM overlay with ARIA attributes mirroring canvas content
- [ ] Screen reader announcements on state change (live regions equivalent)
- [x] Compiler: warn when interactive elements lack `role` or `label`

### M8e: Scroll & Overflow (partial)
**Crates:** `naze-parser`, `naze-ir`, `naze-layout`, `naze-runtime`

Depends on M1 (state), M2 (events), M7 (improved layout).

- [x] `scroll` container: `scroll height: 400px { ... }` for overflow scrolling
- [x] Scroll position as implicit state (per scroll container)
- [x] Runtime: scrollbar rendering, mouse wheel handling
- [x] Layout: clip children to container bounds
- [x] `on scroll` event
- [x] `scroll-to` action: `scroll-to element-id`
- [ ] Runtime: touch scroll
- [ ] Virtual scrolling for large lists (stretch goal)

---

## Phase 2b: Tooling & Platforms

### M9: VS Code Extension
**New crate:** `naze-lsp`, **new dir:** `editors/vscode/`

Can start in parallel with Phase 2a — only needs existing parser/compiler.

- [ ] TextMate grammar for `.naze` syntax highlighting
- [ ] LSP server (`tower-lsp` crate): real-time diagnostics as you type
- [ ] Autocomplete: element names, prop names, component names, type names
- [ ] Hover: show element/component prop signatures
- [ ] Go-to-definition: jump to component source file
- [ ] VS Code extension wrapper (TypeScript, `vsce` packaging)
- [ ] Extension published to VS Code Marketplace (or `.vsix` for local install)

### M10: Dev Server & Hot Reload
**Crate:** `nazec`

Fast iteration loop. Native hot reload is done; browser hot reload remaining.

- [x] File watcher on `.naze` files (notify crate) — implemented in `nazec run`
- [x] Native hot reload: file change → rebuild → re-render in native window
- [x] Debounced file watching (300ms quiet period, coalesces editor events)
- [ ] `nazec dev` command: embedded HTTP server (hyper or tiny-http)
- [ ] On change: rebuild → notify browser via WebSocket
- [ ] Browser client: WebSocket listener, auto-reload on message
- [ ] Incremental compilation: only re-parse changed files, reuse cached results
- [ ] Console output: "rebuilt in Xms" timing

### M11: Native Desktop Prototype (x86/ARM)
**Crates:** `nazec` (integrated), `naze-native` (standalone)

Proof of concept: same `.naze` source renders in a desktop window. **Core functionality done.**

- [x] Desktop window via `winit`
- [x] Software renderer via `tiny-skia` drawing same primitives as Canvas2D
- [x] Reuses `naze-ir` (deserialize `app_data.bin`) and `naze-layout` (compute positions)
- [x] `nazec run` previews in native window with live reload
- [x] Renders rectangles, text, colors — same visual output as browser
- [ ] `nazec build --target native` produces standalone binary
- [ ] GPU renderer option (`wgpu`) for better performance (stretch)

### M12: Android Prototype
**New dir:** `platforms/android/`

Proof of concept: same `.naze` source renders on Android.

- [ ] Option A: Android app embedding Wasmtime, running existing WASM pipeline
- [ ] Option B: Android app with native renderer (Skia / Canvas) reading `app_data.bin`
- [ ] Same `naze-ir` format, same layout engine
- [ ] Basic Android app shell (Kotlin)
- [ ] Demonstrates cross-platform rendering from single `.naze` source

### M13: Data Fetching ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Depends on M1 (state) and M3 (conditionals).

- [x] `data` keyword: `data users: fetch "https://api.example.com/users"`
- [x] Data states: `users.loading`, `users.error`, `users.data`
- [x] Runtime: async fetch, JSON parse, populate state
- [x] Reactive: components re-render when data arrives
- [x] `if users.loading { text "Loading..." }` pattern
- [x] Error handling: `if users.error { text users.error }`

### M14: Animation ✅
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Depends on M1 (state) and M2 (events).

- [x] `transition` prop: `transition: "color 150ms ease"` (string format)
- [x] Easing functions: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`
- [x] Runtime: animation scheduler on requestAnimationFrame
- [x] Property interpolation: color, number, opacity
- [ ] `animate` blocks for explicit keyframe animations (stretch goal)

---

## Build Order

```
Track A (language):   M1 ✅ → M2 ✅ → M3 ✅ → M4 ✅ → M5 ✅ → M6 ✅ → M7 ✅ → M8 ✅
Track B (tooling):    M9 → M10
Track C (platforms):       M11 (done) → M12
Interactive:          M8b ✅ (after M1+M2), M8c ✅ (after M2), M8d ✅ (after M2), M8e (partial)
Late features:        M13 ✅ (after M1+M3), M14 ✅ (after M1+M2)
```

- **M1-M7** are **complete** — state, events, conditionals, slots, images, theming, layout (flex-grow/shrink, percentages)
- **M2** is **complete** — click, hover, keypress events; focus management with Tab/Shift+Tab navigation; focus ring rendering
- **M7** is **complete** — flex-grow, flex-shrink, percentages, align, justify, wrap, min/max constraints
- **M8** is **complete** — multi-page routing with `page`, `link`, and `navigate` action
- **M8b** is **complete** — form inputs with validation
- **M8c** is **complete** — drag & drop with visual feedback (ghost element, drop zone highlighting)
- **M8e** is **nearly complete** — scroll containers with wheel handling, scrollbars, on scroll event, scroll-to action; touch scroll remains
- **M8d** is **complete** — role/label props, compiler warnings, hidden DOM overlay for screen readers
- **M9** (VS Code) can start immediately, no dependencies on Phase 2a
- **M11** (native desktop) core is **done** — `nazec run` with live reload works
- **M13** (data fetching) is **complete** — `data` keyword with loading/error/data states
- **M14** (animation) is **complete** — `transition` prop with easing functions, color/number/opacity interpolation

**Bonus:** `nazec gallery` command builds an interactive web gallery of all examples with instant switching (no page reload).

## WASM Size Budget

Phase 1 runtime: 69KB. Phase 2 adds event handling, state management, image loading, animation. Target: **< 150KB** for the runtime WASM. Monitor after each milestone.

---

## Current Work (Session Notes)

**M2 Event System** - Complete:
- `on hover` triggers when mouse enters element with hover handlers
- `on keypress` fires on focused elements when key pressed
- Tab/Shift+Tab cycles through focusable elements
- Focus ring renders around currently focused element
- Enter key activates focused element, Escape clears focus

**M7 Improved Layout** - Complete:
- `flex-shrink` added - items shrink proportionally when total exceeds available space
- Percentage widths/heights: `width: 50%` resolves relative to parent
- `tab-index` prop added for focus order control

**M8e Scroll & Overflow** - Nearly complete:
- `scroll` element with mouse wheel scrolling and scrollbars
- Content clipping via Canvas2D save/clip/restore
- `on scroll` event fires when scroll position changes
- `scroll-to` action scrolls to element by ID
- Touch scroll remains (stretch goal)

**M8d Accessibility** - Complete:
- `role` and `label` props accepted on all elements
- `id` prop for element identification
- Compiler warnings when interactive elements lack `role` or `label`
- Hidden DOM overlay mirrors canvas content for screen readers
- ARIA roles inferred from element kind (heading, link, button, etc.)
- Form elements mirrored to accessible DOM (input, checkbox, radio, select)
- Screen reader live region announcements remain (stretch goal)

**M13 Data Fetching** - Complete:
- `data` keyword: `data users: fetch "https://api.example.com/users"`
- Derived states: `{name}.loading`, `{name}.error`, `{name}.data`
- Async fetch with wasm-bindgen-futures, JSON parsing to RenderValue
- Components re-render when data arrives

**M14 Animation** - Complete:
- `transition` prop: `transition: "property duration easing"` (e.g., `transition: "background 300ms ease-out"`)
- Easing functions: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out` (cubic-bezier based)
- Animation scheduler using requestAnimationFrame with continuous rendering during active animations
- Property interpolation for colors (RGB component-wise) and numbers (opacity, dimensions)
- Detects property changes and automatically starts animations for transitioned properties

**Next steps (in dependency order):**
1. M9: VS Code extension (TextMate grammar, LSP server)
