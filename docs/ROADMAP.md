# Naze Roadmap

The long-term vision for Naze in six phases, from proof of concept to a full platform.

**Current stats:** 11 crates, 385 workspace tests, ~153 grammar rules, 374KB WASM runtime, 69 examples.

---

## Phase 1: Proof of Life — Complete

Establish the end-to-end pipeline: `.naze` source compiles to WASM + HTML, renders colored rectangles and text in the browser via Canvas2D.

**Delivered:** 7 Rust crates, 84 tests, 69KB WASM runtime, 16 example files, custom layout engine (~200 LOC), custom binary serialization format. Components with typed parameters and defaults. `use` imports. `nazec new`, `build`, `check`, `parse` CLI commands.

See [MVP.md](MVP.md) for details.

---

## Phase 2: Real Apps + Developer Experience — Complete

Make apps dynamic and interactive. Add developer tooling. Prototype cross-platform rendering.

See [PHASE2.md](PHASE2.md) for the detailed milestone tracker.

**Phase 2a (M1-M14): Complete** — state, events, conditionals, slots, images, theming, layout (flex-grow/shrink, percentages, align/justify), routing, form inputs with validation, drag & drop, accessibility (ARIA, screen reader DOM, focus management), scroll containers, data fetching, animation with easing.

**Phase 2b (M9-M12): Complete** — VS Code extension with LSP, TextMate grammar, and visual editor. Dev server with WebSocket hot reload. Native desktop builds via `nazec run`. Android build prototype via `nazec build --target android`.

---

## Phase 3: Language Completion & Developer Experience — Complete

Complete the language with advanced computation features, finalize tooling, add testing framework.

See [PHASE3.md](PHASE3.md) for the detailed milestone tracker (M15-M22).

**Delivered:** Pipeline operators (M15), pattern matching (M16), layout templates & responsive design (M17), advanced animation (M18), component events & theme inheritance (M19), overlay system (M19b), visual properties (M19c), application logic primitives (M19d), remaining gap closures — textarea, JS interop, device APIs (M19e), testing framework (M20), build pipeline polish (M22). All milestones complete except M21 (LSP polish, moved to Phase 6 as M45).

---

## Phase 4: Ecosystem & External Integration — Complete

Server rendering, package ecosystem, AI integration, SEO.

See [PHASE4.md](PHASE4.md) for the detailed milestone tracker (M23-M30).

**Delivered:** WASM module imports (M23), server functions (M24), SSG (M25a), SSR + hydration (M25b), local package dependencies (M26a), package registry (M26b), SEO/meta-index (M27), AI grammar export (M28a), AI validation & fine-tuning (M28b), AI prompting runtime (M29), advanced dev tools — inspector, playground, size analyzer (M30). All milestones complete.

---

## Phase 5: Production-Ready Application Platform — Complete

Harden Naze for production deployment with environment config, dynamic routing, error handling, authentication, and database integration.

See [PHASE5.md](PHASE5.md) and [PHASE5B.md](PHASE5B.md) for the detailed milestone trackers.

**Delivered:**
- **M31:** Environment variables (`[env]` in naze.toml, compile-time interpolation)
- **M32:** Dynamic routing (`:param` path segments, catch-all `*` routes)
- **M33:** Error boundaries (`error` block with fallback UI and retry)
- **M34:** Multi-statement server functions (sequential steps, variable binding, SQL queries)
- **M35:** Page meta & guards (`meta` blocks for per-route SEO, `guard` with auth checks and redirects)
- **M36:** Auth patterns (header interpolation in data declarations, server-side auth forwarding)
- **M37:** Database integration (SQL queries in server functions, PostgreSQL via feature flag)
- **M38:** Production polish (environment-aware builds, SSR guard evaluation)
- **M39:** Declarative database queries (Prisma-like `model` definitions, compile-time SQL generation for find/insert/update/delete)
- **M40:** Browser API parity (textarea rendering, real Notification API, JS interop, device APIs — geolocation + accelerometer)
- **M41:** WASM size optimization (wasm-opt enabled, unused web-sys features removed, format!() reduced → 374KB)

---

## Phase 6: Developer Experience & Adoption — In Progress

Make Naze usable and discoverable by external developers.

See [PHASE6.md](PHASE6.md) for the detailed milestone tracker (M42-M49).

**M42 (CI/CD Pipeline): Complete** — GitHub Actions for test/lint/release, WASM size regression check, matrix testing, release workflow with cross-platform binaries.

**M44 (Hosted Playground): Complete** — Split-pane editor with CodeMirror 6, live compilation, error display, example selector, URL sharing, mobile-responsive layout.

**Remaining:**
- **M43:** Documentation site (getting started, language reference, tutorials)
- **M45:** VS Code extension polish (type-checking diagnostics, cross-file go-to-def, Marketplace publish)
- **M46:** Binary distribution (crates.io, Homebrew, install script, GitHub Releases)
- **M47:** AI validation & model (benchmarks, fine-tuning, published model)
- **M48:** Standard library packages (official component packages on deployed registry)
- **M49:** Production deployment guide (Dockerfiles, deploy guides, CDN config)

---

## Future: Dedicated Browser

The Naze browser is the user-facing interface for the agent ecosystem. It solves the agent bootstrap problem — how does an AI agent discover Naze services, learn the language, and manage credentials? — by embedding all of this into the app the user is already running.

### Rendering (native performance)

- Standalone WASM runtime (no browser engine overhead)
- Native window with GPU rendering (no Canvas indirection)
- URL bar, navigation, tabs, bookmarks
- HTML fallback (embedded lightweight webview for legacy content)
- Content-type detection: route `application/naze` to native pipeline, HTML to webview

### Embedded discovery

The browser ships with `discover.naze.dev` built in — the same way Chrome ships with Google Search as the default. No bootstrap problem: the agent's entry point to the discovery network is the app the user already has open. Structural search from the URL bar: type a capability query ("find services with cart and checkout") and the browser queries the discovery service directly.

### Agent configuration

Settings page where the user enters their API key for Claude, GPT, Gemini, or other LLM providers. Model selection, token budgets, and approval policies (which actions the agent can take autonomously vs. which require confirmation). The browser becomes the single place to manage agent identity — the credential wallet described in [AGENT_RUNTIME.md](AGENT_RUNTIME.md). One location for API keys, OAuth tokens, and payment methods across all Naze services.

### Embedded language spec

The grammar (GBNF/EBNF from `nazec grammar`), language documentation, and example corpus are bundled into the browser. An agent operating within the browser has full Naze comprehension without searching the web — the specification is part of its runtime context. This is what makes the agent "Naze-native": it doesn't need to learn the language, the language is already loaded.

### Agent execution environment

The `naze-agent` crate (see [AGENT_RUNTIME_PLAN.md](AGENT_RUNTIME_PLAN.md), Phase C) runs headless binaries in-process. The user sees results rendered natively. The browser is both the human UI and the agent execution environment — the agent loads a service binary, executes actions, and the user watches state changes in real time.

### Justification

This isn't "only justified once there's significant Naze content." The browser is how agents and users interact with the Naze ecosystem. It connects three concepts that are otherwise disconnected: native rendering performance, the credential wallet from the agent runtime vision, and the discovery bootstrap from the implementation plan. The `naze-agent` MCP server (Post-C) also makes the browser an integration surface — any MCP-compatible orchestrator gains Naze's typed discovery without rebuilding it. Everything in Phases 1-6 runs in standard browsers today — the dedicated browser is the upgrade path for users who want the full agent-native experience.

---

## Design Principles

These hold across all phases:

1. **AI-native, human-readable.** The language is designed as a compilation target for AI, but `.naze` files should be readable by anyone. No framework magic, no hidden state, no implicit behavior.

2. **Kilobytes, not megabytes.** The runtime is 374KB after wasm-opt. Every dependency is justified by the bytes it costs.

3. **One source, every platform.** The same `.naze` file should render identically on web, desktop, and mobile. Platform differences are handled by the renderer, not the language.

4. **Compile-time over runtime.** Components are inlined, theme tokens are resolved, types are checked, and dead code is eliminated at compile time. The runtime does as little work as possible.

5. **No middle layers.** No bundler, no transpiler, no CSS preprocessor, no virtual DOM. Intent goes to pixels through the shortest path: parse, typecheck, serialize, deserialize, layout, render.

6. **Token-efficient by construction.** Naze targets Λ-Linear token complexity — AI cost scales linearly with application size because components are self-contained, styling is inline, and each concept has one canonical form. See [TOKEN_EFFICIENCY.md](TOKEN_EFFICIENCY.md) for the full framework.
