# Agent Runtime Implementation Plan

> Concrete engineering plan for the vision described in [AGENT_RUNTIME.md](AGENT_RUNTIME.md).

## Overview

The Naze binary (`app_data.bin`) already contains the complete application semantics — state schema, UI tree, actions, computed values, data bindings, and server function signatures. Today this data is consumed only by the WASM runtime for browser rendering. This plan turns it into an agent-native platform: machine-discoverable, headless-executable, and programmatically composable.

**Five phases, each independently shippable:**

```
Phase A: Manifest Generation ─── naze-manifest.json on every build
Phase B: Headless Binary ─────── Layer 1-only output (no UI)
Phase C: Agent Crate ─────────── Rust API for loading, inspecting, executing
Phase D: Discovery & Registry ── structural search + trust scoring
Phase E: Discovery Service ───── build-time announcement + capability indexing
```

**What makes this tractable:** most of the infrastructure already exists. This is primarily packaging and extension, not greenfield.

---

## Existing Infrastructure Inventory

Before building anything new, here's what already works:

| Component | Location | What it does | Reuse for |
|-----------|----------|-------------|-----------|
| **Context extraction** | `crates/nazec/src/context.rs` | Extracts components, server fns, state, data sources, pages, guards, themes, models, prompts as JSON | Phase A (~60% of manifest) |
| **Action executor** | `crates/nazec/src/exec.rs` | Pure-function state machine: `init_state()`, `execute_action()`, `evaluate_expr()`, `resolve_nodes()` | Phase C (core runtime) |
| **Test runner** | `crates/nazec/src/test_runner.rs` | Headless simulation: click, fill, navigate, assert text visibility, layout computation | Phase C (interaction model) |
| **Content extraction** | `crates/nazec/src/seo.rs` | `extract_text_content()` walks RenderNode tree, extracts `__text` props | Phase A (content section) |
| **Server function eval** | `crates/nazec/src/server_fns.rs` | Executes server functions: HTTP fetch, SQL queries, expression evaluation with auth header forwarding | Phase C (async execution) |
| **SSR server** | `crates/nazec/src/serve.rs` | Production headless rendering: fresh state per request, guard evaluation, route matching, server fn pre-evaluation | Phase C (execution patterns) |
| **Binary serialization** | `crates/naze-ir/src/lib.rs` | Custom binary format for RenderTree — all 15+ fields serialized/deserialized | Phase B (strip and re-serialize) |
| **Build pipeline** | `crates/nazec/src/build.rs` | Writes dist/: `app_data.bin`, WASM, JS, HTML. Clear insertion point at line 192 | Phase A (add manifest output) |
| **HTML template** | `crates/nazec/src/build.rs:18` | Already has `<link rel="alternate" type="application/naze">` for binary discovery | Phase A (add manifest link) |
| **Package registry** | `crates/naze-registry/` | Axum + SQLite server: publish, search, download packages | Phase D (extend with manifest indexing) |
| **Layout engine** | `crates/naze-layout/` | Pure layout computation, no rendering dependency | Phase C (element positioning for queries) |

---

## Phase A: Manifest Generation

**Goal:** Every `nazec build` emits `naze-manifest.json` alongside `app_data.bin` — a machine-readable JSON document describing the application's data model, interactions, content, and external dependencies.

**Why first:** This is the highest-leverage deliverable. A single JSON file enables agent discovery, pre-execution auditing, and structural search — without requiring any changes to how apps are built or deployed. Every existing Naze app gains agent discoverability for free.

### What the manifest contains

```json
{
  "$schema": "https://naze.dev/manifest/v1.json",
  "name": "Todo App",
  "version": "0.1.0",
  "content_hash": "sha256:a3f2b8c1...",

  "content": {
    "text": ["Todo App", "What needs to be done?", "Add", "All", "Active", "Done"],
    "headings": [{ "level": 1, "text": "Todo App" }],
    "inputs": [{
      "bind": "new-task",
      "placeholder": "What needs to be done?",
      "validation": { "required": true, "min_length": 2, "max_length": 100 }
    }]
  },

  "state": {
    "tasks": { "type": "list", "initial": [
      {"text": "Learn Naze", "done": false}
    ]},
    "new-task": { "type": "text", "initial": "" },
    "filter-mode": { "type": "text", "initial": "all" }
  },

  "computed": [
    { "name": "visible-tasks", "depends_on": ["tasks", "filter-mode"] }
  ],

  "actions": [
    { "event": "click", "action": "append", "target": "tasks", "label": "Add" },
    { "event": "click", "action": "remove", "target": "tasks", "label": "Delete" },
    { "event": "click", "action": "set", "target": "filter-mode", "label": "Filter" },
    { "event": "click", "action": "set-theme", "label": "Switch theme" }
  ],

  "pages": [
    { "path": "/", "title": "Todo App" }
  ],

  "themes": ["light", "dark"],

  "external": {
    "endpoints": [],
    "server_functions": ["list-tasks", "add-task", "delete-task"]
  }
}
```

### Architecture

```
RenderTree (already computed at build time)
  │
  ├─ state declarations ──────────→ manifest.state
  ├─ computed declarations ───────→ manifest.computed
  ├─ data declarations ───────────→ manifest.external.endpoints
  ├─ server_functions ────────────→ manifest.external.server_functions
  ├─ pages ───────────────────────→ manifest.pages
  ├─ themes ──────────────────────→ manifest.themes
  └─ root (RenderNode tree) ──────→ manifest.content (text, headings, inputs)
                                  → manifest.actions (event handlers)
```

No new data sources. The manifest is a different **projection** of the same RenderTree that already exists.

The `content_hash` field is a SHA256 of the Layer 1 content (state + computed + server functions + data bindings). It enables change detection: external services (Phase E) compare hashes to determine whether a manifest needs re-fetching.

### Key decisions

- **Always generated** — no opt-out flag. The manifest is ~1-3KB and the value of universal discoverability outweighs the trivial size cost. Developers don't think about it.
- **Separate module** — new `manifest_gen.rs` rather than extending `context.rs`. Context extraction is an internal dev tool; manifest generation is a build artifact with a schema contract.
- **Content extraction** extends `seo.rs` patterns — walk the RenderNode tree, extract by `kind` (heading, text, input), pull props (placeholder, validation, bind).
- **Action labeling** infers labels from sibling/child text content (e.g., a button's `__text` prop becomes the action label). Falls back to action target name.

### Scope

- New module: `manifest_gen.rs` (~400 lines)
- Content/action extraction helpers (~150 lines)
- Build pipeline integration (~30 lines)
- HTML template update (1 line)
- Tests (~200 lines)
- **Total: ~780 lines**

### Testing

- Unit tests: manifest generation from hand-crafted RenderTrees
- Integration: `nazec build` on `examples/apps/todo` → validate `dist/naze-manifest.json` contains expected state, actions, content
- Schema: validate output against a JSON Schema definition
- Regression: manifest for all 5 example apps in CI

---

## Phase B: Headless Binary

**Goal:** `nazec build --headless` emits a minimal binary containing only Layer 1 (state, computed values, server functions, data bindings) — no UI tree, no themes, no animations. For agent-to-agent communication where presentation is pure overhead.

**Why:** The todo app binary is ~7KB. Roughly 85-93% is presentation (UI nodes, theme tokens, text styling). A headless binary would be ~500-800 bytes — small enough to discover, evaluate, and compose thousands of services in milliseconds.

### Architecture

```
nazec build                → app_data.bin  (layers 1+2+3, ~7KB)   → browsers, humans
nazec build --manifest     → naze-manifest.json (layers 1+2, ~2KB) → agent discovery
nazec build --headless     → headless.bin  (layer 1 only, ~500B)  → agent computation
```

All three are projections of the same RenderTree. They can't drift because they're derived from the same compilation.

### Key decisions

- **Strip, don't rebuild** — the compiler always produces the full RenderTree. Headless mode strips fields post-codegen rather than modifying the codegen pass. Simpler, no risk of divergence.
- **What stays:** state declarations, computed values, server function signatures, data bindings (URLs, methods), guards (preconditions).
- **What goes:** root node tree (UI), themes, animations, text content, layout props.
- **Output format:** same binary serialization as `app_data.bin` but with empty root/themes. The existing deserializer handles empty fields already.
- **Headless `.naze` files** — source files with no `app` block. Just `state`, `computed`, `server function`, and `data` declarations. The grammar already supports top-level declarations; the compiler just needs to not require an `app` block when `--headless` is set. This enables a lighter option for pure API-to-API cases (see "Adoption Without Rebuild" below), though the recommended adoption path is a small full app — because it also gives humans a usable interface.

### Scope

- `RenderTree::into_headless()` method in naze-ir (~30 lines)
- CLI flag addition (~5 lines)
- Build pipeline branch (~40 lines)
- Grammar relaxation for headless files (~20 lines)
- Tests (~100 lines)
- **Total: ~200 lines**

### Testing

- Unit: `into_headless()` produces tree with empty root, preserved state/computed/server_functions
- Size: headless todo binary < 1KB
- Round-trip: deserialize headless binary, verify state schema matches full binary
- Integration: `nazec build --headless` on example apps

---

## Phase C: Agent Crate (`naze-agent`)

**Goal:** A standalone Rust crate that loads a Naze binary and provides a programmatic API for AI agents to inspect application structure, execute actions, query state, and observe results — without rendering anything.

**Why:** The binary format enables agent interaction, but currently the execution engine (`exec.rs`) is internal to `nazec` and not exposed as a library. The agent crate makes it a public API.

### API surface

```rust
use naze_agent::Agent;

// Load and inspect
let agent = Agent::load(bytes)?;
let state = agent.state();                    // &HashMap<String, RenderValue>
let actions = agent.available_actions();       // Vec<ActionDesc>
let server_fns = agent.server_functions();     // Vec<ServerFnDesc>
let manifest = agent.manifest();               // NazeManifest (Phase A output)

// Execute actions and observe
let changed = agent.execute(Action::Append {
    target: "tasks",
    item: obj!({ "text": "Buy groceries", "done": false }),
})?;
assert!(changed);
assert_eq!(agent.state().get("tasks").unwrap().as_list().len(), 4);

// Query state
let count = agent.evaluate("tasks | count")?;
let filtered = agent.evaluate("tasks | filter done == true")?;

// Interact like a user
agent.click("Add")?;                          // Find element by text, execute handlers
agent.fill("new-task", "Build an app")?;      // Set input value
agent.navigate("/settings")?;                 // Change page

// Server function execution (async)
let result = agent.call_server_fn("list-tasks", &[]).await?;

// Execution trace for verifiability
let trace = agent.trace();
// [{ action: "append", target: "tasks", before: [...], after: [...] }]
```

### Architecture

```
naze-agent (new crate)
  │
  ├─ Agent struct
  │    ├─ RenderTree (immutable, from deserialized binary)
  │    ├─ state: HashMap<String, RenderValue> (mutable)
  │    ├─ trace: Vec<TraceEntry> (append-only log)
  │    └─ layout: Option<LayoutTree> (computed on demand)
  │
  ├─ Delegates to existing code:
  │    ├─ naze_ir::deserialize() ── binary loading
  │    ├─ exec::init_state() ────── state initialization
  │    ├─ exec::execute_action() ── action execution
  │    ├─ exec::evaluate_expr() ── expression evaluation
  │    ├─ exec::resolve_nodes() ── conditional/loop resolution
  │    ├─ naze_layout::compute_layout() ── element positioning
  │    └─ server_fns::evaluate_server_fn() ── server calls
  │
  └─ New functionality:
       ├─ Element selection (by text, kind, structural query)
       ├─ Execution tracing (before/after state snapshots)
       ├─ Manifest generation from loaded tree
       └─ Expression parsing (string → IrExpression for queries)
```

### Key decisions

- **Wraps, doesn't duplicate** — the agent crate re-exports and delegates to existing `exec`, `naze_ir`, and `naze_layout` code. The execution engine stays in one place.
- **Refactor requirement** — `exec.rs` functions are currently `pub(crate)` in `nazec`. They need to move to a shared location (either `naze-ir` or a new `naze-exec` crate) so both `nazec` and `naze-agent` can use them. This is the main structural change.
- **Sync-first** — the core API is synchronous (state machine operations are instant). Server function calls are the only async boundary.
- **Trace as first-class** — every action records before/after state. This enables the verifiable execution model from AGENT_RUNTIME.md: anyone can replay a trace against the same binary and verify results.

### Exposure roadmap (progressive)

| Interface | Priority | Notes |
|-----------|----------|-------|
| Rust crate (`naze-agent`) | Phase C core | Public API, published to crates.io |
| CLI (`nazec agent`) | Phase C stretch | Shell scripting: `nazec agent load app.bin --execute "append tasks {text: 'test'}"` |
| Python (PyO3) | Post-C | AI/ML ecosystem integration, notebook workflows |
| MCP server | Post-C | Direct integration with Claude, ChatGPT, and other AI assistants |
| WASI module | Future | Sandboxed execution in any WASI runtime |

### Scope

- New crate scaffolding + Agent struct (~200 lines)
- Delegation wrappers (~150 lines)
- Element selection (~150 lines)
- Execution tracing (~100 lines)
- Expression string parsing (~100 lines)
- Refactor: extract `exec.rs` to shared crate (~100 lines of moves, minimal new code)
- Tests (~300 lines)
- **Total: ~1,100 lines** (plus ~100 lines of refactoring moves)

### Testing

- Unit: load binary → inspect state → execute action → verify state change
- Integration: full interaction sequence on todo app binary (add, remove, filter, clear)
- Trace: replay recorded trace, verify deterministic results
- Round-trip: `nazec build` → load in agent → state matches `nazec context` output

---

## Phase D: Discovery & Registry

**Goal:** Agents can find Naze services via well-known URLs and registry search, evaluate them pre-execution via manifests, and make trust decisions based on structural analysis.

**Why:** A manifest sitting in `dist/` is useful but passive. Discovery makes it active — agents find services by capability, not by URL. This is the shift from "search engines index text" to "registries index capabilities."

### Discovery mechanisms

**1. Per-domain (like robots.txt)**

Any domain can serve its manifest at `/.well-known/naze-manifest.json`. An agent that already knows about `example.com` fetches it directly. Convention-based, no registry needed.

```
GET https://example.com/.well-known/naze-manifest.json
→ 200 OK, application/json
→ { "name": "Example Service", "state": { ... }, "external": { ... } }
```

**2. Registry search (extends existing naze-registry)**

The existing `naze-registry` crate (Axum + SQLite) already handles package publishing and search. Extend it to:

- Store manifest JSON alongside package tarballs
- Index manifest fields (state variable names, server function signatures, capability tags)
- Support structural queries: "find services with state matching `{price: number, quantity: number}`"

```
GET /api/packages/search?capability=payment&state_has=cart,total
→ [{ name: "stripe-checkout", manifest_url: "...", trust_score: 92 }]
```

**3. Trust scoring**

Automated pre-execution auditing based on manifest analysis:

| Signal | Score impact |
|--------|-------------|
| Single external domain | +20 (focused) |
| Multiple tracking domains | -30 per domain |
| Device API requests (camera, location, contacts) | -15 per API (unless expected for category) |
| State variables collecting PII (email, phone, ssn) | -10 per field (flag for review) |
| Server functions calling declared domain only | +10 |
| Cross-domain server function calls | -25 |
| Manifest `$schema` present and valid | +5 |

Agents use trust scores to make autonomous decisions: "Is this service safe to execute without human approval?"

### Key decisions

- **Registry-first, not crawler** — services publish manifests to the registry (push model), not crawled from the web (pull model). This avoids the stale-index problem of search engines.
- **Structural search, not keyword search** — queries match against typed state schemas and function signatures, not text content. "Find services with a `search(query: text)` function returning a list" is a typed query, not keyword matching.
- **Trust scoring is advisory** — the registry computes scores, but the agent makes the final decision based on its own policies. No central authority blocks services.

### Scope

- Registry: manifest storage + indexing (~200 lines)
- Structural search API (~150 lines)
- Trust scoring algorithm (~100 lines)
- Well-known URL documentation (~20 lines)
- Tests (~200 lines)
- **Total: ~670 lines**

### Testing

- Unit: trust scoring on known-good and known-suspicious manifests
- Integration: publish package with manifest → search by capability → verify result
- Well-known: serve manifest at convention URL, verify agent fetch

---

## Phase E: Discovery Service

**Goal:** A standalone central directory (`discover.naze.dev`) that Naze services announce to at build time. Agents query it to find services by structural capability — without npm-style publish ceremony or web crawling. The discovery service indexes any domain serving a valid manifest. The site does not need to be fully built with Naze — a small wrapper app pointing server functions at existing API endpoints is sufficient (see "Adoption Without Rebuild" below).

**Why:** Phase D extends the existing registry for packages that are explicitly published. But most Naze apps are deployed to their own domains and never published as packages. The discovery service closes this gap: any deployed Naze app can be found by agents, as long as it opts in at build time.

**What this is NOT:**
- **Not npm** — no publish command, no tarballs, no package ownership. Apps announce their domain; the service fetches the manifest directly.
- **Not a crawler** — no scraping, no link following, no indexing delay. Apps push an announcement; the service pulls the manifest on demand.

### How it works

```
1. Developer builds:  nazec build
2. Build announces:   POST discover.naze.dev/announce { domain: "myapp.com", manifest_hash: "sha256:a3f2b8c1..." }
3. Service compares:  hash differs from last known? → fetch manifest
4. Service fetches:   GET myapp.com/.well-known/naze-manifest.json
5. Service indexes:   extract state schema, server fn signatures, capability tags
6. Agent queries:     GET discover.naze.dev/search?state_has=cart,total&has_fn=checkout
```

### Configuration

Opt-in via `naze.toml`:

```toml
[discovery]
enabled = true
registry = "https://discover.naze.dev"
domain = "myapp.com"
```

When `discovery.enabled = true`, `nazec build` sends a lightweight POST after successful compilation. The announcement contains only the domain and the `content_hash` from the manifest (Phase A). No source code, no binary, no manifest content leaves the build machine.

### Architecture

```
nazec build
  │
  ├─ compile + emit dist/ (existing)
  │
  └─ POST /announce { domain, manifest_hash }
       │
       discover.naze.dev
         │
         ├─ Compare manifest_hash to last known
         │    └─ unchanged? → skip (no-op)
         │    └─ changed?   → fetch manifest from domain
         │
         ├─ GET domain/.well-known/naze-manifest.json
         │    └─ validates manifest schema
         │    └─ verifies domain matches announcement
         │
         ├─ Structural indexing:
         │    ├─ state variable names + types
         │    ├─ server function signatures
         │    ├─ capability tags (from manifest)
         │    ├─ external endpoint domains
         │    └─ trust score (Phase D algorithm)
         │
         └─ Searchable via agent query API
```

### Agent query API

```
GET /search?state_has=cart,total&has_fn=checkout
→ [
    {
      "domain": "shop.example.com",
      "name": "Example Shop",
      "manifest_url": "https://shop.example.com/.well-known/naze-manifest.json",
      "trust_score": 88,
      "capabilities": ["payment", "cart"],
      "last_seen": "2026-02-20T14:30:00Z"
    }
  ]

GET /search?capability=weather&state_has=location
GET /search?has_fn=search&fn_param=query
```

### Key decisions

- **Hash-based change detection** — the discovery service only fetches manifests when the `content_hash` changes. This keeps the service lightweight even with thousands of registered domains. The hash is the same `content_hash` field added to the manifest in Phase A.
- **Domain as identity** — no usernames, no package names, no ownership transfers. The domain IS the identity. If `myapp.com` announces, the service fetches from `myapp.com`. This also provides implicit domain verification: a malicious announcer can't claim a domain they don't control because the service fetches the manifest from that domain directly.
- **Announcement is fire-and-forget** — the build doesn't wait for indexing. The POST returns 202 Accepted immediately. If the service is down, the build succeeds anyway.
- **Structural indexing reuses Phase D's trust scoring** — the same algorithm that scores registry packages scores discovered services.
- **Separate from naze-registry** — the discovery service is a standalone Axum app, not an extension of the package registry. Different concerns: registry manages package tarballs and versions; discovery indexes live deployed services.

### Scope

- Discovery service skeleton (Axum + SQLite) (~200 lines)
- Announcement handler + hash comparison (~100 lines)
- Manifest fetcher + schema validation (~150 lines)
- Structural indexer (state, fns, capabilities → DB) (~200 lines)
- Agent query API (~150 lines)
- Trust scoring integration (reuse Phase D) (~50 lines)
- `nazec build` announcement integration (~50 lines)
- `naze.toml` config parsing (~30 lines)
- Tests (~200 lines)
- **Total: ~1,130 lines**

### Testing

- Unit: hash comparison logic (changed vs unchanged)
- Unit: structural indexing from sample manifests
- Integration: announce → fetch → index → query round-trip
- Edge cases: service down during build (graceful failure), invalid manifest on domain, hash unchanged (skip fetch)
- Trust: scoring consistency between registry (Phase D) and discovery service

---

## Adoption Without Rebuild

The discovery service (Phase E) and manifest (Phase A) are designed around Naze-native apps. But ecosystem value scales with discoverable services. Existing sites — Zillow, Redfin, a local bakery — should be able to expose a Layer 1 API and join the discovery network without rebuilding their entire frontend.

The solution requires no special modes or new concepts: **build a tiny Naze app that wraps your existing API.** A 20-30 line `.naze` file that points server functions at existing endpoints. The Naze toolchain handles Layer 1 extraction, manifest generation, content hashing, and discovery registration automatically.

### Concrete example — a real estate API wrapper

```naze
app "Zillow Search"
  state
    location ""
    price-min 0
    price-max 1000000
    results []

  server function search-listings
    fetch "https://api.zillow.com/v2/listings"
      method "GET"

  column gap 16
    text "Search Listings" size 24
    input bind location placeholder "Location"
    row gap 8
      input bind price-min placeholder "Min price"
      input bind price-max placeholder "Max price"
    button
      text "Search"
      on click
        call search-listings
        set results result
    each results as listing
      text listing.address
```

~25 lines. `nazec build` with discovery enabled gives you:

- **A manifest** — Layer 1 exposed as JSON. Agents discover you.
- **A binary** — agents load and execute against your real endpoints.
- **A usable human interface** — the simple UI you wrote.
- **Automatic registration** with the discovery service.

### What the toolchain does automatically

1. **Compiler** extracts Layer 1 (state schema, server function signatures) into `naze-manifest.json` (Phase A)
2. **Build** computes `content_hash` (SHA256 of Layer 1)
3. **With `[discovery] enabled = true`** in `naze.toml`, announces to `discover.naze.dev` (Phase E)
4. **Discovery service** indexes capabilities — agents find you via structural search

### What agents get

- **Discovery:** `GET discover.naze.dev/search?has_fn=search-listings&state_has=location`
- **Execution:** `naze-agent` loads the binary, calls server fns against real endpoints (Phase C)
- **Interaction:** `agent.fill("location", "Seattle")`, `agent.click("Search")`, read `agent.state().get("results")`

### Why an existing site would want this

**Agent-mediated traffic is a new distribution channel.**

Today, users find services through search engines (keyword matching) and app stores (category browsing). Both are human-driven. AI agents change this: the agent finds services by capability, evaluates them structurally, and uses them on the user's behalf. The user says "find me apartments in Seattle under $500k" — the agent discovers your service, executes against it, returns results. If your API isn't structurally discoverable, you're invisible to this channel.

**Six concrete benefits:**

1. **Agent discoverability** — Findable by capability, not just brand awareness or SEO rank. No developer portal signup on the agent's side, no API key negotiation.

2. **Self-maintaining SDK** — The .naze file IS the client library. Calls real endpoints directly, can't drift from your API. Compare: OpenAPI specs (always stale), npm packages (always lagging), developer portal docs (always wrong). One 25-line file = documentation + SDK + working demo.

3. **Trust scoring as competitive advantage** — Manifest exposes data flows transparently. Agents compute trust scores (Phase D). Clean services score high, agents prefer them. Transparency becomes a market signal.

4. **Pre-execution auditing** — Agents verify before executing. Impossible with HTML/JS sites (must execute JS). Impractical with REST APIs (must read docs and trust them). Wrapper makes your service verifiably safe → agents use it autonomously.

5. **Zero-cost API for businesses without one** — Most small businesses don't have APIs. A 25-line wrapper gives them a machine-queryable interface automatically.

6. **AI can write the wrapper for you** — σ=1 design means an AI generates the wrapper from an API description in seconds. Essentially free.

**What it costs:**

| Cost | Magnitude | Context |
|------|-----------|---------|
| Writing the wrapper | ~25 lines, one-time | AI can generate it |
| Maintenance when API changes | Update endpoint URLs or params | Same as any API client |
| Build step | `nazec build` with discovery config | Seconds in CI/CD |
| Ecosystem bet | Agent adoption is early | Near-zero cost = free option |

**The asymmetry:** Near-zero downside, potentially significant upside, zero risk to existing infrastructure. A free option on a new distribution channel.

### Branding and attribution

The obvious objection: "If agents strip my data out of context and present it generically, I've given away my competitive advantage for free."

**Four layers of protection:**

**1. Manifest carries brand identity.**

The manifest already includes `name`. Extend with a `branding` section:
```json
{
  "branding": {
    "name": "Zillow",
    "logo": "https://zillow.com/logo.svg",
    "url": "https://zillow.com",
    "attribution": "Powered by Zillow. Listing data © Zillow Group.",
    "terms": "https://zillow.com/terms"
  }
}
```
Agents that display results include attribution. Same model as Google Maps embed ("Map data © Google") and news licensing ("Source: Reuters").

**2. Server function responses carry attribution in-band.**

Zillow's `search-listings` server function returns data from their API. Nothing stops them from including attribution fields in every response:
```json
{
  "listings": [...],
  "source": "Zillow",
  "source_url": "https://zillow.com/homes/Seattle",
  "logo_url": "https://zillow.com/logo.svg",
  "legal": "© 2026 Zillow Group. See terms."
}
```
The data carries its own attribution regardless of how it's displayed. This is how RSS feeds, API responses, and syndicated content work today.

**3. The full app IS the branded experience.**

The recommended adoption path is a full app (not headless). The .naze wrapper includes a UI — with your brand's colors, logo, layout, and copy. When an agent surfaces results, it can offer: "View full results on Zillow" → opens the Naze app or the existing site. The wrapper is a funnel to your brand, not a replacement for it.

Think of it as an API that happens to have a demo UI attached. The agent uses the API, but the human can always click through to the branded experience.

**4. Trust scoring incentivizes attribution compliance.**

The discovery service tracks whether agents respect attribution terms. Agents that consistently attribute sources correctly get higher trust scores themselves. This creates a reciprocal incentive: services want high trust scores (so agents choose them), and agents want high trust scores (so services allow them). Attribution compliance becomes a market norm, not just a legal requirement.

**The honest trade-off:**

This is the same tension that exists everywhere data is syndicated. Google surfaces your content in featured snippets — you lose some direct traffic but gain visibility. Amazon lists your product alongside competitors — you lose exclusivity but gain distribution. The agent economy is the same trade-off at a new scale.

The question isn't "will agents perfectly preserve my brand?" — they won't, just as Google doesn't. The question is "is agent-mediated distribution worth the brand dilution?" For most services, the answer is yes — because the alternative is invisibility. Users who ask their agent "find me apartments" will use services the agent can discover. Services without wrappers aren't in that pool.

**What you control:**
- Your data (server functions return what you want, including attribution)
- Your terms (manifest declares usage requirements)
- Your full experience (the .naze app or your existing site, one click away)

**What you don't control:**
- How agents present your data to users (same as today with search engines)
- Whether agents respect attribution terms (mitigated by trust scoring)

### Agent bootstrap

The plan above describes a discovery service and registries — but how does an AI agent learn about them in the first place? Three paths, depending on the agent's environment:

**1. The Naze browser (primary path).** The dedicated browser described in [ROADMAP.md](ROADMAP.md) ships with `discover.naze.dev` built in. Discovery is not something the agent configures — it's part of the app the user is already running. The browser also bundles the Naze grammar (GBNF from `nazec grammar --format gbnf`), language docs, and example corpus, so agents operating within it have native Naze comprehension without web lookup. And the browser is where the user manages credentials: API keys, OAuth tokens, model preferences — the credential wallet from [AGENT_RUNTIME.md](AGENT_RUNTIME.md). This connects Phase E (discovery) to the "Future: Dedicated Browser" in the roadmap.

**2. Headless/programmatic agents.** The `naze-agent` crate (Phase C) ships with the default discovery URL (`discover.naze.dev`) as a compile-time constant. Any Rust program using the crate can query the discovery service immediately. No configuration needed for the default; override via `AgentConfig` for private registries.

**3. LLM-based agents via MCP.** The MCP server (Post-C in the exposure roadmap) registers Naze discovery as a tool. An AI assistant with the Naze MCP server configured can search for services, load binaries, and execute actions — all through the standard MCP tool-calling interface. The discovery URL is part of the server configuration.

All three paths converge on the same discovery service. The browser is the most complete environment (discovery + credentials + language spec + rendering), but headless and MCP paths ensure agents don't need the browser to participate.

### Key points

- **No changes to the existing site.** The Naze app sits alongside it — a thin wrapper that describes the API boundary. Server functions point to real endpoints. The existing site doesn't know or care about Naze.
- **The app IS human-readable API documentation.** Anyone can read the `.naze` source and understand what the API does, what parameters it takes, and how the results are structured.
- **Natural on-ramp.** Developers learn Naze by wrapping something they already have. Later, they may build full Naze frontends — but discovery value starts immediately.
- **Phase B enables an even lighter option** for pure API-to-API cases: `.naze` files with no `app` block, producing only Layer 1 content. But the recommended adoption path is a small full app — because it also gives humans a usable interface.

---

## Phase Dependencies

```
Phase A (Manifest) ←── prerequisite for everything
  │
  ├──→ Phase B (Headless) ── independent of C/D/E, but A's manifest format informs headless output
  │
  ├──→ Phase C (Agent Crate) ── uses manifest for inspection, needs exec.rs refactor
  │       │
  │       └──→ Post-C: PyO3, MCP server, CLI (depend on stable agent API)
  │
  ├──→ Phase D (Registry) ── indexes manifests, needs registry (already exists)
  │       │
  │       └──→ informational for E (trust scoring algorithm reused)
  │
  └──→ Phase E (Discovery Service) ── needs manifest + content_hash from A
                                       reuses trust scoring from D

Recommended order: A → B → C → D → E
  - A is highest leverage (every app gains discoverability)
  - B is smallest scope (~200 lines) and validates the layer separation
  - C is the main engineering effort but has clear patterns to follow
  - D builds on A + existing registry, can start after A is stable
  - E builds on A (manifest + content_hash) and reuses D's trust scoring
```

---

## Risks & Open Questions

### Auth model for headless binaries

AGENT_RUNTIME.md envisions auth declarations in the manifest (`"auth": { "type": "oauth2", ... }`). This requires:
- Schema design for auth declarations in manifest
- Agent credential wallet concept (stores tokens per service)
- How auth integrates with existing `DataDecl` headers

**Current state:** `DataDecl` already supports `headers: Vec<(String, RenderValue)>` with interpolation. Auth headers can be passed today, but there's no declarative auth schema.

**Decision needed:** Is declarative auth in scope for Phase A, or deferred?

### Server function trust boundary

The manifest declares server function **signatures** but not their implementation. An agent can verify client-side behavior is safe (all data flows visible), but the server is opaque. This is the same limitation as any client-server architecture.

**Mitigation:** Domain verification (server calls should match binary source domain), response size monitoring, and the manifest makes the trust boundary explicit rather than hidden.

### Headless grammar changes

Phase B proposes `.naze` files without `app` blocks. This requires grammar changes — currently `app` is expected as a top-level rule. The change should be additive (headless files are valid `.naze`, regular files remain valid) but needs careful grammar design to avoid ambiguity.

### `exec.rs` extraction for Phase C

The execution engine is currently `pub(crate)` inside `nazec`. Making it available to `naze-agent` requires either:
1. Moving it to `naze-ir` (adds native-only dependencies to a both-target crate)
2. Creating a new `naze-exec` crate (adds a workspace member)
3. Making `nazec` a library + binary crate (adds complexity)

**Recommended:** Option 2 — new `naze-exec` crate. Clean separation, minimal disruption.

### Manifest schema versioning

The manifest needs a `$schema` field and versioning strategy from day one. Adding fields is non-breaking; removing or renaming is breaking. Semver the schema URL: `https://naze.dev/manifest/v1.json`.

### Discovery service hosting (Phase E)

Who runs `discover.naze.dev`? Options: (1) Naze project hosts it centrally, (2) self-hostable with a reference instance, (3) federated model where multiple discovery services exist. Central hosting is simplest for adoption but creates a single point of failure.

### Domain verification in discovery (Phase E)

The announcement contains a domain, and the service fetches the manifest from that domain — which implicitly verifies ownership (you can't serve a manifest on a domain you don't control). However, this doesn't prevent announcement spam: a malicious actor could announce domains they don't own, causing the service to make outgoing requests. Rate limiting on announcements (per IP, per domain) mitigates this.

### Discovery bootstrap

How does an agent first learn about the discovery service? The "Agent bootstrap" section above (under "Adoption Without Rebuild") outlines three paths: the Naze browser (embedded discovery), the `naze-agent` crate (default URL), and MCP (tool registration). The dedicated browser is the primary answer — see [ROADMAP.md](ROADMAP.md) "Future: Dedicated Browser" for how it connects discovery, credential management, and language spec into a single environment. For agents outside the browser, the default discovery URL in the crate and MCP server configurations provide the entry point.

### Discovery service privacy (Phase E)

Should manifests indexed by the discovery service be public by default? The manifest is already served at a well-known URL on the domain, so it's inherently public. But the discovery service makes it *searchable*, which is a different privacy posture. Consider: opt-in categories (public/unlisted), allow domains to request de-indexing.

---

## Total Scope Summary

| Phase | New code | Tests | Total | Key deliverable |
|-------|----------|-------|-------|-----------------|
| A: Manifest | ~580 lines | ~200 lines | ~780 lines | `naze-manifest.json` in every build |
| B: Headless | ~100 lines | ~100 lines | ~200 lines | `nazec build --headless` |
| C: Agent Crate | ~800 lines | ~300 lines | ~1,100 lines | `naze-agent` on crates.io |
| D: Discovery | ~470 lines | ~200 lines | ~670 lines | Structural search + trust scoring |
| E: Discovery Service | ~930 lines | ~200 lines | ~1,130 lines | Build-time announcement + capability indexing |
| **Total** | **~2,880 lines** | **~1,000 lines** | **~3,880 lines** | |

For context: the entire Naze codebase is ~25,000 lines across 11 crates. This adds ~15% new code to deliver the agent runtime platform described in AGENT_RUNTIME.md.

---

*This plan implements the vision from [AGENT_RUNTIME.md](AGENT_RUNTIME.md). Each phase is independently shippable and provides immediate value. Phase A is the recommended starting point — it requires no breaking changes, adds ~1KB to build output, and makes every Naze app agent-discoverable with zero developer effort. Phase E completes the discovery story: any deployed Naze app can be found by agents through build-time announcement, without publish ceremony or web crawling.*
