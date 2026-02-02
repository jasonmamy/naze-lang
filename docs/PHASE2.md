# Phase 2: Real Apps + Developer Experience

**Goal:** Build non-trivial interactive apps with state, events, and real UI features. VS Code support. Desktop and Android prototypes.

**Architecture shift:** Phase 1's runtime is a one-shot renderer (deserialize → layout → draw). Phase 2 adds a render loop: state changes trigger re-layout and re-draw. The IR format (`naze-ir`) grows to include event handlers, conditionals, and state bindings.

---

## Phase 2a: Dynamic Apps

### M1: State & Reactivity
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Make apps dynamic. This is the foundation for everything else in Phase 2.

- [ ] `let` bindings: `let label = "Hello"` (immutable, compile-time)
- [ ] `state` keyword: `state count = 0` (mutable, triggers re-render on change)
- [ ] Grammar: add `let_stmt` and `state_stmt` rules to pest
- [ ] AST: new `Node::Let` and `Node::State` variants
- [ ] IR: state variable declarations, initial values
- [ ] Compiler: track state variables, emit state info in render tree
- [ ] Runtime: render loop — state change → re-layout → re-draw (requestAnimationFrame)
- [ ] Runtime: state store with change detection

### M2: Event System & Interaction
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `naze-renderer`

Make elements clickable and interactive.

- [ ] `on click: <action>` syntax on any element
- [ ] `on hover`, `on keypress` event handlers
- [ ] Actions: `set count = count + 1`, `set visible = true`, `navigate "/page"`
- [ ] Grammar: `on_handler` rule, action expressions
- [ ] Hit testing: canvas click → point-in-rectangle walk on positioned layout tree
- [ ] Event dispatch: find target → bubble up tree → run handler
- [ ] Cursor changes: pointer cursor on clickable elements
- [ ] Focus management: tab order from layout tree, focusedElement tracking

### M3: Conditional Rendering
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`

Show/hide UI based on state.

- [ ] `if condition { ... }` / `if condition { ... } else { ... }`
- [ ] `each item in list { ... }` for iteration
- [ ] Grammar: `if_stmt`, `each_stmt` rules
- [ ] IR: conditional nodes, list nodes
- [ ] Compiler: validate condition expressions reference valid state
- [ ] Runtime: evaluate conditions during render, include/exclude subtrees

### M4: Content Slots
**Crates:** `naze-parser`, `naze-compiler`

Enable component composition — putting content inside components.

- [ ] `slot name` declaration in component bodies
- [ ] `name: { ... }` syntax at call sites to fill slots
- [ ] Optional slots: `slot name?`
- [ ] Grammar: `slot_decl` rule, slot fill syntax in element blocks
- [ ] Compiler: validate slot names match declarations, inline slot content
- [ ] Default slot content (used when caller doesn't provide)

### M5: Images & Richer Rendering
**Crates:** `naze-parser`, `naze-compiler`, `naze-renderer`, `naze-runtime`

- [ ] `image` element: `image src: "photo.jpg", width: 200px, height: 150px`
- [ ] Runtime: fetch image, decode, draw to canvas (`drawImage`)
- [ ] Image caching (don't re-fetch on re-render)
- [ ] `opacity` prop on any element
- [ ] `border` prop: `border: 1px #cccccc`
- [ ] Viewport resize: `window.resize` → re-layout → re-draw
- [ ] Background color on `row`/`column` (already in grammar, wire to renderer)

### M6: Theming
**Crates:** `naze-parser`, `naze-compiler`

Design tokens for consistent styling.

- [ ] `theme.naze` file format:
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
- [ ] Token references in components: `color: theme.colors.primary`
- [ ] Compiler resolves tokens at compile time (inlines values)
- [ ] Built-in default theme
- [ ] Warn on unknown token references

### M7: Improved Layout
**Crates:** `naze-layout`

Make the layout engine production-quality.

- [ ] `flex-grow` / `flex-shrink` on children
- [ ] `min-width`, `max-width`, `min-height`, `max-height` constraints
- [ ] `align` and `justify` actually computed (currently accepted but ignored in layout)
- [ ] `scroll` container (overflow scrolling with scroll position state)
- [ ] `wrap` prop on `row` (wrapping flex layout)
- [ ] Percentage-based widths/heights (relative to parent)

### M8: Navigation & Routing
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Multi-page apps.

- [ ] `page "/path" { ... }` blocks (multiple per app)
- [ ] `link "text" to: "/path"` element
- [ ] Runtime: History API integration, popstate handling
- [ ] Compiler: multi-page IR (multiple render trees per app)
- [ ] Runtime: page switching without full reload
- [ ] `navigate "/path"` action in event handlers

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

Fast iteration loop.

- [ ] `nazec dev` command: embedded HTTP server (hyper or tiny-http)
- [ ] File watcher on `.naze` files (notify crate)
- [ ] On change: rebuild → notify browser via WebSocket
- [ ] Browser client: WebSocket listener, auto-reload on message
- [ ] Incremental compilation: only re-parse changed files, reuse cached results
- [ ] Console output: "rebuilt in Xms" timing

### M11: Native Desktop Prototype (x86/ARM)
**New crate:** `naze-native`

Proof of concept: same `.naze` source renders in a desktop window.

- [ ] Desktop window via `winit`
- [ ] Software renderer (or `wgpu` / `tiny-skia`) drawing same primitives as Canvas2D
- [ ] Reuses `naze-ir` (deserialize `app_data.bin`) and `naze-layout` (compute positions)
- [ ] `nazec build --target native` produces standalone binary
- [ ] Renders rectangles, text, colors — same visual output as browser
- [ ] Not production quality — proof that the architecture supports native targets

### M12: Android Prototype
**New dir:** `platforms/android/`

Proof of concept: same `.naze` source renders on Android.

- [ ] Option A: Android app embedding Wasmtime, running existing WASM pipeline
- [ ] Option B: Android app with native renderer (Skia / Canvas) reading `app_data.bin`
- [ ] Same `naze-ir` format, same layout engine
- [ ] Basic Android app shell (Kotlin)
- [ ] Demonstrates cross-platform rendering from single `.naze` source

### M13: Data Fetching
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Depends on M1 (state) and M3 (conditionals).

- [ ] `data` keyword: `data users: fetch "https://api.example.com/users"`
- [ ] Data states: `users.loading`, `users.error`, `users.data`
- [ ] Runtime: async fetch, JSON parse, populate state
- [ ] Reactive: components re-render when data arrives
- [ ] `if users.loading { text "Loading..." }` pattern
- [ ] Error handling: `if users.error { text users.error }`

### M14: Animation
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`

Depends on M1 (state) and M2 (events).

- [ ] `transition` prop: `transition color: 150ms ease`
- [ ] Easing functions: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`
- [ ] Runtime: animation scheduler on requestAnimationFrame
- [ ] Property interpolation: color, number, opacity
- [ ] `animate` blocks for explicit keyframe animations (stretch goal)

---

## Build Order

```
Track A (language):   M1 → M2 → M3 → M4 → M5 → M6 → M7 → M8
Track B (tooling):    M9 → M10
Track C (platforms):       M11 → M12
Late features:        M13 (after M1+M3), M14 (after M1+M2)
```

- **M1** is the critical path — state/reactivity unlocks everything
- **M9** (VS Code) can start immediately, no dependencies on Phase 2a
- **M11** (native desktop) can start anytime — reuses existing `naze-ir` and `naze-layout`
- **M13/M14** depend on earlier milestones but can slot in whenever ready

## WASM Size Budget

Phase 1 runtime: 69KB. Phase 2 adds event handling, state management, image loading, animation. Target: **< 150KB** for the runtime WASM. Monitor after each milestone.
