# Phase 5: Production-Ready Application Platform

**Goal:** Close the critical gaps vs Next.js/React for production SaaS applications. After Phase 5, Naze apps can have configurable deployments, dynamic URL-driven pages, server-side data access with databases, protected routes, graceful error handling, and per-page SEO metadata.

**Phase 4 status:** M23-M30 all complete. 337 workspace tests passing. WASM budget: 360KB. See [PHASE4.md](PHASE4.md).

**Architecture shift:** Phase 4 added server-side capabilities (SSR/SSG, server functions, AI prompts) and the package ecosystem. Phase 5 deepens the server story (multi-statement server functions, database queries, auth) and adds production infrastructure (env vars, dynamic routing, guards, error boundaries, head management).

---

## Dependency Graph

```
M31 (Env Vars) ──────────────────────────────── foundation
  ├── M32 (Dynamic Routing) ─── M34 (Page Meta)
  │                          └── M36 (Guards) ── M37 (Auth)
  ├── M33 (Server Fns) ──────── M38 (Database)
  └── M35 (Error Boundaries) ── independent
```

**Implementation order:** M31 → M32 → M35 → M33 → M34 → M36 → M37 → M38

---

## M31: Environment Variables
**Crates:** `nazec` (manifest, build, dev), `naze-compiler` (codegen, typecheck), `naze-ir`

Configure applications per deployment. API URLs, secrets, and feature flags declared in `naze.toml [env]`, referenced via `env.NAME` in code.

- [ ] `[env]` section in `naze.toml` with string defaults and `{ from: "VAR", required: true }` specs
- [ ] Compile-time substitution of `env.*` references in client code (inlined as literals)
- [ ] `IrExpression::EnvRef` for server-side runtime resolution via `std::env::var()`
- [ ] Type checker validates `env.*` references exist in manifest
- [ ] `.env` file loading in `nazec dev` (simple KEY=VALUE, no third-party crate)
- [ ] Build-time validation of required env vars in `nazec build`
- [ ] `nazec new` template includes commented `[env]` example

---

## M32: Dynamic Routing
**Crates:** `naze-parser` (AST), `naze-compiler` (codegen, typecheck), `naze-ir`, `naze-runtime`, `nazec` (serve)

Dynamic URL segments, catch-all routes, and automatic parameter extraction.

- [ ] Route parameter syntax: `page "/users/:id" { ... }` — `:id` segments extracted by compiler
- [ ] `params.*` namespace auto-bound from URL: `{params.id}` in interpolations
- [ ] Catch-all route: `page "/*" { ... }` for 404 pages
- [ ] Segment-based pattern matching in runtime `get_page_nodes()` (replaces exact string match)
- [ ] Same pattern matching in SSR `find_page_nodes()` with param injection
- [ ] Type checker validates `params.*` references match declared route parameters
- [ ] Type checker warns on duplicate routes, validates catch-all is last

---

## M35: Error Boundaries
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck)

Graceful error recovery wrapping data-fetching subtrees.

- [ ] `boundary { ... } catch { ... }` grammar rule
- [ ] Compiler desugars to `__if` — scans boundary children for data declarations, builds `!(d1.error || d2.error)` condition
- [ ] Normal content rendered when no errors, catch content rendered when any data source errors
- [ ] Type checker validates boundary contains at least one data declaration
- [ ] No runtime or IR changes — pure compile-time desugaring

---

## M33: Enhanced Server Functions
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck), `naze-ir`, `nazec` (exec, server_fns)

Multi-statement server function bodies with `let` bindings and `fetch` expressions. Enables real backend logic.

- [ ] Grammar: `server_body`, `server_let`, `server_fetch` rules
- [ ] `let name = fetch "url" { method: post, body: {...} }` inside server functions
- [ ] Sequential evaluation: each `let` binding available to subsequent expressions
- [ ] `IrServerBody` with `steps: Vec<(String, IrServerStep)>` and `result: IrExpression`
- [ ] `evaluate_server_body()` in exec.rs — actual HTTP fetches via `reqwest`
- [ ] Env var references (`env.API_URL`) resolved at request time in server context

---

## M34: Page Metadata and Head Management
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen), `naze-ir`, `naze-runtime`, `nazec` (serve, seo)

Per-page `<title>`, `<meta>`, and OpenGraph tags declared inside page blocks.

- [ ] `meta title: "..."`, `meta description: "..."`, `meta image: "..."` declarations in page blocks
- [ ] Interpolation support: `meta title: "Post: {post.data.title}"`
- [ ] Client-side `document.title` updates on page navigation (runtime)
- [ ] SSR: per-page meta tags override `naze.toml [seo]` defaults
- [ ] `PageDef.meta` field in IR with binary serialization

---

## M36: Guards and Middleware
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck), `naze-ir`, `naze-runtime`, `nazec` (serve)

Declarative route protection with named guards.

- [ ] `guard name { if condition { navigate "/path" } }` top-level definition
- [ ] `page "/admin" guard: guard-name { ... }` guard reference on pages
- [ ] Guard evaluation before page render in runtime — redirect if check fails
- [ ] SSR: guard evaluation returns HTTP 302 redirect
- [ ] Type checker validates guard names exist, guard bodies are if/navigate patterns
- [ ] `GuardDef` and `GuardCheck` in IR with serialization

---

## M37: Authentication Patterns
**Crates:** `naze-parser` (AST), `naze-compiler` (codegen), `naze-ir`, `naze-runtime`, `nazec` (serve, server_fns)

Enable auth workflows without new language primitives — header interpolation, server header forwarding, and auth project templates.

- [ ] `DataConfig.headers` supports interpolated values: `headers: { "Authorization": "Bearer {auth-token}" }`
- [ ] Runtime resolves header interpolations against state before fetch
- [ ] SSR: incoming request `Authorization` header forwarded to server function execution
- [ ] `nazec new --template auth` scaffolds login/logout/protected pages with guards
- [ ] Example: JWT login flow using shared state + storage + guards + server functions

---

## M38: Database Integration
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck), `naze-ir`, `nazec` (exec)

SQL queries inside server function bodies, with parameterized queries preventing injection.

- [ ] `sql "SELECT ... WHERE id = $1" [param]` expression in server functions
- [ ] Parameterized queries only (no string interpolation in SQL — prevents injection)
- [ ] `$N` placeholder validation: compiler checks param count matches placeholders
- [ ] `env.DATABASE_URL` required when `sql` is used (compiler validates)
- [ ] `tokio-postgres` and `rusqlite` behind feature flags in nazec Cargo.toml
- [ ] Results returned as `RenderValue::List(Vec<RenderValue::Object>)`

---

## Totals

| Metric | Value |
|--------|-------|
| Milestones | 8 (M31-M38) |
| New LOC (est.) | ~4,050 |
| New grammar rules | ~11 |
| Grammar total | ~135 (from ~124) |
| New Cargo deps | `tokio-postgres` (optional), `rusqlite` (optional) |
