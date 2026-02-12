use std::collections::HashMap;

use naze_parser::ast::{
    Action, BinOp, DataSource, EventHandler, Expression, FuncParam, MatchArm, MatchPattern, Node,
    PipelineStage, Prop, Span, StorageType, StringPart, TimerKind, Unit, Value,
};

/// Convert an AST Span to the IR span format (file, line, col).
fn ir_span(span: &Span) -> Option<(String, u32, u32)> {
    Some((span.file.clone(), span.line as u32, span.col as u32))
}

use std::cell::RefCell;

// Thread-local storage for function definitions during lowering.
// Maps function name → (params, body expression).
thread_local! {
    static FUNCTIONS: RefCell<HashMap<String, (Vec<FuncParam>, Expression)>> = RefCell::new(HashMap::new());
    static IMPORT_NAMES: RefCell<std::collections::HashSet<String>> = RefCell::new(std::collections::HashSet::new());
    static ENV_VARS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static IN_SERVER_CONTEXT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Set resolved environment variables for compile-time substitution.
/// Call this before `lower()` to make env vars available during codegen.
pub fn set_env_vars(vars: HashMap<String, String>) {
    ENV_VARS.with(|ev| *ev.borrow_mut() = vars);
}

use crate::resolve::{ComponentDef, ResolvedProject};

// Re-export IR types so existing consumers can use `naze_compiler::codegen::*`
pub use naze_ir::{
    ComputedDecl, DataDecl, IrAction, IrBinOp, IrEventHandler, IrExpression, IrPipelineStage,
    PageDef, ParamDecl, RenderNode, RenderTree, RenderValue, StateDecl, StorageDecl, TextPart,
    TimerDecl,
};

/// Lower a resolved project into a flattened RenderTree.
/// All component invocations are inlined with prop substitution.
pub fn lower(project: &ResolvedProject) -> RenderTree {
    // Clear and collect function definitions into thread-local for inlining
    FUNCTIONS.with(|f| f.borrow_mut().clear());
    collect_functions(&project.entry.nodes);

    // Populate WASM import names for qualified call detection
    IMPORT_NAMES.with(|names| {
        let mut names = names.borrow_mut();
        names.clear();
        for imp in &project.imports {
            names.insert(imp.name.clone());
        }
    });

    let by_name: HashMap<&str, &ComponentDef> = project
        .components
        .values()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let mut title = String::new();
    let mut root = Vec::new();
    let mut state = Vec::new();
    let mut data = Vec::new();
    let mut computed = Vec::new();
    let mut storage = Vec::new();
    let mut timers = Vec::new();
    let mut params = Vec::new();
    let mut pages = Vec::new();
    let mut server_functions = Vec::new();
    let mut server_calls = Vec::new();
    let mut prompts = Vec::new();
    let mut guards = Vec::new();
    let mut let_scope: HashMap<String, RenderValue> = HashMap::new();

    // Pre-populate scope with theme tokens from the default (first) theme
    let default_theme = &project.themes[0];
    for (name, color) in &default_theme.colors {
        let key = format!("theme.colors.{}", name);
        let_scope.insert(key, RenderValue::Color(*color));
    }
    for (name, value) in &default_theme.spacing {
        let key = format!("theme.spacing.{}", name);
        let_scope.insert(key, RenderValue::Num(*value, Some("px".to_string())));
    }

    for node in &project.entry.nodes {
        match node {
            Node::App {
                title: t, children, ..
            } => {
                title = t.clone();
                // Collect state, data, computed, and let declarations from inside the app block
                collect_declarations(
                    children,
                    &mut state,
                    &mut data,
                    &mut computed,
                    &mut storage,
                    &mut timers,
                    &mut params,
                    &mut let_scope,
                    &mut server_calls,
                    &mut prompts,
                );
                // Check for page blocks within the app
                let (app_root, app_pages) = lower_nodes_with_pages(children, &by_name, &let_scope);
                root = app_root;
                pages = app_pages;
            }
            // Top-level let/state/data outside app block
            Node::Let { name, value, .. } => {
                let_scope.insert(name.clone(), lower_value(value, &let_scope));
            }
            Node::State {
                name,
                value,
                shared,
                ..
            } => {
                state.push(StateDecl {
                    name: name.clone(),
                    initial: lower_value(value, &let_scope),
                    shared: *shared,
                });
            }
            Node::Data {
                name,
                url,
                source,
                config,
                ..
            } => {
                data.push(DataDecl {
                    name: name.clone(),
                    url: url.clone(),
                    source_type: match source {
                        DataSource::Fetch => 0,
                        DataSource::Stream => 1,
                        DataSource::JsCall => 3,
                        DataSource::Device => 4,
                    },
                    method: config.method.clone().unwrap_or_else(|| "get".to_string()),
                    cache_ms: config.cache_ms.unwrap_or(0),
                    retry_count: config.retry.unwrap_or(0),
                    trigger_mode: if config.trigger.as_deref() == Some("manual") {
                        1
                    } else {
                        0
                    },
                    content_type: config.content_type.clone().unwrap_or_default(),
                    watch: config.watch,
                    headers: config
                        .headers
                        .iter()
                        .map(|(k, v)| (k.clone(), lower_value(v, &let_scope)))
                        .collect(),
                });
            }
            Node::Computed { name, expr, .. } => {
                computed.push(ComputedDecl {
                    name: name.clone(),
                    expr: lower_expression(expr),
                });
            }
            Node::Storage {
                name,
                storage_type,
                key,
                default,
                ..
            } => {
                storage.push(StorageDecl {
                    name: name.clone(),
                    storage_type: match storage_type {
                        StorageType::Local => 0,
                        StorageType::Session => 1,
                    },
                    key: key.clone(),
                    default: lower_value(default, &let_scope),
                });
            }
            Node::Timer {
                name,
                kind,
                duration_ms,
                action,
                ..
            } => {
                timers.push(TimerDecl {
                    name: name.clone(),
                    kind: match kind {
                        TimerKind::After => 0,
                        TimerKind::Every => 1,
                    },
                    duration_ms: *duration_ms,
                    action: lower_action(action),
                });
            }
            Node::ServerFunction {
                name, params, body, ..
            } => {
                IN_SERVER_CONTEXT.with(|c| c.set(true));
                let ir_lets: Vec<(String, naze_ir::IrServerStep)> = body
                    .lets
                    .iter()
                    .map(|(n, se)| {
                        let step = match se {
                            naze_parser::ast::ServerExpr::Fetch(url) => {
                                naze_ir::IrServerStep::Fetch(url.clone())
                            }
                            naze_parser::ast::ServerExpr::Sql { query, params } => {
                                naze_ir::IrServerStep::Sql {
                                    query: query.clone(),
                                    params: params.iter().map(lower_expression).collect(),
                                }
                            }
                            naze_parser::ast::ServerExpr::Expr(e) => {
                                naze_ir::IrServerStep::Expr(lower_expression(e))
                            }
                            naze_parser::ast::ServerExpr::Find {
                                model,
                                conditions,
                                order,
                                limit,
                            } => compile_find_to_sql(model, conditions, order, limit),
                            naze_parser::ast::ServerExpr::Insert { model, fields } => {
                                compile_insert_to_sql(model, fields)
                            }
                            naze_parser::ast::ServerExpr::Update {
                                model,
                                set_fields,
                                conditions,
                            } => compile_update_to_sql(model, set_fields, conditions),
                            naze_parser::ast::ServerExpr::Delete { model, conditions } => {
                                compile_delete_to_sql(model, conditions)
                            }
                        };
                        (n.clone(), step)
                    })
                    .collect();
                let ir_result = lower_expression(&body.result);
                IN_SERVER_CONTEXT.with(|c| c.set(false));
                server_functions.push(naze_ir::ServerFuncDecl {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: naze_ir::IrServerBody {
                        lets: ir_lets,
                        result: ir_result,
                    },
                });
            }
            Node::ServerData {
                name,
                func_name,
                args,
                ..
            } => {
                server_calls.push(naze_ir::ServerCallDecl {
                    name: name.clone(),
                    func_name: func_name.clone(),
                    args: args.iter().map(lower_expression).collect(),
                });
            }
            Node::Prompt {
                name,
                provider,
                props,
                ..
            } => {
                let get_str = |key: &str, default: &str| -> String {
                    props
                        .iter()
                        .find(|(k, _)| k == key)
                        .and_then(|(_, v)| match v {
                            Value::Str(s) => Some(s.clone()),
                            Value::InterpolatedStr(parts) => {
                                // Flatten interpolated string to plain text for now
                                let mut out = String::new();
                                for part in parts {
                                    match part {
                                        StringPart::Literal(s) => out.push_str(s),
                                        StringPart::Interpolation(segs) => {
                                            out.push('{');
                                            out.push_str(&segs.join("."));
                                            out.push('}');
                                        }
                                    }
                                }
                                Some(out)
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| default.to_string())
                };
                let get_num = |key: &str, default: f64| -> f64 {
                    props
                        .iter()
                        .find(|(k, _)| k == key)
                        .and_then(|(_, v)| match v {
                            Value::Num(n, _) => Some(*n),
                            _ => None,
                        })
                        .unwrap_or(default)
                };
                prompts.push(naze_ir::PromptDecl {
                    name: name.clone(),
                    provider: provider.clone(),
                    system: get_str("system", ""),
                    user: get_str("user", ""),
                    model: get_str("model", "gpt-4o"),
                    max_tokens: get_num("max-tokens", 1000.0) as u32,
                    temperature: get_num("temperature", 0.7),
                });
            }
            Node::Guard { name, checks, .. } => {
                let ir_checks: Vec<naze_ir::GuardCheck> = checks
                    .iter()
                    .map(|c| naze_ir::GuardCheck {
                        condition: lower_expression(&c.condition),
                        redirect: c.redirect.clone(),
                    })
                    .collect();
                guards.push(naze_ir::GuardDef {
                    name: name.clone(),
                    checks: ir_checks,
                });
            }
            // Skip use statements, comments, component defs at top level
            _ => {}
        }
    }

    // Build ThemeDef list from resolved themes
    let themes: Vec<naze_ir::ThemeDef> = project
        .themes
        .iter()
        .map(|t| naze_ir::ThemeDef {
            name: t.name.clone(),
            colors: t.colors.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            spacing: t.spacing.iter().map(|(k, v)| (k.clone(), *v)).collect(),
        })
        .collect();

    RenderTree {
        title,
        root,
        state,
        data,
        computed,
        storage,
        timers,
        params,
        pages,
        themes,
        imports: project
            .imports
            .iter()
            .map(|imp| {
                let functions = crate::typecheck::read_wasm_exports(&imp.wasm_path);
                naze_ir::ImportDecl {
                    name: imp.name.clone(),
                    wasm_url: format!("{}.wasm", imp.name),
                    functions: functions.into_iter().collect(),
                }
            })
            .collect(),
        server_functions,
        server_calls,
        prompts,
        guards,
    }
}

/// Walk children to collect state/let/data/computed declarations (does not recurse into elements).
#[allow(clippy::too_many_arguments)]
fn collect_declarations(
    nodes: &[Node],
    state: &mut Vec<StateDecl>,
    data: &mut Vec<DataDecl>,
    computed: &mut Vec<ComputedDecl>,
    storage: &mut Vec<StorageDecl>,
    timers: &mut Vec<TimerDecl>,
    params: &mut Vec<ParamDecl>,
    let_scope: &mut HashMap<String, RenderValue>,
    server_calls: &mut Vec<naze_ir::ServerCallDecl>,
    prompts: &mut Vec<naze_ir::PromptDecl>,
) {
    for node in nodes {
        match node {
            Node::Let { name, value, .. } => {
                let_scope.insert(name.clone(), lower_value(value, let_scope));
            }
            Node::State {
                name,
                value,
                shared,
                ..
            } => {
                state.push(StateDecl {
                    name: name.clone(),
                    initial: lower_value(value, let_scope),
                    shared: *shared,
                });
            }
            Node::Data {
                name,
                url,
                source,
                config,
                ..
            } => {
                data.push(DataDecl {
                    name: name.clone(),
                    url: url.clone(),
                    source_type: match source {
                        DataSource::Fetch => 0,
                        DataSource::Stream => 1,
                        DataSource::JsCall => 3,
                        DataSource::Device => 4,
                    },
                    method: config.method.clone().unwrap_or_else(|| "get".to_string()),
                    cache_ms: config.cache_ms.unwrap_or(0),
                    retry_count: config.retry.unwrap_or(0),
                    trigger_mode: if config.trigger.as_deref() == Some("manual") {
                        1
                    } else {
                        0
                    },
                    content_type: config.content_type.clone().unwrap_or_default(),
                    watch: config.watch,
                    headers: config
                        .headers
                        .iter()
                        .map(|(k, v)| (k.clone(), lower_value(v, let_scope)))
                        .collect(),
                });
            }
            Node::Computed { name, expr, .. } => {
                computed.push(ComputedDecl {
                    name: name.clone(),
                    expr: lower_expression(expr),
                });
            }
            Node::Storage {
                name,
                storage_type,
                key,
                default,
                ..
            } => {
                storage.push(StorageDecl {
                    name: name.clone(),
                    storage_type: match storage_type {
                        StorageType::Local => 0,
                        StorageType::Session => 1,
                    },
                    key: key.clone(),
                    default: lower_value(default, let_scope),
                });
            }
            Node::Timer {
                name,
                kind,
                duration_ms,
                action,
                ..
            } => {
                timers.push(TimerDecl {
                    name: name.clone(),
                    kind: match kind {
                        TimerKind::After => 0,
                        TimerKind::Every => 1,
                    },
                    duration_ms: *duration_ms,
                    action: lower_action(action),
                });
            }
            Node::Param {
                name, ty, default, ..
            } => {
                let param_type = match ty {
                    naze_parser::ast::Type::Text => "text",
                    naze_parser::ast::Type::Number => "number",
                    naze_parser::ast::Type::Bool => "bool",
                    naze_parser::ast::Type::Color => "color",
                }
                .to_string();
                params.push(ParamDecl {
                    name: name.clone(),
                    param_type,
                    default: lower_value(default, let_scope),
                });
            }
            Node::ServerData {
                name,
                func_name,
                args,
                ..
            } => {
                server_calls.push(naze_ir::ServerCallDecl {
                    name: name.clone(),
                    func_name: func_name.clone(),
                    args: args.iter().map(lower_expression).collect(),
                });
            }
            Node::Prompt {
                name,
                provider,
                props,
                ..
            } => {
                let get_str = |key: &str, default: &str| -> String {
                    props
                        .iter()
                        .find(|(k, _)| k == key)
                        .and_then(|(_, v)| match v {
                            Value::Str(s) => Some(s.clone()),
                            Value::InterpolatedStr(parts) => {
                                let mut out = String::new();
                                for part in parts {
                                    match part {
                                        StringPart::Literal(s) => out.push_str(s),
                                        StringPart::Interpolation(segs) => {
                                            out.push('{');
                                            out.push_str(&segs.join("."));
                                            out.push('}');
                                        }
                                    }
                                }
                                Some(out)
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| default.to_string())
                };
                let get_num = |key: &str, default: f64| -> f64 {
                    props
                        .iter()
                        .find(|(k, _)| k == key)
                        .and_then(|(_, v)| match v {
                            Value::Num(n, _) => Some(*n),
                            _ => None,
                        })
                        .unwrap_or(default)
                };
                prompts.push(naze_ir::PromptDecl {
                    name: name.clone(),
                    provider: provider.clone(),
                    system: get_str("system", ""),
                    user: get_str("user", ""),
                    model: get_str("model", "gpt-4o"),
                    max_tokens: get_num("max-tokens", 1000.0) as u32,
                    temperature: get_num("temperature", 0.7),
                });
            }
            _ => {}
        }
    }

    // Also collect derived state for validated inputs
    collect_validation_state(nodes, state);
}

/// Walk tree recursively to find inputs with validate props and create derived state.
fn collect_validation_state(nodes: &[Node], state: &mut Vec<StateDecl>) {
    for node in nodes {
        match node {
            Node::Element {
                name,
                props,
                children,
                ..
            } => {
                // Check if this is an input/textarea with both bind and validate props
                if name == "input" || name == "textarea" {
                    let has_validate = props.iter().any(|p| p.key == "validate");
                    if has_validate {
                        // Find the bind variable
                        if let Some(bind_prop) = props.iter().find(|p| p.key == "bind") {
                            if let Value::Bind(bind_var) = &bind_prop.value {
                                // Create derived state variables for validation
                                let valid_key = format!("{}_valid", bind_var);
                                let error_key = format!("{}_error", bind_var);

                                // Add them if they don't already exist
                                if !state.iter().any(|s| s.name == valid_key) {
                                    state.push(StateDecl {
                                        name: valid_key,
                                        initial: RenderValue::Bool(true), // Initially valid
                                        shared: false,
                                    });
                                }
                                if !state.iter().any(|s| s.name == error_key) {
                                    state.push(StateDecl {
                                        name: error_key,
                                        initial: RenderValue::Str(String::new()), // No error initially
                                        shared: false,
                                    });
                                }
                            }
                        }
                    }
                }
                // Recurse into children
                collect_validation_state(children, state);
            }
            Node::If {
                then_children,
                else_children,
                ..
            } => {
                collect_validation_state(then_children, state);
                collect_validation_state(else_children, state);
            }
            Node::Each { children, .. } => {
                collect_validation_state(children, state);
            }
            Node::Page { children, .. } => {
                collect_validation_state(children, state);
            }
            Node::App { children, .. } => {
                collect_validation_state(children, state);
            }
            Node::Component { children, .. } | Node::Template { children, .. } => {
                collect_validation_state(children, state);
            }
            Node::Slot {
                default_children, ..
            } => {
                collect_validation_state(default_children, state);
            }
            Node::Fill { children, .. } => {
                collect_validation_state(children, state);
            }
            _ => {}
        }
    }
}

// Re-export serialize/deserialize from naze-ir
pub use naze_ir::{deserialize, serialize};

// ─── Query-to-SQL compilation (M39) ─────────────────────────────────────────

fn compile_where(
    conditions: &[naze_parser::ast::QueryCondition],
) -> (String, Vec<naze_ir::IrExpression>) {
    let mut parts = Vec::new();
    let mut params = Vec::new();
    for cond in conditions {
        params.push(lower_expression(&cond.value));
        let sql_op = match cond.op.as_str() {
            "==" => "=",
            "!=" => "!=",
            other => other,
        };
        parts.push(format!("{} {} ${}", cond.field, sql_op, params.len()));
    }
    (parts.join(" AND "), params)
}

fn compile_find_to_sql(
    model: &str,
    conditions: &[naze_parser::ast::QueryCondition],
    order: &Option<(String, bool)>,
    limit: &Option<naze_parser::ast::Expression>,
) -> naze_ir::IrServerStep {
    let (where_sql, mut params) = compile_where(conditions);
    let mut sql = format!("SELECT * FROM {}", model);
    if !where_sql.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_sql));
    }
    if let Some((field, ascending)) = order {
        sql.push_str(&format!(
            " ORDER BY {} {}",
            field,
            if *ascending { "ASC" } else { "DESC" }
        ));
    }
    if let Some(lim) = limit {
        params.push(lower_expression(lim));
        sql.push_str(&format!(" LIMIT ${}", params.len()));
    }
    naze_ir::IrServerStep::Sql { query: sql, params }
}

fn compile_insert_to_sql(
    model: &str,
    fields: &[(String, naze_parser::ast::Value)],
) -> naze_ir::IrServerStep {
    let columns: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
    let placeholders: Vec<String> = (1..=fields.len()).map(|i| format!("${}", i)).collect();
    let params: Vec<naze_ir::IrExpression> =
        fields.iter().map(|(_, v)| lower_value_to_ir(v)).collect();
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
        model,
        columns.join(", "),
        placeholders.join(", ")
    );
    naze_ir::IrServerStep::Sql { query: sql, params }
}

fn compile_update_to_sql(
    model: &str,
    set_fields: &[(String, naze_parser::ast::Value)],
    conditions: &[naze_parser::ast::QueryCondition],
) -> naze_ir::IrServerStep {
    let mut params: Vec<naze_ir::IrExpression> = Vec::new();
    let set_clauses: Vec<String> = set_fields
        .iter()
        .map(|(k, v)| {
            params.push(lower_value_to_ir(v));
            format!("{} = ${}", k, params.len())
        })
        .collect();
    let (where_sql, where_params) = compile_where(conditions);
    // Renumber where params to continue after set params
    let where_offset = params.len();
    params.extend(where_params);
    let renumbered_where: String = if !where_sql.is_empty() {
        let mut result = where_sql;
        // Replace $1, $2, etc. with $N+offset
        for i in (1..=conditions.len()).rev() {
            result = result.replace(&format!("${}", i), &format!("${}", i + where_offset));
        }
        result
    } else {
        String::new()
    };
    let mut sql = format!("UPDATE {} SET {}", model, set_clauses.join(", "));
    if !renumbered_where.is_empty() {
        sql.push_str(&format!(" WHERE {}", renumbered_where));
    }
    sql.push_str(" RETURNING *");
    naze_ir::IrServerStep::Sql { query: sql, params }
}

fn compile_delete_to_sql(
    model: &str,
    conditions: &[naze_parser::ast::QueryCondition],
) -> naze_ir::IrServerStep {
    let (where_sql, params) = compile_where(conditions);
    let mut sql = format!("DELETE FROM {}", model);
    if !where_sql.is_empty() {
        sql.push_str(&format!(" WHERE {}", where_sql));
    }
    sql.push_str(" RETURNING *");
    naze_ir::IrServerStep::Sql { query: sql, params }
}

/// Lower an AST Value to an IrExpression for query field values (no scope needed).
fn lower_value_to_ir(v: &naze_parser::ast::Value) -> naze_ir::IrExpression {
    match v {
        naze_parser::ast::Value::Str(s) => naze_ir::IrExpression::Str(s.clone()),
        naze_parser::ast::Value::Num(n, _) => naze_ir::IrExpression::Num(*n),
        naze_parser::ast::Value::Bool(b) => naze_ir::IrExpression::Bool(*b),
        naze_parser::ast::Value::Color(c) => naze_ir::IrExpression::Num(*c as f64),
        naze_parser::ast::Value::Ref(segments) => {
            naze_ir::IrExpression::StateRef(segments.join("."))
        }
        naze_parser::ast::Value::InterpolatedStr(parts) => {
            // For query params, resolve interpolated strings to state refs if single segment
            if parts.len() == 1 {
                match &parts[0] {
                    naze_parser::ast::StringPart::Interpolation(segs) => {
                        naze_ir::IrExpression::StateRef(segs.join("."))
                    }
                    naze_parser::ast::StringPart::Literal(s) => {
                        naze_ir::IrExpression::Str(s.clone())
                    }
                }
            } else {
                // Flatten to literal string for compound interpolations
                let s: String = parts
                    .iter()
                    .map(|p| match p {
                        naze_parser::ast::StringPart::Literal(s) => s.clone(),
                        naze_parser::ast::StringPart::Interpolation(segs) => {
                            format!("{{{}}}", segs.join("."))
                        }
                    })
                    .collect();
                naze_ir::IrExpression::Str(s)
            }
        }
        _ => naze_ir::IrExpression::Str(String::new()),
    }
}

/// Lower nodes, separating page blocks from regular content.
/// Returns (root nodes, page definitions).
fn lower_nodes_with_pages(
    nodes: &[Node],
    components: &HashMap<&str, &ComponentDef>,
    scope: &HashMap<String, RenderValue>,
) -> (Vec<RenderNode>, Vec<PageDef>) {
    let mut root = Vec::new();
    let mut pages = Vec::new();

    for node in nodes {
        match node {
            Node::Page {
                path,
                params,
                guard,
                children,
                ..
            } => {
                let is_catch_all = path == "/*" || path.ends_with("/*");
                // Extract meta nodes from page children
                let mut meta = Vec::new();
                let mut content_children = Vec::new();
                for child in children {
                    if let Node::Meta { key, value, .. } = child {
                        meta.push((key.clone(), lower_value(value, scope)));
                    } else {
                        content_children.push(child.clone());
                    }
                }
                pages.push(PageDef {
                    path: path.clone(),
                    params: params.clone(),
                    is_catch_all,
                    guard: guard.clone(),
                    meta,
                    root: lower_nodes(&content_children, components, scope),
                });
            }
            _ => {
                // Lower regular nodes
                let lowered = lower_node(node, components, scope);
                root.extend(lowered);
            }
        }
    }

    (root, pages)
}

/// Lower a single node, returning zero or more RenderNodes.
fn lower_node(
    node: &Node,
    components: &HashMap<&str, &ComponentDef>,
    scope: &HashMap<String, RenderValue>,
) -> Vec<RenderNode> {
    match node {
        Node::Element {
            name,
            props,
            children,
            handlers,
            span,
        } => {
            if let Some(comp) = components.get(name.as_str()) {
                inline_component(comp, props, children, handlers, components, scope)
            } else {
                let resolved_props = resolve_props(props, scope);
                let child_nodes = lower_nodes(children, components, scope);
                let mut ir_handlers = lower_handlers(handlers);

                // Auto-generate click handler for checkbox with bind
                if name == "checkbox" {
                    if let Some(RenderValue::Bind(bind_var)) = resolved_props.get("bind") {
                        ir_handlers.push(IrEventHandler {
                            event: "click".to_string(),
                            action: IrAction::Set {
                                target: bind_var.clone(),
                                // Toggle: set var = var == false
                                expr: IrExpression::BinOp {
                                    left: Box::new(IrExpression::StateRef(bind_var.clone())),
                                    op: IrBinOp::Eq,
                                    right: Box::new(IrExpression::Bool(false)),
                                },
                            },
                            modifier_kind: 0,
                            modifier_ms: 0,
                        });
                    }
                }

                // Auto-generate click handler for radio with bind and value
                if name == "radio" {
                    if let Some(RenderValue::Bind(bind_var)) = resolved_props.get("bind") {
                        if let Some(value) = resolved_props.get("value") {
                            let expr = render_value_to_expr(value);
                            ir_handlers.push(IrEventHandler {
                                event: "click".to_string(),
                                action: IrAction::Set {
                                    target: bind_var.clone(),
                                    expr,
                                },
                                modifier_kind: 0,
                                modifier_ms: 0,
                            });
                        }
                    }
                }

                vec![RenderNode {
                    kind: name.clone(),
                    props: resolved_props,
                    children: child_nodes,
                    handlers: ir_handlers,
                    condition: None,
                    else_children: None,
                    each_binding: None,
                    span: ir_span(span),
                }]
            }
        }
        Node::Link {
            text,
            to,
            children,
            span,
            ..
        } => {
            let mut props = HashMap::new();
            props.insert("__text".to_string(), lower_value(text, scope));
            props.insert("to".to_string(), RenderValue::Str(to.clone()));
            let child_nodes = lower_nodes(children, components, scope);
            // Link is a special element that triggers navigation
            let navigate_handler = IrEventHandler {
                event: "click".to_string(),
                action: IrAction::Navigate { path: to.clone() },
                modifier_kind: 0,
                modifier_ms: 0,
            };
            vec![RenderNode {
                kind: "link".to_string(),
                props,
                children: child_nodes,
                handlers: vec![navigate_handler],
                condition: None,
                else_children: None,
                each_binding: None,
                span: ir_span(span),
            }]
        }
        Node::If {
            condition,
            then_children,
            else_children,
            span,
        } => {
            let then_nodes = lower_nodes(then_children, components, scope);
            let else_nodes = if else_children.is_empty() {
                None
            } else {
                Some(lower_nodes(else_children, components, scope))
            };
            vec![RenderNode {
                kind: "__if".to_string(),
                props: HashMap::new(),
                children: then_nodes,
                handlers: vec![],
                condition: Some(lower_expression(condition)),
                else_children: else_nodes,
                each_binding: None,
                span: ir_span(span),
            }]
        }
        Node::Each {
            variable,
            iterable,
            children,
            span,
        } => {
            let child_nodes = lower_nodes(children, components, scope);
            vec![RenderNode {
                kind: "__each".to_string(),
                props: HashMap::new(),
                children: child_nodes,
                handlers: vec![],
                condition: None,
                else_children: None,
                each_binding: Some((variable.clone(), lower_expression(iterable))),
                span: ir_span(span),
            }]
        }
        Node::Boundary {
            children,
            catch_children,
            span,
        } => {
            // Collect data declaration names from boundary children
            let data_names: Vec<String> = children
                .iter()
                .filter_map(|n| match n {
                    Node::Data { name, .. } | Node::ServerData { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();

            let then_nodes = lower_nodes(children, components, scope);
            let else_nodes = lower_nodes(catch_children, components, scope);

            if data_names.is_empty() {
                // No data declarations — boundary just wraps content directly
                then_nodes
            } else {
                // Build condition: !(d1.error || d2.error || ...)
                // Each data.error is a non-empty string when the fetch failed
                // We check: error == "" (empty string means no error)
                let mut error_expr = IrExpression::BinOp {
                    left: Box::new(IrExpression::StateRef(format!("{}.error", data_names[0]))),
                    op: IrBinOp::Eq,
                    right: Box::new(IrExpression::Str(String::new())),
                };
                for name in &data_names[1..] {
                    let this_ok = IrExpression::BinOp {
                        left: Box::new(IrExpression::StateRef(format!("{name}.error"))),
                        op: IrBinOp::Eq,
                        right: Box::new(IrExpression::Str(String::new())),
                    };
                    error_expr = IrExpression::BinOp {
                        left: Box::new(error_expr),
                        op: IrBinOp::And,
                        right: Box::new(this_ok),
                    };
                }
                let condition = error_expr;

                vec![RenderNode {
                    kind: "__if".to_string(),
                    props: HashMap::new(),
                    children: then_nodes,
                    handlers: vec![],
                    condition: Some(condition),
                    else_children: Some(else_nodes),
                    each_binding: None,
                    span: ir_span(span),
                }]
            }
        }
        Node::Page { .. } => {
            // Pages should be handled separately in lower_nodes_with_pages
            vec![]
        }
        _ => {
            // Skip non-renderable nodes
            vec![]
        }
    }
}

/// Lower a list of AST nodes into RenderNodes, inlining components.
fn lower_nodes(
    nodes: &[Node],
    components: &HashMap<&str, &ComponentDef>,
    scope: &HashMap<String, RenderValue>,
) -> Vec<RenderNode> {
    let mut out = Vec::new();
    for node in nodes {
        match node {
            Node::Element {
                name,
                props,
                children,
                handlers,
                span,
            } => {
                if let Some(comp) = components.get(name.as_str()) {
                    // Component invocation: build substitution scope and inline body
                    out.extend(inline_component(
                        comp, props, children, handlers, components, scope,
                    ));
                } else {
                    // Built-in element
                    let resolved_props = resolve_props(props, scope);
                    let child_nodes = lower_nodes(children, components, scope);
                    let mut ir_handlers = lower_handlers(handlers);

                    // Auto-generate click handler for checkbox with bind
                    if name == "checkbox" {
                        if let Some(RenderValue::Bind(bind_var)) = resolved_props.get("bind") {
                            ir_handlers.push(IrEventHandler {
                                event: "click".to_string(),
                                action: IrAction::Set {
                                    target: bind_var.clone(),
                                    expr: IrExpression::BinOp {
                                        left: Box::new(IrExpression::StateRef(bind_var.clone())),
                                        op: IrBinOp::Eq,
                                        right: Box::new(IrExpression::Bool(false)),
                                    },
                                },
                                modifier_kind: 0,
                                modifier_ms: 0,
                            });
                        }
                    }

                    // Auto-generate click handler for radio with bind and value
                    if name == "radio" {
                        if let Some(RenderValue::Bind(bind_var)) = resolved_props.get("bind") {
                            if let Some(value) = resolved_props.get("value") {
                                let expr = render_value_to_expr(value);
                                ir_handlers.push(IrEventHandler {
                                    event: "click".to_string(),
                                    action: IrAction::Set {
                                        target: bind_var.clone(),
                                        expr,
                                    },
                                    modifier_kind: 0,
                                    modifier_ms: 0,
                                });
                            }
                        }
                    }

                    out.push(RenderNode {
                        kind: name.clone(),
                        props: resolved_props,
                        children: child_nodes,
                        handlers: ir_handlers,
                        condition: None,
                        else_children: None,
                        each_binding: None,
                        span: ir_span(span),
                    });
                }
            }
            Node::Link {
                text,
                to,
                children,
                span,
                ..
            } => {
                let mut props = HashMap::new();
                props.insert("__text".to_string(), lower_value(text, scope));
                props.insert("to".to_string(), RenderValue::Str(to.clone()));
                let child_nodes = lower_nodes(children, components, scope);
                // Link is a special element that triggers navigation
                let navigate_handler = IrEventHandler {
                    event: "click".to_string(),
                    action: IrAction::Navigate { path: to.clone() },
                    modifier_kind: 0,
                    modifier_ms: 0,
                };
                out.push(RenderNode {
                    kind: "link".to_string(),
                    props,
                    children: child_nodes,
                    handlers: vec![navigate_handler],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                    span: ir_span(span),
                });
            }
            Node::If {
                condition,
                then_children,
                else_children,
                span,
            } => {
                let then_nodes = lower_nodes(then_children, components, scope);
                let else_nodes = if else_children.is_empty() {
                    None
                } else {
                    Some(lower_nodes(else_children, components, scope))
                };
                out.push(RenderNode {
                    kind: "__if".to_string(),
                    props: HashMap::new(),
                    children: then_nodes,
                    handlers: vec![],
                    condition: Some(lower_expression(condition)),
                    else_children: else_nodes,
                    each_binding: None,
                    span: ir_span(span),
                });
            }
            Node::Each {
                variable,
                iterable,
                children,
                span,
            } => {
                let child_nodes = lower_nodes(children, components, scope);
                out.push(RenderNode {
                    kind: "__each".to_string(),
                    props: HashMap::new(),
                    children: child_nodes,
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: Some((variable.clone(), lower_expression(iterable))),
                    span: ir_span(span),
                });
            }
            Node::Page { children, .. } => {
                // Page blocks inside non-app contexts: just lower their children
                out.extend(lower_nodes(children, components, scope));
            }
            Node::Boundary { .. } => {
                // Delegate to lower_node which desugars boundary → __if
                out.extend(lower_node(node, components, scope));
            }
            Node::Slot { .. } | Node::Fill { .. } => {
                // Slots/fills outside component inlining context are no-ops
            }
            Node::Match { subject, arms, .. } => {
                out.extend(desugar_match(subject, arms, components, scope));
            }
            Node::Comment(_)
            | Node::UseStmt { .. }
            | Node::Import { .. }
            | Node::Component { .. }
            | Node::Template { .. }
            | Node::Let { .. }
            | Node::State { .. }
            | Node::Data { .. }
            | Node::Computed { .. }
            | Node::Storage { .. }
            | Node::Timer { .. }
            | Node::Param { .. }
            | Node::Function { .. }
            | Node::ServerFunction { .. }
            | Node::ServerData { .. }
            | Node::Theme { .. }
            | Node::Prompt { .. }
            | Node::Meta { .. }
            | Node::Guard { .. }
            | Node::Model { .. } => {
                // Skip non-renderable nodes (declarations processed separately)
            }
            Node::App { children, .. } => {
                // Nested apps shouldn't happen, but handle gracefully
                out.extend(lower_nodes(children, components, scope));
            }
        }
    }
    out
}

/// Inline a component invocation: substitute call-site props into the component body.
/// Call-site children are distributed to slots declared in the component.
fn inline_component(
    comp: &ComponentDef,
    call_props: &[Prop],
    call_children: &[Node],
    call_handlers: &[EventHandler],
    components: &HashMap<&str, &ComponentDef>,
    parent_scope: &HashMap<String, RenderValue>,
) -> Vec<RenderNode> {
    // Build emit map: custom event name -> call-site action
    let emit_map: HashMap<String, &Action> = call_handlers
        .iter()
        .map(|h| (h.event.clone(), &h.action))
        .collect();

    // Build component scope: start with parent, overlay defaults, then call-site props
    let mut comp_scope = parent_scope.clone();

    for param in &comp.params {
        if let Some(default) = &param.default {
            comp_scope.insert(param.name.clone(), lower_value(default, parent_scope));
        }
    }

    for prop in call_props {
        if prop.key == "__text" {
            continue; // components don't have text content
        }
        comp_scope.insert(prop.key.clone(), lower_value(&prop.value, parent_scope));
    }

    // Partition call-site children into named fills and default-slot content
    let mut fills: HashMap<String, Vec<&Node>> = HashMap::new();
    let mut default_nodes: Vec<&Node> = Vec::new();

    for child in call_children {
        match child {
            Node::Fill { name, children, .. } => {
                fills
                    .entry(name.clone())
                    .or_default()
                    .extend(children.iter());
            }
            _ => {
                default_nodes.push(child);
            }
        }
    }

    // Lower the component body with slot substitution
    lower_nodes_with_slots(
        &comp.children,
        components,
        &comp_scope,
        parent_scope,
        &default_nodes,
        &fills,
        &emit_map,
    )
}

/// Lower nodes from a component body, substituting slot markers with caller content.
/// `comp_scope` is used for the component's own body, `caller_scope` for fill content.
fn lower_nodes_with_slots(
    nodes: &[Node],
    components: &HashMap<&str, &ComponentDef>,
    comp_scope: &HashMap<String, RenderValue>,
    caller_scope: &HashMap<String, RenderValue>,
    default_slot_nodes: &[&Node],
    fills: &HashMap<String, Vec<&Node>>,
    emit_map: &HashMap<String, &Action>,
) -> Vec<RenderNode> {
    let mut out = Vec::new();
    for node in nodes {
        match node {
            Node::Slot {
                name,
                default_children,
                ..
            } => {
                match name {
                    None => {
                        // Default slot: substitute with caller's non-fill children
                        if !default_slot_nodes.is_empty() {
                            for child in default_slot_nodes {
                                out.extend(lower_nodes(
                                    std::slice::from_ref(*child),
                                    components,
                                    caller_scope,
                                ));
                            }
                        } else if !default_children.is_empty() {
                            // Fallback content in component's scope
                            out.extend(lower_nodes_with_slots(
                                default_children,
                                components,
                                comp_scope,
                                caller_scope,
                                &[],
                                &HashMap::new(),
                                emit_map,
                            ));
                        }
                    }
                    Some(slot_name) => {
                        if let Some(fill_nodes) = fills.get(slot_name) {
                            for child in fill_nodes {
                                out.extend(lower_nodes(
                                    std::slice::from_ref(*child),
                                    components,
                                    caller_scope,
                                ));
                            }
                        } else if !default_children.is_empty() {
                            out.extend(lower_nodes_with_slots(
                                default_children,
                                components,
                                comp_scope,
                                caller_scope,
                                &[],
                                &HashMap::new(),
                                emit_map,
                            ));
                        }
                    }
                }
            }
            Node::Element {
                name,
                props,
                children,
                handlers,
                span,
            } => {
                if let Some(comp) = components.get(name.as_str()) {
                    out.extend(inline_component(
                        comp, props, children, handlers, components, comp_scope,
                    ));
                } else {
                    let resolved_props = resolve_props(props, comp_scope);
                    let child_nodes = lower_nodes_with_slots(
                        children,
                        components,
                        comp_scope,
                        caller_scope,
                        default_slot_nodes,
                        fills,
                        emit_map,
                    );
                    // Resolve emit actions: replace with parent's action from emit_map
                    let resolved_handlers: Vec<EventHandler> = handlers
                        .iter()
                        .filter_map(|h| match &h.action {
                            Action::Emit { event_name, .. } => emit_map
                                .get(event_name.as_str())
                                .map(|parent_action| EventHandler {
                                    event: h.event.clone(),
                                    action: (*parent_action).clone(),
                                    modifier: h.modifier.clone(),
                                    span: h.span.clone(),
                                }),
                            _ => Some(h.clone()),
                        })
                        .collect();
                    let mut ir_handlers = lower_handlers(&resolved_handlers);

                    // Auto-generate click handler for checkbox with bind
                    if name == "checkbox" {
                        if let Some(RenderValue::Bind(bind_var)) = resolved_props.get("bind") {
                            ir_handlers.push(IrEventHandler {
                                event: "click".to_string(),
                                action: IrAction::Set {
                                    target: bind_var.clone(),
                                    expr: IrExpression::BinOp {
                                        left: Box::new(IrExpression::StateRef(bind_var.clone())),
                                        op: IrBinOp::Eq,
                                        right: Box::new(IrExpression::Bool(false)),
                                    },
                                },
                                modifier_kind: 0,
                                modifier_ms: 0,
                            });
                        }
                    }

                    // Auto-generate click handler for radio with bind and value
                    if name == "radio" {
                        if let Some(RenderValue::Bind(bind_var)) = resolved_props.get("bind") {
                            if let Some(value) = resolved_props.get("value") {
                                let expr = render_value_to_expr(value);
                                ir_handlers.push(IrEventHandler {
                                    event: "click".to_string(),
                                    action: IrAction::Set {
                                        target: bind_var.clone(),
                                        expr,
                                    },
                                    modifier_kind: 0,
                                    modifier_ms: 0,
                                });
                            }
                        }
                    }

                    out.push(RenderNode {
                        kind: name.clone(),
                        props: resolved_props,
                        children: child_nodes,
                        handlers: ir_handlers,
                        condition: None,
                        else_children: None,
                        each_binding: None,
                        span: ir_span(span),
                    });
                }
            }
            Node::If {
                condition,
                then_children,
                else_children,
                span,
            } => {
                let then_nodes = lower_nodes_with_slots(
                    then_children,
                    components,
                    comp_scope,
                    caller_scope,
                    default_slot_nodes,
                    fills,
                    emit_map,
                );
                let else_nodes = if else_children.is_empty() {
                    None
                } else {
                    Some(lower_nodes_with_slots(
                        else_children,
                        components,
                        comp_scope,
                        caller_scope,
                        default_slot_nodes,
                        fills,
                        emit_map,
                    ))
                };
                out.push(RenderNode {
                    kind: "__if".to_string(),
                    props: HashMap::new(),
                    children: then_nodes,
                    handlers: vec![],
                    condition: Some(lower_expression(condition)),
                    else_children: else_nodes,
                    each_binding: None,
                    span: ir_span(span),
                });
            }
            Node::Each {
                variable,
                iterable,
                children,
                span,
            } => {
                let child_nodes = lower_nodes_with_slots(
                    children,
                    components,
                    comp_scope,
                    caller_scope,
                    default_slot_nodes,
                    fills,
                    emit_map,
                );
                out.push(RenderNode {
                    kind: "__each".to_string(),
                    props: HashMap::new(),
                    children: child_nodes,
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: Some((variable.clone(), lower_expression(iterable))),
                    span: ir_span(span),
                });
            }
            Node::Link {
                text,
                to,
                children,
                span,
                ..
            } => {
                let mut props = HashMap::new();
                props.insert("__text".to_string(), lower_value(text, comp_scope));
                props.insert("to".to_string(), RenderValue::Str(to.clone()));
                let child_nodes = lower_nodes_with_slots(
                    children,
                    components,
                    comp_scope,
                    caller_scope,
                    default_slot_nodes,
                    fills,
                    emit_map,
                );
                let navigate_handler = IrEventHandler {
                    event: "click".to_string(),
                    action: IrAction::Navigate { path: to.clone() },
                    modifier_kind: 0,
                    modifier_ms: 0,
                };
                out.push(RenderNode {
                    kind: "link".to_string(),
                    props,
                    children: child_nodes,
                    handlers: vec![navigate_handler],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                    span: ir_span(span),
                });
            }
            Node::Page { children, .. } => {
                // Page blocks inside components: just lower their children
                out.extend(lower_nodes_with_slots(
                    children,
                    components,
                    comp_scope,
                    caller_scope,
                    default_slot_nodes,
                    fills,
                    emit_map,
                ));
            }
            Node::Fill { .. } => {
                // Fill nodes inside a component body are meaningless — skip
            }
            _ => {
                // Comments, Let, State, UseStmt, Component, App — skip
            }
        }
    }
    out
}

/// Resolve props on a built-in element, substituting Ref values from scope.
fn resolve_props(
    props: &[Prop],
    scope: &HashMap<String, RenderValue>,
) -> HashMap<String, RenderValue> {
    let mut out = HashMap::new();
    for prop in props {
        out.insert(prop.key.clone(), lower_value(&prop.value, scope));
    }
    out
}

/// Convert an AST Value to a RenderValue, resolving Ref lookups from scope.
fn lower_value(value: &Value, scope: &HashMap<String, RenderValue>) -> RenderValue {
    match value {
        Value::Str(s) => RenderValue::Str(s.clone()),
        Value::InterpolatedStr(parts) => {
            let text_parts: Vec<TextPart> = parts
                .iter()
                .map(|p| match p {
                    StringPart::Literal(s) => TextPart::Literal(s.clone()),
                    StringPart::Interpolation(segments) => {
                        // Handle env var interpolation: {env.NAME}
                        if segments.len() == 2 && segments[0] == "env" {
                            let env_name = &segments[1];
                            let val =
                                ENV_VARS.with(|ev| ev.borrow().get(env_name.as_str()).cloned());
                            return TextPart::Literal(val.unwrap_or_default());
                        }

                        let key = segments.join(".");
                        // Inline let-scope values at compile time
                        if let Some(val) = scope.get(&key) {
                            match val {
                                RenderValue::Str(s) => TextPart::Literal(s.clone()),
                                RenderValue::Num(n, _) => {
                                    if n.fract() == 0.0 {
                                        TextPart::Literal(format!("{}", *n as i64))
                                    } else {
                                        TextPart::Literal(format!("{}", n))
                                    }
                                }
                                RenderValue::Bool(b) => {
                                    TextPart::Literal(if *b { "true" } else { "false" }.to_string())
                                }
                                _ => TextPart::StateRef(key),
                            }
                        } else {
                            TextPart::StateRef(key)
                        }
                    }
                })
                .collect();
            // If all parts are now literals, collapse to a plain string
            if text_parts.iter().all(|p| matches!(p, TextPart::Literal(_))) {
                let s: String = text_parts
                    .into_iter()
                    .map(|p| match p {
                        TextPart::Literal(s) => s,
                        _ => unreachable!(),
                    })
                    .collect();
                RenderValue::Str(s)
            } else {
                RenderValue::InterpolatedStr(text_parts)
            }
        }
        Value::Num(n, unit) => RenderValue::Num(*n, unit.as_ref().map(unit_str)),
        Value::Color(c) => RenderValue::Color(*c),
        Value::Bool(b) => RenderValue::Bool(*b),
        Value::List(items) => {
            RenderValue::List(items.iter().map(|v| lower_value(v, scope)).collect())
        }
        Value::Ref(parts) => {
            // Handle env var references: env.NAME
            if parts.len() == 2 && parts[0] == "env" {
                let env_name = &parts[1];
                let val = ENV_VARS.with(|ev| ev.borrow().get(env_name.as_str()).cloned());
                return RenderValue::Str(val.unwrap_or_default());
            }

            // Try joining all segments for multi-segment refs like theme.colors.primary
            let full_key = parts.join(".");
            if let Some(val) = scope.get(&full_key) {
                return val.clone();
            }

            // Single-segment fallback
            if parts.len() == 1 {
                if let Some(val) = scope.get(&parts[0]) {
                    val.clone()
                } else {
                    // Unresolved ref — produce a placeholder string
                    RenderValue::Str(format!("<unresolved:{}>", parts[0]))
                }
            } else {
                // Multi-segment ref not found in scope
                RenderValue::Str(format!("<unresolved:{}>", full_key))
            }
        }
        Value::Bind(name) => RenderValue::Bind(name.clone()),
        Value::Object(entries) => RenderValue::Object(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), lower_value(v, scope)))
                .collect(),
        ),
    }
}

/// Convert a RenderValue to an IrExpression for use in auto-generated handlers.
fn render_value_to_expr(value: &RenderValue) -> IrExpression {
    match value {
        RenderValue::Str(s) => IrExpression::Str(s.clone()),
        RenderValue::Num(n, _) => IrExpression::Num(*n),
        RenderValue::Bool(b) => IrExpression::Bool(*b),
        // For other types, convert to string representation
        RenderValue::Color(c) => IrExpression::Str(format!("#{:06x}", c)),
        RenderValue::Bind(name) => IrExpression::StateRef(name.clone()),
        _ => IrExpression::Str(String::new()),
    }
}

fn lower_handlers(handlers: &[EventHandler]) -> Vec<IrEventHandler> {
    handlers.iter().map(lower_handler).collect()
}

fn lower_handler(h: &EventHandler) -> IrEventHandler {
    let (modifier_kind, modifier_ms) = match &h.modifier {
        Some(m) => {
            let kind = match m.kind {
                naze_parser::ast::ModifierKind::Debounce => 1,
                naze_parser::ast::ModifierKind::Throttle => 2,
            };
            (kind, m.duration_ms)
        }
        None => (0, 0),
    };
    IrEventHandler {
        event: h.event.clone(),
        action: lower_action(&h.action),
        modifier_kind,
        modifier_ms,
    }
}

fn lower_action(a: &Action) -> IrAction {
    match a {
        Action::Set { target, expr, .. } => IrAction::Set {
            target: target.clone(),
            expr: lower_expression(expr),
        },
        Action::Navigate { path, .. } => IrAction::Navigate { path: path.clone() },
        Action::ScrollTo { element_id, .. } => IrAction::ScrollTo {
            element_id: element_id.clone(),
        },
        Action::Log { expr, .. } => IrAction::Log {
            expr: lower_expression(expr),
        },
        Action::Trigger { data_name, .. } => IrAction::Trigger {
            data_name: data_name.clone(),
        },
        Action::Copy { expr, .. } => IrAction::Copy {
            expr: lower_expression(expr),
        },
        Action::Send {
            stream_name, expr, ..
        } => IrAction::Send {
            stream_name: stream_name.clone(),
            expr: lower_expression(expr),
        },
        Action::JsCall {
            function_name,
            args,
            target,
            ..
        } => IrAction::JsCall {
            function_name: function_name.clone(),
            args: args.iter().map(lower_expression).collect(),
            target: target.clone(),
        },
        Action::Notify {
            title, body, icon, ..
        } => IrAction::Notify {
            title: title.clone(),
            body: body.clone().unwrap_or_default(),
            icon: icon.clone().unwrap_or_default(),
        },
        Action::Emit { event_name, .. } => {
            // Emit should have been resolved during component inlining.
            // If it reaches here, the emit has no matching parent handler.
            IrAction::Log {
                expr: IrExpression::Str(format!("unresolved emit: {}", event_name)),
            }
        }
        Action::SetTheme { theme_name, .. } => IrAction::SetTheme {
            name: theme_name.clone(),
        },
    }
}

/// Recursively collect function definitions from AST nodes into thread-local storage.
fn collect_functions(nodes: &[Node]) {
    for node in nodes {
        match node {
            Node::Function {
                name, params, body, ..
            } => {
                FUNCTIONS.with(|f| {
                    f.borrow_mut()
                        .insert(name.clone(), (params.clone(), body.clone()));
                });
            }
            Node::App { children, .. } | Node::Page { children, .. } => {
                collect_functions(children);
            }
            _ => {}
        }
    }
}

/// Substitute parameter references in an expression with argument expressions (AST-level inlining).
fn substitute_ast_expr(expr: &Expression, subs: &HashMap<&str, &Expression>) -> Expression {
    match expr {
        Expression::StateRef(name) => {
            if let Some(replacement) = subs.get(name.as_str()) {
                (*replacement).clone()
            } else {
                expr.clone()
            }
        }
        Expression::BinOp { left, op, right } => Expression::BinOp {
            left: Box::new(substitute_ast_expr(left, subs)),
            op: *op,
            right: Box::new(substitute_ast_expr(right, subs)),
        },
        Expression::Pipeline { source, stages } => Expression::Pipeline {
            source: Box::new(substitute_ast_expr(source, subs)),
            stages: stages
                .iter()
                .map(|s| PipelineStage {
                    function: s.function.clone(),
                    argument: s.argument.as_ref().map(|a| substitute_ast_expr(a, subs)),
                    argument2: s.argument2.as_ref().map(|a| substitute_ast_expr(a, subs)),
                })
                .collect(),
        },
        Expression::FunctionCall { name, args } => Expression::FunctionCall {
            name: name.clone(),
            args: args.iter().map(|a| substitute_ast_expr(a, subs)).collect(),
        },
        Expression::Literal(_) => expr.clone(),
    }
}

fn lower_expression(e: &Expression) -> IrExpression {
    match e {
        Expression::Literal(Value::Num(n, _)) => IrExpression::Num(*n),
        Expression::Literal(Value::Str(s)) => IrExpression::Str(s.clone()),
        Expression::Literal(Value::Bool(b)) => IrExpression::Bool(*b),
        Expression::Literal(_) => IrExpression::Str(String::new()),
        Expression::StateRef(name) => {
            // Handle env var references: env.NAME
            if let Some(env_name) = name.strip_prefix("env.") {
                let in_server = IN_SERVER_CONTEXT.with(|c| c.get());
                if in_server {
                    // Server-side: resolve at runtime
                    return IrExpression::EnvRef(env_name.to_string());
                }
                // Client-side: inline compile-time value
                let val = ENV_VARS.with(|ev| ev.borrow().get(env_name).cloned());
                return IrExpression::Str(val.unwrap_or_default());
            }
            IrExpression::StateRef(name.clone())
        }
        Expression::BinOp { left, op, right } => IrExpression::BinOp {
            left: Box::new(lower_expression(left)),
            op: lower_binop(*op),
            right: Box::new(lower_expression(right)),
        },
        Expression::Pipeline { source, stages } => IrExpression::Pipeline {
            source: Box::new(lower_expression(source)),
            stages: stages
                .iter()
                .map(|s| {
                    use naze_parser::ast::PipelineFn;
                    IrPipelineStage {
                        function: match s.function {
                            PipelineFn::Filter => 0,
                            PipelineFn::Map => 1,
                            PipelineFn::SortBy => 2,
                            PipelineFn::Take => 3,
                            PipelineFn::Sum => 4,
                            PipelineFn::Count => 5,
                            PipelineFn::Reduce => 6,
                            PipelineFn::GroupBy => 7,
                            PipelineFn::Flatten => 8,
                            PipelineFn::Distinct => 9,
                        },
                        argument: s.argument.as_ref().map(lower_expression),
                        argument2: s.argument2.as_ref().map(lower_expression),
                    }
                })
                .collect(),
        },
        Expression::FunctionCall { name, args } => {
            // Check for qualified WASM module call: "module.function(args)"
            if let Some(dot) = name.find('.') {
                let module = &name[..dot];
                let function = &name[dot + 1..];
                let is_import = IMPORT_NAMES.with(|names| names.borrow().contains(module));
                if is_import {
                    let ir_args = args.iter().map(lower_expression).collect();
                    return IrExpression::WasmCall {
                        module: module.to_string(),
                        function: function.to_string(),
                        args: ir_args,
                    };
                }
            }
            // Fall through to user-defined function inlining
            FUNCTIONS.with(|f| {
                let funcs = f.borrow();
                if let Some((params, body)) = funcs.get(name.as_str()) {
                    let subs: HashMap<&str, &Expression> = params
                        .iter()
                        .zip(args.iter())
                        .map(|(p, a)| (p.name.as_str(), a))
                        .collect();
                    let inlined = substitute_ast_expr(body, &subs);
                    lower_expression(&inlined)
                } else {
                    IrExpression::Str(String::new())
                }
            })
        }
    }
}

/// Desugar a match statement into nested if/else RenderNodes.
fn desugar_match(
    subject: &Expression,
    arms: &[MatchArm],
    components: &HashMap<&str, &ComponentDef>,
    scope: &HashMap<String, RenderValue>,
) -> Vec<RenderNode> {
    let ir_subject = lower_expression(subject);

    // Separate wildcard arm from pattern arms
    let mut pattern_arms: Vec<&MatchArm> = Vec::new();
    let mut wildcard_arm: Option<&MatchArm> = None;
    for arm in arms {
        match &arm.pattern {
            MatchPattern::Wildcard => wildcard_arm = Some(arm),
            _ => pattern_arms.push(arm),
        }
    }

    let else_children = wildcard_arm
        .map(|arm| lower_nodes(&arm.children, components, scope))
        .filter(|nodes| !nodes.is_empty());

    // Build nested if/else chain from last pattern arm to first
    let mut current_else: Option<Vec<RenderNode>> = else_children;

    for arm in pattern_arms.iter().rev() {
        let condition = match &arm.pattern {
            MatchPattern::StringLit(s) => IrExpression::BinOp {
                left: Box::new(ir_subject.clone()),
                op: IrBinOp::Eq,
                right: Box::new(IrExpression::Str(s.clone())),
            },
            MatchPattern::NumberLit(n) => IrExpression::BinOp {
                left: Box::new(ir_subject.clone()),
                op: IrBinOp::Eq,
                right: Box::new(IrExpression::Num(*n)),
            },
            MatchPattern::BoolLit(b) => IrExpression::BinOp {
                left: Box::new(ir_subject.clone()),
                op: IrBinOp::Eq,
                right: Box::new(IrExpression::Bool(*b)),
            },
            MatchPattern::Ident(name) => IrExpression::BinOp {
                left: Box::new(ir_subject.clone()),
                op: IrBinOp::Eq,
                right: Box::new(IrExpression::StateRef(name.clone())),
            },
            MatchPattern::Wildcard => unreachable!(),
        };

        let then_children = lower_nodes(&arm.children, components, scope);

        let if_node = RenderNode {
            kind: "__if".to_string(),
            props: HashMap::new(),
            children: then_children,
            handlers: vec![],
            condition: Some(condition),
            else_children: current_else.take(),
            each_binding: None,
            span: None, // Synthesized from match desugaring
        };

        current_else = Some(vec![if_node]);
    }

    current_else.unwrap_or_default()
}

fn lower_binop(op: BinOp) -> IrBinOp {
    match op {
        BinOp::Add => IrBinOp::Add,
        BinOp::Sub => IrBinOp::Sub,
        BinOp::Mul => IrBinOp::Mul,
        BinOp::Div => IrBinOp::Div,
        BinOp::Eq => IrBinOp::Eq,
        BinOp::Neq => IrBinOp::Neq,
        BinOp::Gt => IrBinOp::Gt,
        BinOp::Lt => IrBinOp::Lt,
        BinOp::Gte => IrBinOp::Gte,
        BinOp::Lte => IrBinOp::Lte,
        BinOp::And => IrBinOp::And,
        BinOp::Or => IrBinOp::Or,
    }
}

fn unit_str(unit: &Unit) -> String {
    match unit {
        Unit::Px => "px".to_string(),
        Unit::Percent => "%".to_string(),
        Unit::Em => "em".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::resolve;
    use std::fs;

    fn setup_and_lower(files: &[(&str, &str)]) -> RenderTree {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            let path = dir.path().join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }
        let project = resolve(dir.path(), "app.naze", &[]);
        assert!(
            project.errors.is_empty(),
            "resolve errors: {:?}",
            project.errors
        );
        lower(&project)
    }

    #[test]
    fn lower_simple_app() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"app "Hello" {
  text "world"
}"#,
        )]);
        assert_eq!(tree.title, "Hello");
        assert_eq!(tree.root.len(), 1);
        assert_eq!(tree.root[0].kind, "text");
        assert_eq!(
            tree.root[0].props.get("__text"),
            Some(&RenderValue::Str("world".to_string()))
        );
    }

    #[test]
    fn lower_nested_layout() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"app "Test" {
  column padding: 20px {
    row gap: 8px {
      rect width: 50px, height: 50px, color: #ff0000
    }
  }
}"#,
        )]);
        assert_eq!(tree.root.len(), 1);
        let col = &tree.root[0];
        assert_eq!(col.kind, "column");
        assert_eq!(
            col.props.get("padding"),
            Some(&RenderValue::Num(20.0, Some("px".to_string())))
        );
        assert_eq!(col.children.len(), 1);
        let row = &col.children[0];
        assert_eq!(row.kind, "row");
        assert_eq!(row.children.len(), 1);
        let rect = &row.children[0];
        assert_eq!(rect.kind, "rect");
        assert_eq!(rect.props.get("color"), Some(&RenderValue::Color(0xff0000)));
    }

    #[test]
    fn lower_inlines_component() {
        let tree = setup_and_lower(&[
            (
                "components/box.naze",
                "component box(color: color) {\n  rect width: 80px, height: 80px, color: color\n}\n",
            ),
            (
                "app.naze",
                "use components/box\n\napp \"Test\" {\n  box color: #ff0000\n}\n",
            ),
        ]);
        // Component should be inlined — we see a rect, not a "box" element
        assert_eq!(tree.root.len(), 1);
        assert_eq!(tree.root[0].kind, "rect");
        assert_eq!(
            tree.root[0].props.get("color"),
            Some(&RenderValue::Color(0xff0000))
        );
        assert_eq!(
            tree.root[0].props.get("width"),
            Some(&RenderValue::Num(80.0, Some("px".to_string())))
        );
    }

    #[test]
    fn lower_component_with_defaults() {
        let tree = setup_and_lower(&[
            (
                "components/box.naze",
                "component box(color: color = #000000, size: number = 80px) {\n  rect width: size, height: size, color: color\n}\n",
            ),
            (
                "app.naze",
                "use components/box\n\napp \"Test\" {\n  box color: #ff0000\n}\n",
            ),
        ]);
        // color overridden, size uses default
        let rect = &tree.root[0];
        assert_eq!(rect.props.get("color"), Some(&RenderValue::Color(0xff0000)));
        assert_eq!(
            rect.props.get("width"),
            Some(&RenderValue::Num(80.0, Some("px".to_string())))
        );
    }

    #[test]
    fn lower_component_override_default() {
        let tree = setup_and_lower(&[
            (
                "components/box.naze",
                "component box(color: color = #000000, size: number = 80px) {\n  rect width: size, height: size, color: color\n}\n",
            ),
            (
                "app.naze",
                "use components/box\n\napp \"Test\" {\n  box color: #ff0000, size: 120px\n}\n",
            ),
        ]);
        let rect = &tree.root[0];
        assert_eq!(
            rect.props.get("width"),
            Some(&RenderValue::Num(120.0, Some("px".to_string())))
        );
    }

    #[test]
    fn lower_component_nested_children() {
        let tree = setup_and_lower(&[
            (
                "components/card.naze",
                "component card(bg: color = #ffffff) {\n  container color: bg, padding: 16px {\n    text \"inside card\"\n  }\n}\n",
            ),
            (
                "app.naze",
                "use components/card\n\napp \"Test\" {\n  card bg: #eeeeff\n}\n",
            ),
        ]);
        assert_eq!(tree.root.len(), 1);
        let container = &tree.root[0];
        assert_eq!(container.kind, "container");
        assert_eq!(
            container.props.get("color"),
            Some(&RenderValue::Color(0xeeeeff))
        );
        assert_eq!(container.children.len(), 1);
        assert_eq!(container.children[0].kind, "text");
    }

    #[test]
    fn lower_multiple_components() {
        let tree = setup_and_lower(&[
            (
                "components/box.naze",
                "component box(color: color) {\n  rect width: 80px, height: 80px, color: color\n}\n",
            ),
            (
                "components/card.naze",
                "component card(bg: color = #ffffff) {\n  container color: bg, padding: 16px {\n    text \"card\"\n  }\n}\n",
            ),
            (
                "app.naze",
                "use components/box\nuse components/card\n\napp \"Test\" {\n  box color: #ff0000\n  card bg: #00ff00\n}\n",
            ),
        ]);
        assert_eq!(tree.root.len(), 2);
        assert_eq!(tree.root[0].kind, "rect");
        assert_eq!(tree.root[1].kind, "container");
    }

    #[test]
    fn serialize_roundtrip() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"app "Hello" {
  column padding: 20px {
    text "world"
    rect width: 100px, height: 50px, color: #ff0000
  }
}"#,
        )]);
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn serialize_size_reasonable() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"app "Hello" {
  column padding: 20px {
    text "world"
    rect width: 100px, height: 50px, color: #ff0000
  }
}"#,
        )]);
        let bytes = serialize(&tree);
        // MessagePack should be very compact — well under 1KB for this
        assert!(
            bytes.len() < 1024,
            "serialized size {} bytes is unexpectedly large",
            bytes.len()
        );
    }

    #[test]
    fn all_examples_lower() {
        let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples");

        for name in &[
            "hello.naze",
            "boxes.naze",
            "columns.naze",
            "rows.naze",
            "nested.naze",
            "padding.naze",
            "rounded.naze",
            "colors.naze",
            "typography.naze",
            "grid.naze",
            "dashboard-static.naze",
            "app-shell.naze",
            "counter.naze",
            "conditional.naze",
        ] {
            let project = resolve(&examples_dir, name, &[]);
            let tree = lower(&project);
            assert!(!tree.title.is_empty(), "empty title in {}", name);
            assert!(!tree.root.is_empty(), "empty root in {}", name);

            // Roundtrip through MessagePack
            let bytes = serialize(&tree);
            let restored = deserialize(&bytes).unwrap();
            assert_eq!(tree, restored, "roundtrip failed for {}", name);
        }
    }

    #[test]
    fn lower_state_declarations() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "app \"Counter\" {\n  state count = 0\n  let label = \"Count\"\n  heading \"{label}: {count}\"\n}\n",
        )]);
        assert_eq!(tree.title, "Counter");
        // State declaration should be collected
        assert_eq!(tree.state.len(), 1);
        assert_eq!(tree.state[0].name, "count");
        assert_eq!(tree.state[0].initial, RenderValue::Num(0.0, None));
        // heading should have interpolated text
        assert_eq!(tree.root.len(), 1);
        assert_eq!(tree.root[0].kind, "heading");
        match tree.root[0].props.get("__text") {
            Some(RenderValue::InterpolatedStr(parts)) => {
                assert_eq!(parts.len(), 3); // StateRef("label"), Literal(": "), StateRef("count")
            }
            other => panic!("expected InterpolatedStr, got {:?}", other),
        }
    }

    #[test]
    fn lower_let_inlines_plain_string() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "app \"Test\" {\n  let greeting = \"Hello\"\n  text \"world\"\n}\n",
        )]);
        // let binding shouldn't produce render nodes
        assert_eq!(tree.root.len(), 1);
        assert_eq!(tree.root[0].kind, "text");
        // No state declarations from let bindings
        assert!(tree.state.is_empty());
    }

    #[test]
    fn all_examples_with_components_lower() {
        let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples");

        for name in &[
            "component-basic.naze",
            "component-props.naze",
            "multi-component.naze",
            "slots.naze",
        ] {
            let project = resolve(&examples_dir, name, &[]);
            let tree = lower(&project);
            assert!(!tree.title.is_empty(), "empty title in {}", name);
            assert!(!tree.root.is_empty(), "empty root in {}", name);

            // Verify no component nodes remain — all should be inlined
            fn assert_no_components(nodes: &[RenderNode]) {
                for node in nodes {
                    assert!(
                        !["box", "card", "color-box"].contains(&node.kind.as_str()),
                        "component '{}' was not inlined",
                        node.kind
                    );
                    assert_no_components(&node.children);
                }
            }
            assert_no_components(&tree.root);

            // Roundtrip
            let bytes = serialize(&tree);
            let restored = deserialize(&bytes).unwrap();
            assert_eq!(tree, restored, "roundtrip failed for {}", name);
        }
    }

    #[test]
    fn lower_if_condition() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "app \"Test\" {\n  state count = 0\n  if count > 0 {\n    text \"positive\"\n  } else {\n    text \"zero\"\n  }\n}\n",
        )]);
        assert_eq!(tree.root.len(), 1);
        let if_node = &tree.root[0];
        assert_eq!(if_node.kind, "__if");
        assert!(if_node.condition.is_some());
        assert_eq!(if_node.children.len(), 1); // then branch
        assert!(if_node.else_children.is_some());
        assert_eq!(if_node.else_children.as_ref().unwrap().len(), 1); // else branch

        // Roundtrip
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn lower_each_iteration() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "app \"Test\" {\n  state items = [\"Apple\", \"Banana\"]\n  each item in items {\n    text \"{item}\"\n  }\n}\n",
        )]);
        assert_eq!(tree.root.len(), 1);
        let each_node = &tree.root[0];
        assert_eq!(each_node.kind, "__each");
        assert!(each_node.each_binding.is_some());
        let (var, _) = each_node.each_binding.as_ref().unwrap();
        assert_eq!(var, "item");
        assert_eq!(each_node.children.len(), 1); // template child

        // Roundtrip
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn lower_list_state() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "app \"Test\" {\n  state items = [\"a\", \"b\", \"c\"]\n  text \"hello\"\n}\n",
        )]);
        assert_eq!(tree.state.len(), 1);
        assert_eq!(tree.state[0].name, "items");
        match &tree.state[0].initial {
            RenderValue::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], RenderValue::Str("a".to_string()));
            }
            other => panic!("expected List, got {:?}", other),
        }

        // Roundtrip
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn lower_component_with_default_slot() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"component card(title: text) {
  rect color: #f0f0f0 {
    heading "{title}"
    slot
  }
}

app "Test" {
  card title: "Hello" {
    text "card body"
    text "more content"
  }
}
"#,
        )]);
        // card inlines to: rect { heading, text "card body", text "more content" }
        assert_eq!(tree.root.len(), 1);
        let rect = &tree.root[0];
        assert_eq!(rect.kind, "rect");
        assert_eq!(rect.children.len(), 3); // heading + 2 slot children
        assert_eq!(rect.children[0].kind, "heading");
        assert_eq!(rect.children[1].kind, "text");
        assert_eq!(
            rect.children[1].props.get("__text"),
            Some(&RenderValue::Str("card body".to_string()))
        );
        assert_eq!(rect.children[2].kind, "text");
    }

    #[test]
    fn lower_component_with_named_slots() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"component page(title: text) {
  column {
    slot "header"
    heading "{title}"
    slot
    slot "footer"
  }
}

app "Test" {
  page title: "My Page" {
    fill "header" {
      text "Header Content"
    }
    text "Main Content"
    fill "footer" {
      text "Footer Content"
    }
  }
}
"#,
        )]);
        assert_eq!(tree.root.len(), 1);
        let col = &tree.root[0];
        assert_eq!(col.kind, "column");
        // header slot content + heading + default slot content + footer slot content
        assert_eq!(col.children.len(), 4);
        assert_eq!(
            col.children[0].props.get("__text"),
            Some(&RenderValue::Str("Header Content".to_string()))
        );
        assert_eq!(col.children[1].kind, "heading");
        assert_eq!(
            col.children[2].props.get("__text"),
            Some(&RenderValue::Str("Main Content".to_string()))
        );
        assert_eq!(
            col.children[3].props.get("__text"),
            Some(&RenderValue::Str("Footer Content".to_string()))
        );
    }

    #[test]
    fn lower_slot_fallback() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"component panel(title: text) {
  rect color: #ffffff {
    heading "{title}"
    slot {
      text "No content provided"
    }
  }
}

app "Test" {
  panel title: "Empty"
}
"#,
        )]);
        let rect = &tree.root[0];
        assert_eq!(rect.kind, "rect");
        assert_eq!(rect.children.len(), 2); // heading + fallback text
        assert_eq!(
            rect.children[1].props.get("__text"),
            Some(&RenderValue::Str("No content provided".to_string()))
        );
    }

    #[test]
    fn lower_slot_fallback_overridden() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"component panel(title: text) {
  rect color: #ffffff {
    heading "{title}"
    slot {
      text "No content provided"
    }
  }
}

app "Test" {
  panel title: "Full" {
    text "Custom content"
  }
}
"#,
        )]);
        let rect = &tree.root[0];
        assert_eq!(rect.children.len(), 2); // heading + custom content
        assert_eq!(
            rect.children[1].props.get("__text"),
            Some(&RenderValue::Str("Custom content".to_string()))
        );
    }

    #[test]
    fn lower_theme_refs() {
        // Theme refs should resolve to actual values from the default theme
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"app "Theme Test" {
  rect width: 100px, height: 100px, color: theme.colors.primary
  column padding: theme.spacing.md {
    text "themed"
  }
}
"#,
        )]);
        assert_eq!(tree.root.len(), 2); // rect + column
                                        // theme.colors.primary is #2563eb (default)
        assert_eq!(
            tree.root[0].props.get("color"),
            Some(&RenderValue::Color(0x2563eb))
        );
        // theme.spacing.md is 16px (default)
        assert_eq!(
            tree.root[1].props.get("padding"),
            Some(&RenderValue::Num(16.0, Some("px".to_string())))
        );
    }

    #[test]
    fn lower_computed_pipeline() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"app "Pipeline Test" {
  state items = [1, 2, 3]
  computed total = items | map __it | sum
  computed n = items | count
  text "hello"
}
"#,
        )]);
        assert_eq!(tree.computed.len(), 2);
        assert_eq!(tree.computed[0].name, "total");
        match &tree.computed[0].expr {
            IrExpression::Pipeline { source, stages } => {
                assert!(matches!(**source, IrExpression::StateRef(ref s) if s == "items"));
                assert_eq!(stages.len(), 2);
                assert_eq!(stages[0].function, 1); // map
                assert_eq!(stages[1].function, 4); // sum
            }
            other => panic!("expected Pipeline, got {:?}", other),
        }
        assert_eq!(tree.computed[1].name, "n");
        match &tree.computed[1].expr {
            IrExpression::Pipeline { stages, .. } => {
                assert_eq!(stages.len(), 1);
                assert_eq!(stages[0].function, 5); // count
            }
            other => panic!("expected Pipeline, got {:?}", other),
        }

        // Roundtrip
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn lower_each_with_pipeline() {
        let tree = setup_and_lower(&[(
            "app.naze",
            r#"app "Pipeline Each" {
  state items = [1, 2, 3]
  each item in items | take 2 {
    text "{item}"
  }
}
"#,
        )]);
        assert_eq!(tree.root.len(), 1);
        let each_node = &tree.root[0];
        assert_eq!(each_node.kind, "__each");
        let (var, iterable) = each_node.each_binding.as_ref().unwrap();
        assert_eq!(var, "item");
        match iterable {
            IrExpression::Pipeline { source, stages } => {
                assert!(matches!(**source, IrExpression::StateRef(ref s) if s == "items"));
                assert_eq!(stages.len(), 1);
                assert_eq!(stages[0].function, 3); // take
            }
            other => panic!("expected Pipeline, got {:?}", other),
        }
    }

    // ─── M39: Declarative Database Query Tests ──────────────────────────────

    #[test]
    fn lower_find_query_to_sql() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "server function get-users() {\n  let users = find users where active == true order name limit 10\n  users\n}\napp \"Test\" {\n  text \"hi\"\n}\n",
        )]);
        assert_eq!(tree.server_functions.len(), 1);
        let func = &tree.server_functions[0];
        assert_eq!(func.name, "get-users");
        assert_eq!(func.body.lets.len(), 1);
        match &func.body.lets[0].1 {
            naze_ir::IrServerStep::Sql { query, params } => {
                assert_eq!(
                    query,
                    "SELECT * FROM users WHERE active = $1 ORDER BY name ASC LIMIT $2"
                );
                assert_eq!(params.len(), 2);
            }
            other => panic!("expected IrServerStep::Sql, got {:?}", other),
        }
    }

    #[test]
    fn lower_insert_query_to_sql() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "server function create-user(name: text, email: text) {\n  let user = insert users { name: name, email: email }\n  user\n}\napp \"Test\" {\n  text \"hi\"\n}\n",
        )]);
        let func = &tree.server_functions[0];
        match &func.body.lets[0].1 {
            naze_ir::IrServerStep::Sql { query, params } => {
                assert_eq!(
                    query,
                    "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *"
                );
                assert_eq!(params.len(), 2);
            }
            other => panic!("expected IrServerStep::Sql, got {:?}", other),
        }
    }

    #[test]
    fn lower_update_query_to_sql() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "server function update-user(id: number, name: text) {\n  let result = update users set { name: name } where id == id\n  result\n}\napp \"Test\" {\n  text \"hi\"\n}\n",
        )]);
        let func = &tree.server_functions[0];
        match &func.body.lets[0].1 {
            naze_ir::IrServerStep::Sql { query, params } => {
                assert_eq!(
                    query,
                    "UPDATE users SET name = $1 WHERE id = $2 RETURNING *"
                );
                assert_eq!(params.len(), 2);
            }
            other => panic!("expected IrServerStep::Sql, got {:?}", other),
        }
    }

    #[test]
    fn lower_delete_query_to_sql() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "server function remove-user(id: number) {\n  let result = delete users where id == id\n  result\n}\napp \"Test\" {\n  text \"hi\"\n}\n",
        )]);
        let func = &tree.server_functions[0];
        match &func.body.lets[0].1 {
            naze_ir::IrServerStep::Sql { query, params } => {
                assert_eq!(query, "DELETE FROM users WHERE id = $1 RETURNING *");
                assert_eq!(params.len(), 1);
            }
            other => panic!("expected IrServerStep::Sql, got {:?}", other),
        }
    }

    #[test]
    fn lower_find_no_clauses() {
        let tree = setup_and_lower(&[(
            "app.naze",
            "server function get-all() {\n  let items = find items\n  items\n}\napp \"Test\" {\n  text \"hi\"\n}\n",
        )]);
        let func = &tree.server_functions[0];
        match &func.body.lets[0].1 {
            naze_ir::IrServerStep::Sql { query, params } => {
                assert_eq!(query, "SELECT * FROM items");
                assert!(params.is_empty());
            }
            other => panic!("expected IrServerStep::Sql, got {:?}", other),
        }
    }
}
