# Phase 4: Ecosystem & External Integration

**Status:** Complete. All milestones delivered. Deferred items (edge compilation, streaming SSR, build cache) tracked in Phase 6 and Future.

**Goal:** Enable production deployment with server rendering, package ecosystem, SEO, and AI integration. Target: Naze apps discoverable by search engines, community contributing shared components, at least 3 non-trivial public Naze apps.

**Architecture shift:** Phase 3 completed the client-side language. Phase 4 adds server-side capabilities (Tier 2 WASM imports, Tier 3 server functions, SSR/SSG), a package ecosystem, and AI tooling. The compiler gains new backends (HTML renderer, server binary, edge WASM) and the CLI gains package management commands.

**Phase 3 status:** M15-M20, M19b-M19e, M22 all complete. Only M21 (LSP polish) remains and can proceed in parallel with Phase 4 work. See [PHASE3.md](PHASE3.md).

---

## Phase 4a: AI & SEO (No Dependencies)

### M28a: AI Grammar Export
**Crates:** `naze-parser` (grammar export), `nazec` (CLI command)

Export the Naze grammar in formats used by LLM constrained-decoding engines. This is the core AI differentiator and has zero dependencies on other Phase 4 work.

- [x] Export PEG grammar as GBNF format (for llama.cpp grammar-constrained decoding)
- [x] Export PEG grammar as EBNF format (for documentation; XGrammar/SGLang also use GBNF)
- [x] `nazec grammar --grammar-format gbnf|ebnf` CLI command (with `--no-test` flag)
- [x] Grammar constraint file ships with `nazec` CLI (embedded via `include_str!()`)
- [x] Compiler structured error JSON output for LLM consumption (`nazec check --format json`)
- [ ] Benchmark: token efficiency vs. equivalent React/HTML generation

### M27: SEO / Meta-Index Generation
**Crate:** `nazec` (build pipeline step)

Make Naze apps discoverable by search engines and social media. Low-hanging fruit with current static builds — no server infrastructure needed.

- [x] Extract metadata from `naze.toml` `[seo]` section: description, image, author, keywords, canonical, twitter, locale
- [x] Generate `<title>` and `<meta description>` from app block + manifest
- [x] Open Graph tags: `og:title`, `og:description`, `og:image`, `og:url`, `og:locale`
- [x] Twitter Card tags: `twitter:card`, `twitter:title`, `twitter:description`, `twitter:image`, `twitter:site`
- [x] JSON-LD structured data (WebApplication schema)
- [x] `<link rel="alternate" type="application/naze">` header
- [x] Route-aware: generate per-route HTML pages for multi-page apps
- [x] Integrate into `nazec build` pipeline as automatic step
- [x] `<noscript>` text content fallback extracted from render tree
- [x] Canonical URL support with per-page paths

---

## Phase 4b: Package & Import Infrastructure

### M26a: Local Package Dependencies
**Crate:** `nazec`, `naze-compiler`

Add `[dependencies]` support to `naze.toml` for local and git-based packages. This unblocks M23 (WASM imports need dependency resolution).

- [x] `[dependencies]` section in `naze.toml`: local path and git-based deps (`manifest.rs`: `DependencySpec` enum, `DetailedDep` struct)
- [x] `naze.lock` lockfile: exact resolved versions + source hashes for reproducible builds (`deps.rs`: TOML lockfile with `schema_version`, `[[package]]` entries)
- [ ] Semver constraint solving: `version = "^1.0"` resolves to latest compatible (deferred to M26b — needs registry)
- [x] Git-based dependencies: `{ git = "url", tag/branch/rev }` with pinning (`deps.rs`: `resolve_git_dep()`, shell out to `git` CLI)
- [x] `nazec add @org/lib` — add dependency to `naze.toml`, fetch source, update lockfile (`dep_commands.rs`, uses `toml_edit`)
- [x] `nazec remove @org/lib` — remove dependency
- [x] `nazec update` — update dependencies to latest matching versions (clears git cache, re-resolves)
- [x] `@org/package` namespace scoping to prevent name conflicts (grammar: `use_path` supports `@org/package/component`)
- [x] Compiler: resolve imports against `[dependencies]` during build (`resolve.rs`: `discover_dep_files()`, deps threaded through `resolve()`/`resolve_incremental()`)
- [x] Cache location: `.nazec/deps/` for fetched git packages (per-project, not global)
- [x] Source-distributed packages: packages ship as `.naze` files (no pre-compiled WASM)

**Manifest extensions for `naze.toml`:**
```toml
[dependencies]
"@naze/ui-kit" = "^1.0"
"@myorg/charts" = { git = "github.com/myorg/charts", tag = "v2.1.0" }
"local-lib" = { path = "../my-lib" }
```

### M23: WASM Module Imports (Tier 2 Computation)
**Crates:** `naze-parser`, `naze-compiler`, `naze-ir`, `naze-runtime`, `nazec`
**Depends on:** M26a (dependency resolution)

Enable apps to import pre-compiled WASM modules for heavy computation (crypto, image processing, data parsing) without leaving the Naze ecosystem.

**MVP (JS bridge approach — no extra toolchain on user machines):**
- [x] Grammar: `import` statement rule (`import name from "package"`, `import name from "./path"`)
- [x] Grammar: qualified function calls (`module.func(args)`) via `ref_path | ident` in `function_call` rule
- [x] AST: `Node::Import` variant with name and source (registry path or local path)
- [x] Compiler resolve: resolve import to `.wasm` file via M26a dependency resolution (local paths + `@org/package` deps)
- [x] IR: `ImportDecl` type (name, wasm_url, functions) + `IrExpression::WasmCall` variant
- [x] IR: custom binary serialization/deserialization for imports section and WasmCall (tag 6), backward compatible
- [x] Codegen: lower qualified `module.func(args)` calls to `IrExpression::WasmCall` via thread-local import names
- [x] Type check: `wasmparser` reads WASM export tables; validates referenced functions exist in module exports
- [x] Runtime: JS bridge dispatch via `window.__naze_wasm_call(module, func, args)` called from WASM
- [x] Build: copy imported `.wasm` files to dist/, generate `wasm_imports.js` bridge loader, update HTML templates
- [x] Packages can ship as `.naze` (source) or `.wasm` (pre-compiled)

**Deferred (future optimization):**
- [ ] `wasm-merge` for module merging (imported + runtime -> single WASM, zero-overhead cross-module calls)
- [ ] `wasm-opt` tree-shaking post-merge
- [ ] Lazy loading: large libraries split into separate `.wasm` chunks loaded on demand
- [ ] String/buffer passing across WASM boundary (MVP is numeric f64 args/returns)

**Example syntax:**
```naze
import crypto from "./lib/crypto.wasm"
import math from "@naze/math"

app "Dashboard" {
  computed hash = crypto.sha256(x)
  computed result = math.add(a, b)
}
```

---

## Phase 4c: Server Infrastructure

### M24: Server Functions (Tier 3 Computation)
**Crates:** `naze-parser`, `naze-compiler`, new `naze-server`
**Extends:** existing Axum dev server (`crates/nazec/src/dev.rs` — HTTP routing, WebSocket, static file serving, broadcast reload)

Functions that run on the server, never ship to client WASM. Auto-generated RPC stubs bridge client and server with type safety. The existing dev server provides the Axum foundation; M24 extends it with RPC endpoints and server function execution.

- [x] Grammar: `server function` definition rule (`server_function_def`) and `server data` call rule (`server_data_stmt`)
- [ ] Grammar: `server(edge)` variant for edge deployment
- [x] AST: `Node::ServerFunction` and `Node::ServerData` variants
- [x] Compiler: separate server functions from client code during compilation (codegen lowers to `ServerFuncDecl`/`ServerCallDecl`)
- [x] Compiler: auto-generate client RPC stub (runtime POSTs to `/api/{name}` with JSON args)
- [x] Compiler: auto-generate server handler (dev server evaluates function body via `exec::evaluate_expr`)
- [x] Server runtime: execute server functions, handle incoming RPC requests (Axum `/api/{name}` route in dev server)
- [ ] Edge compilation: `server(edge)` functions compiled to WASI WASM for edge runtimes
- [x] Type-safe client<->server boundary (typecheck validates server function references + arg counts)
- [x] IR: `ServerFuncDecl` and `ServerCallDecl` types with custom binary serialization (backward compatible)
- [x] Runtime (WASM): three-state pattern (`{name}.loading`, `{name}.error`, `{name}.data`) for server calls
- [x] Build: production warning when server functions present without `nazec dev`

**Example syntax:**
```naze
server function get-user(id: number) -> { name: text, email: text } {
  -- Runs on server only; implementation accesses database
  db.query "SELECT name, email FROM users WHERE id = ?" [id]
}

server(edge) function validate-token(token: text) -> bool {
  -- Compiled to WASM for Cloudflare Workers / Fermyon Spin
  crypto.verify(token, env.SECRET_KEY)
}

app "Profile" {
  data user: get-user(42)
  if user.loading { text "Loading..." }
  if user.data { text user.data.name }
}
```

### M25a: Static Site Generation (SSG)
**Crates:** `nazec` (html_renderer module)
**Depends on:** none (can start independently)

Pre-render Naze apps to static HTML at build time. Uses CSS flexbox/grid for layout (not the layout engine's absolute positioning) — Naze's row/column/grid/stack primitives map directly to CSS flex/grid.

- [x] HTML renderer module: RenderNode tree -> semantic HTML + inline CSS (`html_renderer.rs`)
- [x] Map node kinds to semantic HTML elements (`<p>`, `<h1>`-`<h6>`, `<nav>`, `<img>`, `<input>`, etc.)
- [x] CSS generation: convert props to CSS properties (color, padding, gap, border-radius, shadow presets, etc.)
- [x] Text rendering: output text nodes as `<p>`/`<h1>` with CSS typography (font-size, color, weight, align)
- [x] Image references: `<img>` tags with src/alt attributes
- [x] SSG mode: `nazec build --static` pre-renders all routes to static HTML at build time
- [x] Route-aware: generates per-route HTML files for multi-page apps
- [x] Client WASM bundle included for interactivity (progressive enhancement)
- [x] State resolution at build time: evaluates `__if`/`__each`/interpolated strings via `exec.rs` functions
- [x] Form elements: input, textarea, checkbox, radio, select rendered to native HTML
- [x] Layout containers: row (flex-row), column (flex-column), grid, stack (relative/absolute), scroll (overflow:auto)
- [ ] Loading skeleton generation: extract layout structure from `.naze` for SPA mode placeholder

**Build mode:**
```bash
nazec build --static         # SSG: pre-render all routes to HTML + client WASM
nazec build                  # SPA: HTML shell + client WASM only (default)
```

### M25b: Server-Side Rendering (SSR + Hydration)
**Crates:** `nazec` (serve module, server_fns module)
**Depends on:** M24 (server runtime), M25a (HTML renderer)

Dynamic server rendering with client-side hydration. `nazec serve` starts a production Axum server that renders HTML per request, pre-evaluates server functions server-side, and includes the WASM bundle for progressive enhancement.

- [x] SSR mode: `nazec serve` starts production SSR server (reads from `dist/` built by `nazec build`)
- [x] Server uses `html_renderer` to render HTML on each request
- [x] Embedded HTTP server (Axum, reuses dev server pattern)
- [x] URL router: maps paths to page routes + server function RPC endpoints
- [x] Static asset serving: client WASM, images, fonts via ServeDir fallback
- [x] Hydration: client WASM takes over from server-rendered HTML (CSS swap pattern from M25a)
- [x] Server function pre-evaluation: server calls evaluated server-side, results embedded in HTML
- [x] Hydration state serialization: `<script id="__naze_state">` for future client-side state reuse
- [ ] Streaming: send HTML head immediately, body progressively as data loads
- [ ] WASI HTTP for edge runtimes (abstract HTTP layer behind trait)

**Usage:**
```bash
nazec build                  # Compile .naze -> dist/ (app_data.bin, WASM, JS)
nazec serve [--port 8080]    # Start SSR server that renders from dist/
```

**Compilation targets (future):**
- WASM binary for edge/serverless (Cloudflare Workers, Fermyon Spin, Fastly Compute)
- Native binary for containers/VPS (x86-64, ARM via Cranelift/LLVM, ~5-20MB standalone)

---

## Phase 4d: Ecosystem & AI Runtime

### M26b: Package Registry & Publishing
**Crates:** `nazec` (registry module), new `naze-registry`
**Depends on:** M26a (local dependencies), M24 (server for registry API)

Public registry for sharing Naze packages. Extends the local dependency system from M26a with remote publishing, search, and versioning.

- [x] `nazec publish` — package project as tarball (.naze + naze.toml + README.md), upload to registry (`registry.rs`: `publish_package()`)
- [x] `nazec search <query>` — search registry for packages, print results (`registry.rs`: `search_packages()`)
- [x] Registry HTTP API: `naze-registry` crate (Axum + SQLite + filesystem storage) with publish, search, download, version listing
- [x] Semver version resolution: `version = "^1.0"` in naze.toml resolves via registry to latest compatible version (`deps.rs`: `resolve_registry_dep()`)
- [x] Registry client: `RegistryClient` struct with resolve, download, extract, publish operations (`registry.rs`)
- [x] Lockfile: `source = "registry"` entries with `version` and `checksum` fields for reproducible builds
- [x] Download cache: extracted packages cached in `.nazec/deps/{name}-registry-{version}/`
- [x] `nazec add --version "^1.0"` — add registry dependency; bare `nazec add foo` defaults to `"*"`
- [ ] Build cache: compiled artifacts keyed by source hash + compiler version + target platform

### M28b: AI Validation & Fine-Tuning
**Crates:** new `naze-ai` module
**Depends on:** M28a (grammar export)

Build the validation loop and training data that make AI-generated Naze reliable.

- [x] Validation feedback loop: LLM generates -> compiler validates -> structured errors fed back -> retry (`nazec ai generate`)
- [x] Intent-to-Naze pipeline: natural language -> `.naze` -> compile -> preview -> iterate (`nazec ai generate` + `nazec ai fix`)
- [x] Fine-tuning dataset: export examples as JSONL training pairs with LLM-generated descriptions (`nazec ai dataset export`)
- [x] Dataset validation: verify all JSONL entries compile correctly (`nazec ai dataset validate`)
- [x] Prompt template library: curated few-shot examples for common UI patterns (counter, dashboard, data-fetch) embedded in binary

### M29: AI Prompting Runtime
**Crates:** `naze-runtime`, `naze-compiler`
**Depends on:** M28b (validation ensures reliable generation)

Enable Naze apps to interact with AI services at runtime via a `prompt` keyword and provider abstraction.

- [x] Grammar: `prompt` keyword in component declarations
- [ ] `ai.naze` config file format: named provider definitions with type, model, endpoint, `env.VAR` credentials
- [x] Provider adapters: OpenAI, Anthropic, Ollama/local, generic HTTP (OpenAI-compatible)
- [ ] Streaming support for AI responses
- [ ] Caching: identical prompts with same inputs return cached responses (configurable TTL)
- [x] Compile-time validation of prompt templates (variable references, provider existence)
- [x] Runtime: AI response data consumed via reactive data binding

**Example syntax:**
```naze
prompt summary: from openai {
  system: "Summarize the following text concisely."
  user: "{article-text}"
}

if summary.loading { text "Generating summary..." }
if summary.data { text summary.data }
```

### M30: Advanced Developer Tools
**Crates:** `nazec`, `naze-compiler`, potentially new crates

Rich tooling for debugging, inspecting, and optimizing Naze apps. Interleaved throughout Phase 4, not blocked on other milestones.

- [x] **Inspector:** visual element bounds overlay on rendered output (like browser DevTools)
- [x] **Inspector:** component tree view with expandable hierarchy
- [x] **Inspector:** property panel showing computed layout values, styles, data bindings
- [x] **Inspector:** accessibility tree viewer
- [ ] **Debugger:** WASM debugging via Chrome DevTools Protocol (source maps from M22 provide the foundation)
- [ ] **Debugger:** breakpoints in `.naze` source files
- [x] **Debugger:** state inspection (view reactive state values at runtime)
- [x] **Debugger:** event log (what events fired, what handlers ran)
- [x] **Playground:** browser-based editor (textarea with compile-on-type)
- [x] **Playground:** compiler compiled to WASM (runs client-side, no server needed)
- [x] **Playground:** live split-pane preview (edit left, render right)
- [x] **Playground:** shareable URLs (encode source in URL hash)
- [x] **Playground:** example gallery with instant switching
- [x] **Size analyzer:** parse WASM binary sections into table visualization
- [x] **Size analyzer:** before/after comparison mode for optimization work

---

## Build Order

```
Track A (AI export):    M28a (independent, start first — core differentiator)
Track B (SEO):          M27 (independent, low-hanging fruit with current static builds)
Track C (packages):     M26a → M23 (imports need dependency resolution)
Track D (server):       M24 → M25a → M25b (SSG before SSR, extend existing Axum server)
Track E (ecosystem):    M26b (needs M26a + M24 for registry server)
Track F (AI runtime):   M28b → M29 (validation/fine-tuning before runtime)
Track G (tools):        M30 (continuous, interleaved throughout)
```

**Suggested priority order:**
1. ~~M28a (AI grammar export)~~ **DONE**
2. ~~M27 (SEO/meta-index)~~ **DONE**
3. ~~M26a (local package deps)~~ **DONE**
4. ~~M23 (WASM imports)~~ **DONE**
5. ~~M24 (server functions)~~ **DONE**
6. ~~M25a (SSG)~~ **DONE**
7. ~~M25b (SSR + hydration)~~ **DONE**
8. ~~M26b (package registry)~~ **DONE**
9. ~~M29 (AI prompting runtime)~~ **DONE** (core: grammar, codegen, providers, runtime binding)
10. ~~M28b (AI validation & fine-tuning)~~ **DONE** (`nazec ai generate/fix/dataset`)
11. ~~M30 (dev tools)~~ **DONE** (inspector, debugger events/state, playground, size analyzer)

## WASM Size Budget

**Current state:** Runtime WASM is 355KB with `wasm-opt = false` (never enabled in `crates/naze-runtime/Cargo.toml`). Test budget is 360KB.

**Immediate action (before Phase 4 milestones):**
- Enable `wasm-opt` in `crates/naze-runtime/Cargo.toml` — expect ~30-40% reduction (~200-235KB)
- Profile WASM binary sections to identify further optimization targets

**Phase 4 target:** Runtime WASM stays **< 250KB** after wasm-opt. App-specific WASM (M23 imports) is budgeted separately and varies by imported modules. Tree-shaking via `wasm-opt` keeps unused code out of merged bundles.

Server-side features (M24, M25) don't affect client WASM size.
