# Phase 5: Production-Ready Application Platform

**Goal:** Close the critical gaps vs Next.js/React for production SaaS applications. After Phase 5, Naze apps can have configurable deployments, dynamic URL-driven pages, server-side data access with databases, protected routes, graceful error handling, and per-page SEO metadata.

**Status:** M31-M38 all complete. M39-M41 complete (see [PHASE5B.md](PHASE5B.md)). 385 workspace tests passing. WASM binary: 374KB.

**Architecture shift:** Phase 4 added server-side capabilities (SSR/SSG, server functions, AI prompts) and the package ecosystem. Phase 5 deepens the server story (multi-statement server functions, database queries, auth) and adds production infrastructure (env vars, dynamic routing, guards, error boundaries, head management).

---

## Dependency Graph

```
M31 (Env Vars) ──────────────────────────────── foundation
  ├── M32 (Dynamic Routing) ─── M34 (Page Meta)
  │                          └── M36 (Guards) ── M37 (Auth)
  ├── M33 (Server Fns) ──────── M38 (Database) ── M39 (Declarative Queries)
  └── M35 (Error Boundaries) ── independent
                                                   M40 (Browser API Parity) ── independent
                                                   M41 (WASM Size Optimization) ── independent
```

**Implementation order:** M31 → M32 → M35 → M33 → M34 → M36 → M37 → M38 → M39 → M40 → M41

---

## M31: Environment Variables
**Crates:** `nazec` (manifest, build, dev), `naze-compiler` (codegen, typecheck), `naze-ir`

Configure applications per deployment. API URLs, secrets, and feature flags declared in `naze.toml [env]`, referenced via `env.NAME` in code.

- [x] `[env]` section in `naze.toml` with string defaults and `{ from: "VAR", required: true }` specs
- [x] Compile-time substitution of `env.*` references in client code (inlined as literals)
- [x] `IrExpression::EnvRef` for server-side runtime resolution via `std::env::var()`
- [x] Type checker validates `env.*` references exist in manifest
- [x] `.env` file loading in `nazec dev` (simple KEY=VALUE, no third-party crate)
- [x] Build-time validation of required env vars in `nazec build`
- [x] `nazec new` template includes commented `[env]` example

---

## M32: Dynamic Routing
**Crates:** `naze-parser` (AST), `naze-compiler` (codegen, typecheck), `naze-ir`, `naze-runtime`, `nazec` (serve)

Dynamic URL segments, catch-all routes, and automatic parameter extraction.

- [x] Route parameter syntax: `page "/users/:id" { ... }` — `:id` segments extracted by compiler
- [x] `params.*` namespace auto-bound from URL: `{params.id}` in interpolations
- [x] Catch-all route: `page "/*" { ... }` for 404 pages
- [x] Segment-based pattern matching in runtime `get_page_nodes()` (replaces exact string match)
- [x] Same pattern matching in SSR `find_page_nodes()` with param injection
- [x] Type checker validates `params.*` references match declared route parameters
- [x] Type checker warns on duplicate routes, validates catch-all is last

---

## M35: Error Boundaries
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck)

Graceful error recovery wrapping data-fetching subtrees.

- [x] `boundary { ... } catch { ... }` grammar rule
- [x] Compiler desugars to `__if` — scans boundary children for data declarations, builds `!(d1.error || d2.error)` condition
- [x] Normal content rendered when no errors, catch content rendered when any data source errors
- [x] Type checker validates boundary contains at least one data declaration
- [x] No runtime or IR changes — pure compile-time desugaring

---

## M33: Enhanced Server Functions
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck), `naze-ir`, `nazec` (exec, server_fns)

Multi-statement server function bodies with `let` bindings and `fetch` expressions. Enables real backend logic.

- [x] Grammar: `server_body`, `server_let`, `server_fetch` rules
- [x] `let name = fetch "url" { method: post, body: {...} }` inside server functions
- [x] Sequential evaluation: each `let` binding available to subsequent expressions
- [x] `IrServerBody` with `steps: Vec<(String, IrServerStep)>` and `result: IrExpression`
- [x] `evaluate_server_body()` in exec.rs — actual HTTP fetches via `reqwest`
- [x] Env var references (`env.API_URL`) resolved at request time in server context

---

## M34: Page Metadata and Head Management
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen), `naze-ir`, `naze-runtime`, `nazec` (serve, seo)

Per-page `<title>`, `<meta>`, and OpenGraph tags declared inside page blocks.

- [x] `meta title: "..."`, `meta description: "..."`, `meta image: "..."` declarations in page blocks
- [x] Interpolation support: `meta title: "Post: {post.data.title}"`
- [x] Client-side `document.title` updates on page navigation (runtime)
- [x] SSR: per-page meta tags override `naze.toml [seo]` defaults
- [x] `PageDef.meta` field in IR with binary serialization

---

## M36: Guards and Middleware
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck), `naze-ir`, `naze-runtime`, `nazec` (serve)

Declarative route protection with named guards.

- [x] `guard name { if condition { navigate "/path" } }` top-level definition
- [x] `page "/admin" guard: guard-name { ... }` guard reference on pages
- [x] Guard evaluation before page render in runtime — redirect if check fails
- [x] SSR: guard evaluation returns HTTP 302 redirect
- [x] Type checker validates guard names exist, guard bodies are if/navigate patterns
- [x] `GuardDef` and `GuardCheck` in IR with serialization

---

## M37: Authentication Patterns
**Crates:** `naze-parser` (AST), `naze-compiler` (codegen), `naze-ir`, `naze-runtime`, `nazec` (serve, server_fns)

Enable auth workflows without new language primitives — header interpolation, server header forwarding, and auth project templates.

- [x] `DataConfig.headers` supports interpolated values: `headers: { "Authorization": "Bearer {auth-token}" }`
- [x] Runtime resolves header interpolations against state before fetch
- [x] SSR: incoming request `Authorization` header forwarded to server function execution
- [x] `nazec new --template auth` scaffolds login/logout/protected pages with guards
- [x] Example: JWT login flow using shared state + storage + guards + server functions

---

## M38: Database Integration
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck), `naze-ir`, `nazec` (exec)

SQL queries inside server function bodies, with parameterized queries preventing injection.

- [x] `sql "SELECT ... WHERE id = $1" [param]` expression in server functions
- [x] Parameterized queries only (no string interpolation in SQL — prevents injection)
- [x] `$N` placeholder validation: compiler checks param count matches placeholders
- [x] `env.DATABASE_URL` required when `sql` is used (compiler validates)
- [x] `tokio-postgres` and `rusqlite` behind feature flags in nazec Cargo.toml
- [x] Results returned as `RenderValue::List(Vec<RenderValue::Object>)`

---

## Totals

| Metric | Value |
|--------|-------|
| Milestones | 11 (M31-M41) |
| Workspace tests | 385 |
| WASM binary | 374KB |
| Grammar rules | ~138 |
