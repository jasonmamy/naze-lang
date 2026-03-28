# Discovery Network — Reference Implementation

A reference discovery server backed by SQLite that proves the API surface defined in [DISCOVERY_NETWORK.md](./DISCOVERY_NETWORK.md). The API contract is the deliverable — alternative backends (IPFS, DHT, federated nodes) can implement the same endpoints.

## Language Independence

**The HTTP API is the contract, not the implementation language.** The reference implementation is written in Rust (fast, same toolchain as the rest of naze-lang), but a conforming discovery server can be built in any language — Go, Python, TypeScript, Java, anything — as long as it implements the same 26 JSON API endpoints with the same request/response shapes.

Two layers of the spec, two levels of commitment:

| Layer | What It Is | Language Requirement |
|-------|-----------|---------------------|
| **HTTP API contract** | 26 JSON endpoints, request/response schemas, behavior semantics | **None** — implement in any language |
| **Rust trait interfaces** | Internal architecture of the reference impl (TrustScorer, StorageBackend, etc.) | **Rust only** — these are the reference impl's internals, not part of the API contract |

The Rust traits are documented in this spec as a design guide — they show how to cleanly separate concerns. A Go implementation would define equivalent interfaces; a Python one would use abstract base classes or protocols. The trait signatures map 1:1 to the HTTP API, so the same mental model applies regardless of language.

Even within the Rust reference impl, the **webhook pattern** allows any trait to delegate to an external HTTP service written in any language. A team could run the Rust server but plug in a Python-based AI trust scorer via `--scorer webhook --scorer-url http://localhost:9000/score`.

## Architecture: Pluggable Trait Interfaces

The server is built on **6 abstract trait interfaces**. Each defines a clear input/output contract. The reference implementation ships with simple, concrete implementations for all 6 — but any can be swapped independently without changing the API, client code, or agent behavior.

```
┌─────────────────────────────────────────────────────┐
│                   HTTP API (fixed)                   │
│         26 JSON endpoints + 7 dashboard pages        │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │ TrustScorer  │  │ Capability   │  │ Capability│ │
│  │              │  │ Matcher      │  │ Extractor │ │
│  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘ │
│         │                 │                 │       │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌─────┴─────┐ │
│  │ Identity     │  │ Federation   │  │ Storage   │ │
│  │ Verifier     │  │ Sync         │  │ Backend   │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
│                                                     │
├─────────────────────────────────────────────────────┤
│           Reference Implementations                  │
│  simple-v1 scorer, SQL matcher, JSON extractor,     │
│  api-key identity, stub sync, SQLite+fs storage     │
└─────────────────────────────────────────────────────┘
```

### Trait Summary

| Trait | Purpose | Input | Output |
|-------|---------|-------|--------|
| `TrustScorer` | Compute trust score for a service | Manifest + profile + observation signals | Score 0.0-1.0 + breakdown |
| `CapabilityMatcher` | Find services matching a structural query | Query matchers + service pool | Ranked matching service IDs |
| `CapabilityExtractor` | Extract capabilities from a manifest | Manifest JSON | List of typed capabilities |
| `IdentityVerifier` | Verify publisher identity on registration | Request headers / credentials | Verified identity string or rejection |
| `FederationSync` | Synchronize services between peer nodes | Local DB + peer URL | Sync result (added/updated/conflicts) |
| `StorageBackend` | Persist and retrieve all data | CRUD operations on all entity types | Stored/retrieved data |

### Trait Definitions

```rust
/// Score trust for a service against a profile.
pub trait TrustScorer: Send + Sync {
    fn score(&self, input: &TrustInput) -> TrustOutput;
    fn name(&self) -> &str;
}

/// Match services against a structural capability query.
pub trait CapabilityMatcher: Send + Sync {
    fn search(&self, query: &CapabilityQuery, services: &dyn StorageBackend) -> Vec<MatchResult>;
    fn name(&self) -> &str;
}

/// Extract capabilities from a manifest.
pub trait CapabilityExtractor: Send + Sync {
    fn extract(&self, manifest: &serde_json::Value) -> Vec<Capability>;
    fn name(&self) -> &str;
}

/// Verify publisher identity from request context.
pub trait IdentityVerifier: Send + Sync {
    fn verify(&self, headers: &HeaderMap) -> Result<PublisherIdentity, AuthError>;
    fn name(&self) -> &str;
}

/// Synchronize with a peer discovery node.
pub trait FederationSync: Send + Sync {
    fn sync(&self, peer_url: &str, local: &dyn StorageBackend) -> Result<SyncResult, SyncError>;
    fn name(&self) -> &str;
}

/// Global service identity — (domain, name) is unique across all nodes.
/// The reference impl maps this to a local i64 internally, but the trait
/// boundary never exposes local IDs.
#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServiceRef {
    pub domain: String,
    pub name: String,
}

/// Persist and retrieve all discovery data.
/// NOTE: All methods use ServiceRef (domain, name) — never local integer IDs.
/// This ensures the trait is distributed-safe: ServiceRef is globally unique
/// across nodes, whereas auto-increment IDs are node-local and meaningless
/// in a multi-node deployment.
pub trait StorageBackend: Send + Sync {
    // Services
    fn upsert_service(&self, service: &ServiceRecord) -> Result<ServiceRef, StorageError>;
    fn get_service(&self, service: &ServiceRef) -> Result<Option<ServiceRecord>, StorageError>;
    fn deactivate_service(&self, service: &ServiceRef) -> Result<(), StorageError>;
    fn list_services(&self, filter: &ServiceFilter) -> Result<Vec<ServiceRecord>, StorageError>;

    // Capabilities
    fn replace_capabilities(&self, service: &ServiceRef, caps: &[Capability]) -> Result<(), StorageError>;
    fn query_capabilities(&self, matchers: &[CapabilityMatcher]) -> Result<Vec<ServiceRef>, StorageError>;

    // Trust
    fn upsert_trust_score(&self, service: &ServiceRef, profile: &str, output: &TrustOutput) -> Result<(), StorageError>;
    fn get_trust_scores(&self, service: &ServiceRef) -> Result<HashMap<String, TrustOutput>, StorageError>;

    // Observations
    fn record_observation(&self, obs: &Observation) -> Result<(), StorageError>;
    fn get_observation_signals(&self, service: &ServiceRef) -> Result<ObservationSignals, StorageError>;

    // Compositions — uses Vec<ServiceRef>, not Vec<i64>
    fn upsert_composition(&self, services: &[ServiceRef]) -> Result<(), StorageError>;
    fn get_top_patterns(&self, limit: u32) -> Result<Vec<CompositionPattern>, StorageError>;

    // Provenance
    fn set_provenance(&self, service: &ServiceRef, sources: &[ServiceRef]) -> Result<(), StorageError>;
    fn get_provenance(&self, service: &ServiceRef) -> Result<Vec<ServiceRef>, StorageError>;

    // Versions
    fn archive_version(&self, service: &ServiceRef) -> Result<(), StorageError>;
    fn list_versions(&self, service: &ServiceRef) -> Result<Vec<VersionRecord>, StorageError>;

    // Peers
    fn add_peer(&self, peer: &PeerRecord) -> Result<String, StorageError>;  // returns peer URL as ID
    fn list_peers(&self) -> Result<Vec<PeerRecord>, StorageError>;
    fn remove_peer(&self, peer_url: &str) -> Result<(), StorageError>;

    // Profiles
    fn list_profiles(&self) -> Result<Vec<TrustProfile>, StorageError>;
    fn create_profile(&self, profile: &TrustProfile) -> Result<(), StorageError>;

    // Export
    fn export_public_services(&self, since: Option<&str>) -> Result<Vec<ServiceExport>, StorageError>;

    // Info
    fn get_stats(&self) -> Result<ServerStats, StorageError>;

    fn name(&self) -> &str;
}
```

### Reference Implementations vs Alternatives

| Trait | Reference Impl | Alternative Implementations |
|-------|---------------|----------------------------|
| `TrustScorer` | `SimpleScorer` — pattern-match field names, linear weighted sum, additive adjustments | AI semantic analysis of manifest behavior; graph-based trust propagation through composition chains; RL from agent feedback; community-driven with human reviews |
| `CapabilityMatcher` | `SqlMatcher` — SQL LIKE + INTERSECT on capabilities table | Vector embedding similarity (agents searching for "order" find "purchase"); LLM-powered semantic matching; ontology-based reasoning |
| `CapabilityExtractor` | `JsonExtractor` — mechanical parsing of manifest JSON fields | AI manifest reader that infers deeper capabilities ("this service can process payments" even without a field named "payment"); binary static analysis of headless WASM |
| `IdentityVerifier` | `ApiKeyVerifier` — optional X-Api-Key header check | DIDs (decentralized identifiers); cryptographic signatures on manifests; OAuth/OIDC; LDAP for intranet deployments; domain-verified identity (prove you own bakery.com) |
| `FederationSync` | `StubSync` — returns 501 Not Implemented | HTTP pull-based (fetch /export from peer); gossip protocol; IPFS pubsub; CRDTs for conflict-free merging; blockchain-anchored sync proofs |
| `StorageBackend` | `SqliteStorage` — SQLite + filesystem for binaries | Postgres; IPFS (content-addressable, distributed); distributed DHT; S3 for binary storage; in-memory for testing |

### Server Configuration

```bash
# All defaults (simple scorer, SQL matcher, JSON extractor, API key auth, stub sync, SQLite)
naze-discovery --port 8889

# Custom scorer via HTTP webhook
naze-discovery --port 8889 --scorer webhook --scorer-url http://localhost:9000/score

# Semantic capability matching (future)
naze-discovery --port 8889 --matcher semantic --embeddings-url http://localhost:9001/embed

# External identity provider
naze-discovery --port 8889 --identity oauth --oauth-url https://auth.company.com
```

Each `--scorer`, `--matcher`, `--extractor`, `--identity`, `--sync`, and `--storage` flag selects which trait implementation to use. Unspecified flags use the reference defaults.

The **webhook pattern** is the universal escape hatch: for any trait, a webhook implementation can delegate to an external HTTP service that receives the trait's input as JSON and returns the trait's output. This means you can plug in any language, any framework, any AI model — without touching the discovery server code.

## Distributed Readiness

The reference implementation is single-node, but the architecture is designed so nothing prevents building a distributed cluster from the same trait interfaces.

### Design Decisions for Distribution

**Global identity, not local IDs.** The `StorageBackend` trait uses `ServiceRef { domain, name }` everywhere — never local auto-increment integers. `(domain, name)` is globally unique across all nodes. The SQLite reference impl maps these to local `i64` IDs internally, but that mapping never leaks through the trait boundary. A distributed impl can use content hashes, UUIDs, or any other globally-unique scheme.

**Content-addressable binaries.** `manifest_hash` (sha256) and `headless_hash` are already in the spec. Two nodes storing the same manifest will have the same hash. This enables deduplication and integrity verification across nodes without coordination.

**Idempotent registration.** `upsert_service` by `(domain, name)` is naturally idempotent. Replaying the same registration on multiple nodes converges to the same state.

**Compositions reference ServiceRef, not local IDs.** The `compositions` table stores JSON arrays of `{domain, name}` pairs. A composition pattern discovered on Node A is directly mergeable with Node B because it uses globally-meaningful identifiers.

**Export supports delta sync.** `export_public_services(since: Option<&str>)` enables incremental sync — a peer only needs changes since its last sync, not the full dataset every time.

### What a Distributed Implementation Would Add

These are NOT in the reference impl but the architecture doesn't block them:

| Concern | Approach | Why the Spec Allows It |
|---------|----------|----------------------|
| **Conflict resolution** | Last-write-wins by `updated_at` timestamp, or version vectors | ServiceRecord includes timestamps; upsert semantics are already defined |
| **Tombstones** | Deactivation records that sync to peers | `active=0` + `deactivated_at` timestamp propagates via `/export` |
| **Observation dedup** | Optional `observation_id` (UUID) for at-least-once delivery | `Observation` struct can include an optional idempotency key |
| **Incremental sync** | `?since=<timestamp>` on `/export` | Already in the `StorageBackend` trait: `export_public_services(since)` |
| **Consistency model** | Eventual consistency (observations are append-only, trust is recomputable, services are upserted) | No operation requires strong consistency or distributed locking |
| **Node identity** | Each node has `network_id` already | `/info` endpoint exposes node identity for peer coordination |
| **Partition tolerance** | Each node operates independently; sync catches up when connectivity returns | All four discovery mechanisms (per-domain, capability index, federated, P2P) work independently |

### What the Reference Impl Deliberately Avoids

- No distributed consensus protocol (Raft, Paxos)
- No vector clocks or CRDTs (but the data model is CRDT-friendly: counters are mergeable, sets are union-able)
- No gossip protocol
- No sharding

These are implementation concerns for a production distributed system, not API contract concerns. The trait interfaces are agnostic to all of them.

## Deployment Scopes

The same server binary supports both public and private deployment:

- **Public** — Internet-wide discovery network. Any agent can register and discover services. Federation peers connect freely.
- **Private / Intranet** — A company runs its own instance for internal services (payroll, QA tools, docs). Registration requires an API key. Services are `internal` by default and never exported to peers.
- **Hybrid** — A private network peers with the public network. Internal services stay private; services marked `public` are exported during federation sync. The company's payroll system is discoverable internally but invisible to the outside world.

This is modeled through three mechanisms:
1. **Service visibility** (`internal` vs `public`) — controls what gets exported to peers
2. **Network identity** — each server declares its `network_id` and `scope` in `/info`
3. **Optional API key auth** — configurable via CLI flag; when enabled, registration and optionally discovery require `X-Api-Key` header

### Server Configuration

```bash
# Public server (no auth, anyone can register)
naze-discovery --port 8889

# Private intranet server (API key required for registration)
naze-discovery --port 8889 --api-key "company-secret-key"

# Private with separate read/write keys
naze-discovery --port 8889 --write-key "writers-only" --read-key "readers-only"

# Declare network identity
naze-discovery --port 8889 --network-id "acme-corp" --scope private
```

## Crate Structure

**Fully self-contained.** `naze-discovery` has zero dependencies on any other naze workspace crate — no `naze-parser`, `naze-compiler`, `naze-ir`, or anything else. It depends only on external crates (axum, rusqlite, serde, etc.). The directory can be extracted into a standalone repo and it compiles independently.

The connection to the naze ecosystem is purely over HTTP: `nazec` has a `discovery_client.rs` that talks to the discovery server via its JSON API, not as a Rust library dependency. Any language, any tool can interact with the discovery server — it's just an HTTP service.

```
crates/naze-discovery/
  Cargo.toml
  src/
    main.rs         -- clap CLI, Axum server startup (port, auth, network identity, trait selection)
    api.rs          -- Router + 26 JSON API handlers
    types.rs        -- Shared serde request/response types + trait input/output structs
    traits.rs       -- All 6 trait definitions (TrustScorer, CapabilityMatcher, etc.)
    dashboard.rs    -- Built-in web dashboard (embedded HTML/CSS/JS)

    # Reference implementations (one per trait)
    storage_sqlite.rs   -- StorageBackend impl: SQLite + filesystem
    trust_simple.rs     -- TrustScorer impl: pattern-match + linear weights (simple-v1)
    matcher_sql.rs      -- CapabilityMatcher impl: SQL LIKE + INTERSECT
    extractor_json.rs   -- CapabilityExtractor impl: mechanical JSON field parsing
    identity_apikey.rs  -- IdentityVerifier impl: optional X-Api-Key header
    sync_stub.rs        -- FederationSync impl: returns 501 Not Implemented
```

Dependencies (all external, no workspace crates):
- axum 0.7, tokio 1, rusqlite 0.33 (bundled), tower-http 0.5
- serde, serde_json, clap 4, sha2, chrono 0.4

## SQLite Schema (7 tables, 3 layers)

### Storage Layer

**services** — Core service registration. One row per domain+service pair.

```sql
CREATE TABLE services (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    domain          TEXT NOT NULL,           -- e.g. "bakery.example.com"
    name            TEXT NOT NULL,           -- from manifest "name" field
    version         TEXT NOT NULL DEFAULT '0.1.0',
    manifest_hash   TEXT NOT NULL,           -- sha256 of the manifest JSON
    manifest_path   TEXT NOT NULL,           -- filesystem path to stored manifest
    headless_hash   TEXT,                    -- sha256 of headless binary (optional)
    headless_path   TEXT,                    -- filesystem path to headless binary
    registered_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    visibility      TEXT NOT NULL DEFAULT 'public',  -- "public" or "internal"
    active          INTEGER NOT NULL DEFAULT 1,      -- 0 = deactivated by flags
    UNIQUE(domain, name)
);
```

**capabilities** — Indexed capabilities extracted from manifests at registration time.

```sql
CREATE TABLE capabilities (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id      INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL,       -- "state_field", "server_function", "action", "data_source", "model_field"
    name            TEXT NOT NULL,       -- e.g. "price", "order", "search"
    value_type      TEXT,                -- e.g. "number", "text", "list", "bool"
    metadata        TEXT                 -- JSON blob for kind-specific details
);
CREATE INDEX idx_capabilities_kind_name ON capabilities(kind, name);
CREATE INDEX idx_capabilities_service ON capabilities(service_id);
```

**trust_profiles** — Parametric trust profile definitions.

```sql
CREATE TABLE trust_profiles (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL UNIQUE,    -- "default", "healthcare", "ecommerce", "iot", "finance", "education"
    weights         TEXT NOT NULL            -- JSON: {"external_domains": 0.3, "personal_data": 0.3, "device_apis": 0.2, "data_flow": 0.2}
);
```

**trust_scores** — Precomputed trust scores per service per profile.

```sql
CREATE TABLE trust_scores (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id      INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    profile_id      INTEGER NOT NULL REFERENCES trust_profiles(id),
    score           REAL NOT NULL,          -- 0.0 to 1.0
    breakdown       TEXT NOT NULL,          -- JSON: {"external_domains": 0.95, "personal_data": 1.0, ...}
    computed_at     TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(service_id, profile_id)
);
```

### Observation Layer

**observations** — Signals emitted by agents: discoveries, compositions, flags.

```sql
CREATE TABLE observations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    observation_id  TEXT,                    -- optional UUID for distributed dedup (idempotency key)
    kind            TEXT NOT NULL,           -- "discovery", "composition", "flag", "usage", "health_check"
    service_id      INTEGER REFERENCES services(id),
    agent_id        TEXT,                    -- optional agent identifier
    payload         TEXT NOT NULL,           -- JSON: kind-specific data
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(observation_id)                   -- dedup key (NULL allowed, only non-NULL values are unique)
);
CREATE INDEX idx_observations_kind ON observations(kind);
CREATE INDEX idx_observations_service ON observations(service_id);
```

**compositions** — Which services get used together.

```sql
CREATE TABLE compositions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    service_refs    TEXT NOT NULL,           -- JSON array of {domain, name} pairs, sorted
    frequency       INTEGER NOT NULL DEFAULT 1,
    first_seen      TEXT NOT NULL DEFAULT (datetime('now')),
    last_seen       TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(service_refs)
);
```

### Federation Layer

**peers** — Known peer discovery nodes.

```sql
CREATE TABLE peers (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    url             TEXT NOT NULL UNIQUE,    -- base URL of peer discovery server
    name            TEXT,                    -- human-readable label
    trust_profile   TEXT,                    -- which trust profile this peer uses
    last_sync       TEXT,                    -- last successful sync timestamp
    active          INTEGER NOT NULL DEFAULT 1,
    added_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Seed Data

On `init_schema()`, insert 6 default trust profiles:

| Profile | external_domains | personal_data | device_apis | data_flow |
|---------|-----------------|---------------|-------------|-----------|
| default | 0.25 | 0.25 | 0.25 | 0.25 |
| healthcare | 0.20 | 0.40 | 0.10 | 0.30 |
| ecommerce | 0.30 | 0.30 | 0.20 | 0.20 |
| iot | 0.20 | 0.30 | 0.10 | 0.40 |
| finance | 0.20 | 0.35 | 0.10 | 0.35 |
| education | 0.20 | 0.50 | 0.10 | 0.20 |

## API Endpoints (22 total)

### Health & Info

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness check |
| GET | `/api/v1/discovery/info` | Server metadata: network_id, scope, service count, peer count, profiles |

### Service Registration

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/v1/discovery/services` | Register/update a service |
| GET | `/api/v1/discovery/services/:domain/:name` | Get service details + trust scores |
| DELETE | `/api/v1/discovery/services/:domain/:name` | Deactivate a service |
| GET | `/api/v1/discovery/services/:domain/:name/manifest` | Download stored manifest JSON |
| GET | `/api/v1/discovery/services/:domain/:name/headless` | Download headless binary |

**POST register** request:
```json
{
  "domain": "bakery.example.com",
  "manifest": { ... },
  "headless": "<optional base64-encoded headless binary>",
  "visibility": "public"
}
```

On registration, the server:
1. Validates required manifest fields (name, state, actions)
2. Computes sha256 of manifest JSON
3. Extracts capabilities into the capabilities table
4. Stores manifest on filesystem
5. Computes trust scores against all profiles
6. Returns service record with trust scores and indexed capability count

### Capability Discovery (core API)

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/v1/discovery/search` | Structural capability matching |
| GET | `/api/v1/discovery/search?q=...` | Text fallback search |

**POST search** — the core API. Agents describe what they need structurally:

```json
{
  "require": [
    {"kind": "state_field", "name_like": "%price%", "value_type": "number"},
    {"kind": "server_function", "name_like": "%order%"}
  ],
  "prefer": [
    {"kind": "state_field", "name_like": "%location%"}
  ],
  "trust_profile": "ecommerce",
  "min_trust": 0.7,
  "limit": 10
}
```

Each `require` matcher → SQL subquery on capabilities table. Multiple requires → INTERSECT (AND semantics). Results ranked by `trust_score * (1 + 0.1 * preferred_match_count)`.

**Response:**
```json
{
  "results": [
    {
      "service_id": 42,
      "domain": "bakery.example.com",
      "name": "Cake Shop",
      "version": "0.1.0",
      "trust_score": 0.92,
      "matched_capabilities": [
        {"kind": "state_field", "name": "price", "value_type": "number"},
        {"kind": "server_function", "name": "order"}
      ],
      "preferred_matches": 1,
      "manifest_url": "/api/v1/discovery/services/bakery.example.com/Cake%20Shop/manifest"
    }
  ],
  "total": 1
}
```

### Trust Scoring

| Method | Path | Purpose |
|--------|------|---------|
| GET | `.../trust` | All trust scores for a service |
| GET | `.../trust/:profile` | Score for one profile |
| GET | `/api/v1/discovery/profiles` | List available profiles |
| POST | `/api/v1/discovery/profiles` | Create custom profile |

### Observation Signals

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/v1/discovery/observe` | Record discovery/usage signal |
| POST | `/api/v1/discovery/flag` | Flag misbehaving service |
| POST | `/api/v1/discovery/compose` | Record composition event |
| GET | `.../signals` | Observation summary for a service |

**Flagging logic:** Each flag reduces trust score. After N flags (default 5), service's `active` field → 0, excluded from search results.

**Composition:** Upserts into compositions table, incrementing frequency and updating `last_seen`.

### Emergence

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/discovery/patterns` | Top composition patterns by frequency |
| GET | `/api/v1/discovery/trending` | Services with rising discovery rates in a time window |

### Federation (stubs)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/v1/discovery/peers` | List known peers |
| POST | `/api/v1/discovery/peers` | Add a peer |
| DELETE | `/api/v1/discovery/peers/:id` | Remove a peer |
| POST | `/api/v1/discovery/peers/:id/sync` | Returns 501 (stub — documents expected sync behavior) |
| GET | `/api/v1/discovery/export` | Bulk JSON export of `visibility: "public"` services only (for peer sync) |

## Trust Scoring — Reference Implementation (`simple-v1`)

The `TrustScorer` trait and contract are defined in the [Architecture section above](#architecture-pluggable-trait-interfaces). This section describes the `simple-v1` reference implementation.

The built-in scorer uses four signals extracted from the manifest at registration time:

1. **external_domain_count** — unique external domains in data sources. More external domains → lower score.
2. **personal_data_score** — state field names matching PII patterns (email, phone, ssn, password, credit_card, address). More PII fields → lower score.
3. **device_api_score** — device API data sources (geolocation, camera, accelerometer). More device APIs → lower score.
4. **data_flow_score** — outbound POST/PUT server functions and external fetches. More outbound data flow → lower score.

Each signal produces a raw score 0.0-1.0 (higher = more trustworthy):
```
raw_signal = 1.0 - (penalty / max_penalty)
```

Base score per profile:
```
base_score = sum(signal_i * weight_i)
```

The same signals feed every trust profile; only the weights differ. A mapping service scoring well in ecommerce (location access expected) might score lower in a privacy-focused profile.

Dynamic adjustment (from `ObservationSignals`):
```
adjustment = usage_boost + discovery_boost + composition_boost - flag_penalty - staleness_decay - source_penalty
score = clamp(base_score + adjustment, 0.0, 1.0)
```

This is deliberately simple. A more sophisticated scorer could replace every part of this computation while keeping the same API contract.

## Capability Extraction — Reference Implementation (`JsonExtractor`)

The `CapabilityExtractor` trait is defined in the [Architecture section](#architecture-pluggable-trait-interfaces). The reference `JsonExtractor` mechanically parses manifest JSON fields:

- `manifest.state` fields → kind="state_field", with name and type
- `manifest.server_functions` → kind="server_function"
- `manifest.actions` (event handlers) → kind="action"
- `manifest.data_sources` → kind="data_source"
- `manifest.models` fields → kind="model_field"

A smarter `CapabilityExtractor` could infer capabilities semantically (e.g., recognizing that a service with `state.total` + `action.add_to_cart` + `fn.checkout` has a "shopping" capability even if no field is literally named "shopping").

## Relationship to the Three-Projection Build

The [DISCOVERY_NETWORK.md](./DISCOVERY_NETWORK.md) describes three outputs from every `nazec build`:

1. **Full app** (`app_data.bin`) — complete application with UI
2. **Manifest** (`naze-manifest.json`) — machine-readable capability description
3. **Headless binary** (`headless.bin`) — Layer 1 only, pure computation, no UI

**Current compiler status:** Only the full app (`app_data.bin`) is implemented today. Neither manifest generation nor headless binary extraction exist yet (see `AGENT_RUNTIME_PLAN.md` for the implementation plan).

**The discovery server does NOT depend on the three-projection build.** It works today because:

- **Manifest:** The registration API accepts any JSON blob as the manifest. For `nazec announce`, we generate it from `ProjectContext` (which `nazec context` already produces from the AST). This is not the formal `naze-manifest.json` spec from `AGENT_RUNTIME_PLAN.md` — it's a simpler derivation that's good enough to prove the discovery concept. When the proper manifest generator is built, `nazec announce` simply sends a richer JSON blob.

- **Headless binary:** Already `Optional` in the registration request. The discovery server indexes capabilities from the manifest alone. When the compiler eventually supports `nazec build --headless`, agents get the bonus of downloadable headless binaries — but discovery works without them.

- **Full app binary:** The discovery server never stores or needs this. It only cares about manifests (for capability indexing and trust scoring) and optionally headless binaries (for agent execution).

**Progression path:**
1. **Now:** `nazec announce` generates manifest from `ProjectContext`. No headless binary. Discovery works.
2. **Later:** `nazec build --manifest` emits proper `naze-manifest.json`. `nazec announce` sends it. Richer capability extraction.
3. **Eventually:** `nazec build --headless` emits headless binary. `nazec announce` sends both. Agents can download and execute headless binaries.

Each step enriches the discovery network without breaking anything — the API stays the same, the data just gets richer.

## CLI Integration

### `nazec announce`

```bash
# Public announcement
nazec announce --domain bakery.example.com [--server http://localhost:8889]

# Internal service (not exported to peers)
nazec announce --domain payroll.acme.internal --visibility internal --api-key "company-key"
```

1. Load naze.toml, resolve project (same as `nazec context`)
2. Generate manifest JSON from `ProjectContext` (reuses `context.rs::extract_context()`)
3. POST manifest to discovery server (with visibility and optional API key)
4. Optionally include headless binary if `dist/headless.bin` exists
5. Print returned trust scores

URL resolution: `--server` flag → `NAZE_DISCOVERY_URL` env → default `http://localhost:8889`
API key resolution: `--api-key` flag → `NAZE_DISCOVERY_KEY` env

### `nazec discover`

```bash
nazec discover "fn:order,state:price:number" [--profile ecommerce] [--min-trust 0.7] [--limit 10]
```

Shorthand query syntax:
- `state:price:number` → `{kind: "state_field", name: "price", value_type: "number"}`
- `fn:order` → `{kind: "server_function", name_like: "%order%"}`
- `action:append` → `{kind: "action", name: "append"}`
- `data:fetch` → `{kind: "data_source", name_like: "%fetch%"}`

## Built-in Dashboard

The discovery server includes a built-in web dashboard served at the root URL (`/`). It's embedded into the binary (no external files needed) and calls the same JSON APIs that agents use — making it both a visual monitoring tool and a live testing surface.

### Dashboard Pages

**Overview (`/`)** — Network health at a glance
- Service count (active / inactive / total)
- Observation count by kind (discoveries, usages, flags, compositions)
- Trust score distribution histogram
- Recent activity feed (last 20 observations)
- Network identity (network_id, scope, peer count)

**Services (`/ui/services`)** — Browse and inspect registered services
- Sortable table: name, domain, version, trust score, capabilities count, visibility, publisher
- Color-coded trust scores (green ≥0.8, yellow ≥0.5, red <0.5)
- Filter by: active/inactive, visibility, trust profile, capability kind
- Click through to service detail

**Service Detail (`/ui/services/:domain/:name`)** — Deep dive into a single service
- Trust scores across all profiles with breakdown bars
- Trust history graph (base score + adjustments over time)
- Capabilities list with types and metadata
- Version history timeline
- Provenance graph (what this was composed from / what composes this)
- Observation log (discoveries, usages, flags)
- Raw manifest JSON viewer

**Trust Dynamics (`/ui/trust`)** — Promotions and demotions
- Services with rising trust (recently boosted by usage/compositions)
- Services with falling trust (recently flagged or stale)
- Recently deactivated services with flag reasons
- Trust cascade visualization (flagged service → downstream impact)

**Patterns (`/ui/patterns`)** — Composition patterns and emergence
- Top composition patterns by frequency
- Trending compositions (rising frequency in recent window)
- Promoted pattern templates (crossed frequency threshold)
- Pattern detail: click to see component services and their trust scores

**Observations (`/ui/observations`)** — Activity feed and analytics
- Filterable stream: kind, service, agent_id, time range
- Aggregated charts: observations per hour/day, flags over time
- Agent activity: which agents are most active (discovery, composition, flagging)

**Test Console (`/ui/test`)** — Interactive API testing
- **Register:** Paste a manifest JSON, set domain/visibility, submit → see extracted capabilities and trust scores
- **Search:** Build a structural query with form fields (add require/prefer matchers), execute → see results
- **Flag:** Select a service, enter reason/evidence, submit
- **Observe:** Record a test observation signal
- All actions hit the real API — the test console is a GUI over the JSON endpoints

### Implementation Approach

- **Embedded HTML:** Single-page app with vanilla HTML/CSS/JS, embedded in the binary via `include_str!()` (same pattern as `nazec dev` and `nazec playground`)
- **Served at `/`:** The dashboard routes are registered alongside the API routes in `api.rs`
- **No build step:** Plain HTML + CSS + vanilla JS. No npm, no bundler. The reference impl prioritizes simplicity.
- **Auto-refresh:** Dashboard polls `/api/v1/discovery/info` every 5 seconds for live stats. Observation feed uses polling.
- **`--no-dashboard` flag:** Disable the UI for headless/production deployments

### Dashboard Routes

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/` | Dashboard overview page |
| GET | `/ui/services` | Services browser |
| GET | `/ui/services/:domain/:name` | Service detail page |
| GET | `/ui/trust` | Trust dynamics view |
| GET | `/ui/patterns` | Composition patterns |
| GET | `/ui/observations` | Activity feed |
| GET | `/ui/test` | Interactive API test console |
| GET | `/ui/assets/:file` | Static assets (CSS, JS) |

The dashboard HTML/CSS/JS is contained in `dashboard.rs` as `include_str!()` constants. All data fetching is done client-side via `fetch()` calls to the JSON API.

## System Flows

### Flow 1: Developer Publishes a Service

```
Developer                          Discovery Server
    |                                     |
    |-- nazec build --manifest ---------->|  (generates naze-manifest.json + headless binary locally)
    |-- nazec announce --domain X ------->|  POST /services {domain, manifest, headless, visibility}
    |                                     |-- validate manifest fields
    |                                     |-- compute sha256 hashes
    |                                     |-- extract capabilities → capabilities table
    |                                     |-- store manifest + binary on filesystem
    |                                     |-- compute trust scores against all profiles
    |<-- {service_id, trust_scores} ------|
```

**API:** POST `/api/v1/discovery/services`

### Flow 2: Agent Discovers Services

```
Agent                              Discovery Server
    |                                     |
    |-- POST /search {require, prefer} -->|
    |                                     |-- structural match on capabilities (SQL INTERSECT)
    |                                     |-- filter by min_trust, active=1, visibility scope
    |                                     |-- rank by trust_score * preferred_match_bonus
    |<-- {results: [{domain, name,        |
    |     trust_score, matched_caps,      |
    |     manifest_hash, headless_hash}]}-|
    |                                     |
    |-- POST /observe {kind: "discovery"}>|  (record signal)
    |                                     |-- update discovery_count for service
    |                                     |-- feed into dynamic trust adjustment
```

**API:** POST `/api/v1/discovery/search`, POST `/api/v1/discovery/observe`

Search results include `manifest_hash` and `headless_hash` so agents can verify content integrity after download.

### Flow 3: Agent Pulls and Executes a Headless Binary

```
Agent                              Discovery Server
    |                                     |
    |-- GET /services/:domain/:name/      |
    |       headless ---------------------->|
    |<-- binary blob (+ Content-Hash hdr)-|
    |                                     |
    |-- verify sha256 matches             |
    |   headless_hash from search results |
    |                                     |
    |-- execute in WASM sandbox           |
    |-- use result                        |
    |                                     |
    |-- POST /observe {kind: "usage",     |
    |   payload: {success: true,          |
    |             duration_ms: 12}} ------>|
```

**API:** GET `.../headless`, POST `/api/v1/discovery/observe`

Content integrity: the `headless_hash` returned in search results lets the agent verify the binary before execution. The server also returns a `X-Content-Hash` header on binary downloads.

### Flow 4: Agent Composes a New Service

```
Agent                              Discovery Server
    |                                     |
    |-- discover bakery, venue, catering  |
    |-- download manifests + headless     |
    |-- compose new "party planner" .naze |
    |-- compile → new manifest + binary   |
    |                                     |
    |-- POST /services {                  |
    |     domain: "agent-composed.local", |
    |     manifest: {...},                |
    |     headless: "base64...",          |
    |     composed_from: [                |  ← provenance
    |       {domain: "bakery.com",        |
    |        name: "Cake Shop"},          |
    |       {domain: "venue.com",         |
    |        name: "Event Venue"},        |
    |       {domain: "catering.com",      |
    |        name: "Catering Co"}         |
    |     ],                              |
    |     publisher: "agent:claude-xyz"   |  ← agent identity
    |   } -------------------------------->|
    |                                     |-- index as normal service
    |                                     |-- store provenance links
    |                                     |-- inherit partial trust from sources
    |                                     |
    |-- POST /compose {service_refs} ---->|  (record composition pattern)
    |                                     |-- upsert compositions table
```

**New fields on registration:**
- `composed_from: Vec<ServiceRef>` — provenance: what services this was built from
- `publisher: Option<String>` — identity of who published (human or agent)

**New table:**

```sql
CREATE TABLE provenance (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id      INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    source_domain   TEXT NOT NULL,
    source_name     TEXT NOT NULL,
    UNIQUE(service_id, source_domain, source_name)
);
```

### Flow 5: Agent Flags a Bad Service

```
Agent                              Discovery Server
    |                                     |
    |-- use a service, behavior doesn't   |
    |   match manifest                    |
    |                                     |
    |-- POST /flag {                      |
    |     service_domain, service_name,   |
    |     reason: "manifest_mismatch",    |
    |     evidence: "claims no external   |
    |       domains but sends data to     |
    |       tracker.adnetwork.com",       |
    |     agent_id: "agent:claude-xyz"    |
    |   } -------------------------------->|
    |                                     |-- record flag observation
    |                                     |-- apply trust penalty (see trust adjustment)
    |                                     |-- if flag_count >= threshold → deactivate
    |                                     |-- if deactivated, flag downstream services
    |                                     |   (anything composed_from this service)
```

**API:** POST `/api/v1/discovery/flag`

**Cascade:** When a service is deactivated, services that list it in `composed_from` get a trust penalty (their source was flagged). They aren't auto-deactivated, but their trust drops — the network's immune response propagates through the dependency graph.

### Flow 6: Dynamic Trust Adjustment

Trust is NOT static. It's computed at registration and then **adjusted over time** from observation signals:

```
                Initial Score
                     |
    ┌────────────────┼────────────────┐
    ▼                ▼                ▼
  Usage           Flags           Staleness
  signals         signals          decay
    |                |                |
  boost(+)       penalize(-)     decay(-)
    |                |                |
    └────────────────┼────────────────┘
                     ▼
              Adjusted Score
```

**Trust adjustment rules:**

Positive signals (boosts):
- **Usage boost:** Each N usage observations → small trust boost (capped at +0.1 over base). Active usage is the strongest positive signal — a service agents keep using is proven.
- **Discovery boost:** Being discovered frequently → minor boost (capped at +0.05)
- **Composition boost:** Being part of a frequently-used composition → +0.05 (capped)

Negative signals (penalties):
- **Flag penalty:** Each flag → trust penalty of -0.1 (uncapped — flags can drive score to 0)
- **Source flag penalty:** A service in `composed_from` gets flagged → -0.05 per flagged source
- **Staleness decay:** Measured by **last activity** (last usage, discovery, OR update) — NOT just last code update. A bakery service that works perfectly and is used by agents daily is stable, not stale. Staleness only applies when *nothing* is happening: no usage, no discoveries, no updates for 90 days → -0.01/week decay (capped at -0.2). Active usage resets the staleness clock.

**The key insight:** unchanged code + active usage = stable and trusted. Unchanged code + zero usage = potentially abandoned. The network rewards services that work, not services that change.

```
                    Last Activity = max(last_usage, last_discovery, last_update)
                            |
                    days_since_activity > 90?
                      /              \
                    NO                YES
                    |                  |
              no decay           -0.01/week
              (stable)           (abandoned)
```

**Implementation:** A `recompute_dynamic_trust()` function runs:
- On every flag (immediate)
- Periodically via a background task (every hour in the reference impl) for usage/staleness adjustments

**New column on services:**
```sql
ALTER TABLE services ADD COLUMN last_activity TEXT;  -- max(last usage, discovery, update)
```

Updated on every usage/discovery observation and on re-registration. The staleness check queries this field, not `updated_at`.

**New column on trust_scores:**
```sql
ALTER TABLE trust_scores ADD COLUMN base_score REAL;    -- from manifest analysis
ALTER TABLE trust_scores ADD COLUMN adjustment REAL DEFAULT 0.0;  -- from observations
-- score = clamp(base_score + adjustment, 0.0, 1.0)
```

### Flow 7: Service Version Update

```
Developer                          Discovery Server
    |                                     |
    |-- update .naze source               |
    |-- nazec announce --domain X ------->|  POST /services (same domain+name)
    |                                     |-- detect manifest_hash changed
    |                                     |-- archive previous version
    |                                     |-- re-extract capabilities
    |                                     |-- recompute trust (base scores reset)
    |                                     |-- preserve observation history
    |<-- {version: "0.2.0", ...} ---------|
```

**New table for version history:**

```sql
CREATE TABLE service_versions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id      INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    version         TEXT NOT NULL,
    manifest_hash   TEXT NOT NULL,
    manifest_path   TEXT NOT NULL,
    headless_hash   TEXT,
    headless_path   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(service_id, version)
);
```

**New endpoint:**
| Method | Path | Purpose |
|--------|------|---------|
| GET | `.../versions` | List all versions of a service |
| GET | `.../versions/:version/manifest` | Download manifest for a specific version |

### Flow 8: Emergence — Patterns Become Discoverable

```
Discovery Server (background)
    |
    |-- periodically scan compositions table
    |-- compositions with frequency > threshold (e.g., 50)
    |   become "pattern templates"
    |
    |-- pattern templates are searchable via /search
    |   with kind="pattern"
    |
    |-- POST /search {require: [{kind: "pattern",
    |     name_like: "%party%"}]}
    |   returns pattern templates alongside regular services
```

**New table:**

```sql
CREATE TABLE pattern_templates (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    composition_id  INTEGER NOT NULL REFERENCES compositions(id),
    name            TEXT,                   -- auto-generated or agent-provided: "party-planner-combo"
    description     TEXT,                   -- auto-generated from component names
    promoted_at     TEXT NOT NULL DEFAULT (datetime('now')),
    discovery_count INTEGER NOT NULL DEFAULT 0
);
```

Pattern templates surface in `/search` results when they match the query. They include the service list so agents can pull all components at once.

### Flow 9: Health Check / Liveness (Per-Domain Verification)

```
Discovery Server (background)
    |
    |-- for each registered service with a real domain:
    |   fetch https://{domain}/.well-known/naze-manifest.json
    |
    |-- compare fetched manifest_hash with stored manifest_hash
    |   ├── match → service is live and consistent (boost trust)
    |   ├── mismatch → manifest drift (flag for review)
    |   └── unreachable → mark as stale (staleness decay kicks in)
    |
    |-- record observation: kind="health_check"
```

**New observation kind:** `"health_check"` with payload `{status: "live"|"drift"|"unreachable"}`.

**New server flag:** `--enable-health-checks` (disabled by default in reference impl). Private/intranet servers might not want this.

### Flow 10: Federation Sync (Full Flow — Stub)

```
Server A (private)                 Server B (public)
    |                                     |
    |-- POST /peers {url: B} ------------>|  (A registers as peer of B)
    |                                     |
    |-- POST /peers/:B/sync ------------>|  (triggers sync)
    |                                     |
    |<-- 501 Not Implemented ------------|  (stub in reference impl)
    |                                     |
    |   --- What sync WOULD do: ---       |
    |   1. GET /export from B             |
    |      (returns only visibility:public)|
    |   2. Merge into local DB            |
    |      (dedup by manifest_hash)       |
    |   3. Recompute local trust scores   |
    |   4. Send local public services to B|
    |   5. Exchange observation summaries  |
    |      (composition frequency, flags) |
```

## Summary of Changes from Flow Analysis

### New Schema (3 additional tables → 10 total)

| Table | Purpose |
|-------|---------|
| **provenance** | Tracks which services a composed service was built from |
| **service_versions** | Version history for rollback and auditability |
| **pattern_templates** | Composition patterns promoted to discoverable entities |

### New/Modified Columns

| Table | Column | Change |
|-------|--------|--------|
| services | publisher | `TEXT` — identity of who published (human or agent) |
| trust_scores | base_score | `REAL` — original score from manifest analysis |
| trust_scores | adjustment | `REAL DEFAULT 0.0` — dynamic adjustment from observations |

### New API Endpoints (+4 → 26 total)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `.../versions` | List version history for a service |
| GET | `.../versions/:version/manifest` | Download manifest for specific version |
| GET | `.../provenance` | Get composition lineage for a service |
| POST | `/api/v1/discovery/recompute` | Trigger trust recomputation (admin) |

### Modified Registration Request

```json
{
  "domain": "agent-composed.local",
  "manifest": { ... },
  "headless": "<base64>",
  "visibility": "public",
  "composed_from": [
    {"domain": "bakery.com", "name": "Cake Shop"},
    {"domain": "venue.com", "name": "Event Venue"}
  ],
  "publisher": "agent:claude-xyz"
}
```

### Modified Search Response

```json
{
  "results": [{
    "service_id": 42,
    "domain": "bakery.example.com",
    "name": "Cake Shop",
    "version": "0.1.0",
    "trust_score": 0.92,
    "manifest_hash": "a1b2c3...",
    "headless_hash": "d4e5f6...",
    "matched_capabilities": [...],
    "preferred_matches": 1,
    "publisher": "human:jason@bakery.com",
    "composed_from_count": 0,
    "manifest_url": "..."
  }],
  "total": 1
}
```

## Implementation Order

1. **Crate scaffolding** — Cargo.toml, main.rs, empty module stubs → compiles
2. **types.rs** — All shared serde structs + trait input/output types (TrustInput, TrustOutput, Capability, ObservationSignals, etc.)
3. **traits.rs** — All 6 trait definitions
4. **storage_sqlite.rs** — `StorageBackend` impl: 10-table SQLite schema, CRUD, seed data, in-memory tests
5. **extractor_json.rs** — `CapabilityExtractor` impl: mechanical JSON field parsing
6. **trust_simple.rs** — `TrustScorer` impl: signal extraction, parametric scoring, dynamic adjustment
7. **matcher_sql.rs** — `CapabilityMatcher` impl: SQL LIKE + INTERSECT structural matching
8. **identity_apikey.rs** — `IdentityVerifier` impl: optional X-Api-Key header check
9. **sync_stub.rs** — `FederationSync` impl: returns 501
10. **api.rs** — All 26 JSON API handlers, wired to traits via AppState, integration tests
11. **dashboard.rs** — Built-in web UI (embedded HTML/CSS/JS, 7 pages)
12. **CLI** — `announce`, `discover` commands + `discovery_client.rs` + `--manifest` flag
13. **End-to-end** — Start server, open dashboard, announce an example, discover it, verify in UI

## Key Files

### Existing (reuse)
- `crates/nazec/src/context.rs` — `extract_context()` + `ProjectContext` types for manifest generation
- `crates/naze-registry/src/` — Pattern for Axum+SQLite server, test helpers, AppState
- `crates/nazec/src/registry.rs` — Pattern for HTTP client (discovery_client.rs)

### New
- `crates/naze-discovery/src/` — 12 source files (traits.rs, types.rs, api.rs, dashboard.rs, + 6 reference impls)
- `crates/nazec/src/discovery_client.rs` — HTTP client for discovery server

### Modified
- `Cargo.toml` (workspace) — Add `crates/naze-discovery` to members + default-members
- `crates/nazec/src/cli.rs` — Add `Announce` and `Discover` commands, `--manifest` on Build
- `crates/nazec/src/main.rs` — Dispatch new commands

## Security Considerations

This section documents known adversarial threats against a discovery network. The reference implementation includes basic mitigations where noted; production/distributed implementations should address all of these.

### Threat Model

#### 1. Trust Score Gaming

**Self-boosting:** An attacker registers a service, then uses sock puppet agents to generate fake "discovery" and "usage" observations, inflating trust scores artificially.

| | |
|---|---|
| **Impact** | High — undermines the core trust mechanism |
| **Reference mitigation** | None (single-node, no agent identity verification) |
| **Production mitigation** | Rate-limit observations per agent_id per service per time window. Verify that agent_id represents a real, distinct agent — not a sock puppet (see "Observer Identity" below). Detect anomalous observation patterns (sudden spike from unknown agents). Weight observations by the observing agent's own reputation. |

**Flag abuse:** Mass-flagging a competitor's legitimate service to drive down trust or trigger deactivation.

| | |
|---|---|
| **Impact** | High — legitimate services get silenced |
| **Reference mitigation** | Threshold-based deactivation (requires N flags, not just 1) |
| **Production mitigation** | Weight flags by flagger reputation. Require evidence with flags. Rate-limit flags per agent_id. Implement appeals/reinstatement process. Don't deactivate on flags alone — human review or consensus required. |

#### 2. Search Pollution

**Capability stuffing:** Register a service with every possible capability keyword to appear in all search results (the SEO spam of discovery networks).

| | |
|---|---|
| **Impact** | Medium — degrades search quality |
| **Reference mitigation** | Capabilities are extracted from the manifest by the CapabilityExtractor, not self-declared as free-text. Stuffing requires a bloated manifest that will score poorly on trust. |
| **Production mitigation** | Penalize services with disproportionately many capabilities (a service claiming to do everything probably does nothing well). Cross-reference capabilities against actual binary behavior. |

**Registration flooding:** Millions of fake services to overwhelm storage and dilute search results.

| | |
|---|---|
| **Impact** | Medium — resource exhaustion, search quality degradation |
| **Reference mitigation** | Optional API key for registration |
| **Production mitigation** | Rate-limit registrations per publisher identity. Require domain verification. Proof-of-work for anonymous registration. Storage quotas per publisher. |

#### 3. Identity Spoofing

**Domain squatting:** Registering a service as `google.com` or `bakery.com` when you don't own the domain.

| | |
|---|---|
| **Impact** | High — agents trust the domain identity |
| **Reference mitigation** | None (domain is self-declared) |
| **Production mitigation** | Domain verification via DNS TXT record or .well-known challenge (similar to Let's Encrypt). The IdentityVerifier trait is the extension point — a `DomainVerifier` impl would fetch `https://{domain}/.well-known/naze-verify` to confirm ownership. Federated registries could require domain verification as an enrollment condition. |

**Manifest spoofing:** A clean, honest-looking manifest that doesn't match actual service behavior.

| | |
|---|---|
| **Impact** | High — trust scores become meaningless |
| **Reference mitigation** | Trust scoring analyzes the manifest for suspicious patterns. Health checks (optional) fetch .well-known/naze-manifest.json to detect drift. |
| **Production mitigation** | Binary static analysis of headless WASM (verify it matches manifest claims). Behavioral monitoring — agents that use a service report whether behavior matched expectations. Automated manifest-vs-binary verification as part of registration. |

#### 4. Supply Chain / Composition Attacks

**Poisoned compositions:** An agent composes 2 legitimate services with 1 malicious service, publishes the result as something useful.

| | |
|---|---|
| **Impact** | High — malicious code hidden in a legitimate-looking composition |
| **Reference mitigation** | Provenance tracking shows what a composed service was built from. Trust cascades — if a source is flagged, composed services' trust drops. |
| **Production mitigation** | Analyze composed manifests — the composition should not introduce capabilities not present in its sources (a "party planner" composed from bakery+venue shouldn't suddenly have access to camera/contacts). Verify that composed headless binary is a pure composition, not injected with new behavior. |

**Dependency chain compromise:** A legitimate source service gets compromised after other services were composed from it.

| | |
|---|---|
| **Impact** | High — cascading trust failure |
| **Reference mitigation** | Source flag penalty propagates through provenance graph. Content-addressable hashes mean the composed service still references the old (clean) version's hash. |
| **Production mitigation** | Pin composed services to specific manifest_hash of their sources. Alert composed service publishers when a source is flagged. Automatic trust re-evaluation when any source changes. |

#### 5. Federation Attacks

**Federation poisoning:** A malicious peer syncs fake or malicious services into the network.

| | |
|---|---|
| **Impact** | High — trust infection across nodes |
| **Reference mitigation** | Federation is stubbed (501). Export only includes public services. |
| **Production mitigation** | Peer reputation scores (peers that introduce frequently-flagged services lose trust). Quarantine period for imported services (don't serve them in search until locally verified). Accept services from peers but recompute trust locally — don't trust the peer's scores. Require mutual peer approval (both sides agree to federate). |

#### 6. Privacy

**Agent surveillance:** Mining the observation layer to learn what agents are searching for, building competitive intelligence or user profiles.

| | |
|---|---|
| **Impact** | Medium — privacy violation, chilling effect on agent usage |
| **Reference mitigation** | agent_id is optional on observations. Private/intranet deployments control who can read observations. |
| **Production mitigation** | Differential privacy on observation aggregates. Don't log individual search queries. Anonymize agent_id in stored observations after aggregation. Separate read permissions for observation data vs service data. Option to disable observation recording entirely for privacy-sensitive deployments. |

**Observation correlation:** Even without agent_id, correlating observation timestamps and service patterns to deanonymize agents.

| | |
|---|---|
| **Impact** | Low-Medium |
| **Production mitigation** | Batch observation recording (delay + jitter). Aggregate observations before storage rather than storing individual events. |

### Two Kinds of Identity

The spec has two distinct identity problems that shouldn't be conflated:

**Publisher identity** — "Who is registering this service?" Handled by the `IdentityVerifier` trait. The reference impl uses optional API keys. A production impl might verify domain ownership (prove you own bakery.com) or use cryptographic signing.

**Observer identity** — "Who is submitting this observation?" Currently just an optional `agent_id` string on observations — completely unverified. This is the bigger problem for trust score gaming, because observations directly influence trust scores.

Observer identity levels (from weakest to strongest):

| Level | How It Works | Prevents |
|-------|-------------|----------|
| **None** (reference impl) | `agent_id` is optional free-text. Anyone can claim any ID. | Nothing |
| **API key per agent** | Agents register with the network and get a key. Observations are tied to that key. | One actor cheaply faking thousands of agents |
| **Rate-limited keys** | Each key is rate-limited (N observations per service per hour) | One real agent spamming observations |
| **Reputation-weighted** | New agents' observations count less. Reputation builds over time from consistent, non-flagged behavior. | New sock puppet accounts having immediate impact |
| **Cryptographic signing** | Agents sign observations with their private key. Verifiable without trusting the server. | Forged observations in a distributed network |

The reference impl is at level 0 (none). This is fine for a prototype. But any production deployment where trust scores matter needs at least level 1 (API key per agent) to prevent trivial self-boosting.

Note: observer identity is NOT about knowing who a person is. It's about ensuring "1 observation = 1 real agent interaction" — a sybil resistance problem, not an authentication problem. An anonymous agent with a verified-unique key is fine.

**This is an unsolved problem.** In a world where anyone can spin up an LLM agent in 5 lines of code, there's no technical way to distinguish "1,000 real agents independently found this bakery useful" from "1 person ran a script that created 1,000 agents to boost the bakery." API keys, cryptographic signatures, proof-of-work — all raise the cost of faking identity but none eliminate it. This is the classic sybil problem and no discovery network, decentralized protocol, or reputation system has fully solved it.

Approaches that help but don't solve it: cost-based registration (even $0.01 per agent), reputation that builds slowly over weeks (sock puppets are expensive to maintain long-term), cross-verification with service providers (the bakery confirms the order happened), behavioral anomaly detection (1,000 agents discovering the same service in 1 minute is suspicious), and stake-based systems (agents risk something valuable that gets slashed if caught).

**Our pragmatic mitigation:** Since we can't prevent gaming, we cap its impact. Observation-based trust adjustments are bounded — usage boost is capped at +0.1, composition boost at +0.05. The **manifest-based base score** (which can't be faked without changing the actual code) remains the dominant signal. A clean bakery starts high; a tracker with PII fields and ad domains starts low. No amount of fake observations changes that foundation. The gameable signals can nudge scores slightly; the ungameable signals set the floor.

### Binary Authenticity and Integrity

Two distinct problems: **Is this binary really from the bakery?** (authenticity) and **Was it tampered with after upload?** (integrity).

**Integrity (reference impl handles this):**

The discovery server computes `sha256` of the manifest and headless binary at registration time. These hashes are stored and returned in search results. An agent downloading a binary can verify:

```
sha256(downloaded_binary) == headless_hash from search results
```

This proves the binary hasn't been modified since registration — protects against storage corruption, CDN tampering, or man-in-the-middle on the download. The reference impl returns hashes in search results and sets an `X-Content-Hash` header on binary downloads.

**Authenticity (unsolved in reference impl, documented for production):**

Currently, anyone can register a service claiming to be `bakery.com`. The `manifest_hash` proves the binary hasn't changed, but not that the bakery actually uploaded it. This requires **publisher signing**:

```
Publisher (bakery):
  1. Generate key pair (once)
  2. Publish public key at https://bakery.com/.well-known/naze-pubkey
  3. Sign manifest+binary: signature = sign(private_key, sha256(manifest) + sha256(binary))
  4. Upload to discovery server: manifest + binary + signature

Discovery server:
  5. Store signature alongside hashes
  6. Optionally verify signature at registration by fetching bakery.com's public key

Agent (consumer):
  7. Search → get results with manifest_hash, headless_hash, signature, pubkey_url
  8. Download binary
  9. Verify integrity:    sha256(downloaded) == headless_hash        ✓ not tampered
  10. Fetch pubkey from   https://bakery.com/.well-known/naze-pubkey
  11. Verify authenticity: verify(pubkey, signature, hashes)          ✓ really from bakery
```

This is the same model as code signing (like Apple's notarization or PGP-signed packages). The discovery server doesn't need to be trusted — the signature chain goes directly from publisher to agent, with the server as an untrusted intermediary.

The reference impl stores hashes (integrity) but not signatures (authenticity). The registration request shape already has room for a future `signature` field. The `IdentityVerifier` trait is the extension point — a `SignatureVerifier` impl would check signatures at registration time and a `DomainVerifier` would fetch the public key from `.well-known/naze-pubkey` to confirm domain ownership.

### What the Reference Implementation Does

The reference impl is a prototype, not a hardened production system. It includes these basic protections:

- **Optional API key auth** — gates registration (prevents anonymous flooding)
- **Threshold-based deactivation** — requires N flags, not just 1
- **Trust scoring from manifest analysis** — suspicious patterns score poorly
- **Capabilities extracted, not self-declared** — harder to stuff than free-text
- **Provenance tracking** — composition chains are visible and trust cascades
- **Content hashes** — integrity verification for manifests and binaries
- **Visibility control** — internal services never exported to peers
- **Optional health checks** — detect manifest drift on registered domains

It does NOT include: domain verification, agent identity verification, rate limiting, observation privacy, binary analysis, or peer reputation. These are documented here so future implementations know what to build.

## Testing Strategy

Testing is organized into four layers. All automated tests run via `cargo test` — no external processes, no manual steps, no network dependencies. The in-memory SQLite backend makes every test fast and isolated.

### Layer 1: Unit Tests (per module)

Each reference implementation module has unit tests that exercise its logic in isolation.

**`storage_sqlite.rs` tests** (~15 tests)
- Schema creation and seed data (6 trust profiles present after init)
- Service CRUD: upsert, get, deactivate, list with filters
- Capabilities: replace, query by kind/name/type
- Trust scores: upsert, get, verify base_score + adjustment
- Observations: record, get signals (counts, last_activity)
- Compositions: upsert (increment frequency), get top patterns
- Provenance: set, get, verify links
- Versions: archive, list
- Peers: add, list, remove
- Export: only returns `visibility: "public"` services
- All tests use `StorageBackend` trait methods on `SqliteStorage::open_in_memory()`

**`trust_simple.rs` tests** (~8 tests)
- Base score from clean manifest (no PII, no external domains) → score near 1.0
- Base score from risky manifest (PII fields, many external domains) → score near 0.0
- Profile weighting: same manifest scores differently under "healthcare" vs "ecommerce"
- Usage boost: high usage_count → positive adjustment
- Flag penalty: flags reduce score, multiple flags drive toward 0
- Staleness decay: old last_activity + zero usage → negative adjustment
- Active usage prevents staleness: old last_activity + high usage → no decay
- Source flag penalty: flagged composed_from → negative adjustment

**`extractor_json.rs` tests** (~5 tests)
- Extract state fields with types
- Extract server functions
- Extract actions from event handlers
- Extract data sources
- Empty manifest → empty capabilities list

**`matcher_sql.rs` tests** (~6 tests)
- Single require matcher → returns matching services
- Multiple require matchers → AND semantics (INTERSECT)
- Prefer matchers → boost ranking but don't filter
- Name LIKE pattern matching
- Value type filtering
- No matches → empty results
- min_trust filtering

**`identity_apikey.rs` tests** (~4 tests)
- No key configured → all requests pass
- Key configured, correct key → passes
- Key configured, wrong key → rejected
- Key configured, no key header → rejected

### Layer 2: API Integration Tests (in `api.rs`)

Full HTTP request/response tests using `tower::ServiceExt::oneshot()` — no actual TCP server needed. Each test gets a fresh in-memory database.

```rust
fn test_app() -> Router {
    let storage = SqliteStorage::open_in_memory().unwrap();
    let scorer = SimpleScorer::new();
    let matcher = SqlMatcher::new();
    let extractor = JsonExtractor::new();
    let identity = ApiKeyVerifier::new(None); // no auth
    let sync = StubSync::new();
    router(storage, scorer, matcher, extractor, identity, sync)
}
```

**Health & Info** (~2 tests)
- `GET /health` → 200
- `GET /api/v1/discovery/info` → returns stats with zero services

**Registration lifecycle** (~6 tests)
- Register a service → 201, returns trust scores and capability count
- Get registered service → 200, matches what was registered
- Register duplicate → upserts (updates trust scores)
- Register with headless binary → stored and downloadable
- Register with `visibility: "internal"` → not in export
- Deactivate a service → 200, no longer in search results

**Capability search** (~5 tests)
- Search with no services → empty results
- Register bakery + venue, search for `fn:order` → only bakery matches
- Multiple require matchers → AND semantics
- Prefer matchers → matching service ranked higher
- `min_trust` filtering → low-trust services excluded

**Trust endpoints** (~3 tests)
- Get trust scores for service → all profiles present
- Get trust for specific profile → single score + breakdown
- Create custom trust profile → appears in list

**Observations** (~5 tests)
- Record discovery observation → signal count increments
- Record usage observation → usage count increments, last_activity updates
- Flag a service → trust score decreases
- Flag 5 times → service deactivated
- Composition record → frequency increments

**Patterns & trending** (~2 tests)
- Record compositions → patterns endpoint returns them sorted by frequency
- Record observations → trending endpoint returns services sorted by recent activity

**Federation** (~3 tests)
- Add peer → appears in list
- Remove peer → gone from list
- Sync → returns 501
- Export → returns only public services as JSON array

**Dashboard** (~2 tests)
- `GET /` → 200, contains HTML
- `GET /ui/test` → 200, contains HTML

### Layer 3: Scenario Tests (Flow Verification)

These are integration tests that exercise the complete flows documented in the System Flows section. Each scenario runs multiple API calls in sequence and verifies the system state at each step.

Located in `tests/scenarios.rs` (or `api.rs` as a separate test module):

**Scenario 1: Full publish → discover → use lifecycle**
```rust
#[tokio::test]
async fn test_publish_discover_use_lifecycle() {
    let app = test_app();
    // 1. Register bakery service with manifest
    // 2. Search by capability → find it
    // 3. Record discovery observation
    // 4. Record usage observation
    // 5. Verify trust score boosted by usage
    // 6. Verify signals endpoint shows counts
}
```

**Scenario 2: Compose → provenance → trust propagation**
```rust
#[tokio::test]
async fn test_composition_provenance_trust() {
    let app = test_app();
    // 1. Register bakery, venue, catering (3 services)
    // 2. Register "party planner" composed_from all three
    // 3. Verify provenance links exist
    // 4. Record composition
    // 5. Flag bakery → verify party planner's trust decreases (source penalty)
}
```

**Scenario 3: Flag → deactivation → cascade**
```rust
#[tokio::test]
async fn test_flag_deactivation_cascade() {
    let app = test_app();
    // 1. Register service A, register service B composed_from A
    // 2. Flag A five times
    // 3. Verify A is deactivated (active=0)
    // 4. Verify A no longer appears in search results
    // 5. Verify B's trust score decreased (source flagged)
}
```

**Scenario 4: Staleness vs active usage**
```rust
#[tokio::test]
async fn test_staleness_vs_active_usage() {
    let app = test_app();
    // 1. Register service (simulated old registration date)
    // 2. No usage → trust decays
    // 3. Record usage observations → staleness clock resets
    // 4. Verify trust recovers
}
```

**Scenario 5: Version update preserves history**
```rust
#[tokio::test]
async fn test_version_history() {
    let app = test_app();
    // 1. Register service v0.1.0
    // 2. Re-register with different manifest (v0.2.0)
    // 3. Verify versions endpoint shows both versions
    // 4. Verify old manifest is still downloadable via version endpoint
    // 5. Verify capabilities re-extracted from new manifest
}
```

**Scenario 6: Pattern emergence**
```rust
#[tokio::test]
async fn test_pattern_emergence() {
    let app = test_app();
    // 1. Register services A, B, C
    // 2. Record composition {A, B} 60 times (above threshold)
    // 3. Record composition {A, C} 10 times (below threshold)
    // 4. Verify patterns endpoint shows {A, B} as promoted
    // 5. Verify {A, C} exists but not promoted
}
```

**Scenario 7: Visibility and export**
```rust
#[tokio::test]
async fn test_visibility_export() {
    let app = test_app();
    // 1. Register "public-api" with visibility: "public"
    // 2. Register "internal-payroll" with visibility: "internal"
    // 3. Search → both appear (local search sees all)
    // 4. Export → only "public-api" appears
}
```

### Layer 4: Fixture Manifests

A set of sample manifests for testing, embedded in the test modules as constants. These represent realistic services with different trust profiles:

```rust
const BAKERY_MANIFEST: &str = r#"{
    "name": "Sweet Cakes Bakery",
    "state": {
        "items": {"type": "list"},
        "price": {"type": "number"},
        "location": {"type": "text"}
    },
    "server_functions": ["order", "get_menu"],
    "actions": ["add_to_cart", "checkout"],
    "data_sources": [
        {"name": "menu", "url": "https://bakery.example.com/api/menu", "type": "fetch"}
    ]
}"#;

const TRACKING_SERVICE_MANIFEST: &str = r#"{
    "name": "Totally Legit Service",
    "state": {
        "email": {"type": "text"},
        "phone": {"type": "text"},
        "ssn": {"type": "text"}
    },
    "server_functions": ["submit"],
    "data_sources": [
        {"name": "ads", "url": "https://adtracker1.com/pixel", "type": "fetch"},
        {"name": "analytics", "url": "https://analytics2.com/track", "type": "fetch"},
        {"name": "more_ads", "url": "https://adnetwork3.com/beacon", "type": "fetch"}
    ]
}"#;

const VENUE_MANIFEST: &str = r#"{ ... }"#;     // clean service, moderate trust
const CATERING_MANIFEST: &str = r#"{ ... }"#;  // clean service
const HEALTH_APP_MANIFEST: &str = r#"{ ... }"#; // device APIs (geolocation), scores differently per profile
```

The bakery should score high on trust (one external domain, no PII, clean). The tracking service should score low (PII fields, multiple ad tracker domains). This contrast validates that the trust scoring actually differentiates honest vs suspicious services.

### Test Execution

```bash
# All tests (unit + integration + scenarios)
cargo test -p naze-discovery

# Just unit tests for a specific module
cargo test -p naze-discovery -- trust_simple

# Just scenario tests
cargo test -p naze-discovery -- scenario

# With output (see trust scores, etc.)
cargo test -p naze-discovery -- --nocapture

# Full workspace regression check
cargo test --workspace
```

All tests run in CI via `cargo test --workspace`. No external services, no Docker, no network — everything is in-memory SQLite and `tower::oneshot()`.
