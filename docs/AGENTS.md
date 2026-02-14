# Naze Language — AI Agent Reference

Naze is a declarative UI language that compiles `.naze` files to Canvas2D via WASM — no DOM. One canonical form per concept. Kebab-case identifiers. Comments start with `--`.

## Minimal App

```naze
app "Hello" {
  text "Hello, world!"
}
```

Every entry file has one `app "Title" { ... }` block. The string becomes the page title.

## Types & Values

| Type | Syntax | Examples |
|------|--------|---------|
| number | digits with optional unit | `20px`, `50%`, `2em`, `100`, `0.5` |
| text | double-quoted string | `"Hello"`, `"Count: {count}"` |
| color | hex literal | `#fff`, `#2563eb`, `#00000080` |
| bool | keyword | `true`, `false` |
| list | brackets | `["a", "b"]`, `[1, 2, 3]` |
| object | braces | `{ name: "Alice", age: 30 }` |

**Units:** `px` (pixels), `%` (parent-relative), `em` (font-relative). Duration: `ms`, `s`, `min`, `h`.

**String interpolation:** `"Hello, {name}!"`, `"{item.title} by {item.author}"`

## Expressions

```naze
-- Arithmetic: + - * / %
-- Comparison: == != > < >= <=
-- Logical: && ||
-- Negation: !expanded
-- Grouping: (a + b) * c
-- Function call: area(width, height)
-- Member access: user.name, posts.data
-- Inline conditional: if active { #3b82f6 } else { #94a3b8 }
```

## Layout Elements

### row — horizontal layout

```naze
row gap: 12px, padding: 16px {
  text "Left"
  spacer
  text "Right"
}
```

Key props: `gap`, `padding`, `width`, `height`, `color`, `radius`, `align` (cross-axis: start/center/end/stretch), `justify` (main-axis: start/center/end/space-between), `wrap`, `flex-grow`, `responsive` (breakpoint to switch to vertical), `collapsible` (breakpoint to hide).

### column — vertical layout

```naze
column gap: 16px, padding: 20px {
  heading "Title"
  text "Body text"
}
```

Same props as `row`. Default layout direction for most containers.

### container — styled box (vertical layout)

```naze
container padding: 16px, color: #f8fafc, radius: 8px {
  heading "Card"
  text "Content"
}
```

Extra props: `border`, `border-color`, `shadow`, `gradient`, `transition`.

### stack — overlapping layers

```naze
stack width: 200px, height: 200px {
  rect width: 200px, height: 200px, color: #e2e8f0
  text "On top"
}
```

### grid — wrapping grid

```naze
grid columns: 3, gap: 8px {
  rect width: 60px, height: 60px, color: #ff0000
  rect width: 60px, height: 60px, color: #00ff00
  rect width: 60px, height: 60px, color: #0000ff
}
```

### spacer — fills remaining space

```naze
row { text "Left"  spacer  text "Right" }
```

### scroll — scrollable container

```naze
scroll height: 400px {
  column gap: 8px { text "Item 1"  text "Item 2" }
}
```

### separator — horizontal line

```naze
separator
```

## Content Elements

### text — body text (default 16px)

```naze
text "Hello, world!"
text "Styled" color: #666, font-size: 14px, font-weight: bold
text "Count: {count}"
```

Props: `color`, `font-size`, `font-weight` (normal/bold), `font-style` (normal/italic), `align` (start/center/end), `text-decoration` (underline/line-through), `text-overflow` (clip/ellipsis), `line-height`, `letter-spacing`.

### heading — heading text (default 24px, bold)

```naze
heading "Page Title"
heading "Small" font-size: 18px, color: #1e293b
```

Same props as `text`.

### link — navigation link

```naze
link "About", to: "/about"
```

### code — monospace text

```naze
code "const x = 42;"
```

### image

```naze
image src: "photo.jpg", width: 200px, height: 150px, fit: "cover", alt: "Photo"
```

### rect — colored rectangle / button

```naze
rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
  text "Click me" color: #ffffff
  on click: set count = count + 1
}
```

Props: `width`, `height`, `color`, `radius`, `border`, `border-color`, `opacity`, `shadow`, `gradient`, `transform`, `transition`, `animate`.

## Input Elements

### input — text input with two-way binding

```naze
state name = ""
input bind: name, placeholder: "Your name"
input bind: email, type: "email", placeholder: "Email"
input bind: age, type: "number", placeholder: "Age"
input bind: pw, type: "password", placeholder: "Password"
input bind: avatar, type: "file", accept: "image/*", max-size: "5mb"
```

`bind` is **required** — connects input to a state variable.

### textarea — multi-line input

```naze
state bio = ""
textarea bind: bio, placeholder: "About you...", rows: 4, max-length: 500
```

### checkbox — boolean toggle

```naze
state agreed = false
checkbox bind: agreed, label: "I agree to the terms"
```

### radio — single selection from group

```naze
state choice = "a"
radio bind: choice, value: "a", label: "Option A"
radio bind: choice, value: "b", label: "Option B"
```

### select — dropdown

```naze
state color = "red"
select bind: color {
  option "Red" value: "red"
  option "Green" value: "green"
  option "Blue" value: "blue"
}
```

### Validation

```naze
input bind: username, validate: { required: true, min-length: 3, max-length: 20 }
if username_error { text "{username_error}" color: #dc2626 }
if username_valid { text "Valid" color: #16a34a }
```

Rules: `required`, `min-length`, `max-length`, `pattern` (regex), `min`, `max`.
Auto-generated state: `{name}_valid` (bool), `{name}_error` (text).

## Overlay Elements

```naze
state show = false

rect padding: 8px, color: #3b82f6, radius: 4px {
  text "Open" color: #fff
  on click: set show = true
}

if show {
  overlay focus-trap: true, scroll-lock: true {
    rect width: 400px, padding: 24px, radius: 12px, color: #fff, shadow: "xl" {
      heading "Dialog"
      text "Content here."
      rect padding: 8px, color: #ef4444, radius: 4px {
        text "Close" color: #fff
        on click: set show = false
      }
    }
    on click-outside: set show = false
  }
}
```

Props: `focus-trap`, `scroll-lock`, `dismiss-on-escape`, `anchor`, `anchor-placement`.

## State & Reactivity

### state — mutable, reactive, page-scoped

```naze
state count = 0
state name = ""
state items = ["Apple", "Banana"]
state user = { name: "Alice", role: "admin" }
on click: set count = count + 1
on click: set expanded = !expanded
```

### shared state — persists across page navigation

```naze
shared state current-user = null
shared state cart = []
```

Declared in the `app` block. Accessible from all pages. Mutated with `set`.

### computed — read-only derived values

```naze
computed total = price * quantity
computed passing = students | filter score > 60
computed top-3 = students | sort-by score | take 3
```

Auto-updates when dependencies change. Cannot be target of `set`.

### let — compile-time constant

```naze
let title = "My App"
let max-retries = 3
```

### storage — persistent browser storage

```naze
storage theme: local "theme-pref" default: "light"
storage sid: session "session-id" default: ""
```

Syntax: `storage name: (local|session) "key" default: value`

### param — URL query string binding

```naze
param page: number default: 1
param q: text default: ""
```

Changing via `set` auto-updates the URL. Browser back/forward updates the param.

### timer — scheduled actions

```naze
timer tick: every 1s { set seconds = seconds + 1 }
timer toast: after 5s { set visible = false }
```

Controls: `start timer-name`, `stop timer-name`.

## Events & Actions

### Events

```naze
on click: action
on hover: action
on change: action
on keypress: action
on scroll: action
on drag-start: action
on drag-over: action
on drop: action
on click-outside: action
on context-menu: action
on pointer-move: action
on arrow-up: action
on arrow-down: action
```

Custom component events: `on toggled: action` (matches `emit toggled`).

### Modifiers

```naze
on change debounce 300ms: trigger search
on scroll throttle 100ms: set pos = pos + 1
```

### Actions

| Action | Syntax |
|--------|--------|
| set | `set count = count + 1` |
| navigate | `navigate "/about"` |
| scroll-to | `scroll-to "element-id"` |
| log | `log "debug: {count}"` |
| trigger | `trigger data-name` |
| copy | `copy "text to clipboard"` |
| send | `send stream-name message` |
| js | `js "func"(args)` or `js "func"(args) -> state-var` |
| notify | `notify "Title" { body: "text", icon: "url" }` |
| emit | `emit event-name` |
| set-theme | `set-theme "dark"` |
| start | `start timer-name` |
| stop | `stop timer-name` |

## Conditionals & Iteration

### if / else if / else

```naze
if count > 0 {
  text "Positive"
} else if count == 0 {
  text "Zero"
} else {
  text "Negative"
}
```

### Inline conditional in props

```naze
rect color: if active { #3b82f6 } else { #94a3b8 } {
  text "Toggle"
}
```

### Truthiness

Falsy: `false`, `0`, `""`, `null`. Everything else is truthy.

### each — iteration

```naze
each item in items {
  text "{item.name}: {item.score}"
}

each student in students | filter score > 80 | sort-by name {
  text "{student.name}"
}
```

## Pipeline Operators

Transform data declaratively with `|`. Used in `computed`, `each`, function bodies.

```naze
computed passing = students | filter score > 60
computed top = students | sort-by score | take 3
computed total = students | map score | sum
```

| Operator | Args | Description |
|----------|------|-------------|
| `filter` | condition | Keep items where condition is true |
| `map` | field | Extract field from each item |
| `sort-by` | field | Sort ascending by field |
| `take` | N | Keep first N items |
| `sum` | — | Sum numeric values |
| `count` | — | Count items |
| `reduce` | expr, init | Fold: `reduce acc + it 0` |
| `group-by` | field | Group into object by field |
| `flatten` | — | Flatten nested lists one level |
| `distinct` | field? | Remove duplicates (optionally by field) |

Chaining: `items | filter score > 70 | sort-by name | take 10 | map name`

## Pattern Matching

```naze
match status {
  "active": text "Active" color: #16a34a
  "error": text "Error" color: #dc2626
  _: text "Unknown"
}

match theme {
  "dark": {
    rect color: #333 { text "Dark" color: #fff }
  }
  _: text "Default"
}
```

Arms: string literal, number, bool, identifier, `_` (wildcard). First match wins.

## Pure Functions

```naze
function area(w: number, h: number) -> number {
  w * h
}

computed surface = area(width, height)
```

Compile-time inlined. Pure (no side effects, no state access). Single expression body. Types: `text`, `number`, `bool`, `color`, `list`.

## Components

### Defining (one per file)

```naze
-- components/pill.naze
component pill(color: color, size: number = 60px) {
  rect width: size, height: 32px, color: color, radius: 16px
}
```

Parameters: `name: type` (required) or `name: type = default` (optional).

### Using

```naze
use components/pill

pill color: #22c55e
pill color: #ef4444, size: 80px
```

### Slots

```naze
-- components/card.naze
component card(bg: color = #ffffff) {
  container padding: 16px, radius: 8px, color: bg {
    slot
    slot "footer" {
      text "Default footer"
    }
  }
}
```

```naze
card bg: #f0f9ff {
  text "Default slot content"
  fill "footer" {
    text "Custom footer"
  }
}
```

### Component Events

```naze
-- components/toggle-btn.naze
component toggle-btn(label: text) {
  rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
    text "{label}" color: #ffffff
    on click: emit toggled
  }
}
```

```naze
toggle-btn label: "Toggle" {
  on toggled: set open = !open
}
```

## Templates

```naze
template two-panel(left, right) {
  row gap: 16px {
    column width: 300px { slot "left" }
    column flex-grow: 1 { slot "right" }
  }
}

two-panel {
  fill "left" { text "Navigation" }
  fill "right" { heading "Content" }
}
```

Built-in templates: `app-shell`, `dashboard`, `sidebar-layout`, `split-view`, `centered`.

## Pages & Routing

```naze
app "My Site" {
  -- Shell: renders on every page
  row padding: 16px, color: #1e293b {
    link "Home", to: "/"
    link "About", to: "/about"
  }

  page "/" {
    heading "Home"
  }

  page "/about" {
    heading "About"
  }
}
```

Navigation: `link "text", to: "/path"` (declarative), `on click: navigate "/path"` (programmatic).

Elements outside `page` blocks render on every page (headers, navbars).

## Data Fetching

All data sources produce `.loading`, `.error`, `.data` lifecycle fields.

### Basic fetch

```naze
data posts: fetch "https://api.example.com/posts"

if posts.loading { text "Loading..." }
if posts.error { text "Error: {posts.error}" color: #dc2626 }
if posts.data {
  each post in posts.data { text "{post.title}" }
}
```

### Enhanced fetch

```naze
data result: fetch "/api/submit" {
  method: post
  headers: { "Authorization": "Bearer {token}" }
  body: { name: name, email: email }
  trigger: manual
}

on click: trigger result
```

Props: `method` (get/post/put/delete/patch), `headers`, `body`, `content-type` (json/multipart), `cache` (duration), `retry` (count), `trigger` (manual).

### WebSocket stream

```naze
data chat: stream "wss://api.example.com/chat"
on click: send chat msg
```

### JS data source

```naze
data checkout: js "createCheckoutSession"(cart) { trigger: manual }
on click: trigger checkout
```

### Device APIs

```naze
data location: device "geolocation"
data location: device "geolocation" { watch: true }
data camera: device "camera"
```

## Server Functions

```naze
server function get-users(limit: number) {
  fetch "/db/users?limit={limit}"
}

server function create-post(title: text, body: text) {
  fetch "/db/posts" {
    method: post
    body: { title: title, body: body }
  }
}

-- Call from events
on click: call server.get-users(10)

-- Server data (resolved at SSR time)
data posts: get-latest-posts(10)
```

## Theming

```naze
theme light {
  colors { bg: #ffffff  fg: #0f172a  primary: #2563eb }
  spacing { sm: 8px  md: 16px  lg: 24px }
}

theme dark extends light {
  colors { bg: #1e293b  fg: #f8fafc  primary: #60a5fa }
}
```

Token refs: `theme.colors.primary`, `theme.spacing.md`.
Switch: `on click: set-theme "dark"`.
`extends` inherits all tokens, overrides redeclared ones.

## Animation

```naze
-- Transition (on prop change)
rect width: 200px, height: if expanded { 150px } else { 60px },
  color: #3b82f6, transition: "height 300ms ease-out" {
  text "Toggle"
  on click: set expanded = !expanded
}

-- Keyframe animation
rect animate: "color [#3b82f6, #10b981, #f59e0b] 2s ease infinite" {
  text "Animated"
}

-- Spring physics
rect animate: "height spring(200, 15)" {
  text "Springy"
}
```

Easing: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`, `spring(stiffness, damping)`.

## Visual Properties

```naze
-- Shadow presets: sm, md, lg, xl
rect shadow: "lg" { text "Shadowed" }

-- Gradient
rect gradient: "linear(to right, #3b82f6, #8b5cf6)" { text "Gradient" }

-- Transform
rect transform: "rotate(15deg)" { text "Rotated" }
rect transform: "scale(1.2)" { text "Scaled" }

-- Cursor
rect cursor: "pointer" { text "Clickable" }
```

## Responsive Layout

```naze
row responsive: 768px, gap: 16px {
  column width: 250px { text "Sidebar" }
  column flex-grow: 1 { text "Content" }
}
```

Below 768px viewport, `row` switches to vertical stacking.
`collapsible: 1200px` hides the element below the breakpoint.

## JavaScript Interop

```naze
-- Sync call
on click: js "alert"("Hello!")
on click: js "Date.now"() -> timestamp

-- Async data
data result: js "fetchExternal"(url) { trigger: manual }
```

Functions must be on `globalThis`. Declare scripts in `naze.toml` `[scripts]` section.

## AI Prompts

```naze
prompt summary: from openai {
  system: "You are a concise summarizer."
  user: "Summarize: {content}"
  model: "gpt-4o"
  max-tokens: 200
  temperature: 0.3
}

if summary.loading { text "Generating..." }
if summary.data { text "{summary.data}" }
```

Providers: `openai`, `anthropic`, `ollama`, or URL.

## Accessibility

```naze
rect role: "button", label: "Increment counter", tab-index: 0 {
  text "+" color: #fff
  on click: set count = count + 1
}
```

Props: `role`, `label`, `tab-index`, `id`.
Compiler warns if interactive elements lack `role`/`label`.

## Drag and Drop

```naze
rect draggable: true, drag-data: "item-1" {
  text "Drag me"
  on drag-start: set dragging = true
}

rect drop-target: true {
  text "Drop here"
  on drop: set dropped = drag-data
}
```

## Testing

```naze
-- tests/counter.test.naze
use counter

test "counter increments" {
  render counter
  assert text "Count: 0" is visible
  click "Increment"
  assert text "Count: 1" is visible
}

flow "login flow" {
  navigate "/"
  fill "email" with "user@example.com"
  click "Sign In"
  assert page is "/dashboard"
}
```

Steps: `render`, `click`, `fill "placeholder" with "value"`, `navigate`, `wait 300ms`.
Asserts: `text "..." is visible`, `text "..." is not visible`, `page is "/path"`, `state name is value`, `no accessibility violations`.

## Project Structure

```
my-app/
  naze.toml       -- project manifest
  app.naze        -- entry file
  theme.naze      -- optional theme tokens
  components/     -- one component per file
    card.naze
    pill.naze
  tests/          -- test files
    app.test.naze
```

### naze.toml

```toml
[app]
name = "my-app"
version = "0.1.0"

[build]
entry = "app.naze"

[scripts]
analytics = "./js/analytics.js"

[dependencies]
"@naze/ui-kit" = "^1.0"
```

### CLI

```
nazec new <name>           -- scaffold project
nazec build                -- compile to dist/
nazec dev                  -- dev server + hot reload
nazec run                  -- native desktop preview
nazec check                -- type-check only
nazec test                 -- run .test.naze files
nazec serve                -- production SSR server
nazec grammar --format gbnf  -- export grammar for constrained decoding
nazec ai generate "prompt" -- AI code generation
```

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
      if stats.loading { text "Loading..." }
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
    if username_error { text "{username_error}" color: #dc2626 }

    input bind: email, type: "email", placeholder: "Email", validate: { required: true }
    if email_error { text "{email_error}" color: #dc2626 }

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

### Multi-Page App with Navigation

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

**DO NOT** use HTML tags → use Naze elements (`text`, `heading`, `row`, `column`, `rect`, etc.)

**DO NOT** use CSS or `style` props → all styling is inline props (`color`, `font-size`, `padding`, etc.)

**DO NOT** use semicolons → newlines separate statements

**DO NOT** quote color values → `color: #2563eb` not `color: "#2563eb"`

**DO NOT** forget `bind:` on inputs → `input bind: name` is required for state binding

**DO NOT** use camelCase → use `kebab-case` for identifiers (`font-size`, `my-component`, `search-query`)

**DO NOT** nest text inside text → `text` is a leaf element with no children

**DO NOT** omit the app block → every entry file needs `app "Title" { ... }`

**DO NOT** use `className`, `style`, `div`, `span` → no DOM concepts exist in Naze

**DO NOT** create state in component files → state is declared in app/page scope, passed to components as props

**DO NOT** use `=` for props → props use colon: `width: 200px` not `width=200px`

**DO NOT** use `{` for string interpolation outside quotes → interpolation only works inside `"..."` strings

**DO NOT** use `var`, `let`, `const` from JS → use `state` (mutable) or `let` (immutable constant)

**DO NOT** write `else if` without space → it's `} else if condition {` with the else/if on one line
