# Naze Roadmap

The long-term vision for Naze in six phases, from proof of concept to a full platform.

**Current stats:** 11 crates, 389 workspace tests, ~157 grammar rules, 395KB WASM runtime (budget: 405KB), 109 examples.

---

## Phase 1: Proof of Life — Complete

Establish the end-to-end pipeline: `.naze` source compiles to WASM + HTML, renders colored rectangles and text in the browser via Canvas2D.

**Delivered:** 7 Rust crates, 84 tests, 69KB WASM runtime, 16 example files, custom layout engine (~200 LOC), custom binary serialization format. Components with typed parameters and defaults. `use` imports. `nazec new`, `build`, `check`, `parse` CLI commands.

See [MVP.md](MVP.md) for details.

---

## Phase 2: Real Apps + Developer Experience — Complete

Make apps dynamic and interactive. Add developer tooling. Prototype cross-platform rendering.

See [HISTORY.md](HISTORY.md) for the consolidated milestone record.

**Phase 2a (M1-M14): Complete** — state, events, conditionals, slots, images, theming, layout (flex-grow/shrink, percentages, align/justify), routing, form inputs with validation, drag & drop, accessibility (ARIA, screen reader DOM, focus management), scroll containers, data fetching, animation with easing.

**Phase 2b (M9-M12): Complete** — VS Code extension with LSP, TextMate grammar, and visual editor. Dev server with WebSocket hot reload. Native desktop builds via `nazec run`. Android build prototype via `nazec build --target android`.

---

## Phase 3: Language Completion & Developer Experience — Complete

Complete the language with advanced computation features, finalize tooling, add testing framework.

See [HISTORY.md](HISTORY.md) for the consolidated milestone record (M15-M22).

**Delivered:** Pipeline operators (M15), pattern matching (M16), layout templates & responsive design (M17), advanced animation (M18), component events & theme inheritance (M19), overlay system (M19b), visual properties (M19c), application logic primitives (M19d), remaining gap closures — textarea, JS interop, device APIs (M19e), testing framework (M20), build pipeline polish (M22). All milestones complete except M21 (LSP polish, moved to Phase 6 as M45).

---

## Phase 4: Ecosystem & External Integration — Complete

Server rendering, package ecosystem, AI integration, SEO.

See [HISTORY.md](HISTORY.md) for the consolidated milestone record (M23-M30).

**Delivered:** WASM module imports (M23), server functions (M24), SSG (M25a), SSR + hydration (M25b), local package dependencies (M26a), package registry (M26b), SEO/meta-index (M27), AI grammar export (M28a), AI validation & fine-tuning (M28b), AI prompting runtime (M29), advanced dev tools — inspector, playground, size analyzer (M30). All milestones complete.

---

## Phase 5: Production-Ready Application Platform — Complete

Harden Naze for production deployment with environment config, dynamic routing, error handling, authentication, and database integration.

See [HISTORY.md](HISTORY.md) for the consolidated milestone record.

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
- **M41:** WASM size optimization (wasm-opt enabled, unused web-sys features removed, format!() reduced → 395KB)

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

## Future: The Naze Browser

A dedicated browser where the URL bar takes natural language and responses are live, interactive applications — not text, not code, but running software built on the fly. The browser is the user-facing interface for the agent ecosystem, the Discovery Network's primary client, and the human interface to FAAD.

Three modes of use: **generate** (describe intent, get a working app), **discover** (find existing services by capability, not name), and **compose** (agent discovers packages and wires them together into something new). Apps are not ephemeral — they can be saved, forked, edited, and published back to the discovery network, creating a flywheel where each generation enriches the ecosystem.

See [NAZE_BROWSER.md](NAZE_BROWSER.md) for the full vision.

---

## Design Principles

These hold across all phases:

1. **AI-native, human-readable.** The language is designed as a compilation target for AI, but `.naze` files should be readable by anyone. No framework magic, no hidden state, no implicit behavior.

2. **Kilobytes, not megabytes.** The runtime is 395KB after wasm-opt. Every dependency is justified by the bytes it costs.

3. **One source, every platform.** The same `.naze` file should render identically on web, desktop, and mobile. Platform differences are handled by the renderer, not the language.

4. **Compile-time over runtime.** Components are inlined, theme tokens are resolved, types are checked, and dead code is eliminated at compile time. The runtime does as little work as possible.

5. **No middle layers.** No bundler, no transpiler, no CSS preprocessor, no virtual DOM. Intent goes to pixels through the shortest path: parse, typecheck, serialize, deserialize, layout, render.

6. **Token-efficient by construction.** Naze targets Λ-Linear token complexity — AI cost scales linearly with application size because components are self-contained, styling is inline, and each concept has one canonical form. See [TOKEN_EFFICIENCY.md](TOKEN_EFFICIENCY.md) for the full framework.
