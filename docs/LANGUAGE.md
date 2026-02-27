# Naze Language Reference

Naze is a declarative UI language designed to replace HTML/CSS/JS. It compiles `.naze` source files into a custom binary IR (`app_data.bin`), which is deserialized and rendered via Canvas2D in the browser (WASM) or via tiny-skia on the desktop (native). Naze bypasses the DOM entirely.

The language is AI-native: there is one canonical form per concept, the grammar is compact (~157 rules), and every feature is designed for low token cost and constrained LLM decoding. For the long-term architecture vision, see [PROTOTYPE.md](PROTOTYPE.md).

---

## File Structure

A `.naze` file contains any combination of the following top-level constructs, in any order:

- Comments (`--`)
- `use` imports (local paths or `@scope/package/component`)
- `import` statements (WASM/JS modules)
- `app` blocks (entry files only)
- `page` blocks (routing)
- `component` definitions
- `function` definitions (pure, compile-time inlined)
- `server function` definitions (server-side logic)
- `template` definitions
- `theme` definitions (with named variants and inheritance)
- `let` bindings (immutable, compile-time)
- `state` declarations (mutable, reactive)
- `shared state` declarations (cross-page mutable state)
- `computed` declarations (read-only derived state)
- `storage` declarations (persistent browser storage)
- `param` declarations (URL query string binding)
- `timer` declarations (scheduled actions)
- `data` declarations (fetch, stream, JS, device)
- `prompt` declarations (AI providers)
- `match` statements (pattern matching)
- Elements (UI primitives)

A minimal Naze application:

```naze
app "Hello" {
  text "Hello, world!"
}
```

A typical application with state, components, and layout:

```naze
-- app.naze

use components/pill

app "My App" {
  state count = 0

  column padding: 20px, gap: 16px {
    heading "Hello"
    text "Count: {count}"

    rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
      text "Increment"
      on click: set count = count + 1
    }

    pill color: #22c55e
  }
}
```

Each file serves a single purpose: either an app entry point, a component definition, a template definition, a theme definition, or a test file. The compiler discovers all `.naze` files in the project directory recursively.

---

## Comments

Line comments start with `--` and extend to the end of the line. There are no block comments.

```naze
-- This is a full-line comment

heading "Hello" -- This is an inline comment
```

Comments are preserved in the AST for tooling (LSP, formatters) but stripped during compilation.

---

## Types and Values

Naze has six value types. Four are primitive types used in component parameter declarations (`number`, `text`, `bool`, `color`), and two are compound types (`list`, `object`).

| Type | Literal syntax | Examples |
|------|---------------|----------|
| `number` | Digits with optional decimal and unit | `20px`, `3.5em`, `50%`, `100`, `-8px`, `0.5` |
| `text` | Double-quoted string | `"Hello"`, `"Dashboard"`, `""` |
| `color` | `#` followed by 3-8 hex digits | `#fff`, `#ff0000`, `#2563eb`, `#00000080` |
| `bool` | `true` or `false` | `true`, `false` |
| `list` | Square brackets with comma-separated values | `["a", "b"]`, `[1, 2, 3]` |
| `object` | Curly braces with key-value pairs | `{ name: "Alice", age: 30 }` |

### Units

Numbers can have an optional unit suffix:

- `px` -- pixels (default for layout dimensions)
- `%` -- percentage (relative to parent dimension)
- `em` -- relative to the current font size

Numbers without a unit are treated as raw values. For layout properties (`width`, `height`, `padding`, `gap`), raw numbers are interpreted as pixels. For `font-size`, raw numbers are interpreted as points. For multiplier properties (`flex-grow`, `opacity`, `line-height`), raw numbers are used directly.

```naze
rect width: 200px, height: 100px     -- pixel dimensions
rect width: 50%, height: 100%        -- percentage of parent
text "Large" font-size: 2em          -- relative to parent font size
rect opacity: 0.8                    -- raw number (multiplier)
```

### Duration Units

Duration values are used in timers, debounce, throttle, and cache settings:

- `ms` -- milliseconds (e.g., `300ms`)
- `s` -- seconds (e.g., `5s`)
- `min` -- minutes (e.g., `5min`)
- `h` -- hours (e.g., `1h`)

```naze
timer auto-save: every 30s {
  trigger save-draft
}

on change debounce 300ms: trigger search
```

### String Interpolation

Strings can embed variable references using `{name}` syntax. Dot notation is supported for member access inside interpolation braces.

```naze
text "Hello, {name}!"
text "Current count: {count}"
text "{item.title} by {item.author}"
text "Total: ${cart.total}"
```

Interpolation works in any string literal position: element text, property values, data URLs, and notification bodies.

### Compound Values

**Lists** contain ordered sequences of values:

```naze
state items = ["Apple", "Banana", "Cherry"]
state numbers = [1, 2, 3, 4, 5]
state mixed = [{name: "Alice", score: 92}, {name: "Bob", score: 67}]
```

**Objects** contain key-value pairs:

```naze
state user = { name: "Alice", age: 30, active: true }
validate: { required: true, min-length: 3, max-length: 20 }
body: { name: name-input, email: email-input }
```

### References

Multi-segment references use dot notation to access nested values:

```naze
theme.colors.primary          -- theme token reference
user.name                     -- object field access
posts.data                    -- data lifecycle field
item.title                    -- iteration variable field
location.data.latitude        -- nested data access
```

References resolve at compile time for theme tokens and at runtime for state, data, and iteration variables.

---

## Expressions

Expressions appear in `set` actions, `computed` declarations, `if` conditions, `match` subjects, function bodies, and inline conditional property values.

### Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `count + 1` |
| `-` | Subtraction | `total - discount` |
| `*` | Multiplication | `price * quantity` |
| `/` | Division | `total / count` |
| `%` | Modulo | `(index + 1) % 5` |

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `status == "active"` |
| `!=` | Not equal | `count != 0` |
| `>` | Greater than | `score > 80` |
| `<` | Less than | `age < 18` |
| `>=` | Greater or equal | `quantity >= 1` |
| `<=` | Less or equal | `index <= 10` |

### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `logged-in && admin` |
| `\|\|` | Logical OR | `error \|\| timeout` |

### Grouping

Parentheses control evaluation order:

```naze
set index = (index + 1) % 5
set result = (a + b) * c
```

### Negation

In `set` actions, the `!` prefix toggles a boolean:

```naze
on click: set expanded = !expanded
on click: set visible = !visible
```

### Function Calls

Call pure functions or module functions in any expression context:

```naze
computed surface = area(width, height)
computed big = double(count)
```

### Member Access

Dot notation accesses fields on objects, data results, and iteration variables:

```naze
text "{user.name}"
text "Lat: {location.data.latitude}"
```

### Inline Conditionals

Properties can use inline `if`/`else` for conditional values:

```naze
row height: if expanded { 150px } else { 60px } {
  text "Content"
}

rect color: if active { #3b82f6 } else { #94a3b8 } {
  text "Toggle"
}
```

---

## Pipeline Operators

Pipeline operators transform data declaratively using the `|` (pipe) syntax. Pipelines read left to right: the source flows through each stage in sequence. Pipelines can be used in `computed` declarations, `each` bindings, and function bodies.

```naze
state students = [
  {name: "Alice", score: 92},
  {name: "Bob", score: 67},
  {name: "Carol", score: 85},
  {name: "Dave", score: 45},
  {name: "Eve", score: 78}
]

computed passing = students | filter score > 60
computed top-scores = students | filter score > 80 | sort-by name
computed total-score = students | map score | sum
computed student-count = students | count
computed top-3 = students | sort-by score | take 3
```

### Built-in Pipeline Functions

| Function | Arguments | Description |
|----------|-----------|-------------|
| `filter` | condition | Keep items where the condition is true |
| `map` | field or expression | Extract a field or transform each item |
| `sort-by` | field name | Sort items by field value (ascending) |
| `take` | number | Keep only the first N items |
| `sum` | *(none)* | Sum all numeric values in the list |
| `count` | *(none)* | Count the number of items in the list |
| `reduce` | accumulator expression, initial value | Fold items into a single value |
| `group-by` | field name | Group items into an object keyed by field value |
| `flatten` | *(none)* | Flatten nested lists one level deep |
| `distinct` | optional field name | Remove duplicate items (optionally by field) |

### filter

Keep items where a condition evaluates to true. The condition references fields on each item.

```naze
computed passing = students | filter score > 60
computed adults = people | filter age >= 18
computed active = users | filter status == "active"
```

### map

Extract a single field from each item, producing a list of values.

```naze
computed scores = students | map score
computed names = students | map name
```

### sort-by

Sort items by a field value in ascending order.

```naze
computed by-name = students | sort-by name
computed by-score = students | sort-by score
```

### take

Keep only the first N items from the list.

```naze
computed top-5 = students | sort-by score | take 5
computed recent = posts | take 10
```

### sum

Sum all numeric values in the list. Typically used after `map` to extract a numeric field.

```naze
computed total = students | map score | sum
computed revenue = orders | map amount | sum
```

### count

Count the number of items in the list.

```naze
computed total-students = students | count
computed active-count = users | filter active == true | count
```

### reduce

Fold items into a single accumulated value. Takes two arguments: an accumulator expression and an initial value. Inside the expression, `acc` refers to the accumulated value and `it` refers to the current item.

```naze
computed total = items | map score | reduce acc + it 0
computed product = numbers | reduce acc * it 1
```

### group-by

Group items into an object keyed by the specified field value. Each key maps to a list of matching items.

```naze
computed by-dept = employees | group-by dept
computed by-status = tasks | group-by status
```

### flatten

Flatten nested lists one level deep.

```naze
state nested = [[1, 2, 3], [4, 5], [6]]
computed flat = nested | flatten
-- flat = [1, 2, 3, 4, 5, 6]
```

### distinct

Remove duplicate items from a list. Optionally takes a field name for object lists.

```naze
state tags = ["rust", "wasm", "rust", "js", "wasm"]
computed unique = tags | distinct
-- unique = ["rust", "wasm", "js"]

computed unique-depts = employees | distinct dept
```

### Pipelines in each

Pipelines can be used inline in `each` statements to filter and transform the iteration source:

```naze
each student in students | filter score > 80 | sort-by name {
  text "{student.name}: {student.score}"
}

each n in nested | flatten {
  text "{n}"
}
```

### Chaining Multiple Stages

Stages chain naturally. Data flows left to right through each transformation:

```naze
computed result = items
  | filter score > 70
  | sort-by dept
  | take 10
  | map name
```

### shuffle

The `shuffle` pipeline function randomizes the order of items in a list using Fisher-Yates shuffle.

```naze
state items = [1, 2, 3, 4, 5]

on click: set items = items | shuffle
```

---

## Built-in Functions

Built-in functions are called with parentheses (unlike pipeline functions which use `|`). They can be used in any expression context.

| Function | Arguments | Returns | Description |
|----------|-----------|---------|-------------|
| `length(list)` | A list | Number | Returns the number of items in the list |
| `random(min, max)` | Two numbers | Number | Returns a random integer in [min, max] inclusive |

### length

Returns the number of items in a list:

```naze
state items = ["Apple", "Banana", "Cherry"]

text "Count: {length(items)}"

-- Use in expressions
on click: set items[length(items) - 1] = "Replaced last"
```

### random

Returns a random integer between min and max (inclusive):

```naze
state roll = 0

on click: set roll = random(1, 6)

text "Dice roll: {roll}"
```

Built-in functions can be used in `set` actions, `computed` declarations, `if` conditions, and any other expression context. They are distinct from pipeline functions -- `length()` takes arguments in parentheses, while `count` follows a `|` pipe.

---

## App Block

The `app` block is the entry point of a Naze application. Every entry file (`app.naze` by default) must contain exactly one `app` block. The string argument becomes the HTML `<title>` in the generated web output.

```naze
app "My Application" {
  -- state, data, computed, theme, pages, elements go here
}
```

The app block can contain any declaration or element:

```naze
app "Dashboard" {
  state sidebar-open = true
  shared state current-user = null
  data stats: fetch "/api/stats"

  computed has-user = current-user != null

  column padding: 20px, gap: 16px {
    heading "Dashboard"
    text "Welcome, {current-user.name}"
  }

  page "/" {
    heading "Home"
  }

  page "/settings" {
    heading "Settings"
  }
}
```

When the app contains `page` blocks, the top-level elements outside page blocks serve as the persistent shell (header, navigation) that wraps all pages. When there are no `page` blocks, all elements render directly.

---

## Components

Components are reusable UI pieces defined in `.naze` files. One component per file; the filename determines the component name.

### Defining a Component

A component has a name, optional typed parameters with optional defaults, and a body:

```naze
-- components/pill.naze

component pill(color: color, size: number = 60px) {
  rect width: size, height: 32px, color: color, radius: 16px
}
```

Parameters have a name, a type annotation, and an optional default value. Parameters without defaults are required at every call site.

**Parameter types:**

| Type | Accepts |
|------|---------|
| `text` | String literals |
| `number` | Numbers with or without units |
| `bool` | `true` or `false` |
| `color` | Hex color literals |

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

### Compile-Time Inlining

At compile time, component invocations are **inlined** -- the component body is substituted with parameter values filled in. The three `pill` calls above become three `rect` elements in the final render tree. There is no runtime component overhead, no virtual DOM diffing, and no component lifecycle.

### Import Resolution

`use components/pill` resolves to `components/pill.naze` relative to the project root. The compiler discovers all `.naze` files in the project directory recursively.

For registry packages, use scoped imports:

```naze
use @naze/ui-kit/button
```

This resolves through the dependency declared in `naze.toml`.

### Type Checking

The compiler checks component usage at compile time:

- Required props (those without default values) must be provided
- Prop types must match the parameter type declarations
- Unknown props produce a compile error
- Built-in element props are validated against known prop tables
- Interactive elements without `role` or `label` produce an accessibility warning

### Content Slots

Components can define insertion points for caller-provided content using `slot`. A bare `slot` is the default slot; `slot "name"` creates a named slot.

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

At the call site, children go into the default slot. Named slots are filled with `fill`:

```naze
use components/card

app "Slots Demo" {
  card title: "My Card" {
    text "This goes in the default slot"

    fill "footer" {
      text "Custom footer content"
    }
  }
}
```

Slots with a body block provide fallback content used when the caller does not fill them. In the example above, the "footer" slot has a default `text "Default footer"` that renders when no `fill "footer"` is provided.

### Component Events (Emit)

Components can emit custom events to their parent using the `emit` action. The parent handles emitted events with `on event-name: action` in the component's child block.

**Component definition:**

```naze
-- components/toggle-btn.naze

component toggle-btn(label: text) {
  rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
    text "{label}" color: #ffffff
    on click: emit toggled
  }
}
```

**Parent usage:**

```naze
use components/toggle-btn

app "Demo" {
  state sidebar-open = true

  toggle-btn label: "Toggle Sidebar" {
    on toggled: set sidebar-open = sidebar-open == false
  }

  if sidebar-open {
    rect width: 200px, height: 300px, color: #e2e8f0, radius: 8px, padding: 16px {
      text "Sidebar Content"
    }
  }
}
```

**Semantics:**

- `emit event-name` fires a custom event from within a component
- The parent handles it with `on event-name: action` in the component's child block
- Events are resolved at compile time during component inlining -- no runtime event propagation overhead
- The compiler validates that `emit` names match parent `on` handlers

---

## Templates

Templates are reusable layout scaffolds with named content regions. They define the structure; callers fill in the content via `fill`.

### Defining a Template

```naze
template two-panel(left, right) {
  row gap: 16px {
    column width: 300px {
      slot "left"
    }
    column flex-grow: 1 {
      slot "right"
    }
  }
}
```

Template parameters name the content slots. The template body defines layout structure using `slot "name"` to mark where caller content goes.

### Using a Template

```naze
two-panel {
  fill "left" {
    column gap: 8px {
      text "Navigation"
      text "Item 1"
      text "Item 2"
      text "Item 3"
    }
  }
  fill "right" {
    column gap: 8px {
      heading "Main Content"
      text "This is the main content area."
      text "It uses a template for consistent layout."
    }
  }
}
```

### Built-in Templates

Naze includes common layout templates:

| Template | Slots | Description |
|----------|-------|-------------|
| `app-shell` | `toolbar`, `sidebar`, `main`, `footer` | Standard application shell |
| `dashboard` | `header`, `cards`, `detail-panel` | Dashboard with card grid |
| `sidebar-layout` | `nav`, `content` | Sidebar navigation with content area |
| `split-view` | `left`, `right` | Side-by-side panels |
| `centered` | `content` | Centered content block |

### How Templates Work

Templates are expanded to spatial primitives at compile time. A `two-panel` call becomes `row` and `column` elements with caller content substituted at slot positions. There is no runtime template overhead.

---

## Pure Functions

Functions define reusable expressions that are inlined at compile time. Function bodies are single expressions -- they can use pipeline operators but have no side effects and no state access.

```naze
function area(w: number, h: number) -> number {
  w * h
}

function double(x: number) -> number {
  x + x
}
```

Call functions in any expression context:

```naze
state width = 200
state height = 100

computed surface = area(width, height)
computed big-width = double(width)
```

Functions can use pipeline syntax in their body:

```naze
function average-score(items: list) -> number {
  items | map score | sum
}
```

**Semantics:**

- Functions are **pure** -- no side effects, no access to state variables
- Bodies are single expressions (including pipeline expressions)
- **Compile-time inlined** -- the function body is substituted with argument values at compile time; no runtime function call overhead
- Parameters are typed; return type is declared after `->`
- Available types: `text`, `number`, `bool`, `color`

**Syntax:** `function name(param: type, ...) -> return-type { expression }`

---

## Elements

Elements are the building blocks of a Naze UI. An element consists of a name, optional inline text, optional properties, and an optional child block:

```naze
element "optional text" prop: value, prop: value {
  -- children
}
```

Elements that have no children and no block end at the newline:

```naze
text "Hello, world!"
rect width: 100px, height: 100px, color: #3b82f6
```

### Common Props

The following properties are accepted by most elements:

| Prop | Type | Description |
|------|------|-------------|
| `width` | number | Element width |
| `height` | number | Element height |
| `padding` | number | Inner padding on all sides |
| `gap` | number | Space between children |
| `opacity` | number | Transparency (0.0 to 1.0) |
| `cursor` | text | Mouse cursor style |
| `tab-index` | number | Keyboard navigation order |
| `role` | text | ARIA role for accessibility |
| `label` | text | ARIA label for accessibility |
| `id` | text | Element identifier (for `scroll-to`) |

---

### Layout Containers

Layout containers accept children and control how they are positioned.

#### row

Lays children out **horizontally** (left to right). Children are placed side by side. Spacers in a row expand horizontally to fill remaining width.

```naze
row gap: 12px, padding: 16px {
  rect width: 50px, height: 50px, color: #ff0000
  rect width: 50px, height: 50px, color: #00ff00
  rect width: 50px, height: 50px, color: #0000ff
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `padding` | number | Inner padding on all sides |
| `gap` | number | Space between children |
| `width` | number | Container width |
| `height` | number | Container height |
| `color` | color | Background color |
| `columns` | number | Column count (for grid-like behavior) |
| `align` | text | Cross-axis alignment: `start`, `center`, `end`, `stretch` |
| `justify` | text | Main-axis alignment: `start`, `center`, `end`, `space-between`, `space-around`, `space-evenly` |
| `wrap` | bool | Wrap children to next line when they exceed width |
| `flex-grow` | number | Grow factor to fill remaining space |
| `flex-shrink` | number | Shrink factor when children overflow |
| `min-width` | number | Minimum width constraint |
| `max-width` | number | Maximum width constraint |
| `min-height` | number | Minimum height constraint |
| `max-height` | number | Maximum height constraint |
| `responsive` | number | Below this viewport width, switch to vertical stacking |
| `collapsible` | number | Below this viewport width, hide the element |
| `cursor` | text | Mouse cursor style on hover |
| `shadow` | text | Box shadow preset or custom value |
| `overflow` | text | Content clipping: `visible`, `hidden`, `clip` |
| `gradient` | text | Gradient fill (overrides `color`) |
| `transform` | text | 2D transformation |
| `opacity` | number | Transparency (0.0 to 1.0) |
| `transition` | text | Animated property transitions |
| `animate` | text | Keyframe animation sequence |

#### column

Lays children out **vertically** (top to bottom). Children are stacked. Spacers in a column expand vertically to fill remaining height.

```naze
column gap: 16px, padding: 20px {
  heading "Title"
  text "Body text"
  text "More text"
}
```

**Props:** Same as `row`. When `responsive` is set on a column, it has no visual effect (columns are already vertical).

#### stack

Layers children on top of each other at the same position. All children share the same origin point. Useful for overlays, backgrounds, and layered compositions.

```naze
stack width: 200px, height: 200px {
  rect width: 200px, height: 200px, color: #e2e8f0
  rect width: 100px, height: 100px, color: #3b82f6
  text "On top"
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `padding` | number | Inner padding on all sides |
| `gap` | number | Space between children (offset for layering) |
| `width` | number | Container width |
| `height` | number | Container height |
| `color` | color | Background color |
| `columns` | number | Column count |
| `align` | text | Alignment: `start`, `center`, `end`, `stretch` |
| `justify` | text | Justification: `start`, `center`, `end`, `space-between`, `space-around`, `space-evenly` |
| `cursor` | text | Mouse cursor style |
| `shadow` | text | Box shadow |
| `overflow` | text | Content clipping |
| `gradient` | text | Gradient fill |
| `transform` | text | 2D transformation |
| `opacity` | number | Transparency |

#### grid

Lays children out in a wrapping grid. The `columns` prop controls how many columns. Column width is computed automatically from available space.

```naze
grid columns: 3, gap: 8px {
  rect width: 60px, height: 60px, color: #ff0000
  rect width: 60px, height: 60px, color: #00ff00
  rect width: 60px, height: 60px, color: #0000ff
  rect width: 60px, height: 60px, color: #ffff00
  rect width: 60px, height: 60px, color: #ff00ff
  rect width: 60px, height: 60px, color: #00ffff
}
```

**Props:** Same as `row`. The `columns` prop is particularly important here -- it sets the number of columns in the grid. When `responsive` is set, the grid switches to a single column below the breakpoint.

#### container

A styled box that lays children out vertically (like `column`). Supports background color, border radius, and border styling. Commonly used for cards and panels.

```naze
container padding: 16px, color: #eff6ff, radius: 8px {
  heading "Card Title"
  text "Card body content goes here."
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `padding` | number | Inner padding |
| `width` | number | Container width |
| `height` | number | Container height |
| `radius` | number | Border radius (rounded corners) |
| `color` | color | Background color |
| `border` | number | Border width |
| `border-color` | color | Border color |
| `opacity` | number | Transparency |
| `collapsible` | number | Hide below viewport width |
| `cursor` | text | Mouse cursor style |
| `shadow` | text | Box shadow |
| `overflow` | text | Content clipping |
| `gradient` | text | Gradient fill |
| `transform` | text | 2D transformation |
| `transition` | text | Animated transitions |

#### spacer

An invisible element that expands to fill remaining space in its parent's layout direction. In a `row`, it expands horizontally. In a `column`, it expands vertically. If given explicit dimensions, it uses those instead.

```naze
row {
  text "Left"
  spacer
  text "Right"
}

column {
  heading "Top"
  spacer
  text "Bottom"
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `width` | number | Explicit width (overrides flex) |
| `height` | number | Explicit height (overrides flex) |
| `flex-grow` | number | Growth factor |
| `flex-shrink` | number | Shrink factor |
| `cursor` | text | Mouse cursor style |

#### scroll

A scrollable container. Content that exceeds the container's dimensions can be scrolled with the mouse wheel. Scrollbars render automatically when content overflows.

```naze
scroll height: 400px {
  column gap: 8px {
    text "Item 1"
    text "Item 2"
    text "Item 3"
    -- ... many more items that exceed 400px
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `width` | number | Container width |
| `height` | number | Container height |
| `overflow` | text | Scroll direction: `"x"`, `"y"`, `"both"` |
| `padding` | number | Inner padding |
| `radius` | number | Border radius |
| `border` | number | Border width |
| `border-color` | color | Border color |
| `opacity` | number | Transparency |
| `color` | color | Background color |
| `cursor` | text | Mouse cursor style |

The `on scroll` event can be attached to scroll containers:

```naze
scroll height: 300px {
  on scroll throttle 100ms: set scroll-pos = scroll-pos + 1
  column gap: 4px {
    -- content
  }
}
```

#### separator

A thin horizontal line divider. Renders as a 1px line spanning the available width.

```naze
column gap: 16px {
  text "Section A"
  separator
  text "Section B"
}
```

---

### Text Elements

#### text

Renders body text. The string content is passed inline after the element name. Default font size is 16px.

```naze
text "Hello, world!"
text "Colored text" color: #666666, font-size: 14px
text "Count: {count}"
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `color` | color | Text color |
| `font-size` | number | Font size (default: 16px) |
| `opacity` | number | Transparency |
| `cursor` | text | Mouse cursor style |
| `text-decoration` | text | Decoration: `underline`, `line-through`, `overline`, `none` |
| `text-align` | text | Horizontal alignment: `start`, `center`, `end`, `right` |
| `line-height` | number | Line height multiplier (e.g., 1.5) |
| `letter-spacing` | number | Space between characters |
| `text-overflow` | text | Overflow behavior: `clip`, `ellipsis` |
| `transform` | text | 2D transformation |
| `tab-index` | number | Keyboard navigation order |

#### heading

Renders heading text. Default font size is 24px, rendered bold.

```naze
heading "Page Title"
heading "Small heading" font-size: 18px, color: #1e293b
heading "Section {section-number}"
```

**Props:** Same as `text`.

#### link

A clickable navigation link for routing between pages. Uses the router's History API integration.

```naze
link "About", to: "/about"
link "Home", to: "/"
link "Contact Us", to: "/contact"
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `to` | text | Target route path (required) |
| `color` | color | Link text color |
| `cursor` | text | Mouse cursor style |
| `text-decoration` | text | Text decoration |
| `text-align` | text | Text alignment |
| `line-height` | number | Line height multiplier |

Links can contain child elements for more complex link layouts:

```naze
link "Dashboard", to: "/dashboard" {
  row gap: 8px {
    rect width: 16px, height: 16px, color: #3b82f6
    text "Dashboard"
  }
}
```

#### code

Renders text in a monospace font. Used for code blocks and inline code.

```naze
code "const x = 42;"
code "npm install naze" font-size: 14px, color: #1e293b
```

**Props:** Same as `text`.

---

### Media Elements

#### image

Displays an image. Images are loaded asynchronously and cached by the runtime.

```naze
image src: "photo.jpg", width: 200px, height: 150px
image src: "https://example.com/avatar.png", width: 64px, height: 64px, fit: "cover"
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `src` | text | Image URL or local path (required) |
| `width` | number | Display width |
| `height` | number | Display height |
| `fit` | text | Object fit: `contain`, `cover`, `fill` |
| `alt` | text | Alternative text for accessibility |
| `cursor` | text | Mouse cursor style |
| `transform` | text | 2D transformation |
| `opacity` | number | Transparency |

---

### Input Elements

#### input

A text input with two-way state binding. The `bind` prop connects the input to a state variable -- typing in the input updates the state, and changing the state updates the input.

```naze
state name = ""

input bind: name, placeholder: "Type your name..."
text "Hello, {name}!"
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `bind` | reference | State variable for two-way binding (required) |
| `placeholder` | text | Placeholder text when empty |
| `type` | text | Input type: `"text"`, `"number"`, `"email"`, `"password"`, `"file"` |
| `accept` | text | MIME filter for file type (e.g., `"image/*"`) |
| `max-size` | text | Size limit for file type (e.g., `"5mb"`) |
| `validate` | object | Validation rules object |
| `tab-index` | number | Keyboard navigation order |
| `width` | number | Input width |
| `height` | number | Input height |
| `font-size` | number | Font size |
| `color` | color | Text color |
| `border` | number | Border width |
| `border-color` | color | Border color |
| `radius` | number | Border radius |

Input types:

```naze
input bind: email, type: "email", placeholder: "Email address"
input bind: password, type: "password", placeholder: "Password"
input bind: age, type: "number", placeholder: "Age"
input bind: avatar, type: "file", accept: "image/*", max-size: "5mb"
```

#### textarea

Multi-line text input for longer content. Supports the same two-way binding and validation as `input`.

```naze
state bio = ""

textarea bind: bio, placeholder: "Tell us about yourself...", rows: 4, max-length: 500

if bio {
  text "Preview: {bio}"
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `bind` | reference | State variable for two-way binding (required) |
| `placeholder` | text | Placeholder text |
| `rows` | number | Visible height in text rows (default: 4) |
| `max-length` | number | Character limit |
| `width` | number | Input width |
| `height` | number | Input height |
| `font-size` | number | Font size |
| `color` | color | Text color |
| `border` | number | Border width |
| `border-color` | color | Border color |
| `radius` | number | Border radius |
| `line-height` | number | Line height multiplier |
| `letter-spacing` | number | Character spacing |
| `opacity` | number | Transparency |
| `tab-index` | number | Keyboard navigation order |
| `cursor` | text | Mouse cursor style |
| `text-align` | text | Text alignment |
| `shadow` | text | Box shadow |
| `transform` | text | 2D transformation |
| `validate` | object | Validation rules object |

**Semantics:**

- Two-way binding via `bind` (same as other form elements)
- Enter key inserts a newline (unlike `input`, where Enter unfocuses)
- Default sizing: 200px wide, height computed from `rows` times line height plus padding

#### checkbox

A boolean toggle bound to a state variable.

```naze
state agreed = false

checkbox bind: agreed, label: "I agree to the terms"

if agreed {
  text "Thank you for agreeing!"
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `bind` | reference | Boolean state variable (required) |
| `label` | text | Label text displayed next to the checkbox |

#### radio

A radio button for selecting one value from a group. Multiple radio elements sharing the same `bind` form a group.

```naze
state choice = "a"

radio bind: choice, value: "a", label: "Option A"
radio bind: choice, value: "b", label: "Option B"
radio bind: choice, value: "c", label: "Option C"

text "Selected: {choice}"
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `bind` | reference | State variable for the group (required) |
| `value` | text | Value to set when selected (required) |
| `label` | text | Label text displayed next to the radio button |

#### select

A dropdown select element. Contains `option` children that define the choices.

```naze
state color = "red"

select bind: color {
  option "Red" value: "red"
  option "Green" value: "green"
  option "Blue" value: "blue"
}

text "Selected color: {color}"
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `bind` | reference | State variable (required) |

#### option

A child of `select`. Defines one choice in the dropdown.

```naze
option "Display Label" value: "stored-value"
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `value` | text | Value stored in the bound state when selected (required) |

### Validation

Attach validation rules to `input` and `textarea` elements. The compiler automatically generates `{name}_valid` (bool) and `{name}_error` (text) state variables for each validated input.

```naze
state username = ""
state email = ""
state age = ""

input bind: username, placeholder: "Username", validate: {
  required: true,
  min-length: 3,
  max-length: 20
}

input bind: email, type: "email", placeholder: "Email", validate: {
  required: true
}

input bind: age, type: "number", placeholder: "Age", validate: {
  required: true,
  min: 18,
  max: 120
}

if username_error {
  text "{username_error}" color: #dc2626
}
if username_valid {
  text "Username is valid" color: #16a34a
}
```

**Validation rules:**

| Rule | Type | Description |
|------|------|-------------|
| `required` | bool | Field must not be empty |
| `min-length` | number | Minimum string length |
| `max-length` | number | Maximum string length |
| `pattern` | text | Regex pattern to match |
| `min` | number | Minimum numeric value |
| `max` | number | Maximum numeric value |

**Auto-generated state:**

For an input with `bind: username`, the compiler generates:
- `username_valid` -- boolean, `true` when all rules pass
- `username_error` -- text, the first failing validation message (empty string when valid)

---

### Overlay Elements

#### overlay

Renders content above normal layout flow. Used for dialogs, modals, dropdowns, tooltips, popovers, toasts, and menus.

```naze
state show-dialog = false

rect padding: 8px, radius: 4px, color: #3b82f6 {
  text "Open Dialog" color: #ffffff
  on click: set show-dialog = true
}

if show-dialog {
  overlay focus-trap: true, scroll-lock: true {
    rect width: 400px, padding: 24px, radius: 12px, color: #ffffff, shadow: "xl" {
      heading "Dialog Title"
      text "Dialog content goes here."

      rect padding: 8px, radius: 4px, color: #ef4444 {
        text "Close" color: #ffffff
        on click: set show-dialog = false
      }
    }
    on click-outside: set show-dialog = false
  }
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `focus-trap` | bool | Trap keyboard focus within the overlay |
| `scroll-lock` | bool | Prevent background scrolling |
| `dismiss-on-escape` | bool | Close on Escape key press |
| `anchor` | text | Positioning reference element ID |
| `anchor-placement` | text | Placement relative to anchor |
| `width` | number | Overlay width |
| `height` | number | Overlay height |
| `color` | color | Background color |
| `radius` | number | Border radius |
| `padding` | number | Inner padding |
| `border` | number | Border width |
| `border-color` | color | Border color |
| `opacity` | number | Transparency |
| `shadow` | text | Box shadow |
| `cursor` | text | Mouse cursor style |

**Events:**

- `on click-outside: action` -- fires when the user clicks outside the overlay content

---

### Special Elements

#### rect

Draws a colored rectangle. The most versatile visual primitive.

```naze
rect width: 80px, height: 80px, color: #2563eb, radius: 8px
rect width: 200px, height: 50px, color: #10b981, radius: 4px, border: 2px, border-color: #065f46
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `width` | number | Rectangle width |
| `height` | number | Rectangle height |
| `color` | color | Fill color |
| `radius` | number | Corner radius |
| `border` | number | Border width |
| `border-color` | color | Border color |
| `opacity` | number | Transparency |
| `tab-index` | number | Keyboard navigation order |
| `cursor` | text | Mouse cursor style |
| `shadow` | text | Box shadow |
| `gradient` | text | Gradient fill |
| `transform` | text | 2D transformation |
| `transition` | text | Animated transitions |
| `animate` | text | Keyframe animations |

`rect` can contain children, making it useful as a button or card:

```naze
rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
  text "Click me" color: #ffffff
  on click: set count = count + 1
}
```

---

## Layout System

Layout is computed top-down from the viewport dimensions. Each element receives available width and height from its parent.

### Dimension Resolution

Dimensions resolve in this priority order:

1. **Explicit dimensions** -- `width: 200px`, `height: 100px` are used as-is
2. **Percentage dimensions** -- `width: 50%` resolves relative to parent's available width
3. **Implicit dimensions** -- computed from children when no explicit size is given

Implicit dimension rules vary by container type:

| Container | Width | Height |
|-----------|-------|--------|
| `row` | Sum of children widths + gaps | Tallest child |
| `column` | Widest child | Sum of children heights + gaps |
| `container` | Widest child | Sum of children heights + gaps |
| `grid` | Available width | Sum of row heights + gaps |
| `stack` | Widest child | Tallest child |
| `text` / `heading` | Measured from text and font size | Measured from text and font size |
| `rect` | 0 if not specified | 0 if not specified |

### Padding and Gap

- **Padding** insets the content area on all sides. A `column` with `padding: 20px` has 20px between its edge and its children.
- **Gap** adds space between children (not before the first or after the last). A `row` with `gap: 12px` has 12px between each pair of adjacent children.

```naze
column padding: 20px, gap: 16px {
  text "First"   -- 20px from top, 20px from left
  text "Second"  -- 16px gap from "First"
  text "Third"   -- 16px gap from "Second"
}
```

### Spacer

`spacer` fills remaining space in its parent's layout direction:

```naze
row {
  text "Left"
  spacer          -- fills all horizontal space between left and right
  text "Right"
}

column height: 400px {
  heading "Header"
  spacer          -- fills vertical space
  text "Footer"
}
```

### Flex Properties

Flex properties control how elements grow and shrink within their parent:

```naze
row {
  rect width: 100px, height: 50px, color: #ff0000
  rect flex-grow: 1, height: 50px, color: #00ff00  -- fills remaining space
  rect width: 100px, height: 50px, color: #0000ff
}
```

| Prop | Description |
|------|-------------|
| `flex-grow: N` | Element grows to fill remaining space, proportional to N |
| `flex-shrink: N` | Element shrinks proportionally when children overflow parent |
| `wrap: true` | On `row` -- children wrap to the next line when they exceed available width |

### Size Constraints

Clamp computed dimensions to minimum and maximum bounds:

| Prop | Description |
|------|-------------|
| `min-width` | Minimum width (element will not shrink below this) |
| `max-width` | Maximum width (element will not grow beyond this) |
| `min-height` | Minimum height |
| `max-height` | Maximum height |

```naze
column min-width: 200px, max-width: 600px {
  text "Responsive content area"
}
```

### Alignment

| Prop | Axis | Values |
|------|------|--------|
| `align` | Cross-axis | `start`, `center`, `end`, `stretch` |
| `justify` | Main-axis | `start`, `center`, `end`, `space-between`, `space-around`, `space-evenly` |

For a `row`, the main axis is horizontal and the cross axis is vertical. For a `column`, the main axis is vertical and the cross axis is horizontal.

```naze
row align: center, justify: space-between {
  text "Left"
  text "Center"
  text "Right"
}

column align: center, justify: center, height: 400px {
  text "Perfectly centered"
}
```

**justify values:**

| Value | Description |
|-------|-------------|
| `start` | Pack children at the start |
| `center` | Center children |
| `end` | Pack children at the end |
| `space-between` | Equal space between children, none at edges |
| `space-around` | Equal space around each child |
| `space-evenly` | Equal space between children and at edges |

---

## Responsive Layout

Naze handles responsive design through layout-level breakpoint properties. No JavaScript media queries are needed -- the layout engine evaluates breakpoints directly during each layout pass.

### Responsive Breakpoints

The `responsive` property on `row` and `grid` containers triggers a layout mode change at a viewport width breakpoint. A `row` with `responsive: 768px` behaves as a `column` when the viewport is narrower than 768px.

```naze
row responsive: 768px, gap: 16px {
  column flex-grow: 1 {
    heading "Main Content"
    text "On narrow screens, the sidebar stacks below."
  }
  column width: 280px {
    heading "Sidebar"
    text "Navigation or supplementary info."
  }
}
```

A `grid` with `responsive: 768px` switches to a single column below the breakpoint:

```naze
grid columns: 3, responsive: 768px, gap: 12px {
  rect width: 100px, height: 80px, color: #2563eb
  rect width: 100px, height: 80px, color: #22c55e
  rect width: 100px, height: 80px, color: #f59e0b
}
```

### Collapsible Panels

The `collapsible` property hides an element entirely when the viewport is narrower than the specified width. Useful for hiding optional panels on smaller screens.

```naze
column collapsible: 1200px {
  text "Extra detail panel (visible on wide screens only)"
}
```

**Semantics:**

- `responsive: Npx` on `row` or `grid` -- below N pixels viewport width, layout switches to vertical stacking
- `collapsible: Npx` -- element is hidden when viewport width is less than N pixels
- Breakpoints are evaluated during each layout pass (viewport width check)
- No JavaScript media queries -- the layout engine handles breakpoints directly

---

## State and Reactivity

Naze provides several declaration forms for managing application state. Each form has different scope, mutability, and persistence characteristics.

### Let Bindings

`let` creates compile-time constants. They are immutable and inlined at compile time.

```naze
let title = "My Counter"
let colors = ["#3b82f6", "#10b981", "#f59e0b"]
let max-retries = 3
```

Let bindings cannot be changed at runtime. They are substituted directly into the render tree during compilation.

### State

`state` creates mutable, reactive variables. When a state variable changes (via `set`), any element referencing it re-renders.

```naze
state count = 0
state name = ""
state items = ["Apple", "Banana", "Cherry"]
state expanded = false
state user = { name: "Alice", role: "admin" }
```

State variables can hold any value type: numbers, text, booleans, lists, and objects.

Mutate state with the `set` action:

```naze
on click: set count = count + 1
on click: set expanded = !expanded
on click: set name = "Bob"
```

State is scoped to the page or component where it is declared. It resets when navigating away from the page.

### Computed

`computed` creates read-only derived values that auto-update when their dependencies change. Replaces the need for React's `useMemo` or repeated inline expressions.

```naze
state quantity = 1
state price = 25

computed total = quantity * price
computed discounted = total * 0.9
computed has-items = quantity > 0
computed full-name = "{first-name} {last-name}"
```

Computed values support pipeline syntax:

```naze
computed passing = students | filter score > 60
computed top-3 = students | sort-by score | take 3
computed total-score = students | map score | sum
```

**Semantics:**

- Read-only -- cannot be the target of `set`
- Dependencies tracked at compile time (the compiler scans the expression for state and computed references)
- Re-evaluates only when a dependency changes
- Can reference other `computed` values (the compiler validates no cycles exist)
- No runtime overhead beyond the expression evaluation

### Shared State

`shared state` creates state that persists across page navigation and is accessible from any page or component in the app. Replaces the need for React Context, Redux, or Zustand.

```naze
shared state current-user = null
shared state auth-token = ""
shared state cart-items = []
shared state notification-count = 0
```

**Semantics:**

- Same as `state` but not scoped to a page -- persists across `navigate` actions
- Changes trigger re-render on any page that references the shared state variable
- Mutated with the same `set` action as regular state
- Declared at the app level (inside the `app` block)

```naze
app "Shop" {
  shared state cart = []

  page "/" {
    text "Items in cart: {cart}"
    rect padding: 8px, color: #2563eb, radius: 4px {
      text "Add item" color: #ffffff
      on click: set cart = ["Widget"]
    }
  }

  page "/cart" {
    text "Cart contents: {cart}"
  }
}
```

### Storage

`storage` creates reactive state bound to browser localStorage or sessionStorage. Values persist across browser sessions (localStorage) or within the current tab (sessionStorage).

```naze
storage theme-preference: local "theme" default: "light"
storage cart: local "shopping-cart" default: []
storage recent-searches: local "recent" default: []
storage session-id: session "sid" default: ""
storage font-size: session "font-size" default: 16
```

**Syntax:** `storage name: (local | session) "key" default: value`

**Semantics:**

- Behaves like `state` -- reactive, triggers re-render on change
- Initialized from browser storage on load; uses `default` if key is not found
- Changes via `set` auto-sync to storage: `set theme-preference = "dark"` writes to localStorage
- JSON serialization for non-string values (lists, objects)
- `local` = localStorage (persists across browser sessions)
- `session` = sessionStorage (persists within the current tab only)

```naze
app "Theme Switcher" {
  storage theme: local "theme-preference" default: "light"
  storage font-size: session "font-size" default: 16

  column padding: 20px, gap: 16px {
    heading "Settings"
    text "Theme: {theme}"
    text "Font size: {font-size}px"

    row gap: 8px {
      rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
        text "Light"
        on click: set theme = "light"
      }
      rect width: 120px, height: 40px, color: #1e293b, radius: 8px {
        text "Dark"
        on click: set theme = "dark"
      }
    }

    row gap: 8px {
      rect width: 80px, height: 40px, color: #059669, radius: 8px {
        text "A+"
        on click: set font-size = font-size + 2
      }
      rect width: 80px, height: 40px, color: #dc2626, radius: 8px {
        text "A-"
        on click: set font-size = font-size - 2
      }
    }
  }
}
```

### Params

`param` declares reactive state bound to URL query string parameters. Enables bookmarkable, shareable search, filter, and pagination state.

```naze
param page: number default: 1
param query: text default: ""
param sort: text default: "newest"
param dark: bool default: false
```

**Syntax:** `param name: type default: value`

**Semantics:**

- Behaves like `state` -- reactive, usable in expressions and templates
- Two-way bound to the URL query string via `replaceState`
- Type-validated: `number` params parse from the query string to numeric, `text` stays as string, `bool` maps from `"true"`/`"false"`
- `default` used when the param is absent from the URL
- Browser back/forward updates param values and triggers re-render

```naze
app "Search" {
  param page: number default: 1
  param q: text default: ""

  column padding: 20px, gap: 16px {
    text "Page: {page}"
    text "Query: {q}"

    rect padding: 8px, color: #2563eb, radius: 4px {
      text "Next page" color: #ffffff
      on click: set page = page + 1
    }
  }
}
```

Changing `page` via `set page = page + 1` updates the URL to `?page=2&q=` automatically.

### Timers

`timer` schedules actions based on time. Two forms: `after` (one-shot) and `every` (repeating).

```naze
timer toast-dismiss: after 5s {
  set show-toast = false
}

timer tick: every 1s {
  set seconds = seconds + 1
}

timer auto-save: every 30s {
  trigger save-draft
}
```

**Syntax:** `timer name: (after | every) duration { action }`

**Semantics:**

- `after duration { action }` -- executes once after the delay, then stops
- `every duration { action }` -- repeats at the given interval until the page/component unmounts
- Duration units: `ms`, `s`, `min`, `h` (e.g., `300ms`, `5s`, `30min`, `1h`)
- Timers are automatically cleaned up when their page or component is no longer rendered
- Timer actions: `start name` resumes a stopped timer, `stop name` pauses a running timer

```naze
app "Timer Demo" {
  state seconds = 0
  state toast-visible = true

  timer tick: every 1s {
    set seconds = seconds + 1
  }

  timer hide-toast: after 5s {
    set toast-visible = false
  }

  column padding: 20px, gap: 16px {
    heading "Timer Demo"
    text "Elapsed: {seconds}s"

    if toast-visible {
      rect width: 300px, height: 50px, color: #059669, radius: 8px {
        text "Welcome! This toast disappears after 5s"
      }
    }
  }
}
```

---

## Events and Actions

### Events

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
| `on hover` | Mouse pointer enters the element |
| `on change` | Form input value changes |
| `on keypress` | Key pressed while element is focused |
| `on scroll` | Scroll position changes in a scroll container |
| `on drag-start` | Drag operation begins on a draggable element |
| `on drag-over` | A dragged item is over a drop target |
| `on drop` | An item is dropped on a drop target |
| `on click-outside` | Click occurs outside an overlay |
| `on context-menu` | Right-click on the element |
| `on pointer-move` | Pointer moves over the element |
| `on arrow-up` | Up arrow key while element is focused |
| `on arrow-down` | Down arrow key while element is focused |
| `on arrow-left` | Left arrow key while element is focused |
| `on arrow-right` | Right arrow key while element is focused |

Custom events emitted from components can also be handled:

```naze
my-component {
  on toggled: set open = !open
  on selected: set current = "new"
}
```

### Event Modifiers

Event handlers can have `debounce` or `throttle` modifiers to control firing rate:

```naze
-- Debounce: wait for 300ms of inactivity before firing
input bind: search, on change debounce 300ms: trigger search-results

-- Throttle: fire at most once per 100ms
on scroll throttle 100ms: set scroll-pos = scroll-pos + 1
```

| Modifier | Behavior |
|----------|----------|
| `debounce Nms` | Delays action until N milliseconds of inactivity |
| `throttle Nms` | Executes at most once per N milliseconds |

### Actions

| Action | Syntax | Description |
|--------|--------|-------------|
| `set` | `set var = expression` | Update a state variable |
| `navigate` | `navigate "/path"` | Route to a different page |
| `scroll-to` | `scroll-to "element-id"` | Scroll to an element by its `id` prop |
| `log` | `log expression` | Output to browser console (web) or stderr (native) |
| `trigger` | `trigger data-name` | Trigger a manual data fetch |
| `copy` | `copy expression` | Copy value to the clipboard |
| `send` | `send stream-name expression` | Send a message on a WebSocket stream |
| `js` | `js "func"(args)` | Call a JavaScript function |
| `js` (with return) | `js "func"(args) -> state-var` | Call JS function and store return value |
| `notify` | `notify "title" { body: "text", icon: "url" }` | Send a browser notification |
| `emit` | `emit event-name` | Emit a custom event from a component |
| `set-theme` | `set-theme "name"` | Switch the active named theme |
| `start` | `start timer-name` | Start or resume a timer |
| `stop` | `stop timer-name` | Stop a running timer |

### set

The most common action. Updates a state, shared state, storage, or param variable:

```naze
on click: set count = count + 1
on click: set expanded = !expanded
on click: set name = "Alice"
on click: set index = (index + 1) % 5
on click: set total = price * quantity
```

#### Index Assignment

Set a specific item in a list by index:

```naze
state items = ["Apple", "Banana", "Cherry"]

on click: set items[0] = "Avocado"
on click: set items[length(items) - 1] = "New last"
```

### List Mutation Actions

#### append

Add an item to the end of a list:

```naze
state items = ["Apple", "Banana"]

on click: append "Cherry" to items
```

#### remove

Remove an item from a list by index:

```naze
state items = ["Apple", "Banana", "Cherry"]

on click: remove 0 from items
on click: remove length(items) - 1 from items
```

### Multi-Action Handlers

Multiple actions can be combined in a single event handler, separated by commas:

```naze
on click: set count = count + 1, set total = total + price
on click: set loading = true, trigger fetch-data
```

### Conditional Actions

Actions can be conditionally executed using `if` inside an event handler:

```naze
on click: if count > 0 { set count = count - 1 } else { set error = "Cannot go below zero" }
```

### navigate

Route to a different page within the app:

```naze
on click: navigate "/about"
on click: navigate "/dashboard"
on click: navigate "/"
```

### scroll-to

Scroll to an element identified by its `id` prop:

```naze
heading "Top" id: "page-top"

-- later in the page:
rect padding: 8px, color: #3b82f6, radius: 4px {
  text "Back to top" color: #ffffff
  on click: scroll-to "page-top"
}
```

### log

Output values to the browser console (web) or stderr (native). Useful for debugging:

```naze
on click: log "button clicked"
on click: log count
on click: log "count is: {count}"
```

### trigger

Trigger a manual data fetch. Used with `data` declarations that have `trigger: manual`:

```naze
data result: fetch "/api/submit" {
  method: post
  body: { name: name-input }
  trigger: manual
}

rect padding: 8px, color: #2563eb, radius: 4px {
  text "Submit" color: #ffffff
  on click: trigger result
}
```

### copy

Copy a value to the system clipboard:

```naze
on click: copy "https://example.com/share/123"
on click: copy invite-code
```

### send

Send a message on an active WebSocket stream:

```naze
data chat: stream "wss://api.example.com/chat"

on click: send chat msg
on click: send chat "Hello, world!"
```

### js

Call a JavaScript function. See the [JavaScript Interop](#javascript-interop) section for full details:

```naze
on click: js "alert"("Hello from Naze!")
on click: js "analytics.track"("button_click")
on click: js "Date.now"() -> timestamp
```

### notify

Send a browser notification:

```naze
on click: notify "Order Shipped!" {
  body: "Your order is on its way."
  icon: "icon.png"
}

on click: notify "Quick update!"
```

### emit

Emit a custom event from within a component (see [Component Events](#component-events-emit)):

```naze
on click: emit toggled
on click: emit selected
```

### set-theme

Switch the active named theme at runtime (see [Theming](#theming)):

```naze
on click: set-theme "dark"
on click: set-theme "light"
```

---

## Conditional Rendering

### if / else

Show or hide UI based on conditions:

```naze
if count > 0 {
  text "Count is {count} (positive)"
} else {
  text "Count is zero"
}
```

Chained conditions with `else if`:

```naze
if status == "loading" {
  text "Loading..."
} else if status == "error" {
  text "Error!" color: #dc2626
} else {
  text "Done"
}
```

### Inline Conditional Values

Properties can use inline `if`/`else` for conditional values without creating separate element branches:

```naze
row height: if expanded { 150px } else { 60px } {
  text "Content"
}

rect color: if active { #3b82f6 } else { #94a3b8 } {
  text "Status"
}

text "Result" color: if valid { #16a34a } else { #dc2626 }
```

### Truthiness

Naze evaluates conditions as follows:

- `false`, `0`, `""` (empty string), `null` are falsy
- Everything else is truthy
- Data lifecycle fields: `data.loading` is truthy while loading, `data.error` is truthy when an error occurred, `data.data` is truthy when data is available

```naze
if posts.loading {
  text "Loading..."
}

if posts.error {
  text "Error: {posts.error}" color: #dc2626
}

if posts.data {
  each post in posts.data {
    text "{post.title}"
  }
}
```

### Error Boundaries

Wrap data-fetching subtrees with `boundary`/`catch` for graceful error recovery. If any data source inside the boundary errors, the catch block renders instead.

```naze
boundary {
  data users: fetch "/api/users"
  data stats: fetch "/api/stats"

  column gap: 16px {
    text "Users: {users.data}"
    text "Stats: {stats.data}"
  }
} catch {
  text "Something went wrong. Please try again." color: #dc2626
}
```

**Semantics:**

- The compiler scans the boundary block for `data` declarations
- Generates a combined error condition: `!(users.error || stats.error)`
- Desugars to `__if` nodes at compile time -- no runtime or IR changes
- The boundary block must contain at least one `data` declaration (compiler error otherwise)
- The catch block renders when any data source in the boundary has an error

---

## Iteration

`each` iterates over a list, rendering its body once for each item:

```naze
state items = ["Apple", "Banana", "Cherry"]

each item in items {
  text "{item}"
}
```

### Dot Access on Items

When iterating over a list of objects, use dot notation to access fields:

```naze
each post in posts.data {
  column padding: 12px, color: #f3f4f6, radius: 8px {
    text "{post.title}" font-size: 18px
    text "{post.body}" color: #666666
  }
}
```

### Pipeline in each

Pipelines can be used inline to filter, sort, and transform the list before iteration:

```naze
each student in students | filter score > 80 | sort-by name {
  text "{student.name}: {student.score}"
}

each tag in tags | distinct {
  text "{tag}"
}

each person in items | filter score > 70 | sort-by dept {
  text "{person.name} ({person.dept}): {person.score}"
}
```

---

## Pattern Matching

`match` renders different UI based on a value. It desugars to nested if/else chains at compile time -- no runtime pattern matching overhead.

```naze
match status {
  "loading": text "Please wait..."
  "error": text "Something went wrong" color: #dc2626
  "success": text "Done!" color: #16a34a
  _: text "Unknown state"
}
```

Each arm has a pattern and either a single element or a block of children:

```naze
match theme {
  "dark": {
    rect width: 200px, height: 100px, color: #333333 {
      text "Dark Mode" color: #ffffff
    }
  }
  "light": {
    rect width: 200px, height: 100px, color: #eeeeee {
      text "Light Mode" color: #000000
    }
  }
  _: text "Unknown theme"
}
```

### Patterns

| Pattern | Syntax | Example |
|---------|--------|---------|
| String literal | `"value"` | `"active"`, `"error"` |
| Number literal | `N` | `0`, `42`, `3.14` |
| Boolean literal | `true` / `false` | `true` |
| Identifier | `name` | Variable reference |
| Wildcard | `_` | Matches anything |

### Semantics

- Arms are evaluated top to bottom; the first matching arm wins
- The compiler warns if no `_` (wildcard) arm is present (incomplete match)
- Duplicate patterns produce a compiler warning
- `match` compiles to nested `if`/`else` nodes -- no new runtime construct

---

## Data Fetching

Declare async data sources with the `data` keyword. All data sources produce three reactive lifecycle fields:

- `name.loading` -- `true` while the request is in flight
- `name.error` -- error message string (falsy if no error)
- `name.data` -- the fetched result (available after loading completes)

### Basic Fetch

```naze
data posts: fetch "https://jsonplaceholder.typicode.com/posts?_limit=5"

if posts.loading {
  text "Loading posts..."
}

if posts.error {
  text "Error: {posts.error}" color: #dc2626
}

if posts.data {
  each post in posts.data {
    text "{post.title}"
  }
}
```

By default, `fetch` requests fire automatically when the component mounts.

### Enhanced Data

The `data` keyword supports an optional block body for full HTTP configuration:

```naze
data users: fetch "/api/users" {
  method: get
  headers: { "Authorization": "Bearer {auth-token}" }
  cache: 5min
  retry: 3
}

data create-result: fetch "/api/users" {
  method: post
  body: { name: name-input, email: email-input }
  headers: { "Authorization": "Bearer {auth-token}" }
  trigger: manual
}

data upload-result: fetch "/api/upload" {
  method: post
  body: { file: avatar-file }
  content-type: multipart
  trigger: manual
}
```

**Block properties:**

| Property | Type | Description |
|----------|------|-------------|
| `method` | text | HTTP method: `get`, `post`, `put`, `delete`, `patch` (default: `get`) |
| `headers` | object | HTTP headers (supports string interpolation) |
| `body` | object | Request body for `post`/`put`/`patch` |
| `content-type` | text | `json` (default) or `multipart` (for file uploads) |
| `cache` | duration | Reuse response for identical requests within duration |
| `retry` | number | Retry count on network failure with exponential backoff |
| `trigger` | text | `manual` to suppress auto-fetch; activated by `trigger name` |

**Reactive URL interpolation:**

When the URL contains interpolated state references, the fetch re-triggers automatically when those state values change (for GET requests):

```naze
data results: fetch "/api/search?q={search-query}" {
  cache: 30s
}
```

**Triggering manual fetches:**

```naze
data result: fetch "/api/submit" {
  method: post
  body: { name: name }
  trigger: manual
}

rect padding: 8px, color: #2563eb, radius: 4px {
  text "Submit" color: #ffffff
  on click: trigger result
}
```

### Data Streams: WebSocket and SSE

`data: stream` declares a persistent connection for real-time push data.

```naze
-- WebSocket connection
data chat: stream "wss://api.example.com/chat/{room-id}"

-- Server-Sent Events
data notifications: stream "/api/events" {
  type: sse
}
```

**Semantics:**

- `.data` is a reactive list that grows as messages arrive (most recent appended)
- `.loading` is true until the connection is established
- `.error` is set on connection failure
- URL interpolation is reactive -- changing `{room-id}` closes the old connection and opens a new one
- `send` action pushes a message to a WebSocket stream: `on click: send chat "Hello"`
- Default type is WebSocket; add `type: sse` for Server-Sent Events (read-only)
- Auto-reconnect on disconnect with exponential backoff

```naze
app "Chat" {
  state msg = ""
  data chat: stream "wss://echo.websocket.org"

  column padding: 20px, gap: 16px {
    heading "WebSocket Stream"

    if chat.loading {
      text "Connecting..."
    }

    text "Messages received: {chat.data}"

    rect width: 200px, height: 40px, color: #2563eb, radius: 8px {
      text "Send Hello"
      on click: send chat msg
    }
  }
}
```

### JS Data Sources

Call JavaScript functions as async data sources with the same lifecycle pattern:

```naze
data checkout: js "createCheckoutSession"(cart-items) {
  trigger: manual
}

on click: trigger checkout

if checkout.loading {
  text "Processing..."
}
if checkout.error {
  text "Error: {checkout.error}" color: #dc2626
}
if checkout.data {
  text "Session created: {checkout.data.id}"
}
```

### Device APIs

Declarative access to browser hardware APIs using the `data` lifecycle pattern:

```naze
-- Geolocation (one-shot)
data location: device "geolocation"

-- Geolocation (continuous watch)
data location: device "geolocation" {
  watch: true
}

-- Camera
data camera: device "camera"
```

**Semantics:**

- `device` signals a browser hardware API requiring permissions
- Permission handling is implicit -- denied permissions surface as `.error`
- `.data` contains the result object (e.g., `{ latitude: 40.7, longitude: -74.0, accuracy: 10 }`)
- `watch: true` enables continuous position updates (vs one-shot)
- Supported APIs: `geolocation`, `camera`
- Not available in native mode (logs a warning)

```naze
app "Location" {
  data location: device "geolocation" {
    trigger: manual
  }

  column padding: 20px, gap: 16px {
    rect padding: 8px, radius: 4px, color: #3b82f6 {
      text "Get Location" color: #ffffff
      on click: trigger location
    }

    if location.loading {
      text "Getting location..." color: #6b7280
    }

    if location.error {
      text "Error: {location.error}" color: #ef4444
    }

    if location.data {
      text "Latitude: {location.data.latitude}" color: #10b981
      text "Longitude: {location.data.longitude}" color: #10b981
    }
  }
}
```

---

## Server Functions

Server functions define server-side logic that is compiled into SSR endpoints. They run on the server, not in the browser.

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
```

**Syntax:** `server function name(param: type, ...) { expression }`

Server functions are called from event handlers or data declarations:

```naze
on click: call server.get-users(10)
```

**Semantics:**

- Run on the server, not in the client WASM runtime
- Can access server-side resources (databases, file systems)
- Single expression body (can use pipeline syntax)
- The compiler generates API endpoints automatically

---

## Server Data

Server data declarations call server functions at SSR time. They produce the standard `.loading`/`.error`/`.data` lifecycle.

```naze
data user: get-user(42)
data posts: get-latest-posts(10)
```

**Semantics:**

- Resolved at SSR time (server-side rendering)
- Produces the same `.loading`/`.error`/`.data` lifecycle as `fetch` data
- Data is available immediately on page load (no client-side fetch)

---

## Database Models and Queries

Naze provides declarative database access through `model` definitions and type-safe query expressions that compile to parameterized SQL at build time.

### Model Definitions

Define database table schemas with `model` blocks at the top level:

```naze
model users {
  id number primary
  name text
  email text unique
  active bool
  created_at timestamp default now
}

model posts {
  id number primary
  title text
  body text
  author_id number
  published bool
}
```

**Field types:** `number`, `text`, `bool`, `timestamp`

**Field constraints:**

| Constraint | Description |
|------------|-------------|
| `primary` | Primary key |
| `unique` | Unique constraint |
| `default value` | Default value (e.g., `default now`, `default true`) |

**Semantics:**

- Models are compile-time only -- they inform query generation but are not included in the IR or runtime
- The compiler validates that query expressions reference defined models
- Models do not create database tables -- schema migration is handled externally

### Query Expressions

Query expressions are used inside server function bodies. They compile to parameterized SQL at build time -- no string interpolation in SQL, preventing injection.

#### find

Retrieve rows matching conditions:

```naze
server function get-users() {
  let users = find users where active == true order name limit 10
  users
}

server function get-user(id: number) {
  let users = find users where id == id limit 1
  users
}
```

Compiles to: `SELECT * FROM users WHERE active = $1 ORDER BY name LIMIT $2`

#### insert

Insert a new row:

```naze
server function create-user(name: text, email: text) {
  let user = insert users { name: name, email: email }
  user
}
```

Compiles to: `INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *`

#### update

Update rows matching conditions:

```naze
server function update-user(id: number, name: text) {
  let result = update users set { name: name } where id == id
  result
}
```

Compiles to: `UPDATE users SET name = $1 WHERE id = $2 RETURNING *`

#### delete

Delete rows matching conditions:

```naze
server function remove-user(id: number) {
  let result = delete users where id == id
  result
}
```

Compiles to: `DELETE FROM users WHERE id = $1 RETURNING *`

### Query Clauses

| Clause | Syntax | Description |
|--------|--------|-------------|
| `where` | `where field == value` | Filter condition (supports `==`, `!=`, `>`, `<`, `>=`, `<=`) |
| `order` | `order field` | Sort by field (ascending) |
| `limit` | `limit N` | Maximum number of rows |

Multiple `where` conditions can be combined with `and`:

```naze
find users where active == true and role == "admin" order name limit 20
```

### Prerequisites

- `env.DATABASE_URL` must be declared in `naze.toml` `[env]` section when queries are used
- PostgreSQL supported via `tokio-postgres` (feature flag: `cargo build -p nazec --features database`)
- SQLite supported via `rusqlite` (feature flag)
- Results are returned as `List(Vec<Object>)` -- each row is an object with column keys

---

## AI Prompts

`prompt` declares an AI provider call with the same data lifecycle pattern.

```naze
prompt summary: from openai {
  system: "You are a concise summarizer."
  user: "Summarize: {content}"
  model: "gpt-4o"
  max-tokens: 200
  temperature: 0.3
}
```

**Syntax:** `prompt name: from provider { props }`

**Providers:** `openai`, `anthropic`, `ollama`, or a generic URL.

**Block properties:**

| Property | Type | Description |
|----------|------|-------------|
| `system` | text | System prompt |
| `user` | text | User prompt (supports interpolation) |
| `model` | text | Model identifier |
| `max-tokens` | number | Maximum response tokens |
| `temperature` | number | Sampling temperature (0.0 to 1.0) |

**Lifecycle:**

- `summary.loading` -- true while the AI request is in flight
- `summary.error` -- error message if the request fails
- `summary.data` -- the generated response text

```naze
app "AI Summary" {
  state content = ""
  prompt summary: from openai {
    system: "You are a concise summarizer."
    user: "Summarize: {content}"
    model: "gpt-4o"
    max-tokens: 200
    temperature: 0.3
  }

  column padding: 20px, gap: 16px {
    textarea bind: content, placeholder: "Paste text to summarize..."

    if summary.loading {
      text "Generating summary..."
    }

    if summary.data {
      text "{summary.data}"
    }
  }
}
```

---

## Imports and Modules

### Local Imports

Import components from local files using `use`:

```naze
use components/pill
use components/card
use components/toggle-btn
```

`use components/pill` resolves to `components/pill.naze` relative to the project root. The filename (without `.naze`) becomes the component name.

### Registry Imports

Import components from published packages using scoped paths:

```naze
use @naze/ui-kit/button
use @acme/charts/bar-chart
```

Registry packages are declared in `naze.toml`:

```toml
[dependencies]
"@naze/ui-kit" = "^1.0"
"@acme/charts" = "^2.0"
```

### WASM/JS Module Imports

Import external WASM or JavaScript modules:

```naze
import crypto from "./lib/crypto.wasm"
import utils from "@naze/crypto"
```

Imported modules are available as namespaces in the current file.

### Package Management

The CLI provides package management commands:

```bash
nazec add @naze/ui-kit --version "^1.0"
nazec add ./local-package --path ./libs/my-lib
nazec remove @naze/ui-kit
nazec update @naze/ui-kit
nazec update                    # update all dependencies
nazec search "chart"
nazec publish --registry https://registry.naze.dev
```

---

## Theming

### Theme Definition

Define design tokens in a `theme` block or in a dedicated `theme.naze` file:

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
    muted: #94a3b8
    border: #e2e8f0
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

Themes have two sections: `colors` (color tokens) and `spacing` (dimension tokens).

### Named Themes and Inheritance

Themes can have names. Named themes can inherit from other themes using `extends`, overriding only the tokens that differ:

```naze
theme light {
  colors {
    bg: #ffffff
    fg: #0f172a
    primary: #2563eb
    card-bg: #f8fafc
  }
  spacing {
    sm: 8px
    md: 16px
    lg: 24px
  }
}

theme dark extends light {
  colors {
    bg: #1e293b
    fg: #f8fafc
    primary: #60a5fa
    card-bg: #334155
  }
}
```

**Semantics:**

- `extends` inherits all tokens from the parent theme, then overrides any redeclared tokens
- Inheritance chains are resolved via topological sort at compile time (cycles produce an error)
- The first named theme defined becomes the default active theme

### Token References

Use `theme.section.token` to reference tokens in any property value:

```naze
column padding: theme.spacing.lg, gap: theme.spacing.md {
  heading "Title" color: theme.colors.foreground
  rect width: 60px, height: 60px, color: theme.colors.primary, radius: 8px
}
```

Tokens are resolved at compile time for unnamed themes (values inlined). For named themes, tokens resolve at runtime so that `set-theme` can swap them.

### Runtime Theme Switching

Use the `set-theme` action to switch the active named theme at runtime:

```naze
theme light {
  colors {
    bg: #ffffff
    fg: #0f172a
    primary: #2563eb
    card-bg: #f8fafc
  }
  spacing {
    sm: 8px
    md: 16px
    lg: 24px
  }
}

theme dark extends light {
  colors {
    bg: #1e293b
    fg: #f8fafc
    primary: #60a5fa
    card-bg: #334155
  }
}

app "Theme Switching Demo" {
  column padding: 20px, gap: 16px, color: theme.colors.bg {
    heading "Theme Switching" color: theme.colors.fg

    row gap: 12px {
      rect width: 120px, height: 40px, color: theme.colors.primary, radius: 8px {
        text "Light" color: #ffffff
        on click: set-theme "light"
      }
      rect width: 120px, height: 40px, color: theme.colors.primary, radius: 8px {
        text "Dark" color: #ffffff
        on click: set-theme "dark"
      }
    }

    rect padding: 16px, color: theme.colors.card-bg, radius: 8px {
      text "This card uses theme tokens." color: theme.colors.fg
    }
  }
}
```

**Semantics:**

- `set-theme "name"` swaps the active theme's token values at runtime
- All `theme.section.token` references update immediately and trigger re-render
- The compiler warns on unknown theme token references

### Built-in Default Tokens

Apps have access to default tokens without defining a custom theme:

**Color tokens:** `primary`, `secondary`, `success`, `warning`, `danger`, `background`, `foreground`, `muted`, `border`

**Spacing tokens:** `xs` (4px), `sm` (8px), `md` (16px), `lg` (24px), `xl` (32px)

---

## Pages and Navigation

### Page Blocks

Define multiple pages inside an `app` block. Each page has a URL path:

```naze
app "My Site" {
  -- Persistent shell (renders on all pages)
  row padding: 16px, gap: 24px, color: #1e293b {
    heading "My App" color: #ffffff
    link "Home", to: "/"
    link "About", to: "/about"
    link "Contact", to: "/contact"
  }

  page "/" {
    column padding: 24px, gap: 16px {
      heading "Welcome Home"
      text "This is the home page."
    }
  }

  page "/about" {
    column padding: 24px, gap: 16px {
      heading "About Us"
      text "Learn more about us."
    }
  }

  page "/contact" {
    column padding: 24px, gap: 16px {
      heading "Contact"
      text "Get in touch."
    }
  }
}
```

Elements outside `page` blocks render on every page (useful for navigation bars, headers, footers).

### Navigation Methods

- **link element** -- declarative navigation: `link "About", to: "/about"`
- **navigate action** -- programmatic navigation: `on click: navigate "/about"`
- **Browser history** -- back/forward buttons work via History API integration

### Dynamic Route Parameters

Pages can declare path parameters using `:param` syntax in the route path. Parameters are automatically extracted from the URL and bound to the `params.*` namespace.

```naze
app "Blog" {
  page "/posts/:id" {
    data post: fetch "/api/posts/{params.id}"

    if post.data {
      text "{post.data.title}"
      text "{post.data.body}"
    }
  }

  page "/users/:user-id/posts/:post-id" {
    text "User: {params.user-id}, Post: {params.post-id}"
  }
}
```

**Semantics:**

- `:name` segments in the path are extracted as route parameters
- Access via `params.name` in interpolations and expressions
- The compiler validates that `params.*` references match declared route parameters
- Parameters are available in data URLs, server function calls, and element text

### Catch-All Routes

Use `/*` as the route path to match any URL that doesn't match a more specific page. Useful for 404 pages.

```naze
app "My App" {
  page "/" {
    heading "Home"
  }

  page "/about" {
    heading "About"
  }

  page "/*" {
    heading "Page Not Found"
    text "The page you're looking for doesn't exist."
  }
}
```

**Semantics:**

- Catch-all routes match any path not matched by other pages
- The compiler warns if the catch-all is not the last page definition
- The compiler warns on duplicate route patterns

### Query String Parameters

For query string parameters (e.g., `?page=2&q=search`), use `param` declarations instead of path parameters. See [Params](#params) for details.

---

## Accessibility

### Props

Any element can have accessibility properties:

| Prop | Description |
|------|-------------|
| `role` | ARIA role: `button`, `link`, `navigation`, `main`, `heading`, `list`, `listitem`, etc. |
| `label` | Accessible name (equivalent to `aria-label`) |
| `tab-index` | Keyboard navigation order (0 = natural order, -1 = not focusable) |
| `id` | Element identification (used by `scroll-to` and accessibility relationships) |

```naze
rect width: 200px, height: 50px, color: #2563eb, radius: 8px, role: "button", label: "Increment counter" {
  text "Increment" color: #ffffff
  on click: set count = count + 1
}
```

### Keyboard Navigation

- **Tab / Shift+Tab** -- cycles through focusable elements (interactive elements, inputs, links, elements with `tab-index`)
- **Enter** -- activates the focused element (triggers `on click`)
- **Escape** -- clears focus
- **Arrow keys** -- handled by `on arrow-up/down/left/right` events
- A **focus ring** renders around the currently focused element

### Screen Reader Support

A hidden DOM overlay mirrors canvas content with ARIA attributes. This overlay is invisible to sighted users but accessible to screen readers. ARIA roles are automatically inferred from element kind:

| Element | Inferred role |
|---------|---------------|
| `heading` | `heading` |
| `link` | `link` |
| `input` | `textbox` |
| `checkbox` | `checkbox` |
| `radio` | `radio` |
| `select` | `listbox` |

### Compiler Warnings

The compiler produces accessibility warnings when:

- Interactive elements (elements with `on click`) lack `role` or `label`
- Form inputs lack accessible labels
- Images lack `alt` text

---

## Drag and Drop

### Draggable Elements

Mark elements as draggable with the `draggable` prop and attach data with `drag-data`:

```naze
rect draggable: true, drag-data: "Red", width: 80px, height: 80px, color: #ef4444, radius: 8px {
  text "Drag me" color: #ffffff
  on drag-start: set drag-active = true
}
```

**Props:**

| Prop | Type | Description |
|------|------|-------------|
| `draggable` | bool | Enable drag on this element |
| `drag-data` | expression | Data attached to the drag operation |

### Drop Targets

Mark elements as drop targets with the `drop-target` prop:

```naze
rect drop-target: true, width: 300px, height: 120px, color: #f1f5f9, radius: 8px, border: 2px, border-color: #cbd5e1 {
  on drag-over: set status = "Release to drop..."
  on drop: set status = "Dropped!"
  text "{status}" color: #64748b
}
```

### Events

| Event | Triggers when |
|-------|--------------|
| `on drag-start` | User begins dragging a `draggable` element |
| `on drag-over` | A dragged item hovers over a `drop-target` element |
| `on drop` | A dragged item is released over a `drop-target` element |

```naze
app "Drag and Drop Demo" {
  state dropped-item = ""
  state drag-active = false

  column padding: 20px, gap: 16px {
    heading "Drag and Drop"

    row gap: 12px {
      rect draggable: true, drag-data: "Red", width: 80px, height: 80px, color: #ef4444, radius: 8px {
        on drag-start: set drag-active = true
        text "Red" color: #ffffff
      }
      rect draggable: true, drag-data: "Green", width: 80px, height: 80px, color: #22c55e, radius: 8px {
        on drag-start: set drag-active = true
        text "Green" color: #ffffff
      }
      rect draggable: true, drag-data: "Blue", width: 80px, height: 80px, color: #3b82f6, radius: 8px {
        on drag-start: set drag-active = true
        text "Blue" color: #ffffff
      }
    }

    rect drop-target: true, width: 300px, height: 120px, color: #f1f5f9, radius: 8px, border: 2px, border-color: #cbd5e1 {
      on drop: set dropped-item = "Item dropped!"
      on drag-over: set dropped-item = "Release to drop..."
      text "{dropped-item}" color: #16a34a
    }
  }
}
```

---

## Animation

Naze supports three forms of animation: transitions, spring physics, and keyframe sequences. All animations are declared as string-valued props on elements.

### Transitions

Animate property changes with the `transition` prop. When a property value changes (due to a state change), the runtime smoothly interpolates between old and new values.

```naze
state expanded = false

row width: 200px, height: if expanded { 150px } else { 60px }, color: #3b82f6, radius: 8px, transition: "height 300ms ease-out" {
  text "Click to toggle" color: #ffffff
  on click: set expanded = !expanded
}
```

**Format:** `transition: "property duration easing"`

| Component | Values |
|-----------|--------|
| **Property** | Any numeric or color property (`height`, `width`, `opacity`, `color`, etc.) |
| **Duration** | Milliseconds (e.g., `300ms`, `500ms`) |
| **Easing** | `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out` |

**Examples:**

```naze
transition: "height 300ms ease-out"
transition: "color 200ms ease"
transition: "opacity 300ms ease"
transition: "width 500ms ease-in-out"
```

Color transitions interpolate RGB components smoothly:

```naze
rect width: 200px, height: 80px, color: if active { #3b82f6 } else { #ef4444 }, radius: 8px, transition: "color 200ms ease" {
  text "Click to change color" color: #ffffff
  on click: set active = !active
}
```

### Custom Cubic-Bezier Easing

Define custom easing curves with `cubic-bezier(x1, y1, x2, y2)` for precise control over animation timing. Overshoot values (y > 1) create bounce effects.

```naze
rect width: 300px, height: 60px, color: #10b981, radius: 8px, transition: "width 500ms cubic-bezier(0.34, 1.56, 0.64, 1)" {
  text "Overshoot easing" color: #ffffff
}
```

**Format:** `transition: "property duration cubic-bezier(x1, y1, x2, y2)"`

The four parameters define the control points of a cubic Bezier curve. Standard easing names are shortcuts for common curves.

### Spring Physics

Spring physics creates natural, physically-based animations with bounce and settle behavior. Springs run until the animation settles (position near target, velocity near zero) rather than for a fixed duration.

```naze
rect width: 200px, height: 80px, color: #3b82f6, radius: 12px, transition: "color spring(180, 12)" {
  text "Spring!" color: #ffffff
  on click: set active = !active
}
```

**Format:** `transition: "property spring(stiffness, damping)"`

| Parameter | Range | Description |
|-----------|-------|-------------|
| **Stiffness** | 100-400 typical | Higher values = faster spring, more snappy |
| **Damping** | 10-30 typical | Higher values = less bounce, more controlled |

Springs have a 5-second hard timeout to prevent infinite oscillation.

### Keyframe Animations

The `animate` prop plays multi-step keyframe sequences when an element appears or its animation values change. Unlike transitions (which animate between two values), keyframes define an explicit sequence of values.

```naze
rect width: 80px, height: 80px, color: #8b5cf6, radius: 8px, animate: "scale [1, 1.3, 0.9, 1.1, 1] 600ms ease-in-out" {
  text "!" color: #ffffff, font-size: 24px
}
```

**Format:** `animate: "property [value1, value2, ...] duration easing"`

| Component | Description |
|-----------|-------------|
| **Property** | `scale`, `rotate`, `opacity`, and numeric properties |
| **Values** | Comma-separated list in brackets -- the animation interpolates through each value in sequence |
| **Duration** | Total animation time in milliseconds |
| **Easing** | Same easing functions as transitions |

Multiple keyframe animations can be comma-separated:

```naze
animate: "scale [1, 1.2, 1] 400ms ease, opacity [0, 1] 200ms ease-in"
```

### Layout-Skip Fast Path

Animations targeting only visual properties skip layout recomputation and reuse the cached layout tree. These fast-path properties are:

- `transform`
- `opacity`
- `color`
- `shadow`
- `scale`
- `rotate`

Animations on these properties are significantly cheaper than animations on layout-affecting properties (`width`, `height`, `padding`, `gap`).

---

## JavaScript Interop

Naze provides a controlled escape hatch for calling third-party JavaScript SDKs (Stripe, Mapbox, analytics, auth providers) from Naze code. Functions must be on `globalThis` -- no module imports, keeping it simple and auditable.

### Script Inclusion

Declare external JavaScript files in `naze.toml`:

```toml
[scripts]
stripe = "https://js.stripe.com/v3/"
mapbox = "https://api.mapbox.com/mapbox-gl-js/v3/mapbox-gl.js"
analytics = "./js/analytics.js"
```

The compiler embeds `<script>` tags in the generated `index.html`, loaded before the Naze runtime.

### Sync Calls

Call JavaScript functions as event actions:

```naze
-- Fire and forget
on click: js "alert"("Hello from Naze!")
on click: js "analytics.track"("button_clicked", { page: "home" })
on click: js "gtag"("event", "button_click")

-- Store return value in state
on click: js "Date.now"() -> timestamp
on click: js "Stripe"(stripe-key) -> stripe-instance
```

### Async Calls

Use the `data` keyword with `js` source for async JavaScript calls with the standard loading/error/data lifecycle:

```naze
data checkout: js "createCheckoutSession"(cart-items) {
  trigger: manual
}

on click: trigger checkout

if checkout.loading {
  text "Processing..."
}
if checkout.error {
  text "Error: {checkout.error}" color: #dc2626
}
if checkout.data {
  text "Session: {checkout.data.id}"
}
```

### Type Marshalling

| Naze type | JavaScript type |
|-----------|-----------------|
| `number` | `Number` (f64) |
| `text` | `String` |
| `bool` | `Boolean` |
| `list` | `Array` |
| `object` | `Object` |

### Limitations

- Functions must be on `globalThis` -- no ES module imports, no `require()`
- Scripts are opt-in via `naze.toml` `[scripts]` section
- Not available in native mode (logs a warning at runtime)

---

## Device APIs

Declarative access to browser hardware APIs using the `data` lifecycle pattern.

### Geolocation

```naze
-- One-shot location
data location: device "geolocation"

-- Continuous location updates
data location: device "geolocation" {
  watch: true
}
```

Result fields: `location.data.latitude`, `location.data.longitude`, `location.data.accuracy`

### Camera

```naze
data camera: device "camera"
```

### Notifications

Browser notifications are sent via the `notify` action (not a data source):

```naze
-- With body and icon
on click: notify "Order Shipped!" {
  body: "Your order is on its way."
  icon: "icon.png"
}

-- Simple notification
on click: notify "Quick update!"
```

**Semantics:**

- Permission handling is implicit -- denied permissions surface as `.error`
- `notify` requests notification permission on first use, then shows the notification
- `watch: true` enables continuous updates (vs one-shot)
- Not available in native mode (logs a warning)
- Supported APIs: `geolocation`, `camera`

---

## Testing

Naze has a first-class testing framework using the same language syntax. Tests live in `.test.naze` files and use a declarative step-based approach.

### Test Files

Test files use the `.test.naze` extension and import the component or app under test with `use`:

```naze
-- tests/counter.test.naze

use counter

test "counter starts at zero" {
  render counter
  assert text "Current count: 0" is visible
}

test "counter increments on click" {
  render counter
  assert text "Current count: 0" is visible
  click "Increment"
  assert text "Current count: 1" is visible
}

test "counter resets to zero" {
  render counter
  click "Increment"
  click "Increment"
  assert text "Current count: 2" is visible
  click "Reset"
  assert text "Current count: 0" is visible
}
```

### Test Steps

| Step | Syntax | Description |
|------|--------|-------------|
| `render` | `render component-name prop: value` | Render a component with optional props |
| `click` | `click "Button Text"` | Simulate a click on an element with matching text |
| `fill` | `fill "placeholder" with "value"` | Type a value into an input matching the placeholder |
| `navigate` | `navigate "/path"` | Navigate to a route |
| `wait` | `wait 300ms` | Wait for a duration before the next step |

### Assertions

| Assertion | Syntax | Description |
|-----------|--------|-------------|
| Text visible | `assert text "..." is visible` | Assert that text content is visible on screen |
| Text not visible | `assert text "..." is not visible` | Assert that text is not visible |
| Page route | `assert page is "/path"` | Assert the current page route |
| State value | `assert state name is value` | Assert a state variable's current value |
| Event emitted | `assert emitted event-name` | Assert that a component emitted an event |
| Accessibility | `assert no accessibility violations` | Assert no accessibility warnings |

### Flow Tests

`flow` blocks test multi-page user journeys with longer step sequences:

```naze
flow "login and navigate to dashboard" {
  navigate "/"
  assert text "Login" is visible
  fill "email" with "user@example.com"
  fill "password" with "secret"
  click "Sign In"
  assert page is "/dashboard"
  assert text "Welcome" is visible
}
```

### Running Tests

```bash
nazec test                        # Discover and run all .test.naze files
nazec test --filter "counter"     # Run tests matching the pattern
```

Test files use a separate grammar entry point (`test_file`) with `test` blocks, `flow` blocks, and `assert` statements.

---

## Project Structure

A Naze project has this directory structure:

```
my-project/
  naze.toml           # Project manifest
  app.naze            # Entry file
  theme.naze          # Optional theme tokens
  components/         # Component files
    pill.naze
    card.naze
    toggle-btn.naze
  tests/              # Test files
    counter.test.naze
    input.test.naze
  dist/               # Build output (generated)
    index.html
    naze_runtime.js
    naze_runtime_bg.wasm
    app_data.bin
```

### naze.toml

The project manifest configures the build and declares dependencies:

```toml
[app]
name = "my-project"
version = "0.1.0"

[build]
entry = "app.naze"
output = "dist/"

# Optional: JavaScript scripts for JS interop
[scripts]
stripe = "https://js.stripe.com/v3/"
analytics = "./js/analytics.js"

# Optional: package dependencies
[dependencies]
"@naze/ui-kit" = "^1.0"
"@acme/charts" = "^2.0"
```

**Sections:**

| Section | Description |
|---------|-------------|
| `[app]` | Project name and version |
| `[build]` | Entry file and output directory |
| `[env]` | Environment variables with defaults |
| `[scripts]` | JavaScript files to include (for `js` interop) |
| `[dependencies]` | Package registry dependencies |

### Build Output

`nazec build` produces the `dist/` directory:

| File | Description |
|------|-------------|
| `index.html` | Generated HTML shell |
| `naze_runtime.js` | JavaScript wrapper for WASM runtime |
| `naze_runtime_bg.wasm` | WASM binary (runtime + layout + renderer) |
| `app_data.bin` | Compiled render tree (custom binary IR) |

The WASM runtime and JS wrapper are embedded in the `nazec` binary via `include_bytes!()`, so the CLI is a self-contained single binary.

### Environment Variables

The `[env]` section in `naze.toml` declares environment variables with defaults and optional requirements:

```toml
[env]
API_URL = "https://api.example.com"
STRIPE_KEY = { from = "STRIPE_PUBLIC_KEY", required = true }
DEBUG = "false"
```

**Referencing in code:**

Use `env.NAME` syntax to reference environment variables. In client code, values are substituted at compile time (inlined as string literals). In server functions, values are resolved at runtime via `std::env::var()`.

```naze
data users: fetch "{env.API_URL}/users"

server function get-data() {
  fetch "{env.API_URL}/internal/data" {
    headers: { "X-Api-Key": "{env.API_SECRET}" }
  }
}
```

**Loading:**

- `nazec dev` loads variables from a `.env` file in the project root (simple KEY=VALUE format)
- `nazec build` validates that all `required` variables are present in the environment
- Variables without `from` use the key name as the environment variable name
- Variables with `from` map to a differently-named environment variable

**Type checking:**

- The compiler validates that all `env.*` references exist in the `[env]` table
- Missing required variables at build time produce a compile error

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `nazec new <name>` | Create a new project with `naze.toml` and `app.naze` |
| `nazec build [--target web\|native\|android] [--static]` | Compile `.naze` files to the target platform |
| `nazec check` | Type-check without building (fast validation) |
| `nazec dev [--port N] [--open]` | Dev server with hot reload and inspector (Ctrl+Shift+I) |
| `nazec run` | Native desktop preview (winit + tiny-skia, hot reload) |
| `nazec test [--filter pattern]` | Discover and run `.test.naze` files |
| `nazec serve [--port N] [--host addr]` | Production SSR server |
| `nazec parse <file>` | Dump AST as JSON |
| `nazec gallery [--build] [--native]` | Build the interactive example gallery |
| `nazec grammar [--grammar-format gbnf\|ebnf] [--no-test]` | Export grammar in GBNF or EBNF format |
| `nazec analyze --bin path [--wasm path] [--compare path]` | Binary size analysis |
| `nazec playground [--port N]` | Interactive browser playground |
| `nazec add <package> [--version\|--path\|--git\|--tag\|--branch\|--rev]` | Add a dependency to `naze.toml` |
| `nazec remove <package>` | Remove a dependency |
| `nazec update [package]` | Update dependencies (all or specific) |
| `nazec publish [--registry url]` | Publish package to registry |
| `nazec search <query> [--limit N]` | Search the package registry |
| `nazec ai generate <prompt> [--provider\|--model\|--retries\|--output]` | Generate Naze code from a natural language prompt |
| `nazec ai fix <file> [--provider\|--model\|--retries]` | AI-assisted error fixing |
| `nazec ai dataset export [--dir\|--provider\|--output]` | Export training data for fine-tuning |
| `nazec ai dataset validate <file>` | Validate a training dataset |

### nazec new

Creates a new project directory with boilerplate:

```bash
nazec new my-app
```

Generates:

```
my-app/
  naze.toml
  app.naze
```

### nazec build

Compiles the project to the target platform:

```bash
nazec build                    # default: web target
nazec build --target web       # WASM + Canvas2D output
nazec build --target native    # native desktop binary
nazec build --target android   # Android target
nazec build --static           # static site generation
```

### nazec dev

Starts a development server with hot reload. File changes trigger recompilation and browser refresh:

```bash
nazec dev                      # default port 3000
nazec dev --port 8080          # custom port
nazec dev --open               # open browser automatically
```

The dev server includes a visual inspector activated with Ctrl+Shift+I.

### nazec run

Launches the app in a native desktop window using winit + tiny-skia. Supports hot reload (file changes update the window):

```bash
nazec run
```

### nazec check

Type-checks all `.naze` files without producing build output. Useful for CI/CD:

```bash
nazec check
```

---

## Grammar Summary

The Naze grammar is a PEG (Parsing Expression Grammar) with approximately 157 rules, implemented using pest. The grammar is designed to be small enough for constrained LLM decoding while expressive enough for complete UI applications.

### File Structure Rules

- `file` -- sequence of statements separated by newlines
- `statement` -- dispatches to all top-level construct types
- `block` -- `{` statements `}` (used by app, page, component, element, etc.)

### Statement Types

The grammar supports 30+ statement kinds:

- `comment` -- line comments starting with `--`
- `import_stmt` -- WASM/JS module imports
- `use_stmt` -- local or registry component imports
- `app_block` -- application entry point
- `page_block` -- route page definition
- `component_def` -- component with typed parameters
- `template_def` -- layout scaffold with named slots
- `theme_def` -- design token definitions with optional inheritance
- `function_def` -- pure function definitions
- `server_function_def` -- server-side function definitions
- `model_def` -- database model definitions
- `guard_def` -- route guard definitions
- `boundary_stmt` -- error boundary with catch block
- `let_stmt` -- immutable bindings
- `state_stmt` -- mutable reactive state
- `shared_state_stmt` -- cross-page shared state
- `computed_stmt` -- derived reactive state (pipeline-enabled)
- `storage_stmt` -- persistent browser storage
- `data_stmt` -- async data sources (fetch, stream, js, device)
- `server_data_stmt` -- server-resolved data
- `prompt_stmt` -- AI provider declarations
- `timer_stmt` -- scheduled actions
- `param_stmt` -- URL query string bindings
- `if_stmt` -- conditional rendering
- `each_stmt` -- list iteration
- `match_stmt` -- pattern matching
- `on_handler` -- event handlers with actions
- `slot_stmt` -- content slot definitions
- `fill_stmt` -- content slot fills
- `link_element` -- navigation links
- `element` -- generic UI elements

### Expression Rules

- `expression` -- atoms connected by binary operators
- `expr_atom` -- number, bool, string, grouped expression, index access, function call, reference, identifier
- `index_access` -- `list[expression]` (list element access by index)
- `bin_op` -- arithmetic (`+`, `-`, `*`, `/`), comparison (`==`, `!=`, `>`, `<`, `>=`, `<=`), logical (`&&`, `||`)
- `pipe_expression` -- expression optionally followed by pipeline stages
- `pipe_stage` -- pipeline function with optional arguments
- `pipe_fn` -- `filter`, `map`, `sort-by`, `take`, `sum`, `count`, `reduce`, `group-by`, `flatten`, `distinct`
- `function_call` -- name followed by parenthesized arguments

### Value Types

- `string_lit` -- double-quoted with interpolation support (`{name}`, `{obj.field}`)
- `number_lit` -- digits with optional decimal and unit (`px`, `%`, `em`)
- `color_lit` -- `#` followed by 3-8 hex digits
- `bool_lit` -- `true` or `false`
- `list_lit` -- `[value, value, ...]`
- `object_lit` -- `{ key: value, key: value }`
- `ref_path` -- dot-separated identifiers
- `duration_lit` -- number with duration unit (`ms`, `s`, `min`, `h`)

### Element Syntax

- `element` -- name, optional string, optional props, optional block
- `inline_props` -- comma-separated key-value pairs
- `prop` -- identifier `:` value

### Event Handlers and Actions

- `on_handler` -- `on` event-name modifier? `:` action
- `event_name` -- `click`, `hover`, `change`, `keypress`, `scroll`, `drag-start`, `drag-over`, `drop`, `click-outside`, `context-menu`, `pointer-move`, `arrow-up/down/left/right`, or custom identifier
- `event_modifier` -- `debounce`/`throttle` followed by duration
- `action` -- `set`, `set-index`, `append`, `remove`, `navigate`, `scroll-to`, `log`, `trigger`, `copy`, `send`, `js`, `notify`, `emit`, `set-theme`, `start`, `stop`
- `conditional_action` -- `if` expression `{` actions `}` (`else` `{` actions `}`)?
- `action_list` -- comma-separated actions (multi-action handlers)

### Control Flow

- `if_stmt` -- `if` expression block (`else` (if_stmt | block))?
- `each_stmt` -- `each` name `in` pipe_expression block
- `match_stmt` -- `match` expression `{` match_arm+ `}`
- `match_arm` -- pattern `:` (element | block)
- `match_pattern` -- wildcard `_`, string, number, bool, or identifier

### Declarations

- `state_stmt` -- `state` name `=` value
- `shared_state_stmt` -- `shared` `state` name `=` value
- `computed_stmt` -- `computed` name `=` pipe_expression
- `storage_stmt` -- `storage` name `:` (local|session) string `default:` value
- `timer_stmt` -- `timer` name `:` (after|every) duration `{` action `}`
- `param_stmt` -- `param` name `:` type `default:` value
- `data_stmt` -- `data` name `:` source string block?
- `prompt_stmt` -- `prompt` name `:` `from` provider block

### Test File Grammar

- `test_file` -- separate entry point for `.test.naze` files
- `test_block` -- `test` string `{` test_step* `}`
- `flow_block` -- `flow` string `{` test_step* `}`
- `test_step` -- `render`, `click`, `fill`, `navigate`, `wait`
- `test_assert` -- `assert` followed by assertion kind
- `assert_kind` -- text visible/not visible, page is, state is, emitted, no accessibility violations

### Whitespace and Newlines

Spaces and tabs are ignored between tokens. Newlines are significant as statement terminators for elements and declarations without blocks. Inside blocks (`{ ... }`), newlines separate statements.

### Identifiers

Identifiers start with a letter or underscore, followed by alphanumeric characters, underscores, or hyphens. Hyphens in identifiers are idiomatic in Naze (e.g., `my-component`, `drag-start`, `search-query`).

---

## Visual Properties

Additional styling properties for production visual fidelity.

### cursor

Sets the mouse cursor style when hovering over an element. Available on all elements.

```naze
rect cursor: "pointer" {
  text "Click me"
}
text "Not allowed" cursor: "not-allowed"
```

**Values:** `pointer`, `grab`, `grabbing`, `text`, `not-allowed`, `crosshair`, `move`, `resize`, `default`

### text-decoration

Adds visual decoration to text elements.

```naze
text "Underlined" text-decoration: "underline"
text "Struck through" text-decoration: "line-through"
text "Overline" text-decoration: "overline"
```

**Values:** `underline`, `line-through`, `overline`, `none`

Available on: `text`, `heading`, `link`

### shadow

Adds a box shadow to container and shape elements.

```naze
container shadow: "lg", padding: 24px, radius: 12px, color: #ffffff {
  text "Card with shadow"
}

rect width: 100px, height: 100px, color: #fff, shadow: "0 4px 6px rgba(0,0,0,0.1)"
```

**Named presets:**

| Preset | Description |
|--------|-------------|
| `sm` | Small, subtle shadow |
| `md` | Medium shadow |
| `lg` | Large shadow |
| `xl` | Extra-large shadow |

**Custom format:** `"offsetX offsetY blur color"` (e.g., `"0 4px 6px rgba(0,0,0,0.1)"`)

Available on: `row`, `column`, `stack`, `grid`, `rect`, `container`, `overlay`

### text-align

Aligns text horizontally within its container.

```naze
text "Centered" text-align: "center"
text "Right aligned" text-align: "right"
```

**Values:** `start` (default), `center`, `end`, `right`

Available on: `text`, `heading`, `link`

### line-height

Sets line height as a multiplier of font size.

```naze
text "Spacious text" line-height: 2.0
text "Tight text" line-height: 1.0
```

**Value:** number (multiplier of font-size, e.g., `1.5` = 1.5 times the font size)

Available on: `text`, `heading`, `link`

### letter-spacing

Adjusts spacing between characters.

```naze
text "S P A C E D" letter-spacing: 3px
heading "TITLE" letter-spacing: 2px
```

**Value:** number with `px` unit

Available on: `text`, `heading`

### text-overflow

Controls how text is displayed when it overflows its container.

```naze
text "This very long text will be truncated..." text-overflow: "ellipsis"
```

**Values:** `clip` (default), `ellipsis` (truncates with "...")

Available on: `text`, `heading`

### overflow

Controls clipping of child content that exceeds container bounds.

```naze
column width: 200px, height: 100px, overflow: "hidden" {
  text "This content will be clipped at the container boundary"
}
```

**Values:** `visible` (default), `hidden`, `clip`

Available on: `row`, `column`, `stack`, `grid`, `container`

### gradient

Fills an element with a gradient instead of a solid color. Takes priority over the `color` prop.

```naze
rect width: 300px, height: 100px, gradient: "linear(to-right, #3b82f6, #8b5cf6)", radius: 8px
rect width: 150px, height: 150px, gradient: "radial(#ffffff, #10b981)", radius: 75px
```

**Linear gradient format:** `"linear(direction, color1, color2, ...)"`

Directions: `to-right`, `to-left`, `to-bottom`, `to-top`, `to-bottom-right`, `to-top-right`

**Radial gradient format:** `"radial(center-color, edge-color, ...)"`

Available on: `row`, `column`, `stack`, `grid`, `rect`, `container`

### transform

Applies 2D transformations to elements.

```naze
rect width: 80px, height: 80px, color: #3b82f6, transform: "rotate(45deg)"
rect width: 60px, height: 60px, color: #ef4444, transform: "scale(1.3)"
rect width: 60px, height: 60px, color: #10b981, transform: "translate(10px, -5px)"
```

**Values:**

| Transform | Syntax | Description |
|-----------|--------|-------------|
| Rotate | `"rotate(Ndeg)"` | Rotate by N degrees |
| Scale | `"scale(N)"` or `"scale(X, Y)"` | Uniform or non-uniform scale |
| Translate | `"translate(Xpx, Ypx)"` | Move by X and Y pixels |

Available on: `row`, `column`, `stack`, `grid`, `rect`, `text`, `heading`, `container`, `image`

---

## Design Principles

### No DOM

Naze renders directly to Canvas2D (via WASM in the browser) or to a pixel buffer (via tiny-skia on the desktop). There is no DOM, no virtual DOM, and no CSS. The entire rendering pipeline is under Naze's control, enabling consistent cross-platform behavior and smaller binary size.

### AI-Native

The language is designed from the ground up for AI code generation:

- **One canonical form per concept** -- there is only one way to express each idea, eliminating ambiguity for LLM generation
- **Small grammar** -- approximately 157 PEG rules, suitable for constrained LLM decoding via GBNF export
- **Single-file components** -- all information the AI needs is in the current file (sigma = 1), no cross-file context required
- **Low token cost** -- concise, declarative syntax minimizes the number of tokens per unit of intent

### Compile-Time Expansion

Components, templates, functions, and match expressions are all inlined at compile time. The runtime is a thin interpreter that reads a data blob -- it has no concept of components, templates, or functions. This produces smaller binaries and simpler runtime behavior.

### Custom Binary IR

The compiled output is a custom binary format (not JSON, not serde). This saves approximately 40KB in the WASM binary compared to using serde. The IR types (`RenderTree`, `RenderNode`, `RenderValue`) have hand-written serialization and deserialization.

### Single-File Components

Every component is self-contained in a single `.naze` file. All type information, state declarations, and visual structure are co-located. This keeps the AI scatter factor (sigma) at 1: an AI generating or modifying a component needs only the current file, not a graph of imports and external type definitions.

### Declarative Over Imperative

Naze favors declarations over imperative code:

- `state` instead of variable assignment
- `computed` instead of manual memoization
- `data: fetch` instead of imperative HTTP calls
- `each` instead of for-loops
- `match` instead of switch/case
- Pipeline operators instead of method chains
- Event handlers instead of callbacks

This declarative style maps naturally to LLM generation patterns and produces more predictable, analyzable code.
