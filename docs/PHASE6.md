# Phase 6: Developer Experience & Adoption

**Goal:** Make Naze usable and discoverable by external developers. After Phase 6, a developer can discover Naze online, try it in a playground, install the CLI and VS Code extension with one command, follow a tutorial to build an app, deploy it to production, and share components via the registry. The AI generation claim is validated with benchmarks and a published fine-tuned model.

**Status:** In Progress. M42 (CI/CD) and M44 (Playground) complete.

**Architecture shift:** Phases 1-5 built the language, compiler, runtime, and server infrastructure. Phase 6 builds the adoption infrastructure around it. No major language changes — the focus is CI/CD, documentation, distribution, tooling polish, and ecosystem seeding.

**Prerequisite:** Phases 1-5 complete (M1-M41). 385 workspace tests. WASM binary: 374KB.

---

## Dependency Graph

```
M42 (CI/CD) ─────────────────────── foundation for everything
  ├── M43 (Docs Site) ──────── independent, enables M47
  ├── M44 (Playground) ─────── independent
  ├── M45 (LSP Polish) ─────── independent (the last Phase 3 item)
  ├── M46 (Distribution) ───── depends on M42 (needs release builds)
  ├── M47 (AI Validation) ──── independent, informed by M43 examples
  ├── M48 (Standard Lib) ───── depends on M46 (needs deployed registry)
  └── M49 (Prod Deploy) ────── independent
```

**Implementation order:** M42 → M45 → M44 → M43 → M46 → M49 → M47 → M48

---

## M42: CI/CD Pipeline
**Location:** `.github/workflows/`, `Makefile`

Automated quality gates on every commit. Foundation for binary distribution (M46) and release management.

- [x] GitHub Actions workflow: `cargo test --workspace` on push/PR (exclude WASM-only crates)
- [x] GitHub Actions workflow: `cargo clippy --workspace -- -D warnings` lint check
- [x] GitHub Actions workflow: `cargo fmt --all -- --check` format check
- [x] WASM size regression check: fail if `naze_runtime_bg.wasm` exceeds budget
- [x] Matrix testing: stable + nightly Rust, Linux + macOS
- [x] Release workflow: build binaries for Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)
- [x] Release workflow: attach binaries + checksums to GitHub Release on tag push
- [x] Cache `~/.cargo` and `target/` for faster CI runs

---

## M43: Documentation Site
**Location:** `docs-site/` (new directory)

A public documentation website — the primary entry point for new developers. Use a static site generator (Docusaurus, mdBook, or VitePress).

- [ ] Getting Started guide: install → `nazec new` → `nazec dev` → first app running in 2 minutes
- [ ] Language Reference: complete syntax for all ~140 grammar rules with examples
- [ ] Tutorial: "Build a Todo App" — state, events, conditionals, each, data fetching, pages
- [ ] Tutorial: "Build a Dashboard" — templates, computed, pipelines, theming
- [ ] Concepts: compile pipeline, render tree model, Canvas2D rendering, SSR/SSG modes
- [ ] CLI Reference: all 17 `nazec` commands with flags and examples
- [ ] AI Guide: `nazec ai generate`, grammar-constrained decoding, fine-tuning workflow
- [ ] Server Functions Guide: `server function`, `sql`, database integration, auth patterns
- [ ] Package Guide: `nazec add/publish`, registry, `@org/package` namespacing
- [ ] Deploy to GitHub Pages or Vercel (automated from CI)
- [ ] SEO: each page has title, description, Open Graph tags

---

## M44: Hosted Playground
**Crate:** `naze-playground` (WASM backend exists), **Location:** `playground/` (new frontend)

An interactive web page where anyone can write Naze code and see it render live — no installation required. The compiler WASM module already exists (`crates/naze-playground/`); this milestone builds the frontend.

- [x] Web frontend: split-pane editor (left: code, right: Canvas2D preview)
- [x] Code editor with syntax highlighting (CodeMirror 6 with custom Naze language mode)
- [x] Live compilation on keystroke (debounced): compile → serialize → load into runtime → render
- [x] Error display: parse/type errors shown inline below editor
- [x] Example selector: load curated examples (6 already embedded in playground WASM)
- [x] Share via URL: encode source in URL hash
- [x] Deploy to GitHub Pages (static — GitHub Actions workflow)
- [x] Mobile-responsive layout (stack editor/preview vertically on small screens)

---

## M45: VS Code Extension Polish (M21 Completion)
**Crate:** `naze-lsp`, **Dir:** `editors/vscode/`

The last remaining Phase 3 milestone. Developers need real-time type errors in their editor.

- [ ] Type-checking diagnostics: run lightweight single-file validation on `didChange` (parse + type-check, report errors as diagnostics)
- [ ] Cross-file go-to-definition: resolve `use` imports to component source files
- [ ] Format document: integrate `nazec fmt` or implement LSP-based formatter
- [ ] Signature help: show parameter names/types for components and functions on trigger
- [ ] Publish to VS Code Marketplace under "illuminaze" publisher
- [ ] Extension README with feature screenshots, installation instructions, keybindings

---

## M46: Binary Distribution
**Location:** `.github/workflows/release.yml`, `install.sh`, Homebrew formula

Zero-friction installation. A developer should go from zero to `nazec new` in under 60 seconds.

- [ ] `cargo install nazec` — publish to crates.io (requires crate metadata: description, repository, license)
- [ ] GitHub Releases: prebuilt binaries for Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64
- [ ] Install script: `curl -fsSL https://naze.dev/install.sh | sh` (detects OS/arch, downloads binary)
- [ ] Homebrew formula: `brew install naze` (tap: `naze-lang/homebrew-naze`)
- [ ] Version checking: `nazec --version` shows semver + git hash
- [ ] Update checking: `nazec` prints notice when a newer version is available (check GitHub releases API, cached daily)

---

## M47: AI Validation & Model
**Crates:** `nazec` (ai.rs, dataset tools), **Location:** `ai/` (new directory for training data)

Validate the core differentiator: Naze is the most token-efficient language for AI code generation.

- [ ] Benchmark: generate 50 UI specs in Naze vs React/HTML, measure token count, success rate, retry rate
- [ ] Benchmark: test grammar-constrained decoding (GBNF) with llama.cpp on Llama 3 8B / Mistral 7B
- [ ] Seed dataset: hand-write 200 instruction/code pairs covering all major features
- [ ] Expanded dataset: use `nazec ai dataset export` on all 75 examples to generate JSONL
- [ ] Fine-tune: QLoRA on Llama 3 8B with Naze dataset, evaluate on held-out test set
- [ ] Publish model to HuggingFace and Ollama library
- [ ] Integration: `nazec ai generate --provider ollama --model naze-7b` uses the fine-tuned model
- [ ] Write up results in docs site AI Guide (M43)

---

## M48: Standard Library Packages
**Location:** `packages/` (new directory), registry deployment

Seed the package ecosystem with official packages that demonstrate best practices.

- [ ] Deploy registry to a hosted environment (Fly.io, Railway, or VPS — SQLite is fine for low traffic)
- [ ] Add auth to registry: API key-based publish authentication
- [x] `@naze/ui-kit`: button, card, badge, avatar, alert, progress, divider, tooltip, chip, accordion, tabs, skeleton (12 components)
- [x] `@naze/forms`: form-field, text-field, select-field, checkbox-field, radio-group, search-input (6 components)
- [ ] `@naze/icons`: icon system — either embedded SVG-to-image pipeline or glyph font approach
- [x] `@naze/layouts`: navbar, sidebar-layout, hero, footer, page-shell, center-card (6 templates)
- [ ] Publish all packages to deployed registry
- [ ] Update docs site with package catalog and usage examples

---

## M49: Production Deployment Guide
**Location:** `docs-site/` (deploy section), `deploy/` (new directory for templates)

Nobody can deploy a Naze app to production today. Fix that.

- [ ] `Dockerfile` template: multi-stage build (cargo build → nginx serve static) for SSG apps
- [ ] `Dockerfile` template: SSR mode (nazec serve as entrypoint) for server-rendered apps
- [ ] `docker-compose.yml` example with database (PostgreSQL) for full-stack apps
- [ ] Deploy guide: Vercel (SSG), Fly.io (SSR), Cloudflare Pages (SSG), Railway (SSR)
- [ ] CDN configuration: cache headers, asset hashing, gzip/brotli compression
- [ ] Environment variables: document `[env]` section patterns for staging/production
- [ ] Health check endpoint for SSR mode (`/health`)
- [ ] Error boundary patterns for production (graceful degradation guide)

---

## Deferred (Not in Phase 6)

These items are tracked but intentionally deferred beyond Phase 6:

| Item | Reason |
|------|--------|
| Dedicated Naze browser | Premature — no ecosystem to consume. Revisit after significant public Naze content exists |
| GPU rendering (WebGPU/Metal/Vulkan) | Canvas2D + tiny-skia sufficient for current scale |
| Proper text engine (HarfBuzz/FreeType) | Complex, needed for i18n/RTL — track separately |
| i18n/localization | Design needed — touches grammar, compiler, and runtime |
| RTL layout support | Depends on text engine and i18n design |
| SVG/vector graphics | Needs design — Canvas2D can draw paths but no SVG parser exists |
| Edge deployment (WASI HTTP) | Low demand until production apps exist |
| SSR streaming | Optimization — basic SSR works |
| Embedded AI authoring layer (C13) | Research phase — fine-tuned model (M47) is prerequisite |
| `zip` pipeline, list comprehensions, match destructuring | Low-priority language sugar |
| Pipeline fusion, constant folding | Compiler optimizations — premature |
| Virtual scrolling | Optimization for large lists |
| WASM module merging (wasm-merge) | Optimization for WASM import overhead |
| Android APK end-to-end | Low demand, WebView prototype exists |

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Installation time (zero to `nazec new`) | < 60 seconds |
| Playground: code → render | < 2 seconds |
| Documentation pages | 20+ |
| Published VS Code extension | On Marketplace |
| Standard library packages | 4+ on deployed registry |
| AI generation benchmark | Naze success rate > React success rate at equal model size |
| Binary distribution | 3+ channels (cargo, brew, curl script) |
| CI pipeline | Green on every merge to main |
| Public Naze apps | 3+ (tutorials count) |

---

## Totals

| Metric | Value |
|--------|-------|
| Milestones | 8 (M42-M49) |
| Current workspace tests | 385 |
| Current WASM binary | 374KB |
| Current grammar rules | ~140 |
