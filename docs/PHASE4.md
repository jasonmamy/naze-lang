# Phase 4: Ecosystem & External Integration

**Goal:** Enable production deployment with server rendering, package ecosystem, SEO, and AI integration. Target: Naze apps discoverable by search engines, community contributing shared components, at least 3 non-trivial public Naze apps.

**Architecture shift:** Phase 3 completed the client-side language. Phase 4 adds server-side capabilities (Tier 2 WASM imports, Tier 3 server functions, SSR/SSG), a package ecosystem, and AI tooling. The compiler gains new backends (server binary, edge WASM) and the CLI gains package management commands.

**Prerequisite:** Phase 3 milestones M15-M22 complete. See [PHASE3.md](PHASE3.md).

---

## Phase 4a: External Integration

### M23: WASM Module Imports (Tier 2 Computation)
**Crates:** `naze-parser`, `naze-compiler`

Enable apps to import pre-compiled WASM modules for heavy computation (crypto, image processing, data parsing) without leaving the Naze ecosystem.

- [ ] Grammar: `import` statement rule (`import name from "package"`, `import name from "./path"`)
- [ ] AST: `Node::Import` variant with source (registry path or local path)
- [ ] Compiler: resolve import to `.wasm` file (local path or package registry)
- [ ] Compiler: integrate `wasm-merge` for module merging (imported + runtime → single WASM)
- [ ] Compiler: integrate `wasm-opt` for tree-shaking and dead code elimination post-merge
- [ ] Type-check imported function signatures against Naze type system
- [ ] After merging: cross-module calls become intra-module calls (zero overhead)
- [ ] Lazy loading opt-in: large libraries can be split into separate `.wasm` chunks loaded on demand
- [ ] Packages can ship as `.naze` (source) or `.wasm` (pre-compiled)

**Example syntax:**
```naze
import charts from "@naze/charts"
import crypto from "./lib/crypto.wasm"

app "Dashboard" {
  let hash = crypto.sha256(data)
  charts.line-chart data: sales, width: 600px, height: 300px
}
```

### M24: Server Functions (Tier 3 Computation)
**Crates:** `naze-parser`, `naze-compiler`, new `naze-server`

Functions that run on the server, never ship to client WASM. Auto-generated RPC stubs bridge client and server with type safety.

- [ ] Grammar: `server` function definition rule
- [ ] Grammar: `server(edge)` variant for edge deployment
- [ ] AST: `Node::ServerFunction` variant
- [ ] Compiler: separate server functions from client code during compilation
- [ ] Compiler: auto-generate client RPC stub (HTTP POST, JSON-serialized arguments)
- [ ] Compiler: auto-generate server handler (shares same type signature)
- [ ] Server runtime: execute server functions, handle incoming RPC requests
- [ ] Edge compilation: `server(edge)` functions compiled to WASI WASM for edge runtimes
- [ ] Type-safe client↔server boundary (same types, validated at compile time)

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

### M25: Server Rendering (SSR / SSG)
**Crates:** `naze-server` (extends M24), `naze-layout`

Render Naze apps to HTML on the server for instant first paint and SEO indexing.

- [ ] Server renderer: `naze-layout` → HTML/CSS serialization (no GPU, no canvas)
- [ ] SSG mode: `nazec build` pre-renders all routes to static HTML at build time
- [ ] SSR mode: `nazec build --server` produces server binary that renders per request
- [ ] Embedded HTTP server (Axum for native, WASI HTTP for WASM edge)
- [ ] URL router: maps paths to page routes + server function RPC endpoints
- [ ] Static asset serving: client WASM, images, fonts
- [ ] Loading skeleton generation: extract layout structure from `.naze` for SPA mode placeholder
- [ ] Hydration: client WASM takes over from server-rendered HTML without flicker

**Build modes:**
```bash
nazec build                  # SSG: pre-render all routes to HTML + client WASM
nazec build --client-only    # SPA: HTML shell + client WASM only (current default)
nazec build --server         # Fullstack: server binary + client WASM bundle
```

**Compilation targets:**
- WASM binary for edge/serverless (Cloudflare Workers, Fermyon Spin, Fastly Compute)
- Native binary for containers/VPS (x86-64, ARM via Cranelift/LLVM, ~5-20MB standalone)

---

## Phase 4b: Ecosystem

### M26: Package Manager
**Crate:** `nazec`

Full dependency management for sharing and reusing Naze components.

- [ ] `naze.lock` lockfile: exact resolved versions + source hashes for reproducible builds
- [ ] Semver constraint solving: `version = "^1.0"` resolves to latest compatible
- [ ] Git-based dependencies: `source = "git:github.com/org/repo"` with tag/branch/commit pinning
- [ ] `nazec add @org/lib` — add dependency to `naze.toml`, fetch source, update lockfile
- [ ] `nazec remove @org/lib` — remove dependency
- [ ] `nazec update` — update dependencies to latest matching versions
- [ ] `nazec publish` — publish package to registry
- [ ] `nazec search <query>` — search registry for packages
- [ ] `@org/package` namespace scoping to prevent name conflicts
- [ ] Registry HTTP API: search, fetch, publish, version listing
- [ ] Build cache: compiled artifacts keyed by source hash + compiler version + target platform
- [ ] Cache location: `.nazec/cache/` for compiled artifacts, `.nazec/registry/` for fetched packages
- [ ] Source-distributed packages: packages ship as `.naze` files (no pre-compiled WASM)

**Manifest extensions for `naze.toml`:**
```toml
[dependencies]
"@naze/ui-kit" = "^1.0"
"@myorg/charts" = { git = "github.com/myorg/charts", tag = "v2.1.0" }
```

### M27: SEO / Meta-Index Generation
**Crate:** `nazec` (build pipeline step)

Make Naze apps discoverable by search engines and social media.

- [ ] Extract metadata from `.naze` source: title, description, content structure, route map
- [ ] Generate `<title>` and `<meta description>` from app block
- [ ] Open Graph tags: `og:title`, `og:description`, `og:image`
- [ ] JSON-LD structured data
- [ ] `<link rel="alternate" type="application/naze">` header
- [ ] Route-aware: generate per-route HTML pages for multi-page apps
- [ ] Integrate into `nazec build` pipeline as automatic step
- [ ] Optional: basic HTML text content fallback for no-JS environments

---

## Phase 4c: AI & Advanced Tools

### M28: AI Integration Layer
**Crates:** new `naze-ai` module, `naze-parser` (grammar export)

Enable AI tools to generate correct Naze code reliably via grammar constraints and validation loops.

- [ ] Export C1 grammar as GBNF format (for llama.cpp grammar-constrained decoding)
- [ ] Export C1 grammar as CFG format (for XGrammar/SGLang)
- [ ] Grammar constraint file ships with `nazec` CLI (`nazec grammar --format gbnf`)
- [ ] Validation feedback loop: LLM generates → C2 compiler validates → structured errors fed back → retry
- [ ] Intent-to-Naze pipeline: natural language → `.naze` → compile → preview → iterate
- [ ] Fine-tuning dataset: 500+ Naze examples paired with natural language descriptions
- [ ] Compiler structured error JSON output optimized for LLM consumption
- [ ] Benchmark: token efficiency vs. equivalent React/HTML generation
- [ ] Prompt template library: curated few-shot examples for common UI patterns (nav bars, cards, forms, dashboards)

### M29: AI Prompting Runtime
**Crates:** `naze-runtime`, `naze-compiler`

Enable Naze apps to interact with AI services at runtime via a `prompt` keyword and provider abstraction.

- [ ] Grammar: `prompt` keyword in component declarations
- [ ] `ai.naze` config file format: named provider definitions with type, model, endpoint, `env.VAR` credentials
- [ ] Provider adapters: OpenAI, Anthropic, Ollama/local, generic HTTP (OpenAI-compatible)
- [ ] Streaming support for AI responses
- [ ] Caching: identical prompts with same inputs return cached responses (configurable TTL)
- [ ] Compile-time validation of prompt templates (variable references, provider existence)
- [ ] Runtime: AI response data consumed via reactive data binding

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

Rich tooling for debugging, inspecting, and optimizing Naze apps.

- [ ] **Inspector:** visual element bounds overlay on rendered output (like browser DevTools)
- [ ] **Inspector:** component tree view with expandable hierarchy
- [ ] **Inspector:** property panel showing computed layout values, styles, data bindings
- [ ] **Inspector:** accessibility tree viewer
- [ ] **Debugger:** source map generation (compiler binary offset → `.naze` source location)
- [ ] **Debugger:** WASM debugging via Chrome DevTools Protocol
- [ ] **Debugger:** breakpoints in `.naze` source files
- [ ] **Debugger:** state inspection (view reactive state values at runtime)
- [ ] **Debugger:** event log (what events fired, what handlers ran)
- [ ] **Playground:** browser-based editor (Monaco or CodeMirror)
- [ ] **Playground:** C2 compiler compiled to WASM (runs client-side, no server needed)
- [ ] **Playground:** live split-pane preview (edit left, render right)
- [ ] **Playground:** shareable URLs (encode source in URL hash or short-link)
- [ ] **Playground:** example gallery with instant switching
- [ ] **Size analyzer:** parse WASM binary sections into treemap visualization
- [ ] **Size analyzer:** before/after comparison mode for optimization work

---

## Build Order

```
Track A (server):     M24 → M25 (SSR depends on server runtime)
Track B (imports):    M23 (independent, parallel with Track A)
Track C (ecosystem):  M26 → M27 (SEO benefits from package infrastructure)
Track D (AI):         M28 → M29 (prompting runtime uses grammar constraints)
Track E (tools):      M30 (independent, parallel with everything)
```

**Suggested priority order:**
1. M24 (server functions) — unlocks fullstack apps
2. M25 (SSR/SSG) — unlocks SEO and production deployment
3. M23 (WASM imports) — unlocks third-party computation
4. M26 (package manager) — enables ecosystem and sharing
5. M28 (AI integration) — grammar constraints for reliable AI generation
6. M27 (SEO) — builds on M25
7. M29 (AI prompting runtime) — builds on M28
8. M30 (advanced dev tools) — can be interleaved throughout

## WASM Size Budget

Phase 3 target: <150KB. Phase 4 adds server-side features that don't affect client WASM. WASM imports (M23) increase app-specific size, but tree-shaking keeps unused code out. Target: **runtime WASM stays < 200KB**, app-specific WASM varies by imported modules.
