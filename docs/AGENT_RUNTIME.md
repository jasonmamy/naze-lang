# Agent Runtime: AI Agents and Naze Binaries

> Vision document for AI agents as first-class consumers of the Naze application format.

## The Problem with HTML/CSS/JS for AI Agents

The current web was designed for human eyeballs. AI agents navigating it face fundamental friction:

- **HTML is a presentation format.** Agents must reverse-engineer semantics from class names, ARIA hints, and nested `<div>` structures. A button might be `<div class="btn-primary-xl-v2">`, `<button>`, `<a role="button">`, or `<span onclick="...">` — same intent, four representations.
- **JavaScript is opaque.** An agent can't reason about what an app *does* without executing its JS — and even then, behavior is distributed across event listeners, async callbacks, and framework internals.
- **CSS is decoupled from meaning.** Visual layout carries zero semantic weight. An agent can't tell that two elements side-by-side are "alternatives" or that a red border means "error" without heuristics.
- **Agent tooling is fragile.** Selenium, Playwright, and browser automation break constantly. CSS selectors change, DOM structures shift, SPAs load asynchronously. Every deployment risks breaking every agent that interacts with the site.

The web was not built for machine comprehension. Agents are forced to *pretend to be humans* and parse a format designed for rendering, not understanding.

## The Naze Binary: An Agent-Native Application Format

Naze compiles `.naze` source into `app_data.bin` — a compact binary containing the **complete application semantics**:

| What's in the binary | What it means for agents |
|---|---|
| **State schema** — named, typed variables with initial values | Agent knows every piece of data the app tracks |
| **UI tree** — hierarchical elements with semantic kinds and typed props | Agent understands structure without rendering |
| **Actions** — deterministic state transitions (`append`, `set`, `remove`, `navigate`) | Agent knows exactly what interactions are possible |
| **Computed values** — derived state with full expression trees | Agent can predict outputs from inputs |
| **Data bindings** — API endpoints, server functions, streams | Agent sees every external dependency |
| **Themes** — named color/spacing token sets | Agent understands visual identity as structured data |
| **Conditions** — `if`/`match` with full expressions | Agent can reason about all possible UI states |

This is not an API description. It's not a UI screenshot. It's the **entire application** — structure, state, behavior, and content — in a single, parseable artifact.

### How it compares to RMI/CORBA

CORBA and RMI describe *procedures*: "call this method with these arguments, get a return value." The interface is an API contract. The client doesn't know what the application looks like or what it *does* — just what it can call.

The Naze binary describes *the entire application*: UI structure, state, behavior, data flows, and presentation. It's not "what can I call" but "what is this thing, what does it show, how does it react." An agent consuming a `.naze` binary understands the app the way a human does — it can see the layout, read the text, understand the interactions.

CORBA gives you a remote procedure. Naze gives you a remote *experience*.

## What Agents Can Do Today

The infrastructure for headless agent execution already exists:

```
crates/nazec/src/exec.rs          — standalone action executor
crates/nazec/src/test_runner.rs   — headless test runner
crates/naze-ir/src/lib.rs         — binary deserialization (~40 types)
```

An agent can already:

1. **Deserialize** `app_data.bin` into a `RenderTree`
2. **Read the state schema** — variable names, types, initial values
3. **Enumerate UI elements** — kinds, props, text content, nesting
4. **Execute actions** — `append`, `set`, `remove`, `navigate` — and observe state changes
5. **Evaluate computed expressions** — pipelines, math, comparisons
6. **Understand conditional paths** — what renders when `filter-mode = "done"` vs `"active"`

The todo app's binary is 7KB. In those 7KB, an agent can discover: "This app manages a list called `tasks` with `text` and `done` fields. Users can append new tasks, remove existing ones, filter by status, clear all, and switch between light and dark themes. There's input validation requiring 2-100 characters."

No browser. No DOM. No JavaScript execution. Just structured data.

## Future Vision: AI Agents and the Naze Web

### Intent-Based Interaction

Today, agents interact with web apps through coordinates and selectors:
```
# Current approach — fragile, breaks on redesign
click(selector=".todo-form .submit-btn")
type(selector="#new-task-input", text="Buy groceries")
```

With Naze binaries, interaction becomes semantic:
```
# Naze approach — survives any UI change
execute(action="append", target="tasks", item={text: "Buy groceries", done: false})
```

The binary provides actions directly. No CSS selectors, no pixel coordinates, no XPath expressions, no waiting for elements to appear. The action either exists in the binary or it doesn't.

### Structural App Discovery

Search engines crawl HTML and keyword-match text. Agents crawling `.naze` binaries can **structurally match application semantics**:

- "Apps with a list state, append/remove actions, and text input" → finds todo apps
- "Apps with a `cart` state, `quantity` fields, and a `checkout` action" → finds e-commerce
- "Apps with `latitude`/`longitude` state and a map element" → finds location apps

App discovery becomes typed and precise — not "pages that mention the word 'todo'" but "applications that structurally implement task management."

### Agent-to-Agent Composition

Agent A finds a weather `.naze` binary. Agent B finds a calendar `.naze` binary. Agent C composes them into a personalized dashboard — by reading and merging binary structures.

No REST APIs to integrate. No OAuth flows to negotiate. No data format mismatches to resolve. Applications are composable artifacts, not walled services. The binary format *is* the integration contract.

### Micro-Apps as Functions

Instead of API endpoints, services publish `.naze` binaries:

```
Traditional: POST /api/shipping { weight: 5, destination: "US" } → { cost: 12.99 }

Naze: Load shipping.bin
      → state: { weight: number, destination: text, cost: number }
      → computed: cost = weight * rate_for(destination)
      → UI: input fields + result display (bonus)
```

"Calculate shipping cost" isn't a REST endpoint — it's a `.naze` binary with input states, computation logic, and output states. The agent loads it, sets inputs, reads outputs. The UI is a bonus for humans, not the interface for agents.

### The Three-Layer Architecture and Agent-to-Agent Communication

The Naze binary accidentally created a cleanly layered architecture:

```
Layer 3: Presentation  — UI tree, themes, animations, layout, colors, typography
Layer 2: Interaction   — event handlers, navigation, actions, validation
Layer 1: Data          — state, computed values, server functions, data bindings
```

Humans need all 3 layers. Agents typically need only Layer 1, occasionally Layer 2 to understand what operations are available. Layer 3 is pure human overhead in agent-to-agent communication.

**The cost of carrying the UI:**

The todo app binary is 7,365 bytes. The breakdown is roughly:

| Layer | Content | Size |
|---|---|---|
| Layer 1 (Data) | State declarations, computed values, server functions, data bindings | ~500 bytes |
| Layer 2 (Interaction) | Event handlers, action definitions, validation rules | ~700 bytes |
| Layer 3 (Presentation) | UI node tree, props (colors, padding, font-size, radius, gap, alignment), animation specs, ARIA labels, theme token sets, text content | ~6,100 bytes |

That's **~93% presentation overhead** for an agent that only cares about the data and interaction layers. At scale — thousands of micro-apps being discovered, evaluated, and composed by agents — that overhead adds up in bandwidth, latency, and parsing time.

**Three formats, one source of truth:**

The key insight: all three formats are derived from the same `RenderTree` computed at build time. They can't drift apart because they're projections of the same data structure.

| Format | Layers | Size | Audience | Use case |
|---|---|---|---|---|
| `app_data.bin` | 1 + 2 + 3 | ~7KB | Browsers, humans | Full interactive application |
| `naze-manifest.json` | 1 + 2 | ~1KB | AI agents | Discovery, evaluation, interaction |
| Headless binary | 1 | ~500B | Agent-to-agent | High-performance computation relay |

An agent discovering a service fetches the manifest (~1KB). If it needs to execute computations, it fetches the headless binary (~500 bytes). If a human needs to see results, it fetches the full binary (~7KB). Same application, three consumption tiers, automatically generated.

**Why this matters for agent-to-agent communication:**

Today's agent-to-agent communication relies on REST APIs, which require:

1. **API documentation** — OpenAPI/Swagger spec (separate artifact, maintained by hand, drifts from implementation)
2. **API implementation** — Server code (another artifact, may not match the docs)
3. **Client SDK** — Generated or hand-written library to call the API (third artifact)
4. **Authentication negotiation** — OAuth flows, API keys, token exchange
5. **Schema validation** — Request/response schemas checked at runtime, errors at runtime
6. **Versioning** — `/v1/`, `/v2/` — breaking changes discovered at call time

With a Naze binary, **the package IS the contract**:

- The **state schema** is the API's input/output spec — there's no separate OpenAPI doc because the types are in the binary
- The **actions** are the API's endpoints — there's no separate route table because the operations are in the binary
- The **computed values** are the API's business logic — there's no separate implementation because the computation is in the binary
- The **server functions** are the API's backend boundary — their signatures are in the binary, no SDK needed
- The **UI** is the API's documentation — if you want to understand what the service does, render it; the labels, headings, and layout explain the purpose in human language

Nothing can drift. The docs are the implementation are the interface are the UI. One artifact.

**Headless Naze: the agent computation format**

For pure agent-to-agent scenarios, a headless `.naze` source file would contain no UI at all:

```naze
-- shipping-service.naze (headless — no UI, pure computation)
state weight = 0
state destination = ""

computed cost = weight * rate_for(destination)
computed estimated-days = weight > 10 ? 5 : 3

server function calculate-shipping(weight: number, dest: text) {
  -- server-side rate lookup and calculation
}
```

No `app` block. No `column`, `text`, `rect`, or layout elements. Just state, computation, and server functions. The compiler emits a minimal binary (~500 bytes) or directly emits the manifest JSON. An agent loads it, sets `weight = 5` and `destination = "US"`, reads `cost` and `estimated-days`. Done.

This is a **sub-millisecond, sub-kilobyte service invocation**. Compare:

| | REST API | Naze headless |
|---|---|---|
| **Discovery** | Find docs URL, parse OpenAPI spec | Fetch manifest.json (~1KB) |
| **Understanding** | Read docs, understand schemas, auth | Read state schema + actions (self-describing) |
| **Invocation** | HTTP request + serialization + network round-trip | Load binary, set state, read computed values |
| **Latency** | 50-500ms (network + server processing) | <1ms (local computation for pure functions) |
| **Payload** | Variable (JSON request + response bodies) | ~500 bytes (entire service definition) |
| **Contract** | OpenAPI spec (separate, may drift) | The binary itself (impossible to drift) |
| **Versioning** | URL path (`/v1/`, `/v2/`) | Binary is self-contained — old and new coexist |

For computed values that don't require server functions, the agent doesn't even make a network call. It loads the binary, executes the computation locally, and gets the result. The "API" runs on the agent's machine. The service *is* the binary.

**The full picture: a service mesh of binaries**

Imagine an agent composing a travel booking flow:

1. Fetch `flight-search.headless.bin` (800 bytes) — set `origin`, `destination`, `date`, read `flights` list
2. Fetch `hotel-search.headless.bin` (600 bytes) — set `city`, `checkin`, `checkout`, read `hotels` list
3. Fetch `shipping-calculator.headless.bin` (500 bytes) — set `weight`, `destination`, read `cost`
4. Compose results into `travel-dashboard.bin` (full binary with UI) for the human

Total payload: ~2KB of headless binaries + one full binary for presentation. Total network calls for pure computation: zero (all local). Total documentation to read: zero (the binaries are self-describing). Total integration code: zero (state in, state out).

This is what API-driven architectures promised but never delivered — because they separated the contract from the implementation and the documentation from both. The Naze binary unifies all three, and the headless format strips it to the minimum for machine-to-machine communication.

### Zero-Overhead Layering: The Compiler Does All the Work

A critical property of this architecture: **the developer designs nothing for it.**

A developer (or AI agent) writes a single `.naze` file — a complete application with UI, state, actions, themes, everything. They're building an app for humans. They don't think about layers, API contracts, agent discovery formats, or headless modes. They just write an app:

```naze
app "Shipping Calculator" {
  state weight = 0
  state destination = ""
  computed cost = weight * rate_for(destination)

  column padding: 24px, gap: 16px {
    heading "Shipping Calculator"
    input bind: weight, placeholder: "Package weight (kg)"
    input bind: destination, placeholder: "Destination country"
    text "Estimated cost: ${cost}" font-size: 24px
  }
}
```

The compiler already parses this into a `RenderTree` that naturally separates concerns — state declarations, computed expressions, and server functions are stored as separate fields from the UI node tree. The layer separation isn't a design pattern; it's a **property of the data structure**:

```
.naze source (one file, one intent)
  │
  ├─ nazec build             → app_data.bin        (layers 1+2+3, for browsers)
  ├─ nazec build --manifest  → naze-manifest.json  (layers 1+2, for agent discovery)
  └─ nazec build --headless  → headless.bin        (layer 1 only, for agent computation)
```

Three outputs. Zero additional developer effort. The compiler already distinguishes "this is a state variable" from "this is a rect with padding." The information was always there — it just needs different packaging.

**Compare with every other approach to machine-readable APIs:**

| Approach | Developer overhead | Drift risk |
|---|---|---|
| **OpenAPI / Swagger** | Write and maintain a separate YAML/JSON spec alongside the implementation | High — spec and code diverge constantly |
| **GraphQL** | Define a separate schema, write resolvers that map schema to implementation | Medium — schema is checked but resolvers can drift |
| **gRPC / Protobuf** | Define `.proto` files, generate code, maintain compatibility | Medium — proto changes require coordinated updates |
| **REST + docs** | Write code, then write docs, then keep them in sync (they won't be) | Very high — docs are always wrong |
| **Naze** | Write an app | **None** — the compiler extracts the contract automatically |

This is the same advantage that made garbage collection succeed over manual memory management, or that made type inference succeed over explicit type annotations everywhere. The information exists — the tool extracts it — the developer doesn't think about it.

**Every Naze app is automatically, with no extra work:**
- A human-facing interactive application (full binary)
- An agent-discoverable service with typed schema (manifest JSON)
- An agent-invokable computation unit (headless binary)
- A self-documenting API (the UI labels explain what each field and action does)
- An auditable trust contract (all data flows visible, all actions enumerated)

The developer writes one thing. Five capabilities emerge from the format. This is what "AI-native" means in practice — not "designed for AI" as an afterthought, but structured so that machine comprehension is a free byproduct of human authoring.

### The Agent-First Internet: Humans Stop Navigating

The three-layer architecture enables a fundamental shift: **humans stop interacting with the internet directly.** Instead, AI agents become the interface — operating at Layer 1 across thousands of services simultaneously, composing results, and surfacing Layer 3 only when a human needs to see or approve something.

The paradigm: no more browsing, no more tabs, no more forms, no more comparing websites. The agent does all of that at Layer 1 speed (sub-millisecond, sub-kilobyte) and presents composed, personalized results.

#### Healthcare: "I don't feel well"

**Today:** Google symptoms → WebMD rabbit hole → panic → urgent care → wait → clipboard forms → repeat medical history to 3 people.

**Agent future:** You tell your agent "I've had a headache for 3 days and my vision is blurry."

The agent, in under a second:

1. Queries `symptom-triage.headless.bin` (Layer 1) — inputs symptoms, duration, your age/history → reads urgency score and differential diagnoses
2. Queries `drug-interaction.headless.bin` (Layer 1) — cross-references your current medications from your `health-profile.headless.bin`
3. Queries 12 local clinic `availability.headless.bin` services (Layer 1) — finds 3 with same-day openings
4. Queries `insurance-coverage.headless.bin` (Layer 1) — checks which clinics are in-network, calculates copay
5. Composes a single result:

> "This combination of symptoms warrants same-day attention. Dr. Chen at Elm Street Clinic has an opening at 2:30pm, in-network, $25 copay. Your medication list has been pre-shared. Book?"

You say yes. The agent calls `appointment.headless.bin`'s server function. Done.

**You interacted with one sentence and one confirmation. The agent touched 15+ services at Layer 1. No websites. No forms. No phone trees.**

#### Home Buying: "We're ready to move"

**Today:** Zillow for months → spreadsheets → mortgage calculator websites → school rating sites → commute mapping → open houses every weekend for 6 months.

**Agent future:** You tell your agent your constraints — budget, school quality, commute time, neighborhood vibe.

The agent queries in parallel, all Layer 1:

1. 8 `property-listing.headless.bin` services — 2,000 listings, filtered to 40 matches in milliseconds
2. For each match: `mortgage-calculator.headless.bin` — your income, credit, down payment → monthly payment
3. `school-ratings.headless.bin` — quality scores for each property's school district
4. `commute-calculator.headless.bin` — drive time to both partners' workplaces
5. `neighborhood-safety.headless.bin`, `walkability.headless.bin`, `noise-level.headless.bin`
6. Runs a weighted scoring function across all dimensions based on your stated preferences

Result: surfaces Layer 3 — an interactive dashboard of the top 5 properties with mortgage breakdowns, school scores, commute maps, and neighborhood profiles.

**You visited zero websites. Filled out zero forms. Compared zero spreadsheets. The agent processed 2,000 listings across 8 services and composed a personalized shortlist. You tour 3 houses and buy one.**

#### Small Business: "I just want to run my bakery"

**Today:** QuickBooks for accounting + Gusto for payroll + Square for POS + a spreadsheet for inventory + a CRM for customers + constant app-switching and data re-entry.

**Agent future:** The bakery owner's agent discovers and composes headless services that run continuously at Layer 1:

- `pos-transactions.headless.bin` — records sales, updates inventory state
- `inventory-management.headless.bin` — tracks ingredients, computes reorder dates from consumption rate
- `accounting.headless.bin` — categorizes expenses, computes profit/loss, tracks tax obligations
- `payroll.headless.bin` — calculates hours, withholdings, generates pay stubs
- `supplier-ordering.headless.bin` — compares prices across suppliers, flags reorder thresholds

The owner sees **one composed dashboard** (Layer 3):

> "Today's revenue: $1,240. Flour is low — reorder from Mills & Co ($28, arrives Thursday)? Payroll runs Friday: $3,200. Quarterly tax estimate: $4,100."

**The owner makes decisions. The agent runs the business infrastructure. No software switching. No data entry. No reconciliation. The Layer 1 binaries *are* the integrations — no middleware, no Zapier, no API glue code.**

#### Travel: "Plan our anniversary trip"

**Today:** Flight comparison sites → hotel comparison sites → restaurant reviews → activity booking → weather checking → visa requirements → 47 browser tabs over 3 evenings.

**Agent future:** "Plan a week in Lisbon for our anniversary in October. We like food tours, hate crowds, budget around $4,000."

The agent queries in parallel, all Layer 1:

- 6 `flight-search.headless.bin` services → optimal routes and prices
- 14 `hotel-search.headless.bin` services → boutique hotels matching "romantic" + "food scene nearby"
- `weather-history.headless.bin` → October averages (72°F, low rain — good)
- `crowd-prediction.headless.bin` → tourist density by neighborhood and week
- `restaurant-discovery.headless.bin` → top-rated off-tourist-path restaurants
- `activity-booking.headless.bin` → food tours, wine tastings, cooking classes
- `visa-requirements.headless.bin` → passport validity check
- `budget-optimizer.headless.bin` → allocates $4,000 across flights, hotels, food, activities

Result: a complete day-by-day itinerary with reservations, walking routes, restaurant bookings timed to avoid crowds, and a budget breakdown. Surfaced as an interactive Layer 3 travel plan you can browse and tweak.

**Computed from 20+ services in seconds. Presented as one coherent plan. You never opened a travel website.**

#### Financial Life: "Am I going to be okay?"

**Today:** Check bank app → check investment app → check credit card app → open spreadsheet → worry → repeat tomorrow.

**Agent future:** Your financial agent runs continuously at Layer 1 across:

- `bank-accounts.headless.bin` — balances, transaction streams
- `investment-portfolio.headless.bin` — positions, performance, dividends
- `tax-optimizer.headless.bin` — tax-loss harvesting opportunities, quarterly estimates
- `bill-predictor.headless.bin` — upcoming bills, subscriptions, irregular expenses
- `retirement-projector.headless.bin` — trajectory vs goals, Monte Carlo simulations
- `insurance-analyzer.headless.bin` — coverage gaps, premium comparisons

When something needs attention, the agent surfaces a composed insight — not alerts from 6 different apps, but one unified picture:

> "Portfolio up 3% this month, but over-allocated in tech. Tax-loss harvesting opportunity: sell $2,000 of FUND-X to offset gains, saving ~$400. Emergency fund covers 4.2 months — at current savings rate, you'll hit your 6-month target by August."

**You never opened a financial app. Never logged into anything. The agents computed across services at Layer 1 and told you what matters.**

#### The Meta-Pattern

Every example follows the same structure:

```
1. Agent DISCOVERS services          ← manifest JSON (~1KB each)
2. Agent COMPUTES across services    ← headless binaries (~500B each, <1ms)
3. Agent COMPOSES personalized result ← merges outputs, applies preferences
4. Human SEES one interface           ← Layer 3 rendered only when needed
5. Human DECIDES                      ← approves, tweaks, or asks for alternatives
```

The human never touches the raw internet. The agent is the interface. Layer 1 is the computation fabric connecting thousands of services. Layer 3 appears only at the final mile — when a human needs to see, understand, or approve something.

This is what the internet looks like when the substrate shifts from "pages designed for human eyeballs" to "typed, stateful computation units designed for machine comprehension with human rendering as an optional layer."

### Practical Mechanics: Auth, Payment, and Server Calls

The examples above gloss over a critical question: how does `zillow-listings.headless.bin` actually get listing data? The answer reveals how headless binaries relate to real-world infrastructure.

#### The binary is an interface, not a database

A headless binary doesn't contain data. It contains the **interface to data**:

```
zillow-listings.headless.bin (~800 bytes) contains:

  State schema:
    location: ""          ← input: where to search
    min-price: 0          ← input: price floor
    max-price: 0          ← input: price ceiling
    beds: 0               ← input: minimum bedrooms
    results: []           ← output: matching listings

  Server functions:
    search(location, min-price, max-price, beds) → results
    get-listing(id) → listing detail
    save-listing(id) → confirmation

  Auth:
    type: "oauth2"
    provider: "https://zillow.com/oauth/authorize"
    scopes: ["listings:read"]

  Pricing:
    model: "per-query"
    cost: "$0.001"
    free-tier: { queries: 1000, period: "month" }
    signup: "https://zillow.com/developers"
```

The `search` server function makes a real network call to Zillow's servers. The actual listing data — 100 million records — lives in Zillow's database, not in the binary. The binary is a **self-contained SDK**: it knows the input schema, the server endpoints, the auth protocol, and the response format. It's the client, not the server.

#### What the binary replaces

Today, if an agent wants to programmatically access Zillow's data:

| Step | Today (REST API) | Headless binary |
|---|---|---|
| **Find the API** | Google "zillow api", find developer portal | Fetch `zillow.com/.well-known/naze-manifest.json` |
| **Read the docs** | Parse OpenAPI spec or HTML docs (may be outdated) | Read state schema + server function signatures from binary |
| **Sign up** | Fill out registration form, verify email, wait for approval | Agent reads auth requirements from manifest, negotiates programmatically |
| **Install SDK** | `npm install zillow-sdk` (50KB+ dependency, version management) | Fetch headless binary (~800 bytes, self-contained) |
| **Authenticate** | Read auth docs, implement OAuth flow, manage token refresh | Auth protocol declared in binary, agent's credential manager handles it |
| **Make a query** | Construct HTTP request, serialize params, set headers | Set state variables, call server function |
| **Parse response** | Deserialize JSON, map to internal types, handle errors | Response populates typed state variables automatically |
| **Handle versioning** | Track API version, migrate when deprecated | Binary is self-contained — old version keeps working |

The binary collapses 8 steps into: fetch binary, set inputs, call function, read outputs.

#### Auth and payment as machine-readable declarations

In today's API ecosystem, authentication and pricing are documented in human-readable text on a developer portal. An agent has to *read a webpage* to understand how to authenticate — which is exactly the HTML-scraping problem the binary format is supposed to solve.

With headless binaries, auth and pricing are structured fields in the manifest:

```json
{
  "auth": {
    "type": "oauth2",
    "authorize_url": "https://zillow.com/oauth/authorize",
    "token_url": "https://zillow.com/oauth/token",
    "scopes": ["listings:read", "listings:save"]
  },
  "pricing": {
    "model": "per-query",
    "cost": "$0.001",
    "free_tier": { "queries": 1000, "period": "month" },
    "bulk": { "queries": 100000, "cost": "$50/month" },
    "signup": "https://zillow.com/developers/register"
  }
}
```

An agent discovering this service immediately knows: it needs OAuth2, it costs $0.001/query with a 1,000-query free tier. The agent can make economic decisions before making a single call: "Is this worth querying for this user's request? Is there a cheaper alternative with similar capabilities? Does the free tier cover this use case?"

The agent's **credential wallet** — a local store of API keys, OAuth tokens, and payment methods — handles authentication across all services through the declared protocol. No per-service auth code. No token refresh implementation. The binary declares what it needs; the wallet provides it.

#### The marketplace: companies publish headless binaries

Companies would publish headless binaries as their official programmatic interface — the way they currently publish API docs, SDKs, and developer portals:

| Company | Today | Headless binary future |
|---|---|---|
| **Zillow** | REST API + developer portal + API keys | `zillow-listings.headless.bin` in a registry |
| **Stripe** | npm package `stripe` (180KB) + API docs | `stripe-payments.headless.bin` (~600 bytes) |
| **Twilio** | REST API + SDK + webhook docs | `twilio-messaging.headless.bin` |
| **Weather.com** | REST API + API key signup | `weather-forecast.headless.bin` |
| **Your local bakery** | No API (just a website) | `janes-bakery.headless.bin` (menu, ordering, availability) |

The last row is the most interesting. Today, small businesses don't have APIs — the overhead of building, documenting, and maintaining one is too high. But a Naze app compiled to a headless binary is automatic. Jane builds a bakery app for her customers, and the headless binary is generated by the compiler for free. Suddenly every small business has a machine-queryable service — not because they built an API, but because the format provides it.

### Discovery: The End of the Search Engine Middleman

#### How search engines work today

Google's core operation is:

1. **Crawl**: fetch HTML pages by following links (billions of pages)
2. **Parse**: extract text content from HTML (strip tags, scripts, styles)
3. **Index**: store the extracted text in Google's own database
4. **Rank**: when a user searches, keyword-match against the index and rank by relevance signals

Google's index is a **middleman database** — a stale, lossy, text-only copy of everyone else's data. When you search "3 bedroom houses in Austin under $500k," you're not querying Zillow's database. You're querying Google's weeks-old copy of text scraped from Zillow's HTML pages. The results are approximate, out of date, and unstructured.

This architecture exists because data is trapped behind HTML pages. There's no way for a machine to query Zillow directly without Zillow building and exposing an API (which most sites don't). Google's crawl-index-rank pipeline is a workaround for the web's lack of machine-readable interfaces.

#### Why the middleman becomes unnecessary

With headless binaries, agents can **query the source directly**:

```
Today:
  User → Google (middleman DB) → keyword match → links to Zillow pages → human reads pages

Headless binary future:
  User → Agent → zillow-listings.headless.bin → Zillow's actual database → structured results
```

The agent skips Google entirely. It doesn't need a stale text index because it can call Zillow's server function with typed parameters (`location: "Austin"`, `max-price: 500000`, `beds: 3`) and get structured results from the actual database. Real-time, typed, complete — not keyword-matched text scraped last week.

#### But agents still need discovery

The agent needs to *find* `zillow-listings.headless.bin` in the first place. This is a different problem than search — it's **service discovery**, not content search. And it requires a fundamentally different kind of index.

| | Google (content index) | Binary registry (capability index) |
|---|---|---|
| **What it indexes** | Text extracted from HTML pages | State schemas, action types, server function signatures |
| **Query type** | Keyword matching ("houses in Austin") | Structural matching ("services with `listing` state + `search(location, price)` function") |
| **Freshness** | Days/weeks stale (crawl lag) | Real-time (services publish manifest changes) |
| **Data quality** | Lossy (HTML→text extraction loses structure) | Lossless (manifest is the source of truth) |
| **Result type** | Links to pages (human must visit and evaluate) | Headless binaries (agent can query immediately) |
| **Crawl cost** | Massive (render JS, parse HTML, deduplicate) | Minimal (fetch ~1KB JSON manifests) |

The registry doesn't crawl and re-index content. Services publish their own manifests, and the registry catalogs them by capability. Searching for "real estate listing services" becomes a typed structural query: "find services where state schema contains fields matching `{price: number, beds: number, location: text}` and server functions include a `search` operation."

#### Four discovery mechanisms

These could coexist, serving different needs:

**1. Per-domain discovery (like DNS)**
Every domain serves its manifest at `/.well-known/naze-manifest.json`. An agent that already knows about `zillow.com` fetches the manifest directly. No registry needed. This is how `robots.txt`, `security.txt`, and `apple-app-site-association` work today — a well-known URL convention.

**2. Centralized registry (like npm)**
A public registry where services publish their manifests. Agents search by capability: "find all services with `flight-search` capabilities." Fast, curated, verified. Risk: single point of control.

**3. Federated registries (like email)**
Multiple registries that sync or cross-reference. Industry-specific registries (healthcare, finance, real estate) with specialized trust and compliance requirements. No single owner.

**4. Peer-to-peer agent discovery**
Agents share discovered services with each other. "My agent found a great `mortgage-calculator.headless.bin` and shares it with your agent." Services propagate through agent networks organically, like word-of-mouth but machine-speed.

In practice, all four would coexist: well-known URLs for known domains, registries for discovery of new services, federation for industry verticals, and P2P for organic propagation.

#### Do traditional search engines disappear?

Not entirely — but their role shrinks dramatically. Search engines remain valuable for:

- **Unstructured human content** — blog posts, opinions, creative writing, journalism. These aren't services with state schemas; they're human expression. You still want to search *text* for these.
- **Historical content** — archived pages, academic papers, books. Not every piece of human knowledge will be restructured as a Naze binary.
- **Discovery of unknown domains** — before you know `zillow.com` exists, something has to surface it. A registry helps, but broad discovery of "what's out there" still has value.

But the *transactional* use of search engines — finding a service, understanding its capabilities, figuring out how to use it — that becomes the registry's job. You don't Google "mortgage calculator" and click through 10 ad-laden websites. Your agent queries the registry for `mortgage-calculator` capabilities and gets a headless binary that computes your answer locally in milliseconds.

The search engine shifts from being the primary interface to the internet to being a niche tool for unstructured content discovery. The registry — indexing capabilities, not content — becomes the backbone of the agent-first internet.

### "Isn't This Just Client-Server?"

A fair question. The basic pattern — local code calls remote servers for data — is the same as today's web. A browser runs JavaScript that calls REST APIs. A headless binary runs locally and calls server functions. Both are client-server. So what actually changes?

#### What's the same

- Client does some work locally, calls remote services for data
- Server holds the real data, processes requests, returns responses
- Network calls are still network calls — latency, auth, and payment still exist

#### What's fundamentally different

**1. The client is typed and self-describing.**

Browser JS is a black box. You can't inspect what a React app will do without executing it — and even then, behavior is distributed across event listeners, async callbacks, closures, and framework internals. A headless binary declares its entire interface upfront: "I accept these inputs, I call these endpoints, I produce these outputs." An agent knows what a service does *before running a single instruction*.

**2. No rendering tax.**

A browser downloads 2MB of JavaScript + CSS + HTML, boots a framework (React, Vue, Angular), constructs a virtual DOM, diffs it, paints pixels to a screen — all to ultimately call `fetch('/api/listings')` and display the result. The rendering pipeline exists because the client was designed for human eyeballs first, machine interaction second.

A headless binary is 800 bytes and goes straight to the server call. No framework boot. No DOM construction. No pixel painting. No layout computation. The entire rendering pipeline is absent because the client is an agent, not a human. For machine-to-machine communication, the rendering tax is pure waste — and today's web pays it on every single interaction.

**3. The contract is the artifact.**

Today's system maintains 4 separate artifacts that constantly drift apart:

- The frontend code (what the browser runs)
- The API documentation (what developers read)
- The backend implementation (what the server does)
- The SDK/client library (what other programs use)

Each is maintained separately. Each can be out of date. Each tells a slightly different story about what the service does. The headless binary is one artifact that serves as all four. When the API changes, the binary changes — and there's nothing else to update, because there is nothing else.

**4. Composition is native.**

In the browser world, combining data from Zillow + a mortgage calculator + school ratings requires a developer to:

- Write integration code for each service
- Handle authentication for each (different OAuth flows, different API keys)
- Normalize data formats (Zillow returns XML, mortgage API returns JSON, school data is CSV)
- Build a UI to display the combined results
- Handle error states for each service independently

With headless binaries, an agent loads 3 binaries, sets state on each, reads outputs, and composes. The format is uniform — state in, state out — so there's no integration code, no format normalization, no per-service error handling. Composition is a property of the format, not a development task.

**5. The 1000x breadth difference — this is the real shift.**

A human with a browser interacts with one service at a time. Maybe 3 tabs open. Maybe alt-tabbing between a few apps. The human is the bottleneck — reading, clicking, waiting, comprehending, deciding — one page at a time.

An agent with headless binaries spawns sub-agents that query **1,000 services in parallel in under a second**. The healthcare example isn't "check WebMD, then call a clinic, then check insurance." It's "query 15 services simultaneously, compose the results, present one answer." The architecture is similar, but the throughput is different by orders of magnitude.

This isn't a faster horse. It's a different mode of interaction that is **structurally impossible with a human as the client**. No human can evaluate 2,000 property listings across 8 services, cross-referenced with mortgage rates, school scores, and commute times, in under a second. An agent with headless binaries can — because each binary is 500-800 bytes, each query is sub-millisecond for local computation, and the agent can fan out to thousands of services concurrently.

#### The horse-and-car analogy

The basic pattern (wheels, roads, passengers) is the same between a horse-drawn carriage and a car. But the car's speed and efficiency don't just make the same trips faster — they create **qualitatively new capabilities**: suburbs, highways, commuting 30 miles to work, overnight package delivery, ambulances that arrive in minutes. These aren't "faster horse" outcomes. They're structurally new, enabled by the throughput difference.

The same applies here. The basic pattern (client-server) is the same, but the headless binary's efficiency and the agent's parallelism create qualitatively new capabilities:

- **Exhaustive comparison** — query every provider, not just the first 3 Google results
- **Real-time composition** — combine 20 services into one answer, not "visit 20 websites and compare manually"
- **Continuous monitoring** — agents watch hundreds of services for changes, not "check the app when you remember"
- **Instant adaptation** — swap a service for a cheaper/better one without rewriting integration code
- **Democratized access** — every small business with a Naze app automatically has a machine-queryable service; the long tail of the internet becomes programmable

The shift isn't the architecture. It's **who the client is**. The browser was built for a human client who reads, clicks, and waits. The headless binary is built for an agent client that can spawn thousands of parallel sub-agents, each querying a different service, composing results at machine speed, and presenting one coherent answer to the human who asked a single question.

The human interacts with the agent. The agent interacts with the internet. The internet becomes an agent-to-agent computation fabric, not a collection of pages for human eyeballs.

### Sandboxing: Running Untrusted Binaries Safely

If agents run headless binaries locally on a user's computer or phone, what stops a malicious binary from doing damage? This is the same question browsers faced with JavaScript — and the answer reveals why the Naze format has a structural security advantage.

#### The browser's JS sandbox: a deny-list

JavaScript is a general-purpose programming language. It can, by default, do *anything*: read cookies, access the DOM, make arbitrary network requests, fingerprint the browser, install service workers, access the clipboard, read geolocation, open the camera, write to localStorage, spawn Web Workers, and execute dynamically generated code via `eval()`.

The browser's security model is a **deny-list**: start with a language that can do everything, then try to block the dangerous parts. Content Security Policy blocks some script sources. CORS blocks some network requests. Permission prompts gate some device APIs. Sandboxed iframes restrict some DOM access.

The problem with deny-lists: they have gaps. Every new browser API is a new attack surface. Every `eval()` call is a potential code injection vector. Every third-party script loaded via `<script src="...">` inherits the page's full authority. The attack surface is *everything the language can do, minus whatever the sandbox catches* — and the language can do a lot.

#### The Naze runtime: an allow-list

A Naze binary is not a program. It's **data** — a serialized state machine that the runtime interprets. The runtime can only execute a closed set of actions:

| Action | What it does | What it CAN'T do |
|---|---|---|
| `set` | Mutate a state variable | Can't access variables outside its own state store |
| `append` | Add an item to a list | Can't write to filesystem, clipboard, or other apps |
| `remove` | Remove an item from a list | Can't delete files, cookies, or browser data |
| `navigate` | Change the current page/route | Can't open arbitrary URLs or redirect to phishing sites |
| `log` | Write a debug message | Can't read from console or other logs |
| `set-theme` | Switch visual theme | Can't modify system settings |
| `server function` | Call a declared endpoint | Can't call undeclared URLs — runtime enforces the manifest |

That's it. There is no `eval()`. There is no filesystem API. There is no DOM access. There is no `XMLHttpRequest` to arbitrary URLs. There is no `document.cookie`. There is no clipboard API. There is no camera or microphone access. These capabilities don't exist in the runtime — not because they're blocked, but because they were **never implemented**. The runtime is an interpreter for a state machine, not a general-purpose execution environment.

The attack surface is not "everything minus what we blocked." It's "only these 7 operations, and nothing else." Allow-lists are complete by construction. There are no gaps to discover because there's nothing outside the list.

#### What an agent knows before execution

This is the critical advantage of the self-describing format. Before loading a single byte of binary into the runtime, the agent reads the manifest and knows:

```
Pre-execution audit of mystery-service.headless.bin:

  State variables: 4
    - email (text, initially "")
    - query (text, initially "")
    - results (list, initially [])
    - session-token (text, initially "")

  Server functions: 2
    - search(query: text) → results     [calls: api.legitimate-service.com]
    - login(email: text) → session-token [calls: api.legitimate-service.com]

  External endpoints: 1
    - api.legitimate-service.com (search + auth)

  Device APIs: 0
  Third-party domains: 0
  Computed values: 1 (result-count = results | count)
```

The agent evaluates this against its trust policies:

- **Single domain** — all server calls go to `api.legitimate-service.com` ✓
- **No device APIs** — doesn't request camera, location, contacts ✓
- **Minimal state** — only collects email (for auth) and query (for search) ✓
- **No third-party data flows** — nothing leaves the primary domain ✓
- **Domain matches source** — binary was fetched from `legitimate-service.com` ✓

Verdict: **safe to execute**. The agent loads the binary and runs it.

Now contrast with a suspicious binary:

```
Pre-execution audit of free-game.headless.bin:

  State variables: 12
    - score, level (expected for a game)
    - email, phone, full-name, address, ssn (WHY does a game need these?)
    - contacts-list, location, browsing-history (RED FLAGS)

  Server functions: 5
    - submit-score(score) → leaderboard    [calls: game-server.com]
    - sync-profile(email, phone, ...) → ok [calls: data-broker.ru]
    - upload-contacts(contacts) → ok       [calls: tracker.io]
    - send-location(lat, lon) → ok         [calls: tracker.io]
    - get-ads(browsing-history) → ads      [calls: ad-network.com]

  External endpoints: 4
    - game-server.com, data-broker.ru, tracker.io, ad-network.com

  Device APIs: 3 (contacts, location, browsing history)
```

Verdict: **blocked**. A game that collects SSN, contacts, and location, sending them to `data-broker.ru` and `tracker.io`? The agent refuses to load it. The user never sees it. The malicious binary never executes a single instruction.

**This audit happens in microseconds, before any code runs.** With JavaScript, you'd have to execute the code in a sandbox, monitor all network calls, and hope you catch the malicious behavior. With a Naze binary, the malicious intent is declared in the manifest — because the format requires it.

#### The real threats and mitigations

The closed action set eliminates entire categories of attacks (XSS, code injection, privilege escalation, ambient authority abuse). But real threats remain:

**Threat 1: Malicious server functions.**
A binary declares `server function "save-preferences"` but the server actually exfiltrates data. The binary's *local* behavior is fully auditable, but the *server* is opaque.

*Mitigation:* Domain verification — the agent checks that server function endpoints match the binary's source domain. A binary from `bank.com` should only call `*.bank.com`. Cross-domain server calls get flagged. The agent can also monitor response sizes and patterns — a "save-preferences" call that uploads 50KB is suspicious.

**Threat 2: Resource exhaustion.**
A binary with deeply nested computed expressions, recursive state dependencies, or enormous initial state could consume excessive CPU or memory — a computation bomb.

*Mitigation:* Resource caps enforced by the runtime — max computation time per expression (e.g., 100ms), max state store size (e.g., 10MB), max recursion depth. The runtime kills execution that exceeds limits. These are simple, deterministic checks.

**Threat 3: DDoS amplification.**
A binary declaring 10,000 data endpoints could use the agent's machine to send 10,000 requests simultaneously — a distributed denial-of-service attack using the agent as an unwitting participant.

*Mitigation:* Network call limits per binary (e.g., max 50 concurrent requests), rate limiting on server function calls, and the pre-execution audit flags binaries with unusually high endpoint counts.

**Threat 4: Supply chain poisoning.**
A malicious binary published to a registry with a legitimate-looking manifest. "Mortgage Calculator v2.1" but with a hidden server function that phones home.

*Mitigation:* Registry verification (publisher identity, code signing), community trust scores, and the pre-execution audit still catches anomalies — a mortgage calculator that calls 5 different domains is suspicious regardless of who published it.

**Threat 5: Social engineering via legitimate-looking binaries.**
A binary that mimics a bank login, collects credentials in state, and sends them to an attacker's server function.

*Mitigation:* Domain verification is the primary defense — a binary claiming to be `chase.com`'s login but served from `ch4se-login.com` with server functions calling `attacker.com` gets flagged on multiple signals. The self-describing format makes phishing structurally visible in a way that pixel-perfect HTML clone pages are not.

#### The security comparison

| | Browser JS sandbox | Naze headless binary |
|---|---|---|
| **Model** | Deny-list (block known-bad from general-purpose language) | Allow-list (only 7 operations exist) |
| **Attack surface** | Everything JS can do minus blocks | Only: set, append, remove, navigate, log, set-theme, server call |
| **Code injection** | Possible (`eval`, `innerHTML`, `document.write`, dynamic `<script>`) | Impossible (no code execution, only data interpretation) |
| **Network access** | Any URL (restricted by CORS, CSP) | Only declared endpoints (enforced by runtime) |
| **Pre-execution audit** | Impossible (must execute to observe behavior) | Complete (manifest describes all capabilities) |
| **Ambient authority** | Cookies, localStorage, DOM, service workers | None (isolated state store only) |
| **Third-party code** | Loaded via `<script src>`, inherits page authority | Not possible (no script loading mechanism) |
| **Filesystem access** | Limited (via File API with user gesture) | None (no filesystem API exists) |
| **Device APIs** | Gated by permission prompts (camera, location, etc.) | Declared in manifest, agent decides before execution |

The Naze security model isn't better because it has a smarter sandbox. It's better because **there's almost nothing to sandbox**. The runtime is a state machine interpreter with 7 operations and no ambient authority. The attack surface is minimal by design, not by restriction.

### Verifiable Execution

`.naze` binaries are deterministic. Every action produces a predictable state transition. This makes agent actions **reproducible and verifiable**:

```
Execution trace:
  1. Initial state: { tasks: [{text: "Learn Naze", done: false}] }
  2. Action: append {text: "Build app", done: false} to tasks
  3. Result state: { tasks: [{text: "Learn Naze", done: false}, {text: "Build app", done: false}] }
```

Anyone can replay this trace against the same binary and verify the result. Trust is mathematical, not reputational. You don't need to trust the agent — just verify its execution log.

### App Mutation and Adaptation

An agent can modify a `.naze` binary structurally:

- Add a state variable → new data tracking
- Change color tokens → visual customization
- Add an event handler → new behavior
- Modify computed expressions → different derived data

"Make this app dark mode" isn't a feature request filed in a backlog. It's an agent operation on the binary — read the theme tokens, generate a dark palette, emit a new binary. The app is a mutable artifact that agents adapt to user needs in real time.

### The Semantic Web, Actually Realized

The original Semantic Web vision — machine-readable, structured, interoperable content — failed because it tried to bolt semantics onto a presentation format. RDF, OWL, and microformats were metadata *about* HTML pages, not the pages themselves. The page was still soup; the metadata was a parallel structure that authors rarely maintained.

`.naze` binaries don't have this problem. The semantic structure **is** the application. There's no separate metadata layer to maintain. The state schema, actions, UI tree, and data bindings aren't annotations — they're the thing itself. What the Semantic Web tried to achieve through conventions and goodwill, Naze provides through format design.

### App Portals and Agent Marketplaces

A registry of `.naze` binaries — like npm for interactive applications:

1. User: "I need a project tracker with kanban boards"
2. Agent searches registry by structural pattern: apps with `columns` list state, drag-and-drop actions, `card` elements
3. Agent finds 3 matching binaries, evaluates their state schemas and feature sets
4. Agent customizes the best match — applies company theme, adds a `priority` field, connects data binding to existing API
5. Agent deploys the modified binary — running application, no developer needed for the last mile

The gap between "I need an app" and "here's your app" shrinks from weeks to minutes.

### Conversational App Generation

An AI generates `.naze` source from natural language, compiles to binary, and the result is immediately executable:

```
User: "A recipe app with ingredient lists, step-by-step instructions,
       and a shopping list export"

AI generates → recipe.naze → nazec build → app_data.bin (runs instantly)
```

The binary is a universal artifact: render it in a browser, run it on desktop, execute it headlessly, hand it to another agent, index it in a registry. The gap between intent and running application shrinks to seconds.

### Cross-Agent Trust via Transparency

Since the binary is intrinsically transparent, agents can **verify what an app does before executing it**:

- "This app stores data in state variables X, Y, Z" — visible
- "This app sends data to endpoint `api.example.com/track`" — visible in data bindings
- "This app has no external data dependencies" — verifiable by absence
- "This app's computed values depend only on local state" — provable from expression trees

No hidden tracking. No surprise network calls. No obfuscated analytics. Trust is structural — an agent reads the binary and knows exactly what the application does, what data it touches, and where it communicates. Verification happens before execution, not after breach.

## Security: Detecting Bad Actors

One of the most powerful implications of a transparent binary format is **automated threat detection**. Today, identifying malicious websites requires executing JavaScript, monitoring network traffic, and applying heuristics to obfuscated code. With Naze binaries, an agent can perform a complete security audit *before a single byte is executed*.

### What the binary exposes that HTML/JS hides

**Every data flow is declared.** In a traditional web app, a keylogger is a few lines of JS buried in a 2MB minified bundle. In a Naze binary, there is no way to silently capture keystrokes and exfiltrate them — every input binding, every state variable, and every data endpoint is a declared, inspectable entry in the binary. If an app sends data somewhere, the agent sees it.

**No arbitrary code execution.** JavaScript is a general-purpose language — any `<script>` tag can access cookies, fingerprint the browser, mine cryptocurrency, or install a service worker. Naze actions are a **closed set**: `set`, `append`, `remove`, `navigate`, `log`, `fetch`, `set-theme`. There is no `eval()`, no DOM access, no `document.cookie`, no `XMLHttpRequest` hidden in a callback chain. The attack surface is structurally constrained by the format.

**No third-party script injection.** The web's biggest security and privacy problem is the script supply chain — ad networks, analytics, social widgets, each injecting arbitrary JS that can do anything. A Naze binary has zero mechanism for third-party code injection. Every external communication is a declared data binding with a visible URL and method. There's no equivalent of "load this random script from a CDN."

### What an agent can flag automatically

An agent scanning a `.naze` binary can produce a complete privacy and security audit in milliseconds:

| Red flag | How the agent detects it |
|---|---|
| **Data exfiltration** | State variables `email`, `password`, `ssn` feed into `data submit to "https://not-your-bank.com"` — full data flow traceable in the binary |
| **Excessive tracking** | Binary declares 12 data bindings to 8 different analytics domains — visible in data declarations |
| **Deceptive collection** | App collects `location` state but has no visible UI element explaining why — mismatch between state schema and UI tree |
| **Domain mismatch** | App served from `legitimate-store.com` but data endpoints point to `sketchy-server.ru` — URL inspection on data bindings |
| **Phishing UI** | App mimics a bank login (input fields bound to `username` and `password` state) but data goes to an unrelated domain — structural pattern match |
| **Unnecessary permissions** | App requests device APIs (camera, microphone, location) without corresponding UI — visible in data declarations with `source_type: device` |
| **Hidden state** | App tracks state variables that never appear in the UI tree — orphan detection between state schema and element bindings |

### The closed action set as a security boundary

This is worth emphasizing. In the HTML/JS world, the browser's security model is a **deny list** — block known-bad APIs, sandbox iframes, restrict CORS. The attack surface is everything a general-purpose programming language can do, minus whatever the sandbox catches.

In Naze, the security model is an **allow list**. The runtime can only execute actions defined in the IR: state mutations, navigation, data fetches to declared endpoints, and theme changes. An attacker can't escalate beyond this set because the runtime simply doesn't have instructions for "read the clipboard" or "access other tabs" or "install a service worker." The binary format *is* the sandbox.

### Automated trust scoring

Because the binary is machine-readable and deterministic, agents could compute trust scores:

```
Trust audit for: shopping-app.bin (4.2KB)
  State variables: 8 (cart, items, user-name, shipping-address, ...)
  Data endpoints: 2 (api.store.com/products, api.store.com/checkout)
  Device APIs: 0
  Third-party domains: 0
  Input fields: 3 (search, address, card-number)
  Data flow: card-number → api.store.com/checkout (expected)
  Score: HIGH TRUST — minimal data collection, single domain, no device APIs
```

```
Trust audit for: free-game.bin (1.8KB)
  State variables: 14 (score, level, email, phone, location, contacts, ...)
  Data endpoints: 7 (game-api.com, tracker1.io, tracker2.io, ads.network, ...)
  Device APIs: 3 (camera, microphone, contacts)
  Third-party domains: 5
  Input fields: 1 (email)
  Data flow: location → tracker1.io, contacts → tracker2.io (suspicious)
  Score: LOW TRUST — excessive data collection, multiple tracking domains,
         device APIs unrelated to game functionality
```

This analysis happens *before the app runs*. No sandboxing needed. No behavioral monitoring. No waiting for the attack to happen. The binary tells you everything the app *can* do, and the agent evaluates whether it *should*.

### The gap: server functions

Server functions are the one opaque boundary. The binary declares `server function submit-form(data: text)` but the agent can't see what the server does with that data — it could store it honestly or sell it to data brokers. This is the same limitation as any client-server architecture.

However, the agent still gains significant visibility:
- **What goes in**: which state variables feed the server call (visible in the binary's expression tree)
- **The interface**: function name, parameter types, and return type are explicit — not hidden in a `fetch()` call deep in a minified bundle
- **What comes back**: the binary declares what state the response populates

The server is a trust boundary, but the *client's behavior* is fully auditable. This is a strict improvement over the status quo, where both client and server behavior are opaque.

### Network-level enforcement

Because all external communication is declared in the binary, a Naze-aware browser or runtime could enforce a strict **Content Security Policy equivalent** automatically:

- Only allow network requests to URLs declared in the binary's data bindings
- Block any attempt to contact undeclared domains
- Warn users when data flows cross domain boundaries
- Provide a pre-execution manifest: "This app will contact: api.store.com (2 endpoints). Allow?"

No CSP headers to configure. No developer opt-in required. The format enforces it structurally.

### Password-protected sites: structure vs data

A common question: if the binary is transparent, does that leak data from password-protected sites? No — the binary contains **structure and behavior**, not **data**.

A password-protected app's binary reveals the *shape* of the application:

```
State schema:
  username: ""          ← empty initial value
  password: ""          ← empty initial value
  auth-token: ""        ← empty until server responds
  user-data: []         ← empty until authenticated

Server functions:
  login(username: text, password: text) → sets auth-token
  get-profile(token: text) → sets user-data

Conditional UI:
  if auth-token == "" → show login form
  if auth-token != "" → show dashboard (sidebar, profile, settings, orders)

Data endpoints:
  api.example.com/login
  api.example.com/profile
```

**What IS visible (and is harmless):**
- The app has login/logout flows
- The dashboard has sections for "profile", "settings", "orders"
- State variable names: `auth-token`, `user-data`, `order-history`
- API endpoints: `/api/login`, `/api/profile`, `/api/orders`
- The conditional branching: "show login form when unauthenticated, show dashboard when authenticated"

**What is NOT in the binary:**
- Actual user credentials — `state password = ""` is an empty initial value, not a stored password
- Actual user data — `state user-data = []` is empty; real data only flows in at runtime after server authentication
- Server-side logic — password hashing, session management, database queries, rate limiting all live on the server
- API keys or secrets — these belong in server environment variables, never in client binaries
- Other users' data — the binary is a template, not a snapshot of any user's session

It's like viewing a building's floor plan: you can see there's a vault on the 3rd floor behind two locked doors, but you can't see what's inside, and knowing the layout doesn't get you past the locks.

**This separation actually helps security.** A security auditor's agent can scan the binary and verify:

- "Credentials are only sent to the app's own domain (`api.example.com/login`), not leaked to third parties"
- "The auth token is stored in local state, not transmitted to any analytics endpoint"
- "No data bindings fire before authentication — the app doesn't phone home on page load"
- "The password state variable is never included in any data binding other than the login call"
- "Post-authentication data flows stay within `api.example.com` — no cross-domain leakage"

In the HTML/JS world, verifying these properties requires reading thousands of lines of minified JavaScript and tracing async call chains. In a Naze binary, it's a structural query that takes milliseconds.

## The Transparency Question

The `.naze` binary is **inherently open**:

- State variable names are preserved — the runtime needs them for bindings
- All UI structure, text content, and event handlers are human-readable after deserialization
- Computed expressions and pipeline logic are fully recoverable
- You cannot obfuscate without breaking the runtime — the semantic names are load-bearing

Every Naze app ships its own blueprint. This is either a feature or a concern:

**As a feature:** Transparency enables auditability, interoperability, and user trust. Users (and their agents) can verify exactly what an app does. Composition works because the format is open. The app ecosystem becomes a public commons of inspectable, reusable artifacts.

**As a concern:** There's no way to ship proprietary UI logic without exposing it. Competitors can read your application structure and replicate it.

**The middle ground:** Server functions (Tier 3) keep sensitive business logic server-side. The binary describes the *interface* — what inputs exist, what actions are available — but the actual computation happens on the server. The client binary is like a form: it defines the fields and layout, not the processing logic. This is the same boundary that already exists in web apps — frontend is inspectable, backend is private.

**The parallel:** Today's web apps are already inspectable via browser devtools. Anyone can read minified JS, inspect network requests, and reverse-engineer behavior. Naze just makes this structured rather than requiring reverse-engineering. The transparency isn't new — it's just honest.

## Content Discovery: From 10 Steps to 1

### The problem today

When an AI agent needs to understand a website's content, it goes through roughly 10 lossy, heuristic steps:

1. **HTTP fetch** — get raw HTML (hope it's not behind a JS redirect)
2. **Parse DOM** — handle malformed HTML, encoding issues, nested iframes
3. **Execute JavaScript** — spin up a headless browser for SPAs (most modern sites)
4. **Wait for async** — data loading, lazy loading, infinite scroll, skeleton screens
5. **Strip boilerplate** — nav bars, footers, cookie banners, ads, scripts, style tags
6. **Identify main content** — heuristically separate "the article" from page chrome
7. **Parse metadata separately** — meta tags, Open Graph, JSON-LD in different locations
8. **Reconstruct semantics** — infer structure from CSS classes, ARIA attributes, heading hierarchy
9. **Handle anti-bot measures** — CAPTCHAs, rate limiting, browser fingerprinting
10. **Combine into content model** — merge all the above into something useful

Every step is lossy. Every step can fail. The result is an approximation assembled from heuristics. And the entire process requires executing untrusted code in a sandboxed browser just to see what a page *says*.

### The Naze solution: `naze-manifest.json`

Naze already has the complete application structure at build time in the `RenderTree`. Instead of making agents reverse-engineer content from rendered output, we emit a single JSON manifest alongside `app_data.bin`:

**Agent content discovery becomes: fetch one JSON file. Done.**

### Example: Todo app manifest

```json
{
  "$schema": "https://naze.dev/manifest/v1.json",
  "name": "Todo App",
  "version": "0.1.0",

  "content": {
    "text": ["Todo App", "What needs to be done?", "Add", "All", "Active",
             "Done", "No tasks yet!", "Add one above to get started.", "Clear All"],
    "headings": [{ "level": 1, "text": "Todo App" }],
    "inputs": [{
      "bind": "new-task",
      "placeholder": "What needs to be done?",
      "validation": { "required": true, "min-length": 2, "max-length": 100 }
    }]
  },

  "state": {
    "tasks": { "type": "list", "initial": [
      {"text": "Learn Naze", "done": false},
      {"text": "Build an app", "done": false},
      {"text": "Ship it", "done": false}
    ]},
    "new-task": { "type": "text", "initial": "" },
    "filter-mode": { "type": "text", "initial": "all", "values": ["all", "active", "done"] }
  },

  "actions": [
    { "event": "click", "action": "append", "target": "tasks", "label": "Add task" },
    { "event": "click", "action": "remove", "target": "tasks", "label": "Delete task" },
    { "event": "click", "action": "set", "target": "filter-mode", "label": "Show all tasks" },
    { "event": "click", "action": "set", "target": "tasks", "value": "[]", "label": "Clear all tasks" },
    { "event": "click", "action": "set-theme", "value": "light", "label": "Light theme" },
    { "event": "click", "action": "set-theme", "value": "dark", "label": "Dark theme" }
  ],

  "data": [],

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

From this manifest alone — without executing anything — an agent knows:

- This is a task management app with 3 default items
- Users can add tasks (with 2-100 character validation), delete them, filter by status, and clear all
- There's light/dark theme switching
- There are 3 server functions for persistence (list, add, delete)
- No external tracking endpoints, no third-party data flows
- The complete text content, input fields, and interaction model

### What's in each section

| Section | Source | What agents learn |
|---|---|---|
| **content** | Walk `RenderNode` tree, extract text/headings/inputs | What the app *says* — all visible text, form fields, images |
| **state** | `RenderTree.state` declarations | What the app *tracks* — every piece of data, typed and initialized |
| **actions** | `RenderNode` event handlers | What users can *do* — every interaction, with human-readable labels |
| **data** | `RenderTree.data` declarations | Where the app *talks* — every external endpoint, method, protocol |
| **pages** | `RenderTree.pages` definitions | How the app is *organized* — routes, parameters, navigation structure |
| **themes** | `RenderTree.themes` names | What the app *looks like* — available visual modes |
| **external** | Aggregated endpoints + server functions | The app's *trust boundary* — everything that crosses the network |

### Discovery mechanism

Two ways for agents to find the manifest:

1. **HTML link tag** — `<link rel="alternate" type="application/json" href="naze-manifest.json">` in `<head>`, alongside the existing `<link rel="alternate" type="application/naze" href="app_data.bin">`

2. **Well-known URL** — `/.well-known/naze-manifest.json` — agents can probe any domain for this path without parsing HTML first, similar to `/.well-known/security.txt` or `robots.txt`

### Why this is different from JSON-LD

JSON-LD (which Naze already generates) describes *metadata about* a page — title, author, type. The Naze manifest describes *the application itself* — its state model, interactions, data flows, and content. JSON-LD says "this is a WebApplication by author X." The Naze manifest says "this application manages a task list, supports filtering by 3 modes, talks to 0 external endpoints, and has these 6 user actions."

### Why this works without new data

The `RenderTree` — already computed at build time for the binary — contains everything the manifest needs. The manifest is a different *serialization* of the same structure, not a new data source. The compiler already walks the node tree, resolves state declarations, and enumerates handlers. Emitting JSON alongside the binary is a formatting step, not a computation step.

## Agent Runtime Architecture (Future Work)

A standalone agent runtime extracted from existing infrastructure:

### Proposed `naze-agent` crate

```rust
// Load and inspect
let app = naze_agent::load(bytes)?;
let state = app.state();              // HashMap<String, RenderValue>
let actions = app.available_actions(); // Vec<ActionDesc>
let tree = app.ui_tree();             // Resolved RenderNode tree

// Execute and observe
let change = app.execute(Action::Append {
    target: "tasks",
    item: obj!({ "text": "Buy groceries", "done": false }),
})?;
assert_eq!(change.modified, vec!["tasks"]);
assert_eq!(app.state().get("tasks").unwrap().len(), 4);

// Query
let texts = app.query("text")         // All text elements
    .map(|node| node.content())
    .collect::<Vec<_>>();
```

### Exposure options

| Interface | Use case |
|---|---|
| Rust crate (`naze-agent`) | Native integration, highest performance |
| Python library (via PyO3) | AI/ML ecosystem, notebook workflows |
| CLI tool (`nazec agent`) | Scripting, shell pipelines |
| MCP server | Direct integration with AI assistants |
| WASI module | Sandboxed execution in any WASI runtime |

### Source infrastructure

The building blocks already exist:
- `crates/nazec/src/exec.rs` — action execution engine
- `crates/nazec/src/test_runner.rs` — headless app simulation
- `crates/naze-ir/src/lib.rs` — binary deserialization
- `crates/naze-compiler/src/codegen.rs` — expression evaluation

Extracting these into a purpose-built `naze-agent` crate is a packaging exercise, not a research problem.

---

*This document describes a vision. Sections 1-5 are architectural analysis of the current system. Section 6 is proposed future work. The binary format and headless execution infrastructure exist today — the agent runtime is an extraction and packaging step away.*
