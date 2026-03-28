# The Naze Browser

A browser where you describe what you want, and the response is a running application.

Not a chatbot. Not a code editor. Not a website. A new kind of software — where AI responses are interactive apps, built on the fly, rendered natively, and fed back into a living ecosystem.

## The Concept

Traditional browsers render documents. The Naze Browser renders intent.

You open it. You type: "build me a meal planner with a shopping list." Seconds later, you're using a meal planner with a shopping list. Not looking at a mockup. Not reading code. Using it — clicking, typing, dragging, interacting with a real application that didn't exist moments ago.

The app is the response.

Say "add a budget tracker" and the app updates. Say "make it work offline" and it does. Each instruction refines the running application. The conversation IS the development process, and the output IS the product.

## The Metaphor

Every concept from the traditional browser has a counterpart:

| Traditional Browser | Naze Browser |
|---|---|
| URL bar | Natural language prompt bar |
| Websites | Generated or discovered apps |
| Bookmarks | Saved apps (full applications, not links) |
| Tabs | Concurrent running apps |
| History | Conversation history (each entry is a version of the app) |
| Search engine | Discovery network (structural capability matching) |
| View Source | View `.naze` source (hidden by default) |
| Downloads | Forked apps (editable copies) |
| Extensions | Published packages on the registry |

The difference: a traditional browser consumes content that humans built. The Naze Browser generates content from intent. The user doesn't navigate to solutions — the solutions are built around them.

## Three Modes of Use

### 1. Generate

The user describes. The agent builds. The app appears.

> "Build me a recipe organizer that syncs across devices."

The agent knows the Naze language natively — the grammar, documentation, and example corpus are embedded in the browser. It generates a `.naze` application, compiles it in-process, and renders it to the screen. The entire cycle — prompt to pixels — takes seconds.

If the first attempt has issues, the agent sees the compiler errors and fixes them automatically. The user never sees an error message. They see a working app or a status indicator while the agent iterates.

Follow-up instructions refine the running app:

> "Add a way to tag recipes by cuisine."

The agent reads the current source, modifies it, recompiles, and the app updates. The conversation is the development loop. The user's words are the only interface.

### 2. Discover

The user queries the Discovery Network for existing apps and services — not by name, but by capability.

> "Find me a service that can convert currencies with live exchange rates."

The browser queries the network using structural matching. Not keyword search — the query describes what the service must *do*, and the network returns services whose typed manifests match. Results are ranked by trust score, not ad spend. A one-person operation with honest code ranks alongside a Fortune 500 company.

The user sees results with trust scores, capability breakdowns, and privacy assessments — all derived from analyzing the code itself, not from reviews or ratings. They select a service, and it renders immediately in the browser. No installation. No download. No sign-up page.

### 3. Compose

This is where generation and discovery converge. The agent doesn't just generate code from scratch or discover a single existing service — it discovers multiple services and composes them into something new.

> "Plan a dinner party for 12 people."

The agent queries the discovery network and finds: a recipe service with serving-size scaling, a grocery delivery service with price comparison, a seating arrangement tool, a playlist generator. It pulls these services as packages, generates the glue code that wires them together, compiles the composite application, and renders it — a single, cohesive dinner party planner assembled from independent services that were never designed to work together.

The composition works because every Naze service exposes a typed manifest — its state schema, server functions, and data bindings are machine-readable. The agent doesn't scrape HTML or guess at APIs. It reads the manifest, understands the interface, and wires the pieces together at compile time.

## In Practice

### The Job Seeker

Maria lost her job. She opens the Naze Browser and types:

> "Build me a resume editor."

Seconds later, she's looking at a resume builder — fields for experience, education, skills, a live preview of the formatted document. She starts filling it in. Halfway through, she types:

> "Can you pre-fill this from my LinkedIn profile?"

The agent discovers a LinkedIn data service on the network, pulls her public profile, and populates the fields. Maria adjusts a few things, tweaks the formatting, and has a polished resume.

Then she types:

> "Now find me senior marketing jobs in Portland and let me apply with this resume."

The agent queries the discovery network for job listing services. It finds three — a general job board, an industry-specific marketing careers site, and a startup job aggregator. It composes them into a single search interface alongside her resume, with filters for location, salary, and role type. Each listing has a "Submit Application" button that sends her resume through the job board's typed API.

Maria didn't install three different apps. She didn't create accounts on three job sites. She didn't manually upload her resume three times. She described what she needed, and the browser assembled it from services that exist independently on the network — a resume editor she generated, job services that businesses published, and glue code the agent wrote to connect them.

She applies to four jobs before lunch. The whole thing took twenty minutes.

### The Bakery Owner

David runs a small bakery. No website. No app. No technical skills. He opens the Naze Browser:

> "I need an online ordering page for my bakery. We sell sourdough loaves, croissants, and custom cakes. Pickup only, Tuesday through Saturday."

The agent generates an ordering app: a menu with his three product categories, a pickup date/time selector that respects his Tuesday-Saturday schedule, a simple cart, and a checkout flow. David looks at it, tries ordering a sourdough loaf, sees it work.

> "Add custom cake options — size, flavor, and a text field for decoration instructions."

The app updates. David tests it again. It works. He publishes it.

Now David's bakery is on the discovery network. His manifest says: sells baked goods, accepts orders, pickup only, located at his address. When Maria's agent — or anyone's agent — searches for "birthday cake, pickup, nearby," David's bakery shows up. Ranked not by how much he spent on advertising, but by his trust score — which is high, because his app is simple: it takes an order and hits one payment API. No trackers, no third-party data sharing.

David paid nothing to be discoverable. He wrote zero code. He has no DevOps team. His bakery competes on the same terms as a chain with a million-dollar tech budget.

### The Birthday Party

Six months later, a parent in David's neighborhood opens the Naze Browser:

> "Plan my daughter's 7th birthday party. Unicorn theme. About 15 kids."

The agent queries the discovery network. It finds: David's bakery (custom cakes, pickup, nearby — high trust score), a party venue with availability checking, an invitation service with RSVP tracking, a party supply store with themed decorations.

It composes all four into a planning dashboard: a cake order form (pre-filtered to custom cakes, unicorn decoration field ready), a venue browser showing Saturday availability, an invitation builder with a guest list and RSVP status, and a supply list with unicorn-themed items and prices.

The parent orders a unicorn cake from David, books a venue, sends invitations, and orders supplies — from one app that didn't exist thirty seconds ago, assembled from four services that were never designed to work together.

David gets a cake order. He doesn't know it came from a composed app. He doesn't need to. His ordering service handled it the same as any other order.

The parent, satisfied, publishes the party planner. Now "birthday party planner" exists on the network as a proven composition template. The next parent who asks gets it instantly — zero tokens spent regenerating what's already been solved.

### The Freelancer

Priya does freelance graphic design. She starts simple:

> "Make me an invoice template with my business name, client fields, line items, and a total."

She gets a clean invoice app. She uses it for a few weeks, sending invoices by screenshot. Then:

> "Add time tracking. I want to log hours per project and auto-fill invoices from tracked time."

The app grows. She uses it daily. A month later:

> "Add expense tracking with categories — software subscriptions, hardware, travel."

Then:

> "Show me a dashboard with monthly revenue, expenses, and profit margin over the last 6 months."

Over three months, Priya has iterated a simple invoice template into a complete freelance business management tool. Every version is in her conversation history — she can rewind to any previous state. The app was never "designed" by anyone. It grew organically from her actual needs.

She publishes it. A freelance photographer discovers it, forks it, and customizes the expense categories for photography (equipment rental, studio fees, print costs). The photographer publishes their version. Now the network has two specialized freelance tools — both descendants of Priya's original prompt, both adapted to different industries, both available for the next freelancer who needs something similar.

### The Student

Alex is studying economics. The textbook explanation of compound interest isn't clicking. They open the Naze Browser:

> "Show me how compound interest works with a calculator I can play with."

An interactive tool appears: sliders for principal amount, interest rate, and time period. A chart shows the growth curve updating in real-time as Alex drags the sliders. Below, a table breaks down year-by-year balances.

Alex plays with it. Moves the rate slider. Watches the curve steepen. Gets it. Then:

> "Now show simple interest next to it so I can compare."

A second curve appears on the same chart, linear against the exponential. The gap between them widens visually as the time slider increases.

> "Add inflation adjustment. I want to see the real return."

A third line appears — compound interest minus inflation. Alex sees that a 5% return with 3% inflation isn't really 5%. The interactive visualization taught them something the textbook couldn't.

Alex didn't install a financial calculator app. They didn't search for a compound interest website and hope it had the right features. They described what they wanted to understand, and the browser built a tool for understanding it.

## Integration with the Discovery Network

The browser and the Discovery Network are symbiotic. The browser is the network's primary user interface. The network is the browser's content source.

### The Browser as Discovery Client

The prompt bar is a discovery endpoint. When the user types a capability query, the browser routes it through the four discovery mechanisms:

- **Per-domain manifests** — the agent checks known domains for `.well-known/naze-manifest.json`
- **Capability index** — structural search across the distributed index
- **Federated registries** — industry-specific registries with specialized trust models
- **Peer-to-peer** — services shared by other agents in the network

Results surface in the browser with full transparency: trust score, capability match strength, data flow analysis, external connections. The user sees why a service ranked where it did.

### The Browser as Discovery Publisher

Generated apps don't have to be ephemeral. When a user generates a meal planner and it works well, they can publish it back to the discovery network with one action. The app becomes a discoverable service — its manifest is extracted automatically from the `.naze` source, its trust score is computed from its code, and it's available to every agent on the network.

This creates the flywheel:

1. A user generates a "recipe organizer" from a natural language prompt
2. They publish it to the discovery network
3. Another user asks for a "meal planning suite"
4. The agent discovers the recipe organizer alongside a grocery service and a nutrition tracker
5. The agent composes all three into a meal planning suite
6. The user publishes the composed app
7. Next time someone asks for meal planning, the agent discovers the complete solution — zero tokens spent regenerating what already exists

Each generation enriches the network. Each composition creates something greater than its parts. The ecosystem compounds.

### Trust in the Browser

Trust scores are not hidden metadata. They are a first-class part of the user experience.

When the browser renders a discovered service, the user sees:

- **Trust score** — derived from code analysis, not reviews
- **Data flow** — where data goes, what leaves the device, what stays local
- **External connections** — which third-party services are contacted and why
- **Device access** — camera, location, contacts — what's requested and whether it's justified for the service's purpose
- **Trust profile context** — the same service may score differently in healthcare vs. e-commerce contexts

The parametric trust profiles from the Discovery Network are surfaced here. A mapping service that scores well in e-commerce (location access expected) might score lower in a privacy-focused context (location tracking is a concern). The user sees the trust score in context, not as an abstract number.

## Agent Configuration

The browser is the single location for managing agent identity — the credential wallet.

### Credentials

API keys for LLM providers (Claude, GPT, Gemini, local models), OAuth tokens for external services, payment methods for paid APIs. One place for everything, managed by the user, never sent to any server the user doesn't control.

### Model Preferences

Which model to use for generation. Token budgets per session. Quality vs. speed trade-offs. The user can prefer a frontier model for complex compositions and a small local model for simple edits.

### Approval Policies

Which actions the agent can take autonomously and which require confirmation:

- **Autonomous:** Compile, render, fix compiler errors, retry generation
- **Confirm:** Publish to discovery network, access device APIs, make purchases, send data externally

The policies are granular. A user might allow autonomous publishing of apps they've reviewed but require confirmation for anything that accesses location data. The browser enforces these policies before the agent acts.

### Provider Agnosticism

The browser works with any LLM provider — cloud or local. The lingua franca is Naze, not any provider's API. A Claude-powered agent's published service is discovered identically by a GPT-powered agent or a local LLaMA model. Different providers, different models — same network, same structural matching. The browser makes this seamless: switch providers in settings, and everything else works the same.

## The Embedded Language Spec

Agents operating within the Naze Browser are Naze-native. They don't need to learn the language — the language is already loaded.

The browser bundles:

- **The grammar** — in GBNF and EBNF formats, enabling constrained decoding that guarantees syntactically valid output
- **Language documentation** — the complete reference, from basic elements to server functions
- **Example corpus** — hundreds of working examples demonstrating every language feature
- **Type system** — the full type checker runs in-browser, providing semantic validation beyond syntax

This is what makes generation reliable. The agent doesn't hallucinate syntax. The grammar constrains its output to valid Naze. The type checker catches semantic errors. The retry loop fixes anything that slips through. The user sees working apps, not error messages.

For constrained decoding with local models, the GBNF grammar is tiered:

- **Tier 0 (Core UI):** Layout, elements, state, events — enough for a small model to generate simple apps
- **Tier 0-1 (Data):** Adds fetch, storage, timers — a mid-size model handles data-connected apps
- **Tier 0-2 (Fullstack):** Adds database models, server functions — a capable model builds complete applications

A 7B model with the Tier 0 grammar can reliably generate UI components. A frontier model with the full grammar can generate complete fullstack applications. The browser selects the appropriate tier based on the user's model and the complexity of the request.

## App Lifecycle

Apps in the Naze Browser are not throwaway. They have a lifecycle:

### Generate

The user describes intent. The agent generates a `.naze` application. The browser compiles and renders it. The app exists.

### Use

The app is interactive. Buttons click, inputs accept text, state persists, data fetches complete. It's a real application, not a preview or mockup. The user can use it for as long as they need.

### Iterate

"Add dark mode." "Make the list sortable." "Show a chart of spending over time." Each instruction modifies the running app. The conversation history shows every version — the user can rewind to any previous state.

### Edit

For users who want control, "View Source" opens the `.naze` code. They can edit directly, and the browser recompiles on save. The source is always human-readable — one file, declarative syntax, no hidden state. A user who started by describing can transition to direct editing at any point.

### Fork

See an app you like? Fork it. The browser creates an editable copy. Modify it to your needs. The original is unchanged. Forking is how apps evolve — small variations on proven patterns.

### Publish

Push the app to the discovery network. Its manifest is extracted automatically. Its trust score is computed from its code. It becomes available to every agent and user on the network. Publishing is one action, not a deployment pipeline.

### Compose

Other agents discover the published app and compose it with other services. The app's capabilities become building blocks. The user who built a simple recipe organizer might find it incorporated into a full meal planning platform — built by an agent they've never interacted with, for a user they've never met.

## Data Sovereignty

If interfaces are ephemeral — generated on demand, regenerated at will — then data cannot be. The Naze Browser decouples what every traditional app conflates: the interface and the data.

Today, your invoices live "in" QuickBooks. Your resume lives "in" Google Docs. Your orders live "in" Shopify. Each app is both a UI and a data container. Switching apps means migrating data — or losing it. The app owns your data, not you.

In the Naze Browser, the interface is a `.naze` file that can be regenerated in seconds. What matters is the data underneath. Priya's invoices, Maria's resume, David's customer orders — these need to survive browser crashes, device changes, and interface regeneration. They need to live somewhere permanent, independent of the app that created them.

### Two Persistence Layers

**App persistence** — where the `.naze` source and compiled binary live.

- **Public:** Publish to the discovery network. The app becomes a discoverable, composable service. This is how David's bakery ordering page and the birthday party planner persist and grow the ecosystem.
- **Private:** Save to the user's chosen storage provider. Not everything should be public. Priya's custom invoice tool with her business details, Maria's resume with her personal information — these are private apps stored in the user's own space.

**Data persistence** — where the user's actual data lives. Separate from the app entirely.

- Priya's invoices are not stored "in" her invoice app. They're stored in her data layer. If she regenerates the interface, or forks a different invoice tool, or switches browsers — the invoices are still there.
- David's orders are not stored "in" his ordering page. They're in his data layer. He can swap his ordering interface for a better one without losing a single order.
- Maria's resume is not stored "in" her resume builder. It's in her data layer. She can use three different resume editors on three different devices, all reading from the same source.

### Two Kinds of Data

Not all data is the same. The persistence layer handles both:

**Structured data** — typed, relational, queryable. Todo items with text, status, due date, and category. Invoices with line items linked to clients linked to projects. Orders with customer details, product selections, and timestamps. This is the data that apps create, read, update, and delete in real time.

**Blob data** — documents, images, exports, attachments. Maria's resume as a PDF. Priya's invoice exports. David's product photos. Files that are stored and retrieved but not queried by field.

Both go through the same persistence API. The provider handles the complexity — structured data maps to tables or collections; blob data maps to object storage or a filesystem. The app doesn't care about the distinction. It declares what it needs; the provider handles how.

### Models as Schema

Naze already has declarative model definitions — Prisma-like syntax that describes data shapes:

```
model Todo
  text string
  done bool
  category string
  due_date string
```

```
model Invoice
  client string
  items list
  total number
  status string
  created_at string
```

These model declarations are the schema. The persistence API uses them directly:

- When a user generates a todo app, the agent includes a `model Todo` declaration in the `.naze` source
- The browser reads the model declaration and tells the persistence provider: "this app needs a `Todo` collection with these fields"
- The provider creates the storage — a SQLite table, a Postgres schema, a Firestore collection, whatever it is behind the API
- The app reads and writes through the persistence API using the declared types
- If the user says "add a priority field," the agent updates the model declaration, and the persistence API handles the migration

The user doesn't write SQL. The agent doesn't configure databases. The model declaration in the `.naze` file IS the schema, and the persistence provider makes it real.

### Schema Evolution

Apps evolve. A user starts with a simple todo list, then says "add a priority field," then "add due dates," then "add project categories." Each change modifies the model declaration — and the persistence provider must migrate the existing data to match the new schema without losing anything.

This must be seamless. The user says "add priority." The agent updates the model. The persistence provider diffs the old schema against the new one, applies the migration — adds the column, sets a default for existing records — and the app renders with the new field. Existing todos are still there, now with an empty priority. The user never sees a migration step, a SQL command, or a warning dialog.

Schema evolution is a solved problem. Prisma, Rails, Django, and every mature ORM have handled it for decades. The persistence API contract defines what providers must support: additive changes (new fields, new models) are applied automatically. Destructive changes — removing fields that contain data, changing a field's type in a way that could lose information — require user approval through the browser's approval policies. The agent explains what will happen ("removing the `category` field will delete category data from 47 todos — proceed?") and the user decides.

No external ORM or middleware layer is needed to make this work. The Naze compiler already generates parameterized SQL from model declarations at build time — `find users where id == 5` compiles to `SELECT * FROM users WHERE id = $1`. Schema lifecycle is a natural extension of the same pattern: `model Todo { text string, done bool }` compiles to `CREATE TABLE Todo (text TEXT, done BOOLEAN)`, and diffing an old model against a new one produces `ALTER TABLE Todo ADD COLUMN priority TEXT`. The compiler IS the schema management layer. The persistence provider just executes the operations.

Blob data follows the same principle. Documents and files can have version history — prior versions retrievable, changes trackable. Priya's invoice template from three months ago is still accessible even after twenty iterations. Version history is a provider capability, not a browser requirement — but the persistence API defines the contract for providers that support it.

### The Full Spectrum

The Naze Browser doesn't limit what you can build. The persistence provider determines what's possible at any given scale.

**Local / casual** — A personal todo list. A reading tracker. A recipe collection. These need minimal infrastructure: SQLite on the desktop surface, IndexedDB on the web surface. Zero configuration. The user types a prompt, the app appears, data persists locally. No server, no account, no setup.

**Shared / collaborative** — A family grocery list. A team task board. A small business inventory tracker. These need a hosted backend: Supabase, Turso, PlanetScale, a self-hosted Postgres. The user connects a provider in settings (or the agent discovers one on the network and provisions it). Multiple users can interact with the same data through independently generated interfaces.

**Production / SaaS-scale** — A full project management platform. A CRM. An e-commerce storefront with inventory, orders, customer accounts, and analytics. These need real infrastructure: managed databases, object storage, CDN, perhaps queuing and background jobs. The persistence API is the same contract — but the provider behind it is a production-grade deployment.

The browser is still the interface at every level. The same prompt bar, the same generation loop, the same Canvas2D rendering. What changes is the backend the data writes to. A todo list and a SaaS CRM use the same persistence API — the difference is whether it's backed by a local SQLite file or a managed Postgres cluster with read replicas.

This means there's no cliff. A user doesn't start in the Naze Browser and then "outgrow" it. They start with local storage for a personal tool, connect a cloud provider when they need sharing, scale to production infrastructure when it becomes a business — all through the same interface, the same `.naze` source, the same persistence API. The app evolves. The backend scales. The browser stays.

### The Persistence API

Naze defines a standardized persistence contract — an interface that any backend can implement. The pattern is the same one the project uses everywhere else:

- **LLM providers:** OpenAI, Anthropic, Ollama, custom → same interface, user picks
- **Discovery mechanisms:** per-domain, capability index, federated, P2P → same contract
- **Storage providers:** same pattern → same API, user picks the backend

The persistence API handles structured operations (create, read, update, delete, query, migrate) and blob operations (store, retrieve, list, delete) through the same contract. The API is HTTP-based, language-independent, and backend-swappable — just like the discovery network's endpoints.

Any backend can implement it:

- **Local:** SQLite (desktop), IndexedDB (web) — zero config, zero cost
- **Cloud managed:** Supabase, PlanetScale, Turso, Neon — hosted, scalable, managed
- **Self-hosted:** Postgres, MySQL, MongoDB on your own server — full control
- **Object storage:** S3, GCS, Azure Blob — for documents and files
- **Decentralized:** IPFS, peer-to-peer storage — for users who want no central dependency
- **Minimal:** A JSON file on a VPS — for the simplest possible setup

Naze doesn't build the storage layer. Naze defines the contract. The ecosystem builds the backends.

### Storage as a Discoverable Service

Storage providers can themselves be services on the discovery network. When a user's app needs persistence beyond local storage, the agent can discover providers automatically:

> "Build me a project management tool for my team of five."

The agent generates the `.naze` app with model declarations. It sees the app needs shared structured storage. It queries the discovery network for storage providers matching the requirements: relational data, multi-user, moderate scale. It finds three providers — ranked by trust score, priced by usage. The user picks one. The agent provisions the backend, connects it, and the app is live with shared persistence.

No infrastructure knowledge required. No AWS console. No database setup. The agent handles provisioning the same way it handles code generation — autonomously, with user approval at decision points.

### The User Controls the Data

The data belongs to the user. Not to the app that created it. Not to the service that processes it. Not to the browser that renders it.

This means:

- **Switching browsers** doesn't lose data. Open the Naze Browser on a new device, point it at your storage provider, regenerate your interfaces. Everything is there.
- **Switching storage providers** migrates data. Because every provider implements the same API, moving from S3 to a self-hosted server is a copy operation, not a rewrite.
- **Regenerating interfaces** doesn't lose data. The app is a view over data, not a container of data. Delete the app, regenerate it, use a completely different one — the data persists because it's independent.
- **Device failure** doesn't lose data. Priya's laptop dies. She opens the browser on her phone, connects to her storage provider, and her invoices are there. The interface regenerates. The data was never at risk.

This is the fundamental inversion from the traditional app model. Apps come and go. Data endures. The user owns it.

### Connection to the Discovery Network

The persistence API and the discovery network are complementary:

- The discovery network stores **public capabilities** — services, compositions, manifests, trust scores. It's the shared ecosystem layer.
- The persistence API stores **private data** — user information, business records, personal content. It's the sovereign user layer.

A service published to the discovery network might declare in its manifest: "I produce data of type `Invoice`" or "I need data of type `Recipe`." The browser's agent reads these declarations and routes data to/from the user's persistence layer automatically. The service never touches the user's storage directly — it goes through the browser, which enforces the user's approval policies.

This is also how data portability works. If Maria uses a resume builder that produces data of type `Resume`, and a different resume builder also reads type `Resume`, she can switch between them freely. The data schema — defined in the manifest — is the compatibility layer. The data lives in her storage. The interface is whatever she prefers.

## Emergent Behaviors

The browser and discovery network together produce behaviors that no single component was designed to create.

### Strengthened Pathways

When the browser's agent discovers that "recipe + grocery + nutrition" gets composed together frequently, that composition pattern itself becomes discoverable. Future agents don't need to figure out the combination — the network already knows it works.

### Diminishing Cold-Start

Early on, most requests require generation from scratch. Over time, the network accumulates proven compositions. Common requests — "build me a todo app," "make a dashboard," "create a booking system" — resolve to existing, battle-tested applications in milliseconds. The token cost per request approaches zero for common patterns.

### Distributed Intelligence

A powerful model solves a complex composition once and publishes it. That solution IS the knowledge — frozen on the network. A small 7B model running on a phone can deliver the same result by discovering it, without spending the tokens to reason from scratch. Intelligence decouples from model size. Access to good results decouples from access to expensive models.

### Natural Selection

A cleaner implementation of the same capability appears — better structured, higher trust score, faster response. Agents start preferring it. The old one fades. Apps evolve without anyone deprecating anything.

### Composition Depth

Apps compose with apps that are themselves compositions. A "party planner" composed from recipe + venue + catering services might itself be composed into a "wedding coordinator" alongside invitation + photography + travel services. Layers of composition, each one a single `.naze` file, each one a typed manifest the next agent can read.

## Two Surfaces, One Core

The core loop is platform-agnostic. The LLM generates a `.naze` source string. The compiler WASM takes that string and returns a binary blob. The runtime WASM takes that blob and renders to Canvas2D. No filesystem. No native APIs. No platform dependencies. Anywhere WASM runs, the Naze Browser runs.

This means two deployment surfaces — web and desktop — sharing the same core but offering different capabilities at the edges.

### The Web Surface

Zero-install, shareable via URL, accessible from any device with a browser.

The web surface is the entry point. A user clicks a link and is immediately in the Naze Browser — no download, no setup. They type a prompt, the LLM generates code, the WASM compiler produces a binary, the WASM runtime renders the app. The entire pipeline runs in-browser.

Storage uses web platform APIs: IndexedDB for saved apps and conversation history, localStorage for preferences and model settings. LLM access is via cloud APIs — the user enters an API key in settings, and fetch calls go directly to OpenAI, Anthropic, or any compatible endpoint.

The web surface is good for: first experiences, sharing apps via URL, casual generation, mobile access, situations where installing software isn't practical.

### The Desktop Surface

A native shell (Tauri or equivalent) wrapping the same web core, with platform capabilities that browsers can't provide.

The desktop surface adds:

- **OS keychain** for credential storage — API keys and OAuth tokens stored securely, isolated from the rendering surface
- **Filesystem access** for project persistence — save apps as `.naze` files, organize into directories, version with git
- **Bundled local LLM** (Ollama or equivalent) — the entire generate-compile-render loop works offline, no internet required
- **Native window management** — real tabs, system menus, keyboard shortcuts, multi-window
- **GPU rendering** (future) — bypass Canvas2D entirely, go from layout engine to GPU directly

The desktop surface is good for: daily use, offline work, credential security, local AI, serious app development, power users.

### What's Shared

Everything that matters is shared between surfaces:

- The compiler WASM (`naze-playground` crate)
- The runtime WASM (`naze-runtime` crate)
- The discovery network client
- The UI layer (prompt bar, conversation history, app viewport)
- The agent logic (generation, retry, composition)
- The embedded language spec (grammar, docs, examples)

One codebase. Two deployment targets. The web surface proves the concept and provides reach. The desktop surface provides depth. A user can start on the web, generate an app, share it via URL — and later install the desktop version for offline use, local models, and secure credential management. Their apps, conversation history, and preferences carry over.

## Why a Desktop Surface

The web surface works. Everything described in this document — generation, discovery, composition, the full app lifecycle — runs in a browser tab. So why build a desktop surface at all?

### Offline Generation

With a local model, the entire loop — prompt, generate, compile, render — works without internet. The browser bundles the grammar, examples, and compiler. Combined with cached discovery results, even composition can work offline. This is impossible in the web surface, which depends on cloud LLM APIs.

### Credential Security

API keys and OAuth tokens stored in the OS keychain are isolated from the rendering surface. A malicious discovered service cannot access credentials through the rendering pipeline. In the web surface, credentials live in localStorage — accessible to any JavaScript running in the same origin.

### Native Performance

The desktop surface can eventually bypass Canvas2D entirely. The layout engine produces positioned rectangles; a native renderer can send those directly to the GPU without the browser's compositing overhead. This matters for complex apps with hundreds of elements or high-refresh animations.

### The Agent Runtime

In the desktop surface, the agent runs in-process with direct access to the compiler, discovery client, and credential wallet. No postMessage serialization, no worker overhead, no browser tab lifecycle management. The agent is a first-class citizen of the application, not a guest in someone else's sandbox.

## Connection to FAAD

FAAD — Fully Autonomous Agentic Development — is the paradigm where AI agents handle the complete software lifecycle: build, test, debug, deploy, maintain. Humans provide direction and approval.

The Naze Browser is the human interface to FAAD.

The user's role is to describe intent: what they want, how it should work, what to change. The agent's role is everything else — discovering existing solutions, generating new code, compiling, rendering, fixing errors, iterating until the result matches the intent.

This is not "AI-assisted development." The user is not a developer using AI tools. The user is a person with a need, and the browser is the mechanism that translates that need into a working application. The code exists, and it's readable, and anyone can edit it — but nobody has to.

The three-layer architecture of Naze makes this possible:

- **Layer 1 (Data):** State, computed values, server functions, data bindings — the semantic layer agents work with (~500 bytes for a typical app)
- **Layer 2 (Interaction):** Event handlers, navigation, actions, validation — the behavior layer
- **Layer 3 (Presentation):** UI tree, themes, animations, layout — the visual layer (~6KB)

An agent building a meal planner works primarily in Layer 1 (what data, what computations) and Layer 2 (what happens when the user clicks). Layer 3 follows from sensible defaults and theme application. The agent doesn't spend tokens on pixel-pushing — it works at the semantic level, and the runtime handles the rest.

This is why FAAD works for Naze and struggles for the traditional web. In React, an agent must manage JSX templates, CSS files, state management libraries, build configurations, and framework-specific patterns — scattered across dozens of files. In Naze, everything is in one file, the language has one canonical form per concept, and the agent's token cost scales linearly with application size.

The Naze Browser is where FAAD meets the user. The agent builds. The network remembers. The user benefits. And every interaction makes the next one cheaper, faster, and smarter.
