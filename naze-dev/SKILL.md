---
name: naze-dev
description: "Generate and edit Naze UI applications (.naze files). Use when user asks to create naze apps, write .naze code, build naze components, or work with the nazec CLI. Covers layout, state, events, routing, data fetching, and theming."
---

# Naze Development Skill

Naze is a declarative UI language that compiles `.naze` files to Canvas2D via WASM. There is no DOM, no CSS, no JavaScript. Everything renders through a custom layout engine to a canvas. One canonical form per concept. Kebab-case identifiers. Comments use `--`.

## Quick Syntax

Every entry file wraps in `app "Title" { ... }`. Layout uses `row` (horizontal), `column` (vertical), `container` (styled box). Content uses `text`, `heading`, `image`, `code`. Inputs use `input bind: var`, `checkbox bind: var`, `select bind: var`.

State is declared with `state name = value` and mutated via `on click: set name = expr`. Constants use `let`. Derived values use `computed name = expr`.

Props use colon syntax: `width: 200px`. Colors are unquoted hex: `color: #2563eb`. Units: `px`, `%`, `em`.

## Minimal Counter App

```naze
app "Counter" {
  state count = 0

  column padding: 20px, gap: 16px {
    heading "Counter"
    text "Count: {count}"
    rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
      text "Increment" color: #fff
      on click: set count = count + 1
    }
    rect width: 120px, height: 40px, color: #dc2626, radius: 8px {
      text "Reset" color: #fff
      on click: set count = 0
    }
  }
}
```

## Key Anti-Patterns (Do NOT)

- No HTML tags -- use `text`, `heading`, `row`, `column`, `rect`
- No CSS or `style` -- all styling is inline props (`color`, `font-size`, `padding`)
- No semicolons -- newlines separate statements
- No quoted colors -- `color: #2563eb` not `color: "#2563eb"`
- No `className`, `div`, `span` -- no DOM concepts exist
- No `var`/`const` -- use `state` (mutable) or `let` (constant)
- No camelCase -- use `kebab-case` for identifiers (`font-size`, `my-var`)
- No `=` in props -- use colon: `width: 200px` not `width=200px`
- `bind:` is required on inputs -- `input bind: name` not `input placeholder: "..."`
- `text` is a leaf element -- it cannot have children
- `!var` does NOT work for negation -- use `var == false`
- `each` does NOT work inside `match` arms -- use `if` blocks instead
- Interpolation only works inside `"..."` strings: `"Count: {count}"`

## Common Workflow

```bash
nazec new my-app     # Scaffold project (naze.toml + app.naze)
cd my-app
nazec dev            # Dev server with hot reload (default port 3000)
nazec build          # Compile to dist/ (index.html + WASM + app_data.bin)
nazec check          # Type-check without building
nazec test           # Run .test.naze test suites
```

Build output goes to `dist/` and must be served over HTTP (WASM requires it, not `file://`).

## Elements Reference

**Layout:** `row`, `column`, `container`, `stack`, `grid`, `spacer`, `scroll`, `separator`
**Content:** `text`, `heading`, `link`, `code`, `image`, `rect`
**Input:** `input`, `textarea`, `checkbox`, `radio`, `select` (with `option`)
**Overlay:** `overlay`

Common props: `width`, `height`, `padding`, `gap`, `color`, `radius`, `border`, `shadow`, `gradient`, `opacity`, `transition`, `animate`

## State and Reactivity

```naze
state count = 0                          -- mutable, reactive
shared state user = ""                   -- persists across pages
computed total = items | count           -- derived, read-only
let label = "Hello"                      -- compile-time constant
storage theme: local "theme" default: "light"  -- browser storage
param page: number default: 1           -- URL query param
timer tick: every 1s { set count = count + 1 }  -- periodic
```

## Events and Actions

Events: `click`, `hover`, `change`, `keypress`, `scroll`, `drag-start`, `drop`, `click-outside`, `pointer-move`, `arrow-up`, `arrow-down`

Actions: `set var = expr`, `navigate "/path"`, `append item to list`, `remove index from list`, `trigger data-name`, `log "msg"`, `copy "text"`, `notify "Title" { body: "text" }`, `set-theme "name"`, `start timer`, `stop timer`, `js "func"(args)`, `emit event`

Modifiers: `on click debounce 300ms: action`, `on scroll throttle 100ms: action`

## Components

Define one per file:
```naze
-- components/card.naze
component card(title: text, color: color = #f3f4f6) {
  container padding: 16px, color: color, radius: 8px {
    heading "{title}" font-size: 18px
    slot
  }
}
```

Use with import:
```naze
use components/card
app "Demo" {
  card title: "Hello" {
    text "Card content goes in the slot"
  }
}
```

## Pages and Routing

```naze
app "Site" {
  row padding: 16px, color: #1e293b {
    link "Home", to: "/"
    link "About", to: "/about"
  }
  page "/" {
    text "Home page"
  }
  page "/about" {
    text "About page"
  }
}
```

## Data Fetching

```naze
data posts: fetch "https://api.example.com/posts"

if posts.loading { text "Loading..." }
if posts.error { text "{posts.error}" color: #dc2626 }
if posts.data {
  each post in posts.data {
    text "{post.title}"
  }
}
```

All data sources produce `.loading`, `.error`, `.data` states automatically.

## Server Functions and Database Models

```naze
model users {
  id number primary
  name text
  email text
}

server function list-users() {
  find users order id desc
}

server function add-user(name: text, email: text) {
  insert users {name: name, email: email}
}
```

Server functions and models must be at the top level, outside `app` blocks.

## Theming

```naze
theme light {
  colors { primary: #2563eb, bg: #ffffff }
  spacing { md: 16px }
}
theme dark extends light {
  colors { primary: #60a5fa, bg: #0f172a }
}
```

Reference tokens: `theme.colors.primary`, `theme.spacing.md`. Switch: `on click: set-theme "dark"`.

## Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| "expected app_block" | Missing `app "Title" { }` wrapper | Wrap all content in an app block |
| "unresolved binding" | Using undeclared state variable | Add `state name = value` |
| "input requires bind" | Input without `bind:` prop | Add `bind: variable-name` |
| "expected NEWLINE" | Single-line if inside app block | Use multi-line if/else format |
| Colors not showing | Quoted hex color | Remove quotes: `#2563eb` not `"#2563eb"` |
| Component not found | Missing `use` import | Add `use path/component-name` at top |

## Reference Files

- `references/language.md` -- Complete syntax reference (all elements, props, expressions)
- `references/examples.md` -- Curated code examples by category
- `references/cli.md` -- nazec command reference and flags
