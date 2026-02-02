# AI/Machine-Optimized Web — "Web 4.0"

> **Note:** This project was originally brainstormed under the working name "WUI" (Web UI) and renamed to **Naze** (`.naze` files, `nazec` CLI) — see discussion entry 26 for the rationale. Naze is both the AI-native language described here and the AI assistant in the [Illuminaze](https://illuminaze.com) productivity app. The language is a product under the Illuminaze umbrella.

A brainstorm on replacing the HTML/CSS/JS paradigm with an AI-native, binary-first web platform.

Open Questions:

1. How to deal with arg passing / query parameters? Routing handles path params (`/detail/{item.id}`) but what about `?key=value` query strings — are they exposed as a data binding, a route parameter, or something else? *(Partially covered in routing examples but query strings not explicitly addressed.)*
2. How to deal with things like Prisma (non-sql ORM), and integrate into the framework as a first-class concept? The data sources layer handles raw connections (REST, postgres, GraphQL) but doesn't address ORM-like schema modeling or migrations. *(Partially covered in Data Sources section — raw connections yes, ORM abstraction no.)*
3. Does Figma use pixel generation for their UI building? If so, how could that integrate or be similar for .naze files — a WYSIWYG visual editor that generates .naze source? *(Not yet covered. An interesting Phase 3+ tool — a visual layout editor that emits `.naze` source, similar to how Figma uses canvas rendering internally.)*
4. What about fonts? Can we reuse existing font formats (OTF/TTF/WOFF2)? *(Partially covered — C4a Text Engine in PROTOTYPE.md uses HarfBuzz + FreeType which natively handle OTF/TTF. WOFF2 decompresses to OTF. Standard web fonts work. Should be explicitly noted in the doc.)*
5. Hybrid integration with legacy tools — say a user has a .naze app but wants AWS Cognito (or another legacy auth provider). How do we allow that? *(Partially covered — Tier 2 WASM imports and Tier 3 server functions are the escape hatches. A Cognito integration would be a server function calling the Cognito SDK. Could document an explicit "legacy integration" pattern.)*
6. Given this is a very pixel-based framework, much like game engines, what are other possibilities it could be used for? Video? *(Partially covered in "Beyond the Web" section for cross-platform. Video rendering, data visualization, game UIs, kiosk interfaces, and digital signage are all natural extensions worth exploring.)*


---

## The Problem

The current web stack is built on technology from the 1990s. HTML, CSS, and JavaScript have been patched, extended, and layered upon for decades, but the fundamental model remains: ship human-readable text to a browser, and let the browser figure out how to render it.

This made sense when humans hand-wrote web pages. It no longer does.

**The toolchain is absurd.** A modern web app goes through something like this:

1. Developer writes TypeScript (because JavaScript isn't safe enough)
2. TypeScript gets transpiled to JavaScript
3. JavaScript gets bundled by Webpack/Vite/esbuild
4. CSS gets processed through PostCSS/Sass/Tailwind
5. Everything gets minified, tree-shaken, code-split
6. The browser downloads all of it (often 1-5MB of JavaScript alone)
7. The browser parses it back into an AST
8. The browser builds a DOM, resolves CSS cascade/specificity, computes layout
9. Finally, pixels appear on screen

Most of this pipeline exists to manage complexity that humans created for humans. The frameworks, the bundlers, the transpilers — they're developer ergonomics layers that the end user never benefits from. The browser engine itself (Blink, Gecko, WebKit) is among the most complex software ever built, largely because HTML/CSS has so many edge cases, layout modes, and backwards-compatibility requirements.

**The inefficiency is measurable.** A typical single-page application ships megabytes of JavaScript, CSS frameworks (95% unused), polyfills for browser inconsistencies, and transpiled output. The equivalent UI expressed as direct rendering instructions could be orders of magnitude smaller — potentially kilobytes instead of megabytes.

And now AI is writing more and more of this code. AI doesn't need developer ergonomics. It doesn't need readable class names or semantic HTML for its own sake. It could target something far more direct.

---

## The Proposal: Naze

Rather than patching the existing stack further, what if we built a parallel paradigm?

**Naze** — an AI-native, binary-first UI format designed from scratch for the AI era. Not an HTML replacement in the sense of "let's migrate everything." Instead, a parallel track — the way native apps (iOS/Android) exist alongside the web today, but with the web's openness preserved.

Think of it like the app store analogy: app stores proved that people will adopt a parallel ecosystem when it's meaningfully better. Native apps won massive marketshare from the web because they were faster, smoother, and more capable. Naze could offer those same advantages while staying open — anyone can publish, anyone can consume, no gatekeeper.

Key properties of Naze:
- **Binary format** — not human-readable text, but a compact, efficiently-parseable representation of UI structure, layout, interaction, and data binding
- **Intent-first** — describes *what* the UI should do, not *how* to render it through layers of abstraction
- **Open specification** — governed by a standards body (like W3C/WHATWG), not a single corporation
- **AI-native** — designed as a compilation target for AI-generated applications, not as something humans hand-author (though human-readable source formats would exist)

---

## WASM as the Engine

WebAssembly (WASM) is the most credible foundation for this. It already exists, it's already standardized, and it's already proving the concept in limited ways.

**What's happening today:**
- Flutter for web compiles Dart to WASM and renders to a canvas, bypassing the DOM entirely
- Figma runs a C++ engine compiled to WASM inside the browser
- WASI (WebAssembly System Interface) is extending WASM beyond the browser into a general-purpose runtime
- Game engines (Unity, Unreal) export to WASM for web targets

But today WASM is a *guest* inside the browser — it still lives within the HTML/JS sandbox, still goes through the browser's compositor, still plays by the browser's rules.

**The shift: WASM as the browser's core execution model, not a plugin.**

What if the "new browser" was essentially a WASM runtime with a GPU-accelerated rendering pipeline? Not WASM running *inside* Chrome, but WASM running *as* the renderer. The browser becomes a lightweight VM — closer to a game engine than a document viewer.

Possible architecture:
- **WASM runtime** handles application logic, state, interaction
- **GPU rendering layer** takes direct rendering instructions (draw this rectangle, render this text, composite these layers) — no CSS cascade, no DOM diffing, no reflow
- **Native UI primitives** — a small set of built-in components (text, image, input, scroll container, etc.) that the runtime renders natively, rather than building them from HTML elements
- **Networking/IO layer** — handles fetch, WebSocket, storage, etc. (similar to what browsers provide today but as a cleaner system interface)

This is not that far from what a game engine already does. The question is: can you make it serve the web's use cases (documents, apps, media) while keeping it lightweight and open?

---

## The Key Insight: It's the Language

Here's the realization that simplifies everything: **WASM already runs in every major browser.** Chrome, Firefox, Safari, Edge — they all have WASM runtimes today. And projects like Flutter/web, Figma, and game engines already prove you can bypass the DOM entirely by rendering through WASM + Canvas/WebGL.

The runtime isn't the missing piece. The infrastructure exists. What's missing is the **language layer** — a high-level, declarative, intent-based language designed for AI to author (and humans to read/edit) that compiles down to WASM.

**The actual stack:**

```
     Missing (needs to be built)          Already exists
     ───────────────────────────          ──────────────
  AI declarative language  ──→  Naze compiler  ──→  WASM runtime (in browsers)
  (the source format)           (lang → WASM)       Canvas/WebGL (rendering)
          ?                          ?                     ✓
```

Plus a **rendering library** — a WASM-based framework that provides UI primitives (text, layout, inputs, scrolling, animation) and renders directly to Canvas/WebGL. This replaces the DOM, not the browser.

This reframing has massive implications:

- **You don't need a new browser to start.** The language and apps can ship inside existing browsers today, right now, via WASM. No "wait for the new browser" chicken-and-egg problem.
- **The scope shrinks dramatically.** From "build a new browser + new format + new language" to "design a language + build a compiler + build a rendering library." Still ambitious, but tractable.
- **Adoption can be gradual.** A developer (or AI) can build a Naze app today and it runs in Chrome. No new browser download required. The "new browser" becomes a later optimization — a dedicated shell that runs the stack natively without the overhead of a full browser engine underneath.
- **The competitive moat is the language, not the runtime.** Whoever designs the right AI-native language wins, because the execution layer is commoditized.

Think about what happened with JavaScript: the language was the innovation, not the browser. Browsers were the delivery mechanism. Similarly, Naze's value isn't "a new browser" — it's "a new language that makes AI-generated web apps 100x more efficient," and existing browsers are the delivery mechanism.

---

## AI-Native Development Workflow

This is where the paradigm shift hits hardest. The entire modern web development toolchain exists because humans need help managing complexity. Remove the human-facing complexity, and the toolchain collapses.

**Today's workflow (human-centric):**
```
Human writes code → Framework abstracts complexity → Bundler packages it →
Browser interprets it → Rendering engine draws pixels
```

**Naze workflow (AI-native):**
```
Human/AI describes intent → Naze compiler generates binary → Runtime renders directly
```

The middle layers vanish. No bundler. No framework selection. No "should I use React or Vue or Svelte." No Tailwind vs CSS modules debate. No Webpack config files.

**What "describing intent" might look like:**

The Naze syntax should be its own purpose-built format — not markdown, not an existing language, but something designed from scratch to be readable like a document while being compilable. A non-developer should be able to read a `.naze` file and roughly understand what the app does. The syntax can also be rendered to a formatted preview (like markdown renders to HTML), giving users a "document view" of their app's structure.

Why not markdown? Markdown was designed for text documents, not UI applications. Adopting it (or Djot, AsciiDoc, MyST, etc.) would mean inheriting limitations and eventually creating extensions to handle layout, interaction, and data binding — which defeats the purpose of a clean-slate design.

**Example: A dashboard app**

```
-- dashboard.naze

app "Metrics Dashboard" {

  use app-shell(sidebar, main)
  data metrics: fetch("/api/metrics", refresh: 30s)

  sidebar: {
    nav-menu {
      link "Overview"  to: /
      link "Settings"  to: /settings
      link "Export"    to: /export
    }
  }

  main: {
    heading "Dashboard"

    grid(3 columns, responsive: stack below 768px) {
      card(each: metrics.items) {
        title: item.name
        value: item.current, format: number
        trend: item.history, display: sparkline
        on click: navigate to /detail/{item.id}
      }
    }
  }
}
```

A few things to note about this syntax:

- **Layout uses named slots on spatial primitives.** `app-shell(sidebar, main)` is a built-in layout template that defines two named regions. Under the hood it's a two-column grid. Users place content into named slots (`sidebar: { ... }`, `main: { ... }`), not into CSS grid cells.
- **Readable as prose.** Even without knowing the language, you can see: there's a sidebar with navigation links, a main area with a heading, and a grid of cards showing metrics.
- **No boilerplate.** No imports of React/Vue/Svelte. No CSS files. No `useState` or `useEffect`. No `className`. Data fetching, layout, and interaction are all inline.

**Example: Layout templates — using built-in and defining custom**

```
-- Built-in templates (ship with the standard library)
use app-shell(toolbar, sidebar, main, footer)
use dashboard(header, cards, detail)
use sidebar-layout(nav, content)
use split-view(left, right)

-- Custom template (defined using spatial primitives)
template "my-layout"(top, left-panel, center, right-panel) {
  grid {
    row(height: 60px, span: full) {
      slot top
    }
    row(fill: remaining) {
      column(width: 250px) { slot left-panel }
      column(fill: remaining) { slot center }
      column(width: 300px) { slot right-panel }
    }
  }
}
```

Named slot templates are the high-level API. Spatial primitives (grid, row, column, stack) are the low-level building blocks. You can use a preset or build your own.

**Example: Reusable components**

Components are defined in `.naze` files with declared inputs and content slots. They're imported and reused across pages — like a toolbar that appears on every page:

```
-- components/toolbar.naze

component toolbar(title: text, show-search: bool = true) {
  row(height: 56px, align: center, padding: 0 16px) {
    icon "menu", on click: toggle sidebar
    heading title, size: medium
    spacer
    if show-search { search-input placeholder: "Search..." }
    avatar current-user.image {
      on click: open user-menu
    }
  }

  accessibility {
    role: navigation
    label: "Main toolbar"
    keyboard: tab cycles through interactive elements
  }
}
```

```
-- pages/home.naze

use components/toolbar
use app-shell(toolbar-area, main)

toolbar-area: { toolbar title: "Home" }

main: {
  heading "Welcome back"
  -- ...rest of page
}
```

```
-- pages/settings.naze

use components/toolbar
use app-shell(toolbar-area, main)

toolbar-area: { toolbar title: "Settings", show-search: false }

main: {
  heading "Preferences"
  -- ...rest of page
}
```

The toolbar component is defined once, imported everywhere. It declares its inputs (`title`, `show-search`), its layout, its interactions, and its accessibility metadata — all in one place. No separate CSS file, no separate test file, no separate storybook story. The component *is* the complete description.

An AI generating this doesn't need to think about div nesting, CSS grid syntax, responsive breakpoints in media queries, JavaScript fetch error handling patterns, or React re-rendering strategies. It describes *what* should happen. The Naze compiler handles *how*.

Humans can read and edit this source format. But they don't have to touch the compiled output, ever. And AI can also skip the source format entirely and generate Naze binary directly if that's more efficient.

**What properties does this language need?**

Since the language is now the central missing piece, its design matters enormously:

- **Declarative, not imperative** — describe outcomes, not steps. "Show a sortable table of users" not "create a div, iterate over array, append child nodes, attach click handlers to headers, implement sort comparison function..."
- **Layout as a primitive** — grid, stack, scroll, responsive breakpoints baked into the language, not bolted on as a separate styling system
- **Data binding as a primitive** — connecting UI to data sources should be a first-class concept, not a framework feature
- **Interaction as a primitive** — click, drag, hover, keyboard, gesture — declared alongside the elements they apply to
- **AI-optimized grammar** — token-efficient (LLMs pay per token), unambiguous (no syntax that could be interpreted multiple ways), compositional (small pieces combine predictably)
- **Human-readable but not human-first** — readable enough for a developer to understand and modify, but optimized for AI generation speed and correctness rather than typing ergonomics
- **Statically analyzable** — the compiler should be able to catch errors at compile time, not at runtime in the browser. AI makes mistakes; the compiler should catch them before the user sees them

---

## Toolchain & Package System

The Naze toolchain is a single binary CLI — `nazec` — with no dependency on Node.js, npm, or any JavaScript tooling. No `node_modules`, no `package.json`. Think `cargo` or `go`, not `npm` + `vite` + `webpack`.

**The CLI:**

```
nazec build           # compile .naze → .wasm + index.html (meta-index)
nazec dev             # dev server with hot reload + inspector
nazec check           # type-check and validate without building
nazec add @org/lib    # add a dependency
nazec publish         # publish package to registry
nazec new my-app      # scaffold a new project
nazec test            # run component tests
nazec size            # analyze binary size
```

One binary. Downloads as a single executable. No install scripts, no postinstall hooks, no phantom dependencies.

**The manifest — `naze.toml`:**

Every Naze project has a `naze.toml` at the root. It's the equivalent of `package.json` / `Cargo.toml` / `go.mod`, but purpose-built for Naze.

```toml
[app]
name = "my-dashboard"
version = "0.1.0"
title = "Metrics Dashboard"

[dependencies]
charts = { source = "@naze/charts", version = "^1.0" }
icons  = { source = "@material/icons-naze", version = "^2.0" }
shared = { source = "../shared-components" }       # local path

[build]
target = ["wasm", "arm64"]         # which compilation targets
output = "dist/"
```

No `devDependencies` vs `dependencies` distinction. No `peerDependencies`. No lockfile sprawl. Dependencies are resolved, fetched, and cached by `nazec` itself.

**Packages are source:**

A Naze package is a directory of `.naze` source files + a `naze.toml`. When you `nazec add` a dependency, you get the actual `.naze` source — not a pre-compiled binary blob. This means:

- **Inspectable** — you can read any dependency's source to understand what it does
- **Forkable** — copy a component into your project and modify it
- **Tree-shakeable** — the compiler only compiles the components you actually use
- **Auditable** — no hidden code, no minified bundles, no supply-chain attacks hiding in compiled output

**Build cache:**

`nazec` maintains a local build cache (in `.nazec/cache/`). Dependencies are compiled once and cached. If the source hasn't changed, the cached compilation is reused. This gives you the speed benefits of pre-compiled packages without distributing opaque binaries.

```
.nazec/
  cache/          # compiled dependency artifacts
  registry/       # downloaded package sources
```

**Import syntax:**

```
-- from a published package
use @naze/charts/line-chart
use @material/icons-naze/icon

-- from a local path
use components/toolbar
use ../shared/auth-form
```

**Project structure:**

```
my-app/
  naze.toml                  # manifest
  app.naze                   # entry point
  components/
    toolbar.naze
    sidebar.naze
    metric-card.naze
  pages/
    home.naze
    settings.naze
    detail.naze
  dist/                     # build output
    app.wasm
    index.html              # meta-index (auto-generated)
```

**Registry (open question):**

The package registry is a later decision. Options:
- **Git-based** (like Go modules) — import directly from git URLs, no central registry, decentralized. Simplest to start.
- **Dedicated Naze registry** — purpose-built, with search/discovery. Requires infrastructure.
- **npm as transport** — publish `.naze` source packages to npm for distribution. Leverages npm's existing infrastructure and developer familiarity, even though the packages aren't JavaScript.

The `nazec` CLI is registry-agnostic — it resolves dependencies from whatever source the `naze.toml` specifies. The registry can evolve independently of the toolchain.

---

## Styling & Theming

No CSS. No separate stylesheet files. Styling in Naze is inline — part of the component definition — with design tokens for consistency and theming.

**Design tokens live in `theme.naze`:**

```
-- theme.naze

theme "my-app" {

  colors {
    primary:     #2563eb
    secondary:   #64748b
    danger:      #dc2626
    surface:     #ffffff
    surface-alt: #f8fafc
    text:        #0f172a
    text-muted:  #64748b
    border:      #e2e8f0
  }

  fonts {
    heading: "Inter", weight: 600
    body:    "Inter", weight: 400
    mono:    "JetBrains Mono", weight: 400
  }

  spacing { xs: 4px; sm: 8px; md: 16px; lg: 24px; xl: 32px }

  radii { sm: 4px; md: 8px; lg: 16px; full: 9999px }

  shadows {
    sm: 0 1px 2px rgba(0,0,0,0.05)
    md: 0 4px 6px rgba(0,0,0,0.1)
    lg: 0 10px 15px rgba(0,0,0,0.1)
  }
}
```

**Components reference tokens, not raw values:**

```
-- components/card.naze

component card(variant: "default" | "highlighted" = "default") {
  container {
    padding: theme.spacing.md
    radius: theme.radii.md
    shadow: theme.shadows.sm
    background: if variant == "highlighted" then theme.colors.primary else theme.colors.surface
    color: if variant == "highlighted" then theme.colors.surface else theme.colors.text

    slot content
  }
}
```

**Themes can be swapped without touching components:**

```
-- themes/dark.naze

theme "dark" extends "my-app" {
  colors {
    surface:     #1e293b
    surface-alt: #0f172a
    text:        #f8fafc
    text-muted:  #94a3b8
    border:      #334155
  }
}
```

The `extends` keyword means: only override what's specified, inherit everything else. Switching between light and dark is a theme swap at the app level — zero component changes.

**Inline styling for one-off overrides:**

```
-- Sometimes you need a specific value, not a token
button "Submit" {
  background: theme.colors.primary
  padding: theme.spacing.sm theme.spacing.md
  radius: theme.radii.md
  font: theme.fonts.body, size: 14px
  color: white    -- raw value override when needed
}
```

The key principle: **tokens are the default, raw values are the escape hatch.** The compiler can warn when raw values are used instead of tokens, encouraging consistency without enforcing it rigidly.

---

## Data Sources & Connections

Components shouldn't know how to connect to databases or APIs. They should declare what data they need, and a separate layer handles where it comes from.

**The data layer abstraction:**

A `sources.naze` file (or `[sources]` section in `naze.toml`) defines named data sources. Components reference sources by name. Connection details and credentials stay out of component code.

```
-- sources.naze

source api: rest {
  base: env.API_URL                -- from environment variable
  auth: bearer env.API_TOKEN
  timeout: 5s
}

source db: postgres {
  connection: env.DATABASE_URL
  pool: 5
}

source analytics: graphql {
  endpoint: env.GRAPHQL_URL
  auth: header "x-api-key" env.ANALYTICS_KEY
}

source notifications: websocket {
  url: env.WS_URL
  reconnect: true
}

source config: static { file: "config.json" }
```

**Components reference sources by name:**

```
-- pages/dashboard.naze

data metrics:    from api "/metrics", refresh: 30s
data user:       from db "SELECT * FROM users WHERE id = $current_user"
data live-stats: from notifications, channel: "stats"

main: {
  heading "Welcome, {user.name}"

  grid(3 columns) {
    card(each: metrics.items) {
      title: item.name
      value: item.current
    }
  }
}
```

The component says "I need metrics from the api source" — it doesn't know the base URL, the auth token, or the connection string. Those are configured in `sources.naze` and `naze.toml`, with secrets in environment variables.

**Source types:**
- `rest` — HTTP/REST APIs with GET/POST/PUT/DELETE
- `graphql` — GraphQL endpoints with queries/mutations
- `postgres` / `mysql` / `sqlite` — direct database connections (for server-side rendering or native apps)
- `websocket` — real-time streaming data
- `static` — local JSON/TOML/CSV files

**Why this matters:** a component library can be published and reused across projects that use completely different backends. The `@naze/charts` package doesn't care if your data comes from a REST API or a PostgreSQL database — it just receives data through the named source interface.

---

## Testing (Built Into the Language)

Tests in Naze are `.naze` files — written in the same language as the app. They're inspectable, reviewable, and AI-generated alongside the app code. No separate testing framework, no Playwright config, no Jest setup.

**Two layers:**

**1. Component tests** — test individual components in isolation:

```
-- tests/toolbar.test.naze

test "toolbar renders title" {
  render toolbar(title: "Hello")
  assert text "Hello" is visible
}

test "toolbar search is visible by default" {
  render toolbar(title: "Test")
  assert search-input is visible
}

test "toolbar hides search when disabled" {
  render toolbar(title: "Test", show-search: false)
  assert search-input is not visible
}

test "toolbar menu emits toggle-sidebar" {
  render toolbar(title: "Test")
  click icon "menu"
  assert emitted toggle-sidebar
}
```

**2. Flow tests** — test multi-page user journeys:

```
-- tests/user-flow.test.naze

flow "user can update settings" {
  navigate to /settings

  assert heading "Preferences" is visible

  toggle "Email notifications"
  assert toggle "Email notifications" is on

  select "Theme", choose: "Dark"
  assert select "Theme" shows "Dark"

  click button "Save"
  assert notification "Settings saved" appears

  navigate to /settings
  assert toggle "Email notifications" is on    -- persisted
  assert select "Theme" shows "Dark"            -- persisted
}
```

**Tests as readiness constraints:**

Tests aren't just for CI — they define "what does it mean for this app to be ready?" An AI generating a Naze app can also generate the tests, then run them to verify the app meets the requirements before presenting it to the user.

```
-- tests/ready.test.naze

flow "app is ready" {
  navigate to /
  assert page loads within 2s
  assert no accessibility violations
  assert all images have alt text
  assert toolbar is visible
  assert navigation has 3 links
  assert data loads without errors
}
```

**Running tests:**

```
nazec test                     # run all tests
nazec test tests/toolbar       # run specific test file
nazec test --component         # component tests only
nazec test --flow              # flow tests only
nazec test --watch             # re-run on file changes
```

Tests run in a headless Naze renderer (the same C5 renderer used for the app). No browser needed for component tests. Flow tests can optionally run in a real browser for screenshot comparison.

**Why `.naze` test files matter:**
- **Same language** — no learning Jest, Playwright, Vitest. If you can read a `.naze` app, you can read a `.naze` test.
- **AI-native** — an LLM generating an app can generate tests in the same prompt, same syntax, same mental model.
- **Inspectable** — tests are readable descriptions of expected behavior, not opaque assertion chains.
- **Constraint-based** — tests describe what "correct" looks like. The AI (or developer) builds until all constraints pass.

---

## Server-Side Rendering & Deployment

Modern web frameworks (Next.js, Nuxt, SvelteKit) use server-side rendering: the server generates HTML on each request, the browser shows it instantly, then JavaScript "hydrates" the page to make it interactive. This solves two problems: fast first paint (no blank white screen while JS loads) and SEO (crawlers see full HTML content).

Canvas-based WASM frameworks (Flutter, Makepad, Compose Multiplatform) **cannot do SSR**. They render pixels to a canvas — there's no semantic HTML to serialize. When you load a Flutter web app, you stare at a blank screen until the 1.5MB CanvasKit binary downloads and initializes.

**Naze has a unique advantage here.** Because Naze is declarative and the compiler understands the full semantic structure of the app — what text, what layout, what data — it *can* generate an HTML representation alongside the WASM binary. The compiler knows what a page *means*, not just what pixels to draw. No other canvas-based WASM framework can do this.

### Three Rendering Modes

Configurable per-route, like Next.js:

**1. Static (SSG)** — `nazec build` pre-renders each route to HTML at build time. Deploy as static files to any CDN. WASM takes over on the client.

```toml
# naze.toml
[routes]
"/"         = { render = "static" }
"/about"    = { render = "static" }
"/blog/*"   = { render = "static" }
```

Best for: content sites, marketing pages, documentation, blogs.

**2. Server-rendered (SSR)** — a server generates HTML on each request. Can run as a WASM binary on edge platforms or a native binary in a container. WASM takes over on the client.

```toml
[routes]
"/dashboard"  = { render = "server" }
"/profile/*"  = { render = "server" }
```

Best for: personalized content, dynamic data, authenticated pages.

**3. Client-only (SPA)** — HTML shell + WASM binary. No server rendering. Client downloads WASM, renders via Canvas/WebGL.

```toml
[routes]
"/app/*"    = { render = "client" }
```

Best for: app-like experiences (dashboards, editors, tools) where SEO is less important.

### How SSR Works (The Compiler's Advantage)

The C2 compiler emits two outputs from the same `.naze` source:

```
.naze source
    │
    ├──→ WASM binary         (client-side Canvas/WebGL rendering)
    │
    └──→ HTML representation (server-side or build-time output)
```

The HTML output isn't a full app — it's a semantic representation: headings, text, images, links, structured data. Enough for instant first paint and full SEO indexing. When the WASM binary loads, it replaces the HTML with canvas rendering (similar to React hydration, but switching from DOM to canvas rather than enhancing the DOM).

This subsumes the HTML meta-index concept described in the Discoverability section below — the meta-index is just the simplest form of server rendering (metadata only). SSG and SSR produce richer HTML with actual content.

### Server Functions (Server-Side Compute)

SSR handles rendering. But apps also need server-side logic — database queries, authentication, heavy calculations, third-party API calls that shouldn't run in the browser.

Naze handles this with `server` functions — code that runs on the server, never ships to the client WASM bundle:

```
-- pages/dashboard.naze

server function calculate-risk(portfolio: list) -> risk-score {
  -- this runs on the server, not in the browser
  -- can access databases directly, use secret API keys
  data rates: from db "SELECT * FROM interest_rates"
  return compute-var(portfolio, rates, confidence: 0.95)
}

-- the component calls it like a local function
-- the compiler generates the RPC call automatically
main: {
  button "Calculate Risk" {
    on click: result = calculate-risk(user.portfolio)
  }

  if result {
    card {
      title: "Portfolio Risk Score"
      value: result.score, format: number
    }
  }
}
```

The component doesn't know the function runs remotely. The compiler auto-generates client-side stubs (HTTP POST with JSON-serialized arguments) and server-side handlers. Type safety is enforced at compile time — the client stub and server function share the same type signature from the `.naze` source.

**Three server compute modes:**

1. **Server functions** — individual functions marked `server` in `.naze` files. Called via RPC from client. Like Next.js Server Actions.
2. **API routes** — `.naze` files in an `api/` directory define HTTP endpoints. Handle webhooks, external integrations, third-party callbacks.
3. **Edge functions** — server functions that run at CDN edge nodes. Marked `server(edge)`. For auth checks, personalization, geolocation.

### The Compiled Server

The server is a single compiled binary. No Node.js, no JVM, no runtime dependencies.

**Two compilation targets:**

1. **WASM binary** (for edge/serverless) — runs on WASI-compatible runtimes: Wasmtime (Fermyon Spin, Fastly), V8 isolates (Cloudflare Workers), WasmEdge (AWS Lambda). This is how Leptos already deploys to Cloudflare Workers today.

2. **Native binary** (for containers/VPS) — compiled to x86-64 or ARM. Standalone executable like a compiled Go or Rust server. Runs in Docker, on EC2, anywhere. ~5-20MB binary. Starts in milliseconds.

```
server binary (WASM or native)
  ├── HTTP listener           (embedded — no external web server needed)
  ├── Router                  (URL routes + server function RPC endpoints)
  ├── Server Renderer         (renders .naze → HTML for SSR/SSG, no GPU needed)
  ├── Server Functions        (compiled from `server` blocks in .naze files)
  ├── Data Source Manager     (database pools, API clients from sources.naze)
  └── Static Asset Server     (serves client .wasm, images, fonts)
```

### Build Output

```
nazec build                    # SSG: pre-render all routes to HTML + client WASM
nazec build --client-only      # SPA: HTML shell + client WASM only
nazec build --server           # fullstack: server binary + client WASM bundle

# SSG output:
dist/
  index.html                  # pre-rendered HTML (SSG)
  about/index.html            # pre-rendered route
  app.wasm                    # client WASM binary
  assets/                     # images, fonts

# Fullstack output:
dist/
  client/
    app.wasm                  # client bundle (no server code)
    assets/
  server/
    server.wasm               # server binary (WASM target)
    -- OR --
    server                    # server binary (native target)
```

### Deployment

Naze deploys to every major platform with zero configuration changes:

| Platform | Static (SSG) | SSR/Server Functions | Edge | Deploy command |
|----------|-------------|---------------------|------|----------------|
| **Vercel** | Upload dist/ | Serverless functions | Edge functions (2-4MB limit) | `vercel deploy` |
| **Cloudflare** | Pages (free, unlimited) | Workers | Workers (330+ cities) | `wrangler deploy` |
| **AWS** | S3 + CloudFront | Lambda | Lambda@Edge | CDK/Terraform |
| **Netlify** | Upload dist/ | Functions | Partial (Deno edge) | `netlify deploy` |
| **Fastly Compute** | — | Native WASI | Full WASI, µs cold starts | `fastly compute publish` |
| **Fermyon Spin** | — | Native WASM | 0.52ms cold starts | `spin deploy` |
| **Docker** | nginx + dist/ | Native binary | — | `docker run` |

**The SSG path is the simplest.** `nazec build` produces static files. Upload to any CDN. No server needed. This covers the majority of sites.

**The fullstack path** adds server functions and SSR. The server binary (WASM or native) IS the deployment artifact. No runtime dependencies to install, no `node_modules` to upload.

### First Paint Solution

The "blank white screen" problem that plagues Flutter/Makepad/Compose is solved by SSR/SSG:

```
1. User requests page
2. Server returns pre-rendered HTML (instant — from CDN cache or SSR)
3. Browser renders HTML immediately → user sees content
4. Browser downloads .wasm binary (cached at edge, Brotli compressed)
5. WASM initializes → canvas rendering takes over
6. Transition is seamless (same visual output, now interactive)
```

For client-only mode: the compiler generates a loading skeleton from the `.naze` layout structure (it knows the page dimensions, the component positions, where text goes). Not a blank screen — a structural preview.

---

## Input Handling Without the DOM

When Naze renders everything to Canvas/WebGL, there are no DOM elements to click, no HTML inputs to type into, no browser focus system. How does user input work?

**This is a solved problem.** Flutter, Makepad, Figma, and Google Docs canvas mode all handle input in canvas-based rendering. Naze uses the same proven patterns.

### Hit Testing — "Which Element Did I Click?"

The C4 Layout Engine already computes exact positioned rectangles for every element on screen. When the user clicks at pixel coordinate (x, y), hit testing is a tree walk: "which positioned rectangle contains this point?" Check children in reverse order (topmost layer first), respect clipping regions, bubble events up the tree.

This is identical to how game engines handle input. No DOM required — the layout tree IS the coordinate system.

### Text Input — Keyboard, Cursor, Selection

The C4a Text Engine (HarfBuzz/FreeType) already computes exact glyph positions for all rendered text. When the user clicks inside a text field:

- **Cursor placement**: "which glyph boundary is nearest to x?" — a spatial query on data the text engine already has
- **Text selection**: two cursor positions defining a range. Mouse-down sets start, mouse-drag updates end. Render a highlight rectangle between them.
- **Keyboard input**: browser keyboard events are available to WASM via JS interop. The C3 Runtime dispatches them to the focused text field.

The text engine knows where every character is. Cursor and selection are just visual state drawn on top.

### Focus Management

The C3 Runtime maintains a `focusedElement` reference — internal state, not DOM focus. Tab order is computed from the layout tree in visual order (top-to-bottom, left-to-right). Keyboard events route to the focused element. Components render their own focus indicators (border color, outline) based on theme tokens.

### IME — The Genuinely Hard Part

IME (Input Method Editor) is how users type Chinese, Japanese, Korean, emoji, and accented characters. Canvas elements don't trigger IME — the browser needs an actual input element.

**The proven solution: a hidden `<input>` element behind the canvas.** When a Naze text field receives focus, a hidden (invisible) HTML input element is positioned at the cursor location. The browser's IME system sees it and works normally. Naze reads the input and renders the text on canvas. The user never sees the hidden element.

This is exactly what Flutter Web does. And what Figma does. And Google Docs canvas mode.

**Naze's elegant advantage:** The C6 Accessibility Bridge already creates a "side DOM" — a hidden HTML tree for screen readers. The hidden input element lives in this same side DOM. One hidden DOM tree serves both accessibility and IME. No additional DOM management needed.

```
Canvas (visible — what the user sees)
  └── GPU-rendered text fields, buttons, content

Side DOM (invisible — maintained by C6 Accessibility Bridge)
  ├── ARIA tree nodes (for screen readers)
  └── Hidden <input> elements (for IME support)
```

### Native Platforms (Phase 2+)

On iOS, Android, and desktop, the hidden input trick isn't needed. These platforms expose IME APIs directly:
- **iOS**: UITextInput protocol
- **Android**: InputConnection interface
- **macOS/Windows/Linux**: native IME APIs

The C3 Runtime's event system is platform-agnostic — same hit testing, focus management, and event dispatch regardless of whether input comes from browser events or native OS events.

### Copy/Paste

The browser's Clipboard API (`navigator.clipboard`) is available to WASM via JS interop. The C3 Runtime knows exactly what text is selected (it tracks selection state). Ctrl+C reads the selected text and writes it to the clipboard. Ctrl+V reads from the clipboard and inserts at the cursor position. Right-click renders a Naze-drawn context menu with Copy/Paste options. This is how Flutter, Figma, and Google Docs canvas mode all handle it.

### Find in Page (Ctrl+F)

The browser's native Ctrl+F searches the DOM. Canvas content is invisible to it. Naze provides a built-in find overlay instead — like VS Code's Ctrl+F. The runtime has access to all text content in the layout tree, so it can search the actual content, highlight matches, and navigate between them. Google Docs canvas mode took the same approach — their Ctrl+F searches the document model, not the DOM.

Naze's version is arguably better than browser Ctrl+F because the compiler knows the full semantic structure. It can search across component boundaries, skip non-visible content, and search structured data (not just rendered text).

### Browser Extensions

This is the most honest trade-off of canvas-based rendering. Some extensions work, some don't:

- **Password managers** (1Password, Bitwarden, LastPass) — work. The C6 hidden input elements (already there for IME) can be annotated with `autocomplete="current-password"` and `type="password"`. Password managers detect these. Same approach Flutter uses.
- **Ad blockers** — work. They operate at the network level. No DOM needed.
- **Screen readers** — work. Already handled by C6's side DOM.
- **Dark mode extensions** (Dark Reader) — don't work on canvas. But Naze has native theming (`theme.naze` with `extends` for dark variants), so dark mode is solved at the language level instead of needing a browser extension.
- **Translation extensions** — don't work on canvas text directly. But Naze can expose text content in the C6 side DOM for translation tools, and internationalization is a planned language feature.
- **DOM-manipulating extensions** — don't work. This is a genuine loss, same as every canvas framework (Flutter, Figma). The trade-off is deterministic rendering and GPU performance.

The pattern: extensions that work at the network level or through accessibility APIs work fine. Extensions that inject into or modify the DOM don't. Naze's language-level features (theming, i18n, accessibility) cover many of the use cases that DOM-manipulating extensions serve.

### Screen Sizes & Responsive Rendering

Responsive layout is already a first-class language feature:
```
grid(3 columns, responsive: stack below 768px)
column(width: 250px, collapsible: below 1200px)
```

Under the hood, the runtime handles:

- **Resize detection** — the canvas listens to `window.resize` events (browser) or viewport change notifications (native). When the window size changes, C4 Layout Engine recalculates all positions with the new dimensions, and C5 re-renders. This is the same resize → re-layout → re-render cycle that every UI framework uses.

- **Device pixel ratio / retina** — the canvas element is sized at CSS pixels but its backing buffer renders at the device's physical pixel density (`devicePixelRatio`). On a 2x retina display, a 400x300 CSS-pixel canvas has an 800x600 backing buffer. Text and UI elements render crisp at native resolution. This is standard canvas technique — Flutter, Makepad, and every game engine do it.

- **Viewport configuration** — the HTML shell (generated by C8/C14) includes `<meta name="viewport" content="width=device-width, initial-scale=1">`. Mobile browsers report correct dimensions to the WASM runtime. The C4 Layout Engine receives the actual viewport size and computes layout accordingly.

- **Orientation changes** — mobile device rotation triggers a resize event. The layout engine recalculates. If the layout uses responsive breakpoints, the UI reorganizes (e.g., sidebar collapses, grid switches to stack). Same resize handling, just triggered by hardware rotation.

### The Key Point

The layout engine computes coordinates. The text engine computes glyph positions. Hit testing is a rectangle lookup. Cursor placement is a glyph boundary lookup. IME and password managers are handled by the same side DOM that accessibility needs. Copy/paste uses the Clipboard API. Ctrl+F searches the layout tree. Responsive layout is a language primitive. DPI is handled at the canvas buffer level.

Every piece of Naze's existing architecture contributes — no new subsystem required, just wiring together components that already exist for other reasons. The only genuine loss is DOM-manipulating browser extensions, and Naze's language-level features cover most of those use cases natively.

---

## Rendering Pipeline & Performance

A natural question: if Naze renders everything to pixels via Canvas/WebGL, how much work does the device do compared to a traditional web app? What happens on resize? And how do animations work?

### Resize Is Continuous, Not Pre-Determined

Screen sizes are not pre-determined. The C4 Layout Engine handles any arbitrary dimensions — there's no fixed set of sizes it expects. The `responsive: stack below 768px` syntax is a declarative constraint, but the layout engine computes positions for whatever window size it receives.

When the browser window resizes:

```
1. Browser fires resize event
2. C3 Runtime receives new width/height
3. C4 Layout Engine recalculates ALL element positions with new dimensions
   (single-pass, top-down — no cascade resolution, no reflow chains)
4. C5 Renderer draws one new frame to canvas
```

This is a single-pass pipeline. The layout engine doesn't need to resolve CSS inheritance or cascade — it reads the layout tree directly. For a typical UI with hundreds of elements, a full layout recalculation takes under 1ms. Game engines do this 60 times per second with thousands of elements.

If the layout uses responsive breakpoints, the recalculation handles them automatically — a three-column grid becomes a single-column stack when the width drops below the threshold. This isn't a "media query" that triggers a separate layout mode — it's the same layout computation with different input dimensions.

### Rendering Workload: Naze vs. Traditional DOM

Every time something changes in a traditional web app, the browser runs through a multi-stage pipeline:

```
Traditional DOM pipeline (on every state change):
  1. Parse HTML → DOM tree
  2. Parse CSS → CSSOM
  3. Cascade + specificity resolution → Render tree
  4. Layout (reflow) → compute positions
  5. Paint → generate display lists
  6. Composite → GPU composites layers

Naze pipeline (on every state change):
  1. C4 Layout Engine → compute positions
  2. C5 Renderer → draw directly to GPU
```

**Naze skips steps 1-3 entirely.** There's no HTML to parse, no CSS cascade to resolve, no specificity rules to evaluate, no render tree to construct. These are the most expensive parts of DOM rendering — CSS cascade resolution alone can dominate render time on complex pages with thousands of rules.

Additional overhead Naze avoids:
- **No virtual DOM diffing** — React's reconciliation, Vue's reactivity tracking, Svelte's compiled updates. Naze's layout engine works directly on the component tree.
- **No style recalculation cascades** — in the DOM, changing one element's class can trigger style recalculation on ancestors, siblings, and descendants. Naze's layout is a single top-down pass.
- **No reflow chains** — in the DOM, changing one element's size can cause its parent to resize, which causes siblings to reflow, which causes the page to shift. Naze's layout engine resolves constraints in one pass.
- **No JS↔DOM bridge overhead** — DOM operations cross the JavaScript/native boundary. Naze's rendering stays entirely within WASM.

**The honest trade-off:** browsers have decades of rendering optimization — incremental layout, layer compositing, scroll optimization, text rendering caches, GPU texture atlases. Naze needs to build some of these optimizations over time. For Phase 1, Naze will be simpler but not necessarily faster than a highly-optimized browser page. The performance advantage grows as apps get more complex — more components, more state changes, more layout recalculation — because Naze's overhead grows linearly while DOM overhead grows superlinearly (cascade, reflow chains, style recalculation all compound).

**Real-world proof:** Flutter Web already renders via Canvas/WebGL and achieves 60fps for complex UIs. Figma handles massive design files with thousands of objects at 60fps. The performance model is proven.

### Animations — First-Class, Declarative, GPU-Accelerated

Animations in Naze are a language feature, not a separate system bolted on like CSS transitions/animations. You declare what should animate, and the runtime handles the rest.

**Sidebar that slides in:**
```
component sidebar(open: bool) {
  column(width: 250px) {
    animate position-x: if open then 0 else -250px {
      duration: 300ms
      easing: ease-out
    }

    slot content
  }
}
```

When `open` changes from `false` to `true`, the sidebar slides in over 300ms with ease-out timing. The runtime handles the interpolation frame-by-frame.

**Button with hover animation:**
```
button "Submit" {
  background: theme.colors.primary
  transition background: 150ms ease
  transition scale: 200ms spring

  on hover {
    background: theme.colors.primary-dark
    scale: 1.02
  }
}
```

When the user hovers, the background color transitions over 150ms and the button scales up with spring physics. When the hover ends, both animate back.

**How it works under the hood:**

The C3 Runtime has an animation scheduler that runs on `requestAnimationFrame` (60fps):

1. Component declares an animation (`animate`, `transition`)
2. When the animated property changes, the scheduler registers an active animation with start value, end value, duration, and easing function
3. Each frame: scheduler interpolates the current value based on elapsed time and easing curve
4. Updated values are applied to the layout tree
5. C5 Renderer draws the frame
6. When animation completes: final value is set, animation is removed from the scheduler

**GPU optimization — the key performance insight:**

Not all property changes require a full re-layout:

- **Transform-only changes** (position, rotation, scale) → C5 can update the GPU transform matrix without calling C4. No re-layout needed. This is essentially free.
- **Opacity changes** → same thing. Just update the GPU blend parameter.
- **Color/background changes** → update the draw command's color. No re-layout.
- **Size changes** (width, height, padding) → DO require C4 re-layout, because they affect sibling and parent positioning.

This is the same optimization that browsers use: CSS `transform` and `opacity` animations are "composited" (GPU-only, no reflow), while `width`/`height`/`margin` animations trigger expensive reflow. Naze makes this distinction explicit — the developer doesn't need to know which properties are "safe" to animate because the runtime handles it automatically, but transform/opacity animations are effectively zero-cost.

**Animation types:**

| Type | Syntax | Use case |
|------|--------|----------|
| **Property animation** | `animate opacity from 0 to 1 over 300ms` | Explicit start/end animation |
| **Transition** | `transition background: 150ms ease` | Auto-animate when value changes |
| **Spring** | `animate position-y: target, physics: spring(stiffness: 300, damping: 20)` | Natural-feeling motion (like iOS) |
| **Keyframe** | `animate scale: [1, 1.2, 0.95, 1] over 400ms` | Multi-step animation |

**Easing curves:** `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`, `cubic-bezier(x1, y1, x2, y2)`, `spring(stiffness, damping)`

All of this compiles down to the WASM binary. The animation scheduler is part of the C3 Runtime — no JavaScript animation libraries, no CSS keyframe overhead. The GPU does the actual interpolation rendering at native speed.

### Rendering Optimizations — Partial Repaint, Layer Compositing, Caching

Naze doesn't redraw the entire screen on every frame. Like game engines, it uses a hierarchy of optimizations from cheapest to most expensive:

**Level 1 — Skip unchanged subtrees (zero cost):**
If a component's state hasn't changed since the last frame, don't even walk its tree. The C3 Runtime tracks which components have "dirty" state. Unchanged components produce no draw commands — their previous frame's output is still valid. For typical UIs where most of the screen is static on any given frame, this is the biggest win.

**Level 2 — Layer compositing (the big one):**
The UI is separated into independent GPU texture layers. Each layer is rendered to its own offscreen framebuffer. When compositing the final frame, the GPU just overlays the textures — no re-rendering of unchanged layers.

```
Layer 0: Background (white) ← never repaints after first frame
Layer 1: Toolbar             ← repaints only when toolbar state changes
Layer 2: Main content        ← repaints on navigation (new page content)
Layer 3: Overlays/modals     ← repaints only when shown/hidden
```

Navigate to a new page with the same white background → only Layer 2 (main content) repaints. Layers 0, 1, 3 reuse their cached GPU textures. The final composition (overlaying 4 textures) is essentially free on GPU.

This is exactly how browsers work (CSS `will-change: transform` creates a compositing layer), how Flutter works (its layer tree separates independent regions into GPU textures), and how game engines work (background layers, entity layers, UI layers rendered independently).

**Naze's advantage:** the compiler knows the component tree structure at compile time. It can automatically determine which subtrees should be separate layers (toolbar, sidebar, main content area, modals) without developer hints. Browsers need CSS `will-change` or rely on implicit heuristics at runtime, which leads to over-compositing (too many layers → GPU memory waste), under-compositing (too few layers → unnecessary repaints), and "layer explosion" bugs in complex CSS. Naze's compiler makes these decisions statically from the component tree.

**Level 3 — Dirty rectangle tracking:**
Even within a layer that needs repainting, only redraw the region that actually changed. If a text input cursor blinks in one corner of the main content area, clip the repaint to just that rectangle, not the entire layer. The C3 Runtime tracks which elements changed, computes their bounding boxes, and passes dirty rectangles to C5, which clips rendering to those regions.

**Level 4 — Texture caching for complex components:**
Expensive components (data visualizations, charts, large images with filters, complex shapes) are rendered to their own GPU textures. The cached texture is reused on every frame until the component's data changes. This turns an O(n) render into a single texture blit.

**Additional standard GPU techniques:**
- **Glyph atlas** — already documented: rendered glyphs cached as GPU textures
- **Draw call batching** — group similar draw commands (same shader, same texture) into single GPU calls, reducing CPU→GPU overhead
- **Frustum culling** — skip elements entirely outside the visible viewport (important for long scrollable content)
- **Occlusion culling** — skip elements hidden behind opaque elements (e.g., content behind a modal backdrop)

### The Performance Verdict — Is Naze Faster Than the DOM?

The existing pipeline comparison (above) shows that Naze skips the parse/cascade/specificity stages entirely. The rendering optimizations above show that Naze matches the browser's compositing and caching strategies. So where does that leave us?

**Where Naze is structurally faster:**
- **No parse/cascade/specificity resolution** — the three most expensive stages of DOM rendering are eliminated entirely. CSS cascade resolution alone can dominate render time on pages with thousands of rules.
- **No virtual DOM diffing** — no React reconciliation, no Vue reactivity tracking. The runtime works directly on the component tree.
- **No JS↔DOM bridge** — DOM operations in traditional web apps cross the JavaScript-to-native boundary on every call. Naze stays entirely within WASM.
- **Compile-time layer assignment** — the compiler determines optimal compositing layers statically, versus browsers guessing at runtime with heuristics that frequently over- or under-composite.
- **Single-pass layout** — no reflow chains. In the DOM, changing one element's size can cascade through ancestors, siblings, and descendants. Naze's layout resolves in one top-down pass.
- **No style recalculation cascades** — in the DOM, adding a CSS class can trigger style recalculation across the entire subtree. Naze has no cascade.

**Where browsers have the edge (today):**
- **Decades of incremental optimization** for common patterns — scrolling, text rendering, image decoding are all highly tuned.
- **Hardware-accelerated scroll compositing** — browsers have purpose-built scroll optimization that Naze needs to implement.
- **Mature text rendering pipeline** — subpixel antialiasing, font fallback chains, and text layout edge cases are deeply optimized.

**The verdict:** DOM overhead scales superlinearly with application complexity. Cascade resolution, reflow chains, and style recalculation all compound — each additional component doesn't just add its own cost, it increases the cost of processing every other component. Naze's overhead scales linearly — each component is independent, layout is a single pass, and there's no cascade to resolve.

For a simple static page with minimal interactivity, browsers are already fast enough that the difference is negligible. For complex, interactive applications — dashboards, editors, data-heavy UIs with hundreds of components and frequent state changes — Naze's architecture is structurally faster because it eliminates the abstraction layers where DOM overhead compounds. This is the same fundamental reason native mobile apps feel faster than web apps at scale: they skip the abstraction layers between application code and pixels. Naze brings that architectural advantage to the web platform.

This isn't theoretical. Flutter Web renders complex UIs at 60fps via Canvas/WebGL. Figma handles massive design files with thousands of objects at 60fps. Game engines manage tens of thousands of objects with layer compositing and partial repaint. The rendering model is proven at scale — Naze applies the same techniques with the added advantage of compile-time optimization.

---

## Computation in Naze — Three Tiers

Naze is a declarative UI language, not a general-purpose programming language. But apps need logic — sorting, filtering, validation, data transformation, complex algorithms. Where does that live?

Naze answers this with three tiers of computation, from simplest to most powerful:

### Tier 1: Built-In Declarative Logic (Client-Side)

Naze includes functional, expression-oriented constructs that fit its declarative philosophy. No imperative loops, no mutation — data flows through transformations.

**Pipeline operators** — chain transformations on data:
```
data active-users: users | filter(active) | sort-by(.last-login) | take(10)
```

**Pure functions** — define reusable logic with expression bodies:
```
function format-price(amount: number, currency: text) -> text {
  match currency {
    "USD" -> "$" + amount.round(2)
    "EUR" -> "€" + amount.round(2)
    _     -> amount.round(2) + " " + currency
  }
}
```

**Pattern matching** — exhaustive branching without `if`/`else` chains:
```
match order.status {
  "pending"   -> badge "Pending", color: yellow
  "shipped"   -> badge "Shipped", color: blue
  "delivered" -> badge "Delivered", color: green
  _           -> badge "Unknown", color: gray
}
```

**List comprehensions:**
```
let names = [user.name for user in users if user.active]
```

**Local `let` bindings** — computed values within a component:
```
component user-dashboard(users: list) {
  let active = users | filter(active)
  let total-revenue = active | map(.revenue) | sum
  let avg-revenue = total-revenue / active.length

  card {
    title: "Active Users: {active.length}"
    subtitle: "Avg Revenue: {format-price(avg-revenue, 'USD')}"
  }
}
```

This covers ~80% of client-side logic: filtering, sorting, formatting, validation, conditional display. No `while` loops, no mutation, no side effects — all transformations return new values. The compiler can statically analyze every expression, which keeps the language AI-optimizable and makes bugs easy to catch at compile time.

### Tier 2: WASM Library Imports (Client-Side — Like Java's JNI)

For anything that doesn't fit the declarative model — complex algorithms, cryptography, data parsing, ML inference — Naze can import pre-compiled WASM modules. This is the JNI parallel: call out to native-speed code written in Rust, C, Go, or any language that compiles to WASM.

```
import crypto from "naze-crypto"           -- community package, ships as .wasm
import csv from "naze-csv"                 -- CSV parser, compiled from Rust
import utils from "./lib/my-utils"        -- local Rust project compiled to .wasm

component data-import {
  let hash = crypto.sha256(file.contents)
  let parsed = csv.parse(file.contents, delimiter: ",")
  let validated = parsed | filter(row -> row.length == expected-columns)

  table(data: validated) {
    column "Name": row.name
    column "Email": row.email
    column "Hash": crypto.sha256(row.email)
  }
}
```

**How the compilation works:**

The `nazec build` pipeline merges everything into a single binary:

```
1. Compile .naze files → WASM
2. Resolve import declarations → locate .wasm modules (from packages or local paths)
3. wasm-merge → combine all WASM modules into one binary
4. wasm-opt → tree-shake unused functions, dead code eliminate, optimize
5. Output: single .wasm file
```

After merging, imported functions aren't "foreign" calls — they're normal function calls within the same WASM binary. There's no bridge, no FFI boundary, no marshalling overhead. This is fundamentally different from (and faster than) how other platforms handle native interop:

| Platform | Mechanism | Runtime overhead |
|----------|-----------|-----------------|
| Java/JNI | Call native C/C++ from JVM | High — JNI bridge, memory copying, GC coordination |
| Flutter FFI | Call Rust/C via dart:ffi | Medium — FFI boundary crossing |
| React + WASM | Import .wasm via JS fetch + instantiate | Medium — JS↔WASM bridge on every call |
| **Naze** | **Merge .wasm at compile time** | **Zero — same binary after merge** |

Naze's approach works because both the app and the libraries target the same compilation output (WASM). There's no bridge between runtimes — it's all one binary.

**The Rust/WASM ecosystem is large.** Many production libraries already compile to WASM: serde_json (JSON parsing), chrono (date/time), regex, flate2/brotli (compression), unicode-segmentation (text processing), and more. Naze packages can ship as either `.naze` source files (source-distributed, inspectable) or `.wasm` pre-compiled modules (for Rust/C libraries). Both are tree-shaken during compilation — import a large library, only pay for the functions you actually call.

**Bundle size management:**
- `wasm-merge` + `wasm-opt` with LTO (link-time optimization) typically reduces merged binary size by ~50%
- A typical app with a few library imports: 200-500KB compressed
- Lazy loading (opt-in): large libraries can be split into separate `.wasm` chunks loaded on demand, like code splitting in JavaScript bundlers

### Tier 3: Server Functions (Server-Side)

Already documented in the SSR section — `server function` runs on the server via auto-generated RPC. For anything that needs database access, API keys, heavy computation, or secrets that shouldn't ship to the client.

**The three tiers work together:**
```
import geo from "naze-geo"                          -- Tier 2: WASM library

function format-distance(meters: number) -> text {  -- Tier 1: pure function
  match {
    meters < 1000  -> "{meters}m"
    meters < 10000 -> "{(meters / 1000).round(1)}km"
    _              -> "{(meters / 1000).round(0)}km"
  }
}

server function find-nearby(lat: number, lon: number) -> list {  -- Tier 3: server
  data places: from db "SELECT * FROM places WHERE active = true"
  return places
    | map(p -> { ...p, distance: geo.haversine(lat, lon, p.lat, p.lon) })
    | filter(p -> p.distance < 5000)
    | sort-by(.distance)
}

component nearby-places(location: geo-point) {
  data places: find-nearby(location.lat, location.lon)     -- calls server
  let formatted = places | map(p -> { ...p, dist-label: format-distance(p.distance) })

  list(data: formatted) {
    row {
      text item.name, weight: bold
      text item.dist-label, color: theme.colors.secondary
    }
  }
}
```

### Why Naze Doesn't Need to Be a General-Purpose Language

The three-tier model means Naze never needs `while` loops, mutable variables, or imperative control flow. Tier 1 handles declarative data transformation. Tier 2 handles anything algorithmically complex via pre-compiled WASM. Tier 3 handles server-side computation.

This is deliberate. By constraining the language:
- **AI optimization**: the output space stays small — a fine-tuned 3-7B model can generate correct Naze because there are fewer ways to express the same thing
- **Compile-time verification**: pure functions and immutable data mean the compiler can catch more bugs statically
- **Readability**: a non-developer can read a `.naze` file and understand what the app does, even if it imports complex libraries
- **Security**: no arbitrary code execution in the UI layer — imported WASM runs in the same sandbox, and server functions are isolated

The parallel to SQL is instructive: SQL is declarative, doesn't have `while` loops, and is incredibly powerful for its domain. When you need imperative logic, you write a stored procedure or handle it in application code. Naze follows the same pattern — declarative for UI and data flow, with escape hatches to full computation when needed.

---

## Discoverability: The HTML Meta-Index Bridge

One of the biggest practical challenges: how do search engines find and index Naze sites?

Google, Bing, and every web crawler in existence speak HTML. They parse `<title>`, `<meta>`, `<h1>`, structured data (JSON-LD). If your site is a binary blob, it's invisible to search.

**The solution: a transitional HTML meta-index.**

A Naze deployment serves *both* formats from the same URL:

- **For legacy crawlers and old browsers**: an HTML shell containing metadata — title, description, structured data, Open Graph tags, maybe a basic text representation of the content. Think of it as a "card" that describes the Naze app, not the app itself.
- **For the new browser/runtime**: the Naze binary, which is what actually gets loaded and rendered.

This is similar to how SPAs today serve a server-rendered HTML shell for SEO while the "real" app loads via JavaScript. But cleaner — the HTML meta-index is explicitly a compatibility layer, not the application.

**How detection might work:**

```html
<!-- The HTML meta-index served to legacy crawlers -->
<html>
<head>
  <title>My Dashboard</title>
  <meta name="description" content="Real-time metrics dashboard">
  <link rel="alternate" type="application/naze" href="/app.naze">
  <!-- structured data, Open Graph, etc. -->
</head>
<body>
  <noscript>This site is optimized for Naze-compatible browsers.</noscript>
  <!-- optional: basic HTML fallback content -->
</body>
</html>
```

The new browser sees the `application/naze` link and loads that instead. Old browsers render whatever HTML is there. Crawlers index the metadata.

**The endgame:** eventually, search engines and indexing systems learn to crawl Naze natively. The HTML meta-index becomes unnecessary and fades away, just like many transitional web technologies before it. But it provides the bridge that makes adoption possible without sacrificing discoverability from day one.

---

## Dual-Branch Architecture & The New Browser (Phase 3+ Optimization)

Since Naze apps can run inside existing browsers via WASM, the "new browser" isn't a prerequisite — it's a later optimization. But it's still worth designing for, because a dedicated runtime would eliminate the overhead of the full browser engine sitting underneath.

**Phase 1-2: Naze inside existing browsers.**

Naze apps compile to WASM and render via Canvas/WebGL, running inside Chrome/Firefox/Safari today. The HTML meta-index page loads the WASM binary, similar to how SPAs load JavaScript bundles now. Existing browsers become the delivery mechanism — no new software to install.

**Phase 3+: The dedicated Naze browser.**

Once the ecosystem has traction, a dedicated browser makes sense as an optimization. It drops the HTML rendering engine (Blink/Gecko/WebKit) and runs Naze natively — lower memory, faster startup, no DOM overhead whatsoever.

**When the new browser navigates to a URL:**

1. Fetch the resource
2. Check the response: does it include Naze content? (via content-type header, `<link rel="alternate" type="application/naze">`, or a Naze manifest)
3. **If Naze is available**: load and render the Naze binary using the native runtime. Fast, efficient, full experience.
4. **If only HTML**: fall back to an embedded HTML rendering engine (could embed a lightweight webview). The site still works, just rendered the old way.

This means:
- **Naze-first sites** get the full performance and capability benefits
- **Legacy HTML sites** still work in the new browser — nobody is forced to migrate
- **Hybrid sites** (HTML meta-index + Naze binary) get the best of both: discoverable by legacy systems, performant in the new browser

**What the dedicated browser IS (when it eventually arrives):**
- A WASM runtime with native GPU rendering — no DOM engine overhead
- A small set of native UI primitives (text, images, inputs, scrolling, animation)
- A networking stack (HTTP, WebSocket, etc.)
- A security sandbox (see Hard Problems below)
- An embedded HTML fallback for legacy compatibility during transition

**But the critical point: you don't need this to start.** Everything works in today's browsers. The dedicated browser is an optimization for when the ecosystem is mature enough to justify it.

---

## Backwards Compatibility & Transition Path

The web has trillions of pages. You can't just throw them away. But you also can't let backwards compatibility prevent progress forever (which is arguably what's happening now).

**For existing sites:**

Mechanical transpilation (HTML → Naze) is probably not the right approach. HTML is so riddled with implicit behaviors, browser-specific quirks, and layout edge cases that a 1:1 translation would carry all the old problems into the new format.

Instead: **AI re-interpretation**. An AI looks at an existing website — its visual layout, its interactions, its data flows — and *regenerates* it as a Naze application. Not a translation, but a re-creation of intent. This is feasible today with multimodal AI that can "see" a website and understand what it's supposed to do.

**For new sites:**

Author in Naze-first. Include the HTML meta-index for compatibility. As the Naze ecosystem matures, the HTML layer becomes increasingly vestigial.

**Phased transition:**

1. **Phase 1 — The language**: design the AI-native declarative language and build the compiler (language → WASM). Build the rendering library (WASM-based UI primitives rendering to Canvas/WebGL). Apps run inside existing browsers — no new browser needed.
2. **Phase 2 — Ecosystem growth**: developer tools, AI agents, and hosting platforms target Naze. The HTML meta-index bridges discoverability for search engines. Early adopters ship Naze apps that run in Chrome/Firefox/Safari today.
3. **Phase 3 — The dedicated browser**: once there's enough Naze content to justify it, a lightweight dedicated browser/runtime emerges. Drops the DOM engine overhead, runs Naze natively. Distributed as a downloadable app (like Chrome was in 2008), with embedded HTML fallback for legacy sites.
4. **Phase 4 — Tipping point**: enough sites are Naze-native that the dedicated browser becomes the primary way people access the web. Legacy HTML becomes like legacy IE sites — still out there, but increasingly a niche.

---

## Prior Art & Landscape Research

Research conducted February 2026, re-validated February 2026 (pre-MVP check). The specific intersection we're describing — **AI-native language + declarative UI + WASM compilation + Canvas/WebGL rendering (no DOM)** — does not exist as a single project. The gap has been confirmed twice. But the pieces are all out there, and several projects are close on different axes.

### The Foundational Vision Document

**Ian Hickson — "Towards a Modern Web Stack" (January 2023)**
The former spec editor of HTML5 and Flutter tech lead at Google wrote a [proposal](https://docs.google.com/document/d/1peUSMsvFGvqD5yKh3GprskLC3KVdAlLGOsK6gFoEOD0/edit) for enabling browsers to render pages served as WASM files instead of HTML. The stack: WASM + WebGPU (rendering) + ARIA (accessibility) + WebHID (input). Explicitly cites Flutter as a proof of concept. This is the closest articulation of the overall rendering vision from a highly authoritative source. It does not address AI-native language design. No formal standardization effort has emerged from it.

### DOM-Bypassing Frameworks (the rendering layer exists)

Several production and near-production frameworks already bypass the DOM entirely, rendering via WASM + Canvas/WebGL:

| Framework | Language | Rendering | Status | Notes |
|-----------|----------|-----------|--------|-------|
| **Flutter Web** | Dart | Skia → WebGL (WASM) | Production | Google deprecated the HTML renderer entirely. ~1.5MB Skia engine download. Most mature DOM-bypass approach. |
| **Figma** | C++ | Custom engine → WebGPU | Production (proprietary) | Proof that WASM + GPU rendering works at scale. Not a reusable framework. |
| **Makepad** | Rust | Custom shaders → WebGL | 1.0 (May 2025) | Has its own declarative DSL. All GPU rendering. WASM binaries ~hundreds of KB. Closest to "new language + WASM + no DOM." |
| **Compose Multiplatform** | Kotlin | Skia → Canvas (WasmGC) | Beta (Sep 2025) | JetBrains. Same Skia-on-canvas approach as Flutter. ~3x faster than JS. Stable expected late 2026. |
| **Uno Platform** | C#/XAML | Skia → WebGL | Production | Unique: offers both DOM and Skia modes. 90M+ NuGet downloads. |
| **egui** | Rust | WebGL (immediate mode) | Active | No DOM at all. ~500KB WASM. Not declarative. Good for tools. |
| **GPUI (Zed)** | Rust | Metal/Vulkan/DX12 (GPU) | Pre-1.0, desktop only | Built for Zed editor. 120 FPS. Custom shaders per UI primitive. Proves GPU UI can match game-engine perf. |
| **Xilem + Vello** | Rust | WebGPU via wgpu | Alpha | Linebender project. GPU compute-centric 2D renderer. Most direct path to WebGPU-based UI in browser. |
| **Ribir** | Rust | WebGPU/WebGL | Alpha (v0.4.0) | Data-driven, compile-time view update generation. Non-intrusive declarative API. |
| **Slint** | Own `.slint` DSL | WebGL (FemtoVG) | Production | **Closest overall to Naze (3/4 properties)** — has its own declarative language + WASM + canvas rendering. But the `.slint` language was designed for human developers, not AI generation. |

**Key takeaway:** The rendering layer is a solved problem with multiple implementations. Flutter and Makepad prove you can ship real apps this way today. Slint is the closest to the full Naze vision — it has its own language, WASM compilation, and canvas rendering — but the language isn't AI-native.

### DOM-Targeting WASM Frameworks (for comparison)

These use WASM for logic but still render to the DOM — they don't bypass HTML/CSS:

| Framework | Language | Notes |
|-----------|----------|-------|
| **Leptos** | Rust | Fine-grained reactive DOM. Near-vanilla-JS performance. |
| **Dioxus** | Rust | Virtual DOM. Cross-platform (web/desktop/mobile). |
| **Yew** | Rust | Virtual DOM. React-like. Mature for Rust WASM. |
| **Blazor** | C# | .NET in WASM. DOM diffing via JS interop. Production. |

### AI-Native Languages (the language layer is emerging)

A few projects are exploring languages designed for AI to write:

**NERD — "The World's First LLM-Native Language"** ([nerd-lang.org](https://www.nerd-lang.org/))
Uses plain English words for token efficiency (54% fewer tokens than JS). Compiles to LLVM IR. No WASM target, no web UI. Very early/experimental.

**Synapse v1.0** ([github.com/Xzdes/synapse](https://github.com/Xzdes/synapse))
S-expression syntax designed to eliminate LLM ambiguity. Uses Abstract Semantic Graph instead of AST. **Has a WebAssembly backend.** No UI layer — general-purpose language. 16,500 lines of Rust. Early but functional.

**MoonBit** ([moonbitlang.com](https://www.moonbitlang.com/))
Statically-typed language designed specifically for WASM as a first-class target. Created by the creator of ReScript. Targets wasm-gc. Not AI-native, but WASM-first. Post-beta, active development.

**GlyphLang** ([Show HN](https://news.ycombinator.com/item?id=46571166))
AI-first language that replaces verbose keywords with symbols optimized for LLM tokenization. Claims ~35% fewer tokens than Python. Has bytecode compiler, JIT, LSP, PostgreSQL integration. No UI rendering, no WASM — compiles to its own bytecode/VM.

**Universalis** ([ACM Queue, 2025](https://queue.acm.org/detail.cfm?id=3746223))
AI-first program-synthesis language designed for knowledge workers to read and LLMs to execute. Minimal syntax (sequential composition, implicit looping, dataframe queries). Formal pre/post-conditions for AI safety. Named after Leibniz's "characteristica universalis." No UI, no WASM — focused on data processing.

**"What Language Should LLMs Program In?"** ([Dev Interrupted, August 2025](https://devinterrupted.substack.com/p/what-language-should-llms-program))
Widely-read essay arguing existing languages are poorly suited for LLM code generation, and purpose-built languages with formal guarantees are inevitable.

### AI App Generators (still targeting HTML/CSS/JS)

The leading AI code generation tools all target the standard web stack:
- **v0.dev** (Vercel): generates React + Tailwind CSS + shadcn/ui
- **bolt.new** (StackBlitz): generates React + Node.js full-stack apps
- **val.town**: explicitly moved *away* from custom syntax back to standard JavaScript/TypeScript — lesson learned: "don't mess with language standards"

None use a custom language or bypass the DOM. They generate the same HTML/CSS/JS stack, just faster.

### Google A2UI Protocol (Dec 2025)

**A2UI** ([github.com/google/A2UI](https://github.com/google/A2UI)) is an open protocol for AI agents to describe UIs in declarative JSON. Framework-agnostic — renders to native widgets (Flutter, React, Angular). This is a *protocol* for agent-driven interfaces, not a language. It doesn't compile to WASM, doesn't render to canvas, and maps to existing frameworks rather than replacing them. Interesting adjacent concept but solves a different problem.

### GenUI (Privoce)

**GenUI** ([github.com/Privoce/GenUI](https://github.com/Privoce/GenUI)) is a Vue-inspired declarative Rust framework built on top of Makepad. Inherits Makepad's WebGL rendering and WASM compilation. Plans to add AI-generated UI in the future, but the AI features are not yet implemented. Currently uses a human-designed DSL. The closest any project has come to bridging the "AI language" and "canvas/WASM rendering" clusters — but not there yet.

### AI-Generated UI Research (academic)

Several 2025 papers argue that AI needs a semantic intermediate representation for UI, not raw code generation:

- **"Bridging Gulfs in UI Generation"** ([arXiv, Jan 2025](https://arxiv.org/html/2601.19171)) — Proposes a four-level semantic framework (Product → Design System → Feature → Component). Argues direct HTML generation loses the "meaning layer."
- **SpecifyUI / SPEC** ([arXiv, Sep 2025](https://arxiv.org/html/2509.07334v1)) — Introduces SPEC as a "shared language for human-AI collaboration" — a structured intermediate semantic layer between natural language and code. Directly supports the Naze language concept.
- **Generative Interfaces (GenUI)** ([arXiv, Oct 2025](https://arxiv.org/html/2508.19227v2)) — Surveys graph-based, grammar-based, and schema-driven approaches for AI-native UI representation.

### Post-HTML Proposals and Community Sentiment

- **Handmade Network** has been quietly working on a pure-WASM web platform proposal — pages served as WASM binaries with sandboxed graphics/network APIs. Active grassroots effort.
- **W3C/WICG proposals** for HTML alternatives have been met with hostility: "HCJ is used on over a trillion web pages" and replacements "have absolutely no chance."
- **Google Docs migrated from DOM to Canvas** for performance, building a hidden "side DOM" for accessibility. Demonstrates the pattern but also the accessibility cost.
- **WASM 3.0** (September 2025) added 64-bit memory, GC, and multiple memories. Still no direct DOM access — and the standards community considers this "good enough."
- **Servo** (Rust browser engine, rebooted 2023) is a new implementation of existing web standards, not a clean-slate departure. Its modular Rust crates could be reused as components though.
- **W3C Accessibility Object Model (AOM)** is in development — could eventually provide a DOM-free accessibility API, which would be critical for post-DOM rendering approaches.

### The Gap: What Nobody Has Built

| Capability | Exists? | Best Example |
|------------|---------|--------------|
| AI-native language for code generation | Early stage | Synapse, NERD, GlyphLang, Universalis |
| Declarative UI DSL | Mature | Flutter, Compose, Slint, Makepad |
| WASM compilation of UI code | Mature | Flutter, Compose, Makepad, egui, Ribir |
| Canvas/WebGL rendering (no DOM) | Mature | Flutter, Makepad, egui, Compose, Slint |
| **All four combined** | **No** | **Nothing exists** |

The landscape splits into two non-overlapping clusters:
1. **Canvas/WASM UI frameworks** (Slint, Flutter, Compose, Makepad, Xilem, Ribir) — have properties 2, 3, and 4 but use human-oriented languages
2. **AI-native languages** (GlyphLang, Universalis, Synapse, NERD) — have property 1 but zero UI rendering capability

Nobody is bridging them. GenUI (Privoce) is the closest attempt — it builds on Makepad's rendering and plans AI features — but the AI language layer doesn't exist yet. Naze would be the first project to unify all four properties.

### Criticisms and Counterarguments

The "Flash 2.0" criticism comes up repeatedly in community discussions:
- **Accessibility**: Screen readers, keyboard nav, and ARIA must be rebuilt from scratch. Every DOM-bypass project (Flutter, Google Docs) builds a parallel "side DOM" for accessibility. This is the hardest unsolved problem.
- **Browser integration loss**: No Ctrl+F, no native text selection, no browser extensions, no password managers, no "view source."
- **Initial load**: WASM apps must download and initialize the rendering engine before any content appears — unlike HTML's progressive rendering.
- **"The browser IS a GPU renderer"**: Some argue browsers already use GPU compositing internally, and rolling your own means losing decades of optimizations for scrolling, text input, IME, etc.

These are real. But they're also the same arguments that were made against native mobile apps in 2008 — and apps won anyway for the use cases where they were materially better.

---

## What We'd Have to Build

Based on the research, the pieces exist but nobody has unified them. Here's what's net-new vs. what we can build on.

### What already exists (don't reinvent these)

- **WASM runtime** — every major browser ships one. This is our execution layer.
- **GPU rendering pipeline** — WebGL is universal, WebGPU is rolling out (Chrome, Edge, Safari 26, Firefox 141). These are our rendering targets.
- **Text shaping and i18n** — HarfBuzz (shaping), FreeType (rasterization), ICU (internationalization). Battle-tested C/C++ libraries, all compilable to WASM.
- **HTTP/networking** — browsers provide fetch, WebSocket, etc. No need to build a network stack.
- **Security sandbox** — WASM's memory isolation and capability-based security model. Already designed and proven.
- **2D GPU renderers** — Vello (Rust, WebGPU-native), Skia (C++, what Flutter/Compose use), Makepad's shader pipeline. These can be built on or learned from.

### What we'd have to build

**1. The Language** *(net-new — the core innovation)*

The AI-native declarative UI language. This is the centerpiece — everything else serves it.

Design decisions:
- **Grammar and syntax** — optimized for LLM token efficiency and unambiguous parsing. Fewer tokens = faster AI generation, lower cost, fewer errors. Synapse's approach (S-expressions for zero ambiguity) and NERD's approach (English words for token efficiency) are reference points, but neither targets UI.
- **Type system** — static types with inference. The compiler should catch layout errors, type mismatches, and missing data bindings at compile time, not at runtime in the user's browser.
- **Primitives** — layout (grid, stack, scroll, responsive), data binding (reactive connections to data sources), interaction (click, drag, keyboard, gesture), animation (transitions, spring physics), and accessibility semantics (role, label, live region) all as first-class language constructs, not library features.
- **Module system** — composable components that can be shared, versioned, and imported. A Naze equivalent of npm packages, but designed for binary distribution.

**2. The Compiler** *(net-new, but built on existing infrastructure)*

Transforms the language into optimized WASM binaries.

- Could use **Cranelift** (Rust-based, already used by Wasmtime) or target **wasm-tools** for WASM-specific optimization passes
- Must produce **small binaries** — the whole point is that a Naze app should be kilobytes, not megabytes. Dead code elimination, tree-shaking, and binary size optimization are critical.
- **Error messages** must be excellent — both for human developers and for AI feedback loops (so the LLM can self-correct)
- Should support **incremental compilation** for fast development iteration

**3. The Rendering Library** *(build on existing — the biggest integration effort)*

A WASM library that provides UI primitives and renders to Canvas/WebGL/WebGPU. This is the runtime that Naze binaries link against.

- **Layout engine** — flexbox-like layout, but designed cleanly without CSS's decades of edge cases. Handles responsive breakpoints, scrolling, text flow.
- **Text rendering** — integrates HarfBuzz + FreeType + ICU for full international text support. This is complex but well-understood; the libraries exist.
- **GPU rendering** — could build on Vello (Rust, WebGPU), use Skia (proven by Flutter/Compose), or build a custom pipeline like Makepad does. Trade-off: Skia is proven but large (~1.5MB); a custom pipeline could be much smaller.
- **Input handling** — translates browser events (mouse, touch, keyboard) into Naze's interaction primitives
- **Animation system** — declarative animations that the runtime executes at 60fps without application code involvement

**4. The Accessibility Bridge** *(net-new — the hardest piece)*

Every project that bypasses the DOM faces this: screen readers and assistive technologies talk to the browser's accessibility tree, which is built from the DOM. No DOM = no accessibility, unless you build a bridge.

- **Short-term approach**: generate a hidden "side DOM" (like Flutter and Google Docs do) — a minimal, invisible DOM that mirrors the Naze semantic tree for screen readers. Ugly but works today.
- **Long-term approach**: target the W3C **Accessibility Object Model (AOM)** directly, which would let WASM code expose an accessibility tree without a DOM. AOM is still in development but would be the clean solution.
- **Language-level support**: the Naze language should make accessibility declarations mandatory or strongly encouraged — role, label, and keyboard behavior on every interactive element. Compile-time warnings for missing accessibility metadata.

**5. The HTML Meta-Index Generator** *(small, net-new)*

A build tool or compiler plugin that automatically generates the HTML metadata shell alongside the Naze binary.

- Extracts title, description, and content structure from Naze source
- Generates `<meta>` tags, Open Graph, JSON-LD structured data
- Produces the `<link rel="alternate" type="application/naze">` reference
- Optionally generates a basic HTML text fallback for legacy browsers
- Should be automatic — the developer writes Naze, the toolchain produces both the binary and the meta-index

**6. Developer Tooling** *(net-new)*

Nobody adopts a language without good tools.

- **Language Server Protocol (LSP)** implementation — autocomplete, go-to-definition, inline errors in VS Code / any editor
- **Inspector / debugger** — like browser DevTools but for Naze primitives. Visual tree, layout bounds, data binding state, performance profiling
- **Hot reload** — change source, see results instantly without full recompile. Critical for development speed.
- **Binary size analyzer** — visualize what's contributing to WASM binary size, identify optimization opportunities
- **Playground** — browser-based editor where you can write Naze and see it render live (like the Rust Playground or TypeScript Playground)

**7. AI Integration Layer** *(net-new — what makes this "AI-native" rather than just "another language")*

The language is designed for AI to write, so the AI-to-language interface matters as much as the language itself.

- **Prompt templates and few-shot examples** — curated examples that teach LLMs the language efficiently
- **Constrained generation** — using structured output / grammar-constrained decoding so the LLM can only produce syntactically valid Naze code (per MIT's Sequential Monte Carlo research showing this dramatically improves correctness)
- **Validation feedback loop** — the compiler returns structured error messages that the LLM can parse and self-correct, not just human-readable text
- **Fine-tuning dataset** — a corpus of Naze examples across common UI patterns (dashboards, forms, e-commerce, content sites) for model training
- **Intent-to-Naze pipeline** — a higher-level interface where a user describes what they want in natural language, and an AI agent generates, compiles, and iterates on Naze code until it matches intent

### Build order (suggested)

```
Phase 1: Language + Compiler + Minimal Renderer
         → "Hello world" Naze app running in Chrome via WASM + Canvas

Phase 2: Full Rendering Library + Accessibility Bridge
         → Real apps with text, layout, interaction, screen reader support

Phase 3: Developer Tooling + AI Integration
         → LSP, debugger, hot reload, LLM prompt engineering

Phase 4: Meta-Index Generator + Ecosystem
         → SEO story, package registry, community adoption
```

---

## Beyond the Web: Universal UI Platform

An important realization: nothing about this approach is web-only. The same language, compiler, and rendering pipeline could target mobile and desktop natively — not as a web app wrapped in a webview (Electron/Capacitor), but as actual native rendering on each platform.

**This is already how cross-platform frameworks work today:**
- Flutter: Dart → Skia → iOS / Android / Web / macOS / Windows / Linux
- Compose Multiplatform: Kotlin → Skia → Android / iOS / Web / Desktop
- Makepad: Rust → GPU shaders → macOS / Windows / Linux / Web
- Slint: own DSL → GPU rendering → embedded devices / desktop / web

The rendering library already needs to talk to GPU APIs. On the web, that's WebGL/WebGPU. But the same abstraction layer talks to:
- **Metal** on iOS/macOS
- **Vulkan** on Android/Linux
- **DirectX 12** on Windows

These are different backends for the same rendering primitives. Libraries like **wgpu** (used by Vello and Xilem) already abstract across all of these.

**What this means for Naze:**

The compiler could have multiple targets:
```
Naze source → WASM binary        (web, via browser)
Naze source → native ARM binary  (iOS, Android)
Naze source → native x86 binary  (macOS, Windows, Linux)
```

The rendering library would have platform-specific backends but a single API:
```
Rendering library
  ├── WebGL/WebGPU backend  (browsers)
  ├── Metal backend         (Apple platforms)
  ├── Vulkan backend        (Android, Linux)
  └── DirectX 12 backend    (Windows)
```

**This changes the pitch entirely.** It goes from "a new web language" to "a universal AI-native UI language." One language, one codebase, every platform. AI generates a Naze app and it runs on the web, on phones, and on desktops — all natively rendered, no webview wrapper, no platform-specific code.

**The app store angle comes full circle.** Remember the app store analogy from earlier? With Naze targeting native platforms, you could literally ship to app stores. A Naze app compiled to native ARM runs on iOS as a real native app — not a web clip, not a PWA, not a React Native bridge. The same source deploys to the web (via WASM) and to app stores (via native compilation).

**WASI extends this further.** WebAssembly System Interface (WASI) is making WASM a general-purpose runtime beyond browsers. Naze components could run on:
- Edge servers (Cloudflare Workers, Fastly) for server-side rendering
- IoT / embedded devices
- Desktop applications without a browser at all

**Updated build implications:**
- The rendering library's architecture becomes more important — it needs a clean GPU abstraction layer from the start, not a web-only renderer that gets ported later
- The compiler needs pluggable backends: WASM for web, LLVM/Cranelift for native
- Platform-specific concerns (iOS safe areas, Android back button, desktop window management) need to be handled as platform adapters, not baked into the language

---

## AI-Native Prompting & Embedded Intelligence

Two related ideas that push the AI-native concept further — from "AI writes Naze code" to "AI lives inside Naze apps."

### Built-In AI Prompt Compatibility

What if the Naze language had native syntax for AI prompts? Not "call an LLM API" as an afterthought, but prompting as a first-class language primitive — the same way `data` binds to sources and `on click` binds to events.

```
-- components/smart-search.naze

component smart-search(context: text) {

  input query, placeholder: "Ask anything..."

  prompt result: from ai {
    system: "You are a helpful assistant for {context}"
    user: query
    model: default
    stream: true
  }

  if result.loading { spinner }
  else { markdown result.text }
}
```

The `prompt` keyword declares an AI interaction the same way `data` declares a data fetch. The runtime handles model selection, streaming, error states, and caching. The component doesn't know which LLM is backing it — that's configured at the app level (similar to how `sources.naze` abstracts data connections):

```
-- ai.naze (or [ai] section in naze.toml)

ai default: openai {
  model: env.AI_MODEL
  key: env.OPENAI_KEY
  temperature: 0.7
}

ai local: ollama {
  endpoint: env.OLLAMA_URL
  model: "llama3"
}

ai vision: openai {
  model: "gpt-4-vision"
  key: env.OPENAI_KEY
}
```

Components reference AI providers by name, just like data sources:

```
data summary: from ai.local {
  prompt: "Summarize: {article.text}"
  max-tokens: 200
}

data caption: from ai.vision {
  prompt: "Describe this image"
  image: uploaded-file
}
```

**Why this matters:** every AI-powered app today stitches together LLM API calls with custom glue code — retry logic, streaming parsers, prompt templates, model fallback chains. If Naze makes prompting a language primitive, the compiler can optimize it (batch requests, cache identical prompts, validate prompt templates at compile time), and AI agents generating Naze apps can use AI *inside* the apps they generate. It's AI building AI-powered apps.

### Naze as an Embedded LLM — The App That Learns

A more speculative idea that reframes how users interact with apps entirely.

**The key distinction: the LLM is not the runtime — it's the authoring layer.**

The Naze runtime stays deterministic. WASM executes, the renderer draws pixels, the layout engine positions boxes — none of that changes. What changes is *who authors the `.naze` files and when*. Instead of a developer writing all the code upfront and deploying a static app, an embedded LLM generates and modifies `.naze` source *on the fly*, which then compiles to WASM and renders through the normal pipeline.

The user's primary interaction is with the LLM. The LLM's output is `.naze` files. The app is whatever the LLM just generated.

```
The architecture:

  User (natural language)
    │
    ▼
  Embedded LLM (understands the app's .naze source)
    │
    ▼
  Generates / modifies .naze files
    │
    ▼
  nazec compile → WASM → Runtime → Renderer → Pixels
    (unchanged — same deterministic pipeline as any Naze app)
```

**What this looks like in practice:**

```
-- Hypothetical: an app with an embedded AI authoring layer

ai assistant: local
  learn-from: "./**/*.naze"        -- all project files
  learn-from: "sources.naze"       -- data schema
  learn-from: "tests/**/*.naze"    -- test expectations
  update: on-file-change           -- re-index when files change
  model-size: small                -- runs on CPU, no GPU needed
  can-modify: "pages/**/*.naze"    -- allowed to generate/edit these files
  can-modify: "components/**/*.naze"
  read-only: "sources.naze", "theme.naze"  -- can read but not change
```

A user interacts with the app and says: "add a date filter to the metrics table." The LLM:

1. Reads the existing `pages/dashboard.naze` (it already has it indexed)
2. Understands the component structure, data bindings, layout
3. Generates a modified `dashboard.naze` with a date picker component wired to the metrics query
4. The modified `.naze` file compiles to WASM through the normal pipeline
5. The app re-renders with the new date filter — live, in front of the user

The user never sees `.naze` source. They see the running app. They talk to it. It changes.

**Why this is different from today's AI coding assistants:**

Today's workflow: AI reads your codebase → sends to remote GPU → generates code → you review it → you accept/reject → you rebuild → you test. The AI has no persistent understanding — it rebuilds context every query.

The embedded LLM approach:
- **Persistent context** — the model maintains an embedding index of all `.naze` files, updated incrementally as files change. It doesn't re-read from scratch each time.
- **The output IS the app** — the LLM doesn't suggest code for a developer to review. It generates `.naze` files that compile and render immediately. The user interacts with the result, not the source.
- **Constrained generation** — the LLM can only output valid `.naze` syntax (grammar-constrained decoding from C7). The compiler catches any errors before rendering. The runtime is always deterministic.
- **Scoped permissions** — `can-modify` and `read-only` declarations control what the LLM is allowed to change. It can't modify `sources.naze` (breaking your database connections) or `theme.naze` (breaking your brand) unless explicitly permitted.

**The bigger picture — the app that builds itself:**

Today, apps are inert artifacts. A developer builds them, deploys them, and users interact with the fixed result. An app with an embedded LLM that outputs `.naze` source is something different — it's an app that can:

- **Self-extend** — user says "I need a report page" and the LLM generates `pages/report.naze` using the app's existing patterns, components, and data sources
- **Self-document** — "how does this app work?" → the LLM, having indexed all source files, explains the architecture from actual code, not generic documentation
- **Self-debug** — when an error occurs, the LLM has full context to generate a fix (a modified `.naze` file), compile it, and verify the tests pass before applying
- **Adapt to users** — observe which features are used, generate shortcuts or simplified views, create personalized layouts — all as `.naze` modifications compiled to WASM

**The runtime stays dumb. The intelligence is in the authoring layer.** WASM executes deterministically. The renderer draws exactly what the compiled code says. The LLM is "just" a very sophisticated code generator — but one that lives inside the app, understands the app, and responds to users in real time.

**Open questions:**
- How do you handle hallucination? The LLM might generate `.naze` that compiles but is wrong (shows incorrect data, breaks a workflow). Compiler validation catches syntax errors, but semantic correctness is harder.
- How fast does the compile cycle need to be for this to feel interactive? If a user says "add a filter" and it takes 5 seconds to generate + compile + render, is that acceptable?
- Permission boundaries: what happens when the LLM generates a component that accesses a data source it shouldn't? The `can-modify` / `read-only` system needs to be robust.
- Versioning: if the LLM modifies files, how do you track what changed and roll back? Git integration (auto-commit each LLM modification)?
- Is this a development-time tool (AI helps developers build), a runtime feature (AI helps end-users customize), or both? The architecture supports either — the difference is just who's talking to the LLM.

### AI Ecosystem & The Small Model Advantage

Naze is fully free and open-source — compiler, runtime, renderer, toolchain, language spec. No paid tiers, no gated features. The platform is open.

**Why Naze-specialized models can be dramatically smaller than general-purpose coding models:**

Current AI coding tools (Copilot, Claude Code, Cursor) use large models (70B-400B+ parameters) trained on dozens of languages, hundreds of frameworks, and millions of patterns. They need that size because the problem space is enormous — HTML + CSS + JavaScript + TypeScript + React + Vue + Svelte + Tailwind + Redux + hundreds of other combinations.

A Naze-specialized model has a fraction of that problem space:

- **One language, not 50** — only `.naze` syntax. No CSS, no JavaScript, no framework choices.
- **Constrained grammar** — designed to be unambiguous. One way to express layout, data binding, events. Smaller search space for valid outputs.
- **Declarative, not imperative** — describes "what," not "how." Far fewer ways to express the same thing.
- **No framework fragmentation** — no React vs Vue, no Tailwind vs CSS modules, no Redux vs Zustand. One language, one way.
- **Predictable patterns** — components, slots, data bindings, themes all follow consistent structures.

Result: a fine-tuned 3-7B parameter model on `.naze` could plausibly match or outperform a 70B general-purpose model at Naze generation specifically. This changes the economics of AI-assisted development entirely.

**Three tiers of AI-assisted Naze development:**

1. **Existing AI dev tools (works today, no Naze-specific integration needed)** — Claude Code, Cursor, Copilot, Windsurf, etc. already work with any text-based language. Developers use them to write/edit `.naze` files the same way they write TypeScript or Python today. General-purpose models, general-purpose tools. No special integration needed — the AI reads `.naze` source and generates `.naze` source. This is the baseline.

2. **Local Naze-specialized models (the sweet spot)** — small fine-tuned models (3-7B) running locally via Ollama or llama.cpp. CPU-only, no cloud, no cost, fully private. Because the problem space is so constrained, these small models could do remarkable things — potentially matching the quality of much larger general-purpose models for Naze-specific tasks. This is where Naze's language design pays off: by constraining the language, you constrain the model requirements.

3. **Cloud AI services (for complex/novel tasks)** — third-party companies could offer Naze-optimized models as a service for tasks that exceed local model capability (generating entire multi-page apps from scratch, complex data binding logic, novel UI patterns). This is a business opportunity *for them*, not for Naze. Symbiotic: better AI services drive Naze adoption; Naze adoption grows the market for Naze-specialized AI services.

**The local-first advantage:** Unlike today's AI coding landscape where cloud is king and local is a compromise, Naze's constrained design could make local-first the *better* experience for most tasks. The cloud becomes a power-user option, not a requirement. This has major implications:

- **Developing world adoption** — no API costs, works offline, runs on modest hardware
- **Privacy** — code never leaves the machine
- **Speed** — no network round-trip, instant generation
- **Cost** — zero marginal cost per generation

Think of it this way: training a model to be good at one constrained language with predictable patterns is a fundamentally easier problem than training a model to be good at the entire web development ecosystem. Naze's design isn't just good for compilers and runtimes — it's good for AI model efficiency. The constraint *is* the feature.

**Parallels:**
- Like GitHub Copilot vs. open-source alternatives (Codeium, local models). The platform is open; AI services compete on quality.
- Like Vercel/Netlify for hosting — the framework (Next.js) is free; the hosting/DX service is the business.
- Like how Python's ecosystem spawned Anaconda — the language is free; services built on it are businesses.

**Key principle:** Naze doesn't own or gate the AI layer. The `ai.naze` config is provider-agnostic. Naze grows the pie; others build businesses on slices of it.

### Creating the Naze Model: What It Takes

Research (February 2026) shows that creating a fine-tuned Naze model is surprisingly feasible — one engineer, 2-3 weeks, $100-300 total.

**The two-layer approach:**

You don't start with fine-tuning. You start with grammar-constrained decoding, which gives you syntactically valid output for free, with zero training:

1. **Layer 1: Grammar-constrained decoding (GCD)** — write the Naze grammar in GBNF format, feed it to llama.cpp or XGrammar, and any code-capable base model (Qwen2.5-Coder-7B, etc.) will only output tokens that form valid `.naze` syntax. 100% syntax validity guaranteed by construction. The model has never seen Naze, but it can only produce valid Naze. Semantic quality is moderate — the code is valid but may not do what you intended.

2. **Layer 2: Fine-tuning** — QLoRA fine-tune the same model on Naze examples. This doesn't replace GCD — it runs on top. Fine-tuning improves *semantic quality* (the code does what you mean), while GCD continues to guarantee *syntactic validity*. Combined: the model produces code that's both valid and meaningful.

**Bootstrapping from zero — the data pipeline:**

Naze doesn't exist yet, so there's no training corpus. Here's the proven approach:

```
Week 1: Seed + Expand
  Day 1-2:  Write Naze grammar in GBNF format
  Day 2-3:  Test GCD with a base model → baseline quality assessment
  Day 3-5:  Hand-write 100-200 seed .naze examples covering full grammar
  Day 5-7:  Use Claude/GPT-4 to expand seeds to 5,000 examples
            Filter every generated example through the Naze parser
            (this is the secret weapon — automatic quality assurance)

Week 2: First Fine-Tune
  Day 8-9:   QLoRA fine-tune Qwen2.5-Coder-3B on the 5K dataset
  Day 10-11: Evaluate: parse rate, semantic correctness, vs GCD-only baseline
  Day 12-14: Use the fine-tuned model to generate more examples
             Filter with parser → expand to 10K-20K examples

Week 3: Iterate
  Day 15-17: Fine-tune on the larger dataset, with GCD as safety layer
  Day 18-21: Evaluate, debug failure modes, add targeted examples
             → Production candidate
```

**The Apple UICoder precedent — proof this works for UI languages:**

Apple Research (2024) fine-tuned a model on SwiftUI starting from almost zero Swift training data (1 Swift example in 10,000 training samples). Their approach:

- Self-improvement loop: generate SwiftUI → compile → GPT-4V evaluates visual output → filter → retrain
- 5 rounds produced ~996,000 SwiftUI programs
- Compilation rate: **3% → 82%** (matching GPT-4's 81%)
- The researchers explicitly stated this generalizes to "other toolchains with similar properties (e.g., Dart/Flutter, React Native)"

This is directly applicable to Naze. Naze has an even stronger advantage: a simpler, more constrained grammar than SwiftUI, plus a purpose-built parser for filtering.

**Concrete costs:**

| Resource | Amount |
|----------|--------|
| Hand-written seed examples | 100-200 |
| Synthetic examples (generated + parser-filtered) | 5,000-20,000 |
| API costs (frontier model for data generation) | $50-200 |
| GPU costs (QLoRA training on cloud RTX 4090) | $20-100 |
| **Total cash outlay** | **$100-300** |
| Human time | 2-3 weeks (one engineer) |

**Hardware for training:**

| Setup | VRAM | Training time (10K examples) | Cost |
|-------|------|------------------------------|------|
| RTX 4090 (owned) | 24 GB | 2-8 hours | Electricity only |
| Cloud RTX 4090 | 24 GB | 2-8 hours | $5-20 |
| Cloud A100 | 80 GB | 1-4 hours | $10-40 |
| Google Colab T4 (free) | 16 GB | 4-12 hours | Free |

QLoRA trains only ~4-20M parameters out of 7B total (0.06-0.3%), retaining 80-95% of full fine-tuning quality at a fraction of the cost.

**Quality at each stage:**

| Stage | Syntax validity | Semantic quality | Useful for |
|-------|----------------|-----------------|------------|
| GCD only (no training) | 100% | Moderate — valid but often incoherent | Autocomplete, suggestions with human review |
| First fine-tune (5K examples) | 100% (GCD layer) | 60-80% correct for simple constructs | Boilerplate generation, simple components |
| Iterated fine-tune (20K examples) | 100% (GCD layer) | 80-90%+ for common patterns | Primary development tool |

**Why Naze's parser is the secret weapon:**

The entire pipeline depends on one thing: can you automatically verify if generated code is correct? For most languages, this is hard — code can be syntactically valid but fail at runtime in countless ways. But Naze's compiler (C2) can verify:

- Syntax validity (parsing)
- Type correctness (type checker)
- Component interface conformance (prop types, required slots)
- Theme token references (do the tokens exist?)
- Data source references (does the named source exist?)
- Accessibility completeness (are roles and labels present?)

Every generated example gets run through the compiler. Invalid examples are discarded automatically. This execution-verified filtering is the proven method for producing high-quality synthetic training data — and Naze's compile-time checks give you more filtering power than most languages offer.

**Recommended base models (as of early 2026):**

- **3B tier:** Qwen2.5-Coder-3B, Phi-3.5-mini (3.8B)
- **7B tier:** Qwen2.5-Coder-7B, DeepSeek-Coder-V2-Lite, Mistral-7B
- **Tools:** Unsloth (2-5x faster QLoRA), LLaMA Factory (web UI for config), Axolotl (YAML-based)

**Key research references:**
- Apple UICoder: self-improvement loop for SwiftUI generation (VL/HCC 2024)
- Magicoder: 75K synthetic examples → 7B model surpassed ChatGPT on code benchmarks (ICML 2024)
- Prem-1B-SQL: 1.3B model beat Claude 2 on text-to-SQL with 122M tokens of training data
- Grammar-constrained decoding: XGrammar (2024), llama.cpp GBNF, Guidance (Microsoft)
- "Let Me Speak Freely?" (EMNLP 2024): GCD distorts distributions but combined with fine-tuning the effect is mitigated

---

## Hard Problems & Open Questions

These are real challenges that don't have obvious answers yet.

**Accessibility.** HTML's semantic structure (headings, landmarks, ARIA roles) is how screen readers and assistive technologies work. Naze needs accessibility as a first-class primitive from day one — not bolted on after the fact like it was with the web. The native UI primitives in the runtime should carry semantic meaning that assistive tools can consume. This could actually be *better* than HTML accessibility if designed right, since the semantics would be explicit rather than inferred from markup patterns.

**Hyperlinking and composition.** The web's superpower is the hyperlink — any page can reference any other. And iframes/embeds allow composition (embedding one site inside another). Naze needs equivalents. Deep linking, cross-app composition, and URL-based navigation should be foundational, not afterthoughts.

**View source / transparency.** One of the web's cultural values is that you can inspect any site — right-click, view source, see how it works. A binary format feels opaque. There should be a decompilation/inspection story — the Naze runtime could include a built-in inspector that shows the structural representation, similar to browser DevTools but for Naze's primitives instead of DOM elements.

**Security and sandboxing.** The web's security model (same-origin policy, CSP, sandboxed execution) is battle-tested. A new runtime needs an equally rigorous security model from the start, or nobody will trust it. WASM already has a strong sandboxing story, which is another argument for building on it.

**Governance.** Who owns the Naze spec? If it's one company, it becomes a walled garden (like Flash was). It needs to be an open standard with multi-stakeholder governance. But open standards processes are slow, and this idea needs to move fast to catch the AI wave.

**Fonts, text rendering, and i18n.** Browsers handle an enormous amount of complexity around text: bidirectional text, complex scripts (Arabic, Devanagari, CJK), font shaping, ligatures, hyphenation. The Naze runtime needs to handle all of this. Likely by using existing libraries (HarfBuzz, FreeType, ICU) rather than reinventing them.

---

## Who Benefits

**AI agents browsing the web.** Agents operating on behalf of users don't need pixels — they need structured data and actions. Naze's intent-based format could expose a machine-readable interface alongside the visual one, making AI interaction far more efficient than scraping HTML.

**AI-generated applications.** If an AI builds you a custom UI on the fly (an "app for one"), there's zero reason it should generate HTML/CSS/JS. It could target Naze directly — faster to generate, faster to load, fewer things to go wrong.

**Developing world.** The bloated web disproportionately hurts users on low-end devices and slow connections. A 50KB Naze binary instead of a 5MB JS bundle is a meaningful difference when you're on a 2G connection with a $50 phone.

**Developers.** The toolchain simplification alone would be transformative. No more debugging Webpack configs, fighting CSS specificity, or choosing between 15 state management libraries. The complexity budget goes toward the actual problem being solved.

---

## Token Efficiency — Why Naze's Syntax Is Optimal for AI

A core claim of Naze is that it's the most efficient format for AI to generate and humans to read. This was validated with a concrete benchmark (Feb 2026): the same dashboard UI expressed across 12 formats.

**The benchmark UI:** sidebar with 3 nav links, main area with heading, 3-column responsive grid of metric cards (title, numeric value, sparkline trend), click-to-navigate to detail page, polling data source.

```
Format                     Tokens   vs Naze   Notes
─────────────────────────────────────────────────────────────────────
Terse/golfed DSL               63    0.49x    Unreadable, high ambiguity
TOON-adapted                  100    0.78x    Designed for flat data, not UI trees
Natural lang + annotations    123    0.96x    Cannot be deterministically compiled
Naze                          128    1.00x    ◄ Pareto-optimal point
S-expressions                 133    1.04x    Readability cost for non-Lispers
YAML-based                    167    1.30x    Key repetition overhead
JSON-based                    302    2.36x    ~40% tokens are structural punctuation
Slint                         491    3.84x    Can't express all requirements natively
SwiftUI                       590    4.61x    Type definitions + lifecycle overhead
React/JSX + CSS               604    4.72x    Framework ceremony + separate styling
Svelte                        653    5.10x    Template + script + style separation
Flutter/Dart                  912    7.12x    Widget nesting + class boilerplate
```

**What "Pareto-optimal" means here:** Naze is the most token-efficient format that is simultaneously (a) human-readable by non-developers, (b) unambiguous (parseable by a deterministic compiler), and (c) deterministically compilable (no LLM needed at runtime). The three formats that beat Naze on raw tokens each sacrifice one of these properties.

**Key research supporting the design:**

| Finding | Source | Implication for Naze |
|---------|--------|---------------------|
| Custom formats beat JSON despite zero training data (39.6% fewer tokens, 4.2% higher accuracy) | TOON benchmarks, Nov 2025 | Training data volume doesn't determine format effectiveness |
| Grammar-constrained decoding eliminates 96-100% of syntax errors | SynCode 2024, IterGen ICLR 2025 | Naze's formal grammar enables near-perfect syntax generation |
| Compiler-in-the-loop feedback overcomes training data gaps (3% → 82% compilation for SwiftUI) | Apple UICoder, VL/HCC 2024 | `nazec` compiler serves as automatic quality filter |
| Format constraints help structural tasks, hurt reasoning tasks | "Let Me Speak Freely?", EMNLP 2024 | UI description is structural — constraints help, not hurt |
| LL(1) grammars are the sweet spot for constrained decoding | GRAMMAR-LLM, ACL Findings 2025 | Naze's grammar should target LL(1) complexity |
| LLMs struggle with indentation-based syntax | LLM code understanding study, Apr 2025 | Naze uses braces `{}`, not significant whitespace |
| DSLs suffer ~51% initial accuracy drop vs general-purpose languages | ACM TOSEM survey, 2025 | Mitigated by grammar constraints + synthetic training |
| 60-70% of tokens in mainstream frameworks are boilerplate | Token benchmark analysis | Naze eliminates this entirely |

**Identified optimizations (not yet implemented):**
- Tabular syntax for repeated structures (nav lists, table columns) — save ~15-20 tokens
- Implicit item prefix in `each` blocks — save 4-8 tokens per iteration template
- These could bring Naze from 128 to ~105-110 tokens

---

## Discussion History

This document evolved from an initial brainstorming session. Key inflection points:

1. **Starting observation**: the web stack is decades of patches on a fundamentally dated paradigm. As AI becomes the primary code author, we're still targeting a format designed for human hand-coding in the 1990s.

2. **App store parallel**: native apps proved people will adopt a parallel ecosystem when it's meaningfully better. Naze could offer native-like performance with the web's openness.

3. **WASM as foundation**: rather than inventing everything from scratch, WebAssembly provides a credible, already-standardized runtime foundation. The shift is from "WASM as a guest inside the browser" to "WASM as the browser's core."

4. **The discoverability problem and meta-index solution**: without search engine visibility, adoption stalls. The HTML meta-index acts as a backwards-compatible bridge — a lightweight HTML shell with metadata that crawlers can index, while the new browser loads the Naze binary. This is transitional; eventually Naze-native indexing replaces it.

5. **Dual-branch detection**: the new browser checks every URL for Naze availability and routes accordingly. Legacy HTML still works. This means zero disruption for existing sites while enabling incremental adoption of the new paradigm.

6. **Landscape research (Feb 2026)**: confirmed that no project combines all four pieces (AI-native language + declarative UI + WASM compilation + no-DOM rendering). The rendering layer is mature (Flutter, Makepad, Compose). AI-native languages are emerging (Synapse, NERD). Academic research supports the need for a semantic intermediate representation. The gap is the unification. Ian Hickson (HTML5 spec editor) independently proposed a similar rendering vision in 2023 but didn't address the language layer.

7. **Concrete build plan**: broke down the project into 7 deliverables (language, compiler, rendering library, accessibility bridge, meta-index generator, dev tooling, AI integration layer) and identified what already exists vs. what's net-new. Proposed a 4-phase build order starting with language + compiler + minimal renderer.

8. **Beyond the web — universal UI platform**: realized that nothing about the approach is web-only. The same language + compiler + rendering library could target iOS, Android, macOS, Windows, and Linux natively (not webview wrappers). This is already how Flutter, Compose, and Makepad work. The pitch expands from "new web language" to "universal AI-native UI language." One source, every platform. Apps can ship to app stores as native binaries and to the web as WASM — from the same codebase.

9. **Three language design pillars**: (a) Human-readable syntax — purpose-built format, not markdown or any existing markup language. A non-developer should be able to read a `.naze` file and understand what the app does. Not markdown because we'd inevitably create extensions for layout/interaction/data that defeat the clean-slate purpose. (b) Layout model: named slots built on spatial primitives — high-level templates (`app-shell`, `dashboard`) with named regions, backed by low-level grid/row/column/stack primitives. Use a preset or define your own. (c) Reusable components — one component per `.naze` file, typed props, content slots for composition, event emission, accessibility metadata as first-class.

10. **Toolchain & package system**: single binary CLI (`nazec`) — no Node.js, no npm, no `node_modules`, no `package.json`. Own manifest format (`naze.toml`). Packages are source-distributed — `.naze` files, not compiled blobs. Inspectable, forkable, tree-shakeable. Local build cache avoids recompilation. Registry-agnostic (git URLs initially, dedicated registry later).

11. **Styling & theming**: no CSS. `theme.naze` defines design tokens (colors, fonts, spacing, radii, shadows). Components reference tokens (`theme.colors.primary`). Themes can extend other themes (`dark extends my-app` overrides just colors). Compiler warns on raw values where tokens exist.

12. **Data sources**: components don't know connection details. `sources.naze` defines named sources (REST, GraphQL, database, WebSocket, static). Components say `data users: from api "/users"`. Credentials in environment variables. A component library works regardless of what backend the consuming app uses.

13. **Testing built into the language**: tests are `.naze` files — same syntax as the app. Component tests (render + assert) and flow tests (multi-page journeys). `nazec test` runs everything. Tests as readiness constraints: "app is ready when all tests pass." AI generates tests alongside app code. No Playwright, no Jest, no separate test framework.

14. **AI prompt as a language primitive**: if AI is going to write Naze apps, those apps will increasingly *contain* AI. Rather than treating LLM API calls as external glue code, make `prompt` a first-class keyword — same as `data` for data sources. Components declare AI interactions; the runtime handles model selection, streaming, caching. An `ai.naze` config (parallel to `sources.naze`) maps named AI providers to credentials. The compiler can validate prompt templates at compile time. AI builds AI-powered apps.

15. **The LLM as the authoring layer, not the runtime**: key clarification — the embedded LLM doesn't replace the Naze runtime. The runtime stays deterministic (WASM executes, renderer draws pixels). The LLM is the *interaction and authoring layer* — users talk to it, it generates/modifies `.naze` files, those compile to WASM through the normal pipeline. The user interacts with the LLM; the LLM's output is the app. This enables self-extending apps (user says "add a date filter" → LLM generates modified `.naze` → compiles → renders), self-documenting (LLM explains architecture from indexed source), and self-debugging (LLM generates fixes from error context). Scoped permissions (`can-modify`, `read-only`) control what the LLM is allowed to change. The runtime stays dumb; the intelligence is in the authoring layer.

16. **The small model advantage — local-first AI**: because Naze is one language with a constrained grammar (vs. the entire HTML/CSS/JS/framework ecosystem), a fine-tuned 3-7B model could match general-purpose 70B+ models at Naze generation specifically. This flips the AI story: local models via Ollama aren't a compromise — they could be the *primary* development mode. Cloud AI services become the power-user option, not the default. Three ecosystem tiers: existing dev tools (Claude Code, Cursor) work today with no special integration; local Naze-specialized models are the sweet spot; cloud services handle complex/novel generation. Naze is fully free/open-source; AI services are ecosystem opportunities for third parties, not a Naze-owned monetization point. The constraint is the feature — by constraining the language, you constrain the model requirements.

17. **Creating the Naze model is cheap and fast**: research confirmed that bootstrapping a fine-tuned Naze model costs $100-300 total, takes 2-3 weeks, and requires one engineer. Two-layer approach: grammar-constrained decoding (GBNF) gives 100% syntax validity for free with zero training; QLoRA fine-tuning on synthetic data adds semantic quality. Pipeline: 200 hand-written seeds → frontier model expands to 20K examples → filter through Naze compiler → fine-tune 3-7B model on consumer GPU. Apple's UICoder proved this works for UI languages: took SwiftUI compilation from 3% to 82% (matching GPT-4) starting from near-zero Swift training data. Naze's compiler is the secret weapon — it serves as an automatic quality filter for training data at every stage. The model can begin training as soon as the Phase 1 compiler parses `.naze` files.

18. **The key realization — it's the language, not the browser**: since WASM already runs in every major browser, and projects like Flutter/web already bypass the DOM via WASM + Canvas, the runtime infrastructure already exists. The missing piece isn't a new browser — it's the AI-native declarative language that compiles to WASM. This dramatically simplifies the scope: design a language, build a compiler, build a rendering library. Apps can ship inside existing browsers today. The "new browser" becomes a Phase 3+ optimization, not a prerequisite.

19. **Server-side rendering & deployment — Naze's unique advantage**: Canvas-based WASM frameworks (Flutter, Makepad, Compose) cannot do SSR — they render pixels, not HTML. Naze's declarative design means the compiler *can* emit both WASM (for client canvas rendering) and HTML (for server-side first paint + SEO). No other canvas-based framework can do this. Three rendering modes (SSG, SSR, client-only) configurable per-route. Server functions via the `server` keyword handle server-side compute — database queries, auth, heavy calculations — with auto-generated type-safe RPC stubs. The compiled server is a single binary (Rust): WASM target for edge/serverless (Cloudflare Workers, Fermyon Spin) or native binary for containers. No Node.js, no JVM, no runtime dependencies. The first-paint problem is solved: SSR/SSG sends HTML immediately, WASM canvas takes over when ready. Deploys to every major platform (Vercel, Cloudflare, AWS, Netlify, Fastly, Docker) with zero configuration changes.

20. **Input handling without the DOM — a solved problem**: when Naze renders everything to canvas, there are no DOM elements to click or type into. This is a solved problem — Flutter, Makepad, Figma, and Google Docs canvas mode all handle it the same way. Four mechanisms: (1) Hit testing — the C4 Layout Engine already computes positioned rectangles for every element; clicking at (x, y) is just a tree walk asking "which rectangle contains this point?" (2) Text cursor/selection — the C4a Text Engine (HarfBuzz) already computes exact glyph positions; cursor placement is "which glyph boundary is nearest to x?" (3) Focus management — C3 Runtime tracks a `focusedElement` internally, computes tab order from the layout tree. (4) IME (the hard part) — a hidden `<input>` element behind the canvas, same trick Flutter and Figma use. Naze's advantage: the C6 Accessibility Bridge already maintains a hidden "side DOM" for screen readers — the hidden input element lives there. One hidden DOM serves both accessibility and IME with no additional infrastructure.

21. **Browser integration — copy/paste, Ctrl+F, extensions, screen sizes**: the honest trade-offs of canvas rendering. Copy/paste works via the browser Clipboard API (C3 Runtime reads selected text, writes to clipboard). Ctrl+F is replaced by a built-in find overlay that searches Naze's layout tree (same approach as Google Docs canvas mode). Password managers work because C6's hidden input elements carry `autocomplete` and `type` attributes. Ad blockers work (network-level). Screen readers work (C6 side DOM). Dark mode extensions don't work on canvas, but Naze has native theming. Translation extensions don't work directly, but text is exposable via the side DOM. DOM-manipulating extensions are a genuine loss — same trade-off as Flutter and Figma. Responsive layout is already a language primitive. DPI/retina is handled by rendering the canvas backing buffer at `devicePixelRatio` scale. Resize/orientation changes trigger C4 re-layout. Most browser extension use cases are covered by Naze's language-level features (theming, i18n, accessibility).

22. **Rendering performance, resize, and animations (continued in 23)**: Naze's rendering pipeline is dramatically simpler than the DOM pipeline. Traditional web: parse HTML → parse CSS → cascade/specificity resolution → render tree → layout → paint → composite (6 stages). Naze: layout engine → GPU draw (2 stages). Steps 1-3 are skipped entirely — no HTML parsing, no CSS cascade, no specificity, no render tree. Additional savings: no virtual DOM diffing, no style recalculation cascades, no reflow chains. Honest trade-off: browsers have decades of optimization (incremental layout, layer compositing); Naze needs to build these over time. But Flutter and Figma already prove 60fps canvas rendering works. Screen sizes are NOT pre-determined — the layout engine handles any arbitrary dimensions in a single top-down pass. Animations are a first-class language feature: property animations (`animate opacity from 0 to 1 over 300ms`), transitions (auto-animate on value change), spring physics, and keyframes. GPU-optimized: transform/opacity animations skip re-layout entirely (update GPU uniforms directly), same optimization browsers use for CSS `transform`. All compiled into the WASM binary — no CSS animation runtime, no JavaScript animation libraries.

23. **Rendering optimizations and the performance verdict**: Naze uses a four-level optimization hierarchy borrowed from game engines: (1) skip unchanged subtrees entirely (zero cost), (2) layer compositing — separate the UI into GPU texture layers so only dirty layers repaint (navigate to a new page → only the content layer repaints, toolbar/background/overlays reuse cached textures), (3) dirty rectangle tracking — within a dirty layer, clip repaint to just the changed bounding boxes, (4) texture caching — render expensive components to GPU textures and reuse until data changes. The compiler assigns layers automatically at compile time from the component tree structure, avoiding the over/under-compositing heuristic problems browsers face. The performance verdict: DOM overhead scales superlinearly with complexity (cascade, reflow, style recalculation compound), while Naze's scales linearly. For simple pages the difference is negligible; for complex interactive apps, Naze is structurally faster — same reason native apps outperform web apps at scale.

24. **Computation model — three tiers, not a general-purpose language (continued in 25)**: Naze deliberately stays declarative and doesn't add `while` loops, mutation, or imperative control flow. Instead, computation lives in three tiers: (1) Built-in declarative logic — pipeline operators (`users | filter(active) | sort-by(.name)`), pure functions, pattern matching, list comprehensions, local `let` bindings. Covers ~80% of client-side logic. (2) WASM library imports — like Java's JNI but with zero overhead. The `import` keyword brings in pre-compiled WASM modules (Rust, C, Go). The compiler merges them into a single binary via `wasm-merge` + `wasm-opt`, so imported functions become normal intra-module calls after compilation — no bridge, no FFI boundary. The Rust/WASM ecosystem already has production libraries for JSON, crypto, regex, dates, compression, etc. (3) Server functions — already documented, for database access, secrets, and heavy compute. Naze follows the SQL pattern: declarative for its domain, with escape hatches to full computation when needed. The constraint is the feature — it keeps the language AI-optimizable, compile-time verifiable, and readable by non-developers.

26. **Naming decision — WUI → Naze (Feb 2026)**: the original working name "WUI" (Web UI) conflicted with numerous existing projects, frameworks, and search results using the "Web UI" name. After evaluating ~20 candidates (Hue, Dawn, Koda, Glyph, Pxl, Lux, and others — most had taken domains, existing projects, or conflicting file extensions), chose **Naze** for several reasons: (1) the `.naze` file extension is completely unused — no conflicts anywhere, (2) "Naze" is already the AI assistant in the Illuminaze productivity app (illuminaze.com), creating a dual identity: Naze the language and Naze the AI assistant, (3) Illuminaze's brand concept — "Intelligence Amplification" — aligns perfectly with an AI-native language, (4) the language becomes a product under the Illuminaze umbrella, and Illuminaze itself could eventually be built with Naze (dogfooding the technology). CLI tool: `nazec`. Config files: `naze.toml`, `theme.naze`, `sources.naze`, `ai.naze`.

27. **Token efficiency validation — Naze is Pareto-optimal for AI↔human UI (Feb 2026)**: benchmarked the same dashboard UI (sidebar with 3 nav links, heading, 3-column responsive grid of metric cards with title/value/sparkline, click-to-navigate) across 12 formats using cl100k_base tokenizer. Results: Naze at 128 tokens vs React/JSX 604 (4.7x), Flutter/Dart 912 (7.1x), Svelte 653 (5.1x), SwiftUI 590 (4.6x), Slint 491 (3.8x), JSON 302 (2.4x), YAML 167 (1.3x), S-expressions 133 (1.04x). Only three formats beat Naze: terse/golfed DSL (63 tokens — unreadable, high ambiguity), TOON-adapted (100 tokens — designed for flat data not nested UI trees), and natural language with annotations (123 tokens — cannot be deterministically compiled). Naze sits at the Pareto-optimal point: the most token-efficient format that is simultaneously human-readable, unambiguous, and deterministically compilable. Supporting research: TOON benchmarks (Nov 2025) proved custom formats beat JSON despite zero training data (39.6% fewer tokens, 4.2% higher accuracy). Grammar-constrained decoding eliminates 96-100% of syntax errors (SynCode 2024, IterGen ICLR 2025). Apple UICoder proved compiler-in-the-loop feedback overcomes the training data gap (3% → 82% compilation rate for SwiftUI). The EMNLP 2024 "Let Me Speak Freely?" paper showed format constraints help structural tasks (UI description qualifies) while hurting reasoning tasks. LL(1) grammars are the sweet spot for constrained decoding (ACL Findings 2025). Two optimizations identified: (a) tabular syntax for repeated structures (save ~15-20 tokens on lists), (b) implicit item prefix in `each` blocks (save 4-8 tokens) — could bring Naze to ~105-110 tokens.

28. **Syntax decision — braces over indentation (Feb 2026)**: multiple research sources flagged a concern with Naze's original indentation-based syntax. The B-IR language author noted "indentation trips up LLMs — having whitespace be syntactically meaningful means it is very important for a specific number of tokens to be emitted, and LLMs are not very good at counting." An April 2025 LLM code understanding study found "LLMs struggle with code branching logic in Python" compared to Java's explicit braces. Grammar-constrained decoding can mitigate this (SynCode achieves 96% for Python), but explicit delimiters are inherently safer. Decision: **Naze adopts brace-delimited blocks (`{}`) instead of significant indentation.** Indentation remains conventional for readability but is not syntactically meaningful. This adds a small token cost (~10-15 tokens for a typical component) but eliminates an entire class of LLM generation errors. The syntax remains keyword-rich and readable — braces mark structure, keywords provide meaning. This also makes Naze friendlier to developers coming from C/Java/Rust/Go/JavaScript backgrounds.

25. **Pre-MVP landscape re-validation (Feb 2026)**: re-confirmed that the four-property intersection (AI-native language + WASM compilation + canvas/no-DOM rendering + typed declarative components) still does not exist. The landscape splits into two non-overlapping clusters: (1) Canvas/WASM UI frameworks (Slint, Flutter, Compose, Makepad, Xilem, Ribir) — have WASM + canvas + typed components but use human-oriented languages. Slint is the closest at 3/4 properties — it has its own `.slint` DSL + WASM + canvas rendering, but the language isn't AI-native. (2) AI-native languages (GlyphLang, Universalis, Synapse, NERD) — designed for AI to generate but have zero UI rendering. New entrants since prior research: GlyphLang (symbol-based, LLM token-optimized), Universalis (ACM paper, formal AI-synthesis language), GenUI/Privoce (Vue-like on Makepad, plans AI features but not yet built), Google A2UI protocol (declarative JSON for agent UIs, renders to existing frameworks). AI app generators (v0.dev, bolt.new, val.town) all still target standard HTML/CSS/JS — val.town explicitly abandoned custom syntax. The gap is confirmed: nobody is bridging AI-native languages with canvas/WASM rendering. Naze would be the first.

---

*This is a living brainstorm. Open questions, counterarguments, and wild ideas are welcome.*
