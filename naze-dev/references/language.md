# Naze Language Reference

Naze is a declarative UI language that compiles `.naze` files to Canvas2D via WASM. No DOM. One canonical form per concept. Kebab-case identifiers. Comments: `--`.

## App Structure

Every entry file: `app "Title" { ... }`. The string becomes the page title.

## Types and Values

| Type | Examples |
|------|---------|
| number | `20px`, `50%`, `2em`, `100`, `0.5` |
| text | `"Hello"`, `"Count: {count}"` (interpolation) |
| color | `#fff`, `#2563eb`, `#00000080` |
| bool | `true`, `false` |
| list | `["a", "b"]`, `[1, 2, 3]` |
| object | `{ name: "Alice", age: 30 }` |

Units: `px`, `%`, `em`. Duration: `ms`, `s`.

Expressions: `+ - * / % == != > < >= <= && ||`. Member access: `user.name`. Inline conditional: `if active { #3b82f6 } else { #94a3b8 }`.

Note: `!var` negation does NOT work. Use `var == false` instead.

## Layout Elements

### `row`
Horizontal container.
Props: `gap`, `padding`, `width`, `height`, `color`, `radius`, `align`, `justify`, `wrap`, `flex-grow`, `responsive`, `collapsible`

### `column`
Vertical container. Same props as `row`.

### `container`
Styled box (vertical layout). Extra props: `border`, `border-color`, `shadow`, `gradient`, `transition`

### `stack`
Overlapping layers. Children stack on top of each other.

### `grid`
Wrapping grid layout.
Props: `columns`, `gap`

### `spacer`
Fills remaining space in a row or column.

### `scroll`
Scrollable container.
Props: `height`

### `separator`
Horizontal dividing line.

## Content Elements

### `text "content"`
Body text. Leaf element (no children).
Props: `color`, `font-size`, `font-weight`, `font-style`, `align`, `text-decoration`, `text-overflow`, `line-height`

### `heading "content"`
Heading text (24px bold default). Same props as `text`.

### `link "text", to: "/path"`
Navigation link. Triggers client-side routing.

### `code "content"`
Monospace formatted text.

### `image`
Image element.
Props: `src: "url"`, `width`, `height`, `fit: "cover"`, `alt: "description"`

### `rect`
Colored rectangle, commonly used as a button.
Props: `width`, `height`, `color`, `radius`, `border`, `border-color`, `opacity`, `shadow`, `gradient`, `transform`, `transition`, `animate`, `padding`

## Input Elements

All inputs require `bind:` to connect to a state variable.

### `input`
Text input.
Props: `bind: var`, `placeholder: "text"`, `type: "email"|"number"|"password"|"file"`, `validate: { ... }`

### `textarea`
Multi-line text input.
Props: `bind: var`, `placeholder: "text"`, `rows: N`

### `checkbox`
Boolean toggle.
Props: `bind: var`, `label: "text"`

### `radio`
Radio button (group by binding same variable).
Props: `bind: var`, `value: "val"`, `label: "text"`

### `select`
Dropdown selector. Contains `option` children.
```naze
select bind: choice {
  option "Option A" value: "a"
  option "Option B" value: "b"
}
```

### Validation

Add `validate:` to any input:
```naze
input bind: email, type: "email", validate: { required: true, min-length: 3, max-length: 100, pattern: "regex" }
```

Auto-generated state: `{name}_valid` (bool), `{name}_error` (text).

## Overlay

```naze
overlay {
  -- modal content here
}
```
Props: `focus-trap`, `scroll-lock`, `dismiss-on-escape`. Use `on click-outside:` to dismiss.

## State and Reactivity

```naze
state name = value            -- mutable, reactive, page-scoped
shared state name = value     -- persists across page navigation
computed name = expression    -- read-only derived value
let name = value              -- compile-time constant
```

### Storage

Persistent browser storage:
```naze
storage name: local "key" default: value    -- localStorage
storage name: session "key" default: value  -- sessionStorage
```

### Params

URL query parameter binding:
```naze
param name: type default: value
```

### Timers

```naze
timer name: every Ns { action }   -- repeating
timer name: after Ns { action }    -- one-shot
```

Control: `start timer-name`, `stop timer-name`

## Events

Available events: `click`, `hover`, `change`, `keypress`, `scroll`, `drag-start`, `drag-over`, `drop`, `click-outside`, `context-menu`, `pointer-move`, `arrow-up`, `arrow-down`

Syntax:
```naze
on click: set count = count + 1
on click debounce 300ms: set query = input-val
on scroll throttle 100ms: log "scrolled"
```

## Actions

| Action | Example |
|--------|---------|
| Set variable | `set count = count + 1` |
| Navigate | `navigate "/path"` |
| Append to list | `append item to list` |
| Remove from list | `remove index from list` |
| Scroll | `scroll-to "id"` |
| Log | `log "message"` |
| Trigger data reload | `trigger data-name` |
| Copy to clipboard | `copy "text"` |
| Send to stream | `send stream msg` |
| JS interop | `js "func"(args)` |
| JS with return | `js "func"(args) -> var` |
| Notification | `notify "Title" { body: "text" }` |
| Emit event | `emit event-name` |
| Switch theme | `set-theme "name"` |
| Timer control | `start timer` / `stop timer` |

## Conditionals

```naze
if condition {
  -- content
} else if other-condition {
  -- content
} else {
  -- content
}
```

Falsy values: `false`, `0`, `""`, `null`.

Important: Use multi-line format. Single-line `if condition { element }` does not parse inside app blocks.

## Iteration

```naze
each item in collection {
  text "{item.name}"
}
```

With pipeline operators:
```naze
each x in items | filter score > 80 | sort-by name {
  text "{x.name}: {x.score}"
}
```

Access loop index with `item_index`.

## Pipeline Operators

`filter cond`, `map field`, `sort-by field`, `take N`, `sum`, `count`, `reduce expr init`, `group-by field`, `flatten`, `distinct`

Pipelines work with `each`, `computed`, and inline expressions.

## Pattern Matching

```naze
match expr {
  "value1": text "First"
  "value2": text "Second"
  _: text "Default"
}
```

Arms can match: string, number, bool, identifier, `_` wildcard. Arms can contain simple elements and blocks, but NOT `each` statements.

## Functions

Pure, compile-time inlined, single expression body:
```naze
function double(n: number) -> number { n * 2 }
function greet(name: text) -> text { "Hello, {name}!" }
```

## Components

One component per file. Define:
```naze
-- components/card.naze
component card(title: text, color: color = #f3f4f6) {
  container padding: 16px, color: color, radius: 8px {
    heading "{title}"
    slot
  }
}
```

Import and use:
```naze
use components/card

app "Demo" {
  card title: "My Card" {
    text "Content fills the slot"
  }
}
```

### Named Slots

Define:
```naze
component layout() {
  column {
    slot "header" { heading "Default Header" }
    slot
    slot "footer" { text "Default Footer" }
  }
}
```

Fill:
```naze
layout {
  fill "header" { heading "Custom Header" }
  text "Main content"
  fill "footer" { text "Custom Footer" }
}
```

### Component Events

Emit from component: `emit event-name`
Handle on parent: `on event-name: action`

## Pages and Routing

```naze
app "Site" {
  -- Shared layout (renders on every page)
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

  page "/post/:id" {
    -- id available as route param
    text "Post {id}"
  }
}
```

Navigate programmatically: `on click: navigate "/path"`

## Data Fetching

All data sources produce `.loading`, `.error`, `.data` states.

### Fetch (HTTP GET)
```naze
data posts: fetch "https://api.example.com/posts"
```

### Enhanced Fetch
```naze
data result: fetch "https://api.example.com/data" {
  method: post
  headers: { "Authorization": "Bearer {token}" }
  body: { name: username }
  trigger: manual
}
```

### WebSocket Stream
```naze
data messages: stream "wss://api.example.com/ws"
```

### JS Interop Data
```naze
data result: js "functionName"(args) { trigger: manual }
```

### Device APIs
```naze
data location: device "geolocation"
data motion: device "accelerometer"
```

## Server Functions

Server-side execution. Must be at top level (outside `app` block).

```naze
server function get-data(id: number) {
  fetch "https://api.example.com/items/{id}"
}
```

Call from data: `data items: get-data(42)`
Call from event: `on click: call server.get-data(42)`

## Database Models and Queries

Compile-time model definitions with declarative queries:

```naze
model users {
  id number primary
  name text
  email text unique
  active bool
}

server function list-users() {
  find users where active == true order name
}

server function create-user(name: text, email: text) {
  insert users { name: name, email: email, active: true }
}

server function update-user(id: number, name: text) {
  update users set { name: name } where id == id
}

server function delete-user(id: number) {
  delete users where id == id
}
```

Query operations: `find`, `insert`, `update`, `delete`. Queries compile to SQL at build time.

## Theming

Define themes:
```naze
theme light {
  colors {
    primary: #2563eb
    bg: #ffffff
    text: #1e293b
  }
  spacing {
    sm: 8px
    md: 16px
    lg: 24px
  }
}

theme dark extends light {
  colors {
    primary: #60a5fa
    bg: #0f172a
    text: #f1f5f9
  }
}
```

Reference tokens: `theme.colors.primary`, `theme.spacing.md`
Switch: `on click: set-theme "dark"`

## Animation

### Transition
```naze
rect color: #2563eb, transition: "color 300ms ease" {
  on hover: set color = #1d4ed8
}
```

### Keyframe
```naze
rect animate: "opacity [0, 1] 500ms ease-out"
rect animate: "scale [0.9, 1.1, 1] 300ms ease infinite"
```

### Spring
```naze
rect animate: "x spring(200, 15)"
```

Easing functions: `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`

## Visual Props

- Shadow: `shadow: "sm"`, `shadow: "md"`, `shadow: "lg"`, `shadow: "xl"`
- Gradient: `gradient: "linear(to right, #hex, #hex)"`
- Transform: `transform: "rotate(45deg)"`, `transform: "scale(1.5)"`
- Cursor: `cursor: "pointer"`
- Opacity: `opacity: 0.5`

## Accessibility

Props: `role`, `label`, `tab-index`, `id`

The compiler warns if interactive elements (buttons, inputs) lack `role` or `label`.

```naze
rect width: 120px, height: 40px, color: #2563eb, radius: 8px, role: "button", label: "Submit form" {
  text "Submit" color: #fff
  on click: set submitted = true
}
```

## Guards

Route guards for authentication and authorization:

```naze
guard auth
  check logged-in redirect "/login"

page "/dashboard" guard: auth {
  text "Protected content"
}
```
