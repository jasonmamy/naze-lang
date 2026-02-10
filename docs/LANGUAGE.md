# Naze Language Reference

This documents what the Naze language supports today (Phase 1 + Phase 2) and planned additions (marked as *(planned)* or *not yet implemented*). For the long-term vision, see [PROTOTYPE.md](PROTOTYPE.md). For upcoming features, see [PHASE3.md](PHASE3.md) and [PHASE4.md](PHASE4.md). For the parity analysis that motivated the application logic primitives, see [PARITY.md](PARITY.md).

## File Structure

A `.naze` file contains any combination of:

- Comments
- `use` imports
- `app` blocks (entry files)
- `page` blocks (routing)
- `component` definitions (component files)
- `theme` definitions
- `let` bindings and `state` declarations
- `computed` declarations
- `shared state` declarations
- `storage` declarations
- `data` declarations (async fetch, streams)
- `param` declarations
- `timer` declarations
- Elements

```naze
-- This is a comment

use components/pill

app "My App" {
  state count = 0

  column padding: 20px, gap: 16px {
    heading "Hello"
    text "Count: {count}"
    pill color: #22c55e
  }
}
```

### Comments

Line comments start with `--` and extend to the end of the line.

```naze
-- This is a comment
heading "Hello" -- inline comment
```

## Types

Naze has four types:

| Type | Literal syntax | Examples |
|------|---------------|----------|
| `number` | Digits with optional decimal and unit | `20px`, `3.5em`, `50%`, `100`, `-8px` |
| `text` | Double-quoted string | `"Hello, world!"`, `"Dashboard"` |
| `color` | `#` followed by 3-8 hex digits | `#fff`, `#ff0000`, `#2563eb` |
| `bool` | `true` or `false` | `true`, `false` |

### Units

Numbers can have an optional unit suffix:

- `px` — pixels (default for layout dimensions)
- `%` — percentage (relative to parent)
- `em` — relative to font size

Numbers without a unit are treated as raw values (pixels for layout props, points for font-size).

### Compound Values

Lists and objects can be used as values:

```naze
state items = ["Apple", "Banana", "Cherry"]
validate: { required: true, min-length: 3 }
```

### String Interpolation

Strings can embed variable references using `{name}`:

```naze
text "Current count: {count}"
text "{item.title} by {item.author}"
```

### References

Inside component bodies, parameter names can be used as values. Multi-segment references use dot notation:

```naze
component box(color: color, size: number = 80px) {
  rect width: size, height: size, color: color
}
```

## App Block

Every entry file must contain one `app` block with a title string and a body:

```naze
app "My App" {
  -- children go here
}
```

The title is used as the page `<title>` in the generated HTML.

## Elements

Elements are the building blocks of a Naze UI. An element is a name followed by optional inline text, optional properties, and an optional child block:

```naze
element "optional text" prop: value, prop: value {
  -- children
}
```

### Layout Containers

These elements accept children and control how they are positioned.

#### `row`

Lays children out **horizontally** (left to right). Children are placed side-by-side. Spacers in a row expand horizontally to fill remaining width.

```naze
row gap: 12px, padding: 16px {
  rect width: 50px, height: 50px, color: #ff0000
  rect width: 50px, height: 50px, color: #00ff00
}
```

**Props:** `padding`, `gap`, `width`, `height`, `color`, `columns`, `align`, `justify`, `wrap`, `flex-grow`, `flex-shrink`, `min-width`, `max-width`, `min-height`, `max-height`, `cursor`, `shadow`, `overflow`, `gradient`, `transform`

#### `column`

Lays children out **vertically** (top to bottom). Children are stacked. Spacers in a column expand vertically to fill remaining height.

```naze
column gap: 16px, padding: 20px {
  heading "Title"
  text "Body text"
}
```

**Props:** `padding`, `gap`, `width`, `height`, `color`, `columns`, `align`, `justify`, `wrap`, `flex-grow`, `flex-shrink`, `min-width`, `max-width`, `min-height`, `max-height`, `cursor`, `shadow`, `overflow`, `gradient`, `transform`

#### `stack`

Layers children on top of each other at the same position. All children share the same origin point. Useful for overlays and backgrounds.

**Props:** `padding`, `gap`, `width`, `height`, `color`, `columns`, `align`, `justify`, `cursor`, `shadow`, `overflow`, `gradient`, `transform`

#### `grid`

Lays children out in a wrapping grid. The `columns` prop controls how many columns. Column width is computed automatically from available space.

```naze
grid columns: 3, gap: 8px {
  rect width: 60px, height: 60px, color: #ff0000
  rect width: 60px, height: 60px, color: #00ff00
  rect width: 60px, height: 60px, color: #0000ff
  rect width: 60px, height: 60px, color: #ffff00
}
```

**Props:** `padding`, `gap`, `width`, `height`, `color`, `columns`, `align`, `justify`, `cursor`, `shadow`, `overflow`, `gradient`, `transform`

#### `container`

A styled box that lays children out vertically (like `column`). Supports background color and border radius.

```naze
container padding: 16px, color: #eff6ff, radius: 8px {
  text "Inside a card"
}
```

**Props:** `padding`, `width`, `height`, `radius`, `color`, `border`, `border-color`, `opacity`, `cursor`, `shadow`, `overflow`, `gradient`, `transform`

#### `spacer`

An invisible element that expands to fill remaining space. In a `row`, it expands horizontally. In a `column`, it expands vertically. If given explicit dimensions, it uses those instead.

```naze
column {
  heading "Top"
  spacer
  text "Bottom"
}
```

**Props:** `width`, `height`

#### `scroll`

A scrollable container. Content that exceeds the container's dimensions can be scrolled with the mouse wheel. Renders scrollbars automatically.

```naze
scroll height: 400px {
  column gap: 8px {
    -- many items that exceed 400px
  }
}
```

**Props:** `width`, `height`, `overflow` (`"x"`, `"y"`, `"both"`)

### Drawing Elements

#### `rect`

Draws a colored rectangle.

```naze
rect width: 80px, height: 80px, color: #2563eb, radius: 8px
```

**Props:** `width`, `height`, `color`, `background`, `radius`, `opacity`, `border`, `border-color`, `cursor`, `shadow`, `gradient`, `transform`

#### `text`

Renders body text. The string content is passed inline after the element name. Default font size is 16px.

```naze
text "Hello, world!"
text "Colored text" color: #666666, font-size: 14px
```

**Props:** `color`, `font-size`, `size`, `weight`, `opacity`, `cursor`, `text-decoration`, `text-align`, `line-height`, `letter-spacing`, `text-overflow`, `transform`

#### `heading`

Renders heading text. Default font size is 24px, rendered bold.

```naze
heading "Page Title"
heading "Small heading" font-size: 18px, color: #1e293b
```

**Props:** `color`, `font-size`, `size`, `opacity`, `cursor`, `text-decoration`, `text-align`, `line-height`, `letter-spacing`, `text-overflow`, `transform`

#### `image`

Displays an image. Loaded asynchronously and cached.

```naze
image src: "photo.jpg", width: 200px, height: 150px
```

**Props:** `src`, `width`, `height`, `fit`, `alt`, `cursor`, `transform`

#### `link`

A clickable navigation link for routing between pages.

```naze
link "About" to: "/about"
```

## Components

Components are reusable UI pieces defined in `.naze` files. One component per file, filename is the component name.

### Defining a Component

```naze
-- components/pill.naze

component pill(color: color, size: number = 60px) {
  rect width: size, height: 32px, color: color, radius: 16px
}
```

Parameters have a name, a type, and an optional default value. Parameters without defaults are required.

### Using a Component

Import with `use`, then use the component name as an element:

```naze
use components/pill

app "Demo" {
  row gap: 8px {
    pill color: #22c55e
    pill color: #eab308
    pill color: #ef4444, size: 80px
  }
}
```

### How Components Work

At compile time, component invocations are **inlined** — the component body is substituted with parameter values filled in. The three `pill` calls above become three `rect` elements in the final render tree. There is no runtime component overhead.

### Import Resolution

`use components/pill` resolves to `components/pill.naze` relative to the project root. The compiler discovers all `.naze` files in the project directory recursively.

### Type Checking

The compiler checks:
- Required props (no default) are provided
- Prop types match parameter declarations
- Unknown props produce an error
- Built-in element props have correct types
- Interactive elements without `role` or `label` produce a warning

### Content Slots

Components can define insertion points for caller-provided content using `slot`:

```naze
-- components/card.naze
component card(title: text) {
  container padding: 16px, radius: 8px, color: #ffffff {
    heading title
    slot
    slot "footer" {
      text "Default footer"
    }
  }
}
```

At the call site, children go into the default slot. Named slots use `fill`:

```naze
card title: "My Card" {
  text "This goes in the default slot"
  fill "footer" {
    text "Custom footer"
  }
}
```

Slots with a body block provide fallback content used when the caller doesn't fill them.

## State & Reactivity

### Let Bindings (Immutable)

`let` creates compile-time constants:

```naze
let title = "My Counter"
let colors = ["#3b82f6", "#10b981", "#f59e0b"]
```

### State Variables (Mutable, Reactive)

`state` creates mutable variables. Changes trigger re-render:

```naze
state count = 0
state items = ["Apple", "Banana", "Cherry"]
state expanded = false
```

State variables can hold numbers, text, booleans, and lists.

### Computed State

`computed` creates read-only derived values that auto-update when their dependencies change. Replaces the need for `useMemo` or repeated inline expressions.

```naze
computed full-name = "{first-name} {last-name}"
computed has-items = item-count > 0
computed total = count * price
```

**Semantics:**
- Read-only — cannot be the target of `set`
- Dependencies tracked at compile time (scans expression for state/computed refs)
- Re-evaluates only when a dependency changes
- Can reference other `computed` values (compiler validates no cycles)
- Pipeline syntax (M15, not yet implemented) will work naturally: `computed x = list | filter | sort | take 5`

**Grammar:** Mirrors `state` exactly — `computed name = expression`

### Shared State

`shared state` creates state that persists across pages and is accessible from any component. Replaces the need for React Context, Redux, or Zustand.

```naze
-- Simple shared state (accessible from any page/component)
shared state current-user = null
shared state auth-token = ""
shared state cart-items = []
```

**Semantics:**
- Same as `state` but not scoped to a page — persists across `navigate` actions
- Changes trigger re-render on any page that references the shared state
- Mutated with same `set` action as regular state

**Grammar:** `shared` modifier on existing `state` rule.

### Persistent Storage

`storage` creates reactive state bound to browser localStorage or sessionStorage. Values persist across sessions and auto-sync on change.

```naze
-- localStorage (persists across browser sessions)
storage theme-preference: local "theme" default: "light"
storage cart: local "shopping-cart" default: []
storage recent-searches: local "recent" default: []

-- sessionStorage (persists within tab only)
storage session-id: session "sid" default: ""
```

**Semantics:**
- Behaves like `state` — reactive, triggers re-render on change
- Initialized from browser storage on load; uses `default` if key not found
- Changes via `set` auto-sync to storage: `set theme-preference = "dark"` writes to localStorage
- JSON serialization for non-string values (lists, objects)
- `local` = localStorage (persists across sessions), `session` = sessionStorage (tab-scoped)

**Grammar:** `storage name: (local | session) "key" default: value`

## Events & Actions

### Event Handlers

Attach event handlers to elements with `on event-name: action`:

```naze
rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
  text "Click me"
  on click: set count = count + 1
  on hover: set hovered = true
}
```

**Supported events:**

| Event | Triggers when |
|-------|--------------|
| `on click` | Element is clicked or activated via Enter key |
| `on hover` | Mouse enters element |
| `on keypress` | Key pressed while element is focused |
| `on change` | Form input value changes |
| `on drag-start` | Drag operation begins on draggable element |
| `on drag-over` | Dragged item is over a drop target |
| `on drop` | Item is dropped on a drop target |
| `on scroll` | Scroll position changes in a scroll container |

### Actions

| Action | Syntax | Description |
|--------|--------|-------------|
| `set` | `set variable = expression` | Update a state variable |
| `navigate` | `navigate "/path"` | Navigate to a route |
| `scroll-to` | `scroll-to "element-id"` | Scroll to an element by ID |
| `log` | `log expression` | Output to browser console / stderr |
| `trigger` | `trigger data-name` | Trigger a manual `data` fetch |
| `copy` | `copy expression` | Copy value to clipboard |
| `send` | `send stream-name expression` | Send message on a WebSocket stream |
| `js` | `js "function"(args)` or `js "function"(args) -> target` | Call a JavaScript function *(planned)* |
| `notify` | `notify "title" { body: "text" }` | Send browser notification *(planned)* |

### Expressions

Actions support arithmetic and comparison expressions:

```naze
on click: set count = count + 1
on click: set total = price * quantity
on click: set visible = !visible
on click: set index = (index + 1) % 5
```

**Operators:** `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `>`, `<`, `>=`, `<=`, `&&`, `||`

### Event Modifiers: Debounce & Throttle

Event handlers can have `debounce` or `throttle` modifiers to control firing rate.

```naze
-- Debounce: wait for 300ms of inactivity before firing
input bind: search-query, on change debounce 300ms: trigger search-results

-- Throttle: fire at most once per 100ms
on scroll throttle 100ms: set scroll-position = event.y
```

- `debounce Nms` — delays action until N milliseconds of inactivity
- `throttle Nms` — executes at most once per N milliseconds

### Timer

Timers schedule actions based on time. Two forms: `after` (one-shot) and `every` (repeating).

```naze
-- One-shot: fires once after delay
timer toast-dismiss: after 5s {
  set show-toast = false
}

-- Repeating: fires every interval
timer auto-save: every 30s {
  trigger save-draft
}
```

**Semantics:**
- `after duration { action }` — executes once, then stops
- `every duration { action }` — repeats until the page/component is no longer rendered
- Duration units: `ms`, `s`, `min` (e.g., `300ms`, `5s`, `30min`)
- Timers are automatically cleaned up when their page/component unmounts

## Conditional Rendering

### If / Else

Show or hide UI based on state:

```naze
if count > 0 {
  text "Count is {count} (positive)"
} else {
  text "Count is zero"
}
```

Chained conditions:

```naze
if status == "loading" {
  text "Loading..."
} else if status == "error" {
  text "Error!" color: #dc2626
} else {
  text "Done"
}
```

Inline conditional values on properties:

```naze
row height: if expanded { 150px } else { 60px } {
  text "Content"
}
```

### Each (Iteration)

Iterate over lists:

```naze
state items = ["Apple", "Banana", "Cherry"]

each item in items {
  text "{item}"
}
```

With dot-access on list items:

```naze
each post in posts.data {
  column padding: 12px, background: #f3f4f6, radius: 8px {
    text "{post.title}" weight: bold
    text "{post.body}" color: #666666
  }
}
```

## Theming

### Theme Definition

Define design tokens in a `theme` block or `theme.naze` file:

```naze
theme {
  colors {
    primary: #2563eb
    secondary: #64748b
    success: #16a34a
    warning: #f59e0b
    danger: #dc2626
    background: #ffffff
    foreground: #0f172a
  }
  spacing {
    xs: 4px
    sm: 8px
    md: 16px
    lg: 24px
    xl: 32px
  }
}
```

### Token References

Use `theme.section.token` to reference tokens in any element:

```naze
column padding: theme.spacing.lg, gap: theme.spacing.md {
  heading "Title" color: theme.colors.foreground
  rect width: 60px, height: 60px, color: theme.colors.primary, radius: 8px
}
```

Tokens are resolved at **compile time** (values are inlined into the render tree).

### Built-in Default Theme

Apps have access to default tokens (primary, secondary, success, warning, danger, background, foreground, muted, border; xs, sm, md, lg, xl spacing) without defining a custom theme.

## Layout Semantics

Layout is computed top-down from the viewport dimensions. Each element receives available width and height from its parent.

- **Explicit dimensions** (`width`, `height`) are used as-is
- **Percentage dimensions** resolve relative to parent: `width: 50%`
- **Implicit dimensions** are computed from children:
  - `row`: width = sum of children widths + gaps, height = tallest child
  - `column`/`container`: width = widest child, height = sum of children heights + gaps
  - `grid`: width = available width, height = sum of row heights + gaps
  - `text`/`heading`: measured from text content and font size
  - `rect`: 0x0 if no dimensions specified
- **Padding** insets the content area on all sides
- **Gap** adds space between children (not before first or after last)
- **Spacer** expands to fill remaining space in its parent's layout direction

### Flex Properties

```naze
row {
  rect width: 100px, height: 50px, color: #ff0000
  rect flex-grow: 1, height: 50px, color: #00ff00   -- fills remaining space
  rect width: 100px, height: 50px, color: #0000ff
}
```

- `flex-grow: N` — element grows to fill remaining space (proportional to N)
- `flex-shrink: N` — element shrinks proportionally when children overflow
- `wrap: true` on `row` — children wrap to the next line when they exceed width

### Size Constraints

- `min-width`, `max-width`, `min-height`, `max-height` — clamp computed dimensions

### Alignment

- `align` — cross-axis alignment: `start`, `center`, `end`, `stretch`
- `justify` — main-axis alignment: `start`, `center`, `end`, `space-between`, `space-around`, `space-evenly`

## Navigation & Routing

### Page Blocks

Define multiple pages in a single app file:

```naze
app "My Site" {
  page "/" {
    heading "Home"
    link "Go to About" to: "/about"
  }
  page "/about" {
    heading "About"
    link "Go Home" to: "/"
  }
}
```

### Navigation

- `link "text" to: "/path"` — clickable navigation link
- `on click: navigate "/path"` — programmatic navigation in event handlers
- Browser back/forward works via History API integration

### URL Parameters

`param` declares reactive state bound to URL query parameters. Enables bookmarkable, shareable search/filter/pagination state.

```naze
param page: number default: 1
param search: text default: ""
param sort: text default: "newest"

-- Two-way bound: changing the param updates the URL, and vice versa
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

**Grammar:** `param name: type default: value`

## Form Inputs

### Input

Text input with two-way state binding:

```naze
state name = ""

input bind: name, placeholder: "Type here..."
text "Hello, {name}!"
```

**Props:** `bind` (state variable), `placeholder`, `type` (`"text"`, `"number"`, `"email"`, `"password"`, `"file"`), `accept` (MIME filter for file type), `max-size` (size limit for file type)

### Checkbox

```naze
state agreed = false
checkbox bind: agreed, label: "I agree to the terms"
```

### Radio

```naze
state choice = "a"
radio bind: choice, value: "a", label: "Option A"
radio bind: choice, value: "b", label: "Option B"
```

### Select / Option

```naze
state color = "red"
select bind: color {
  option "Red" value: "red"
  option "Green" value: "green"
  option "Blue" value: "blue"
}
```

### Validation

Attach validation rules to inputs. The compiler generates `{field}_valid` (bool) and `{field}_error` (text) state variables automatically.

```naze
state username = ""

input bind: username, placeholder: "Username", validate: { required: true, min-length: 3, max-length: 20 }

if username_error {
  text "{username_error}" color: #dc2626
}
if username_valid {
  text "Username valid" color: #16a34a
}
```

**Validation rules:** `required`, `min-length`, `max-length`, `pattern` (text); `min`, `max` (number)

### File Input

File selection via the existing `input` element with `type: "file"`. Upload via enhanced `data` POST.

```naze
input type: "file", bind: avatar-file, accept: "image/*", max-size: 5mb

-- Upload via enhanced data POST
data upload-result: fetch "/api/upload" {
  method: post
  body: { file: avatar-file }
  content-type: multipart
  trigger: manual
}

on click: trigger upload-result

if upload-result.loading { text "Uploading..." }
if upload-result.error { text upload-result.error color: #dc2626 }
if upload-result.data { text "Upload complete" color: #16a34a }
```

**Props:** `bind` (state variable), `accept` (MIME type filter, e.g., `"image/*"`), `max-size` (client-side limit, e.g., `5mb`, `100kb`)

### Textarea (Planned — not yet implemented)

Multi-line text input for comments, descriptions, bios, and longer text content.

```naze
state bio = ""
textarea bind: bio, placeholder: "Tell us about yourself...", rows: 4, max-length: 500

if bio {
  text "Preview: {bio}"
}
```

**Props:** `bind` (state variable), `placeholder`, `rows` (visible height in text rows, default: 3), `max-length` (character limit)

**Semantics:**
- Same as `input` but supports multi-line text with line breaks
- Two-way binding via `bind` (same as other form elements)
- Validation rules (`required`, `min-length`, `max-length`) work the same as `input`

## Drag & Drop

### Draggable Elements

```naze
rect draggable: true, drag-data: "Red", width: 80px, height: 80px, color: #ef4444 {
  text "Drag me"
  on drag-start: set drag-active = true
}
```

**Props:** `draggable: true`, `drag-data: expression`

### Drop Targets

```naze
rect drop-target: true, width: 300px, height: 120px, color: #f1f5f9 {
  on drag-over: set status = "Release to drop..."
  on drop: set status = "Dropped!"
}
```

## Accessibility

### Props

Any element can have accessibility properties:

- `role: "button"` — ARIA role (`button`, `link`, `navigation`, `main`, `heading`, `list`, `listitem`, etc.)
- `label: "Accessible name"` — equivalent to `aria-label`
- `tab-index: N` — keyboard navigation order
- `id: "element-id"` — element identification (used by `scroll-to`)

### Keyboard Navigation

- **Tab / Shift+Tab** cycles through focusable elements (interactive elements, inputs, links)
- **Enter** activates the focused element
- **Escape** clears focus
- A **focus ring** renders around the currently focused element

### Screen Reader Support

A hidden DOM overlay mirrors canvas content with ARIA attributes. ARIA roles are automatically inferred from element kind (heading → `role="heading"`, link → `role="link"`). The compiler warns when interactive elements lack `role` or `label`.

## Data Fetching

Declare async data sources with the `data` keyword:

```naze
data posts: fetch "https://jsonplaceholder.typicode.com/posts?_limit=5"

if posts.loading {
  text "Loading posts..."
}

if posts.error {
  text "Error: {posts.error}" color: #dc2626
}

if !posts.loading {
  each post in posts.data {
    text "{post.title}" weight: bold
  }
}
```

Three derived states are available for each `data` declaration:
- `name.loading` — `true` while fetching
- `name.error` — error message string (falsy if no error)
- `name.data` — the fetched data (available after loading completes)

### Enhanced Data: Full HTTP

The `data` keyword supports an optional block body with full HTTP configuration: methods, headers, params, body, caching, retry, and manual triggering.

```naze
-- Read operation (auto-fetches on mount, reactive to interpolated state)
data users: fetch "/api/users" {
  method: get
  params: { page: current-page, limit: 20 }
  headers: { "Authorization": "Bearer {auth.token}" }
  cache: 5min
  retry: 3
}

-- Write operation (only fetches when triggered)
data create-result: fetch "/api/users" {
  method: post
  body: { name: name-input, email: email-input }
  headers: { "Authorization": "Bearer {auth.token}" }
  trigger: manual
}

-- Trigger from event handler:
on click: trigger create-result

-- create-result.loading, create-result.error, create-result.data all work as normal

-- Reactive URL — re-fetches automatically when search-query state changes
data results: fetch "/api/search?q={search-query}" {
  cache: 30s
}
```

**Block properties:**
- `method` — `get`, `post`, `put`, `delete` (default: `get`)
- `params` — query parameters as object
- `headers` — HTTP headers as object (supports string interpolation for auth tokens)
- `body` — request body as object (for `post`/`put`)
- `content-type` — `json` (default) or `multipart` (for file uploads)
- `cache` — reuse response for identical requests within duration (e.g., `5min`, `30s`)
- `retry` — retry count on network failure with exponential backoff
- `trigger: manual` — suppresses auto-fetch; activated by `trigger name` action

**Semantics:**
- Block body is optional — `data x: fetch "url"` (current syntax) still works
- All operations produce the same `.loading`/`.error`/`.data` lifecycle
- Reactive URL interpolation: if `{search-query}` changes, re-fetches automatically (GET only)

### Data Streams: WebSocket / SSE

`data: stream` declares a persistent connection for real-time push data. Reuses the same `data` lifecycle pattern.

```naze
-- WebSocket connection
data chat: stream "wss://api.example.com/chat/{room-id}"
-- chat.data    → reactive list, grows as messages arrive
-- chat.loading → true until connection established
-- chat.error   → set on connection error

-- Server-Sent Events
data notifications: stream "/api/events" {
  type: sse
}

-- Send a message on a WebSocket stream
on click: send chat "{message-input}"
```

**Semantics:**
- Uses `stream` instead of `fetch` to signal a persistent connection
- `.data` is a reactive list that grows as messages arrive (most recent appended)
- URL interpolation is reactive — changing `{room-id}` closes old connection and opens new one
- `send` action pushes a message to a WebSocket stream
- Default type is WebSocket; `type: sse` for Server-Sent Events (read-only)
- Auto-reconnect on disconnect with exponential backoff
- Same `.loading`/`.error`/`.data` lifecycle as `fetch`

## Animation

Animate property changes with the `transition` prop:

```naze
row
    width: 200px,
    height: if expanded { 150px } else { 60px },
    background: #3b82f6,
    radius: 8px,
    transition: "height 300ms ease-out"
{
    text "Click to toggle"
    on click: set expanded = !expanded
}
```

**Format:** `transition: "property duration easing"`

- **Properties:** any numeric or color property (height, width, opacity, background, color)
- **Duration:** milliseconds (e.g., `300ms`)
- **Easing:** `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`

Multiple transitions can target different properties. Color transitions interpolate RGB components.

## Debug Logging

The `log` action outputs values to the browser console (or stderr in native mode). Useful for debugging state and event handlers.

```naze
on click: log "button clicked"
on click: log count              -- log state variable value
on click: log "count is: {count}" -- interpolated string
```

## Project Structure

A Naze project has this structure:

```
my-project/
  naze.toml           # Project manifest
  app.naze            # Entry file
  theme.naze          # Optional theme tokens
  components/         # Component files
    pill.naze
    card.naze
```

### naze.toml

```toml
[app]
name = "my-project"
version = "0.1.0"

[build]
entry = "app.naze"
output = "dist/"
```

## Grammar Summary

### Phase 1 Core Grammar

```
file        = statement*
statement   = comment | use_stmt | component_def | app_block | element

comment     = "--" (any until newline)
use_stmt    = "use" path newline
app_block   = "app" string block
component   = "component" name params? block

element     = name string? props? (block | newline)
props       = prop ("," prop)*
prop        = name ":" value
block       = "{" statement* "}"

params      = "(" param ("," param)* ")"
param       = name ":" type ("=" value)?
type        = "text" | "number" | "bool" | "color"

value       = string | color | number | bool | ref | name
string      = '"' (escaped_char | any)* '"'
color       = "#" hex{3,8}
number      = digits ("." digits)? ("px" | "%" | "em")?
bool        = "true" | "false"
ref         = name ("." name)+
name        = (letter | "_") (alphanumeric | "_" | "-")*
```

### Phase 2 Additions

```
statement  += page_block | theme_def | let_stmt | state_stmt | data_stmt
            | if_stmt | each_stmt | on_handler | slot_stmt | fill_stmt
            | link_element

page_block  = "page" string block
link_element = "link" string "," "to" ":" string
theme_def   = "theme" "{" theme_section* "}"
theme_section = ("colors" | "spacing") "{" (name ":" value)* "}"

let_stmt    = "let" name "=" value
state_stmt  = "state" name "=" value
data_stmt   = "data" name ":" "fetch" string

on_handler  = "on" event_name ":" action
event_name  = "click" | "hover" | "keypress" | "change"
            | "drag-start" | "drag-over" | "drop" | "scroll"
action      = set_action | navigate_action | scroll_to_action | log_action
set_action  = "set" name "=" expression
navigate_action = "navigate" string
scroll_to_action = "scroll-to" string
log_action  = "log" expression

if_stmt     = "if" expression block ("else" (if_stmt | block))?
each_stmt   = "each" name "in" (ref | name) block

slot_stmt   = "slot" string? (block | newline)
fill_stmt   = "fill" string block

expression  = atom (bin_op atom)*
bin_op      = "==" | "!=" | ">=" | "<=" | ">" | "<"
            | "&&" | "||" | "+" | "-" | "*" | "/"

value      += object | list
object      = "{" (name ":" value ("," name ":" value)*)? "}"
list        = "[" (value ("," value)*)? "]"
string      = '"' (interpolation | chars)* '"'
interpolation = "{" (ref | name) "}"
```

Whitespace (spaces, tabs) is ignored between tokens. Newlines are significant as statement terminators for elements without blocks.

### Application Logic Additions (M19d — Implemented)

```
statement  += computed_stmt | shared_state_stmt | storage_stmt
            | param_stmt | timer_stmt

-- Derived reactive state
computed_stmt   = "computed" name "=" expression

-- Global shared state
shared_state_stmt = "shared" "state" name "=" value

-- Persistent browser storage
storage_stmt    = "storage" name ":" ("local" | "session") string "default:" value

-- URL query parameters
param_stmt      = "param" name ":" type "default:" value

-- Timers
timer_stmt      = "timer" name ":" ("after" | "every") duration block
duration        = number ("ms" | "s" | "min")

-- Enhanced data with optional block body
data_stmt       = "data" name ":" "fetch" string data_block?
                | "data" name ":" "stream" string data_block?
data_block      = "{" data_prop* "}"
data_prop       = ("method" | "params" | "headers" | "body" | "cache"
                | "retry" | "trigger" | "content-type" | "type") ":" value

-- Extended actions
action         += trigger_action | copy_action | send_action
trigger_action  = "trigger" name
copy_action     = "copy" expression
send_action     = "send" name expression

-- Event modifiers
on_handler      = "on" event_name event_modifier? ":" action
event_modifier  = ("debounce" | "throttle") duration
```

### Planned Grammar Additions (M19e — not yet implemented)

```
-- JS interop
action         += js_action | notify_action
js_action       = "js" string "(" args? ")" ("->" name)?
notify_action   = "notify" string notify_block?
notify_block    = "{" ("body" | "icon") ":" value "}"

-- Additional data sources
data_stmt      += "data" name ":" "js" string "(" args? ")" data_block?
                | "data" name ":" "device" device_api data_block?
device_api      = "geolocation" | "camera"

-- Textarea element
textarea_stmt   = "textarea" props
```

## Future Ideas

For upcoming language features (pipeline operators, pattern matching, layout templates, server functions, etc.), see:

- [PHASE3.md](PHASE3.md) — Language completion and developer experience
- [PHASE4.md](PHASE4.md) — Ecosystem and external integration
- [PARITY.md](PARITY.md) — Component and application logic parity analysis (includes design rationale for planned primitives)
- [PROTOTYPE.md](PROTOTYPE.md) — Full architecture spec

### Overlay System (Planned — M19b)

An `overlay` element that renders content above normal layout flow, enabling dialogs, dropdowns, tooltips, popovers, toasts, and menus. Includes `focus-trap`, `scroll-lock`, `on click-outside`, and `anchor` positioning.

### Visual Properties (M19c — Implemented)

Additional styling properties for production visual fidelity. These properties are type-checked per element and rendered via Canvas2D (WASM) and tiny-skia (native).

#### `cursor`

Sets the mouse cursor style when hovering over an element. Available on all elements.

```naze
rect cursor: "pointer" { text "Click me" }
text "Not allowed" cursor: "not-allowed"
```

**Values:** `pointer`, `grab`, `grabbing`, `text`, `not-allowed`, `crosshair`, `move`, `resize`, `default`

#### `text-decoration`

Adds visual decoration to text. Available on `text`, `heading`, `link`.

```naze
text "Underlined" text-decoration: "underline"
text "Struck through" text-decoration: "line-through"
text "Overline" text-decoration: "overline"
```

**Values:** `underline`, `line-through`, `overline`, `none`

#### `shadow`

Adds a box shadow to container and shape elements. Available on `row`, `column`, `stack`, `grid`, `rect`, `container`, `overlay`.

```naze
container shadow: "lg", padding: 24px, radius: 12px, color: #ffffff {
  text "Card with shadow"
}
rect width: 100px, height: 100px, color: #fff, shadow: "0 4px 6px rgba(0,0,0,0.1)"
```

**Named presets:** `sm`, `md`, `lg`, `xl`
**Custom format:** `"offsetX offsetY blur color"` (e.g., `"0 4px 6px rgba(0,0,0,0.1)"`)

#### `text-align`

Aligns text horizontally within its container. Available on `text`, `heading`, `link`.

```naze
text "Centered" text-align: "center"
text "Right aligned" text-align: "right"
```

**Values:** `start` (default), `center`, `end`, `right`

#### `line-height`

Sets line height as a multiplier of font size. Available on `text`, `heading`, `link`.

```naze
text "Spacious text" line-height: 2.0
```

**Value:** number (multiplier of font-size, e.g., `1.5` = 1.5× font size)

#### `letter-spacing`

Adjusts spacing between characters. Available on `text`, `heading`.

```naze
text "S p a c e d" letter-spacing: 3px
```

**Value:** number with px unit

#### `text-overflow`

Controls how text is displayed when it overflows its container. Available on `text`, `heading`.

```naze
text "This very long text will be truncated..." text-overflow: "ellipsis"
```

**Values:** `clip` (default), `ellipsis` (truncates with "...")

#### `overflow`

Controls clipping of child content that exceeds container bounds. Available on `row`, `column`, `stack`, `grid`, `container`.

```naze
column width: 200px, height: 100px, overflow: "hidden" {
  text "This content will be clipped at the container boundary"
}
```

**Values:** `visible` (default), `hidden`, `clip`

#### `gradient`

Fills an element with a gradient instead of a solid color. Takes priority over `color`. Available on `row`, `column`, `stack`, `grid`, `rect`, `container`.

```naze
rect width: 300px, height: 100px, gradient: "linear(to-right, #3b82f6, #8b5cf6)", radius: 8px
rect width: 150px, height: 150px, gradient: "radial(#ffffff, #10b981)", radius: 75px
```

**Linear format:** `"linear(direction, color1, color2, ...)"` — directions: `to-right`, `to-left`, `to-bottom`, `to-top`, `to-bottom-right`, `to-top-right`
**Radial format:** `"radial(center-color, edge-color, ...)"`

#### `transform`

Applies 2D transformations to elements. Available on `row`, `column`, `stack`, `grid`, `rect`, `text`, `heading`, `container`, `image`.

```naze
rect width: 80px, height: 80px, color: #3b82f6, transform: "rotate(45deg)"
rect width: 60px, height: 60px, color: #ef4444, transform: "scale(1.3)"
rect width: 60px, height: 60px, color: #10b981, transform: "translate(10px, -5px)"
```

**Values:** `"rotate(Ndeg)"`, `"scale(N)"` or `"scale(X, Y)"`, `"translate(Xpx, Ypx)"`

### JavaScript Interop (Planned — M19e)

Controlled escape hatch for calling third-party JavaScript SDKs (Stripe, Mapbox, analytics, auth providers) from Naze code. Functions must be on `globalThis` — no module imports, keeping it simple and auditable.

#### Script Inclusion (in `naze.toml`)

```toml
[scripts]
stripe = "https://js.stripe.com/v3/"
mapbox = "https://api.mapbox.com/mapbox-gl-js/v3/mapbox-gl.js"
analytics = "./js/analytics.js"
```

The compiler embeds `<script>` tags in the generated `index.html`.

#### Sync Calls

```naze
-- Call a JS function as an event action
on click: js "analytics.track"("button_clicked", { page: "home" })

-- Call and store return value in state
on click: js "Stripe"(stripe-key) -> stripe-instance
```

#### Async Calls (with data lifecycle)

```naze
-- Async JS call with loading/error/data lifecycle
data checkout: js "createCheckoutSession"(cart-items) {
  trigger: manual
}

on click: trigger checkout
if checkout.loading { text "Processing..." }
if checkout.error { text checkout.error color: #dc2626 }
```

**Semantics:**
- `js "functionName"(args)` — calls `globalThis.functionName(args)` synchronously
- `js "name"(args) -> target` — stores return value in state variable
- `data name: js "functionName"(args)` — async JS call with `.loading`/`.error`/`.data` lifecycle
- Type marshalling: `number` ↔ f64, `text` ↔ string, `bool` ↔ boolean, list ↔ Array, object ↔ Object
- Functions must be on `globalThis` — no module imports, no `require()`
- Opt-in via `naze.toml` `[scripts]` section; compiler warns on undeclared JS references

**Grammar:** 1 new action variant (`js_action`), 1 new data source variant (`data: js`).

### Browser Device APIs (Planned — M19e)

Declarative access to browser hardware APIs (geolocation, camera, notifications) using the `data` lifecycle pattern.

```naze
-- Geolocation (one-shot)
data location: device geolocation
-- location.loading → true while acquiring GPS
-- location.error → "Permission denied" if blocked
-- location.data → { latitude: 40.7, longitude: -74.0, accuracy: 10 }

-- Geolocation (continuous watch)
data location: device geolocation { watch: true }

-- Camera
data camera: device camera { facing: "user", width: 640, height: 480 }

-- Send browser notification (fire-and-forget action)
on click: notify "Order Shipped!" { body: "Your order is on its way.", icon: "icon.png" }
```

**Semantics:**
- `data: device API_NAME` — reuses `.loading`/`.error`/`.data` lifecycle
- `device` keyword signals "browser hardware API requiring permissions"
- Permission handling is implicit — denied permissions surface as `.error`
- `notify` action — requests notification permission on first use, then shows notification
- Supported APIs: `geolocation`, `camera`; extensible to `accelerometer`, `bluetooth`, etc.

**Grammar:** 1 new data source variant (`device`), 1 new action (`notify`).
