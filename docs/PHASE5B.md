# Phase 5B: Declarative Database Queries (M39)

**Goal:** Add Prisma-like declarative database queries so developers can avoid writing raw SQL. Model definitions provide compile-time type safety; query expressions (`find`, `insert`, `update`, `delete`) compile to parameterized SQL at compile time using the existing M38 infrastructure.

**Phase 5 status:** M31-M38 all complete. M39 complete. 382 workspace tests passing. See [PHASE5.md](PHASE5.md).

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
