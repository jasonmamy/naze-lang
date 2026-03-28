# Ideas

> Raw brainstorm ideas for Naze's future direction. Some of these have been evaluated and moved to [FUTURE.md](docs/FUTURE.md) (with Psi impact analysis); others were rejected (e.g., orchestration primitives). This file preserves the original thinking.

---

## 1. "Vibe-to-App" Generative Runtime with Live Refinement

Turn the Naze Browser concept into a true agentic shell where the prompt bar isn't just generation — it's a persistent, conversational runtime.

- Users (or agents) describe intent ("Build a personal knowledge base that auto-tags entries from my email and suggests connections"), and the system generates/runs the .naze app instantly.
- **Live refinement loop:** While the app runs, follow-up prompts like "Make the tagging use semantic embeddings" trigger incremental recompilation and hot-swapping of components (leveraging the flat compile-time inlining).
- **Human/AI hybrid mode:** A split view showing the running Canvas app alongside editable .naze source + natural language "why" explanations generated from the code. Agents could propose changes as diffs in the source.

This evolves the browser from "chat → static output" to "intent → living, evolvable tool." It directly competes with emerging AI-native browsers but grounds everything in verifiable, minimal Naze code instead of opaque LLM hallucinations.

---

## 2. Evolutionary Discovery Network + Self-Improving Ecosystem

The Discovery Network (capability-based matching via typed schemas, trust scores based on simplicity, federated registries) is already biological-inspired. Push it further into a "neural network of services":

- **Natural selection layer:** When agents compose services (e.g., a party planner pulling from cake + venue + scheduling), the resulting composite gets published back with performance metrics (token cost, success rate, latency). Over time, the network promotes "fitter" implementations — simpler code wins higher trust scores automatically.
- **Immune system:** Agents flag brittle or hallucination-prone services; the network quarantines or suggests minimal rewrites in Naze.
- **Emergent macro-apps:** Common patterns (e.g., "todo with sync") become reusable "memes" that small local models pull from the network, reducing cold-start problems and letting 3-7B models punch above their weight.
- **Optional Tier 5: Evolutionary Primitives** — built-in syntax for mutation operators or genetic-style crossover of components, so agents can evolve apps autonomously.

This turns the web into a distributed, self-healing intelligence layer rather than static pages.

---

## 3. Agent-Native Persistence and Memory Layer

Data is already decoupled in Naze (stored in user-chosen backends like SQLite). Expand this into a declarative memory fabric:

- Built-in primitives for vector + relational hybrid storage (e.g., `memory "notes" { schema: { embedding: vector, tags: list, relations: graph } }`).
- Apps declare intent for long-term memory ("persist user preferences across sessions with privacy controls"), and the compiler generates secure, sandboxed storage adapters.
- Agents get a headless memory API in the ~500B binary layer, so they can query/update shared state without loading the full UI.

This solves one of the biggest pain points for autonomous agents: persistent, queryable context across sessions without token bloat or external databases reinvented every time.

### Expanded: Hybrid Vector + Relational Memory Primitives

Add a new grammar tier (e.g., T3 Memory) with declarative syntax:

```naze
memory "knowledge"
  schema
    id uuid primary
    content text
    embedding vector[384] index hnsw
    tags list<string>
    relations graph<relation_type>
  persist auto
  privacy encrypted, consent_required
```

The compiler would generate:
- Runtime bindings for vector similarity search (integrate lightweight libs like `usearch` or `sqlite-vec` in WASM/native targets)
- Automatic embedding hooks (call an external model or built-in tiny embedder on insert)
- Query sugar: `query knowledge where embedding ~ "user intent" limit 5` that compiles to efficient hybrid SQL + vector ops

### Expanded: Declarative Long-Term Memory Fabric

Apps declare high-level intent rather than low-level DB ops:

```naze
memory fabric "user_profile"
  sources [local_sqlite, cloud_sync]
  retention 2 years with decay
  access agent_read_write if trust > 0.8
```

This enables stateful agents across sessions without re-parsing everything. An agent could query a user's "personal knowledge base" app, pull embeddings + relations, reason, then update — all via the headless L1 binary. Data survives UI regeneration (e.g., you refine the visual layer via natural language, but notes/relations stay intact).

### Expanded: Agent Memory API in Headless Layer

The ~500-byte binary exposes typed, versioned endpoints for memory ops (get/put/query/summarize). Agents treat Naze services as reliable external memory modules instead of scraping or maintaining their own vector stores. Add optional episodic memory primitives that auto-log agent interactions for audit or self-reflection loops.

**Why this fits the AI trajectory:** Traditional web apps tie data to brittle sessions or external services that agents struggle to introspect. Naze makes persistence machine-first: tiny, typed, schema-exposed, and backend-portable. It reduces token waste (agents don't need to re-ingest full state) and enables emergent behaviors like personal "second brains" that evolve with user + agent input.

**Implementation tip (solo-friendly):** Start by extending the existing model → storage mapping. Add vector support via a lightweight WASM-compatible crate first for local SQLite, then layer on cloud options.

---

## 4. Multi-Agent Orchestration Primitives

Extend the language with lightweight orchestration tiers (building on the existing event and state layers):

- Syntax for declaring roles/teams: `agent team "research" { roles: [searcher, synthesizer]; workflow: parallel → merge }`
- Built-in patterns for common agent workflows (ReAct-style reasoning, tool use, multi-agent debate) that compile to efficient WASM state machines.
- Discovery Network integration: Agents automatically recruit other Naze services by capability during orchestration.

This makes Naze a natural host for multi-agent systems (like CrewAI or AutoGen patterns, but compiled down to tiny binaries instead of Python orchestration overhead).

### Expanded: Role and Team Declarations

New syntax in the Interaction or a new T4 Orchestration tier:

```naze
agent team "research_assistant"
  roles
    searcher capability search_web or browse_page
    synthesizer capability summarize
    critic capability critique_accuracy
  workflow parallel(searcher, critic) → merge(synthesizer)
  timeout 30s
  fallback retry or degrade_gracefully
```

The compiler flattens this into L1 server functions + state machines. Agents invoke the team via the headless binary, getting back structured results.

### Expanded: Built-in Workflow Patterns

Provide reusable primitives for common agentic patterns:

- **ReAct-style:** `reason_act_loop { tools: [search, calculate] }`
- **Multi-agent debate:** `debate { participants: [optimist, pessimist]; rounds: 3 }`
- **Hierarchical:** Parent agent spawns child sub-teams with scoped memory.

These compile to tiny, verifiable state machines — far more efficient than runtime Python loops.

### Expanded: Discovery-Aware Orchestration

Teams automatically recruit external Naze services via the Discovery Network:

```naze
on query
  recruit from discovery where capability matches "calendar_sync" and trust > 0.7
```

The manifest's typed schemas make matching reliable. Composed teams can be published back as new discoverable services, creating compounding intelligence.

This turns Naze into a substrate for scalable multi-agent systems (think lighter-weight AutoGen or CrewAI, but with verifiable binaries and zero dependency bloat). In a post-90s-web world, agents won't want to orchestrate via brittle APIs or screen-scraping — they'll prefer composing tiny, typed Naze binaries that expose clear capabilities.

**Implementation tip:** Leverage the existing event handlers and state updates. The orchestration layer can desugar to expanded state + server functions initially, keeping the grammar small.

> **Note:** This idea was evaluated and **rejected** in [FUTURE.md](docs/FUTURE.md). Orchestration is moving into the model layer (extended thinking, agent SDKs), not application code. Naze should be an excellent tool for agents, not an orchestration framework.

---

## 5. "Headless Web" Bridge and Legacy Wrappers

The existing bridge: sites can add ~25 lines of .naze to expose capabilities (menu, order function, etc.) alongside their HTML, without rewriting anything. Agents consume the clean L1 + manifest instead of parsing markup.

### Expanded: Automated Wrapper Generator

Build a tool (or agent prompt template) that ingests:
- An OpenAPI/Swagger spec,
- A website URL + scraped schema, or
- Even a screenshot + description.

It outputs a minimal .naze wrapper:

```naze
wrapper "legacy_bakery" from "https://oldbakery.com"
  expose
    menu list<item>
    order function(cart) -> confirmation
    location geo
  trust_adjust -0.1
```

The compiler generates the L1 headless binary that proxies calls (with rate limiting, auth passthrough, and sanitization). Over time, popular wrappers could be refined into native Naze services.

### Expanded: Sandboxed Legacy Mode in Runtime

Extend the Naze runtime (WASM/Canvas or native) with an optional "compatibility view":
- Embed a minimal WebView or proxy for pure legacy pages when no wrapper exists.
- Default to Canvas for any new Naze components.
- Agents get a flag in the manifest: `has_legacy_fallback: true` with quality/trust penalty.

### Expanded: Gradual Migration Incentives

Discovery Network scoring could explicitly reward native Naze services (lower token cost, higher trust, simpler data flows) over wrappers. Agents would naturally drift toward fully native services for efficiency, accelerating the phase-out of the old stack.

This creates a practical on-ramp: the web doesn't have to die overnight. Agents start by wrapping legacy services cheaply, then prefer (and evolve toward) native .naze implementations. It bridges today's reality with the vision of a capability-based, binary-first agent web.

**Implementation tip:** The wrapper generator could be a separate CLI tool or even a Naze app itself — low-risk to prototype and high demo value.

---

## 6. Sustainability + Verification Dashboard

Lean into the energy/CO2 math:

- **Built-in telemetry layer** that reports per-app token/energy estimates and trust metrics.
- **"Naze Observatory" dashboard** (itself a Naze app) showing global impact if adoption grows — visualizing savings vs. traditional stacks.
- **Formal verification primitives** for critical components (e.g., `prove safety { no side effects outside declared bounds }`), helping agents audit before composition.

---

## 7. Cross-Modal and Embodied Extensions

For longer-term ambition:

- **Tier for multimodal declarations** (e.g., voice/gesture inputs mapped directly to events, or AR overlays via Canvas/WebXR).
- **Integration hooks for robotics/IoT:** Compile the same .naze file to embedded targets, so a "smart home controller" app works for both web humans and agent-controlled devices.

---

## Implementation Advice (Solo Dev)

Prioritize incremental wins that attract contributors without scope explosion:

1. Start with the live refinement loop in the browser prototype — it's high-visibility and demos the "beyond chatbots" value immediately.
2. Export more grammar/tools for constrained decoding and local models; this lowers barriers for AI researchers to experiment.
3. Document "why Naze" with concrete before/after token traces against popular agent frameworks.
4. Open specific "bounty-style" issues on GitHub for things like the evolutionary scoring algorithm or memory primitives.

The core insight — that the 90s stack isn't compatible with AI's trajectory — is correct and timely. Most "AI-native" efforts today still fight the old web (scraping, vision models on screenshots, brittle automation). Naze sidesteps that by making the entire artifact machine-first from the ground up.

These ideas keep the project radical yet grounded in what's already built: minimalism, efficiency, and agent primacy. If even a subset lands, Naze could become the substrate for the post-web agentic era.
