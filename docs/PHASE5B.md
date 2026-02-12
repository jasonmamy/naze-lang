# Phase 5B: Extensions (M39-M41)

**Goal:** Close remaining gaps after Phase 5 core (M31-M38). M39 adds declarative database queries. M40 completes browser API parity. M41 optimizes WASM binary size.

**Phase 5 status:** M31-M38 all complete. M39-M41 complete. 382 workspace tests passing. WASM binary: 374KB. See [PHASE5.md](PHASE5.md).

---

## M39: Declarative Database Queries
**Crates:** `naze-parser` (grammar, AST), `naze-compiler` (codegen, typecheck)

Declarative model definitions and type-safe query expressions that compile to parameterized SQL.

- [x] `model name { field type constraint... }` top-level definition
- [x] `find model where field == value order field limit N` query expression in server functions
- [x] `insert model { field: value, ... }` expression in server functions
- [x] `update model set { field: value } where field == value` expression in server functions
- [x] `delete model where field == value` expression in server functions
- [x] Compile-time SQL generation — no new IR types, all queries lower to `IrServerStep::Sql`
- [x] Type checker collects model definitions, warns on undefined model references
- [x] Parser tests for model definitions and all query expression types (9 tests)
- [x] Compiler tests verifying generated SQL correctness (5 tests)

---

## Syntax

```naze
model users {
  id number primary
  name text
  email text unique
  active bool
  created_at timestamp default now
}

server function get-users() {
  let users = find users where active == true order name limit 10
  users
}

server function get-user(id: number) {
  let users = find users where id == id limit 1
  users
}

server function create-user(name: text, email: text) {
  let user = insert users { name: name, email: email }
  user
}

server function update-user(id: number, name: text) {
  let result = update users set { name: name } where id == id
  result
}

server function remove-user(id: number) {
  let result = delete users where id == id
  result
}
```

## SQL Generation (compile-time)

| Query | Generated SQL |
|-------|--------------|
| `find users where active == true limit 10` | `SELECT * FROM users WHERE active = $1 LIMIT $2` |
| `find users where id == id limit 1` | `SELECT * FROM users WHERE id = $1 LIMIT 1` |
| `insert users { name: n, email: e }` | `INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *` |
| `update users set { name: n } where id == id` | `UPDATE users SET name = $1 WHERE id = $2 RETURNING *` |
| `delete users where id == id` | `DELETE FROM users WHERE id = $1 RETURNING *` |

## Grammar Rules (+14)

| Rule | Purpose |
|------|---------|
| `model_def` | Top-level model definition |
| `model_field` | Field name + type + constraints |
| `field_type` | `number \| text \| bool \| timestamp` |
| `field_constraint` | `primary \| unique \| default value` |
| `server_find_expr` | `find model clauses...` |
| `server_insert_expr` | `insert model { fields }` |
| `server_update_expr` | `update model set { fields } where...` |
| `server_delete_expr` | `delete model where...` |
| `query_clause` | Silent dispatcher for where/order/limit |
| `query_where` | `where condition and condition...` |
| `query_condition` | `field op value` |
| `query_cmp_op` | `== \| != \| >= \| <= \| > \| <` |
| `query_order` | `order field asc/desc` |
| `query_limit` | `limit expression` |

## Files Modified

| File | Changes |
|------|---------|
| `crates/naze-parser/src/naze.pest` | +14 rules, update `statement` and `server_expr` |
| `crates/naze-parser/src/ast.rs` | `Node::Model`, `ModelField`, `QueryCondition`, 4 `ServerExpr` variants |
| `crates/naze-parser/src/parse.rs` | `parse_model_def`, query expression parsing, ~6 tests |
| `crates/naze-compiler/src/codegen.rs` | Query-to-SQL compilation, `compile_where` helper |
| `crates/naze-compiler/src/typecheck.rs` | Model collection + warning for undefined models |

---

## M40: Close Browser API Parity Gaps
**Crates:** `naze-runtime` (WASM), `naze-renderer`

Four features had full parser + IR + codegen support but missing or stub runtime implementations. M40 completes them all.

- [x] **Textarea rendering** — `"textarea"` match arm in runtime, hidden `<textarea>` DOM element for multi-line text capture, bind/focus/validation support
- [x] **Browser notifications** — Real Notification API replacing `window.alert()` stub, with permission flow (granted/denied/request)
- [x] **JS interop actions** — Real `window[fn]()` calls via `js_sys::Reflect`, dotted path resolution ("Math.random"), arg conversion, return value binding to state
- [x] **Device API data sources** — Geolocation (one-shot + watch), accelerometer (devicemotion), JS call data sources (source_type 3 and 4)
- [x] Fixed pre-existing bug: missing `guards` field in `resolve_tree()` RenderTree construction
- [x] WASM rebuilt, 406KB (was 356KB, +50KB from new web-sys bindings)
- [x] WASM size budget updated to 420KB

### Files Modified

| File | Changes |
|------|---------|
| `crates/naze-runtime/src/lib.rs` | Textarea match arm, notify impl, JS interop impl, device API init (~420 LOC) |
| `crates/naze-runtime/Cargo.toml` | web-sys features: Notification, Geolocation, Position, Coordinates, DeviceMotionEvent |
| `crates/nazec/tests/build_examples.rs` | WASM size limit 360KB → 420KB |
| `examples/device-geolocation.naze` | New example: one-shot + watch mode geolocation |
| `examples/device-accelerometer.naze` | New example: motion data display |

---

## M41: WASM Binary Size Optimization
**Crates:** `naze-runtime` (WASM), workspace config

Reverse the binary size growth from M40 (406KB) with targeted optimizations: enable wasm-opt, remove unused web-sys features, reduce format!() string bloat, and fix unsafe strip setting.

- [x] **Enable wasm-opt** — changed `wasm-opt = false` to `wasm-opt = ['-Os']` in runtime Cargo.toml; installed binaryen via npm
- [x] **Remove 14 unused web-sys features** — removed HtmlDivElement, HtmlBodyElement, HashChangeEvent, Node, CloseEvent, EventSource, EventSourceInit, FileReader, Blob, FormData, History, PopStateEvent, Url, UrlSearchParams (40 features retained)
- [x] **Fix strip setting** — changed workspace `strip = true` (unsafe for WASM, maps to "symbols") to `strip = "debuginfo"`
- [x] **Reduce format!() string bloat** — added `state_key()` helper, replaced 48 `format!("{}.loading/error/data", name)` patterns
- [x] WASM binary: 374KB (was 406KB, -32KB / -8%)
- [x] WASM size budget updated to 390KB
- [x] 382 workspace tests passing

### Size Profile (twiggy)

| Metric | Before (M40) | After (M41) |
|--------|-------------|-------------|
| Binary size | 415,795B (406KB) | 382,942B (374KB) |
| Total items | 1,959 | 1,335 |
| Largest data segment | 59,830B (14.4%) | 27,621B (7.2%) |
| web-sys features | 52 | 40 |

### Files Modified

| File | Changes |
|------|---------|
| `crates/naze-runtime/Cargo.toml` | Enable wasm-opt, remove 14 unused web-sys features |
| `Cargo.toml` (workspace) | `strip = true` → `strip = "debuginfo"` |
| `crates/naze-runtime/src/lib.rs` | Add `state_key()` helper, replace 48 format!() patterns |
| `crates/nazec/tests/build_examples.rs` | WASM size limit 420KB → 390KB |
| `crates/naze-runtime/pkg/*` | Rebuilt WASM binary |
