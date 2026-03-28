# Responsive Design in Naze

How Naze apps adapt to different screen sizes — mobile, tablet, desktop — while maintaining sigma=1 (all information in one file).

## What Exists Today

Two per-element properties, both triggered by viewport width:

```naze
-- Row becomes a column below 768px
row responsive: 768 {
  column grow: 1 { text "Main" }
  column width: 280 { text "Sidebar" }
}

-- Element hides below 1200px
column collapsible: 1200 {
  text "Extra detail (wide screens only)"
}

-- Grid collapses to 1 column below 768px
grid columns: 3, responsive: 768 {
  rect width: 100, height: 80, color: #2563eb
  rect width: 100, height: 80, color: #22c55e
}
```

The layout engine (`crates/naze-layout/src/lib.rs`) checks `viewport_w < breakpoint` and switches behavior:
- `responsive` on `row` → measure/layout as column
- `responsive` on `grid` → force 1 column
- `collapsible` → return (0, 0) size

### Limitations

- **No prop overrides** — can't change font size, padding, gap, or color at a breakpoint
- **No content swapping** — can't show different elements on mobile vs desktop
- **No named breakpoints** — each element hardcodes a pixel value
- **No device targeting** — no concept of "mobile" vs "tablet" vs "desktop"
- **Only two behaviors** — row→column flip and hide/show. No middle ground.

Most modern frameworks (Tailwind, SwiftUI, Flutter) let you adapt *any* property to screen size. Naze only adapts layout direction and visibility.

---

## Design Goals

1. **sigma=1** — all responsive behavior in the same file, right next to the element it modifies. No separate stylesheets, no breakpoint files, no media query imports.
2. **One canonical form** — one way to express responsive behavior, not three equivalent alternatives. Keeps r (retry rate) low.
3. **Minimal grammar cost** — extend existing prop/element patterns, don't invent new block types.
4. **AI-friendly** — an LLM generating a responsive layout should need zero cross-file context and produce valid output on the first try.
5. **Named breakpoints** — `mobile`, `tablet`, `desktop` instead of magic numbers. Defaults that work without declaration.

---

## Option A: Breakpoint-Suffixed Props

Suffix any prop name with `@breakpoint` to override it at that screen size:

```naze
app "Dashboard"
  column padding: 24, padding@mobile: 12, gap: 16, gap@mobile: 8
    row gap: 16, responsive: 768
      column grow: 1
        text "Hello" size: 32, size@mobile: 20, size@tablet: 26
      column width: 300, hidden@mobile: true
        text "Sidebar"
```

Named breakpoints with sensible defaults:

```naze
breakpoints
  mobile 640
  tablet 1024
  desktop 1280
```

If no `breakpoints` block is declared, the defaults apply. The breakpoint means "at or below this width."

### Grammar Rules

```pest
// Extend prop to allow optional @breakpoint suffix
prop = { prop_name ~ ":" ~ value }
prop_name = @{ ident ~ ("@" ~ ident)? }

// New top-level statement for custom breakpoints
breakpoint_def = { "breakpoints" ~ NEWLINE_CHAR ~ breakpoint_entry+ }
breakpoint_entry = { ident ~ number_lit ~ NEWLINE_CHAR }
```

Grammar impact: **2 modified rules, 2 new rules.** `prop_name` changes from plain `ident` to `ident ~ ("@" ~ ident)?`. The `breakpoint_def` and `breakpoint_entry` rules are new top-level statements.

### Pros

- Minimal syntax — one character (`@`) extends existing props
- Zero new block types — all behavior is inline
- Excellent sigma=1 — the mobile override is right next to the desktop value
- Low grammar cost — 2 new rules, follows existing patterns
- Easy for AI — generating `size: 32, size@mobile: 20` is trivial pattern extension
- Composable with existing features — `responsive` and `collapsible` still work alongside

### Cons

- Gets noisy when many props change per breakpoint:
  ```naze
  column padding: 24, padding@mobile: 12, padding@tablet: 16, gap: 16, gap@mobile: 8, width: 400, width@mobile: 100%, width@tablet: 300
  ```
  A single element with 3 breakpoints x 3 props = 9 prop declarations on one line
- No way to show/hide *different* content per breakpoint (only override props on the same element)
- The `@` character in prop names may conflict with future syntax (e.g., `@org/package` import paths already use `@`)

### Conflict Note

The `@` character is already used in `use_path` for scoped packages (`use @org/package/component`). However, `prop_name` and `use_path` are in different grammar contexts (props appear inside elements, `use` appears at statement level), so there's no actual ambiguity in PEG parsing. Still, using `@` for two different purposes may confuse humans reading the grammar.

---

## Option B: `when` Blocks

A scoped block that applies overrides at a named breakpoint:

```naze
app "Dashboard"
  column padding: 24, gap: 16
    row gap: 16
      column grow: 1
        text "Hello" size: 32
      column width: 300
        text "Sidebar"

  when mobile
    column padding: 12, gap: 8
      row gap: 8
        text "Hello" size: 20
        column hidden: true

  when tablet
    text "Hello" size: 26
```

The `when` block mirrors the element tree structure and overrides specific props. Elements are matched by position in the tree (1st column, 1st row, 2nd column, etc.).

### Grammar Rules

```pest
// New statement type
when_block = { "when" ~ ident ~ block }

// Added to statement rule
statement = _{ ... | when_block | ... }
```

Grammar impact: **2 new rules.** Simple additions.

### Pros

- Clean visual separation — the base layout reads without clutter, overrides are grouped
- Familiar to CSS developers (`@media` block analogy)
- Easy to scan — "what changes on mobile?" is answered by reading one block

### Cons

- **Breaks sigma=1 *spirit*** — technically same file, but the mobile behavior for an element is disconnected from the element itself. An AI generating a text element has to look in two places to understand its full behavior.
- **Element matching is fragile** — matching by tree position means inserting a new element above shifts all the overrides. Matching by element name/id would require adding identifiers to elements (a grammar change with its own cost).
- **Duplication risk** — the `when` block partially re-declares the tree structure, which the AI might get wrong (mismatched nesting depth = silent bugs).
- **Higher r (retry rate)** — the AI must keep the `when` block tree structure synchronized with the main tree. If the main tree changes, the `when` block may become stale.

---

## Option C: Inline Variant Lines

Each element can have variant lines directly beneath it — indented child lines prefixed with a breakpoint name:

```naze
app "Dashboard"
  column padding: 24, gap: 16
    @mobile: padding: 12, gap: 8
    @tablet: padding: 16

    row gap: 16, responsive: 768
      column grow: 1
        text "Hello" size: 32
          @mobile: size: 20
          @tablet: size: 26
      column width: 300
        @mobile: hidden: true
        text "Sidebar"
```

A variant line is a child of the element it modifies. It contains only prop overrides, not new elements.

### Grammar Rules

```pest
// Variant line as a new statement type (appears inside blocks)
variant_line = { "@" ~ ident ~ ":" ~ inline_props ~ NEWLINE_CHAR }

// Added to statement rule
statement = _{ ... | variant_line | ... }

// breakpoint_def same as Option A
breakpoint_def = { "breakpoints" ~ NEWLINE_CHAR ~ breakpoint_entry+ }
breakpoint_entry = { ident ~ number_lit ~ NEWLINE_CHAR }
```

Grammar impact: **3 new rules.** The `@ident:` prefix distinguishes variant lines from regular elements.

### Pros

- **Best sigma=1 of all options** — the mobile override is directly under the element, 1 line away
- No element matching problem — the variant modifies its parent, no positional ambiguity
- Reads naturally — "this column has padding 24, on mobile padding 12"
- Supports hide/show via `hidden: true` prop override
- AI-friendly — generating a variant line is a trivial extension of generating a prop line

### Cons

- Adds depth to the indentation tree (variant lines are children, which may confuse the indentation-sensitive parser if Naze ever moves to indentation-based blocks)
- `@` prefix reuse (same concern as Option A)
- Cannot show *entirely different* elements per breakpoint (only override props) — but this is arguably a feature (keeps the tree structure stable)
- Slightly more verbose than Option A for single-prop overrides (`@mobile: size: 20` vs `size@mobile: 20`)

---

## Option D: Hybrid (A + C)

Use `@breakpoint` suffixes for simple one-prop overrides, and `@breakpoint:` variant lines for multi-prop overrides:

```naze
app "Dashboard"
  column padding: 24, gap: 16
    @mobile: padding: 12, gap: 8

    row gap: 16, responsive: 768
      column grow: 1
        text "Hello" size: 32, size@tablet: 26
          @mobile: size: 20, bold: false, color: #666666
      column width: 300, hidden@mobile: true
        text "Sidebar"
```

The rule: if you're overriding 1 prop, use the suffix. If you're overriding 2+, use a variant line.

### Grammar Rules

Combines both:

```pest
prop_name = @{ ident ~ ("@" ~ ident)? }
variant_line = { "@" ~ ident ~ ":" ~ inline_props ~ NEWLINE_CHAR }
breakpoint_def = { "breakpoints" ~ NEWLINE_CHAR ~ breakpoint_entry+ }
breakpoint_entry = { ident ~ number_lit ~ NEWLINE_CHAR }
```

Grammar impact: **~4 rules total.**

### Pros

- Ergonomic for both simple and complex cases
- No verbosity penalty for single-prop overrides
- No noise penalty for multi-prop overrides
- Full expressive power

### Cons

- **Two canonical forms** for the same concept — directly violates "one way to express each concept." An AI must choose between `size@mobile: 20` and `@mobile: size: 20`. Both are valid. This increases r.
- More grammar surface area
- Documentation must explain when to use which form

---

## Comparison

| Criterion | A (Suffix) | B (when) | C (Inline) | D (Hybrid) |
|---|---|---|---|---|
| **sigma** | 1 (inline) | ~1 (same file, different location) | 1 (directly under element) | 1 |
| **r (retry)** | Low (one form) | Medium (tree sync) | Low (one form) | Medium (two forms) |
| **Grammar rules** | +2 | +2 | +3 | +4 |
| **Readability (few overrides)** | Excellent | Good | Good | Excellent |
| **Readability (many overrides)** | Noisy | Clean | Good | Good |
| **Content swapping** | No | Partial | No | No |
| **AI generation** | Trivial | Error-prone | Easy | Needs choice |
| **Tier** | 0 (Core UI) | 0 | 0 | 0 |

---

## Recommendation

**Option C (Inline Variant Lines)** is the strongest fit for Naze's design principles:

1. **Sigma=1 is maximized** — variant behavior is 1 line below the element, not across the file
2. **One canonical form** — there's exactly one way to express a responsive override
3. **Low r** — no tree synchronization, no choice between forms, no positional matching
4. **Natural extension** — variant lines look like child statements, which is already how Naze structures elements
5. **AI-friendly** — generating `@mobile: size: 20` under a `text` element is a trivial pattern

The `@` prefix creates a clear visual marker distinguishing variant lines from elements, props, and event handlers. The `:` after the breakpoint name separates it from regular `@org/package` import syntax.

### Default Breakpoints

These would be built-in (no declaration needed for the common case):

| Name | Max Width | Typical Devices |
|---|---|---|
| `mobile` | 640px | Phones |
| `tablet` | 1024px | Tablets, small laptops |
| `desktop` | 1280px | Laptops, monitors |
| `wide` | 1536px | Large monitors |

Override with a `breakpoints` block if needed. Most apps never declare one.

### Interaction with Existing Features

- `responsive: 768` on rows/grids still works — it's a layout behavior, not a prop override
- `collapsible: 1200` still works — but `@desktop: hidden: true` is now an alternative
- Variant lines are strictly prop overrides — they don't add or remove elements from the tree
- Themes apply to the base props; variants override after theme resolution

### Full Example

```naze
app "Responsive Dashboard"
  column padding: 32, gap: 24
    @mobile: padding: 12, gap: 12
    @tablet: padding: 20, gap: 16

    text "Dashboard" size: 36, bold: true
      @mobile: size: 24
      @tablet: size: 28

    row gap: 24, responsive: 768
      column grow: 2
        text "Revenue" size: 20
          @mobile: size: 16
        text "$42,000" size: 48, bold: true
          @mobile: size: 28

      column grow: 1
        @mobile: hidden: true
        text "Breakdown" size: 20
        text "Details visible on larger screens"

    grid columns: 4, responsive: 1024, gap: 16
      @mobile: gap: 8
      rect height: 120, color: #2563eb
      rect height: 120, color: #22c55e
      rect height: 120, color: #f59e0b
      rect height: 120, color: #ef4444
```

At 1280px+ (desktop): 4-column grid, sidebar visible, large text.
At 800px (tablet): 4-column grid, sidebar visible, medium text.
At 500px (mobile): 1-column grid (via `responsive: 1024`), sidebar hidden, small text, tighter spacing.

---

## Why No User-Agent

Traditional browsers use user-agent strings because the rendering engine (Blink, WebKit, Gecko) doesn't expose device characteristics directly. CSS `@media` queries work around this — they're a proxy layer. Developers parse user-agent strings to detect mobile vs desktop because the browser won't just tell them.

Naze controls the entire rendering stack. The layout engine already receives `viewport_w` and `viewport_h` as direct parameters — there's no CSS engine in between, no DOM, no abstraction to peer through. The runtime *is* the renderer.

What user-agent detection actually solves, and how Naze handles each:

| Traditional (user-agent) | Naze (direct) |
|---|---|
| Screen size | `viewport_w`, `viewport_h` — passed to layout engine every frame |
| Input method (touch vs mouse) | Runtime knows its input surface directly |
| Device capabilities (camera, GPS) | Already exposed via device APIs (M40) |
| Browser engine quirks | Not applicable — Naze is the engine |

The breakpoint system (`@mobile`, `@tablet`, etc.) triggers on `viewport_w` because that's the actual signal. No string parsing, no lookup tables, no lie-prone sniffing. When the Naze Browser (web surface or desktop surface) starts, it passes the canvas dimensions to the layout engine. When the window resizes, it re-layouts. That's it.

If input method ever needs to be a distinct signal (a 1024px tablet with touch input vs a 1024px laptop with a mouse), it could be a separate variant axis:

```naze
@touch: padding: 16    -- larger tap targets
@pointer: padding: 8   -- tighter mouse targets
```

But viewport width covers the vast majority of responsive needs, and it's the primitive the layout engine already operates on. No new detection mechanism needed.

---

## Open Questions

1. **Should variant lines support `visible: true`?** — For elements that are hidden by default and shown only on mobile (e.g., a hamburger menu). Currently `hidden` hides; there's no inverse.

2. **Orientation?** — `@portrait` and `@landscape` in addition to width breakpoints? Mobile apps often care about rotation.

3. **Should breakpoints be "at or below" or "between ranges"?** — CSS uses ranges (`min-width` / `max-width`). Naze's current `responsive` uses "below." If `@mobile` means "640px and below" and `@tablet` means "641px to 1024px," that's range-based. If both mean "at or below their threshold," `@tablet` overrides include mobile too, which may be confusing.

4. **Nesting?** — Should `@mobile: @dark-theme: color: white` be valid? Combining responsive + theme variants. Probably not for the MVP — one axis at a time.

5. **Integration with `each` and `if`?** — Can a variant line appear inside an `each` loop or `if` block? Probably yes — it modifies the nearest parent element regardless of control flow context.
