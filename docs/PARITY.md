# Parity Assessment: Naze vs. shadcn/ui

This document compares Naze's foundational primitives (current + fully specced through M30) against what's needed to build a modern component library equivalent to [shadcn/ui](https://ui.shadcn.com/). The goal is to identify gaps and propose spec additions to close them.

**Key finding:** 4 missing primitives block ~60% of shadcn's components. Two new milestones (M19b, M19c) close the gap from 30/59 to 52/59 buildable (51% to 88%).

---

## What shadcn/ui Actually Is

shadcn/ui is a component library (~59 components) built on three foundational layers:

1. **React** — component model, state, rendering, composition
2. **Tailwind CSS** — utility styling (colors, spacing, typography, shadows, transforms, gradients)
3. **Radix UI primitives** — accessibility, focus management, portals/overlays, keyboard navigation, outside-click detection, scroll locking

The components (Dialog, Dropdown, Tooltip, etc.) are compositions of these three layers. The question isn't "can Naze replicate 59 specific components?" but rather: **does Naze provide equivalent foundational primitives?**

---

## Layer-by-Layer Comparison

### Layer 1: Component Model & State (React equivalent)

| Capability | React | Naze (full spec) | Status |
|---|---|---|---|
| Component definitions | `function Comp()` | `component name(props)` | Covered |
| Composition / children | `{children}` | `slot` / `fill` | Covered |
| State management | `useState` | `state` keyword | Covered |
| Reactivity / re-render | Virtual DOM diffing | Auto on state change | Covered |
| Conditional rendering | JSX expressions | `if`/`else`, `each` | Covered |
| Event handling | `onClick`, etc. | `on click`, `on hover`, etc. | Covered |
| Data fetching | `useEffect` + fetch | `data` keyword | Covered |
| Component events (callbacks) | Callback props | M19: `emit` | Planned |
| Refs / direct element access | `useRef` | -- | **Gap** |
| Effect hooks (side effects) | `useEffect` | -- | **Gap** |
| Context / shared state | `useContext` | Theme tokens only | **Gap** |
| Error boundaries | `ErrorBoundary` | -- | **Gap** |
| Portals (render elsewhere) | `createPortal` | -- | **Gap** (see M19b) |

**Assessment:** Core model is solid. The "portals" gap is addressed by M19b's overlay system. Refs, effects, context, and error boundaries are architectural gaps but don't block most UI components. Most shadcn components need portals (overlays) far more than they need refs or effects.

### Layer 2: Styling (Tailwind equivalent)

| Capability | Tailwind | Naze (full spec) | Status |
|---|---|---|---|
| Colors (hex, named) | `bg-blue-500` | `color: #2563eb`, theme tokens | Covered |
| Spacing (padding, gap, margin) | `p-4`, `gap-2` | `padding: 16px`, `gap: 8px` | Covered |
| Typography (size, weight) | `text-lg`, `font-bold` | `font-size: 18px`, `weight: bold` | Covered |
| Borders + radius | `border`, `rounded-lg` | `border: 2px`, `radius: 8px` | Covered |
| Opacity | `opacity-50` | `opacity: 0.5` | Covered |
| Flex layout | `flex`, `grow`, `shrink` | `row`, `column`, `flex-grow`, `flex-shrink` | Covered |
| Grid layout | `grid`, `grid-cols-3` | `grid columns: 3` | Covered |
| Responsive breakpoints | `md:flex-row` | M17: `responsive: stack below 768px` | Planned |
| Dark mode | `dark:bg-gray-900` | M19: `theme dark extends default` | Planned |
| Transitions + animation | `transition-all` | `transition: "prop 300ms ease"` | Covered |
| Shadows (box, drop) | `shadow-md` | -- | **Gap** (see M19c) |
| Text alignment | `text-center` | -- | **Gap** (see M19c) |
| Text overflow / truncation | `truncate` | -- | **Gap** (see M19c) |
| Text decoration | `underline` | -- | **Gap** (see M19c) |
| Line height / letter spacing | `leading-6` | -- | **Gap** (see M19c) |
| Gradients | `bg-gradient-to-r` | -- | **Gap** (see M19c) |
| Transforms (rotate, scale) | `rotate-45`, `scale-110` | -- | **Gap** (see M19c) |
| Z-index / stacking | `z-50` | -- | **Gap** (see M19b) |
| Cursor styles | `cursor-pointer` | Pointer on clickable only | **Gap** (see M19c) |

**Assessment:** Layout and core styling are strong. The gaps are visual polish properties (shadows, gradients, transforms) and text formatting (alignment, overflow, decoration). All addressed by M19c.

### Layer 3: Interaction Primitives (Radix equivalent)

| Capability | Radix UI | Naze (full spec) | Status |
|---|---|---|---|
| Focus management | Focus scope | Tab order, focus ring | Covered |
| Keyboard nav (Tab/Enter/Escape) | Built-in | `on click` via Enter, Escape clears focus | Covered |
| ARIA roles + labels | Automatic | `role`, `label` props | Covered |
| Screen reader support | ARIA | Hidden DOM overlay | Covered |
| Arrow key nav in menus | `RovingFocusGroup` | -- | **Gap** (see M19b) |
| Focus trapping (modals) | `FocusTrap` | -- | **Gap** (see M19b) |
| Outside click detection | `DismissableLayer` | -- | **Gap** (see M19b) |
| Scroll locking | `ScrollLock` | -- | **Gap** (see M19b) |
| Portal / overlay rendering | `Portal` | -- | **Gap** (see M19b) |
| Floating positioning | `@floating-ui/react` | -- | **Gap** (see M19b) |
| Right-click / context menu | `onContextMenu` | -- | **Gap** (see M19b) |
| Pointer tracking | `onPointerMove` | -- | **Gap** (see M19b) |

**Assessment:** Basic accessibility is good. The critical gap is the entire overlay/floating layer — needed by Dialog, Dropdown, Tooltip, Popover, Toast, Sheet, Command, Context Menu, Hover Card, Combobox, Menubar, Navigation Menu, and Date Picker. All addressed by M19b.

---

## Identified Gaps

### Critical (blocks ~17 components)

These 4 primitives are needed by the majority of blocked components:

| # | Gap | Components Blocked | Proposed Solution (M19b) |
|---|-----|-------------------|--------------------------|
| 1 | **Overlay / z-index** | Dialog, Dropdown, Tooltip, Popover, Toast, Sonner, Sheet, Drawer, Context Menu, Hover Card, Command, Combobox, Menubar, Navigation Menu, Date Picker | `overlay` element renders above normal content |
| 2 | **Focus trapping** | Dialog, Alert Dialog, Sheet, Drawer | `focus-trap: true` prop constrains Tab within subtree |
| 3 | **Outside click detection** | Dropdown, Popover, Combobox, Context Menu, Command | `on click-outside: action` event |
| 4 | **Scroll locking** | Dialog, Alert Dialog, Sheet, Drawer | `scroll-lock: true` on overlay |

### Important (blocks visual fidelity across all components)

| # | Gap | Impact | Proposed Solution (M19c) |
|---|-----|--------|--------------------------|
| 5 | **Shadows** | Used pervasively for depth/elevation (cards, dropdowns, dialogs) | `shadow` prop with presets or custom values |
| 6 | **Text alignment** | Needed for centered headings, right-aligned numbers | `text-align` prop |
| 7 | **Text overflow** | Needed for tables, cards, breadcrumbs with long text | `text-overflow: ellipsis` prop |
| 8 | **Gradients** | Used for backgrounds, hover effects | `gradient` prop |
| 9 | **Transforms** | Needed for rotation (Spinner), scale (hover effects), icons | `transform` prop |

### Minor (polish)

| # | Gap | Impact | Proposed Solution (M19c) |
|---|-----|--------|--------------------------|
| 10 | **Text decoration** | Underline for links, strikethrough for deleted items | `text-decoration` prop |
| 11 | **Line height / letter spacing** | Typography fine-tuning | `line-height`, `letter-spacing` props |
| 12 | **Cursor styles** | Only `pointer` today; need `grab`, `text`, `not-allowed`, etc. | `cursor` prop |
| 13 | **SVG / icon support** | No vector graphics; icons require image files | Future milestone (not in M19b/c) |

---

## Component-by-Component Matrix

### Buildable with Current Spec (M1-M19)

These 30 components can be built with existing + already-planned milestones:

| Component | How to Build in Naze | Key Primitives Used |
|-----------|---------------------|---------------------|
| **Accordion** | `state expanded` + `if` + height `transition` | State, conditionals, animation |
| **Alert** | `container` with icon (`image`) + `text` + color variants | Container, slots, theming |
| **Aspect Ratio** | Computed width/height from ratio in parent container | Layout, state |
| **Avatar** | Circular `image` (radius: 50%) + fallback `text` initials | Image, radius, conditionals |
| **Badge** | Small `rect` + `text`, color variants via props | Component, theming |
| **Breadcrumb** | `row` of `link` elements with separator `text` between | Row, link, each |
| **Button** | `rect` + `text` + `on click` + `role: "button"` | Events, accessibility |
| **Button Group** | `row` of Button components | Row, components |
| **Card** | `container` with `slot "header"`, default slot, `slot "footer"` | Slots, container |
| **Carousel** | `scroll` + state for position + prev/next buttons + `transition` | Scroll, state, animation |
| **Checkbox** | Native `checkbox` element with `bind` | Form inputs |
| **Collapsible** | `state open` + `if` + height `transition` (same as Accordion) | State, conditionals, animation |
| **Data Table** | `grid` + `each` + pipeline `sort-by`/`filter` | M15 pipelines, grid, each |
| **Empty** | `container` with centered placeholder content | Container, slots |
| **Field** | `column` wrapping `label` + `input` + error `text` | Column, input, validation |
| **Input** | Native `input` element with `bind`, `placeholder`, `type` | Form inputs |
| **Input Group** | `row` wrapping prefix/suffix elements + `input` | Row, slots |
| **Kbd** | Styled `text` with border + background (keyboard shortcut display) | Rect, text, border |
| **Label** | `text` with `role: "label"` association | Text, accessibility |
| **Native Select** | Native `select` / `option` elements | Form inputs |
| **Pagination** | `row` of page-number buttons with state for current page | Row, each, state, events |
| **Progress** | Two nested `rect`s, inner width from `state` (0-100%) | Rect, state, percentage width |
| **Radio Group** | Native `radio` elements with shared `bind` | Form inputs |
| **Scroll Area** | Native `scroll` element | Scroll |
| **Separator** | `rect` with 1px height/width and muted color | Rect |
| **Skeleton** | `rect` with animated opacity `transition` (pulse) | Rect, animation |
| **Switch** | `rect` track + animated circle `rect` toggling position | Rect, state, transition |
| **Table** | `grid` or `column`/`row` structure + `each` for rows | Grid, each |
| **Tabs** | `row` of tab buttons + conditional panels per tab + `state` | Row, state, conditionals |
| **Toggle** | `rect` + `state` pressed/unpressed + `transition` | Rect, state, animation |
| **Toggle Group** | `row` of Toggle components with shared state | Row, state, components |
| **Typography** | `text` + `heading` elements with size/weight props | Text, heading |

### Buildable After M19b (Overlay System)

These 17 components are blocked **only** by the overlay/interaction gaps. M19b unblocks all of them:

| Component | Blocking Gaps | How to Build After M19b |
|-----------|--------------|------------------------|
| **Alert Dialog** | Overlay, focus-trap, scroll-lock | `overlay focus-trap: true, scroll-lock: true` + backdrop + content |
| **Command** | Overlay, arrow-key nav | `overlay` with `input` + list with `on arrow-down`/`on arrow-up` |
| **Combobox** | Overlay, outside-click, arrow-key nav | `input` + `overlay anchor: "input-id", on click-outside: close` |
| **Context Menu** | Right-click, overlay, positioning | `on context-menu: show-menu` + `overlay anchor: "trigger-id"` |
| **Date Picker** | Overlay for popup calendar | `input` + `overlay` containing grid of date buttons + month/year nav |
| **Dialog** | Overlay, focus-trap, scroll-lock | `overlay focus-trap: true, scroll-lock: true` + backdrop + card |
| **Drawer** | Overlay, focus-trap, scroll-lock | `overlay` from edge + slide `transition` + focus-trap |
| **Dropdown Menu** | Overlay, outside-click, arrow-key nav | Button trigger + `overlay anchor: "btn-id", on click-outside: close` + items |
| **Hover Card** | Overlay, floating positioning | `on hover: show-card` + `overlay anchor: "trigger-id"` |
| **Menubar** | Overlay, arrow-key nav | `row` of triggers + `overlay` per menu + `on arrow-left`/`on arrow-right` |
| **Navigation Menu** | Overlay for flyout menus | Link triggers + `overlay` flyout panels |
| **Popover** | Overlay, outside-click, positioning | Trigger + `overlay anchor: "trigger-id", on click-outside: close` |
| **Resizable** | Pointer tracking | `on pointer-move: resize-handler` + drag handle |
| **Select** (styled) | Overlay, outside-click, arrow-key nav | Trigger button + `overlay` dropdown with option items |
| **Sheet** | Overlay, focus-trap, scroll-lock | Side `overlay` + slide transition + focus-trap |
| **Sonner** | Overlay (toast layer) | `overlay` positioned at viewport edge + toast stack |
| **Toast** | Overlay (toast layer) | `overlay` at viewport corner + auto-dismiss timer |
| **Tooltip** | Overlay, floating positioning | `on hover: show-tip` + `overlay anchor: "trigger-id"` + delay |

### Buildable After M19c (Visual Properties)

These components are functional without M19c but need its properties for visual fidelity:

| Component | Missing Visual Property | Impact Without M19c |
|-----------|------------------------|---------------------|
| **Spinner** | `transform: "rotate()"` | Cannot animate rotation; would need image-based spinner |
| **All cards/dialogs** | `shadow` | No depth/elevation visual cue |
| **Data Table** | `text-align`, `text-overflow` | Numbers can't right-align; long text can't truncate |
| **Breadcrumb** | `text-overflow: ellipsis` | Long paths can't truncate |
| **Button (loading)** | `transform: "rotate()"` | Loading spinner inside button |

### Remaining Gaps After M19b + M19c

| Component | Remaining Gap | Milestone That Closes It |
|-----------|--------------|--------------------------|
| **Chart** | No SVG or canvas drawing API for data visualization | M23 (WASM imports) — import a charting library |
| **Input OTP** | Per-character input cells with auto-advance | Buildable with creative use of multiple `input` elements + state, but awkward |
| **Sidebar** | Responsive collapse at breakpoint | M17 (responsive breakpoints) |
| **Calendar** | Complex date arithmetic | M15 (pure functions for date math) |
| **Direction** | RTL layout support | Not yet specced; layout engine change |
| **Textarea** | Multi-line text input element | Minor gap: add `textarea` element or `input type: "multiline"` |

---

## Parity Score Summary

| Milestone Set | Components Buildable | Score |
|---|---|---|
| Current spec (M1-M19) | 30 | **51%** |
| + M19b (overlay system) | 47 | **80%** |
| + M19c (visual properties) | 52 | **88%** |
| + M15 (pipelines) + M17 (responsive) | 54 | **92%** |
| + M23 (WASM imports for Chart) | 55 | **93%** |
| + `textarea` element + RTL support | 59 | **100%** |

**M19b is the single highest-leverage addition** — it unblocks 17 components by adding one new element (`overlay`) and a handful of props/events.

---

## Application-Level Parity

The component matrix above answers: "can Naze build equivalent UI components?" But real applications built with shadcn/ui are React + Next.js apps with full JavaScript, third-party libraries, browser APIs, and backend integrations. The broader question is: **could any application built with shadcn/ui be fully replicated in Naze?**

The short answer: **no**. Naze achieves strong component parity (~92% with all planned milestones), but application-level parity has deeper gaps that are inherent to Naze's design as a declarative UI language rather than a general-purpose programming language.

### Application Capabilities Beyond Components

| Capability | What shadcn/React apps use | Naze (all milestones) | Status |
|---|---|---|---|
| **General computation** | Full JS/TS (loops, classes, modules, closures) | Tier 1: pipelines + pure functions (M15) | **Architectural** — by design |
| **Complex business logic** | JS modules, npm packages | Tier 2: WASM imports (M23), Tier 3: server functions (M24) | **Bridged** — logic in Rust, not Naze |
| **Global / shared state** | React Context, Zustand, Redux | M19d: `shared state` with optional grouping | **Closed** |
| **Derived / computed state** | `useMemo`, computed values, selectors | M19d: `computed name = expression` | **Closed** |
| **Side effects** | `useEffect`, lifecycle hooks | Reactive `data` URLs + `computed` (by design — no imperative effects) | **Covered** |
| **Complex async flows** | `async/await`, promises, concurrent rendering | M19d: enhanced `data` (full HTTP, `trigger: manual`) | **Closed** |
| **Third-party JS libraries** | Chart.js, Stripe.js, Mapbox, Clerk, etc. | M19e: `js` interop (sync + async calls to `globalThis` functions) | **Partial** — covers most SDK use cases |
| **WebSockets / real-time** | Socket.io, Pusher, SSE | M19d: `data: stream "wss://..."` | **Closed** |
| **Browser APIs** | localStorage, clipboard, geolocation, camera, notifications | M19d: `storage`, `copy`, `param`; M19e: `device` (geolocation, camera), `notify` | **Closed** |
| **File uploads** | `<input type="file">`, drag-to-upload | M19d: `input type: "file"` + enhanced `data` POST (multipart) | **Closed** |
| **Rich text editing** | Tiptap, Slate, ProseMirror, contentEditable | -- (incompatible with Canvas2D rendering) | **Architectural gap** |
| **Timers / scheduling** | `setTimeout`, `setInterval`, debounce/throttle | M19d: `timer` (after/every), `debounce`/`throttle` modifiers | **Closed** |
| **Error handling** | `try/catch`, error boundaries, fallback UI | `data.error` + `if`/`else`; full error boundaries deferred to M19b | **Partial** |
| **URL search params** | `useSearchParams`, query string state | M19d: `param name: type default: value` | **Closed** |
| **Textarea** | `<textarea>` for multi-line text input | M19e: `textarea` element | **Closed** |
| **Dynamic imports** | `React.lazy()`, route-based code splitting | M23 lazy WASM loading (module-level) | **Partial** |

### What Naze Can Fully Replicate

These categories of apps are within reach with all milestones (M1-M30 + M19d/e) complete:

- **Marketing sites and landing pages** — Static content, responsive layout, animations, theme switching. Naze's sweet spot.
- **Content-heavy sites** — Blogs, documentation, portfolios. SSR/SSG (M25) provides SEO. Routing handles multiple pages.
- **Dashboards with real-time data** — Data fetching + pipeline operators (M15) for filtering/sorting + `stream` for live updates + charts via WASM imports (M23).
- **CRUD applications** — Form inputs + validation + enhanced `data` (full HTTP) + server functions (M24) for database operations.
- **E-commerce** — `shared state` for cart, `storage` for persistence, `stream` for inventory updates, server functions for payments, `js` interop for Stripe Elements if needed.
- **Chat and messaging** — `data: stream` for WebSocket, `timer` for typing indicators, `notify` for push notifications, file uploads for media sharing.
- **Search-driven apps** — `param` for URL state, `debounce` on input, reactive `data` fetching, `computed` for filtered/sorted results.
- **Interactive tools and calculators** — State + events + pipeline operators + `computed` derived values handle most interactive logic.
- **Component galleries and design systems** — The component model + theming + testing framework (M20) support this well.

### What Naze Would Struggle With

These app categories hit architectural gaps that primitives alone can't solve:

- **Rich text editing** — Tiptap, Slate, ProseMirror rely on `contentEditable` and deep DOM manipulation. Naze renders to Canvas2D, not DOM — there's no `contentEditable` equivalent. Would require a hybrid DOM rendering mode.
- **Collaborative editing** — Requires CRDT libraries (Yjs, Automerge) for conflict resolution plus WebSocket sync. The `stream` primitive handles the transport, but CRDTs need complex algorithms best handled via WASM imports.
- **Developer tools** — Code editors (Monaco/CodeMirror), terminal emulators, and database GUIs are JS-native applications with deep DOM/browser API integration. JS interop partially bridges this.
- **Apps embedding complex JS widgets** — Google Maps, Mapbox GL, Stripe Elements, embedded video players — these render their own DOM. JS interop (M19e) provides function calls, but embedding a DOM widget inside a Canvas2D app requires a hybrid rendering approach (not yet specced).

### The Design Trade-Off

The remaining gaps (rich text editing, collaborative editing, complex JS widget embedding) are **architectural**, not feature gaps. Naze's core value proposition is:

1. **AI-native** — The language is simple enough for LLMs to generate correctly. Each concept has one canonical form.
2. **Compile-time over runtime** — Components are inlined, theme tokens resolved, types checked at compile time.
3. **No middle layers** — Intent goes to pixels through the shortest path.

The **computation tiers + escape hatches** cover the full application stack:
- **Tier 1** (M15): Declarative data transformations within Naze syntax (pipelines, pure functions, pattern matching)
- **Tier 2** (M23): Import pre-compiled WASM for heavy computation (crypto, image processing, charting)
- **Tier 3** (M24): Server functions for backend logic (database, auth, external APIs)
- **App Logic** (M19d): Reactive primitives for state management, async data, browser APIs, scheduling
- **JS Interop** (M19e): Controlled escape hatch for third-party JS libraries (Stripe, Mapbox, analytics SDKs)
- **Device APIs** (M19e): Declarative access to browser hardware (geolocation, camera, notifications)

Business logic that doesn't fit Tier 1's declarative model gets written in Rust (or another language compiled to WASM) and imported. JS SDK integration uses the `js` interop for function calls. This keeps Naze simple while enabling the vast majority of real-world applications.

### Application Parity Summary

| Parity Level | Description | Before M19d/e | With M19d/e | With All Milestones |
|---|---|---|---|---|
| **Component UI parity** | Can Naze build equivalent UI components? | 92% | 92% | **92%** |
| **Visual fidelity parity** | Does it look the same? | 88% | 88% | **88%** |
| **Application logic parity** | Can Naze handle the same business logic? | ~40% | ~85% | **~99%** |
| **Full application parity** | Can any app be fully replicated? | ~50-60% | ~85-90% | **~95%** of app categories |

---

## Proposed Spec Additions

### M19b: Overlay System & Interaction Primitives

New element and props that enable all overlay-based components:

```naze
-- Dialog example using proposed overlay system
state dialog-open = false

rect role: "button", label: "Open Dialog" {
  text "Open"
  on click: set dialog-open = true
}

if dialog-open {
  overlay focus-trap: true, scroll-lock: true, on click-outside: set dialog-open = false {
    -- Backdrop
    rect width: 100%, height: 100%, color: #00000080

    -- Dialog content, centered
    container width: 480px, padding: 24px, color: #ffffff, radius: 12px, shadow: lg {
      heading "Are you sure?"
      text "This action cannot be undone."
      row gap: 12px, justify: end {
        rect role: "button", label: "Cancel" {
          text "Cancel"
          on click: set dialog-open = false
        }
        rect role: "button", label: "Confirm", color: #dc2626 {
          text "Confirm" color: #ffffff
          on click: set dialog-open = false
        }
      }
    }
  }
}
```

```naze
-- Dropdown menu example
state menu-open = false

rect id: "menu-trigger", role: "button", label: "Options" {
  text "Options"
  on click: set menu-open = !menu-open
}

if menu-open {
  overlay anchor: "menu-trigger", on click-outside: set menu-open = false {
    column color: #ffffff, shadow: md, radius: 8px, padding: 4px {
      rect role: "menuitem", label: "Edit" {
        text "Edit"
        on click: set menu-open = false
      }
      rect role: "menuitem", label: "Delete" {
        text "Delete" color: #dc2626
        on click: set menu-open = false
      }
    }
  }
}
```

```naze
-- Tooltip example
state tip-visible = false

rect id: "tip-trigger" {
  text "Hover me"
  on hover: set tip-visible = true
}

if tip-visible {
  overlay anchor: "tip-trigger" {
    rect color: #1e293b, radius: 4px, padding: 8px {
      text "Helpful tooltip" color: #ffffff, font-size: 12px
    }
  }
}
```

### M19c: Visual Properties Expansion

New props that bring visual fidelity to parity:

```naze
-- Shadows for elevation
container shadow: sm, padding: 16px, radius: 8px {
  text "Card with subtle shadow"
}

container shadow: "0 10px 30px rgba(0,0,0,0.15)", padding: 24px {
  text "Card with custom shadow"
}

-- Text formatting
text "Centered heading" text-align: center, font-size: 24px
text "This is a very long text that should be truncated..." text-overflow: ellipsis
text "Underlined link" text-decoration: underline, color: #2563eb
text "Body copy" line-height: 1.6, letter-spacing: 0.5px

-- Gradients
rect width: 300px, height: 100px, gradient: "linear(to-right, #3b82f6, #8b5cf6)", radius: 8px

-- Transforms
rect width: 20px, height: 20px, color: #666666, transform: "rotate(45deg)"

-- Cursor
rect cursor: grab, width: 200px, height: 30px {
  text "Drag handle"
}
```

---

## Spec Additions Cross-Reference

| Gap | shadcn Components Affected | Milestone | Priority |
|-----|---------------------------|-----------|----------|
| Overlay / portal rendering | 17 components | **M19b** | Critical |
| Focus trapping | Dialog, Alert Dialog, Sheet, Drawer | **M19b** | Critical |
| Outside click detection | Dropdown, Popover, Combobox, Context Menu, Command | **M19b** | Critical |
| Scroll locking | Dialog, Alert Dialog, Sheet, Drawer | **M19b** | Critical |
| Anchor/floating positioning | Tooltip, Popover, Hover Card, Dropdown, Context Menu | **M19b** | Critical |
| Arrow key navigation | Dropdown, Command, Combobox, Menubar, Select | **M19b** | Critical |
| Right-click event | Context Menu | **M19b** | Critical |
| Pointer tracking | Resizable | **M19b** | Critical |
| Shadows | All elevated components (~30) | **M19c** | Important |
| Text alignment | Tables, headings, numbers | **M19c** | Important |
| Text overflow | Tables, cards, breadcrumbs | **M19c** | Important |
| Transforms | Spinner, animations, icons | **M19c** | Important |
| Gradients | Backgrounds, hover effects | **M19c** | Medium |
| Text decoration | Links, deleted items | **M19c** | Medium |
| Line height / spacing | Typography polish | **M19c** | Medium |
| Cursor styles | Drag handles, disabled states | **M19c** | Medium |
| SVG / icons | Icon support across all components | Future | Low |
| Textarea element | Textarea component | Future | Low |
| RTL layout | Direction component | Future | Low |

---

## Application Logic Primitives

The sections above address **UI component** gaps (M19b overlays, M19c visual properties). This section addresses the **application logic** gaps — the reason application parity sits at ~40% while component parity reaches 92%.

### Design Principle: Reactivity Boundary

Not every gap needs a new Naze keyword. The rule:

> **Add a Naze primitive only when the feature is tied to the reactivity/rendering loop.** Pure computation stays in WASM imports (Tier 2). Server logic stays in server functions (Tier 3). Naze primitives handle reactive state, async data lifecycles, browser APIs, and UI-tied scheduling — things WASM imports cannot do because they must observe state changes and trigger re-renders.

This keeps the language small. Each new keyword is justified by asking: "Can a WASM import do this?" If yes, don't add it. If no (because it needs to integrate with state changes, re-rendering, or browser-specific APIs), add it.

### Grammar Budget

Phase 2 added ~20 grammar rules (`state`, `data`, events, `if`/`else`, `each`, `slot`, `fill`, `page`, `link`, `theme`, etc.). The proposals below add **~8 rules total** — about 40% of Phase 2's grammar expansion. Most mirror existing patterns (`state`, `data`), minimizing AI training burden.

### Tier A: High-Leverage Primitives (5 additions, closes 6 of 10 gaps)

#### A1. `computed` — Derived Reactive State

**Replaces:** `useMemo`, computed properties, selector functions

**The problem:** Without derived state, every computed value must be duplicated inline wherever it's used. AI-generated code becomes repetitive and fragile — change the formula in one place, forget it in another.

```naze
-- Derived from state (auto-updates when dependencies change)
computed filtered-items = items | filter status == "active"
computed total-price = cart | map price * quantity | sum
computed full-name = "{first-name} {last-name}"
computed item-count = filtered-items | count
computed has-items = item-count > 0
```

**Semantics:**
- Read-only — cannot be target of `set`
- Auto-tracks dependencies at compile time (scans expression for state/computed refs)
- Re-evaluates only when a dependency changes
- Can reference other `computed` values (compiler validates no cycles)
- Pipeline syntax (M15 dependency) works naturally: `computed x = list | filter | sort | take 5`

**Grammar:** 1 new rule, mirrors `state_stmt` exactly: `computed_stmt = { "computed" ~ ident ~ "=" ~ expression }`

**AI training burden:** Very low. If an AI knows `state x = 5`, it knows `computed x = y + z`.

---

#### A2. `shared state` — Global Shared State

**Replaces:** React Context, Zustand, Redux, global stores

**The problem:** Today all state is scoped to the app/page level. Multi-page apps can't share auth tokens, user preferences, or cart contents across pages. Components can't communicate without prop drilling.

```naze
-- Simple shared state (accessible from any page/component)
shared state current-user = null
shared state auth-token = ""
shared state cart-items = []
shared state theme-mode = "light"

-- Grouped shared state (optional, for namespacing)
shared state auth {
  user = null
  token = ""
  logged-in = false
}
-- accessed as: auth.user, auth.token, auth.logged-in
-- mutated as: set auth.token = "abc123"
```

**Semantics:**
- Same as `state` but not scoped to a page — persists across `navigate` actions
- Changes trigger re-render on any page that references the shared state
- Grouped form is syntactic sugar for dot-prefixed names
- No special mutation API — uses same `set` action as regular state

**Grammar:** 1 modifier keyword (`shared`) on existing `state_stmt`. Optional grouping block.

**AI training burden:** Very low. One modifier on an existing keyword.

---

#### A3. Enhanced `data` — Full HTTP Operations

**Replaces:** `fetch()`, Axios, React Query, SWR, `useEffect` + fetch patterns

**The problem:** Today `data` only supports GET requests with no configuration. Real apps need POST/PUT/DELETE for mutations, auth headers, request params, caching, and retry logic. Without this, any app with a form that submits data is blocked.

```naze
-- Read operation (auto-fetches on mount, reactive to interpolated state)
data users: fetch "/api/users" {
  method: get
  params: { page: current-page, limit: 20 }
  headers: { "Authorization": "Bearer {auth.token}" }
  cache: 5min
  retry: 3
}
-- users.loading, users.error, users.data all work as before

-- Write operation (only fetches when triggered)
data create-result: fetch "/api/users" {
  method: post
  body: { name: name-input, email: email-input }
  headers: { "Authorization": "Bearer {auth.token}" }
  trigger: manual
}
-- Trigger from event handler:
on click: trigger create-result

-- create-result.loading shows spinner during submission
-- create-result.error shows validation errors
-- create-result.data contains the created user

-- Reactive URL — re-fetches automatically when search-query changes
data results: fetch "/api/search?q={search-query}" {
  cache: 30s
}

-- File upload via POST with multipart
data upload-result: fetch "/api/upload" {
  method: post
  body: { file: avatar-file }
  content-type: multipart
  trigger: manual
}
```

**Semantics:**
- Block body is optional — `data x: fetch "url"` still works (GET, no config)
- `trigger: manual` suppresses auto-fetch; activated by `trigger name` action
- All operations produce the same `.loading`/`.error`/`.data` lifecycle
- Reactive URL interpolation: if `{search-query}` changes, re-fetches automatically (GET only)
- `cache: duration` — reuse response for identical requests within TTL
- `retry: count` — retry on network failure with exponential backoff

**Grammar:** 1 rule change (data_stmt gains optional block body with known properties). No new keywords — `trigger` is both a property and an action.

**AI training burden:** Low. Extends a known keyword. The block body follows the same `key: value` pattern used everywhere in Naze.

---

#### A4. `storage` — Reactive Browser Storage

**Replaces:** `localStorage.getItem/setItem`, custom React hooks for persistence

**The problem:** Apps need to persist user preferences (theme, language), session data (cart contents, form drafts), and feature flags across browser sessions. Today Naze has no access to browser storage. WASM imports could call localStorage, but the value wouldn't be reactive — changes wouldn't trigger re-renders.

```naze
-- localStorage (persists across sessions)
storage theme-preference: local "theme" default: "light"
storage cart: local "shopping-cart" default: []
storage recent-searches: local "recent" default: []

-- sessionStorage (persists within tab only)
storage session-id: session "sid" default: ""
```

**Semantics:**
- Behaves like `state` — reactive, triggers re-render on change
- Initialized from storage on load (uses `default` if key not found)
- Changes via `set` auto-sync to storage: `set theme-preference = "dark"` writes to localStorage
- JSON serialization for non-string values (lists, objects)
- `local` = localStorage (persists), `session` = sessionStorage (tab-scoped)

**Grammar:** 1 new rule, mirrors `state_stmt`: `storage_stmt = { "storage" ~ ident ~ ":" ~ ("local" | "session") ~ string ~ "default:" ~ value }`

**AI training burden:** Very low. Follows existing declaration patterns.

---

#### A5. `data: stream` — WebSocket / Server-Sent Events

**Replaces:** WebSocket API, Socket.io, EventSource, Pusher

**The problem:** Real-time features (chat, notifications, live dashboards, collaborative editing) require push-based data. WASM imports cannot receive push notifications from a server and route them into the render loop — this must be a runtime primitive.

```naze
-- WebSocket connection (data is a reactive list, appended on each message)
data chat: stream "wss://api.example.com/chat/{room-id}"
-- chat.data    → reactive list, grows as messages arrive
-- chat.loading → true until connection established
-- chat.error   → set on connection error

-- Server-Sent Events
data notifications: stream "/api/events" {
  type: sse
}

-- Send message on a WebSocket stream
on click: send chat "{message-input}"
```

**Semantics:**
- Reuses the `data` keyword — same `.loading`/`.error`/`.data` lifecycle
- `stream` instead of `fetch` signals a persistent connection
- `.data` is a reactive list that grows as messages arrive (most recent appended)
- URL interpolation is reactive — changing `{room-id}` closes old connection and opens new one
- `send` action pushes a message to a WebSocket stream
- Default type is WebSocket; `type: sse` for Server-Sent Events (read-only)
- Auto-reconnect on disconnect with exponential backoff

**Grammar:** 1 variant added to `data_stmt` (`stream` as alternative to `fetch`), 1 new action (`send`).

**AI training burden:** Very low. Same pattern as `data: fetch`, just with `stream`.

---

### Tier B: Polish Primitives (4 additions, closes 3 more gaps)

#### B1. `param` — URL Query Parameters

**Replaces:** `useSearchParams`, query string parsing, URL state management

**The problem:** Search pages, paginated lists, and filtered views need state in the URL so users can bookmark, share, and use browser back/forward. Today Naze has `navigate` for path routing but no query parameter support.

```naze
param page: number default: 1
param search: text default: ""
param sort: text default: "newest"

-- Two-way bound: changing the param updates the URL, changing the URL updates the param
on click: set page = page + 1
-- URL becomes: ?page=2&search=&sort=newest

-- Use in data fetching (reactive — re-fetches when params change)
data items: fetch "/api/items" {
  params: { page: page, q: search, sort: sort }
}
```

**Semantics:**
- Behaves like `state` — reactive, usable in expressions and templates
- Two-way bound to URL query string via `replaceState`
- Type-validated: `number` params parse to numeric, `text` stays as string
- `default` used when param is absent from URL
- Browser back/forward updates param values and triggers re-render

**Grammar:** 1 new rule: `param_stmt = { "param" ~ ident ~ ":" ~ type ~ "default:" ~ value }`

**AI training burden:** Very low. Declarative, mirrors existing patterns.

---

#### B2. `timer` — Scheduled Actions + Event Modifiers

**Replaces:** `setTimeout`, `setInterval`, `debounce()`, `throttle()`

**The problem:** Auto-save, toast auto-dismiss, polling, search debouncing, and scroll throttling all require time-based scheduling. The animation scheduler exists internally but isn't exposed to user code. WASM imports can't call `setTimeout` and trigger re-renders.

```naze
-- One-shot timer (fires once after delay)
timer toast-dismiss: after 5s {
  set show-toast = false
}

-- Repeating timer (fires every interval)
timer auto-save: every 30s {
  trigger save-draft
}

-- Debounce on event handlers (wait for pause in activity)
input bind: search-query, on change debounce 300ms: trigger search-results

-- Throttle on event handlers (max once per interval)
on scroll throttle 100ms: set scroll-position = event.y
```

**Semantics:**
- `after duration { action }` — executes once, then stops
- `every duration { action }` — repeats until component unmounts or timer is explicitly stopped
- Duration units: `ms`, `s`, `min` (e.g., `300ms`, `5s`, `30min`)
- `debounce Nms` on events — delays action until N ms of inactivity
- `throttle Nms` on events — executes at most once per N ms
- Timers are automatically cleaned up when their page/component is no longer rendered

**Grammar:** 1 new rule (`timer_stmt`), 2 event modifiers (`debounce`, `throttle`).

**AI training burden:** Low. Two clear forms (`after` vs `every`), predictable duration syntax.

---

#### B3. `copy` / `paste` — Clipboard Actions

**Replaces:** `navigator.clipboard.writeText()`, `navigator.clipboard.readText()`

**The problem:** Copy-to-clipboard buttons (for code snippets, share links, API keys) are ubiquitous in modern apps. WASM can call the clipboard API, but the result must flow back into reactive state to update UI ("Copied!" feedback).

```naze
-- Copy text to clipboard
rect role: "button", label: "Copy" {
  text "Copy link"
  on click: copy share-url
}

-- Copy with feedback
state copied = false
on click: copy api-key, set copied = true
timer reset-copied: after 2s { set copied = false }
if copied { text "Copied!" color: #16a34a }
```

**Semantics:**
- `copy expression` — evaluates expression and writes result to system clipboard
- Added to existing action vocabulary alongside `set`, `navigate`, `scroll-to`, `log`
- Async (clipboard API requires permission) but fire-and-forget — no `.loading` state needed
- Failure silently ignored (clipboard permission denied is rare and non-critical)

**Grammar:** 1 new action variant in existing `action` rule. No new declaration keywords.

**AI training burden:** Negligible. Same pattern as `log expression`.

---

#### B4. `input type: "file"` — File Selection

**Replaces:** `<input type="file">`, drag-and-drop file handling

**The problem:** Profile photo uploads, document attachments, and CSV imports all require file selection. This is a missing input type, not a new concept.

```naze
-- File input (binds selected file to state)
input type: "file", bind: avatar-file, accept: "image/*", max-size: 5mb

-- Upload via enhanced data POST
data upload-result: fetch "/api/upload" {
  method: post
  body: { file: avatar-file }
  content-type: multipart
  trigger: manual
}

on click: trigger upload-result

-- Show upload progress
if upload-result.loading { text "Uploading..." }
if upload-result.error { text upload-result.error color: #dc2626 }
if upload-result.data { text "Upload complete" color: #16a34a }
```

**Semantics:**
- Extends existing `input` element with `type: "file"` variant
- `accept` — MIME type filter (e.g., `"image/*"`, `"application/pdf"`)
- `max-size` — client-side file size limit with unit (e.g., `5mb`, `100kb`)
- `bind` works as with other inputs — selected file is stored in state variable
- Upload handled by enhanced `data` with `method: post` and `content-type: multipart`

**Grammar:** 0 new rules. Extends existing `input` element type values and adds `accept`/`max-size` props.

**AI training burden:** Negligible. Follows existing input element patterns exactly.

---

### What Was Explicitly Excluded

These features were considered and rejected to keep the language simple for AI generation:

| Feature | Why Excluded | What Covers It Instead |
|---|---|---|
| **`watch` (side effects)** | Invites imperative patterns; `useEffect` is the most misused React hook. Undermines declarative design. | Reactive `data` URL interpolation re-fetches automatically when dependencies change. `computed` handles derived values. |
| **`store` blocks** | Introduces a sub-language (actions with fetch/then/catch chains inside store definitions). 4-5 new grammar concepts in one feature. | `shared state` modifier + enhanced `data` + `computed` compose to cover the same use cases with existing patterns. |
| **`action` keyword** | Creates a second HTTP mechanism alongside `data`, with different lifecycle semantics. AI must learn when to use which. | Enhanced `data` with `trigger: manual` handles write operations using the same `.loading`/`.error`/`.data` lifecycle. |
| **Error boundaries (`try`/`catch`)** | Imperative vocabulary in a declarative language. Existing `data.error` + `if`/`else` covers 90% of error UI cases. | Deferred to M19b where `fallback`/`error` blocks make more sense alongside the overlay system. |
| **General `async`/`await`** | Turing-complete async would destroy AI-generability. Naze is declarative — it describes what data to fetch, not how to orchestrate promises. | `data` (with full HTTP), `stream` (WebSocket), `timer` (scheduling), and server functions (M24) cover specific async patterns without general-purpose async. |

---

### Responsibility Division: Naze vs. WASM vs. Server

| Responsibility | Where It Lives | Why |
|---|---|---|
| **Reactive state** (shared, computed, storage, params) | **Naze primitives** | Must observe changes and trigger re-renders |
| **Async data lifecycle** (fetch, stream, loading/error states) | **Naze primitives** | Must integrate with UI rendering (spinners, error messages) |
| **Timers & scheduling** (debounce, auto-save, polling) | **Naze primitives** | Must trigger state changes that cause re-renders |
| **Browser APIs** (localStorage, clipboard, URL params) | **Naze primitives** | Browser-side, must be reactive |
| **Heavy computation** (crypto, image processing, parsing) | **WASM imports (Tier 2)** | Pure input→output, no reactivity needed |
| **Data transformation** (filter, sort, map, reduce) | **Pipeline operators (Tier 1)** | Declarative, compile-time optimizable |
| **Backend logic** (database, auth, payments, email) | **Server functions (Tier 3)** | Server-side, security-sensitive |

---

### AI Training Complexity Analysis

| Feature | New Grammar Rules | Mirrors Existing Pattern | AI Decision Points | Training Burden |
|---|---|---|---|---|
| `computed` | 1 | `state` (identical structure) | 0 (one form) | **Very Low** |
| `shared state` | 0 (modifier) | `state` (adds one word) | 1 (shared or not) | **Very Low** |
| Enhanced `data` | 1 (block body) | `data` (extends with properties) | 3-4 (which props) | **Low** |
| `storage` | 1 | `state` (similar structure) | 1 (local vs session) | **Very Low** |
| `data: stream` | 1 (variant) | `data: fetch` (same lifecycle) | 0 (one form) | **Very Low** |
| `param` | 1 | Component params (similar) | 0 (one form) | **Very Low** |
| `timer` | 1 | New concept | 2 (every vs after) | **Low** |
| `debounce`/`throttle` | 0 (modifiers) | Event handlers (extends) | 1 (which modifier) | **Very Low** |
| `copy` action | 0 (extends action) | `log` (identical pattern) | 0 | **Negligible** |
| `input type: "file"` | 0 | `input type: "text"` (same) | 0 | **Negligible** |
| **Total** | **~6 new rules** | | | **Low overall** |

For comparison, an AI generating the equivalent JS/React code must navigate: `useState` vs `useReducer` vs `useRef`, `useEffect` dependency arrays, `useMemo` vs `useCallback`, `fetch` vs `axios` vs React Query, Context providers vs Zustand vs Redux, `localStorage` API quirks, `Promise` chaining vs `async/await`, `WebSocket` event handlers, `setTimeout`/`clearTimeout` cleanup, `FormData` for file uploads, `URLSearchParams` parsing, `navigator.clipboard` permissions — each with multiple valid patterns and common pitfalls. Naze's ~6 new rules replace all of this with one canonical form per concept.

---

### Application Logic Parity Projection

| Addition | Score Gain | Cumulative | Rationale |
|---|---|---|---|
| Baseline (current spec) | — | **~40%** | `state`, `data` (GET only), basic events |
| + `computed` | +8% | **48%** | Derived state eliminates huge class of boilerplate |
| + `shared state` | +7% | **55%** | Cross-component state unlocks multi-page apps |
| + Enhanced `data` (full HTTP) | +10% | **65%** | POST/PUT/DELETE, headers, params covers most API integration |
| + `storage` | +5% | **70%** | Browser persistence enables preferences, carts, drafts |
| + `data: stream` | +3% | **73%** | Real-time chat, notifications, live data |
| + `param` | +3% | **76%** | URL-driven state enables search, pagination, deep linking |
| + `timer` + debounce/throttle | +4% | **80%** | Auto-save, toast dismiss, search debounce |
| + `copy` + file input | +2% | **82%** | Clipboard and file handling |
| + M15 Pipelines (already planned) | +5% | **87%** | Data transformation, filtering, sorting |
| + M23 WASM imports (already planned) | +5% | **92%** | Third-party computation, charting |
| + M24 Server functions (already planned) | +5% | **97%** | Database, auth, payment processing |
| + M19e: JS interop | +1% | **98%** | Third-party JS SDK calls (Stripe, Mapbox, analytics) |
| + M19e: Browser APIs + textarea | +1% | **99%** | Geolocation, camera, notifications, multi-line text |

The remaining ~1% represents architectural gaps: rich text editing (requires DOM, incompatible with Canvas2D), collaborative editing (requires CRDT libraries), and embedding complex JS widgets that render their own DOM (maps, video players).

---

## M19e: Remaining Gap Closures

Three additions that close the final parity gaps: `textarea` (trivial), JS interop (the ~3% JS SDK gap), and browser device APIs. Total grammar expansion: ~3 new rules.

### E1. `textarea` — Multi-line Text Input

**Replaces:** `<textarea>` HTML element

**The problem:** Comments, descriptions, bio fields, and code input all need multi-line text. This is the only missing form element.

```naze
state bio = ""
textarea bind: bio, placeholder: "Tell us about yourself...", rows: 4, max-length: 500

text "{bio}" -- displays current textarea content
```

**Semantics:**
- Same as `input` but supports multi-line text with line breaks
- `bind` for two-way state binding (same as other form elements)
- `rows` — visible height in text rows (default: 3)
- `max-length` — character limit (optional)
- `placeholder` — hint text when empty

**Grammar:** 0 new rules. New element keyword `textarea` with same props pattern as `input`.

**AI training burden:** Negligible. Identical to `input` usage.

---

### E2. JS Interop — Third-Party Library Integration

**Replaces:** `import` of JS libraries, `<script>` tags, `window.Stripe()`, SDK initialization

**The problem:** ~3% of application parity is blocked by JS-only SDKs (Stripe Elements, Mapbox GL, Google Maps, analytics providers, auth SDKs). WASM imports handle Rust/C/Go libraries, but many browser SDKs only ship JavaScript. A controlled JS interop lets Naze call these without becoming a general-purpose JS runtime.

#### Script Inclusion (in `naze.toml`)

```toml
[scripts]
stripe = "https://js.stripe.com/v3/"
mapbox = "https://api.mapbox.com/mapbox-gl-js/v3/mapbox-gl.js"
analytics = "./js/analytics.js"
```

The compiler embeds `<script>` tags in the generated `index.html`. Scripts expose functions on `globalThis`.

#### Sync Calls (in `.naze` files)

```naze
-- Call a JS function as an event action
on click: js "analytics.track"("button_clicked", { page: "home" })

-- Call and store return value
on click: js "Stripe"(stripe-key) -> stripe-instance

-- Use return value in subsequent calls
on click: js "stripe-instance.redirectToCheckout"({ sessionId: checkout-session }) -> checkout-result
```

#### Async Calls (with data lifecycle)

```naze
-- Async JS call with loading/error/data lifecycle
data checkout: js "createCheckoutSession"(cart-items) {
  trigger: manual
}

on click: trigger checkout
if checkout.loading { text "Redirecting to payment..." }
if checkout.error { text checkout.error color: #dc2626 }
```

**Semantics:**
- `js "functionName"(args)` — calls `globalThis.functionName(args)` synchronously
- `js "name"(args) -> target` — calls function, stores return value in state variable
- `data name: js "functionName"(args)` — async JS call with `.loading`/`.error`/`.data` lifecycle (reuses `data` pattern)
- `trigger: manual` works the same as with `data: fetch`
- Type marshalling: `number` ↔ f64, `text` ↔ string, `bool` ↔ boolean, list ↔ Array, object ↔ Object
- Return values are converted to Naze types; unconvertible values become `text` (JSON stringified)
- Functions must be on `globalThis` — no module imports, no `require()`, no dynamic loading

**Grammar:** 1 new action variant (`js_action`), 1 new data source variant (`data: js`). Low complexity.

**AI training burden:** Low. One action pattern (`js "name"(args)`) plus the existing `data` lifecycle reuse for async. AI learns one interop mechanism, not a full JS embedding.

**Security consideration:** JS interop is opt-in via `naze.toml` `[scripts]` section. The compiler can warn if `js` actions reference functions not provided by declared scripts.

---

### E3. Browser Device APIs — Geolocation, Camera, Notifications

**Replaces:** `navigator.geolocation`, `navigator.mediaDevices`, `Notification API`, `navigator.vibrate`

**The problem:** Location-aware apps, camera capture, and push notifications require browser hardware APIs. These are inherently tied to the render loop (permission prompts, async results, reactive state updates). WASM can't access them directly.

#### Declarative Device Access

```naze
-- Geolocation (one-shot — get current position)
data location: device geolocation
-- location.loading → true while acquiring GPS
-- location.error → "Permission denied" or "Position unavailable"
-- location.data → { latitude: 40.7128, longitude: -74.006, accuracy: 10 }

-- Geolocation (continuous watch)
data location: device geolocation { watch: true }
-- location.data updates as device moves

-- Camera
data camera: device camera { facing: "user", width: 640, height: 480 }
-- camera.data → media stream (rendered in a video-like element)
-- camera.error → "Permission denied" or "No camera available"
```

#### Notification Action

```naze
-- Send browser notification (fire-and-forget, like copy)
on click: notify "Order Shipped!" {
  body: "Your order #1234 is on its way."
  icon: "shipping-icon.png"
}
```

**Semantics:**
- `data: device API_NAME` — reuses the `data` lifecycle (`.loading`/`.error`/`.data`)
- `device` keyword signals "browser hardware API requiring permissions"
- Permission handling is implicit — denied permissions surface as `.error`
- `notify` action — requests notification permission on first use, then shows notification
- Supported APIs (initial set): `geolocation`, `camera`, `notification`
- Future APIs can be added without new grammar: `device accelerometer`, `device bluetooth`, etc.

**Grammar:** 1 new data source variant (`device`), 1 new action (`notify`).

**AI training burden:** Very low. Same `data` lifecycle pattern. AI learns `device geolocation` the same way it learned `fetch "url"` and `stream "wss://url"`.

---

### Updated Application Logic Parity Projection

| Addition | Score Gain | Cumulative | Rationale |
|---|---|---|---|
| M19d (all Tier A+B) | — | **82%** | State, data, scheduling, browser storage |
| + M15 Pipelines | +5% | **87%** | Data transformation |
| + M23 WASM imports | +5% | **92%** | Third-party computation |
| + M24 Server functions | +5% | **97%** | Backend logic |
| + M19e: JS interop | +1% | **98%** | JS SDK calls |
| + M19e: Device APIs + textarea | +1% | **99%** | Hardware access, multi-line text |

**Remaining ~1%:** Rich text editing (Canvas2D architectural gap), collaborative editing (CRDTs), embedding complex JS DOM widgets (maps, video players — would need hybrid DOM rendering).

---

## Development Time: Naze vs. React/Next.js

Beyond feature parity, Naze's design delivers significant development time savings. The simplifications are structural — not marginal improvements to the same workflow, but entire categories of work that disappear.

### The Framework Tax

Every React/Next.js project starts with hours of setup, configuration, and decision-making before a single feature is built. This "framework tax" repeats on every project:

| Framework Tax Item | React/Next.js | Naze |
|---|---|---|
| Project scaffolding (create-next-app, ESLint, Prettier, tsconfig) | 30min - 2hrs | `nazec new` (seconds) |
| Dependency selection (state mgmt, data fetching, styling, forms) | 1 - 4hrs research | Built-in primitives — no choice needed |
| CSS system setup (Tailwind config, CSS modules, styled-components) | 1 - 3hrs | No CSS — styling is inline props |
| Bundler configuration (webpack/turbopack, code splitting, tree shaking) | 0 - 4hrs | No bundler — single-pass compiler |
| TypeScript setup + type plumbing | 1 - 2hrs | Types built into language |
| State management setup (Context/Zustand/Redux boilerplate) | 2 - 6hrs | `state`, `shared state`, `computed` — zero config |
| API integration patterns (React Query/SWR setup, error handling) | 2 - 4hrs | `data` keyword with built-in lifecycle |
| Deployment config (Vercel/Docker/nginx, env vars, build scripts) | 1 - 4hrs | `nazec build` → static files |
| **Total framework tax** | **8 - 29 hours** | **~0** |

This tax is paid on every new project. For agencies, freelancers, or teams spinning up multiple apps, the cumulative cost is substantial.

### Development Time Estimates

Estimated time to build equivalent applications, comparing React/Next.js with Naze for both human developers and AI-assisted workflows:

| App Complexity | Example | React/Next.js | Naze (Human) | Naze (AI-Assisted) | Human Reduction | AI Reduction |
|---|---|---|---|---|---|---|
| **Simple** | Marketing site, portfolio, landing page (3-5 pages) | 20 - 40 hrs | 8 - 16 hrs | 3 - 8 hrs | **50 - 60%** | **70 - 80%** |
| **Medium** | Admin dashboard with auth, data tables, forms, charts | 80 - 160 hrs | 30 - 60 hrs | 15 - 40 hrs | **50 - 65%** | **65 - 80%** |
| **Complex** | E-commerce with cart, payments, real-time inventory, search | 200 - 500 hrs | 90 - 200 hrs | 60 - 150 hrs | **40 - 55%** | **55 - 70%** |

### Where the Savings Come From

The time reductions aren't from writing less code (though Naze is more concise). They come from eliminating entire categories of development work:

- **No framework tax** — 8-29 hours eliminated on every project. No dependency research, no config files, no build tool debugging.
- **No CSS** — Styling is inline props. No class naming conventions, no cascade debugging, no dead CSS hunting, no responsive breakpoint configuration files. The visual appearance is co-located with the structure.
- **No state management library** — `state`, `shared state`, and `computed` replace the entire Context/Zustand/Redux decision tree plus its boilerplate, provider wrappers, and selector patterns.
- **No async plumbing** — The `data` keyword with its built-in `.loading`/`.error`/`.data` lifecycle replaces: `useEffect` + `fetch` + `try/catch` + loading state boolean + error state + cleanup function + stale closure prevention. One line of Naze replaces 15-25 lines of React.
- **One canonical form** — No decision paralysis. One way to do routing (`page`), one way to fetch data (`data`), one way to handle forms (`input` + `bind`), one way to manage events (`on event: action`). Developers spend time building, not choosing between equivalent approaches.
- **AI generation accuracy** — AI generates correct Naze on the first attempt more often because the language has fewer valid alternatives. A React component can be written 50+ ways (class vs function, hooks vs HOC, CSS modules vs Tailwind vs styled-components, relative vs absolute imports). A Naze component has one correct form.

### Caveats

These estimates assume all planned milestones (through M24) are implemented. Additional considerations:

- **Ecosystem advantage** — React/Next.js has a vast library ecosystem. For apps heavily dependent on specific React libraries (rich text editors, complex animation libraries, specialized UI widgets), Naze's savings diminish because equivalent functionality must be built or accessed via WASM imports / JS interop.
- **AI training dependency** — The "AI-Assisted" column assumes AI models trained on Naze syntax. Without training data, initial AI generation will be slower. However, Naze's constrained grammar makes it suitable for few-shot learning — AI models can generate reasonable Naze from a small number of examples.
- **Developer familiarity** — The "Human" column assumes developers proficient in Naze. Initial learning curve is minimal (the full language spec fits in a single document), but first projects may take longer as developers internalize patterns.

---

## Long-Term Maintenance: Naze vs. React/Next.js

Development time is the upfront cost. But the larger cost of software is **maintenance** — the ongoing work of keeping an application functional, secure, and evolvable over months and years. This is where React/Next.js codebases struggle most, and where Naze's design delivers its greatest advantage.

### The React/Next.js Maintenance Problem

These are not theoretical concerns — they are the daily reality of maintaining production React/Next.js applications at scale:

| Problem | Description | Typical Impact |
|---|---|---|
| **Dependency rot** | `node_modules` with 500-2000+ transitive dependencies. `npm audit` flags vulnerabilities weekly. Major version upgrades (React 17→18→19, Next 12→13→14→15) break APIs and require code changes across the project. | 10-20% of ongoing dev time on dependency updates, security patches, and compatibility debugging. |
| **Framework churn** | Pages Router → App Router. `getServerSideProps` → server components. `getStaticProps` → `generateStaticParams`. CSS Modules → Tailwind → CSS-in-JS → back to CSS Modules. Each paradigm shift touches every file. | Multi-week migrations every 1-2 years. Old patterns coexist with new, creating permanent inconsistency. Documentation becomes unreliable as it mixes eras. |
| **Multiple valid patterns** | `useState` vs `useReducer` vs Zustand vs Redux vs Jotai. `fetch` vs Axios vs React Query vs SWR vs tRPC. CSS Modules vs Tailwind vs styled-components vs vanilla-extract. Different developers choose differently across the codebase. | Codebase becomes a patchwork of conventions. Onboarding new developers takes weeks. Code review devolves into style debates. "Which pattern do we use here?" is a recurring question with no stable answer. |
| **Hook complexity** | `useEffect` dependency arrays are the #1 source of React bugs. Stale closures capture outdated state. Cleanup functions race with re-renders. `useMemo`/`useCallback` overuse degrades readability without improving performance. Missing dependencies cause silent bugs. | Subtle bugs that pass code review and testing, only surfacing in production under specific timing conditions. Performance issues from over-memoization or under-memoization. Senior developer time spent debugging hook interactions. |
| **CSS drift** | Tailwind utility class sprawl (`"flex flex-col items-center justify-between gap-4 p-6 bg-white dark:bg-gray-900 rounded-lg shadow-md hover:shadow-lg transition-shadow"`). Dead CSS classes accumulate. Specificity conflicts between component styles. Responsive breakpoint definitions vary across components. | Visual bugs on edge devices and viewport sizes. Designers and developers disagree on what's "correct." Refactoring CSS is high-risk because cascade effects are non-local — changing one class can break unrelated components. |
| **TypeScript tax** | Complex generic types for HOCs, render props, forwarded refs. Discriminated unions for action types. Type errors that don't correspond to runtime bugs. Third-party library type definitions that lag behind releases. | 15-30% of code is type annotations. Complex types slow IDE responsiveness. `any` type escapes accumulate over time as developers work around type system limitations. Types create a false sense of safety while missing the bugs that actually matter. |
| **Build complexity** | Webpack/turbopack configuration, babel plugins, PostCSS pipelines, custom loaders, build-time environment variables, code splitting boundaries, dynamic import paths. | "Works locally, fails in CI" bugs become routine. Build times grow from seconds to minutes as projects scale. Debugging build configuration issues requires specialized knowledge separate from application development. |
| **Implicit behavior** | Server components vs client components (the `"use client"` boundary). Automatic re-renders from context changes. Suspense boundary fallback behavior. Parallel route rendering order. Middleware execution sequence. | "Why did this re-render?" is the most common React debugging question. Application behavior can change without any code changes — simply updating the framework version alters implicit rendering behavior. |

### How Naze Avoids These Problems

Naze's design decisions aren't just simplifications — they are **structural guarantees** that prevent entire categories of maintenance problems from occurring:

| Naze Design Principle | What It Prevents | Mechanism |
|---|---|---|
| **Zero dependencies** | Dependency rot, supply chain vulnerabilities, version conflicts | Single compiler binary (`nazec`). No `node_modules`, no `package.json`, no lock file. Updating Naze means updating one tool, not reconciling 2,000 packages. |
| **One canonical form** | Pattern inconsistency, decision paralysis, codebase style drift | One way to manage state (`state`/`shared state`/`computed`). One way to fetch data (`data`). One way to handle events (`on event: action`). One way to style elements (inline props). No alternatives to disagree about. |
| **No CSS** | CSS drift, specificity conflicts, dead classes, responsive inconsistencies | Styling is inline props on elements. No cascade, no specificity, no separate stylesheet files, no CSS class naming conventions. What you see in the `.naze` file is exactly what renders. |
| **Compile-time validation** | Runtime surprises, implicit behavior, type errors that don't match bugs | Components are inlined, theme tokens resolved, types checked at compile time. If it compiles, the structure is correct. No "works locally, fails in production" from framework-level configuration mismatches. |
| **No hidden state** | Stale closures, hook dependency bugs, mysterious re-renders | State is explicit: `state`, `shared state`, `computed`. No `useEffect`, no cleanup functions, no dependency arrays, no memoization decisions. Reactivity is automatic and deterministic — when state changes, dependent UI updates. No exceptions, no special cases. |
| **Stable language** | Framework churn, migration fatigue, coexisting old/new patterns | `.naze` files written in Phase 1 still compile today. No pages→app router migration. No class→hooks migration. No CSS framework rotation. The language evolves additively — new keywords are added, existing patterns are never replaced or deprecated. |
| **AI-verifiable** | Code review gaps, inconsistent patterns, "it works but it's wrong" | Constrained grammar means AI tools can validate Naze code for correctness with high confidence. One canonical form means AI-generated code is indistinguishable from human-written code — no style debates between human and AI contributions. |
| **Single-pass compilation** | Build complexity, CI debugging, configuration drift | `nazec build` produces static files. No webpack, no babel, no PostCSS, no plugin configuration, no build caching issues, no code splitting boundaries to manage. Build takes seconds regardless of project size. |

### How Codebases Age

The comparison becomes more stark over time. Initial development might feel similar, but maintenance costs diverge dramatically as applications mature:

| Metric | React/Next.js at Scale | Naze at Scale |
|---|---|---|
| **Onboarding time** (new dev productive) | 1-4 weeks — learn which state management pattern to use where, which CSS convention applies to which module, how the build config works, where the "legacy" patterns are | Hours to days — one language, one set of patterns, every file follows the same conventions |
| **Dependency updates** | Weekly — security patches, version bumps, breaking changes; each update risks regressions | Rare — update the `nazec` binary; no transitive dependencies to reconcile |
| **Framework migrations** | Major migration every 1-2 years requiring weeks to months of work (class→hooks, pages→app router, CJS→ESM) | None — additive language evolution only; no deprecations, no migrations |
| **Code consistency** | Degrades over time as patterns accumulate, developers rotate, and framework paradigms shift | Enforced by grammar — one way to express each concept; consistency is structural, not conventional |
| **Dead code accumulation** | High — unused CSS classes, abandoned components, legacy utility functions, vestigial type definitions | Low — compiler reports unused components and state; no CSS layer to accumulate dead rules |
| **Build time growth** | Grows with project size — seconds for small projects, 2-5 minutes for large ones | Sub-second regardless of size — single-pass compiler with binary serialization |
| **"Bus factor" risk** | High — tribal knowledge about which patterns to use, framework quirks, environment-specific build config, "don't touch that file" warnings | Low — the language is the documentation; any developer who knows Naze can read and modify any Naze codebase |
| **AI maintainability** | Moderate — AI struggles with complex hook interactions, implicit framework behavior, project-specific conventions, and the combinatorial explosion of valid patterns | High — constrained grammar, one canonical form per concept; AI can read, modify, and review Naze code with high accuracy |

### A Concrete Comparison

Consider a **dashboard application** with 50+ components, 2 years old, maintained by a rotating team of developers:

**The React/Next.js version** has accumulated:
- **3 different state management approaches** — legacy Redux slices from the original build, React Context for some newer pages, and Zustand stores added by a recent hire who preferred it. All three coexist, and no one has time to unify them.
- **Mixed styling** — Tailwind for most components, CSS Modules for the data grid (because a developer needed `:global` selectors), and inline styles on a few components a contractor wrote.
- **Partial framework migration** — The team started migrating from Pages Router to App Router but only converted half the routes. Both coexist, with different data fetching patterns (`getServerSideProps` on old pages, server components on new ones).
- **1,800 dependencies** — The project started with ~200 direct deps. Transitive dependencies grew to 1,800. `npm audit` reports 14 vulnerabilities, 3 of which are in deeply nested packages the team doesn't control. A React 19 upgrade was attempted but abandoned after it broke 4 third-party UI libraries.
- **Increasingly complex types** — TypeScript generic types for reusable table components span 80+ lines. Several modules use `any` casts to work around type limitations. The IDE is noticeably slower in these files.

**The Naze equivalent** has:
- **One state pattern** — `state`, `shared state`, and `computed`, the same on every page. A new developer reads any component and immediately understands how state flows.
- **No styling layer** — Props on elements. No CSS files to manage, no specificity bugs, no dead class accumulation.
- **No migration debt** — The syntax used on day one still works. New language features (added via milestone updates) don't require changing existing code.
- **Zero dependencies** — `nazec` binary updated once per release. No vulnerability reports. No transitive dependency chains to untangle.
- **Fast builds, fast onboarding** — `nazec build` takes seconds. A new developer reads the language reference (one document) and is productive within a day.

The React version requires a senior developer just to **understand** the codebase before making changes. The Naze version is readable by anyone who knows the language — because the language enforces the conventions that React teams spend years trying to establish through code review, linting rules, and architecture decision records.

### AI Context-Loading Cost

There's a hidden cost to maintaining any codebase with AI assistance that's rarely discussed: **context loading**. Every time an AI agent (Claude Code, Cursor, Copilot Workspace, etc.) works on a codebase, it must read files to build working context before it can make changes. This "read" step costs input tokens — which cost money and time. For maintenance, where the AI touches the codebase hundreds or thousands of times over a project's lifetime, the cost compounds significantly.

Naze codebases are structurally smaller and more information-dense than React/Next.js codebases for equivalent functionality. This isn't about writing terser code — it's about eliminating entire file categories (CSS, type definitions, config, state management boilerplate) that exist only because of framework complexity.

#### Codebase Size Comparison

For an equivalent medium application (~50 components):

**React/Next.js codebase:**

| File Category | Files | Avg Lines | Total Lines | Notes |
|---|---|---|---|---|
| Component files (.tsx) | 50 | 120-180 | 6,000-9,000 | JSX + hooks + imports + type annotations |
| CSS / styling files | 30-50 | 40-80 | 1,200-4,000 | CSS Modules, Tailwind config, or styled-components |
| Type definition files | 15-25 | 60-100 | 900-2,500 | Shared types, API response shapes, prop interfaces |
| State management | 10-20 | 80-120 | 800-2,400 | Redux slices, Zustand stores, Context providers, custom hooks |
| API / data layer | 8-15 | 60-100 | 480-1,500 | React Query hooks, fetch wrappers, API client config |
| Config files | 8-12 | 20-80 | 160-960 | tsconfig, next.config, tailwind.config, eslint, postcss, package.json, .env |
| Test files | 30-50 | 80-150 | 2,400-7,500 | Component tests, hook tests, integration tests |
| **Total** | **~150-220** | | **~12,000-28,000** | |

**Equivalent Naze codebase:**

| File Category | Files | Avg Lines | Total Lines | Notes |
|---|---|---|---|---|
| Component files (.naze) | 50 | 40-80 | 2,000-4,000 | Structure + styling + state + events in one file; no import boilerplate |
| Theme file (theme.naze) | 1 | 20-40 | 20-40 | Design tokens |
| Config file (naze.toml) | 1 | 5-15 | 5-15 | Project metadata |
| Test files (.test.naze) | 30-50 | 30-60 | 900-3,000 | M20 testing framework |
| **Total** | **~52-102** | | **~3,000-7,000** | |

**Reduction: ~70-75% fewer lines of code, ~60-70% fewer files** for equivalent application functionality.

Why such a large difference? React requires separate files for styling (CSS), typing (TypeScript interfaces), state management (store definitions, context providers), data fetching (custom hooks), and configuration (build tools, linting, formatting). Each file category exists because of framework complexity, not application requirements. In Naze, a component's structure, styling, state, events, and data fetching are all expressed in a single `.naze` file — because the language was designed to be complete, not layered on top of a general-purpose runtime.

#### Token Cost per Interaction

AI models charge per token processed. Rough conversion: ~3-4 tokens per line of code (accounting for indentation, variable names, and syntax).

| Metric | React/Next.js | Naze | Reduction |
|---|---|---|---|
| Lines of code (medium app) | 12,000-28,000 | 3,000-7,000 | **70-75%** |
| Tokens per full codebase read | 40,000-100,000 | 10,000-25,000 | **~75%** |
| Cost per full read (~$3/M input tokens) | $0.12-0.30 | $0.03-0.08 | **~75%** |
| Cost per partial read (5-10 files for a bug fix) | $0.01-0.03 | $0.003-0.008 | **~75%** |

#### Cumulative Dollar Cost Over a Project's Lifetime

A production application typically sees 5-10 AI-assisted interactions per week — bug fixes, small features, code reviews, refactoring tasks. Over a multi-year maintenance window, context-loading costs compound:

| Scenario | React/Next.js | Naze | Savings |
|---|---|---|---|
| Per interaction (avg context load) | $0.05-0.15 | $0.01-0.04 | $0.04-0.11 |
| Per year (500 interactions) | $25-75 | $5-20 | **$20-55/year** |
| 3-year lifetime (1,500 interactions) | $75-225 | $15-60 | **$60-165** |
| Fleet of 10 apps, 3 years | $750-2,250 | $150-600 | **$600-1,650** |

These are input token costs only. Output tokens (AI-generated code and responses) are comparable between stacks. The cost differential is entirely in the **reading**, not the writing.

#### Cumulative Time Cost

Context loading isn't just a dollar cost — it's a **time cost**. Every AI interaction has a processing phase where the model reads and reasons over the loaded context. More tokens means longer processing time. Humans wait for every interaction.

| Metric | React/Next.js | Naze | Difference |
|---|---|---|---|
| Time per full context load | 15-45 seconds | 4-12 seconds | **~70% faster** |
| Time per partial read (5-10 files) | 5-15 seconds | 2-5 seconds | **~65% faster** |
| File search/discovery overhead | 3-8 seconds (150-220 files) | 1-3 seconds (52-102 files) | **~65% faster** |
| Total AI interaction (context + reasoning + response) | 30-90 seconds | 10-30 seconds | **~65% faster** |

Over a project's maintenance lifetime, this wait time accumulates:

| Scenario | React/Next.js | Naze | Time Saved |
|---|---|---|---|
| Per interaction (avg wait) | 45-90 sec | 15-30 sec | 30-60 sec |
| Per day (5-10 interactions) | 4-15 min waiting | 1-5 min waiting | **3-10 min/day** |
| Per year (500 interactions) | 6-12 hrs waiting | 2-4 hrs waiting | **4-8 hrs/year** |
| Per year (team of 5, 2,500 interactions) | 30-60 hrs waiting | 10-20 hrs waiting | **20-40 hrs/year** |
| 3-year lifetime (team, 7,500 interactions) | 90-180 hrs | 30-60 hrs | **60-120 hrs saved** |

That's **1.5-3 weeks of developer time** recovered over 3 years for a 5-person team — just from faster AI context loading. This doesn't account for accuracy improvements (fewer retries, fewer wrong suggestions to review and reject), which compound the time savings further.

#### Beyond Dollars and Time: Signal-to-Noise Ratio

The measurable costs (tokens, dollars, seconds) may actually understate the impact. The **qualitative** difference in AI comprehension is potentially more significant:

- **Signal density.** In a React codebase, a large portion of what the AI reads is boilerplate — import statements, TypeScript annotations, hook setup patterns, CSS class strings, build configuration. This is "noise" that conveys framework ceremony, not application intent. In Naze, almost every line conveys intent: what to render, what state to track, what data to fetch, how to respond to events. Higher signal density means better AI comprehension per token spent.

- **Context window utilization.** Current AI models have context windows of 100K-200K tokens. A large React application (28,000 lines = ~100K tokens) can consume the entire context window just to load the codebase, leaving minimal room for the AI to reason about changes, consider alternatives, or plan multi-step modifications. The equivalent Naze application uses ~25% of the window, leaving 75% for reasoning — enabling **whole-codebase understanding** that's physically impossible with larger React applications.

- **Reduced search overhead.** AI agents spend tokens and time searching for the right files before they can begin work. React's 150-220 files spread across components, styles, types, hooks, utilities, and config directories create a larger search space. Naze's 52-102 files (predominantly `.naze` components) are faster to navigate and less likely to lead the AI down irrelevant paths.

- **Single domain of knowledge.** To work effectively in a React/Next.js codebase, an AI must simultaneously reason about React hooks semantics, Next.js App Router conventions, Tailwind utility classes, TypeScript generic types, and build tool configuration. Each is a separate knowledge domain competing for the AI's reasoning capacity. Naze has one domain — the Naze language — freeing the AI to focus entirely on application logic.

- **Fewer retries.** Higher comprehension and simpler patterns mean the AI generates correct code on the first attempt more often. Each avoided retry saves a full context-loading cycle. If Naze reduces retries from ~30% (typical for complex React patterns) to ~10% (simpler, one-canonical-form patterns), that's an additional 20% reduction in effective token and time costs.

Over a project's maintenance lifetime, a Naze codebase costs **~75% less** in AI context-loading tokens and **~65% less in wall-clock time** than an equivalent React/Next.js codebase. For a single application, this saves tens to low hundreds of dollars and hours of cumulative wait time. For a team or agency maintaining a fleet of applications over multiple years, the savings reach thousands of dollars and weeks of recovered developer time. But the largest impact is qualitative: the AI understands Naze code better because it's reading intent, not ceremony — and it can fit the entire application in its context window, enabling whole-codebase reasoning that's impossible with larger React applications.

#### FAAD: Fully Autonomous AI Development

The estimates above model today's workflow: human developers using AI as an assistant, 5-10 interactions per week. But the trajectory of AI coding agents points toward a fundamentally different model — **Fully Autonomous AI Development (FAAD)**.

FAAD is a development paradigm where AI agents handle the entire software lifecycle — initial build, feature development, testing, debugging, code review, monitoring, and ongoing maintenance — with humans providing direction, requirements, and approval, not code. The human role shifts from *writing software* to *specifying intent and reviewing outcomes*.

This shift changes the fundamental **currency of software development**. In traditional development, the unit of cost is the **developer-hour** — framework complexity is paid in human learning time, debugging time, and maintenance time. Under FAAD, the unit of cost becomes the **AI compute hour** — measured in tokens processed. Framework complexity is paid in context-loading tokens (reading the codebase), output tokens (generating code), and retry tokens (recovering from incorrect generations). Every dollar spent, every feature shipped, every bug fixed traces back to how many tokens the AI must consume to accomplish a unit of work.

This reframing has a strategic implication: framework design should be optimized for **token efficiency** — minimum tokens to express intent, minimum tokens to build context — the same way cloud infrastructure is optimized for compute efficiency. Naze's design (one canonical form per concept, no boilerplate, styling and state co-located in the component file) is **token-optimized by construction**. React/Next.js, designed for human developers who accumulate experience across sessions, carries framework ceremony that humans learn to skim but AI must process in full on every interaction.

We formalize this efficiency gap as **Token Complexity Λ(n)** — a metric for measuring how AI token cost scales with application size, analogous to Big O notation for algorithms. Naze achieves **Λ-Linear** scaling (AI cost grows proportionally with app size), while React/Next.js exhibits **Λ-LogLinear** scaling (cost grows faster than app size due to cross-file dependencies). At 50 components, the difference is ~10x. At 200 components, ~15-20x. See [TOKEN_EFFICIENCY.md](TOKEN_EFFICIENCY.md) for the full framework, formula, and multi-language comparison.

In a FAAD workflow, the interaction count increases by an order of magnitude — from hundreds of AI-assisted interactions per year (a human developer asking for help) to tens of thousands of fully autonomous interactions per year (AI agents continuously building, testing, reviewing, debugging, and maintaining code around the clock). At this volume, the cost differential between framework choices stops being a line item and becomes a **dominant factor in total cost of ownership**.

**FAAD interaction model (single medium app):**

| Activity | Frequency | Interactions/Week | Notes |
|---|---|---|---|
| Feature development | Continuous | 30-80 | Multiple plan → build → test → refine cycles per feature |
| Bug fixes & debugging | Daily | 10-20 | AI triages, reproduces, fixes, verifies |
| Code review & validation | Per-change | 20-50 | Every AI-generated change reviewed by a second AI pass |
| Test generation & maintenance | Per-change | 15-30 | Write new tests, update existing, validate coverage |
| Refactoring & optimization | Weekly | 5-15 | Performance profiling, code cleanup, pattern updates |
| Dependency / security updates | Weekly (React) / Rare (Naze) | 5-10 (React) / 0-1 (Naze) | npm audit, upgrade, test compatibility |
| Documentation updates | Per-feature | 5-10 | API docs, changelogs, inline documentation |
| Monitoring & auto-fix | Continuous | 10-20 | Error detection, automated patches, health checks |
| **Total** | | **100-235/week** | **~5,000-12,000/year** |

Note that dependency updates alone add 250-500 interactions per year for React that Naze simply doesn't have — the `nazec` binary has no transitive dependencies to audit, upgrade, or test for compatibility.

**Full cost per interaction (input + output + retries):**

In a FAAD workflow, the AI generates all code — not just reads it. Output tokens (generated code) are also smaller for Naze because the syntax is more concise. And simpler patterns mean fewer retries:

| Cost Component | React/Next.js | Naze | Why Different |
|---|---|---|---|
| Input tokens (context loading) | 40K-100K per read | 10K-25K per read | ~75% less code to read |
| Output tokens (code generation) | Verbose: JSX + hooks + CSS + types | Concise: declarative syntax, inline styling | ~60-70% less code to generate |
| Retry rate | ~30% of interactions need a retry | ~10% need a retry | Fewer valid forms = fewer wrong guesses |

| Metric | React/Next.js | Naze | Reduction |
|---|---|---|---|
| Avg tokens per interaction (input + output) | 80K-200K | 20K-60K | **~70-75%** |
| Avg cost per interaction | $0.25-0.60 | $0.06-0.18 | **~70-75%** |
| Effective cost (including retries) | $0.32-0.78 | $0.07-0.20 | **~75-80%** |

**Cumulative dollar cost — FAAD scenario:**

| Scenario | React/Next.js | Naze | Savings |
|---|---|---|---|
| Per year, single app (~8,000 interactions) | $2,600-6,200 | $560-1,600 | **$2,000-4,600/year** |
| 3-year lifetime, single app (~24,000) | $7,800-18,600 | $1,680-4,800 | **$6,100-13,800** |
| 3-year lifetime, 10 apps (~240,000) | $78,000-186,000 | $16,800-48,000 | **$61,000-138,000** |
| Enterprise: 50 apps, 5 years (~2M) | $650,000-1,550,000 | $140,000-400,000 | **$510,000-1,150,000** |

**Cumulative time cost — FAAD throughput and velocity:**

Under FAAD, "time" isn't human wait time — it's **throughput**. How many features can the AI ship per day? Faster context loading means more iterations per hour, which means faster development velocity:

| Metric | React/Next.js | Naze | Impact |
|---|---|---|---|
| Avg time per interaction | 30-90 sec | 10-30 sec | **2-3x more iterations per hour** |
| Interactions per 8-hr compute day | 320-960 | 960-2,880 | Naze AI agent is 2-3x more productive |
| Time to build a medium app (from scratch) | 3-8 days of AI compute | 1-3 days | **60-65% faster** |
| Time to complete an average feature | 15-45 min | 5-15 min | **65-70% faster** |

| Scenario | React/Next.js | Naze | Time Saved |
|---|---|---|---|
| AI compute hours/year, single app | 200-500 hrs | 55-140 hrs | **145-360 hrs/year** |
| AI compute hours/year, 10 apps | 2,000-5,000 hrs | 550-1,400 hrs | **1,450-3,600 hrs/year** |
| 3-year lifetime, 10 apps | 6,000-15,000 hrs | 1,650-4,200 hrs | **4,350-10,800 hrs** |

Faster iterations mean features ship sooner, bugs are fixed faster, and the feedback loop between "request a change" and "change deployed" shrinks from hours to minutes.

**The FAAD compounding effect.** Under FAAD, the framework choice becomes an infrastructure decision analogous to choosing cloud instance sizes. A React/Next.js codebase is a larger, slower, more expensive machine to operate. A Naze codebase is a smaller, faster, cheaper machine that produces equivalent output.

The compounding is multiplicative: smaller codebase → faster reads → more iterations per hour → fewer retries → higher accuracy → less debugging → even fewer total interactions needed. Over tens of thousands of interactions, Naze's structural simplicity creates a virtuous cycle, while React's framework complexity creates a drag that worsens as the codebase grows.

**Why framework choice matters more under FAAD.** When humans write code, framework complexity is absorbed by developer skill and experience — a senior React developer "just knows" the hook patterns, the Tailwind classes, the Next.js conventions. Under FAAD, that complexity is absorbed by token costs and error rates. An AI agent doesn't accumulate experience across sessions — every interaction starts fresh from context. The most AI-efficient framework is the one that requires the least context to understand and the fewest tokens to express intent. Naze was designed for exactly this from day one: one canonical form per concept, maximum intent per token, minimum ceremony per interaction.

---

## Conclusion

### Component Parity: Strong

Naze's spec provides solid foundational primitives for component model, state management, layout, accessibility, and basic styling. The primary architectural gap — the **overlay/floating layer** — is addressed by M19b, which is the single highest-leverage addition (51% to 80% parity). Adding M19c (visual properties) brings it to 88%. Combined with already-planned milestones (M15 pipelines, M17 responsive), Naze reaches **92% component parity** — effectively feature-complete for building production component libraries equivalent to shadcn/ui.

### Application Parity: Near-Complete

M19d (application logic primitives) closes the gap from **~40% to ~82%** with ~6 new grammar rules. M19e (JS interop, device APIs, textarea) adds ~3 more rules to reach **~85%**. Combined with already-planned M15 (pipelines), M23 (WASM imports), and M24 (server functions), Naze reaches **~99% application logic parity**.

The key insight is the **reactivity boundary**: Naze adds primitives only for features that must integrate with the rendering loop. The `js` interop provides a controlled escape hatch for the JS ecosystem without making Naze a general-purpose language. Total grammar expansion across M19d + M19e: **~9 new rules** — still less than half of Phase 2's grammar scope.

**What Naze can build with all primitives:**
- Marketing sites, landing pages, content sites
- Dashboards with real-time data (`data` + `stream` + `computed` + charts via WASM)
- E-commerce (`shared state` for cart, `storage` for persistence, server functions for payments, `js` interop for Stripe if needed)
- CRUD applications (full HTTP, form validation, file uploads, `textarea` for multi-line input)
- Chat and messaging (`stream` for WebSocket, `timer` for typing indicators, `notify` for push, file uploads for media)
- Location-aware apps (`device geolocation`, reactive map data)
- Search-driven apps (`param` for URL state, `debounce` on input, reactive `data` fetching)
- Apps with third-party JS SDKs (`js` interop for analytics, payments, maps, auth providers)

**What remains out of scope (~1%):**
- **Rich text editing** — ProseMirror, Slate, Tiptap require `contentEditable` DOM. Canvas2D rendering has no equivalent. Would need a hybrid DOM rendering mode (not currently planned).
- **Collaborative editing** — CRDTs (Yjs, Automerge) plus conflict resolution. `stream` handles transport, but the CRDT logic requires WASM imports for complex algorithms.
- **Embedding complex JS DOM widgets** — Google Maps, embedded video players, Monaco editor render their own DOM inside the page. JS interop can call their APIs, but embedding their visual output inside a Canvas2D app requires a hybrid rendering approach (not yet specced).

**Bottom line:** With M19d + M19e, Naze reaches **~99% application logic parity** — covering virtually every app category that a modern Next.js/React app handles. An AI can generate a full-stack app (frontend + data fetching + state management + real-time + persistence + JS SDK integration + device access) entirely in `.naze` files. The ~9 new grammar rules replace dozens of JS/React/Next.js patterns, each with one canonical form — making AI generation dramatically simpler than the equivalent JS stack.
