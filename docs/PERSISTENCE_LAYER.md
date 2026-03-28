# The Persistence Layer

The persistence layer sits between Naze apps and storage backends. It translates model declarations into database schemas, compiles declarative queries to parameterized SQL, handles schema migrations as apps evolve, and routes data to the user's chosen provider — all at compile time, with no runtime ORM overhead.

This document covers what already exists, what needs to be built, and how the architecture extends to support relations, the persistence API contract, and the full spectrum of storage providers described in [NAZE_BROWSER.md](NAZE_BROWSER.md).

## What Naze Already Has

The compile-time SQL layer is surprisingly complete. Four pieces are in place:

### Model Declarations

Naze's grammar (Tier 2: Database) includes declarative model definitions:

```
model Todo
  text string
  done bool
  category string

model Invoice
  client string
  items list
  total number
  status string
  created_at timestamp
```

Four field types: `string`, `number`, `bool`, `timestamp`. Three constraints: `primary`, `unique`, `default <value>`. Models are parsed into the AST, lowered into the IR as `ModelDecl`, and included in the `RenderTree`. They are compile-time declarations — the runtime never sees them. They exist so the compiler can generate correct SQL.

### Declarative Query Compilation

Queries in server functions compile to parameterized SQL at build time:

```
server function list-todos()
  find Todo where done == false order created_at desc limit 50

server function add-todo(text: string, category: string)
  insert Todo {text: text, category: category, done: false}

server function complete-todo(id: number)
  update Todo set {done: true} where id == id

server function remove-todo(id: number)
  delete Todo where id == id
```

The compiler transforms these into `IrServerStep::Sql` with parameterized queries:

- `find Todo where done == false` → `SELECT * FROM Todo WHERE done = $1` with params `[false]`
- `insert Todo {text: text, done: false}` → `INSERT INTO Todo (text, done) VALUES ($1, $2) RETURNING *`
- `update Todo set {done: true} where id == id` → `UPDATE Todo SET done = $1 WHERE id = $2 RETURNING *`
- `delete Todo where id == id` → `DELETE FROM Todo WHERE id = $1 RETURNING *`

Parameters are numbered sequentially. WHERE clauses with multiple conditions join with AND. ORDER BY and LIMIT are supported. All parameter values are type-safe — resolved from server function arguments at runtime.

### Dual Backend Dispatch

At runtime, server functions execute SQL against the database specified by `DATABASE_URL`:

- PostgreSQL URLs (`postgres://...`) → `postgres` crate, native parameterized queries with `$1, $2, ...`
- SQLite URLs (`sqlite:///path/to/db`) → `rusqlite` crate (bundled), with automatic `$N` → `?` placeholder conversion

Results are marshalled row-by-row into `RenderValue::List<RenderValue::Object>`, where each object maps column names to typed values. The app receives structured data regardless of which database is behind it.

### Server Functions as Boundary

Server functions are the execution boundary. Database queries can only appear inside server function bodies. Client-side UI calls server functions via `data` declarations or event handlers. This boundary is enforced at parse time — the grammar only allows query expressions inside `server_function_def` blocks.

## Prisma Ecosystem Comparison

Prisma is the closest existing tool to what Naze's persistence layer does. But Naze needs approximately one component from Prisma's ecosystem. The rest is either already handled by the language or unnecessary.

### What Naze already covers

| Prisma Component | What It Does | Naze Equivalent |
|---|---|---|
| **Schema Language** | `schema.prisma` — model definitions | `model` declarations in `.naze` files. Simpler type system (4 types vs ~15), but self-contained in the same file as the app (σ = 1). |
| **Query Compiler** | Compiles client calls → SQL | `find`/`insert`/`update`/`delete` compile to parameterized SQL in `codegen.rs`. Zero runtime overhead — SQL is generated at build time. |
| **Prisma Client** | Generated type-safe query builder | Not needed. Queries are language syntax, not library calls. Type safety is enforced by the compiler at parse time. |
| **Prisma Studio** | GUI for browsing/editing data | The generated Naze app IS the GUI. The user interacts with their data through the app they described, not a separate database admin tool. |
| **Introspection** | Database schema → Prisma schema | Not needed. The model declaration is the source of truth. Naze generates schemas from models, never the other way around. |
| **Seeding** | Populate with initial data | Not needed. The agent includes initial data in the `.naze` source. "Build me a todo app with some example todos" generates seed data as part of the app. |
| **Type Generation** | Schema → TypeScript types | Not needed. The model declaration IS the type. The compiler enforces type safety directly. |
| **Middleware** | Query interceptors (logging, auth, soft delete) | Not needed. Server functions, guards, and language-level patterns handle these concerns. Middleware is an ORM concept; Naze handles it at the language level. |

### What Naze doesn't need

| Prisma Component | What It Does | Why Naze Doesn't Need It |
|---|---|---|
| **Prisma Accelerate** | Edge caching + connection pooling | Provider responsibility. Supabase, PlanetScale, and other managed services handle this. Not Naze's concern. |
| **Prisma Pulse** | Real-time change events | Could be a future provider capability for collaborative apps. Not a core requirement. |
| **Multi-database drivers** | Postgres, MySQL, MongoDB, CockroachDB, SQL Server | The persistence API contract means any backend can be supported. Providers implement drivers, not Naze. Currently Postgres + SQLite; the ecosystem adds more. |
| **CLI migration workflow** | `prisma migrate dev`, `prisma migrate deploy` | In the Naze Browser context, migrations are automatic — the agent updates the model, the persistence layer handles it. No developer CLI workflow. |

### What Naze needs to build (the gap)

| Prisma Component | What It Does | Naze Gap |
|---|---|---|
| **Schema Engine** | Diff schemas, generate + apply migrations | The only significant gap. See next section. |
| **Relations** | Foreign keys, joins, nested queries | Not needed for simple apps. Needed for complex ones. See the Relations section below. |

## The Schema Engine

This is the gap. The compiler generates queries but doesn't generate schemas. When a user says "build me a todo app," the compiler produces `SELECT * FROM Todo WHERE...` but nothing creates the `Todo` table. When the user says "add a priority field," nothing alters the table.

### What needs to be built

Five capabilities, all extensions of the existing compile-time codegen pattern:

**1. DDL Generation — model to CREATE TABLE**

```
model Todo
  id number primary
  text string
  done bool
  category string
```

Generates:

```sql
CREATE TABLE IF NOT EXISTS Todo (
  id INTEGER PRIMARY KEY,
  text TEXT NOT NULL,
  done BOOLEAN NOT NULL,
  category TEXT NOT NULL
);
```

Type mapping: `string` → `TEXT`, `number` → `INTEGER` or `REAL`, `bool` → `BOOLEAN`, `timestamp` → `TIMESTAMP`. Constraints: `primary` → `PRIMARY KEY`, `unique` → `UNIQUE`, `default <value>` → `DEFAULT <value>`.

This is approximately 30-40 lines of Rust in `codegen.rs`, following the same pattern as the existing `compile_find_to_sql`.

**2. Schema Diffing — old model vs new model**

Given the previous model and the new model, produce a list of changes:

- Fields added: `priority string` is new → `AddColumn { name: "priority", type: "TEXT" }`
- Fields removed: `category` is gone → `DropColumn { name: "category" }`
- Fields changed: `done bool` became `done string` → `AlterColumn { name: "done", old_type: "BOOLEAN", new_type: "TEXT" }`
- Constraints changed: `id number` gained `primary` → `AddConstraint { field: "id", constraint: "PRIMARY KEY" }`

The diff algorithm compares two `Vec<ModelField>` by field name and produces a list of migration operations. This is approximately 50-60 lines of straightforward comparison logic.

**3. Migration Generation — diff to ALTER TABLE**

Each migration operation maps to SQL:

- `AddColumn` → `ALTER TABLE Todo ADD COLUMN priority TEXT`
- `DropColumn` → `ALTER TABLE Todo DROP COLUMN category`
- `AlterColumn` → `ALTER TABLE Todo ALTER COLUMN done TYPE TEXT` (Postgres) or recreate table (SQLite, which doesn't support ALTER COLUMN)

SQLite's limited ALTER TABLE support requires special handling — adding columns works, but changing or dropping columns requires creating a new table, copying data, dropping the old one, and renaming. This is well-understood and documented.

Approximately 40-50 lines of Rust, with Postgres and SQLite variants.

**4. Migration Tracking**

A `_naze_migrations` table tracks which migrations have been applied:

```sql
CREATE TABLE IF NOT EXISTS _naze_migrations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  app_name TEXT NOT NULL,
  schema_hash TEXT NOT NULL,
  applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

Before applying migrations, the persistence layer checks whether the current schema hash matches the last applied migration. If it matches, no migration needed. If it differs, compute the diff and apply.

The schema hash is a deterministic hash of the model declarations — field names, types, and constraints in sorted order. Comparing hashes is O(1); diffing only happens when the hash changes.

**5. Destructive Change Detection**

Some migrations lose data:

- Dropping a column that has values
- Changing a column type in a way that can't be cast
- Dropping a table

These must be flagged. In the Naze Browser context, the agent presents the destructive change to the user for approval: "Removing the `category` field will delete category data from 47 todos — proceed?" The user decides.

Non-destructive changes (adding a column, adding a constraint to empty data) apply automatically.

### Estimated Scope

The entire schema engine is approximately 150-200 lines of Rust, added to the existing codegen and server function infrastructure. No external dependencies. No ORM. No migration framework. Just an extension of the compile-time SQL generation pattern that already exists.

## Relations

The current model system is flat. Each model is an independent table with no foreign keys, no joins, no nested queries. This works for simple apps:

- A todo list — one model, one table, no relations
- An invoice tracker — one model per concept (Invoice, LineItem), but no enforced relationship between them
- A recipe collection — one model, self-contained

It breaks for complex apps:

- A project management tool — Projects have Tasks. Tasks have Comments. Users belong to Teams. Teams own Projects. These are relational.
- An e-commerce storefront — Products belong to Categories. Orders have LineItems. Customers have Addresses. Orders belong to Customers.
- A CRM — Contacts belong to Companies. Deals have Stages. Activities link to Contacts and Deals.

### What relations mean for Naze

Relations introduce foreign keys and joins. The question is how to express them in Naze's declarative, single-file style while maintaining σ = 1 (all information in one file).

### Possible syntax

Following Naze's existing patterns — declarative, minimal, one canonical form:

```
model Team
  id number primary
  name string

model User
  id number primary
  name string
  email string unique
  team Team          # belongs-to: User has a team_id foreign key

model Project
  id number primary
  title string
  team Team          # belongs-to: Project has a team_id foreign key

model Task
  id number primary
  title string
  status string
  project Project    # belongs-to: Task has a project_id foreign key
  assignee User      # belongs-to: Task has a assignee_id foreign key

model Comment
  id number primary
  body string
  task Task          # belongs-to: Comment has a task_id foreign key
  author User        # belongs-to: Comment has a author_id foreign key
  created_at timestamp default now
```

The rule: when a field's type is another model name, it's a foreign key. The compiler generates `team_id INTEGER REFERENCES Team(id)` for the column and resolves joins automatically.

This preserves σ = 1 — all models are in the same `.naze` file. No separate schema file, no reference to external type definitions.

### Query implications

Relations affect queries. Three new capabilities would be needed:

**Filtering across relations:**
```
server function team-tasks(team-id: number)
  find Task where project.team.id == team-id order created_at desc
```

Compiles to: `SELECT Task.* FROM Task JOIN Project ON Task.project_id = Project.id JOIN Team ON Project.team_id = Team.id WHERE Team.id = $1 ORDER BY Task.created_at DESC`

**Including related data (eager loading):**
```
server function task-details(task-id: number)
  find Task where id == task-id include comments, assignee
```

Compiles to multiple queries or a JOIN, returning the task with its comments and assignee nested as objects.

**Cascading operations:**
```
server function delete-project(project-id: number)
  delete Project where id == project-id cascade
```

Compiles to: `DELETE FROM Project WHERE id = $1` with cascading foreign key constraints.

### Impact on σ = 1

Relations stay within σ = 1 as long as all models are in the same file. The AI agent generating a project management app would include all model declarations in the single `.naze` source. The models reference each other by name, and the compiler resolves the references within the same compilation unit.

This differs from Prisma, where `schema.prisma` is a separate file from the application code. In Naze, the schema IS part of the application. The agent doesn't need to read a separate schema file to understand the data model — it's right there in the `.naze` source alongside the UI, state, and event handlers.

### When relations are needed

Relations are not needed for the MVP of the Naze Browser. Simple apps — todo lists, invoice trackers, recipe collections, personal dashboards — work with flat models. The user says "build me a todo app" and gets a working app without relational complexity.

Relations become necessary when:

- The user says "build me a project management tool" (projects → tasks → comments)
- The user says "build me an e-commerce store" (products → categories, orders → line items → products)
- The user iterates a simple app into something complex ("add teams to my task tracker")

The schema engine (CREATE TABLE, migrations) should be built first. Relations layer on top of it — they extend the model syntax, the query compiler, and the DDL generation. The migration system handles relation changes (adding/removing foreign keys) the same way it handles field changes.

## The Persistence API Contract

The persistence API is an HTTP contract — language-independent, backend-swappable. Any provider can implement it. The contract defines six categories of operations:

### Schema Operations

| Operation | Description |
|---|---|
| `POST /schema/apply` | Apply a schema (create tables, run migrations). Accepts model declarations, returns migration result. |
| `POST /schema/diff` | Compare current schema against proposed schema. Returns list of changes without applying them. |
| `POST /schema/status` | Return current schema state — which models exist, which migrations have been applied. |
| `POST /schema/rollback` | Rollback the most recent migration. Returns previous schema state. |

### Structured Data Operations (CRUD)

| Operation | Description |
|---|---|
| `POST /data/{model}/query` | Query records. Accepts filters, sorting, pagination, includes (for relations). Returns typed records. |
| `POST /data/{model}/insert` | Insert one or more records. Accepts typed field values. Returns inserted records with generated IDs. |
| `POST /data/{model}/update` | Update records matching a filter. Accepts set fields and filter conditions. Returns updated records. |
| `POST /data/{model}/delete` | Delete records matching a filter. Returns deleted record count. |

### Blob Operations

| Operation | Description |
|---|---|
| `POST /blob/store` | Store a blob (file, document, image). Accepts binary data + metadata. Returns blob ID. |
| `GET /blob/{id}` | Retrieve a blob by ID. Returns binary data + metadata. |
| `GET /blob/list` | List blobs with optional filtering by type, date, tags. Returns metadata list. |
| `DELETE /blob/{id}` | Delete a blob. |
| `GET /blob/{id}/versions` | List version history for a blob (if provider supports versioning). |
| `GET /blob/{id}/version/{version}` | Retrieve a specific version of a blob. |

### Connection Management

| Operation | Description |
|---|---|
| `GET /health` | Health check. Returns provider status and capabilities. |
| `GET /capabilities` | Return what the provider supports: relations, versioning, full-text search, real-time, etc. |

Providers implement the operations they support. A simple SQLite provider implements schema + CRUD. An S3 provider implements blob operations. A full-featured provider (Supabase, PlanetScale) implements everything. The browser's agent checks `/capabilities` to know what's available and adapts accordingly.

## Provider Spectrum

The same API, different backends, different capabilities:

### Local Providers (zero config)

**IndexedDB (web surface)** — Structured data stored in the browser's IndexedDB. Schema operations create/modify object stores. CRUD uses IndexedDB transactions. No server needed. Data persists across sessions but is tied to the browser/device. Good for: personal apps, offline use, trying things out.

**SQLite (desktop surface)** — A SQLite file on disk. Full SQL support. Schema operations generate SQLite-compatible DDL (with workarounds for ALTER TABLE limitations). CRUD uses parameterized queries. Data persists on the filesystem. Good for: desktop apps, local development, single-user tools.

### Hosted Providers (shared, scalable)

**Managed Postgres** (Supabase, Neon, PlanetScale, Turso) — Full relational database with connection pooling, backups, scaling. Schema operations use standard PostgreSQL DDL. CRUD uses parameterized SQL. Providers handle connection management, replication, and backups. Good for: shared apps, team tools, production workloads.

**Custom self-hosted** — Any server implementing the persistence API. Could be a Postgres instance on a VPS, a Go service backed by CockroachDB, a Python service backed by MongoDB. The API is the contract; the implementation is up to the provider. Good for: users who want full control, specialized backends, compliance requirements.

### Blob Providers

**S3 / GCS / Azure Blob** — Object storage for documents, images, exports. Blob operations map directly to the provider's native API. Versioning is a native capability of most object storage providers. Good for: file-heavy apps, document management, media storage.

**Local filesystem (desktop)** — Files stored in a directory. Blob operations are file read/write. Versioning via git or simple copy-on-write. Good for: desktop apps, development, personal document management.

### The Provider Doesn't Limit the App

A todo app backed by IndexedDB and a project management platform backed by Supabase use the same persistence API. The app's `.naze` source is identical — only the provider configuration changes. Scaling from local to hosted is a settings change, not a rewrite.

The browser's agent can also discover providers on the Discovery Network. "Build me a team task board" → the agent sees it needs shared structured storage → queries the network for persistence providers → presents options with pricing and trust scores → the user picks one → the agent provisions and connects it. No infrastructure knowledge required.
