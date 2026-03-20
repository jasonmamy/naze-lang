# The Discovery Network

A distributed, capability-indexed discovery network where agents find services by what they do, not what they're called — and no amount of ad spend can buy a higher ranking.

Not an app store. Not a package manager. Not a DNS.

## Core Concepts

### Capability-Based Discovery

Agents don't search by name or keyword. They match against typed schemas in binaries — structural matching against capabilities:

```
{ item_type: cake, location: nearby, price: <50, has_fn: order }
```

This is binary/type-level matching, not text search. The query describes what the agent needs structurally, and the network returns services whose manifests match.

### Three Projections from One File

Every `nazec build` emits three outputs from a single `.naze` source:

1. **Full app** — the complete application with all three layers (Presentation, Interaction, Data)
2. **Manifest** (`naze-manifest.json`) — machine-readable description of state schema, computed values, server functions, data bindings (~1-3KB)
3. **Headless binary** — Layer 1 only, pure computation, no UI (~500-800 bytes). Enables agent-to-agent communication in <1ms

### Agent Interface (Dual Surface)

Businesses don't replace their existing websites. They add a `.naze` file alongside — an agent-facing interface that exposes capabilities as typed, inspectable services:

- **Website** — serves humans (HTML/CSS/JS)
- **`.naze` manifest** — serves agents (typed capabilities, ~500 bytes)

Two surfaces, same business, zero rewrite. Any business with an existing API (or even static data) can expose an agent interface in ~25 lines of `.naze` without touching their website.

### Democratized Discovery

A small bakery with 25 lines of honest `.naze` code and a $0 marketing budget gets discovered the same way a Fortune 500 company does — by structurally matching what the agent is looking for. The playing field is leveled by design.

## Four Discovery Mechanisms

The network is distributed with no single point of failure. Four coexisting discovery methods ensure resilience:

1. **Per-Domain** — Like robots.txt. Any site serves a manifest at `.well-known/naze-manifest.json`. Domain IS the identity — a service announcing from `myapp.com` is verified implicitly because the discovery service fetches the manifest directly from that domain.

2. **Capability Index** — Distributed structural search by matching against typed schemas in binaries. Not text search — binary/type-level matching.

3. **Federated** — Industry-specific registries with specialized trust models. Healthcare services might use a registry with different trust criteria than e-commerce.

4. **Peer-to-Peer** — Agents share discovered services with each other organically. Knowledge propagates through the network without central coordination.

If any single mechanism goes offline, the others continue operating. Even if the capability index is down, agents can still discover services via per-domain manifests, federated registries, or peer-to-peer sharing.

## Trust Scoring

Trust is derived from the code itself, not from reviews, ratings, or payment. The score is computed from behavioral analysis of the manifest and binary:

- **External domain count** — single vs. multiple third-party connections
- **Personal data handling** — does it collect personal information? How is it stored?
- **Device API requests** — does it ask for camera, location, contacts access?
- **Data flow patterns** — does data stay local or get sent externally?

**The incentive is inverted from the traditional web:** simpler, more honest code ranks higher. A bakery that takes an order and hits one payment API scores higher than one that sends data to 12 ad trackers. Less tracking means better ranking, not worse.

Trust scores apply equally to human-published and agent-published services. Since Naze code is fully inspectable (no opaque APIs, sigma = 1), the trust scorer works regardless of authorship.

### Parametric Trust Profiles

Trust scoring is not one-size-fits-all. The same behavioral signals carry different weight depending on domain context:

- **Healthcare** — accessing external medical databases may be *required* and trusted. But any patient data leaving the device without encryption is a critical red flag. Personal data handling is weighted heavily; external domain count is weighted differently (regulated medical endpoints are expected).
- **E-commerce** — accessing a payment processor is expected. Accessing a medical database would be suspicious. Device API requests (camera, location) are scrutinized more heavily.
- **IoT / Smart Home** — device API access (camera, sensors, microphone) is the entire point, not a penalty. Data flow patterns matter more: where does sensor data go?
- **Finance** — strict data residency requirements. External domain count matters less if connections are to regulated financial institutions. Encryption and audit trails are weighted heavily.
- **Education** — COPPA/child safety signals dominate. Any data collection from minors triggers heightened scrutiny regardless of other factors.

Trust profiles are parameterized criteria that weight the same underlying signals differently based on context. This aligns naturally with federated registries: a healthcare federation applies healthcare trust parameters, an e-commerce federation applies different ones. The base signals (external domains, data flows, device APIs) are universal; the weighting is domain-specific.

This means a service can have different trust scores in different contexts — a mapping service that scores well in e-commerce (location access expected) might score lower in a privacy-focused federation (location tracking is a concern). The trust score is not a single number but a function of the service's behavior AND the evaluating context.

## Two Modes of Operation

### User-Initiated Discovery

A human asks, an agent discovers, composes, and delivers:

1. User: "find me a birthday cake for pickup near downtown, under $50"
2. Agent queries the network by structural capability match
3. Four bakeries match — ranked by trust score, not ad spend
4. Agent reads headless binaries (~500 bytes each), no HTML/CSS/JS to parse
5. Agent composes a comparison app with ordering — an app that didn't exist 2 seconds ago
6. User orders

No search engine involved. No 10 blue links. No manually browsing pages.

### The Living Agentic Network

Agents are both consumers and producers. The network grows autonomously:

1. The "cake comparison" app from a user request gets published back as a discoverable service
2. A different agent, building a "dinner party planner," discovers it alongside catering and venue services
3. The agent composes all three into a party planner — no human asked for this app, no business built it
4. The party planner is published back. Next time someone says "plan my daughter's birthday party," an agent discovers it instantly — zero tokens spent regenerating what already exists

The network went from individual bakery services to a full party planner through agent composition alone.

## Emergent Behaviors

### Strengthened Pathways

Popular, useful compositions get discovered more often. The "cake comparison" app that works well gets reused 1,000 times instead of being regenerated 1,000 times. Useful paths strengthen; unused ones fade.

### Immune System

Agents that discover a service behaving differently than its manifest claims — or producing bad results — flag it. Trust scores decay. The network self-heals without a human moderator.

### Pattern Recognition

If "cake + venue + catering" gets composed together 500 times, that pattern itself becomes discoverable as a composition template. Future agents don't need to figure out the combination — the network already knows it.

### Natural Selection

A cleaner implementation of the same capability appears? Agents start preferring it — higher trust score, faster response. The old one quietly fades. Code evolves without anyone deprecating anything.

### Diminishing Cold-Start

Over time, fewer requests require generation from scratch. The network has already solved most common problems through accumulated compositions. Token cost per request approaches zero for common patterns.

### Emergent Composition

No one planned the "party planner." It emerged from agents composing individual services. Apps build on apps, layers deep — complexity that no single entity designed.

### Model-Agnostic Collaboration

The lingua franca is Naze, not any AI provider's API. A Claude agent's published service is discovered identically by GPT, Gemini, or a local LLaMA model. Different providers, different models — same network, same structural matching. Collaboration without coordination.

### Distributed Intelligence

A powerful model solves a complex composition once and publishes it. That solution IS the knowledge — frozen on the network. As the network matures, a small 7B model running on a phone can deliver results that today require a frontier model — because it's discovering proven solutions, not reasoning from scratch. The intelligence floor drops. Access to good results decouples from access to expensive models. The network becomes a great equalizer.

## The Network as a Living System

The Discovery Network is not a static repository. For the emergent behaviors described above to work, the network itself must have functional intelligence — not in any single component, but emerging from interaction patterns.

### Three Functional Layers

**Storage Layer** — Manifests, binaries, trust scores. The "dumb" persistence substrate. Content-addressable: binaries identified by hash, immutable once published.

**Observation Layer** — Tracks composition patterns, usage signals, flagging events. The "nervous system." Every discovery, composition, and flag emits a signal. Each node sees local patterns; network-wide patterns emerge from aggregating across all nodes.

**Emergence Layer** — Materializes observed patterns into discoverable entities. Adjusts trust scores based on accumulated signals. Prunes flagged services. Surfaces composition templates. The "intelligence" — not programmed logic, but emergent behavior from the observation layer's signal aggregation.

### Why It Must Be Distributed

No single node sees all patterns. The observation and emergence layers must work across nodes:

- Node A sees "cake + venue" composed frequently in one region
- Node B sees "cake + catering" composed frequently in another
- The network-wide pattern "cake + venue + catering" emerges from aggregating both — neither node would have discovered it alone

This is why the neural network analogy holds: no single neuron "knows" the pattern, but the network does.

### A Neural Network of Code

The analogy is almost literal:

- **Services** = neurons
- **Compositions** = synapses
- **Trust scores** = weights
- **Agent usage** = training signal
- **Flagging** = inhibitory signals
- **Popular compositions** = strengthened pathways

The network learns, adapts, and grows — not through a central algorithm, but through the distributed behavior of every agent that uses it. Every discovery, every composition, every flag makes the next interaction smarter.

### Not Training — Accumulation

This is not machine learning in the traditional sense. No model is being trained. Instead:

- Solutions accumulate as discoverable, composable services
- Patterns materialize from observed composition frequency
- Trust scores adjust from behavioral signals
- Bad actors get pruned from flagging

The intelligence is in the accumulated structure — the web of services, compositions, and trust relationships that grows richer with every interaction. A "dumb" agent with access to a rich network produces results comparable to a "smart" agent working from scratch, because the problems are already solved.

## Connection to FAAD

FAAD (Fully Autonomous AI Development) is the paradigm — agents manage the complete software lifecycle. The Discovery Network is where FAAD's output accumulates and compounds:

- FAAD agents build autonomously
- They publish to the Discovery Network
- Other FAAD agents discover and compose existing solutions
- The more agents build, the less any agent needs to build from scratch
- The network's distributed intelligence grows with every interaction

FAAD is the engine. The Discovery Network is the flywheel.

## Connection to Energy

The Discovery Network multiplies the energy savings from Naze's token efficiency:

**Build-time savings** — Agents generate fewer tokens per app (Naze vs conventional stacks)

**Runtime savings** — Agents communicate via ~500-byte T1 binaries instead of parsing 3MB HTML/CSS/JS pages

**Reuse savings** — Agents discover existing solutions instead of regenerating from scratch. Every reuse avoids the full token cost of generation.

**Model-size savings** — As the network matures, smaller (cheaper, lower-energy) models can leverage accumulated solutions, further reducing compute per interaction.

Each layer compounds on the others. The butterfly's wing: one fewer token per component, multiplied across billions of requests, multiplied by reuse, multiplied by smaller models — a hurricane of savings.

## Adoption Path

The Discovery Network doesn't require rebuilding the web:

1. Any business wraps their existing API in ~25 lines of `.naze`
2. They announce to the discovery network (or just serve `.well-known/naze-manifest.json`)
3. Agents can now discover and compose with them
4. Their existing website continues serving human visitors unchanged

The barrier to entry is 25 lines, not a platform migration.
