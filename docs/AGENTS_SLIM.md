# Naze Language — Condensed AI Reference

Naze is a declarative UI language that compiles `.naze` files to Canvas2D via WASM — no DOM. One canonical form per concept. Kebab-case identifiers. Comments: `--`.

## App Structure

Every entry file: `app "Title" { ... }`. The string becomes the page title.

## Types & Values

| Type | Examples |
|------|---------|
| number | `20px`, `50%`, `2em`, `100`, `0.5` |
| text | `"Hello"`, `"Count: {count}"` (interpolation) |
| color | `#fff`, `#2563eb`, `#00000080` |
| bool | `true`, `false` |
| list | `["a", "b"]`, `[1, 2, 3]` |
| object | `{ name: "Alice", age: 30 }` |

Units: `px`, `%`, `em`. Duration: `ms`, `s`. Expressions: `+ - * / % == != > < >= <= && || !`. Member access: `user.name`. Inline conditional: `if active { #3b82f6 } else { #94a3b8 }`.

## Layout Elements

- `row` — horizontal. Props: `gap`, `padding`, `width`, `height`, `color`, `radius`, `align`, `justify`, `wrap`, `flex-grow`, `responsive`, `collapsible`
- `column` — vertical. Same props as row
- `container` — styled box (vertical). Extra: `border`, `border-color`, `shadow`, `gradient`, `transition`
- `stack` — overlapping layers
- `grid` — wrapping grid. Props: `columns`, `gap`
- `spacer` — fills remaining space
- `scroll` — scrollable container. Props: `height`
- `separator` — horizontal line

## Content Elements

- `text "content"` — body text. Props: `color`, `font-size`, `font-weight`, `font-style`, `align`, `text-decoration`, `text-overflow`, `line-height`
- `heading "content"` — heading (24px bold). Same props as text
- `link "text", to: "/path"` — navigation link
- `code "content"` — monospace text
- `image src: "url", width: N, height: N, fit: "cover", alt: "desc"`
- `rect` — colored rectangle/button. Props: `width`, `height`, `color`, `radius`, `border`, `border-color`, `opacity`, `shadow`, `gradient`, `transform`, `transition`, `animate`

## Input Elements

- `input bind: var, placeholder: "text"` — `bind` required. Types: `"email"`, `"number"`, `"password"`, `"file"`
- `textarea bind: var, placeholder: "text", rows: N`
- `checkbox bind: var, label: "text"`
- `radio bind: var, value: "val", label: "text"`
- `select bind: var { option "Label" value: "val" }`
- Validation: `validate: { required: true, min-length: N, max-length: N, pattern: "regex" }`. Auto state: `{name}_valid`, `{name}_error`

## Overlay

`overlay` wraps modal content. Props: `focus-trap`, `scroll-lock`, `dismiss-on-escape`. Use `on click-outside:` to dismiss.

## State & Reactivity

- `state name = value` — mutable, reactive, page-scoped
- `shared state name = value` — persists across pages
- `computed name = expression` — read-only derived value
- `let name = value` — compile-time constant
- `storage name: local "key" default: value` — persistent browser storage (`local`/`session`)
- `param name: type default: value` — URL query binding
- `timer name: every Ns { action }` / `timer name: after Ns { action }`

## Events & Actions

Events: `click`, `hover`, `change`, `keypress`, `scroll`, `drag-start`, `drag-over`, `drop`, `click-outside`, `context-menu`, `pointer-move`, `arrow-up`, `arrow-down`. Modifiers: `debounce Nms`, `throttle Nms`.

Actions: `set var = expr`, `navigate "/path"`, `scroll-to "id"`, `log "msg"`, `trigger data-name`, `copy "text"`, `send stream msg`, `js "func"(args)`, `js "func"(args) -> var`, `notify "Title" { body: "text" }`, `emit event`, `set-theme "name"`, `start timer`, `stop timer`.

## Conditionals & Iteration

`if cond { ... } else if cond { ... } else { ... }`. Falsy: `false`, `0`, `""`, `null`.

`each item in collection { ... }`. Supports pipeline: `each x in items | filter score > 80 | sort-by name { ... }`.

## Pipeline Operators

`filter cond`, `map field`, `sort-by field`, `take N`, `sum`, `count`, `reduce expr init`, `group-by field`, `flatten`, `distinct`.

## Pattern Matching

`match expr { "val": element, _: element }`. Arms: string, number, bool, identifier, `_` wildcard.

## Functions

`function name(param: type) -> type { expr }` — pure, compile-time inlined, single expression body.

## Components

Define (one per file): `component name(prop: type = default) { ... }`. Use: `use path/name` then `name prop: value`. Slots: `slot` / `slot "name" { default }`. Fill: `fill "name" { content }`. Events: `emit event-name` / `on event: action`.

## Pages & Routing

`page "/path" { ... }` inside app block. Elements outside pages render on every page. `link "text", to: "/path"`.

## Data Fetching

All sources produce `.loading`, `.error`, `.data`.

- `data name: fetch "url"` — GET request
- `data name: fetch "url" { method: post, headers: {...}, body: {...}, trigger: manual }` — enhanced
- `data name: stream "wss://url"` — WebSocket
- `data name: js "func"(args) { trigger: manual }` — JS interop
- `data name: device "geolocation"` — device APIs

## Server Functions

`server function name(params) { fetch "url" }` — server-side execution. Call: `on click: call server.name(args)`. Data: `data x: name(args)`.

## Theming

`theme name { colors { key: #hex } spacing { key: Npx } }`. `theme dark extends light { ... }`. Refs: `theme.colors.primary`. Switch: `set-theme "name"`.

## Animation

Transition: `transition: "prop Nms easing"`. Keyframe: `animate: "prop [v1, v2] Ns easing infinite"`. Spring: `animate: "prop spring(stiffness, damping)"`. Easing: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`.

## Visual Props

Shadow: `shadow: "sm|md|lg|xl"`. Gradient: `gradient: "linear(to right, #hex, #hex)"`. Transform: `transform: "rotate(Ndeg)|scale(N)"`. Cursor: `cursor: "pointer"`.

## Accessibility

Props: `role`, `label`, `tab-index`, `id`. Compiler warns if interactive elements lack `role`/`label`.

## Complete Examples

### Counter with Reset

```naze
app "Counter" {
  state count = 0

  column padding: 20px, gap: 16px {
    heading "Counter"
    text "Count: {count}"

    row gap: 8px {
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
}
```

### Data Dashboard

```naze
app "Dashboard" {
  data stats: fetch "/api/stats"

  column gap: 0px {
    container padding: 16px, color: #1e293b {
      heading "Dashboard" font-size: 20px, color: #fff
    }

    column padding: 20px, gap: 16px {
      if stats.loading {
        text "Loading..."
      }
      if stats.data {
        row gap: 16px {
          container padding: 16px, color: #eff6ff, radius: 8px, width: 180px {
            text "Revenue"
            heading "$12,345" font-size: 24px
          }
          container padding: 16px, color: #f0fdf4, radius: 8px, width: 180px {
            text "Users"
            heading "1,234" font-size: 24px
          }
        }
      }
    }
  }
}
```

### Form with Validation

```naze
app "Sign Up" {
  state username = ""
  state email = ""

  column padding: 20px, gap: 12px {
    heading "Create Account"

    input bind: username, placeholder: "Username", validate: { required: true, min-length: 3 }
    if username_error {
      text "{username_error}" color: #dc2626
    }

    input bind: email, type: "email", placeholder: "Email", validate: { required: true }
    if email_error {
      text "{email_error}" color: #dc2626
    }

    if username_valid {
      if email_valid {
        rect padding: 12px, color: #2563eb, radius: 8px {
          text "Submit" color: #fff
        }
      }
    }
  }
}
```

### Multi-Page App

```naze
app "Blog" {
  shared state logged-in = false

  row padding: 16px, gap: 16px, color: #1e293b {
    heading "Blog" color: #fff, font-size: 18px
    link "Home", to: "/"
    link "About", to: "/about"
  }

  page "/" {
    data posts: fetch "/api/posts"
    column padding: 20px, gap: 12px {
      heading "Latest Posts"
      if posts.data {
        each post in posts.data {
          text "{post.title}" font-size: 18px
        }
      }
    }
  }

  page "/about" {
    column padding: 20px {
      heading "About"
      text "A simple blog built with Naze."
    }
  }
}
```

## Anti-Patterns

- No HTML tags — use `text`, `heading`, `row`, `column`, `rect`, etc.
- No CSS/`style` — all styling is inline props (`color`, `font-size`, `padding`)
- No semicolons — newlines separate statements
- No quoted colors — `color: #2563eb` not `color: "#2563eb"`
- `bind:` required on inputs — `input bind: name`
- Use `kebab-case` — not camelCase
- `text` is a leaf — no children
- Every entry file needs `app "Title" { ... }`
- No `className`, `div`, `span` — no DOM concepts
- State in app/page scope, not component files — pass as props
- Props use colon: `width: 200px` not `width=200px`
- Interpolation only inside `"..."` strings
- Use `state` (mutable) or `let` (constant) — not `var`/`const`
