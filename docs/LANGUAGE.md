# Naze Language Reference (Phase 1)

This documents what the Naze language supports today. For the long-term vision, see [PROTOTYPE.md](PROTOTYPE.md).

## File Structure

A `.naze` file contains any combination of:

- Comments
- `use` imports
- `app` blocks (entry files)
- `component` definitions (component files)
- Elements

```naze
-- This is a comment

use components/pill

app "My App" {
  column padding: 20px, gap: 16px {
    heading "Hello"
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
- `%` — percentage
- `em` — relative to font size

Numbers without a unit are treated as raw values (pixels for layout props, points for font-size).

### References

Inside component bodies, parameter names can be used as values:

```naze
component box(color: color, size: number = 80px) {
  rect width: size, height: size, color: color
}
```

Multi-segment references use dot notation: `theme.primary`.

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

**Props:** `padding`, `gap`, `width`, `height`, `color`, `columns`, `align`, `justify`

#### `column`

Lays children out **vertically** (top to bottom). Children are stacked. Spacers in a column expand vertically to fill remaining height.

```naze
column gap: 16px, padding: 20px {
  heading "Title"
  text "Body text"
}
```

**Props:** `padding`, `gap`, `width`, `height`, `color`, `columns`, `align`, `justify`

#### `stack`

Layers children on top of each other at the same position. All children share the same origin point. Useful for overlays and backgrounds.

**Props:** `padding`, `gap`, `width`, `height`, `color`, `columns`, `align`, `justify`

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

**Props:** `padding`, `gap`, `width`, `height`, `color`, `columns`, `align`, `justify`

#### `container`

A styled box that lays children out vertically (like `column`). Supports background color and border radius.

```naze
container padding: 16px, color: #eff6ff, radius: 8px {
  text "Inside a card"
}
```

**Props:** `padding`, `width`, `height`, `radius`, `color`

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

### Drawing Elements

#### `rect`

Draws a colored rectangle. Requires explicit `width` and `height`.

```naze
rect width: 80px, height: 80px, color: #2563eb, radius: 8px
```

**Props:** `width`, `height`, `color`, `radius`

#### `text`

Renders body text. The string content is passed inline after the element name. Default font size is 16px.

```naze
text "Hello, world!"
text "Colored text" color: #666666, font-size: 14px
```

**Props:** `color`, `font-size`

#### `heading`

Renders heading text. Default font size is 24px, rendered bold.

```naze
heading "Page Title"
heading "Small heading" font-size: 18px, color: #1e293b
```

**Props:** `color`, `font-size`

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

## Layout Semantics

Layout is computed top-down from the viewport dimensions. Each element receives available width and height from its parent.

- **Explicit dimensions** (`width`, `height`) are used as-is
- **Implicit dimensions** are computed from children:
  - `row`: width = sum of children widths + gaps, height = tallest child
  - `column`/`container`: width = widest child, height = sum of children heights + gaps
  - `grid`: width = available width, height = sum of row heights + gaps
  - `text`/`heading`: measured from text content and font size
  - `rect`: 0x0 if no dimensions specified
- **Padding** insets the content area on all sides
- **Gap** adds space between children (not before first or after last)
- **Spacer** expands to fill remaining space in its parent's layout direction

## Project Structure

A Naze project has this structure:

```
my-project/
  naze.toml           # Project manifest
  app.naze            # Entry file
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

For reference, the complete Phase 1 grammar in PEG notation:

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

Whitespace (spaces, tabs) is ignored between tokens. Newlines are significant as statement terminators for elements without blocks.

## Debug Logging

The `log` action outputs values to the browser console (or stderr in native mode). Useful for debugging state and event handlers.

```naze
on click: log "button clicked"
on click: log count              -- log state variable value
on click: log "count is: {count}" -- interpolated string
```

## Future Ideas

This section documents potential future language features under consideration.

### JavaScript Interop

**Status:** Under consideration, not yet implemented.

The ability to call JavaScript functions from .naze code would enable:
- Integration with existing JS libraries (validation, parsing, utilities)
- Accessing browser APIs not directly exposed through naze
- Custom application logic alongside naze UI

#### Proposed Syntax
```naze
on click: js "functionName"(arg1, arg2)
on click: js "compute"(x, y) -> result   -- store return value
```

#### Technical Considerations
- Type marshalling: Num ↔ f64, Str ↔ string, Bool ↔ boolean
- JS functions must be exposed on `globalThis`
- Initial sync-only; async support could be added later
