# AI Context Management: Maintaining σ = 1 at Scale

This document analyzes **all cross-cutting concerns** in a Naze application — not just component interfaces — and shows how each can maintain σ = 1 (single-file context) for AI agents. It identifies 13 categories of shared values, assesses their current and future σ risk, proposes solutions, and extends the Ψ cost equation to model context coverage across the full spectrum.

**Status:** Research / design exploration. No implementation decisions finalized.

**Related:** `docs/TOKEN_EFFICIENCY.md` (Ψ equation and Λ(n) framework), `docs/PARITY.md` (feature-level analysis), `docs/PROTOTYPE.md` (architecture spec).

---

## 1. The Problem: σ Divergence in Multi-File Apps

Naze's Token Efficiency framework claims **σ = 1** — the AI needs only the current file. This holds for single-file apps where state, UI, data, and event handling are co-located. But production applications inevitably use components:

```naze
use components/card
use components/button
use @naze/ui-kit/modal

app "Dashboard" {
  card title: "Revenue" {      -- What params does card accept?
    button label: "Details" {   -- What events does button emit?
      on clicked: set show-modal = true
    }
  }
  modal title: "Revenue Details", show: show-modal {
    fill "body" {               -- What slots does modal expose?
      text "Details here"
    }
  }
}
```

To generate or modify this file correctly, the AI must know:
- `card` accepts `title: text`, `bg: color = #ffffff`, `width: number = 200px`, and has a default slot
- `button` accepts `label: text` and emits `clicked`
- `modal` accepts `title: text`, `show: bool`, exposes slots `body` and `footer`, and emits `close`

That information lives in 3 other files. σ = 4 for this interaction.

But component interfaces are only **one of 13 cross-cutting concerns** that can push σ above 1. Section 2.5 catalogs all of them.

### The File Size Crossover

Single-file apps avoid σ > 1 but face a different problem: file size degrades AI attention quality.

Naze averages ~3–5 tokens per line. The practical thresholds:

| App Size | Tokens | AI Behavior | Components? |
|----------|--------|-------------|-------------|
| 50–200 lines | 200–800 | Full comprehension | Not needed |
| 200–500 lines | 800–2K | Comfortable | Optional (extract if reused) |
| 500–1000 lines | 2–4K | Attention starts softening | Recommended |
| 1000–2000 lines | 4–8K | Details missed in middle | Strongly recommended |
| 2000+ lines | 8K+ | Errors from attention gaps | Essential |

**The sweet spot for component extraction** is when any of:
- A visual pattern appears 2+ times
- A section exceeds ~100–150 lines
- A piece has its own state logic (form widget, modal, chart)

**The sweet spot for "no components needed"** is ~300–500 lines (~1.5–2K tokens). Below this, the σ = 1 advantage of single-file outweighs the verbosity cost.

### Why Every Framework Hits This Wall

This is not a Naze-specific problem. Every UI framework faces the same tension:

| Framework | σ = 1 approach | σ > 1 reality |
|-----------|---------------|---------------|
| React | Single-file components | Props types in separate files, shared state in stores, CSS modules |
| Vue | SFCs co-locate template+script+style | Pinia stores, composables, type imports |
| Svelte | SFCs with minimal boilerplate | Stores, shared types, layout files |
| **Naze** | Everything inline, compile-time inlining | Component params/slots/emits in separate files |

Naze has a structural advantage: component interfaces are tiny (typed params + slots + emit), there are no lifecycle hooks, no context/provider chains, and no side effects to trace. But σ still exceeds 1 when components are split across files.

---

## 2. What Naze Already Knows (The Untapped Goldmine)

The compiler already computes everything an AI agent would need. The gap is purely in **exposure** — the data exists but isn't accessible outside the compilation pipeline.

### 2.1 `ResolvedProject` / `ComponentDef`

During `resolve()` (in `crates/naze-compiler/src/resolve.rs`), the compiler builds:

```rust
pub struct ComponentDef {
    pub import_path: String,    // "components/card"
    pub name: String,           // "card"
    pub params: Vec<Param>,     // [{name: "bg", ty: Color, default: Some(#ffffff)}, ...]
    pub children: Vec<Node>,    // Full AST body (includes slot declarations)
    pub span: Span,
    pub file: PathBuf,
}

pub struct ResolvedProject {
    pub entry: SourceFile,
    pub components: HashMap<String, ComponentDef>,  // ALL components
    pub themes: Vec<Theme>,
    pub imports: Vec<ResolvedImport>,
    pub errors: Vec<CompileError>,
}
```

This is the complete interface map for every component in the project. The AI needs ~10 tokens per component to understand its interface — but currently cannot access this data at all.

### 2.2 The LSP Gap

The LSP (`crates/naze-lsp/`) is a stub. It advertises completions, hover, go-to-definition, and references — but all implementations are:
- **Parse-level only** — never calls `resolve()` or `typecheck()`
- **Single-file only** — no cross-file awareness
- **Static completions** — hardcoded built-in element names, not project components

The LSP source explicitly notes:
```rust
// Note: Full type-checking requires project resolution (resolving imports,
// loading components, etc.). For now, we just validate parsing.
// TODO: Add lightweight single-file validation for common errors.
```

An AI agent using the LSP gets zero component interface information.

### 2.3 The `nazec ai` Gap

The `ai generate` command validates generated code against a `ResolvedProject` with **empty components**:

```rust
// From ai.rs validate_source():
let resolved = ResolvedProject {
    components: HashMap::new(),  // No components!
    themes: vec![],
    imports: vec![],
    // ...
};
```

Generated code that references imported components produces "unknown element" warnings (not errors) — the AI gets no signal about whether it used component interfaces correctly.

### 2.4 The Grammar Export Gap

`nazec grammar --format gbnf` exports the PEG grammar for constrained decoding. This defines valid syntax but has no project awareness. An LLM with GBNF constraints can generate syntactically valid `.naze` files, but cannot be guided to:
- Only use component names that exist in the project
- Pass the correct params with correct types
- Fill only the slots that the component declares

### 2.5 The Full Spectrum of Shared Values

Component interfaces (Section 2.1) are one slice of the problem. A Naze application has **13 categories of cross-cutting information** that an AI agent might need. Each can independently push σ above 1.

The following table catalogs every concern, its current σ score, future risk, and the mechanism that keeps it low (or the gap that threatens to raise it).

#### 1. Design Tokens — Colors and Spacing (σ = 1)

**Current state:** Theme definitions (`theme { colors { ... } spacing { ... } }`) are declared inline in `.naze` files. References use `theme.colors.*` and `theme.spacing.*` prefixes. The grammar supports two sections: `colors` and `spacing` (`theme_section_name = { "colors" | "spacing" }`).

**Why σ = 1:** The prefix `theme.colors.primary` encodes enough information that the AI does not need to read the theme definition. It knows the value is a color. If the AI needs the exact hex value, the theme block is in the same file (or in an imported theme file that `nazec context` can bundle).

**Future risk:** Low. The mechanism works.

#### 2. Design Tokens — Untyped (σ = 1.2)

**Current state:** Radii, font sizes, shadows, layer ordering (z-index), and opacity have **no theme namespace**. Developers hardcode these values throughout the file. Two components that should share `radius: 8px` have no common reference.

**Why σ > 1 eventually:** As apps grow, a designer changes "all border radii from 8px to 12px." Without a `theme.radii.*` namespace, the AI must search the entire codebase. Even in a single file, it must guess which `8px` values are border radii vs. padding.

**Proposed solution:** Expand `theme_section_name` in the grammar:

```
theme_section_name = { "colors" | "spacing" | "radii" | "fonts" | "shadows" | "layers" | "opacity" }
```

This is ~5 new terminals in an existing rule — no new grammar rules needed. References would use `theme.radii.md`, `theme.fonts.body`, `theme.shadows.card`, `theme.layers.overlay`, `theme.opacity.disabled`.

**Impact on σ:** Restores σ = 1 for visual consistency concerns.

#### 3. Project Constants (σ ≈ 1.2)

**Current state:** Naze supports `let` bindings at the top of files for named constants (`let max-items = 50`). These are file-scoped. There is no project-wide constant mechanism.

**Why σ > 1 eventually:** Multiple files that need the same constant (API base URL, max upload size, app name) must each define their own `let` — or the constant is hardcoded in multiple places.

**Proposed solution:** A `[constants]` section in `naze.toml` that becomes accessible via a `project.*` namespace:

```toml
[constants]
max-items = 50
api-version = "v2"
```

Referenced as `project.max-items` in `.naze` files. Compile-time substitution (like theme tokens).

**Impact on σ:** Restores σ = 1. The `project.*` prefix tells the AI "this comes from the manifest" — no need to read it.

#### 4. Component Interfaces (σ > 1)

**Current state:** This is the primary subject of Sections 1–2 above. When a file uses `use components/card`, the AI must read `components/card.naze` to learn the params, slots, and emits.

**Why σ > 1:** Component interfaces live in separate files. σ = 1 + number_of_imported_components.

**Proposed solutions:** See Section 3. The highest-leverage fix is the MCP server + `nazec context --json` (Section 3.1), which provides all component interfaces as structured data without the AI needing to read any component files.

#### 5. Server Function Interfaces (σ = 1)

**Current state:** Server functions are declared at the top level of `.naze` files with explicit signatures:

```naze
server function list-users() { ... }
server function add-user(name: text, email: text) { ... }
```

The calling code uses `data users: list-users()` in the same file.

**Why σ = 1 today:** Server functions must be in the entry file (not inside app blocks). `collect_declarations()` doesn't collect them from components. So the definition and the call site are always in the same file.

**Future risk:** Medium. If Naze adds multi-file server function modules (e.g., `use server/users`), σ would increase. Any such feature must include the function signature in the import context.

**For `nazec context`:** Include server function signatures in the context bundle — even though they're currently same-file, this prepares for the multi-file case and helps AI agents that receive only a partial file.

#### 6. Data Source Schemas (σ = 1)

**Current state:** Data sources are declared inline:

```naze
data posts: fetch "https://api.example.com/posts"
data users: list-users()
```

The AI knows `posts.data`, `posts.loading`, `posts.error` are available. The shape of the data itself (e.g., `post.title`, `post.body`) is inferred from usage.

**Why σ = 1:** The declaration and usage are co-located. No external schema file.

**Future risk:** Low. The convention (`*.data`, `*.loading`, `*.error`) is self-documenting.

**For `nazec context`:** Include data source names and types (fetch/websocket/sse/js/device/server-fn) for completeness.

#### 7. Asset References (σ > 1)

**Current state:** Image paths reference filesystem locations (`image src: "assets/logo.png"`). The AI cannot verify that the file exists without reading the filesystem.

**Why σ > 1:** The AI must check the filesystem to know which assets are available — technically a second "file" to read (the directory listing).

**Future risk:** Low priority. Incorrect asset paths produce runtime 404s, not compile errors.

**Mitigation:** Convention: `assets/` directory. `nazec context` could list available assets.

#### 8. i18n / Translations (not supported)

**Current state:** No internationalization support. All text is inline strings.

**Why it matters:** If i18n is added via external translation files (`translations/en.json`), every text reference would require reading the translation file. σ would jump to 2+ for any file with user-facing text.

**Design constraint:** When i18n is added, it must use a `t.*` namespace (e.g., `t.welcome_message`) with compile-time resolution. The prefix tells the AI "this is a translation key" — the exact string value is rarely needed for code generation. `nazec context` should include the available translation keys.

#### 9. Environment Variables (σ = 1)

**Current state:** The `[env]` section in `naze.toml` declares environment variables with defaults:

```toml
[env]
API_URL = "http://localhost:3000"
SECRET = { from = "SECRET_KEY", required = true }
```

These are resolved at build/serve time via `manifest::resolve_env_vars()` (priority: process env > `.env` file > manifest default).

**Why σ = 1:** The manifest is the single source of truth. Server functions access env vars through the runtime, not by reading files.

**Future risk:** None. Already solved.

#### 10. Validation Rules (not formalized)

**Current state:** No formal validation system. Developers implement validation as conditional logic in event handlers (`if name == "" { ... }`).

**Why it matters:** If validation rules are formalized (e.g., `validate email: email-format`), they could either be inline (σ = 1) or in a shared rules file (σ > 1).

**Design constraint:** When validation is added, rules must be inline — either as property attributes on `input` elements or as a `validate` block within the component. Never in a separate file.

#### 11. Event Contracts — Inter-Component Communication (σ ≈ 1)

**Current state:** Components communicate via shared state mutations and `emit` events. Shared state is declared inline (`shared state logged-in = false`). Emit declarations are implicit — a component does `emit clicked` and the parent handles it with `on clicked: ...`.

**Why σ ≈ 1:** Shared state declarations and emit events are in the same file. The convention is self-documenting: `emit event-name` → `on event-name: action`.

**Future risk:** Medium. As apps grow, the number of shared state variables increases. An AI generating a new page needs to know which shared state variables exist and what they mean. Naming conventions help (`is-*` for booleans, `current-*` for selection state, `show-*` for visibility).

**Mitigation:** `nazec context` should list all shared state declarations with types and initial values. Naming conventions should be documented in language guides.

#### 12. Route / Page Structure (σ = 1)

**Current state:** Pages are declared inline in the entry file:

```naze
page "/login" { ... }
page "/dashboard" guard: auth { ... }
page "/users/:id" { ... }
```

**Why σ = 1:** All routes are in the entry file. No file-system routing. No external route configuration.

**Future risk:** Low. Naze's explicit page declarations are one of its σ = 1 advantages over frameworks with file-system routing (SvelteKit, Next.js) where the AI must understand directory conventions.

**For `nazec context`:** Include valid routes with their params and guard names. This helps AI agents generate correct `navigate "/path"` actions.

#### 13. Guard Logic (σ = 1)

**Current state:** Guards are declared at the top level:

```naze
guard auth
  check logged-in redirect "/login"
```

Pages reference guards by name: `page "/dashboard" guard: auth`.

**Why σ = 1:** Guard definitions and page references are in the same file.

**Future risk:** Low. Guards are inherently concise (a name + check conditions).

**For `nazec context`:** Include guard names and their check conditions.

---

## 2.6 The Unified Principle

Across all 13 concerns, a single principle determines whether σ stays at 1:

> **σ = 1 is maintained when the reference name encodes enough information that reading the source definition is unnecessary for code generation.**

The mechanism is **semantic prefixes**. When an AI agent sees `theme.colors.primary`, it knows:
1. This is a design token (not arbitrary state)
2. It's a color (not a spacing value)
3. Its name is "primary" (the role, not the hex value)

The AI does **not** need to know the hex value to generate correct code. The prefix `theme.colors.` encodes the category, type, and intent. This is why σ = 1 even though the exact value is defined elsewhere.

### The Complete Semantic Prefix Table

| Prefix | Category | Type Implied | AI Needs Exact Value? | σ |
|--------|----------|-------------|----------------------|---|
| `theme.colors.*` | Design tokens | color | No (role name suffices) | 1 |
| `theme.spacing.*` | Design tokens | number (px) | No (scale name suffices) | 1 |
| `theme.radii.*` | Design tokens (proposed) | number (px) | No | 1 |
| `theme.fonts.*` | Design tokens (proposed) | text | No | 1 |
| `theme.shadows.*` | Design tokens (proposed) | text | No | 1 |
| `theme.layers.*` | Design tokens (proposed) | number | No | 1 |
| `theme.opacity.*` | Design tokens (proposed) | number (0–1) | No | 1 |
| `project.*` | Project constants (proposed) | varies | Sometimes | 1 |
| `t.*` | Translations (future) | text | No (key name suffices) | 1 |
| `*.data` | Data source result | list/object | No (shape inferred from usage) | 1 |
| `*.loading` | Data source state | bool | No (always bool) | 1 |
| `*.error` | Data source state | text | No (always text) | 1 |

**Prefixes that don't exist yet but should:** `theme.radii.*`, `theme.fonts.*`, `theme.shadows.*`, `theme.layers.*`, `theme.opacity.*`, `project.*`, `t.*`.

**The pattern:** Every new cross-cutting concern should be addressable via a typed namespace prefix. If the AI must read a second file to use it correctly, the design has failed the σ = 1 test.

---

## 3. Solutions — Optimized for AI Agents

The previous version of this section organized solutions by implementation complexity (tiers 1–4). That framing optimizes for human developers who use editors, read code on screen, and type characters. AI agents work differently:

1. They **read whole files** into a context window (not scan line-by-line)
2. They **generate complete files** in one shot (not type character-by-character)
3. They **parse structured data** better than formatted text (JSON > comments)
4. They **self-correct via tool output** (run validator, read errors, fix, repeat)
5. They **call tools** through protocols like MCP (not click buttons in an IDE)

This section reorganizes solutions around the **AI agent's actual workflow loop**: acquire context → generate code → validate → iterate.

### 3.1 Step 1: Context Acquisition — How the AI Learns About the Project

The AI needs project context before generating code. Three mechanisms, each optimal at a different scale:

#### `nazec context --format json` — Structured Context Bundle

A CLI command that outputs **all 13 concerns** as structured JSON:

```bash
nazec context app.naze --format json
```

```json
{
  "file": "app.naze",
  "components": [
    {"name": "card", "params": [{"name": "bg", "type": "color", "default": "#ffffff"}], "slots": ["default"], "emits": []},
    {"name": "button", "params": [{"name": "label", "type": "text"}], "slots": [], "emits": ["clicked"]}
  ],
  "server_functions": [
    {"name": "list-users", "params": [], "returns": "list"},
    {"name": "add-user", "params": [{"name": "name", "type": "text"}, {"name": "email", "type": "text"}]}
  ],
  "shared_state": [
    {"name": "logged-in", "type": "bool", "initial": false},
    {"name": "current-user", "type": "text", "initial": ""}
  ],
  "routes": [
    {"path": "/login", "params": [], "guard": null},
    {"path": "/dashboard", "params": [], "guard": "auth"},
    {"path": "/users/:id", "params": ["id"], "guard": null}
  ],
  "guards": [
    {"name": "auth", "checks": [{"condition": "logged-in", "redirect": "/login"}]}
  ],
  "data_sources": [
    {"name": "users", "type": "server-fn", "source": "list-users()"},
    {"name": "posts", "type": "fetch", "source": "https://api.example.com/posts"}
  ],
  "theme_tokens": ["colors.primary", "colors.background", "spacing.sm", "spacing.md"],
  "assets": ["logo.png", "icon.svg"],
  "source": "-- actual file content here..."
}
```

**Why JSON, not Naze comments:** An AI agent can programmatically extract exactly the fields it needs. A `components` array is parseable; `-- card(bg: color = #ffffff)` embedded in source text requires regex parsing and is fragile.

**Why this is optimal for AI:** One tool call. Deterministic output. All 13 concerns covered. The AI doesn't need to know which files to read or how the project is organized.

**Scale limit:** At n>100 components, the JSON blob exceeds ~4K tokens of interface data. The AI's attention softens on irrelevant interfaces. This is where semantic retrieval takes over.

#### `nazec mcp-serve` — MCP Server (The Native AI Interface)

MCP (Model Context Protocol) is designed for AI tool use. Claude Code, Cursor, and other AI agents already support it. Instead of the AI calling a CLI command and parsing stdout, it calls structured tools:

```bash
nazec mcp-serve    # starts MCP server on stdio
```

Exposes tools:
- `get_context(file)` — returns the full JSON context bundle (same as CLI)
- `search(query)` — semantic search across project (see vector index below)
- `validate(file, content)` — type-checks content against the real project, returns JSON errors
- `list_components()` — all component signatures
- `list_routes()` — all page routes with guards and params
- `get_component(name)` — full interface for a specific component

**Why MCP before LSP:** LSP is designed for keystroke-level interaction — completions as you type, hover on mouseover. AI agents don't type characters. They generate entire files and need bulk context up front, then structured validation after. MCP matches this workflow exactly. LSP does not.

**Implementation:** The MCP server wraps existing infrastructure: `resolve()` for context, `typecheck()` for validation, `parse()` for syntax checking. The protocol layer is the main new work.

#### `nazec index` + Semantic Search — For Large Projects

At n>100 components, dumping all interfaces wastes the AI's attention budget. The AI needs 5 relevant interfaces, not 200. A local vector index enables semantic retrieval:

```bash
nazec index              # builds/updates index (runs automatically in dev server)
nazec search "auth"      # CLI query: returns guards, auth-related state, login routes
```

Via MCP:
```
search("components that display user data") → [card, profile-card, user-list]
search("how does authentication work?")     → [guard auth, shared state logged-in, page /login]
```

**What gets indexed:**
- Component interfaces (name + params + slots + emits)
- Server function signatures
- Shared state declarations
- Route definitions
- Guard logic
- Code patterns (snippets of how features are used in the project)
- Theme token names

**Storage options:**
- **SQLite + sqlite-vec** — lightweight, embeddable. SQLite is already used by `naze-registry` (precedent in the codebase). The `sqlite-vec` extension adds vector search.
- **BM25/TF-IDF** — for code search, keyword matching is often sufficient without neural embeddings. Simpler to implement, zero model dependency.
- **Small embedding model** — all-MiniLM-L6-v2 (23MB) for semantic search. Runs locally, no API calls.

**Incremental updates:** On file save during `nazec dev`, re-index only changed files. Cost: milliseconds per file.

**Why this matters for σ:** At scale, `nazec context` dumps everything (σ_effective = 1.0 but with attention waste). Semantic search gives the AI exactly what it needs (σ_effective = 1.0 with minimal tokens). The difference is practical: fewer irrelevant tokens in context → fewer generation errors → lower r.

### 3.2 Step 2: Code Generation — How the AI Generates Correct Code

Context acquisition (Step 1) gives the AI information. But errors still happen during generation. Two mechanisms prevent them:

#### Project-Specific GBNF Grammar

`nazec grammar --format gbnf` already exports the PEG grammar for constrained decoding. Extending it with project awareness prevents entire error classes at generation time:

```bash
nazec grammar --format gbnf --project .
```

The GBNF output includes project-specific rules:
```gbnf
component-name ::= "card" | "button" | "modal" | "nav"
card-params ::= ("bg" ":" color-value)? ("width" ":" number-value)?
route-string ::= "\"/login\"" | "\"/dashboard\"" | "\"/users/\" ident"
guard-name ::= "auth"
server-fn-name ::= "list-users" | "add-user" | "remove-user"
```

An LLM using this grammar for constrained decoding is **unable to generate** code with incorrect component names, wrong param types, invalid routes, or nonexistent server function calls. These errors never enter the validation loop — they're prevented at decode time.

**Impact on r:** This is the most direct mechanism for driving r toward 0. Constrained decoding eliminates syntactic and many semantic errors before the model finishes generating output.

**Implementation:** Generate GBNF rules from `ResolvedProject`. ~200 lines of code on top of existing grammar export.

#### Expanded Theme Namespaces

Expand `theme_section_name` in the grammar to support more design token categories:

```pest
theme_section_name = { "colors" | "spacing" | "radii" | "fonts" | "shadows" | "layers" | "opacity" }
```

**Why this is a generation-time solution:** When the AI generates `radius: theme.radii.md` instead of `radius: 8px`, it's using a semantic token that is always correct regardless of the exact value. The AI cannot generate a "wrong" radius token name the way it can generate a wrong magic number. This reduces r for visual property values.

**Grammar impact:** Zero new rules — 5 new terminals in 1 existing rule.

### 3.3 Step 3: Validation — How the AI Knows It Got It Right

After generating code, the AI must validate it. The speed and quality of this feedback loop determines iteration cost.

#### `nazec check --format json` (Already Exists)

The diagnostic system (`crates/nazec/src/diagnostic.rs`) already supports JSON output. `CompileError` serializes to:

```json
{"message": "unknown element 'crad'", "file": "app.naze", "line": 12, "column": 5, "severity": "Error"}
```

This is machine-parseable today. The AI can run `nazec check --format json`, parse the output, and fix errors programmatically.

#### Enhancement: Richer Error Payloads

The current `CompileError` struct has 5 fields: `message`, `file`, `line`, `column`, `severity`. For AI self-correction, two additions would dramatically improve iteration speed:

```rust
pub struct CompileError {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub error_code: Option<String>,       // NEW: "E001", "W012" — programmatic classification
    pub suggested_fix: Option<String>,    // NEW: machine-actionable correction
}
```

Example enhanced output:
```json
{
  "message": "unknown element 'crad'",
  "file": "app.naze",
  "line": 12,
  "column": 5,
  "severity": "Error",
  "error_code": "E101",
  "suggested_fix": "did you mean 'card'? (available components: card, button, modal)"
}
```

**Why `suggested_fix` matters for AI:** Without it, the AI reads "unknown element 'crad'" and must reason about what went wrong. With it, the AI reads "did you mean 'card'?" and applies the fix directly. This is the difference between 1 iteration and 3.

**Why `error_code` matters for AI:** Codes enable programmatic classification. The AI can distinguish "typo in component name" (E101, auto-fixable) from "incompatible type in prop" (E201, requires context review) from "accessibility warning" (W301, can ignore for now). Different error classes warrant different fix strategies.

#### `nazec validate` via MCP

The MCP server exposes validation as a tool:

```
validate(file: "app.naze", content: "...generated code...") → [errors]
```

The AI generates code, calls `validate`, gets structured errors back, fixes, and calls `validate` again — all without writing to disk. This is faster than the CLI loop (generate → write → `nazec check` → read errors → edit → repeat) because it eliminates filesystem I/O.

### 3.4 Step 4: Iteration — How the AI Fixes Errors

The final step in the loop. Two mechanisms:

#### Structured Errors + Direct Edits

With `error_code` and `suggested_fix`, the AI can apply fixes without re-reading the entire file. The error gives the exact line, column, and suggested correction. The AI uses its Edit tool at that location. For common error classes (typos, missing imports, type mismatches), this is a single-turn fix.

#### `nazec ai fix` (Prototype → Production)

`nazec ai fix` already exists as a prototype in `crates/nazec/src/ai.rs`. It takes compiler errors and attempts automatic correction. Currently it uses an empty `ResolvedProject` (no component awareness), limiting its effectiveness.

Evolving this into a reliable self-correction tool requires:
1. Project-aware validation (real `ResolvedProject`, not empty)
2. Fix strategies per error code (E101: fuzzy match component names; E201: check param types against interfaces; W301: add accessibility attributes)
3. Integration with the MCP server (fix errors returned by `validate` without an additional compilation round-trip)

### 3.5 Deprioritized Solutions (Human-First)

The following solutions from the previous analysis are deprioritized because they optimize for human workflows, not AI workflows:

**LSP (Workspace-Aware):** LSP serves completions as you type, hover info on mouseover, and go-to-definition on click. AI agents don't type, hover, or click. They read files in bulk and generate files in bulk. LSP is valuable for human developers using VS Code — it should be built eventually — but it does not materially improve AI agent workflows. The MCP server provides strictly more value for AI at lower implementation cost.

**Rich `use` Statements:** Adding interface declarations to `use` lines (`use components/card(bg: color = #ffffff)`) achieves σ = 1 at the language level. But it adds ~5–8 grammar rules, increases λ (verbosity per import), and requires sync enforcement — all to solve a problem that the MCP server and `nazec context` solve without any language changes. This trades language complexity for a benefit that tooling provides for free.

**Contract Blocks / Component Manifest:** Both add new concepts (contract syntax or TOML declarations) that duplicate information already in component source files. Duplication means sync burden. Tooling (`nazec context`) extracts this information on demand without duplication.

**`nazec flatten` / `nazec split`:** Flattening all components into one file creates true σ = 1 but at the cost of enormous files for large projects. The `split` operation is fragile (mapping edits back to multi-file). Semantic search via the vector index is strictly better: it gives the AI exactly the context it needs without the overhead of a 10K-line flat file.

---

## 4. The Ψ Equation Revisited: σ_effective Across All 13 Concerns

The current equation from `docs/TOKEN_EFFICIENCY.md`:

> **Ψ(L, n) = n × λ × σ × (1 + r) × μ**

This treats σ as binary — either the AI needs one file (σ = 1) or multiple files (σ > 1). But the 13 concerns above show a richer picture: some concerns are already σ = 1 by design, others are σ > 1 and need tooling, and others don't exist yet but could break σ = 1 if designed poorly.

### Proposed Extension: σ_effective

**σ_effective = σ_raw × (1 − coverage)**

Where:
- **σ_raw** = number of files the AI would need to read without any tooling
- **coverage** = fraction of cross-file information available without reading those files (0.0–1.0)

### Full Concern Matrix

| # | Concern | Current σ | σ Mechanism | Future Risk | Fix / Mitigation |
|---|---------|-----------|------------|-------------|-----------------|
| 1 | Design tokens (colors, spacing) | **1.0** | `theme.colors.*` / `theme.spacing.*` prefix | Low | Already solved |
| 2 | Design tokens (radii, fonts, shadows, layers, opacity) | **1.2** | Hardcoded values, no namespace | Medium | Expand `theme_section_name` grammar |
| 3 | Project constants | **1.2** | `let` bindings, file-scoped | Medium | `[constants]` in naze.toml → `project.*` prefix |
| 4 | Component interfaces | **>1** | Must read component source files | High | MCP server, `nazec context --json`, vector index |
| 5 | Server function interfaces | **1.0** | Same-file, top-level declarations | Medium (if split) | Include in `nazec context` output |
| 6 | Data source schemas | **1.0** | Inline `data` declarations | Low | Include in `nazec context` |
| 7 | Asset references | **>1** | Filesystem lookup required | Low priority | Convention: `assets/` dir; list in context |
| 8 | i18n / translations | **N/A** | Not yet supported | High (if added wrong) | Must use `t.*` prefix with compile-time resolution |
| 9 | Environment variables | **1.0** | `[env]` in naze.toml | None | Already solved |
| 10 | Validation rules | **N/A** | Not yet formalized | Medium (if added wrong) | Must be inline, never in separate files |
| 11 | Event contracts | **≈1** | Shared state + emit conventions | Medium | Naming conventions; list in `nazec context` |
| 12 | Route / page structure | **1.0** | Inline `page` declarations | Low | Include routes in `nazec context` |
| 13 | Guard logic | **1.0** | Inline `guard` declarations | Low | Include guards in `nazec context` |

### σ_effective Under Different Tooling Scenarios

| Scenario | Concerns at σ>1 | σ_effective | Notes |
|----------|----------------|-------------|-------|
| Single-file app (no components) | None | **1.0** | All 13 concerns co-located |
| Multi-file, no tooling | #4, #7 | **3–5** | Component interfaces are the main driver |
| Multi-file + `nazec context --json` | None | **1.0** | All 13 concerns in structured JSON bundle |
| Multi-file + MCP server | None | **1.0** | Perfect project awareness via structured tool calls |
| Multi-file + MCP + vector index | None | **1.0** | Optimal at scale: only relevant context, minimal tokens |
| Multi-file + expanded themes | None (within styled concerns) | **1.0** | Design tokens fully namespaced |

### Implications for Λ(n) Complexity Class

With σ_effective = 1.0 maintained by tooling:
- Λ(n) stays **Λ-Linear** even for large multi-file applications
- The constant factor increases slightly (interface tokens in context) but scaling remains O(n)
- Naze retains its competitive advantage over frameworks where σ grows with project size

---

## 5. Recommendations — AI-First Priority

The priority order below is optimized for AI agents as the primary code author. It differs significantly from a human-first ordering (which would prioritize LSP and language-level changes). The principle: **build the AI's feedback loop first, then improve generation quality, then scale**.

### Phase 1: MCP Server + `nazec context --json`

**What:** (a) `nazec context <file> --format json` outputs all 13 concerns as structured JSON. (b) `nazec mcp-serve` wraps this (and `check`, `validate`, `parse`) as MCP tools.

**Why first:** MCP is the native protocol for AI tool use. Claude Code, Cursor, and every major AI coding agent supports it. One implementation serves both workflows: CLI for simple scripts, MCP for agent integration. This is the foundation everything else builds on.

**Effort:** ~300 lines for `nazec context`. ~500 lines for MCP server (protocol layer + tool wrappers around existing `resolve()`, `typecheck()`, `parse()`).

**Impact:** σ_effective drops from 3–5 to 1.0. The AI agent's full workflow loop (acquire → generate → validate → iterate) works through structured tool calls instead of file reads and text parsing.

### Phase 2: Machine-Readable Validation

**What:** Add `error_code` and `suggested_fix` fields to `CompileError`. Enrich error messages with machine-actionable corrections.

**Why second:** `nazec check --format json` already works (the diagnostic system in `crates/nazec/src/diagnostic.rs` supports JSON output). The existing `CompileError` struct (`message`, `file`, `line`, `column`, `severity`) is parseable but not actionable — the AI must reason about fixes from free-text error messages. Adding structured fix suggestions turns the validation step from "AI interprets human-readable text" into "AI applies machine-suggested correction."

**Effort:** Small — add 2 optional fields to `CompileError`, populate them in the type checker and codegen error paths.

**Impact:** Iteration speed improves by ~2–3x. The AI needs fewer turns to fix each error because the error itself contains the fix.

### Phase 3: Project-Specific GBNF Grammar

**What:** `nazec grammar --format gbnf --project .` includes project-specific constrained decoding rules — valid component names, param types, route strings, server function names.

**Why third:** This prevents errors at generation time rather than catching them post-hoc. Constrained decoding is the most direct mechanism for driving r toward 0. It's higher-leverage than Phase 2 (which fixes errors faster) because it prevents errors entirely.

**Why not first:** Constrained decoding requires a local LLM setup (llama.cpp, Ollama with GBNF support). The MCP server and validation improvements benefit all AI agents (cloud and local). GBNF benefits local-first FAAD workflows specifically.

**Effort:** ~200 lines — generate GBNF rules from `ResolvedProject.components`. Builds on existing `nazec grammar` infrastructure.

### Phase 4: Local Vector Index

**What:** `nazec index` builds a local semantic index of all project artifacts. Queryable via MCP `search` tool or `nazec search` CLI.

**Why fourth:** Only needed at scale (n>100 components). For smaller projects, `nazec context --json` dumps everything in under 2K tokens and the AI has full context. The vector index becomes valuable when the context dump exceeds the AI's attention budget and irrelevant interfaces cause generation errors.

**Effort:** Moderate. SQLite + sqlite-vec (or BM25/TF-IDF for simpler implementation). ~500 lines for indexing + query + MCP integration.

**Impact:** At n=200 components, the AI queries for 5 relevant interfaces instead of receiving 200. Reduces attention waste and lowers r for large projects.

### Phase 5: Expanded Theme Namespaces

**What:** Add `radii`, `fonts`, `shadows`, `layers`, `opacity` to `theme_section_name` grammar rule.

**Why fifth:** Small grammar change (5 terminals, 0 new rules). Eliminates magic numbers, makes intent explicit for AI generation. The AI generates `radius: theme.radii.md` (always semantically correct) instead of `radius: 8px` (might be wrong).

**Effort:** Small — grammar change + codegen/runtime support for new sections.

### Future: Workspace-Aware LSP

**What:** Upgrade LSP to call `resolve()` and provide cross-file completions, hover, diagnostics.

**Not in the AI priority phases** because LSP serves human interaction patterns (keystroke completions, hover, click-to-navigate). AI agents benefit more from the MCP server, which provides the same underlying data through a workflow-native protocol.

**Build when:** Human developer adoption becomes a priority. The MCP server infrastructure (resolver integration, incremental updates) will have been built in Phase 1, making the LSP upgrade easier.

---

## 6. Implications for Training Data

**Do not generate multi-file training examples until Phase 1 (`nazec context`) is implemented.** Without context tooling, multi-file examples would train the AI to guess component interfaces — exactly the behavior we want to eliminate.

When context tooling exists:
1. Training examples should include `nazec context` output as part of the prompt
2. Examples should demonstrate component usage with full interface visibility
3. The fine-tuning dataset format should be: `{context_bundle, instruction, response}` not just `{instruction, response}`

For now, continue generating single-file examples (σ = 1 by construction) and focus on implementing Phase 1 tooling.

---

## Appendix A: Component Interface Token Cost

A Naze component interface is remarkably compact:

```
card(bg: color = #ffffff, width: number = 200px) { slot }
```

This is ~20 tokens. Compare to the equivalent TypeScript/React interface:

```typescript
interface CardProps {
  bg?: string;
  width?: number;
  children?: React.ReactNode;
}
```

This is ~25 tokens, plus the AI must understand React's children convention, optional vs. required, and the mapping between TypeScript types and DOM types.

For a project with 30 components, the total interface context cost in Naze is ~600 tokens — less than a single medium-sized React component file. This means `nazec context` can include ALL component interfaces and still keep total context under 2K tokens for most projects.

## Appendix B: σ = 1 Design Checklist for New Features

When proposing a new language feature, verify against this checklist:

1. **Can the AI generate correct code using this feature by reading only the current file?** If no, the feature needs a namespace prefix or must be bundled by `nazec context`.
2. **Does the feature introduce a new reference that points outside the file?** If yes, the reference name must encode enough type/role information that the AI doesn't need to resolve it (e.g., `theme.colors.*` encodes "this is a color token").
3. **Does the feature require a new file type?** If yes, that file's contents must be summarizable in <20 tokens per entry for inclusion in context bundles.
4. **Is there exactly one canonical form?** Multiple valid forms increase r. If the feature can be expressed in more than one way, choose one and reject the others.
5. **Does the feature push grammar rules past the 200-rule hard limit?** (Currently at ~157 rules.) If yes, consider whether it can reuse existing grammar patterns.

## Appendix C: Implementation Sketches

Concrete implementation details for each phase of the FAAD pipeline. All sketches reference real file paths, existing function signatures, and crate dependencies already present in `nazec/Cargo.toml`. No new external dependencies are required.

### C.1 `nazec context --json` (Phase 1a)

**New file:** `crates/nazec/src/context.rs`

**Call chain** (reuses the existing `build::check` pattern):

```rust
// crates/nazec/src/context.rs
use naze_compiler::resolve::{self, ComponentDef, ResolvedProject};
use naze_compiler::typecheck;
use naze_parser::ast::{Node, Param, Type, Value};
use crate::manifest::Manifest;
use crate::deps;

#[derive(serde::Serialize)]
pub struct ProjectContext {
    pub components: Vec<ComponentInterface>,
    pub server_functions: Vec<ServerFnInterface>,
    pub shared_state: Vec<StateVar>,
    pub data_sources: Vec<DataSource>,
    pub pages: Vec<Route>,
    pub guards: Vec<Guard>,
    pub theme_tokens: ThemeTokens,
    pub env_vars: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct ComponentInterface {
    pub name: String,
    pub params: Vec<ParamDef>,
    pub has_slot: bool,
    pub file: String,
}

#[derive(serde::Serialize)]
pub struct ParamDef {
    pub name: String,
    pub ty: String,        // "text" | "number" | "bool" | "color"
    pub default: Option<String>,
}

pub fn run(manifest: &Manifest, deps: &[resolve::ResolvedDep])
    -> Result<(), Box<dyn std::error::Error>>
{
    let project = resolve::resolve(".", &manifest.build.entry, deps);
    let ctx = extract_context(&project);
    println!("{}", serde_json::to_string_pretty(&ctx)?);
    Ok(())
}
```

**Extraction logic:** Walk `project.components.values()` → map `ComponentDef` to `ComponentInterface`. Walk `project.entry.nodes` → filter by AST variant:
- `Node::ServerFunctionDef { name, params, body }` → `ServerFnInterface`
- `Node::SharedState { name, value }` → `StateVar`
- `Node::Data { name, source, .. }` → `DataSource`
- `Node::Page { path, guard, .. }` → `Route`
- `Node::Guard { name, checks }` → `Guard`

**Key types from codebase:**
- `ComponentDef` (`resolve.rs:67`): fields `name`, `params: Vec<Param>`, `children`, `file`
- `Param` (`ast.rs:464`): fields `name: String`, `ty: Type`, `default: Option<Value>`
- `Type` (`ast.rs:501`): variants `Text | Number | Bool | Color`
- `ResolvedProject` (`resolve.rs:105`): fields `entry`, `components: HashMap<String, ComponentDef>`, `themes`, `imports`, `errors`

**CLI addition** (clap derive in `cli.rs`):

```rust
/// Export project context as JSON for AI agents
Context {
    /// Path to naze.toml (default: ./naze.toml)
    #[arg(long, default_value = "naze.toml")]
    manifest: String,
}
```

**Example output** (for a project with 2 components):

```json
{
  "components": [
    {
      "name": "card",
      "params": [
        { "name": "bg", "ty": "color", "default": "#ffffff" },
        { "name": "width", "ty": "number", "default": "200px" }
      ],
      "has_slot": true,
      "file": "components/card.naze"
    }
  ],
  "server_functions": [
    { "name": "list-users", "params": [] },
    { "name": "add-user", "params": [
      { "name": "name", "ty": "text" },
      { "name": "email", "ty": "text" }
    ]}
  ],
  "shared_state": [
    { "name": "logged-in", "ty": "bool", "default": "false" }
  ],
  "pages": [
    { "path": "/login", "guard": null },
    { "path": "/dashboard", "guard": "auth" }
  ],
  "guards": [
    { "name": "auth", "redirect": "/login" }
  ],
  "theme_tokens": {
    "colors": ["primary", "secondary", "success", "warning", "danger"],
    "spacing": ["xs", "sm", "md", "lg", "xl", "xxl"]
  },
  "env_vars": ["API_URL", "SECRET"]
}
```

**Token cost:** ~200 tokens for the example above. Scales linearly at ~20 tokens per component/function.

---

### C.2 `nazec mcp-serve` (Phase 1b)

**New file:** `crates/nazec/src/mcp_serve.rs`

**Protocol:** JSON-RPC 2.0 over stdio. Same transport as LSP but with MCP tool definitions instead of LSP methods. No new crate dependencies — uses `tokio` (already: `version = "1", features = ["full"]`) and `serde_json` (already in workspace).

**Async pattern** (from `playground.rs:10`):

```rust
// crates/nazec/src/mcp_serve.rs
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move { serve_stdio().await })
}

async fn serve_stdio() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    // Read JSON-RPC messages from stdin, dispatch, write to stdout
    loop {
        let msg = read_message(&mut stdin).await?;
        let response = dispatch(msg).await;
        write_message(&mut stdout, response).await?;
    }
}
```

**Tool definitions** (MCP `tools/list` response):

| Tool | Wraps | Purpose |
|------|-------|---------|
| `get_context` | `context::extract_context()` | Full project context JSON |
| `validate` | `build::check()` | Type-check, return errors as JSON |
| `list_components` | `project.components.values()` | Component names + signatures |
| `list_routes` | filter `Node::Page` from entry | Route paths + guards |
| `search` | `index::query()` (Phase 4) | FTS5 search over project symbols |

**Message format** (JSON-RPC 2.0):

```json
// Request
{ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
  "params": { "name": "validate", "arguments": { "file": "app.naze" } } }

// Response
{ "jsonrpc": "2.0", "id": 1, "result": {
    "content": [{ "type": "text", "text": "{\"errors\": [], \"warnings\": []}" }]
  }
}
```

**Dispatch** maps tool names to existing functions:
- `get_context` → `context::run()` (C.1 above), capture stdout
- `validate` → `resolve::resolve()` + `typecheck::typecheck()`, serialize errors to JSON
- `list_components` → `resolve::resolve()`, extract `project.components` keys
- `list_routes` → `resolve::resolve()`, filter `Node::Page` from `project.entry.nodes`
- `search` → `index::query()` when available, fallback to grep-style name matching

**State management:** On `initialize`, resolve the project once and cache the `ResolvedProject`. On `notifications/didChange` (file save), re-resolve incrementally. This matches the `BuildCache` pattern already in `resolve.rs:118`.

---

### C.3 Enhanced `CompileError` (Phase 2)

**File:** `crates/naze-compiler/src/error.rs`

**Current struct** (line 10):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileError {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
}
```

**Add two optional fields:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileError {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<String>,
}
```

**Impact:** `CompileError` already derives `Serialize` — the JSON output from `nazec check --format json` will automatically include the new fields. The `#[serde(skip_serializing_if)]` ensures existing consumers see no change when fields are `None`.

**Construction sites** need no changes — all existing `CompileError { message, file, line, column, severity }` constructions add `..Default::default()` or the two `None` fields. Since there are ~50+ construction sites in `typecheck.rs` and `resolve.rs`, implement `Default` for the new fields by adding a helper:

```rust
impl CompileError {
    pub fn new(message: String, file: String, line: usize, column: usize, severity: Severity) -> Self {
        Self { message, file, line, column, severity, error_code: None, suggested_fix: None }
    }

    pub fn with_fix(mut self, code: &str, fix: &str) -> Self {
        self.error_code = Some(code.to_string());
        self.suggested_fix = Some(fix.to_string());
        self
    }
}
```

**Example error codes and fixes:**

| Code | Message | `suggested_fix` |
|------|---------|-----------------|
| `E001` | unknown component "card" | `Add 'use "components/card"' at top of file` |
| `E002` | type mismatch: expected color, got text | `Use a hex color literal: #2563eb` |
| `E003` | unknown theme token "colors.accent" | `Available tokens: primary, secondary, success, warning, danger` |
| `E004` | unknown state variable "count" | `Declare with: state count = 0` |
| `E005` | server function "list-users" called but not defined | `Define: server function list-users() { ... }` |

**JSON output example:**

```json
{
  "message": "unknown component \"card\"",
  "file": "app.naze",
  "line": 5,
  "column": 4,
  "severity": "Error",
  "error_code": "E001",
  "suggested_fix": "Add 'use \"components/card\"' at top of file"
}
```

The AI agent reads `suggested_fix` directly — no parsing of human-readable messages required.

---

### C.4 Project-Specific GBNF (Phase 3)

**File:** `crates/nazec/src/grammar.rs`

**Current entry point** (line 725):

```rust
pub fn run(format: GrammarFormat, no_test: bool) -> Result<(), Box<dyn std::error::Error>> {
    let rules = parse_pest(PEST_SOURCE);
    let output = match format {
        GrammarFormat::Gbnf => to_gbnf(&rules, !no_test),
        GrammarFormat::Ebnf => to_ebnf(&rules, !no_test),
    };
    print!("{}", output);
    Ok(())
}
```

**Extended signature** — add optional `--project` flag:

```rust
pub fn run(
    format: GrammarFormat,
    no_test: bool,
    project_dir: Option<&str>,  // NEW: if Some, inject project-specific rules
) -> Result<(), Box<dyn std::error::Error>> {
    let rules = parse_pest(PEST_SOURCE);
    let mut output = match format {
        GrammarFormat::Gbnf => to_gbnf(&rules, !no_test),
        GrammarFormat::Ebnf => to_ebnf(&rules, !no_test),
    };

    if let Some(dir) = project_dir {
        let project_rules = extract_project_rules(dir)?;
        output.push_str(&project_rules);
    }

    print!("{}", output);
    Ok(())
}
```

**`extract_project_rules`** resolves the project and generates GBNF terminals:

```rust
fn extract_project_rules(dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    let manifest = crate::manifest::load(format!("{}/naze.toml", dir))?;
    let deps = crate::deps::resolve_deps(&manifest)?;
    let project = resolve::resolve(dir, &manifest.build.entry, &deps);

    let mut out = String::new();
    out.push_str("\n# Project-specific rules (auto-generated)\n\n");

    // Component names as constrained alternatives
    let names: Vec<_> = project.components.keys().collect();
    if !names.is_empty() {
        out.push_str("project-component-name ::= ");
        out.push_str(&names.iter()
            .map(|n| format!("\"{}\"", n))
            .collect::<Vec<_>>()
            .join(" | "));
        out.push('\n');
    }

    // Route paths as constrained alternatives
    let routes = collect_routes(&project.entry.nodes);
    if !routes.is_empty() {
        out.push_str("project-route ::= ");
        out.push_str(&routes.iter()
            .map(|r| format!("\"{}\"", r))
            .collect::<Vec<_>>()
            .join(" | "));
        out.push('\n');
    }

    // Server function names
    let fns = collect_server_fns(&project.entry.nodes);
    if !fns.is_empty() {
        out.push_str("project-server-fn ::= ");
        out.push_str(&fns.iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(" | "));
        out.push('\n');
    }

    // Theme token names
    for theme in &project.themes {
        let tokens: Vec<_> = theme.colors.keys().collect();
        if !tokens.is_empty() {
            out.push_str("project-color-token ::= ");
            out.push_str(&tokens.iter()
                .map(|t| format!("\"theme.colors.{}\"", t))
                .collect::<Vec<_>>()
                .join(" | "));
            out.push('\n');
        }
    }

    Ok(out)
}
```

**Example generated GBNF** (for the `server-fn-crud.naze` example project):

```
# Project-specific rules (auto-generated)

project-component-name ::= "card" | "button" | "nav"
project-route ::= "/login" | "/dashboard" | "/settings"
project-server-fn ::= "list-users" | "add-user" | "remove-user"
project-color-token ::= "theme.colors.primary" | "theme.colors.secondary" | "theme.colors.danger"
```

When an AI uses constrained decoding with this grammar, it physically cannot generate references to nonexistent components, routes, or server functions — the decoder rejects invalid tokens before they're emitted.

---

### C.5 Local Vector Index (Phase 4)

**New file:** `crates/nazec/src/index.rs`

**Dependencies:** `rusqlite` with `bundled` feature — already in `nazec/Cargo.toml` under the `database` feature flag. FTS5 is included in the bundled SQLite. The SQLite access pattern follows `naze-registry/src/db.rs`:

```rust
// crates/nazec/src/index.rs
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct ProjectIndex {
    conn: Mutex<Connection>,
}

impl ProjectIndex {
    pub fn open(project_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = project_dir.join(".naze").join("index.db");
        std::fs::create_dir_all(db_path.parent().unwrap())?;
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("
            CREATE VIRTUAL TABLE IF NOT EXISTS project_index USING fts5(
                kind,           -- 'component' | 'server_fn' | 'state' | 'route' | 'guard'
                name,           -- identifier name
                signature,      -- param list or type info
                source_file,    -- origin .naze file path
                content         -- full declaration text for context
            );
        ")?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn reindex_file(
        &self,
        file: &str,
        entries: Vec<IndexEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        // Delete stale entries for this file
        conn.execute(
            "DELETE FROM project_index WHERE source_file = ?",
            [file],
        )?;
        // Insert fresh entries
        let mut stmt = conn.prepare(
            "INSERT INTO project_index (kind, name, signature, source_file, content)
             VALUES (?, ?, ?, ?, ?)"
        )?;
        for e in entries {
            stmt.execute(rusqlite::params![
                e.kind, e.name, e.signature, file, e.content
            ])?;
        }
        Ok(())
    }

    pub fn query(&self, q: &str) -> Result<Vec<IndexEntry>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT kind, name, signature, source_file, content
             FROM project_index WHERE project_index MATCH ?"
        )?;
        let rows = stmt.query_map([q], |row| {
            Ok(IndexEntry {
                kind: row.get(0)?,
                name: row.get(1)?,
                signature: row.get(2)?,
                source_file: row.get(3)?,
                content: row.get(4)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexEntry {
    pub kind: String,
    pub name: String,
    pub signature: String,
    pub source_file: String,
    pub content: String,
}
```

**Index population** — called from `context.rs` after resolution:

```rust
pub fn build_index(project: &ResolvedProject, project_dir: &Path)
    -> Result<ProjectIndex, Box<dyn std::error::Error>>
{
    let index = ProjectIndex::open(project_dir)?;

    // Index components
    for (path, comp) in &project.components {
        let sig = comp.params.iter()
            .map(|p| format!("{}: {}", p.name, type_name(&p.ty)))
            .collect::<Vec<_>>().join(", ");
        index.reindex_file(
            &comp.file.to_string_lossy(),
            vec![IndexEntry {
                kind: "component".into(),
                name: comp.name.clone(),
                signature: sig,
                source_file: comp.file.to_string_lossy().into(),
                content: format!("use \"{}\"", path),
            }],
        )?;
    }

    // Index server functions, state, routes, guards from entry.nodes
    // (same walk as context::extract_context)

    Ok(index)
}
```

**Incremental update:** On file save (dev server hot-reload or MCP `didChange` notification), call `index.reindex_file(changed_file, new_entries)`. FTS5 handles the deletion + re-insertion atomically.

**MCP integration:** The `search` tool in `mcp_serve.rs` calls `index.query(q)`:

```json
// Request
{ "method": "tools/call",
  "params": { "name": "search", "arguments": { "query": "user auth" } } }

// Response — FTS5 matches across all indexed symbols
{ "result": { "content": [{ "type": "text", "text": "[
    {\"kind\":\"guard\",\"name\":\"auth\",\"signature\":\"check logged-in\",\"source_file\":\"app.naze\"},
    {\"kind\":\"server_fn\",\"name\":\"add-user\",\"signature\":\"name: text, email: text\",\"source_file\":\"app.naze\"},
    {\"kind\":\"route\",\"name\":\"/login\",\"signature\":\"guard: null\",\"source_file\":\"app.naze\"}
  ]" }] } }
```

**Storage:** The index lives at `.naze/index.db` inside the project directory. Typical size: <100KB for projects with 100+ components. The `.naze/` directory should be added to `.gitignore`.

**Feature flag:** Since this uses `rusqlite`, gate behind the existing `database` feature: `cargo build -p nazec --features database`. Projects not using the index get zero binary size impact.
