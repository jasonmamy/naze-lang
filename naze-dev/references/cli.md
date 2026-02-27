# nazec CLI Reference

`nazec` is the Naze compiler and development tool. It is a single self-contained binary that embeds the WASM runtime.

## Commands

### `nazec new <name>`

Scaffold a new project with `naze.toml` and `app.naze`.

```bash
nazec new my-app
cd my-app
```

Creates:
```
my-app/
  naze.toml      # Project manifest
  app.naze       # Entry file
```

### `nazec build`

Compile `.naze` source to distributable output.

```bash
nazec build                    # Default web target
nazec build --target web       # Explicit web target
nazec build --target native    # Native desktop target
nazec build --static           # Static site generation (SSG)
```

Output directory: `dist/`
```
dist/
  index.html             # Generated HTML shell
  naze_runtime.js        # WASM loader
  naze_runtime_bg.wasm   # Runtime engine
  app_data.bin           # Compiled app data
```

Serve with any HTTP server (WASM requires HTTP, not `file://`):
```bash
python3 -m http.server -d dist 8080
```

### `nazec dev`

Start development server with hot reload.

```bash
nazec dev               # Default port 3000
nazec dev --port 4000   # Custom port
```

Watches `.naze` files for changes and auto-recompiles. Opens browser automatically.

### `nazec serve`

Production SSR server with server function support.

```bash
nazec serve               # Default port 8080
nazec serve --port 3000   # Custom port
```

Handles server functions, data fetching, and SSR rendering.

### `nazec check`

Type-check without building. Fast feedback on errors.

```bash
nazec check
```

### `nazec test`

Run `.test.naze` test suites.

```bash
nazec test                # Run all tests
nazec test --format json  # JSON output
```

### `nazec parse <file>`

Dump the AST as JSON. Useful for debugging grammar issues.

```bash
nazec parse app.naze
```

### `nazec grammar`

Export the grammar for LLM constrained decoding.

```bash
nazec grammar              # Default format
nazec grammar --format gbnf  # GBNF format
nazec grammar --format ebnf  # EBNF format
```

### `nazec analyze`

WASM binary size analyzer. Shows what contributes to binary size.

```bash
nazec analyze
```

### `nazec gallery`

Build the interactive example gallery.

```bash
nazec gallery          # Serve gallery
nazec gallery --build  # Build static gallery
```

### `nazec run`

Preview in a native desktop window with hot reload.

```bash
nazec run
```

## Package Management

### `nazec add <package>`

Add a dependency to `naze.toml`.

```bash
nazec add ui-components
```

### `nazec remove <package>`

Remove a dependency from `naze.toml`.

### `nazec update`

Update all dependencies to latest compatible versions.

### `nazec publish`

Publish the current package to the Naze registry.

### `nazec search <query>`

Search the package registry.

```bash
nazec search "date picker"
```

## AI Tools

### `nazec ai generate`

Generate Naze code from a description.

### `nazec ai fix`

Fix errors in Naze source using AI.

### `nazec ai dataset`

Generate training data for AI models.

## Project File (`naze.toml`)

```toml
[app]
name = "my-app"
version = "0.1.0"

[build]
entry = "app.naze"
output = "dist/"
```

## Development Workflow

1. `nazec new my-app` -- create project
2. Edit `app.naze` with your UI
3. `nazec dev` -- start dev server with hot reload
4. Iterate on your `.naze` files
5. `nazec check` -- type-check before committing
6. `nazec build` -- produce final dist/ output
7. Deploy `dist/` to any static host

## Environment Variables

- `DATABASE_URL` -- PostgreSQL connection string (required for server functions with database models)
- `PORT` -- Override default server port for `dev` and `serve`
