# Future Directions: Post-Phase 6

Ideas evaluated from [IDEAS.md](../IDEAS.md) for exploration after Phase 6 (Developer Experience & Adoption) is complete. These are **not committed roadmap items** — they're concepts worth pursuing based on what Naze already has.

All ideas evaluated against the [Psi cost equation](TOKEN_EFFICIENCY.md): **Psi(L, n) = n x lambda x sigma x (1 + r) x mu**. A good idea keeps sigma = 1 and doesn't bloat the grammar.

---

## 1. AI Prompt Bar in Playground

Add a conversational prompt bar to the hosted playground (M44) that generates and iteratively refines .naze code from natural language. Users describe intent, the system generates a working app, and follow-up prompts trigger incremental edits — all running live in the browser.

This is the single best demo of Naze's AI-native thesis: small grammar enables constrained decoding on local models, single-file components mean the LLM needs zero cross-file context, and the WASM compiler gives instant feedback.

### What already exists

- Hosted playground with CodeMirror editor, live compilation, error display (M44)
- `naze-playground` WASM crate — compiler runs in the browser
- `nazec ai generate` — prompt-to-.naze generation with validation
- `nazec grammar --format gbnf` — grammar export for constrained LLM decoding
- 486 training examples (392 generated + 94 hand-crafted)
- Hot reload infrastructure in dev server

### What's new

- LLM integration in the playground (API relay to cloud model, or local model via WebLLM/llama.cpp WASM)
- Prompt bar UI with conversation history
- Diff-based patching: follow-up prompts produce targeted edits, not full regeneration
- Optional "why" annotations — generated comments explaining intent behind code sections
- Error-aware refinement: compilation errors auto-fed back to LLM for self-correction

### Psi impact

- **sigma:** No change (still single-file generation)
- **lambda:** No change (no new syntax)
- **r:** Actively reduces r — the refinement loop catches and fixes generation errors
- **mu:** This is the showcase — constrained decoding on Naze's small grammar means mu stays low
- **Grammar:** Zero new rules

### Open questions

- Cloud model relay vs. in-browser local model? Cloud is simpler; local proves the mu thesis
- How much conversation context to retain? Full history vs. sliding window
- Should the prompt bar also support discovery network queries ("find a service that does X")?

### Ties to

M47 (AI Validation & Model), M44 (Playground)

---

## 2. Vector Memory & Schema Engine

Extend Naze's declarative model system (M39) with vector field types and a schema engine that auto-generates DDL and migrations. This makes Naze the easiest way for agents to build apps with persistent, queryable memory — including semantic search via embeddings.

Today's agents either hallucinate state, bloat context windows with full page re-ingestion, or reinvent vector stores from scratch every time. Naze's model declarations already compile to parameterized SQL. Adding vector support and automatic migrations makes persistence truly declarative.

### What already exists

- Model declarations: `model Todo { text string, done bool }` (M39)
- Query compilation: `find Todo where done == false` compiles to parameterized SQL
- Dual backend dispatch: PostgreSQL and SQLite at runtime
- Server functions as execution boundary
- [PERSISTENCE_LAYER.md](PERSISTENCE_LAYER.md) — full schema engine spec (~150-200 lines to implement)

### Phase A — Schema Engine (~200 lines)

- DDL generation: `model` declarations compile to `CREATE TABLE` statements
- Schema diffing: compare old vs. new model, emit `ALTER TABLE` operations
- Migration tracking: `_naze_migrations` table with schema hashes
- Destructive change detection: flag operations that would lose data
- Auto-run on `nazec build` or `nazec dev` (with confirmation for destructive changes)

### Phase B — Vector Fields

Vector fields give Naze apps **semantic search** — finding things by meaning, not exact matching. Without vectors, `find Note where title == "meeting notes"` only returns exact matches. With vectors, `find Note where embedding ~ "what did we discuss about the API redesign"` returns semantically related notes — even if they never contain those words.

**Why this matters:**

- **Agent memory across sessions** — An agent queries `find Memory where embedding ~ "user's dark mode preference"` instead of re-reading entire history. Tokens saved, context preserved between conversations.
- **Smart apps in ~5 lines** — What takes a React developer a vector database setup (Pinecone/Weaviate), an embedding API, query logic, and ranking code becomes a model declaration and a query. Sigma stays 1.
- **Intelligent persistence** — The storage layer understands *intent*. "Find tasks similar to what I worked on last week" becomes a one-liner.
- **Smarter discovery network** — Service capabilities embedded as vectors. An agent's natural language query ("I need something that handles payments") matches services by meaning, not just keyword tags.

**What's new:**

- New field type: `embedding: vector[384]` in model declarations
- Similarity search operator: `find Note where embedding ~ "search query" limit 5`
- Optional auto-embedding hook: specify a model, and inserts auto-generate embeddings
- All vector ops are server-side (server functions) — zero WASM impact

**Backend support — both existing Naze backends have vector extensions:**

| Backend | Extension | Notes |
|---------|-----------|-------|
| SQLite | `sqlite-vec` | Single-file extension, ~200KB, runs in-process |
| PostgreSQL | `pgvector` | Standard extension, widely available (Neon, Supabase, RDS) |

The compiler generates the right SQL dialect per backend — the .naze code is identical:

```naze
-- Developer writes this (backend-agnostic)
find Note where embedding ~ "API redesign discussion" limit 5
```

```sql
-- Compiles to SQLite (sqlite-vec)
SELECT * FROM notes ORDER BY vec_distance_cosine(embedding, ?) LIMIT 5;

-- Compiles to PostgreSQL (pgvector)
SELECT * FROM notes ORDER BY embedding <=> $1 LIMIT 5;
```

Same pattern as existing query compilation (M39) — `find Todo where done == false` already compiles to different parameterized SQL per backend. Vector queries extend this with similarity operators instead of equality checks.

### Phase C — Memory Fabric (further out)

- `memory` keyword as sugar over model + storage + retention policy
- Retention declarations: `retain: 2 years with summarization`
- Privacy controls: `access: encrypted, consent_required`
- Agent memory API: headless binary exposes typed get/put/query endpoints

### Psi impact

- **sigma:** Stays 1 — memory declarations are in the .naze file, not external config
- **lambda:** Slight increase (new field type syntax), but replaces manual SQL + ORM setup
- **r:** Decreases — compile-time validation catches schema errors before runtime
- **mu:** No change — vector fields follow existing model declaration patterns
- **Grammar:** ~3-5 new rules (vector type, similarity operator, memory block)

### Open questions

- Should `memory` be a distinct keyword or a flavor of `model` with extra annotations?
- Auto-embedding: which default model? Allow user-specified via `naze.toml`?
- How to handle vector index rebuilds on schema changes?

### Ties to

M39 (Declarative DB Queries), [PERSISTENCE_LAYER.md](PERSISTENCE_LAYER.md)

---

## 3. Legacy Wrapper CLI (`nazec wrap`)

A CLI tool that generates .naze service wrappers from existing APIs. Feed it an OpenAPI spec (or URL), and it outputs a minimal .naze file with server functions that proxy to the legacy API. The wrapper gets a manifest and can be published to the discovery network — making existing services first-class participants without rebuilds.

This is the pragmatic adoption bridge and the **discovery network's cold-start solution**. Agents start preferring Naze-native services (cheaper, more reliable, typed manifests) while still accessing the existing web. Over time, popular wrappers get refined into full Naze apps.

### Bootstrap strategy

The discovery network needs real services to be useful. Wrappers solve this:

1. **Curated seed set** — Pick 20-50 popular APIs with clean OpenAPI specs (GitHub REST, OpenWeatherMap, JSONPlaceholder, HackerNews, Stripe, etc.). Run `nazec wrap` on each. Publish to the registry. The discovery network now has real, queryable services.

2. **Batch from public directories** — API directories like APIs.guru maintain thousands of validated OpenAPI specs. A script batch-processes a curated subset (top 100 by category: weather, finance, social, utilities). Each becomes a .naze wrapper with typed server functions and a manifest.

3. **End-to-end demo** — An agent asks the discovery network "find a service that gets weather data." The network returns the OpenWeatherMap wrapper. The agent (or playground prompt bar) generates a .naze app that calls it. Full loop, no human in the loop. This is the demo that proves the thesis.

The flywheel: `nazec wrap` generates wrappers -> wrappers populate discovery -> agents find services -> prompt bar composes them into apps -> those apps prove Naze works. Without services to discover, the prompt bar is just a code generator. The wrappers give it something to compose.

### What already exists

- Server functions with HTTP fetch: `server fn get_data() { fetch "https://api.example.com/data" }`
- Discovery network with capability matching and trust scoring
- `discovery_client.rs` — announce/discover from CLI
- `naze-manifest.json` generation from compiler context
- Package registry (`nazec publish`, `nazec search`)

### What's new

- `nazec wrap <spec.yaml|url>` CLI command
- OpenAPI parser: reads paths, methods, request/response schemas
- Code generator: emits .naze file with server functions wrapping each endpoint
- Trust adjustment: wrapped services get a lower base trust score (external dependency)
- Manifest generation: capabilities extracted from the OpenAPI spec map to discovery schemas

### Example output

```naze
-- Generated wrapper for PetStore API
-- Source: https://petstore.swagger.io/v2/swagger.json

server fn list_pets(limit: number)
  fetch "https://petstore.swagger.io/v2/pet/findByStatus?status=available"

server fn get_pet(id: number)
  fetch "https://petstore.swagger.io/v2/pet/{id}"

server fn add_pet(name: string, status: string)
  fetch "https://petstore.swagger.io/v2/pet" method "POST" body { name: name, status: status }

app "PetStore Wrapper"
  data pets = list_pets(20)

  column gap: 16
    text "PetStore API" size: 24 bold: true
    each pet in pets.data
      row gap: 8
        text pet.name
        text pet.status color: gray
```

### Psi impact

- **sigma:** No change — wrapper is a self-contained .naze file
- **lambda:** No change — uses existing server function syntax
- **r:** No change — generated code follows canonical patterns
- **mu:** No change — no new grammar
- **Grammar:** Zero new rules (pure CLI tooling)

### Open questions

- How to handle authentication passthrough (API keys, OAuth tokens)?
- Should wrappers auto-update when the upstream spec changes?
- Rate limiting: built into the wrapper or left to the discovery network's trust layer?
- GraphQL support in addition to OpenAPI?

### Ties to

M49 (Production Deployment), Discovery Network, `nazec publish`

---

## 4. Discovery Network as Data Flywheel

The discovery network isn't just service infrastructure — it's a **self-generating, quality-labeled training dataset**. Every agent interaction produces data that improves the next interaction. This connects all the other ideas into a compounding loop.

### The insight

Traditional AI training data is scraped from GitHub — "code that exists." Discovery network data is **code that was used, rated, composed, and survived**. A service with trust score 0.95, 10,000 agent invocations, and 50 successful compositions is an enormously stronger training signal than any hand-labeled example. The quality labels come from real-world performance, not human annotation.

### What the network already captures

The `naze-discovery` crate's SQLite schema already records:

- **Service records** — manifest JSON, version history, activity timestamps
- **Observations** — usage, discovery, and flag events with agent IDs and payloads
- **Trust scores** — per service, per profile (healthcare, finance, ecommerce, etc.), with base score + dynamic adjustments
- **Compositions** — which services get wired together, frequency, first/last seen
- **Provenance** — which services were derived from which (evolution chains)

### Five feedback loops, five training signal types

| Feedback loop | What happens | Training signal type |
|---------------|-------------|---------------------|
| Service published with trust score | .naze source code + computed quality label | **Supervised fine-tuning** — "good Naze code looks like this" |
| Agents use a service | Usage observations accumulate | **Reward signal (RLHF)** — real-world preference data |
| Agents compose services together | Composition patterns with frequency | **Few-shot examples** — "when you need X+Y, combine A+B this way" |
| Service flagged, fixed, republished | Before/after code with trust delta | **Preference pairs** — DPO/RLHF training data |
| Trust score changes over time | Score trajectory across versions | **Quality prediction** — predict which code patterns age well |

### How it compounds

```
nazec wrap generates wrappers (seeds the network with services)
  → agents discover and use them (generates usage observations)
    → trust scores adjust from real behavior (labels get refined)
      → high-trust .naze code becomes training data
        → better models generate better .naze services
          → those services get used, scored, composed...
```

Each cycle produces more labeled data AND better models. The longer the network runs, the wider the moat. Anyone can copy Naze's syntax. Nobody can copy millions of quality-labeled interaction observations.

### Gaps to close

The existing schema is ~80% ready for training data extraction. Three small additions get it to ~95%:

**1. Outcome validation (`incidents` table)**

Trust scores today are *calculated* from manifest analysis, not *validated* against real outcomes. Adding an incidents table closes the loop:

```sql
CREATE TABLE incidents (
    incident_id TEXT PRIMARY KEY,
    service_id INTEGER REFERENCES services(id),
    severity TEXT,          -- low, medium, high, critical
    outcome TEXT,           -- true_positive, false_positive, true_negative, false_negative
    discovered_at TEXT,
    validated_by TEXT
);
```

This answers: "Was the trust score prediction correct?" — the missing ground truth label.

**2. Normalized observation payloads**

Currently observations store arbitrary JSON. Normalizing by kind makes them ML-ready:

- **usage:** `{feature: str, result: "success"|"error", latency_ms: int}`
- **flag:** `{reason_code: enum, severity: "low"|"med"|"high", evidence: str}`
- **discovery:** `{method: str, query_matched: bool, rank_position: int}`

**3. Temporal aggregation columns**

Pre-computed rolling metrics on the services table:

- `usage_7d`, `usage_30d` — recent usage velocity
- `flags_30d` — recent flag activity
- `composition_velocity` — how often this service appears in new compositions

These turn point-in-time observations into trend signals ("usage is growing" vs "usage is flat").

### Signal gas: agent contributions as network fuel

Inspired by how blockchain gas fees keep networks healthy — every transaction pays a small cost that compensates validators for doing work and prevents spam — the discovery network can use a similar mechanism where the currency is **signal** instead of money.

**The principle:** Every agent that consumes a service from the network contributes back a small structured observation. This "signal gas" costs the agent almost nothing (~20-100 tokens of feedback) but aggregated across millions of interactions, it's what keeps the network's trust scores grounded in reality.

**Three tiers of contribution:**

| Tier | What the agent contributes | Cost | Value to network |
|------|---------------------------|------|-----------------|
| **Passive** (default) | Success/failure + latency | ~20 tokens | Basic health signal |
| **Active** (suggested) | + data quality rating + composition context | ~80 tokens | Trust refinement + composition patterns |
| **Generative** (opt-in) | + improvement suggestions, alternative approaches | ~200 tokens | Evolution signal, preference pairs |

**How this strengthens trust scoring:**

The current trust scorer (`simple-v1`) computes base scores from static manifest analysis — counting external domains, PII patterns, device APIs. Signal gas makes trust scores **behavioral** instead of just structural:

- A service that 10,000 agents report as successful gets trust boosted by real evidence, not just "its manifest looks clean"
- A service that agents consistently abandon mid-interaction (low completion rates) sinks without anyone explicitly flagging it
- Bad services die from **lack of positive signal** — they don't need explicit complaints. If 1,000 agents try a service and only 2 report success, that's a clear signal
- Composition context reveals **which services work well together** — trust becomes relational, not just per-service

**The incentive structure:**

- Agents that contribute more signal get **better discovery results** — the network learns their preferences and context, returning more relevant matches
- Services with more feedback have **higher-confidence trust scores** — agents prefer them over uncertain alternatives with sparse signal
- The network can **prioritize responses** for agents with strong contribution history — not a paywall, just queue priority

**What this is NOT:**

- Not a paywall — agents can use the network without contributing
- Not a cryptocurrency — no blockchain, no tokens with financial value
- Not mandatory — but like Wikipedia, the aggregate of voluntary contributions creates something no individual could build alone

The key difference from blockchain gas: this is **automated and structured**. Agents don't make a conscious decision to "pay" — the client library includes signal contribution as a default behavior after service consumption. The feedback is machine-readable training signal, not a financial transaction.

### The network as training pipeline

The discovery network's accumulated history — service code, trust scores, observations, compositions, provenance chains — is a continuously growing, self-labeling training dataset. It replaces manual dataset curation with a living pipeline where every interaction generates training signal.

**What the network produces vs. what AI training needs:**

| Network output | Training data type | How it's used |
|---------------|-------------------|---------------|
| .naze source code + trust scores | **Supervised fine-tuning** | Filter to trust > 0.8, usage > 1,000 = curated "good Naze code." No human labeling needed. Replaces hand-crafted example sets. |
| Composition patterns + frequency | **Few-shot examples** | "When agents needed food + venue + scheduling, they composed A, B, C this way, success rate 95%." The prompt bar pulls these directly for multi-service generation. |
| Flag → fix → republish (provenance) | **Preference pairs (DPO/RLHF)** | Service v1 got flagged, v2 fixed it. The before/after code is a preference pair — "v2 is better than v1" — generated automatically from version history. |
| Signal gas observations (millions) | **Reward model** | Structured success/failure per service per context. "This pattern succeeded 98% in healthcare but 60% in ecommerce." Context-dependent reward signal, far richer than binary good/bad. |
| Discovery query → results → chosen → used | **Retrieval training** | "Agent asked for X, network returned [A, B, C], agent chose B, used it successfully." Trains the discovery matcher AND teaches models which services to recommend. |
| Trust score trajectories over time | **Quality prediction** | Services that climb from 0.7 to 0.95 — what patterns do they share? Services that decay from 0.8 to 0.3 — what went wrong? Predicts which code patterns age well. |

**Scale comparison:**

Today: 486 training examples (392 generated + 94 hand-crafted).
After 1 year of network operation with signal gas: potentially millions of quality-labeled, context-rich examples that no competitor can replicate — because the data comes from the network's own usage, not a static corpus.

**The closed loop:** The network doesn't just *use* AI — it produces the data that makes AI better at building for the network. Better models generate better services, which get higher trust scores and more usage, which produces more training data, which trains better models. The dataset is never "done" — it improves as long as the network is alive.

This reframes M47 (AI Validation & Model): the fine-tuned model isn't a one-time artifact trained on a static dataset. It's a continuously improving model fed by the network's own interaction history. The initial hand-crafted dataset bootstraps the model; the network takes over from there.

### Training data export: the network's synapse

The neural network analogy for the discovery network is almost literal — services are nodes, compositions are connections, trust scores are weights, signal gas is the learning signal. But a neural network isn't useful if it only learns internally. **The export layer is the synapse firing outward** — how the network's accumulated learning propagates to the broader ecosystem, strengthening every model that touches Naze.

**Three export tiers, matching different consumers:**

**Tier 1: Curated Snapshots (batch, periodic)**

Like Common Crawl for the web, but for Naze services. A weekly or monthly export of:
- All .naze source code with trust score above threshold (e.g., > 0.8 and usage > 1,000)
- Composition graphs with success rates
- Preference pairs extracted from provenance chains (v1 flagged → v2 fixed)
- Aggregated signal gas metrics per service

Format: Parquet files or HuggingFace dataset format. Anyone can download and train on it. This is how the open-source ML community trains Naze-capable models without needing access to the live network — the discovery network becomes the **Common Crawl of the agentic web**.

**Tier 2: Streaming Firehose (real-time, filtered)**

A Change Data Capture (CDC) stream of network events as they happen:
- New service registered (manifest + source)
- Trust score changed (old → new, with reason)
- Composition created (services involved, context)
- Flag raised (service, reason, severity)
- Signal gas observations (success/failure/latency)

Consumers subscribe with filters — "all trust score changes in healthcare profile" or "all new services with capability `payment`." For model providers doing continuous fine-tuning, or researchers studying network dynamics in real time.

**Tier 3: Aggregate API (on-demand, computed)**

Query endpoints for derived insights:
- Top composition patterns this month
- Code patterns that correlate with trust > 0.9
- Average trust trajectory for services in category X
- Signal gas success rates by service type and context

For the prompt bar (bias generation toward proven patterns) and for ML pipelines that need structured features rather than raw data.

**Who consumes what:**

| Consumer | Export tier | What they do with it |
|----------|-----------|---------------------|
| Open-source ML community | Snapshots | Train/fine-tune Naze-capable models, publish to HuggingFace |
| Model providers (Anthropic, OpenAI, etc.) | Streaming | Continuously improve Naze code generation in their models |
| Naze's own M47 model | Snapshots + Streaming | Self-improving fine-tuned model |
| The prompt bar | Aggregate API | Bias generation toward patterns that work |
| Researchers | Snapshots | Study network dynamics, publish papers on emergent behavior |
| Federated registries | Streaming | Sync trust data and service catalogs across nodes |

**The open data commons play:**

Publishing training data openly creates a second flywheel:
- Open data → more model providers train on it → more models speak Naze fluently
- More fluent models → more agents use the network → more signal gas → better data
- Better data → attracts more model providers...

The network isn't just a service registry or a training pipeline — it's a **distribution mechanism for Naze fluency**. Every model that trains on its data becomes a Naze-native model, expanding the ecosystem without Naze having to build or maintain those models.

**Implementation:** This layers on the existing SQLite storage without changing the discovery network's core:
1. SQLite WAL (Write-Ahead Log) already captures all changes
2. A lightweight CDC process tails the WAL, emits events to a stream (NDJSON file for small scale, Kafka/NATS for large)
3. A periodic job queries the database, filters by quality thresholds, exports to Parquet
4. API endpoints serve aggregate queries over the existing tables

No new tables, no schema changes — pure read-side infrastructure on top of what `naze-discovery` already stores.

### Ties to

- **`nazec wrap`** (#3) — bootstraps the network with services that generate the initial data
- **AI Prompt Bar** (#1) — can bias generation toward patterns from high-trust services
- **M47 (AI Validation)** — the network provides continuously updated training data, not just a static dataset
- **Vector Memory** (#2) — vector embeddings of service capabilities enable semantic discovery matching

---

## Rejected: Orchestration Primitives

### What they are

Workflow patterns for coordinating multiple AI agents on a task — ReAct loops, fan-out/merge, debate/critique, hierarchical delegation, pipelines. Frameworks like CrewAI, AutoGen, and LangGraph implement these as explicit Python code. [IDEAS.md](../IDEAS.md) proposed adding them as Naze language features (`agent team`, `workflow`, `roles` keywords).

### Why not

**Orchestration is moving into the model layer, not application code.**

The trend is clear: each generation of AI models absorbs what was previously explicit orchestration logic.

- **Claude Code** already has built-in agent spawning, parallel tool use, and autonomous task decomposition — no user-defined workflow syntax needed.
- **Extended thinking** *is* the ReAct loop (reason, act, observe) happening inside the model, not in application code.
- **Anthropic's Agent SDK** handles tool routing, retries, and context management at the SDK level.
- **OpenAI Codex** does multi-step planning autonomously.

The "fan-out/merge" pattern that CrewAI needs 50 lines of Python for? Claude just does it when the task calls for it. The debate pattern? That's what extended thinking already does internally. Building `agent team` syntax into Naze would be like hand-writing assembly optimizations right before compilers got good — effort in a layer that's about to be automated away.

As models get smarter, the orchestration framework becomes the model itself. Hardcoding workflow patterns into a language bets against this trajectory.

### What Naze should do instead

Be an excellent **tool** for increasingly capable agents, not an orchestration framework:

- **Typed manifests** so agents know what a service can do (already have this)
- **Headless binaries** so agents invoke services without UI overhead (already have this)
- **Prompt runtime** so Naze apps can call AI providers (already have this)
- **Discovery network** so agents find and compose services by capability (already have this)

The right investment is making these existing interfaces richer and more discoverable — not adding orchestration grammar that competes with the model's own planning capabilities.

---

## Prioritization

The adoption flywheel determines order: wrappers seed the discovery network, the network generates training data, the prompt bar composes services, persistence makes results durable.

| Priority | Idea | Depends on | Scope | Why this order |
|----------|------|-----------|-------|----------------|
| 1 | `nazec wrap` + seed set | Discovery Network | Medium | Cold-start solver — seeds the network, starts the data flywheel |
| 2 | AI Prompt Bar | M47 (AI Validation) | Medium | The demo — generates apps, composes services, produces more interaction data |
| 3 | Schema Engine (Phase A) | M39 (done) | Small (~200 lines) | Foundation for persistence — DDL + migrations |
| 4 | Data Flywheel schema additions | Discovery Network | Small | 3 tables/columns — closes the training data feedback loop |
| 5 | Vector Fields (Phase B) | Schema Engine | Medium | Differentiator — declarative semantic search |
| 6 | Memory Fabric (Phase C) | Vector Fields | Large | Long-term — agent-native persistence with retention + privacy |

**Cross-cutting:** The data flywheel (#4) enhances everything else. Every wrapper published, every prompt bar interaction, every service composition generates training signal. It should be woven into the other work rather than treated as a separate milestone.

---

## Long-term: IoT as a Discovery Network Extension

The discovery network's headless binaries (~500 bytes) and typed manifests are a natural fit for IoT devices. Not as an IoT platform — Naze shouldn't implement Zigbee or MQTT — but as the **interface and discovery layer** on top of existing IoT infrastructure. Devices publish manifests describing capabilities (read temperature, toggle switch), agents discover and compose them across vendors. Trust scoring catches misbehaving devices. Signal gas from millions of homes refines which combinations work. Edge dashboards (395KB WASM runtime on a Raspberry Pi) provide the human interface. The constraint: Naze needs WASM or a native OS, so bare-metal microcontrollers (ESP32, STM32) would need a thin bridge rather than running Naze directly. Low priority — the architecture already supports it, no language changes needed.
