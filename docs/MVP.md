# MVP: The Local Naze Ecosystem

The Naze ecosystem is five components working together: a language, a discovery network, a browser, a seeding pipeline, and a training data flywheel. This MVP proves the full loop runs on a single machine — no cloud infrastructure, no external servers (except the public APIs that wrapped services call out to).

The shift from Phase 1's "can we render rectangles" to this MVP's question: **does the ecosystem loop work?** Can an agent generate apps, publish them to a discovery network, have another agent (or human) discover and render them, and produce training data that improves the next generation of apps?

Everything described here runs locally. The discovery server is SQLite-backed. The browser connects to `localhost`. The training data exports to a file on disk. The only external dependency is the internet — because the sample apps wrap real public APIs (weather, pets, jokes, etc.) and those APIs live on someone else's server.

## The Loop

```
                         ┌─────────────────────────────┐
                         │       Public APIs            │
                         │  (weather, pets, jokes, ...) │
                         └──────────┬──────────────────┘
                                    │ OpenAPI specs
                                    ▼
┌──────────────┐   wrap    ┌──────────────┐  announce   ┌───────────────────┐
│  OpenAPI     │─────────→│    nazec     │───────────→│ Discovery Server  │
│  Specs       │  codegen  │   compiler   │  Layer 1+  │    :8889          │
│  (YAML/JSON) │  .naze    │              │  manifest  │                   │
└──────────────┘  files    └──────────────┘            │  SQLite + Trust   │
                                                        │  + Dashboard      │
                           ┌───────────────────────────│                   │
                           │  fetch service list       │                   │
                           │  + app binaries           │                   │
                           ▼                           │                   │
                    ┌──────────────┐                    │                   │
                    │  Naze        │                    │                   │
                    │  Browser     │                    │                   │
                    │  (native)    │                    └─────────┬─────────┘
                    └──────────────┘                              │
                                                                 │ export
                                                                 ▼
                                                        ┌─────────────────┐
                                                        │ training.jsonl  │
                                                        │ (source + trust │
                                                        │  + observations)│
                                                        └─────────────────┘
```

**The demo flow:**

1. Start the discovery server locally (port 8889)
2. Run `nazec wrap` on a set of public OpenAPI specs — each generates a `.naze` app with server functions wrapping the API endpoints, compiles it, splits the binary into 3 layers, and announces Layer 1 (headless) + manifest to the discovery server
3. Open the Naze Browser, which connects to the local discovery server, lists discovered services with trust scores, and renders any selected app
4. Export training data — .naze source paired with trust scores, observations, and composition patterns — as JSONL

Each step feeds the next. Wrapping seeds the network. The network scores and indexes services. The browser renders them. The export captures the whole graph as training signal.

---

## The Five Components

### 1. Discovery Server (Reference Implementation) — ~90% done

A standalone Axum HTTP server backed by SQLite that implements the discovery network API. Runs locally, no external dependencies.

**This is a reference implementation.** The HTTP API (26 JSON endpoints) is the contract — not the storage backend. The reference impl uses SQLite because it's zero-config and single-binary, but the server is built on 6 pluggable trait interfaces (`TrustScorer`, `CapabilityMatcher`, `CapabilityExtractor`, `IdentityVerifier`, `FederationSync`, `StorageBackend`) that any backend can implement. A production discovery network might use:

- **Storage:** Postgres, IPFS (content-addressable, distributed), a DHT, S3 for binary storage
- **Trust scoring:** AI semantic analysis of manifest behavior, graph-based trust propagation through composition chains, RL from agent feedback
- **Capability matching:** Vector embedding similarity (searching "order" finds "purchase"), LLM-powered semantic matching
- **Identity:** DIDs (decentralized identifiers), cryptographic manifest signatures, OAuth/OIDC
- **Federation:** Gossip protocol, IPFS pubsub, CRDTs for conflict-free merging, blockchain-anchored sync proofs

The reference impl proves the API surface works. Alternative backends implement the same 26 endpoints with the same request/response shapes. See [DISCOVERY_NETWORK_REFERENCE_IMPL.md](DISCOVERY_NETWORK_REFERENCE_IMPL.md) for the full trait architecture and webhook escape hatch (any trait can delegate to an external HTTP service in any language).

**What exists (4,524 LOC, 61 tests, all passing):**

| Capability | Status | Details |
|---|---|---|
| HTTP API | 23/24 endpoints | Only federation sync returns 501 (intentional) |
| SQLite storage | 11 tables | services, capabilities, trust_scores, trust_profiles, observations, compositions, provenance, service_versions, peers, pattern_templates |
| Trust scoring | Complete | `SimpleScorer` with 6 parametric profiles (default, healthcare, ecommerce, iot, finance, education). Manifest signal analysis + dynamic adjustments from observations |
| Capability matching | Complete | SQL INTERSECT with AND semantics, LIKE patterns, type filtering, preference scoring |
| Capability extraction | Complete | JSON manifest parsing → state_field, server_function, action, data_source, model_field |
| Dashboard | 6 pages | Overview, services, trust dynamics, composition patterns, observations, interactive test console. Live JS, auto-refresh, professional styling |
| Observations | Complete | Usage, discovery, flag events. Auto-deactivation at 5 flags. Staleness decay |
| Compositions | Complete | Service combination tracking with frequency, first/last seen, pattern promotion |
| Export | Partial | `GET /export?since=...` returns service metadata JSON array — no training data format |

**What's new for MVP (~10%):**
- Training data export endpoint (`/export/training`) — JSONL pairing .naze source with trust scores, observations, compositions
- Store raw `.naze` source alongside manifests during announce (currently only manifest JSON is stored)

**Crate:** `crates/naze-discovery/`

**Run:** `cargo run -p naze-discovery -- --port 8889`

**Dashboard:** `http://localhost:8889/`

---

### 2. CLI Scraper (`nazec wrap`) — ~20% done

A CLI command that takes an OpenAPI spec (YAML/JSON file or URL), generates a `.naze` app wrapping each endpoint as a server function, compiles it, and announces the result to the discovery server. This is the cold-start solver — it seeds the network with real, queryable services.

**What exists:**

| Piece | Status | Details |
|---|---|---|
| `nazec announce` | Working | Resolves project, builds manifest from `ProjectContext`, POSTs to discovery server |
| `context_to_manifest()` | Partial | Extracts state, server_functions, data_sources, pages. Actions hardcoded empty. Models not included |
| Compilation pipeline | Complete | parse → typecheck → codegen → serialize — ready to reuse |
| `nazec ai dataset export` | Working | Generates JSONL training pairs — demonstrates the output format pattern |
| Example corpus | 109 files | Templates for what generated .naze apps should look like |

**What's new (~80%):**

- **`nazec wrap <spec.yaml|url>` CLI subcommand** — new entry in `cli.rs`
- **OpenAPI spec parser** — read paths, methods, request/response schemas from OpenAPI 3.x YAML/JSON. No openapi/swagger code exists yet
- **.naze code generator** — transform API endpoints into:
  - `server function` declarations wrapping each endpoint (fetch with URL interpolation)
  - A minimal `app` block with `data` bindings and a basic list/detail UI
  - The generated `.naze` file should be human-readable and valid (passes `nazec check`)
- **Curated seed set** — 20-50 public APIs with clean OpenAPI specs. Priority: APIs that work without auth keys
  - Free/no-auth: Open-Meteo (weather), PetStore (demo), JSONPlaceholder (CRUD), PokeAPI, CatFacts, JokeAPI, REST Countries, Open Trivia DB, Bored API, Hacker News (Algolia)
  - Auth-optional: GitHub REST (public endpoints), OpenLibrary, SWAPI (Star Wars)
- **Batch mode** — `nazec wrap --batch seed-apis.txt` processes a list of specs
- **Complete manifest** — actions extracted from event handlers, models included

**Example output:**

```naze
-- Generated wrapper for PetStore API
-- Source: https://petstore.swagger.io/v2/swagger.json

server function list_pets(status: string)
  fetch "https://petstore.swagger.io/v2/pet/findByStatus?status={status}"

server function get_pet(id: number)
  fetch "https://petstore.swagger.io/v2/pet/{id}"

app "PetStore"
  data pets = list_pets("available")

  column gap: 16 padding: 20
    text "PetStore API" size: 24 bold: true
    text "Available Pets" size: 16 color: gray

    each pet in pets.data
      row gap: 8 padding: 8
        text pet.name bold: true
        text pet.status color: #666666
```

**Files:** New `crates/nazec/src/wrap.rs`, additions to `cli.rs` and `main.rs`

---

### 3. Three-Layer Binary Split — ~15% done

Currently `nazec build` emits a single monolithic `app_data.bin`. The MVP splits this into three layers, matching the natural structure of the `RenderTree`:

| Layer | Contents | Purpose | Typical Size |
|---|---|---|---|
| **Layer 1 — Data** | state, computed, server_functions, server_calls, data, models, storage | Agent-to-agent communication. What can this service *do*? | ~500 bytes |
| **Layer 2 — Interaction** | Event handlers, timers, guards, actions, navigation | What happens when a human interacts? | ~700 bytes |
| **Layer 3 — Presentation** | Root UI tree, themes, pages, params, visual props | What does it look like? | ~6KB |

**What exists:**

| Piece | Status | Details |
|---|---|---|
| `RenderTree` struct | Complete | All fields naturally group into the 3 layers |
| `serialize()` / `deserialize()` | Complete | Monolithic — serializes everything unconditionally |
| `build.rs` output | Complete | Writes `app_data.bin`, WASM, JS, HTML, source map to `dist/` |
| Discovery server | Ready | Already has `headless_hash` field and accepts base64-encoded headless binaries |
| Source maps | Complete | Binary offset → .naze source location mapping |

**What's new (~85%):**

- **Layer-aware serialization** in `naze-ir/src/lib.rs`:
  - `serialize_layer1(tree) -> Vec<u8>` — only state, computed, server_functions, data, models, storage
  - `serialize_layer2(tree) -> Vec<u8>` — only handlers, timers, guards, actions
  - `serialize_layer3(tree) -> Vec<u8>` — only root, themes, pages, params
  - Combined `serialize()` unchanged for backward compatibility
- **Build flags** in `nazec build`:
  - `--layers` — emit `layer1_data.bin`, `layer2_interaction.bin`, `layer3_presentation.bin` alongside `app_data.bin`
  - `--headless` — emit only Layer 1 (for `nazec wrap` to push to discovery server)
  - `--manifest` — emit `naze-manifest.json` alongside the binary
- **Runtime deserialization** — `naze-runtime` and `naze-native` can load from either the combined binary or the three layer files
- **`nazec wrap` integration** — wrapping uses `--headless` to produce the ~500-byte Layer 1 binary that gets announced to the discovery server

**Files:** `crates/naze-ir/src/lib.rs` (serialization), `crates/nazec/src/build.rs` (output), `crates/nazec/src/cli.rs` (flags)

**Why this matters:** Layer 1 headless binaries are what make the discovery network efficient. An agent querying "find me a service that converts currencies" doesn't need to download 7KB of UI layout to check if a service has a `convert_currency` server function. It downloads ~500 bytes, inspects the typed schema, and decides in microseconds.

---

### 4. Naze Browser (Minimal) — ~25% done

A local application that connects to the discovery server, lists available services, and renders selected apps. This is NOT the full vision from [NAZE_BROWSER.md](NAZE_BROWSER.md) — no AI prompt bar, no generation, no composition, no credential wallet. Just: browse, select, render.

**What exists:**

| Piece | Status | Details |
|---|---|---|
| `naze-native` crate | Complete | Full native window rendering via winit + softbuffer + tiny-skia + naze-layout. 35KB main.rs + 10KB renderer.rs |
| App rendering | Complete | Deserializes `app_data.bin`, computes layout, renders to pixel buffer with interaction |
| Hot reload | Complete | `nazec run` watches for file changes |
| Layout engine | Complete | Row, column, stack, grid, flex — custom ~200 LOC engine |

**What's new (~75%):**

- **Discovery client integration** — fetch service list from `http://localhost:8889/api/v1/discovery/search` or `/export`
- **Service browser UI** — a panel listing discovered services with:
  - Service name and domain
  - Trust score (color-coded: green > 0.7, yellow > 0.4, red below)
  - Capability summary (server function count, state field count)
  - Click to render
- **App fetching** — download the full binary (or Layer 3 + Layer 2 + Layer 1) from the discovery server and pass it to the existing renderer
- **App selection** — switch between rendered apps (tabs or back-to-list navigation)

**Two possible approaches:**

1. **Extend `naze-native`** — add a service browser as a built-in panel rendered alongside apps. Pure Rust, no new dependencies. Simpler but the UI for the browser chrome itself is hand-coded.
2. **Web-based shell** — a local web page (served by `nazec browser`) that calls the discovery API directly, lists services, and embeds the WASM runtime to render selected apps in a canvas. More polished UI, reuses existing web runtime.

**Recommended:** Option 2 (web shell) for the MVP — the discovery server already has a dashboard with professional HTML/CSS/JS, and the WASM runtime already renders apps in a canvas. The browser is a page that combines both.

**Files:** New command in `cli.rs`, new `crates/nazec/src/browser.rs` (or extend `crates/naze-native/`)

---

### 5. Training Data Export — ~35% done

The discovery server exports its accumulated knowledge as structured JSONL — .naze source code paired with quality signals from real network activity. This is the seed of the data flywheel described in [FUTURE.md](FUTURE.md).

**What exists:**

| Piece | Status | Details |
|---|---|---|
| `GET /export?since=...` | Working | Returns service metadata JSON array (domain, name, version, manifest, hashes) |
| `nazec ai dataset export` | Working | Generates JSONL with `{instruction, response}` pairs using LLM |
| SQLite schema | Complete | services, trust_scores, observations, compositions — all the raw data |
| Trust scoring | Complete | Per-profile scores with breakdowns, base + dynamic adjustments |

**What's new (~65%):**

- **Source storage** — during `nazec announce` (or `nazec wrap`), store the raw `.naze` source text alongside the manifest. Currently only the manifest JSON is stored. Options:
  - Add a `source_code TEXT` column to the `services` table
  - Store source as a blob in the filesystem (like headless binaries)
- **Training export endpoint** — `GET /api/v1/discovery/export/training?min_trust=0.5&format=jsonl`
  - Joins services + trust_scores + observations + compositions
  - Returns JSONL (one JSON object per line), not a JSON array
  - Each line:
    ```json
    {
      "source": "app \"Weather\"\n  server function get_weather(city: string)\n    fetch ...",
      "manifest": { "name": "Weather", "server_functions": ["get_weather"], ... },
      "trust": { "default": 0.85, "ecommerce": 0.72 },
      "observations": { "usage_count": 42, "flag_count": 0, "composition_count": 3 },
      "compositions": [["weather-api", "maps-api"], ["weather-api", "travel-planner"]]
    }
    ```
- **Quality filtering** — `min_trust` and `min_usage` query parameters to export only proven services
- **CLI convenience** — `nazec export training --output training.jsonl` as a wrapper around the HTTP endpoint

**Training signal types produced:**

| Signal | Source | Use |
|---|---|---|
| Source + trust score | service registration + scoring | Supervised fine-tuning: "good Naze code looks like this" |
| Composition patterns | `compositions` table | Few-shot examples: "services A+B combine this way" |
| Usage counts | `observations` table | Reward signal: popular = useful |
| Flag events | `observations` where kind=flag | Negative examples: "this code was flagged" |

**Files:** `crates/naze-discovery/src/api.rs` (new endpoint), `crates/naze-discovery/src/storage_sqlite.rs` (source storage + join query), `crates/nazec/src/discovery_client.rs` (announce with source)

---

## Demo Walkthrough

```bash
# ── Terminal 1: Start the discovery server ──────────────────────────
cargo run -p naze-discovery -- --port 8889
# Discovery server running at http://localhost:8889
# Dashboard: http://localhost:8889/
# 6 trust profiles loaded: default, healthcare, ecommerce, iot, finance, education

# ── Terminal 2: Seed the network with API wrappers ──────────────────
# Single API:
nazec wrap https://petstore.swagger.io/v2/swagger.json
# → Generated: petstore.naze (3 server functions, 1 data binding)
# → Compiled: layer1_data.bin (412 bytes), layer2_interaction.bin (180 bytes),
#             layer3_presentation.bin (2.1KB)
# → Announced to localhost:8889
# → Trust score: 0.72 (1 external domain, no PII, no device APIs)

# Batch seed from curated list:
nazec wrap --batch seed-apis.txt
# → 25 APIs processed, 23 succeeded, 2 failed (invalid specs)
# → 23 services registered on discovery network

# ── Terminal 2: Verify via discovery ────────────────────────────────
nazec discover "fn:get_weather" --server http://localhost:8889
# → open-meteo (trust: 0.85) — matched: server_function "get_weather"
# → weatherapi (trust: 0.71) — matched: server_function "get_weather_forecast"

nazec discover "state:name:string,fn:list%" --server http://localhost:8889
# → petstore (trust: 0.72) — matched: state "name", server_function "list_pets"
# → pokeapi (trust: 0.88) — matched: state "name", server_function "list_pokemon"

# ── Terminal 3: Browse in the Naze Browser ──────────────────────────
nazec browser
# → Opens browser at http://localhost:3030
# → Left panel: 23 discovered services, sorted by trust score
# → Click "PetStore" → right panel renders the pet listing app
# → Click "Open-Meteo Weather" → renders weather lookup app
# → Trust badges: green (>0.7), yellow (>0.4), red (<0.4)

# ── Terminal 2: Export training data ────────────────────────────────
curl "http://localhost:8889/api/v1/discovery/export/training?min_trust=0.5" \
  > training.jsonl
# → 23 lines, each: {source, manifest, trust, observations, compositions}
# → Ready for fine-tuning: filter to trust > 0.8 for "good code" examples

wc -l training.jsonl
# 23

head -1 training.jsonl | jq .trust
# {"default": 0.85, "healthcare": 0.82, "ecommerce": 0.71, ...}
```

---

## What Exists vs. What Needs Building

| Component | % Done | Exists | Needs Building |
|---|---|---|---|
| **Discovery Server** | ~90% | 23/24 endpoints, SQLite, trust scoring, dashboard, 61 tests | Training data endpoint, source storage during announce |
| **CLI Scraper** | ~20% | `announce` command, manifest builder, compilation pipeline, JSONL export pattern | `wrap` subcommand, OpenAPI parser, .naze codegen, seed list, batch mode |
| **Binary Split** | ~15% | `RenderTree` struct, monolithic serializer, `build.rs` output | Layer-aware serialization, build flags, layer file output |
| **Naze Browser** | ~25% | Native renderer (winit+tiny-skia), app rendering, layout engine | Discovery client, service browser UI, app fetching, web shell |
| **Training Export** | ~35% | `/export` endpoint, JSONL pattern in `ai.rs`, SQLite schema | Source storage, training endpoint with joins, quality filtering |

**Weighted overall: ~35% complete.** The heaviest piece (discovery server) is nearly done. The largest remaining work is the CLI scraper and binary split.

---

## Scope Boundaries

This is an MVP. Explicitly **not** in scope:

- **No AI prompt bar** — the browser renders discovered apps, it doesn't generate new ones
- **No composition** — discovering and rendering individual services only, no agent wiring services together
- **No federation** — single local discovery server, no peer sync
- **No schema engine** — no DDL generation, no migrations (see [PERSISTENCE_LAYER.md](PERSISTENCE_LAYER.md))
- **No vector fields** — no semantic search (see [FUTURE.md](FUTURE.md))
- **No Parquet/streaming export** — JSONL files only
- **No auth passthrough** — wrapped APIs must be public (no API key management)
- **No publishing from browser** — the browser is read-only, publishing happens via CLI
- **No mobile** — browser runs on desktop only

---

## Architecture Notes

**Layer split is natural.** The `RenderTree` struct already groups fields by concern. Layer 1 (state, computed, server_functions, data, models, storage) is the semantic core. Layer 2 (handlers, timers, guards) is interaction logic. Layer 3 (root, themes, pages, params) is presentation. The serializer just needs to write them separately.

**`nazec wrap` reuses the full pipeline.** The only new code is OpenAPI → .naze source generation. Once the `.naze` file is generated, the existing parse → typecheck → codegen → serialize pipeline handles compilation, and the existing `announce` command handles registration.

**Training data is a SQL join.** The discovery server's SQLite already has services, trust_scores, observations, and compositions tables. The training endpoint joins them and streams as JSONL. The only schema change is storing .naze source text during announce.

**The browser can be a web page.** The discovery server already serves HTML dashboard pages. The WASM runtime already renders apps in a canvas. The MVP browser is a local web page that combines the discovery API (for listing services) with the WASM runtime (for rendering selected apps). No native UI framework needed.

---

## Success Criteria

1. **`nazec wrap <openapi-spec>`** generates a valid `.naze` file that passes `nazec check`, compiles to 3 layer binaries, and announces to the discovery server
2. **20+ services** seeded from public APIs, all with trust scores and indexed capabilities
3. **Naze Browser** connects to `localhost:8889`, lists discovered services with trust scores, and renders any selected app interactively
4. **`nazec build --layers`** produces `layer1_data.bin`, `layer2_interaction.bin`, `layer3_presentation.bin` alongside the backward-compatible combined `app_data.bin`
5. **Training export** returns JSONL with .naze source + trust scores + observation counts + composition patterns for all registered services
6. **The entire demo** runs on a single machine with `cargo run` and `nazec` commands — no Docker, no cloud, no external infrastructure (except outbound HTTP to the public APIs themselves)

---

## Connection to the Bigger Vision

This MVP proves the inner loop of the [Discovery Network](DISCOVERY_NETWORK.md) and the first three layers of the [Naze Browser](NAZE_BROWSER.md):

- **Discovery Network:** Services are published, indexed by capability, scored for trust, and discoverable via structural queries. The MVP uses a curated seed set instead of organic agent publishing, but the mechanics are identical.
- **Naze Browser:** The MVP browser can render any discovered app. The full vision adds generation (AI prompt bar), composition (wiring services together), and the persistence layer. Each of those layers builds on what the MVP proves.
- **Data Flywheel:** The training export closes the loop described in [FUTURE.md](FUTURE.md) §4. Even with just 23 seeded services, the export demonstrates the pipeline: code + quality signals → training data. At scale, this replaces manual dataset curation.
- **Three-Layer Architecture:** The binary split enables the discovery network's key efficiency — agents read ~500 bytes to evaluate a service, not ~7KB of UI layout. This is what makes agent-to-agent communication practical at <1ms per service evaluation.

The MVP is the foundation. Everything in the [roadmap](ROADMAP.md) after Phase 6 builds on these five components.
