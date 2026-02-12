//! Intermediate representation types shared between the compiler and runtime.
//! Uses a simple custom binary format to minimize WASM size (no serde in WASM).

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A segment of an interpolated string in the render tree.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TextPart {
    Literal(String),
    StateRef(String), // reference to a state variable by name
}

/// A property value in the render tree, stripped of AST-specific details.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RenderValue {
    Str(String),
    Num(f64, Option<String>), // value + optional unit ("px", "%", "em")
    Color(u32),
    Bool(bool),
    InterpolatedStr(Vec<TextPart>), // string with embedded state references
    List(Vec<RenderValue>),
    Object(Vec<(String, RenderValue)>), // Object literal: { key: value, ... }
    Bind(String),                       // Two-way state binding for form elements
}

/// A state variable declaration with its initial value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StateDecl {
    pub name: String,
    pub initial: RenderValue,
    pub shared: bool, // true = persists across page navigation
}

/// An async data fetch declaration.
/// Creates three derived state variables: {name}.loading, {name}.error, {name}.data
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataDecl {
    pub name: String,
    pub url: String,
    pub source_type: u8,      // 0 = fetch, 1 = websocket, 2 = sse, 3 = js, 4 = device
    pub method: String,       // "get", "post", "put", "delete", "patch"
    pub cache_ms: u64,        // 0 = no cache
    pub retry_count: u32,     // 0 = no retry
    pub trigger_mode: u8,     // 0 = auto, 1 = manual
    pub content_type: String, // e.g. "application/json"
    pub watch: bool,          // for device APIs: continuously watch vs one-shot
    pub headers: Vec<(String, RenderValue)>, // request headers with interpolation
}

/// A computed (read-only, derived) state declaration.
/// Value is re-evaluated whenever referenced state variables change.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ComputedDecl {
    pub name: String,
    pub expr: IrExpression,
}

/// A persistent storage declaration backed by localStorage or sessionStorage.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StorageDecl {
    pub name: String,
    pub storage_type: u8, // 0 = local, 1 = session
    pub key: String,
    pub default: RenderValue,
}

/// A timer declaration: one-shot (after) or repeating (every).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimerDecl {
    pub name: String,
    pub kind: u8, // 0 = after (setTimeout), 1 = every (setInterval)
    pub duration_ms: u64,
    pub action: IrAction,
}

/// A URL parameter declaration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParamDecl {
    pub name: String,
    pub param_type: String, // "text", "number", "bool", "color"
    pub default: RenderValue,
}

/// A theme definition with resolved tokens for runtime switching.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ThemeDef {
    pub name: String,
    pub colors: Vec<(String, u32)>,
    pub spacing: Vec<(String, f64)>,
}

/// Binary operators for expressions.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IrBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    And,
    Or,
}

/// A single stage in an IR pipeline expression.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IrPipelineStage {
    pub function: u8, // 0=filter, 1=map, 2=sort-by, 3=take, 4=sum, 5=count, 6=reduce, 7=group-by, 8=flatten, 9=distinct
    pub argument: Option<IrExpression>,
    pub argument2: Option<IrExpression>, // for reduce: initial value
}

/// An expression in the IR (used in event handler actions).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IrExpression {
    Num(f64),
    Str(String),
    Bool(bool),
    StateRef(String),
    BinOp {
        left: Box<IrExpression>,
        op: IrBinOp,
        right: Box<IrExpression>,
    },
    Pipeline {
        source: Box<IrExpression>,
        stages: Vec<IrPipelineStage>,
    },
    WasmCall {
        module: String,
        function: String,
        args: Vec<IrExpression>,
    },
    /// Server-side env var reference, resolved at runtime via std::env::var().
    EnvRef(String),
}

/// An action triggered by an event handler.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IrAction {
    Set {
        target: String,
        expr: IrExpression,
    },
    Navigate {
        path: String,
    },
    ScrollTo {
        element_id: String,
    },
    Log {
        expr: IrExpression,
    },
    Trigger {
        data_name: String,
    },
    Copy {
        expr: IrExpression,
    },
    Send {
        stream_name: String,
        expr: IrExpression,
    },
    JsCall {
        function_name: String,
        args: Vec<IrExpression>,
        target: Option<String>,
    },
    Notify {
        title: String,
        body: String,
        icon: String,
    },
    SetTheme {
        name: String,
    },
}

/// An event handler on a render node.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IrEventHandler {
    pub event: String,
    pub action: IrAction,
    pub modifier_kind: u8, // 0 = none, 1 = debounce, 2 = throttle
    pub modifier_ms: u64,  // 0 if no modifier
}

/// A node in the flattened render tree.
/// Components have been inlined — only built-in elements remain.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderNode {
    pub kind: String,
    pub props: HashMap<String, RenderValue>,
    pub children: Vec<RenderNode>,
    pub handlers: Vec<IrEventHandler>,
    /// Source location (file, line, column) — not serialized to binary, used for source maps.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub span: Option<(String, u32, u32)>,
    pub condition: Option<IrExpression>,
    pub else_children: Option<Vec<RenderNode>>,
    pub each_binding: Option<(String, IrExpression)>,
}

impl PartialEq for RenderNode {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.props == other.props
            && self.children == other.children
            && self.handlers == other.handlers
            && self.condition == other.condition
            && self.else_children == other.else_children
            && self.each_binding == other.each_binding
        // span intentionally excluded — it's metadata for source maps, not semantic data
    }
}

/// A page definition with path and content.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PageDef {
    pub path: String,
    pub params: Vec<String>,
    pub is_catch_all: bool,
    pub guard: Option<String>,
    pub meta: Vec<(String, RenderValue)>,
    pub root: Vec<RenderNode>,
}

/// A guard definition for route protection.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GuardDef {
    pub name: String,
    pub checks: Vec<GuardCheck>,
}

/// A single guard check: condition + redirect.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GuardCheck {
    pub condition: IrExpression, // e.g. !auth-token
    pub redirect: String,       // e.g. "/login"
}

/// A single source location mapping from binary offset to source file position.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SourceMapping {
    pub binary_offset: u32,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Source map linking binary offsets to .naze source locations.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SourceMap {
    pub mappings: Vec<SourceMapping>,
}

/// A WASM module import declaration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImportDecl {
    /// Local module name (e.g., "crypto")
    pub name: String,
    /// Relative URL for runtime loading (e.g., "crypto.wasm")
    pub wasm_url: String,
    /// Exported function names (from wasmparser analysis)
    pub functions: Vec<String>,
}

/// A server function declaration (body evaluated server-side).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ServerFuncDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: IrServerBody,
}

/// Server function body: a sequence of let bindings + a final result expression.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IrServerBody {
    pub lets: Vec<(String, IrServerStep)>,
    pub result: IrExpression,
}

/// A single step in a server function body.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IrServerStep {
    Fetch(String),           // HTTP GET fetch with URL (may contain interpolations)
    Sql { query: String, params: Vec<IrExpression> }, // SQL query with $N placeholders
    Expr(IrExpression),      // Regular expression evaluation
}

/// An AI prompt declaration (calls AI provider via server proxy).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PromptDecl {
    pub name: String,        // binding name: "summary"
    pub provider: String,    // "openai", "anthropic", "ollama"
    pub system: String,      // system prompt (may contain {interpolations})
    pub user: String,        // user prompt (may contain {interpolations})
    pub model: String,       // model name (e.g. "gpt-4o")
    pub max_tokens: u32,     // default 1000
    pub temperature: f64,    // default 0.7
}

/// A server function call as a data source (calls `/api/{func_name}`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ServerCallDecl {
    pub name: String,
    pub func_name: String,
    pub args: Vec<IrExpression>,
}

/// The serializable render tree for the entire app.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderTree {
    pub title: String,
    pub root: Vec<RenderNode>, // Default page content (for single-page apps)
    pub state: Vec<StateDecl>,
    pub data: Vec<DataDecl>,         // Async data fetch declarations
    pub computed: Vec<ComputedDecl>, // Read-only derived state
    pub storage: Vec<StorageDecl>,   // Persistent state (localStorage/sessionStorage)
    pub timers: Vec<TimerDecl>,      // setTimeout/setInterval declarations
    pub params: Vec<ParamDecl>,      // URL parameter declarations
    pub pages: Vec<PageDef>,         // Named pages for routing
    pub themes: Vec<ThemeDef>,       // Theme definitions for runtime switching
    pub imports: Vec<ImportDecl>,    // WASM module imports
    pub server_functions: Vec<ServerFuncDecl>, // Server function definitions
    pub server_calls: Vec<ServerCallDecl>,     // Server function data sources
    pub prompts: Vec<PromptDecl>,              // AI prompt declarations
    pub guards: Vec<GuardDef>,                 // Guard definitions for route protection
}

// ─── Simple binary encoding ─────────────────────────────────────────────────
// Format:
//   String: u32 len + utf8 bytes
//   TextPart: u8 tag + payload
//     0 = Literal(String)
//     1 = StateRef(String)
//   RenderValue: u8 tag + payload
//     0 = Str(String)
//     1 = Num(f64, Option<String>)
//     2 = Color(u32)
//     3 = Bool(bool)
//     4 = InterpolatedStr(u32 part_count + TextParts)
//     5 = List(u32 count + RenderValues)
//     6 = Bind(String) - two-way state binding
//   StateDecl: String name + RenderValue initial
//   RenderNode: String kind + u32 prop_count + props + u32 child_count + children
//              + u32 handler_count + handlers + u8 flags + optional fields
//     flags bit 0: has condition (IrExpression)
//     flags bit 1: has else_children (u32 count + RenderNodes)
//     flags bit 2: has each_binding (String variable + IrExpression iterable)
//   RenderTree: String title + u32 state_count + states + u32 root_count + root nodes

/// Serialize a RenderTree to compact binary bytes.
pub fn serialize(tree: &RenderTree) -> Vec<u8> {
    let mut buf = Vec::new();
    write_string(&mut buf, &tree.title);
    write_u32(&mut buf, tree.state.len() as u32);
    for decl in &tree.state {
        write_string(&mut buf, &decl.name);
        write_value(&mut buf, &decl.initial);
        buf.push(if decl.shared { 1 } else { 0 });
    }
    // Data fetch declarations
    write_u32(&mut buf, tree.data.len() as u32);
    for decl in &tree.data {
        write_string(&mut buf, &decl.name);
        write_string(&mut buf, &decl.url);
        buf.push(decl.source_type);
        write_string(&mut buf, &decl.method);
        write_u64(&mut buf, decl.cache_ms);
        write_u32(&mut buf, decl.retry_count);
        buf.push(decl.trigger_mode);
        write_string(&mut buf, &decl.content_type);
        buf.push(if decl.watch { 1 } else { 0 });
        // Headers
        write_u32(&mut buf, decl.headers.len() as u32);
        for (key, val) in &decl.headers {
            write_string(&mut buf, key);
            write_value(&mut buf, val);
        }
    }
    // Computed declarations
    write_u32(&mut buf, tree.computed.len() as u32);
    for decl in &tree.computed {
        write_string(&mut buf, &decl.name);
        write_expression(&mut buf, &decl.expr);
    }
    // Storage declarations
    write_u32(&mut buf, tree.storage.len() as u32);
    for decl in &tree.storage {
        write_string(&mut buf, &decl.name);
        buf.push(decl.storage_type);
        write_string(&mut buf, &decl.key);
        write_value(&mut buf, &decl.default);
    }
    // Timer declarations
    write_u32(&mut buf, tree.timers.len() as u32);
    for decl in &tree.timers {
        write_string(&mut buf, &decl.name);
        buf.push(decl.kind);
        write_u64(&mut buf, decl.duration_ms);
        write_action(&mut buf, &decl.action);
    }
    // Param declarations
    write_u32(&mut buf, tree.params.len() as u32);
    for decl in &tree.params {
        write_string(&mut buf, &decl.name);
        write_string(&mut buf, &decl.param_type);
        write_value(&mut buf, &decl.default);
    }
    write_u32(&mut buf, tree.root.len() as u32);
    for node in &tree.root {
        write_node(&mut buf, node);
    }
    // Pages (for multi-page routing)
    write_u32(&mut buf, tree.pages.len() as u32);
    for page in &tree.pages {
        write_string(&mut buf, &page.path);
        write_u32(&mut buf, page.params.len() as u32);
        for p in &page.params {
            write_string(&mut buf, p);
        }
        buf.push(if page.is_catch_all { 1 } else { 0 });
        // Guard reference
        match &page.guard {
            Some(g) => { buf.push(1); write_string(&mut buf, g); }
            None => buf.push(0),
        }
        write_u32(&mut buf, page.meta.len() as u32);
        for (key, val) in &page.meta {
            write_string(&mut buf, key);
            write_value(&mut buf, val);
        }
        write_u32(&mut buf, page.root.len() as u32);
        for node in &page.root {
            write_node(&mut buf, node);
        }
    }
    // Theme definitions (for runtime switching)
    write_u32(&mut buf, tree.themes.len() as u32);
    for theme in &tree.themes {
        write_string(&mut buf, &theme.name);
        write_u32(&mut buf, theme.colors.len() as u32);
        for (name, color) in &theme.colors {
            write_string(&mut buf, name);
            write_u32(&mut buf, *color);
        }
        write_u32(&mut buf, theme.spacing.len() as u32);
        for (name, value) in &theme.spacing {
            write_string(&mut buf, name);
            write_f64(&mut buf, *value);
        }
    }
    // WASM module imports
    write_u32(&mut buf, tree.imports.len() as u32);
    for imp in &tree.imports {
        write_string(&mut buf, &imp.name);
        write_string(&mut buf, &imp.wasm_url);
        write_u32(&mut buf, imp.functions.len() as u32);
        for f in &imp.functions {
            write_string(&mut buf, f);
        }
    }
    // Server function definitions
    write_u32(&mut buf, tree.server_functions.len() as u32);
    for sf in &tree.server_functions {
        write_string(&mut buf, &sf.name);
        write_u32(&mut buf, sf.params.len() as u32);
        for p in &sf.params {
            write_string(&mut buf, p);
        }
        // Serialize IrServerBody: let count + lets + result
        write_u32(&mut buf, sf.body.lets.len() as u32);
        for (name, step) in &sf.body.lets {
            write_string(&mut buf, name);
            match step {
                IrServerStep::Fetch(url) => {
                    buf.push(0);
                    write_string(&mut buf, url);
                }
                IrServerStep::Sql { query, params } => {
                    buf.push(2);
                    write_string(&mut buf, query);
                    write_u32(&mut buf, params.len() as u32);
                    for p in params {
                        write_expression(&mut buf, p);
                    }
                }
                IrServerStep::Expr(expr) => {
                    buf.push(1);
                    write_expression(&mut buf, expr);
                }
            }
        }
        write_expression(&mut buf, &sf.body.result);
    }
    // Server function calls (data sources)
    write_u32(&mut buf, tree.server_calls.len() as u32);
    for sc in &tree.server_calls {
        write_string(&mut buf, &sc.name);
        write_string(&mut buf, &sc.func_name);
        write_u32(&mut buf, sc.args.len() as u32);
        for arg in &sc.args {
            write_expression(&mut buf, arg);
        }
    }
    // AI prompt declarations
    write_u32(&mut buf, tree.prompts.len() as u32);
    for p in &tree.prompts {
        write_string(&mut buf, &p.name);
        write_string(&mut buf, &p.provider);
        write_string(&mut buf, &p.system);
        write_string(&mut buf, &p.user);
        write_string(&mut buf, &p.model);
        write_u32(&mut buf, p.max_tokens);
        // Encode temperature as f64 (8 bytes)
        buf.extend_from_slice(&p.temperature.to_le_bytes());
    }
    // Guards
    write_u32(&mut buf, tree.guards.len() as u32);
    for g in &tree.guards {
        write_string(&mut buf, &g.name);
        write_u32(&mut buf, g.checks.len() as u32);
        for check in &g.checks {
            write_expression(&mut buf, &check.condition);
            write_string(&mut buf, &check.redirect);
        }
    }
    buf
}

/// Serialize a RenderTree to binary bytes and produce a source map.
/// The source map records binary offsets for each render node that has a span.
pub fn serialize_with_source_map(tree: &RenderTree) -> (Vec<u8>, SourceMap) {
    let mut buf = Vec::new();
    let mut source_map = SourceMap::default();

    write_string(&mut buf, &tree.title);
    write_u32(&mut buf, tree.state.len() as u32);
    for decl in &tree.state {
        write_string(&mut buf, &decl.name);
        write_value(&mut buf, &decl.initial);
        buf.push(if decl.shared { 1 } else { 0 });
    }
    write_u32(&mut buf, tree.data.len() as u32);
    for decl in &tree.data {
        write_string(&mut buf, &decl.name);
        write_string(&mut buf, &decl.url);
        buf.push(decl.source_type);
        write_string(&mut buf, &decl.method);
        write_u64(&mut buf, decl.cache_ms);
        write_u32(&mut buf, decl.retry_count);
        buf.push(decl.trigger_mode);
        write_string(&mut buf, &decl.content_type);
        buf.push(if decl.watch { 1 } else { 0 });
        // Headers
        write_u32(&mut buf, decl.headers.len() as u32);
        for (key, val) in &decl.headers {
            write_string(&mut buf, key);
            write_value(&mut buf, val);
        }
    }
    write_u32(&mut buf, tree.computed.len() as u32);
    for decl in &tree.computed {
        write_string(&mut buf, &decl.name);
        write_expression(&mut buf, &decl.expr);
    }
    write_u32(&mut buf, tree.storage.len() as u32);
    for decl in &tree.storage {
        write_string(&mut buf, &decl.name);
        buf.push(decl.storage_type);
        write_string(&mut buf, &decl.key);
        write_value(&mut buf, &decl.default);
    }
    write_u32(&mut buf, tree.timers.len() as u32);
    for decl in &tree.timers {
        write_string(&mut buf, &decl.name);
        buf.push(decl.kind);
        write_u64(&mut buf, decl.duration_ms);
        write_action(&mut buf, &decl.action);
    }
    write_u32(&mut buf, tree.params.len() as u32);
    for decl in &tree.params {
        write_string(&mut buf, &decl.name);
        write_string(&mut buf, &decl.param_type);
        write_value(&mut buf, &decl.default);
    }
    write_u32(&mut buf, tree.root.len() as u32);
    for node in &tree.root {
        write_node_mapped(&mut buf, node, &mut source_map);
    }
    write_u32(&mut buf, tree.pages.len() as u32);
    for page in &tree.pages {
        write_string(&mut buf, &page.path);
        write_u32(&mut buf, page.params.len() as u32);
        for p in &page.params {
            write_string(&mut buf, p);
        }
        buf.push(if page.is_catch_all { 1 } else { 0 });
        write_u32(&mut buf, page.root.len() as u32);
        for node in &page.root {
            write_node_mapped(&mut buf, node, &mut source_map);
        }
    }
    write_u32(&mut buf, tree.themes.len() as u32);
    for theme in &tree.themes {
        write_string(&mut buf, &theme.name);
        write_u32(&mut buf, theme.colors.len() as u32);
        for (name, color) in &theme.colors {
            write_string(&mut buf, name);
            write_u32(&mut buf, *color);
        }
        write_u32(&mut buf, theme.spacing.len() as u32);
        for (name, value) in &theme.spacing {
            write_string(&mut buf, name);
            write_f64(&mut buf, *value);
        }
    }
    // WASM module imports
    write_u32(&mut buf, tree.imports.len() as u32);
    for imp in &tree.imports {
        write_string(&mut buf, &imp.name);
        write_string(&mut buf, &imp.wasm_url);
        write_u32(&mut buf, imp.functions.len() as u32);
        for f in &imp.functions {
            write_string(&mut buf, f);
        }
    }
    // Server function definitions
    write_u32(&mut buf, tree.server_functions.len() as u32);
    for sf in &tree.server_functions {
        write_string(&mut buf, &sf.name);
        write_u32(&mut buf, sf.params.len() as u32);
        for p in &sf.params {
            write_string(&mut buf, p);
        }
        // Serialize IrServerBody: let count + lets + result
        write_u32(&mut buf, sf.body.lets.len() as u32);
        for (name, step) in &sf.body.lets {
            write_string(&mut buf, name);
            match step {
                IrServerStep::Fetch(url) => {
                    buf.push(0);
                    write_string(&mut buf, url);
                }
                IrServerStep::Sql { query, params } => {
                    buf.push(2);
                    write_string(&mut buf, query);
                    write_u32(&mut buf, params.len() as u32);
                    for p in params {
                        write_expression(&mut buf, p);
                    }
                }
                IrServerStep::Expr(expr) => {
                    buf.push(1);
                    write_expression(&mut buf, expr);
                }
            }
        }
        write_expression(&mut buf, &sf.body.result);
    }
    // Server function calls (data sources)
    write_u32(&mut buf, tree.server_calls.len() as u32);
    for sc in &tree.server_calls {
        write_string(&mut buf, &sc.name);
        write_string(&mut buf, &sc.func_name);
        write_u32(&mut buf, sc.args.len() as u32);
        for arg in &sc.args {
            write_expression(&mut buf, arg);
        }
    }
    // AI prompt declarations
    write_u32(&mut buf, tree.prompts.len() as u32);
    for p in &tree.prompts {
        write_string(&mut buf, &p.name);
        write_string(&mut buf, &p.provider);
        write_string(&mut buf, &p.system);
        write_string(&mut buf, &p.user);
        write_string(&mut buf, &p.model);
        write_u32(&mut buf, p.max_tokens);
        buf.extend_from_slice(&p.temperature.to_le_bytes());
    }

    (buf, source_map)
}

/// Write a node and record its span in the source map.
fn write_node_mapped(buf: &mut Vec<u8>, node: &RenderNode, source_map: &mut SourceMap) {
    if let Some((file, line, col)) = &node.span {
        source_map.mappings.push(SourceMapping {
            binary_offset: buf.len() as u32,
            file: file.clone(),
            line: *line,
            column: *col,
        });
    }
    write_string(buf, &node.kind);
    write_u32(buf, node.props.len() as u32);
    for (key, val) in &node.props {
        write_string(buf, key);
        write_value(buf, val);
    }
    write_u32(buf, node.children.len() as u32);
    for child in &node.children {
        write_node_mapped(buf, child, source_map);
    }
    write_u32(buf, node.handlers.len() as u32);
    for handler in &node.handlers {
        write_handler(buf, handler);
    }
    let mut flags: u8 = 0;
    if node.condition.is_some() {
        flags |= 1;
    }
    if node.else_children.is_some() {
        flags |= 2;
    }
    if node.each_binding.is_some() {
        flags |= 4;
    }
    buf.push(flags);
    if let Some(cond) = &node.condition {
        write_expression(buf, cond);
    }
    if let Some(else_nodes) = &node.else_children {
        write_u32(buf, else_nodes.len() as u32);
        for child in else_nodes {
            write_node_mapped(buf, child, source_map);
        }
    }
    if let Some((var, expr)) = &node.each_binding {
        write_string(buf, var);
        write_expression(buf, expr);
    }
}

/// Deserialize a RenderTree from binary bytes.
pub fn deserialize(data: &[u8]) -> Result<RenderTree, String> {
    let mut cursor = Cursor::new(data);
    let title = cursor.read_string()?;
    let state_count = cursor.read_u32()? as usize;
    let mut state = Vec::with_capacity(state_count);
    for _ in 0..state_count {
        let name = cursor.read_string()?;
        let initial = cursor.read_value()?;
        let shared = cursor.read_u8()? != 0;
        state.push(StateDecl {
            name,
            initial,
            shared,
        });
    }
    // Data fetch declarations
    let data_count = cursor.read_u32()? as usize;
    let mut data_decls = Vec::with_capacity(data_count);
    for _ in 0..data_count {
        let name = cursor.read_string()?;
        let url = cursor.read_string()?;
        let source_type = cursor.read_u8()?;
        let method = cursor.read_string()?;
        let cache_ms = cursor.read_u64()?;
        let retry_count = cursor.read_u32()?;
        let trigger_mode = cursor.read_u8()?;
        let content_type = cursor.read_string()?;
        let watch = cursor.read_u8()? != 0;
        // Headers
        let header_count = cursor.read_u32()? as usize;
        let mut headers = Vec::with_capacity(header_count);
        for _ in 0..header_count {
            let key = cursor.read_string()?;
            let val = cursor.read_value()?;
            headers.push((key, val));
        }
        data_decls.push(DataDecl {
            name,
            url,
            source_type,
            method,
            cache_ms,
            retry_count,
            trigger_mode,
            content_type,
            watch,
            headers,
        });
    }
    // Computed declarations
    let computed_count = cursor.read_u32()? as usize;
    let mut computed = Vec::with_capacity(computed_count);
    for _ in 0..computed_count {
        let name = cursor.read_string()?;
        let expr = cursor.read_expression()?;
        computed.push(ComputedDecl { name, expr });
    }
    // Storage declarations
    let storage_count = cursor.read_u32()? as usize;
    let mut storage = Vec::with_capacity(storage_count);
    for _ in 0..storage_count {
        let name = cursor.read_string()?;
        let storage_type = cursor.read_u8()?;
        let key = cursor.read_string()?;
        let default = cursor.read_value()?;
        storage.push(StorageDecl {
            name,
            storage_type,
            key,
            default,
        });
    }
    // Timer declarations
    let timer_count = cursor.read_u32()? as usize;
    let mut timers = Vec::with_capacity(timer_count);
    for _ in 0..timer_count {
        let name = cursor.read_string()?;
        let kind = cursor.read_u8()?;
        let duration_ms = cursor.read_u64()?;
        let action = cursor.read_action()?;
        timers.push(TimerDecl {
            name,
            kind,
            duration_ms,
            action,
        });
    }
    // Param declarations
    let param_count = cursor.read_u32()? as usize;
    let mut params = Vec::with_capacity(param_count);
    for _ in 0..param_count {
        let name = cursor.read_string()?;
        let param_type = cursor.read_string()?;
        let default = cursor.read_value()?;
        params.push(ParamDecl {
            name,
            param_type,
            default,
        });
    }
    let count = cursor.read_u32()? as usize;
    let mut root = Vec::with_capacity(count);
    for _ in 0..count {
        root.push(cursor.read_node()?);
    }
    // Pages (for multi-page routing) - optional for backward compatibility
    let pages = if cursor.pos < cursor.data.len() {
        let page_count = cursor.read_u32()? as usize;
        let mut pages = Vec::with_capacity(page_count);
        for _ in 0..page_count {
            let path = cursor.read_string()?;
            let param_count = cursor.read_u32()? as usize;
            let mut params = Vec::with_capacity(param_count);
            for _ in 0..param_count {
                params.push(cursor.read_string()?);
            }
            let is_catch_all = cursor.read_u8()? != 0;
            let guard = if cursor.read_u8()? != 0 {
                Some(cursor.read_string()?)
            } else {
                None
            };
            let meta_count = cursor.read_u32()? as usize;
            let mut meta = Vec::with_capacity(meta_count);
            for _ in 0..meta_count {
                let key = cursor.read_string()?;
                let val = cursor.read_value()?;
                meta.push((key, val));
            }
            let node_count = cursor.read_u32()? as usize;
            let mut page_root = Vec::with_capacity(node_count);
            for _ in 0..node_count {
                page_root.push(cursor.read_node()?);
            }
            pages.push(PageDef {
                path,
                params,
                is_catch_all,
                guard,
                meta,
                root: page_root,
            });
        }
        pages
    } else {
        vec![]
    };
    // Theme definitions (for runtime switching) - optional for backward compatibility
    let themes = if cursor.pos < cursor.data.len() {
        let theme_count = cursor.read_u32()? as usize;
        let mut themes = Vec::with_capacity(theme_count);
        for _ in 0..theme_count {
            let name = cursor.read_string()?;
            let color_count = cursor.read_u32()? as usize;
            let mut colors = Vec::with_capacity(color_count);
            for _ in 0..color_count {
                let color_name = cursor.read_string()?;
                let color_val = cursor.read_u32()?;
                colors.push((color_name, color_val));
            }
            let spacing_count = cursor.read_u32()? as usize;
            let mut spacing = Vec::with_capacity(spacing_count);
            for _ in 0..spacing_count {
                let spacing_name = cursor.read_string()?;
                let spacing_val = cursor.read_f64()?;
                spacing.push((spacing_name, spacing_val));
            }
            themes.push(ThemeDef {
                name,
                colors,
                spacing,
            });
        }
        themes
    } else {
        vec![]
    };
    // WASM module imports - optional for backward compatibility
    let imports = if cursor.pos < cursor.data.len() {
        let import_count = cursor.read_u32()? as usize;
        let mut imports = Vec::with_capacity(import_count);
        for _ in 0..import_count {
            let name = cursor.read_string()?;
            let wasm_url = cursor.read_string()?;
            let func_count = cursor.read_u32()? as usize;
            let mut functions = Vec::with_capacity(func_count);
            for _ in 0..func_count {
                functions.push(cursor.read_string()?);
            }
            imports.push(ImportDecl {
                name,
                wasm_url,
                functions,
            });
        }
        imports
    } else {
        vec![]
    };
    // Server function definitions - optional for backward compatibility
    let server_functions = if cursor.pos < cursor.data.len() {
        let count = cursor.read_u32()? as usize;
        let mut fns = Vec::with_capacity(count);
        for _ in 0..count {
            let name = cursor.read_string()?;
            let param_count = cursor.read_u32()? as usize;
            let mut params = Vec::with_capacity(param_count);
            for _ in 0..param_count {
                params.push(cursor.read_string()?);
            }
            // Deserialize IrServerBody: let count + lets + result
            let let_count = cursor.read_u32()? as usize;
            let mut lets = Vec::with_capacity(let_count);
            for _ in 0..let_count {
                let let_name = cursor.read_string()?;
                let tag = cursor.read_u8()?;
                let step = match tag {
                    0 => IrServerStep::Fetch(cursor.read_string()?),
                    1 => IrServerStep::Expr(cursor.read_expression()?),
                    2 => {
                        let query = cursor.read_string()?;
                        let param_count = cursor.read_u32()? as usize;
                        let mut params = Vec::with_capacity(param_count);
                        for _ in 0..param_count {
                            params.push(cursor.read_expression()?);
                        }
                        IrServerStep::Sql { query, params }
                    }
                    _ => return Err(format!("unknown server step tag: {}", tag)),
                };
                lets.push((let_name, step));
            }
            let result = cursor.read_expression()?;
            let body = IrServerBody { lets, result };
            fns.push(ServerFuncDecl { name, params, body });
        }
        fns
    } else {
        vec![]
    };
    // Server function calls - optional for backward compatibility
    let server_calls = if cursor.pos < cursor.data.len() {
        let count = cursor.read_u32()? as usize;
        let mut calls = Vec::with_capacity(count);
        for _ in 0..count {
            let name = cursor.read_string()?;
            let func_name = cursor.read_string()?;
            let arg_count = cursor.read_u32()? as usize;
            let mut args = Vec::with_capacity(arg_count);
            for _ in 0..arg_count {
                args.push(cursor.read_expression()?);
            }
            calls.push(ServerCallDecl {
                name,
                func_name,
                args,
            });
        }
        calls
    } else {
        vec![]
    };
    // AI prompt declarations - optional for backward compatibility
    let prompts = if cursor.pos < cursor.data.len() {
        let count = cursor.read_u32()? as usize;
        let mut decls = Vec::with_capacity(count);
        for _ in 0..count {
            let name = cursor.read_string()?;
            let provider = cursor.read_string()?;
            let system = cursor.read_string()?;
            let user = cursor.read_string()?;
            let model = cursor.read_string()?;
            let max_tokens = cursor.read_u32()?;
            if cursor.pos + 8 > cursor.data.len() {
                return Err("unexpected end of data reading prompt temperature".to_string());
            }
            let temperature = f64::from_le_bytes(
                cursor.data[cursor.pos..cursor.pos + 8]
                    .try_into()
                    .map_err(|_| "bad f64 bytes")?,
            );
            cursor.pos += 8;
            decls.push(PromptDecl {
                name,
                provider,
                system,
                user,
                model,
                max_tokens,
                temperature,
            });
        }
        decls
    } else {
        vec![]
    };
    let guards = if cursor.pos < cursor.data.len() {
        let count = cursor.read_u32()? as usize;
        let mut defs = Vec::with_capacity(count);
        for _ in 0..count {
            let name = cursor.read_string()?;
            let check_count = cursor.read_u32()? as usize;
            let mut checks = Vec::with_capacity(check_count);
            for _ in 0..check_count {
                let condition = cursor.read_expression()?;
                let redirect = cursor.read_string()?;
                checks.push(GuardCheck { condition, redirect });
            }
            defs.push(GuardDef { name, checks });
        }
        defs
    } else {
        vec![]
    };
    Ok(RenderTree {
        title,
        root,
        state,
        data: data_decls,
        computed,
        storage,
        timers,
        params,
        pages,
        themes,
        imports,
        server_functions,
        server_calls,
        prompts,
        guards,
    })
}

// ─── Writer ─────────────────────────────────────────────────────────────────

fn write_u32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn write_u64(buf: &mut Vec<u8>, val: u64) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn write_f64(buf: &mut Vec<u8>, val: f64) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    write_u32(buf, s.len() as u32);
    buf.extend_from_slice(s.as_bytes());
}

fn write_value(buf: &mut Vec<u8>, val: &RenderValue) {
    match val {
        RenderValue::Str(s) => {
            buf.push(0);
            write_string(buf, s);
        }
        RenderValue::Num(n, unit) => {
            buf.push(1);
            write_f64(buf, *n);
            match unit {
                Some(u) => {
                    buf.push(1);
                    write_string(buf, u);
                }
                None => buf.push(0),
            }
        }
        RenderValue::Color(c) => {
            buf.push(2);
            write_u32(buf, *c);
        }
        RenderValue::Bool(b) => {
            buf.push(3);
            buf.push(if *b { 1 } else { 0 });
        }
        RenderValue::InterpolatedStr(parts) => {
            buf.push(4);
            write_u32(buf, parts.len() as u32);
            for part in parts {
                match part {
                    TextPart::Literal(s) => {
                        buf.push(0);
                        write_string(buf, s);
                    }
                    TextPart::StateRef(name) => {
                        buf.push(1);
                        write_string(buf, name);
                    }
                }
            }
        }
        RenderValue::List(items) => {
            buf.push(5);
            write_u32(buf, items.len() as u32);
            for item in items {
                write_value(buf, item);
            }
        }
        RenderValue::Bind(name) => {
            buf.push(6);
            write_string(buf, name);
        }
        RenderValue::Object(entries) => {
            buf.push(7);
            write_u32(buf, entries.len() as u32);
            for (key, value) in entries {
                write_string(buf, key);
                write_value(buf, value);
            }
        }
    }
}

fn write_node(buf: &mut Vec<u8>, node: &RenderNode) {
    write_string(buf, &node.kind);
    write_u32(buf, node.props.len() as u32);
    for (key, val) in &node.props {
        write_string(buf, key);
        write_value(buf, val);
    }
    write_u32(buf, node.children.len() as u32);
    for child in &node.children {
        write_node(buf, child);
    }
    write_u32(buf, node.handlers.len() as u32);
    for handler in &node.handlers {
        write_handler(buf, handler);
    }
    // Optional fields: flags byte
    let mut flags: u8 = 0;
    if node.condition.is_some() {
        flags |= 1;
    }
    if node.else_children.is_some() {
        flags |= 2;
    }
    if node.each_binding.is_some() {
        flags |= 4;
    }
    buf.push(flags);
    if let Some(cond) = &node.condition {
        write_expression(buf, cond);
    }
    if let Some(else_nodes) = &node.else_children {
        write_u32(buf, else_nodes.len() as u32);
        for child in else_nodes {
            write_node(buf, child);
        }
    }
    if let Some((var, expr)) = &node.each_binding {
        write_string(buf, var);
        write_expression(buf, expr);
    }
}

fn write_handler(buf: &mut Vec<u8>, handler: &IrEventHandler) {
    write_string(buf, &handler.event);
    write_action(buf, &handler.action);
    buf.push(handler.modifier_kind);
    write_u64(buf, handler.modifier_ms);
}

fn write_action(buf: &mut Vec<u8>, action: &IrAction) {
    match action {
        IrAction::Set { target, expr } => {
            buf.push(0);
            write_string(buf, target);
            write_expression(buf, expr);
        }
        IrAction::Navigate { path } => {
            buf.push(1);
            write_string(buf, path);
        }
        IrAction::ScrollTo { element_id } => {
            buf.push(2);
            write_string(buf, element_id);
        }
        IrAction::Log { expr } => {
            buf.push(3);
            write_expression(buf, expr);
        }
        IrAction::Trigger { data_name } => {
            buf.push(4);
            write_string(buf, data_name);
        }
        IrAction::Copy { expr } => {
            buf.push(5);
            write_expression(buf, expr);
        }
        IrAction::Send { stream_name, expr } => {
            buf.push(6);
            write_string(buf, stream_name);
            write_expression(buf, expr);
        }
        IrAction::JsCall {
            function_name,
            args,
            target,
        } => {
            buf.push(7);
            write_string(buf, function_name);
            write_u32(buf, args.len() as u32);
            for arg in args {
                write_expression(buf, arg);
            }
            match target {
                Some(t) => {
                    buf.push(1);
                    write_string(buf, t);
                }
                None => buf.push(0),
            }
        }
        IrAction::Notify { title, body, icon } => {
            buf.push(8);
            write_string(buf, title);
            write_string(buf, body);
            write_string(buf, icon);
        }
        IrAction::SetTheme { name } => {
            buf.push(9);
            write_string(buf, name);
        }
    }
}

fn write_expression(buf: &mut Vec<u8>, expr: &IrExpression) {
    match expr {
        IrExpression::Num(n) => {
            buf.push(0);
            write_f64(buf, *n);
        }
        IrExpression::Str(s) => {
            buf.push(1);
            write_string(buf, s);
        }
        IrExpression::Bool(b) => {
            buf.push(2);
            buf.push(if *b { 1 } else { 0 });
        }
        IrExpression::StateRef(name) => {
            buf.push(3);
            write_string(buf, name);
        }
        IrExpression::BinOp { left, op, right } => {
            buf.push(4);
            write_binop(buf, *op);
            write_expression(buf, left);
            write_expression(buf, right);
        }
        IrExpression::Pipeline { source, stages } => {
            buf.push(5);
            write_expression(buf, source);
            write_u32(buf, stages.len() as u32);
            for stage in stages {
                buf.push(stage.function);
                match &stage.argument {
                    Some(expr) => {
                        buf.push(1);
                        write_expression(buf, expr);
                    }
                    None => buf.push(0),
                }
                match &stage.argument2 {
                    Some(expr) => {
                        buf.push(1);
                        write_expression(buf, expr);
                    }
                    None => buf.push(0),
                }
            }
        }
        IrExpression::WasmCall {
            module,
            function,
            args,
        } => {
            buf.push(6);
            write_string(buf, module);
            write_string(buf, function);
            write_u32(buf, args.len() as u32);
            for arg in args {
                write_expression(buf, arg);
            }
        }
        IrExpression::EnvRef(name) => {
            buf.push(7);
            write_string(buf, name);
        }
    }
}

fn write_binop(buf: &mut Vec<u8>, op: IrBinOp) {
    buf.push(match op {
        IrBinOp::Add => 0,
        IrBinOp::Sub => 1,
        IrBinOp::Mul => 2,
        IrBinOp::Div => 3,
        IrBinOp::Eq => 4,
        IrBinOp::Neq => 5,
        IrBinOp::Gt => 6,
        IrBinOp::Lt => 7,
        IrBinOp::Gte => 8,
        IrBinOp::Lte => 9,
        IrBinOp::And => 10,
        IrBinOp::Or => 11,
    });
}

// ─── Reader ─────────────────────────────────────────────────────────────────

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.pos + n > self.data.len() {
            return Err("unexpected end of data".to_string());
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&mut self) -> Result<u64, String> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_f64(&mut self) -> Result<f64, String> {
        let bytes = self.read_bytes(8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_string(&mut self) -> Result<String, String> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| format!("invalid utf8: {}", e))
    }

    fn read_value(&mut self) -> Result<RenderValue, String> {
        let tag = self.read_u8()?;
        match tag {
            0 => Ok(RenderValue::Str(self.read_string()?)),
            1 => {
                let n = self.read_f64()?;
                let has_unit = self.read_u8()?;
                let unit = if has_unit != 0 {
                    Some(self.read_string()?)
                } else {
                    None
                };
                Ok(RenderValue::Num(n, unit))
            }
            2 => Ok(RenderValue::Color(self.read_u32()?)),
            3 => Ok(RenderValue::Bool(self.read_u8()? != 0)),
            4 => {
                let part_count = self.read_u32()? as usize;
                let mut parts = Vec::with_capacity(part_count);
                for _ in 0..part_count {
                    let part_tag = self.read_u8()?;
                    match part_tag {
                        0 => parts.push(TextPart::Literal(self.read_string()?)),
                        1 => parts.push(TextPart::StateRef(self.read_string()?)),
                        _ => return Err(format!("unknown text part tag: {}", part_tag)),
                    }
                }
                Ok(RenderValue::InterpolatedStr(parts))
            }
            5 => {
                let count = self.read_u32()? as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(self.read_value()?);
                }
                Ok(RenderValue::List(items))
            }
            6 => Ok(RenderValue::Bind(self.read_string()?)),
            7 => {
                let count = self.read_u32()? as usize;
                let mut entries = Vec::with_capacity(count);
                for _ in 0..count {
                    let key = self.read_string()?;
                    let value = self.read_value()?;
                    entries.push((key, value));
                }
                Ok(RenderValue::Object(entries))
            }
            _ => Err(format!("unknown value tag: {}", tag)),
        }
    }

    fn read_node(&mut self) -> Result<RenderNode, String> {
        let kind = self.read_string()?;
        let prop_count = self.read_u32()? as usize;
        let mut props = HashMap::with_capacity(prop_count);
        for _ in 0..prop_count {
            let key = self.read_string()?;
            let val = self.read_value()?;
            props.insert(key, val);
        }
        let child_count = self.read_u32()? as usize;
        let mut children = Vec::with_capacity(child_count);
        for _ in 0..child_count {
            children.push(self.read_node()?);
        }
        let handler_count = self.read_u32()? as usize;
        let mut handlers = Vec::with_capacity(handler_count);
        for _ in 0..handler_count {
            handlers.push(self.read_handler()?);
        }
        // Optional fields
        let flags = self.read_u8()?;
        let condition = if flags & 1 != 0 {
            Some(self.read_expression()?)
        } else {
            None
        };
        let else_children = if flags & 2 != 0 {
            let count = self.read_u32()? as usize;
            let mut nodes = Vec::with_capacity(count);
            for _ in 0..count {
                nodes.push(self.read_node()?);
            }
            Some(nodes)
        } else {
            None
        };
        let each_binding = if flags & 4 != 0 {
            let var = self.read_string()?;
            let expr = self.read_expression()?;
            Some((var, expr))
        } else {
            None
        };
        Ok(RenderNode {
            kind,
            props,
            children,
            handlers,
            span: None,
            condition,
            else_children,
            each_binding,
        })
    }

    fn read_handler(&mut self) -> Result<IrEventHandler, String> {
        let event = self.read_string()?;
        let action = self.read_action()?;
        let modifier_kind = self.read_u8()?;
        let modifier_ms = self.read_u64()?;
        Ok(IrEventHandler {
            event,
            action,
            modifier_kind,
            modifier_ms,
        })
    }

    fn read_action(&mut self) -> Result<IrAction, String> {
        let tag = self.read_u8()?;
        match tag {
            0 => {
                let target = self.read_string()?;
                let expr = self.read_expression()?;
                Ok(IrAction::Set { target, expr })
            }
            1 => {
                let path = self.read_string()?;
                Ok(IrAction::Navigate { path })
            }
            2 => {
                let element_id = self.read_string()?;
                Ok(IrAction::ScrollTo { element_id })
            }
            3 => {
                let expr = self.read_expression()?;
                Ok(IrAction::Log { expr })
            }
            4 => {
                let data_name = self.read_string()?;
                Ok(IrAction::Trigger { data_name })
            }
            5 => {
                let expr = self.read_expression()?;
                Ok(IrAction::Copy { expr })
            }
            6 => {
                let stream_name = self.read_string()?;
                let expr = self.read_expression()?;
                Ok(IrAction::Send { stream_name, expr })
            }
            7 => {
                let function_name = self.read_string()?;
                let arg_count = self.read_u32()? as usize;
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.read_expression()?);
                }
                let has_target = self.read_u8()?;
                let target = if has_target != 0 {
                    Some(self.read_string()?)
                } else {
                    None
                };
                Ok(IrAction::JsCall {
                    function_name,
                    args,
                    target,
                })
            }
            8 => {
                let title = self.read_string()?;
                let body = self.read_string()?;
                let icon = self.read_string()?;
                Ok(IrAction::Notify { title, body, icon })
            }
            9 => {
                let name = self.read_string()?;
                Ok(IrAction::SetTheme { name })
            }
            _ => Err(format!("unknown action tag: {}", tag)),
        }
    }

    fn read_expression(&mut self) -> Result<IrExpression, String> {
        let tag = self.read_u8()?;
        match tag {
            0 => Ok(IrExpression::Num(self.read_f64()?)),
            1 => Ok(IrExpression::Str(self.read_string()?)),
            2 => Ok(IrExpression::Bool(self.read_u8()? != 0)),
            3 => Ok(IrExpression::StateRef(self.read_string()?)),
            4 => {
                let op = self.read_binop()?;
                let left = self.read_expression()?;
                let right = self.read_expression()?;
                Ok(IrExpression::BinOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            }
            5 => {
                let source = self.read_expression()?;
                let stage_count = self.read_u32()? as usize;
                let mut stages = Vec::with_capacity(stage_count);
                for _ in 0..stage_count {
                    let function = self.read_u8()?;
                    let has_arg = self.read_u8()?;
                    let argument = if has_arg != 0 {
                        Some(self.read_expression()?)
                    } else {
                        None
                    };
                    let has_arg2 = self.read_u8()?;
                    let argument2 = if has_arg2 != 0 {
                        Some(self.read_expression()?)
                    } else {
                        None
                    };
                    stages.push(IrPipelineStage {
                        function,
                        argument,
                        argument2,
                    });
                }
                Ok(IrExpression::Pipeline {
                    source: Box::new(source),
                    stages,
                })
            }
            6 => {
                let module = self.read_string()?;
                let function = self.read_string()?;
                let arg_count = self.read_u32()? as usize;
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.read_expression()?);
                }
                Ok(IrExpression::WasmCall {
                    module,
                    function,
                    args,
                })
            }
            7 => Ok(IrExpression::EnvRef(self.read_string()?)),
            _ => Err(format!("unknown expression tag: {}", tag)),
        }
    }

    fn read_binop(&mut self) -> Result<IrBinOp, String> {
        let tag = self.read_u8()?;
        match tag {
            0 => Ok(IrBinOp::Add),
            1 => Ok(IrBinOp::Sub),
            2 => Ok(IrBinOp::Mul),
            3 => Ok(IrBinOp::Div),
            4 => Ok(IrBinOp::Eq),
            5 => Ok(IrBinOp::Neq),
            6 => Ok(IrBinOp::Gt),
            7 => Ok(IrBinOp::Lt),
            8 => Ok(IrBinOp::Gte),
            9 => Ok(IrBinOp::Lte),
            10 => Ok(IrBinOp::And),
            11 => Ok(IrBinOp::Or),
            _ => Err(format!("unknown binop tag: {}", tag)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simple() {
        let tree = RenderTree {
            title: "Hello".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![RenderNode {
                kind: "text".to_string(),
                props: {
                    let mut m = HashMap::new();
                    m.insert("__text".to_string(), RenderValue::Str("world".to_string()));
                    m
                },
                children: vec![],
                handlers: vec![],
                condition: None,
                else_children: None,
                each_binding: None,
                span: None,
            }],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn roundtrip_all_value_types() {
        let tree = RenderTree {
            title: "Test".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![RenderNode {
                kind: "rect".to_string(),
                props: {
                    let mut m = HashMap::new();
                    m.insert(
                        "width".to_string(),
                        RenderValue::Num(100.0, Some("px".to_string())),
                    );
                    m.insert("height".to_string(), RenderValue::Num(50.0, None));
                    m.insert("color".to_string(), RenderValue::Color(0xff0000));
                    m.insert("visible".to_string(), RenderValue::Bool(true));
                    m.insert("label".to_string(), RenderValue::Str("box".to_string()));
                    m.insert("bind".to_string(), RenderValue::Bind("someVar".to_string()));
                    m
                },
                children: vec![],
                handlers: vec![],
                condition: None,
                else_children: None,
                each_binding: None,
                span: None,
            }],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn roundtrip_checkbox_with_handler() {
        // Test checkbox-like scenario with bind prop and click handler
        let tree = RenderTree {
            title: "Checkbox Test".to_string(),
            state: vec![StateDecl {
                name: "agreed".to_string(),
                initial: RenderValue::Bool(false),
                shared: false,
            }],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![RenderNode {
                kind: "checkbox".to_string(),
                props: {
                    let mut m = HashMap::new();
                    m.insert(
                        "__text".to_string(),
                        RenderValue::Str("I agree".to_string()),
                    );
                    m.insert("bind".to_string(), RenderValue::Bind("agreed".to_string()));
                    m
                },
                children: vec![],
                handlers: vec![IrEventHandler {
                    event: "click".to_string(),
                    action: IrAction::Set {
                        target: "agreed".to_string(),
                        expr: IrExpression::BinOp {
                            left: Box::new(IrExpression::StateRef("agreed".to_string())),
                            op: IrBinOp::Eq,
                            right: Box::new(IrExpression::Bool(false)),
                        },
                    },
                    modifier_kind: 0,
                    modifier_ms: 0,
                }],
                condition: None,
                else_children: None,
                each_binding: None,
                span: None,
            }],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn roundtrip_nested() {
        let tree = RenderTree {
            title: "Nested".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![RenderNode {
                kind: "column".to_string(),
                props: HashMap::new(),
                children: vec![
                    RenderNode {
                        kind: "text".to_string(),
                        props: {
                            let mut m = HashMap::new();
                            m.insert("__text".to_string(), RenderValue::Str("hi".to_string()));
                            m
                        },
                        children: vec![],
                        handlers: vec![],
                        condition: None,
                        else_children: None,
                        each_binding: None,
                        span: None,
                    },
                    RenderNode {
                        kind: "rect".to_string(),
                        props: {
                            let mut m = HashMap::new();
                            m.insert("color".to_string(), RenderValue::Color(0x00ff00));
                            m
                        },
                        children: vec![],
                        handlers: vec![],
                        condition: None,
                        else_children: None,
                        each_binding: None,
                        span: None,
                    },
                ],
                handlers: vec![],
                condition: None,
                else_children: None,
                each_binding: None,
                span: None,
            }],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn roundtrip_overlay_node() {
        let tree = RenderTree {
            title: "Overlay Test".to_string(),
            state: vec![StateDecl {
                name: "dialog-open".to_string(),
                initial: RenderValue::Bool(false),
                shared: false,
            }],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![RenderNode {
                kind: "overlay".to_string(),
                props: {
                    let mut m = HashMap::new();
                    m.insert("focus-trap".to_string(), RenderValue::Bool(true));
                    m.insert("scroll-lock".to_string(), RenderValue::Bool(true));
                    m.insert(
                        "anchor".to_string(),
                        RenderValue::Str("menu-btn".to_string()),
                    );
                    m.insert(
                        "anchor-placement".to_string(),
                        RenderValue::Str("bottom".to_string()),
                    );
                    m
                },
                children: vec![RenderNode {
                    kind: "rect".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert(
                            "width".to_string(),
                            RenderValue::Num(480.0, Some("px".to_string())),
                        );
                        m.insert("color".to_string(), RenderValue::Color(0xffffff));
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                    span: None,
                }],
                handlers: vec![IrEventHandler {
                    event: "click-outside".to_string(),
                    action: IrAction::Set {
                        target: "dialog-open".to_string(),
                        expr: IrExpression::Bool(false),
                    },
                    modifier_kind: 0,
                    modifier_ms: 0,
                }],
                condition: None,
                else_children: None,
                each_binding: None,
                span: None,
            }],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn roundtrip_with_pages() {
        let tree = RenderTree {
            title: "Multi-Page App".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![
                PageDef {
                    path: "/".to_string(),
                    params: vec![],
                    is_catch_all: false,
                    guard: None,
                    meta: vec![],
                    root: vec![RenderNode {
                        kind: "text".to_string(),
                        props: {
                            let mut m = HashMap::new();
                            m.insert("__text".to_string(), RenderValue::Str("Home".to_string()));
                            m
                        },
                        children: vec![],
                        handlers: vec![],
                        condition: None,
                        else_children: None,
                        each_binding: None,
                        span: None,
                    }],
                },
                PageDef {
                    path: "/about".to_string(),
                    params: vec![],
                    is_catch_all: false,
                    guard: None,
                    meta: vec![],
                    root: vec![RenderNode {
                        kind: "text".to_string(),
                        props: {
                            let mut m = HashMap::new();
                            m.insert("__text".to_string(), RenderValue::Str("About".to_string()));
                            m
                        },
                        children: vec![],
                        handlers: vec![],
                        condition: None,
                        else_children: None,
                        each_binding: None,
                        span: None,
                    }],
                },
            ],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }

    #[test]
    fn roundtrip_computed_decls() {
        let tree = RenderTree {
            title: "Computed Test".to_string(),
            state: vec![
                StateDecl {
                    name: "count".to_string(),
                    initial: RenderValue::Num(1.0, None),
                    shared: false,
                },
                StateDecl {
                    name: "price".to_string(),
                    initial: RenderValue::Num(10.0, None),
                    shared: false,
                },
            ],
            data: vec![],
            computed: vec![
                ComputedDecl {
                    name: "total".to_string(),
                    expr: IrExpression::BinOp {
                        left: Box::new(IrExpression::StateRef("count".to_string())),
                        op: IrBinOp::Mul,
                        right: Box::new(IrExpression::StateRef("price".to_string())),
                    },
                },
                ComputedDecl {
                    name: "doubled".to_string(),
                    expr: IrExpression::BinOp {
                        left: Box::new(IrExpression::StateRef("total".to_string())),
                        op: IrBinOp::Mul,
                        right: Box::new(IrExpression::Num(2.0)),
                    },
                },
            ],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![RenderNode {
                kind: "text".to_string(),
                props: HashMap::new(),
                children: vec![],
                handlers: vec![],
                condition: None,
                else_children: None,
                each_binding: None,
                span: None,
            }],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.computed.len(), 2);
        assert_eq!(restored.computed[0].name, "total");
        assert_eq!(restored.computed[1].name, "doubled");
    }

    #[test]
    fn roundtrip_storage_decls() {
        let tree = RenderTree {
            title: "Storage Test".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![
                StorageDecl {
                    name: "theme".to_string(),
                    storage_type: 0,
                    key: "theme-pref".to_string(),
                    default: RenderValue::Str("light".to_string()),
                },
                StorageDecl {
                    name: "token".to_string(),
                    storage_type: 1,
                    key: "auth-token".to_string(),
                    default: RenderValue::Str(String::new()),
                },
            ],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.storage.len(), 2);
        assert_eq!(restored.storage[0].name, "theme");
        assert_eq!(restored.storage[0].storage_type, 0);
        assert_eq!(restored.storage[1].storage_type, 1);
    }

    #[test]
    fn roundtrip_pipeline_expression() {
        let tree = RenderTree {
            title: "Pipeline Test".to_string(),
            state: vec![StateDecl {
                name: "items".to_string(),
                initial: RenderValue::List(vec![
                    RenderValue::Num(1.0, None),
                    RenderValue::Num(2.0, None),
                    RenderValue::Num(3.0, None),
                ]),
                shared: false,
            }],
            data: vec![],
            computed: vec![
                ComputedDecl {
                    name: "total".to_string(),
                    expr: IrExpression::Pipeline {
                        source: Box::new(IrExpression::StateRef("items".to_string())),
                        stages: vec![
                            IrPipelineStage {
                                function: 0, // filter
                                argument: Some(IrExpression::BinOp {
                                    left: Box::new(IrExpression::StateRef("score".to_string())),
                                    op: IrBinOp::Gt,
                                    right: Box::new(IrExpression::Num(60.0)),
                                }),
                                argument2: None,
                            },
                            IrPipelineStage {
                                function: 1, // map
                                argument: Some(IrExpression::StateRef("price".to_string())),
                                argument2: None,
                            },
                            IrPipelineStage {
                                function: 4, // sum
                                argument: None,
                                argument2: None,
                            },
                        ],
                    },
                },
                ComputedDecl {
                    name: "count".to_string(),
                    expr: IrExpression::Pipeline {
                        source: Box::new(IrExpression::StateRef("items".to_string())),
                        stages: vec![IrPipelineStage {
                            function: 5, // count
                            argument: None,
                            argument2: None,
                        }],
                    },
                },
            ],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.computed.len(), 2);
        // Verify pipeline structure survived roundtrip
        match &restored.computed[0].expr {
            IrExpression::Pipeline { source, stages } => {
                assert!(matches!(**source, IrExpression::StateRef(ref s) if s == "items"));
                assert_eq!(stages.len(), 3);
                assert_eq!(stages[0].function, 0); // filter
                assert!(stages[0].argument.is_some());
                assert_eq!(stages[1].function, 1); // map
                assert!(stages[1].argument.is_some());
                assert_eq!(stages[2].function, 4); // sum
                assert!(stages[2].argument.is_none());
            }
            other => panic!("expected Pipeline, got {:?}", other),
        }
    }

    #[test]
    fn roundtrip_wasm_call_and_imports() {
        let tree = RenderTree {
            title: "WasmTest".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![ComputedDecl {
                name: "result".to_string(),
                expr: IrExpression::WasmCall {
                    module: "crypto".to_string(),
                    function: "sha256".to_string(),
                    args: vec![IrExpression::StateRef("input".to_string())],
                },
            }],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![],
            themes: vec![],
            imports: vec![
                ImportDecl {
                    name: "crypto".to_string(),
                    wasm_url: "crypto.wasm".to_string(),
                    functions: vec!["sha256".to_string(), "md5".to_string()],
                },
            ],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.imports.len(), 1);
        assert_eq!(restored.imports[0].name, "crypto");
        assert_eq!(restored.imports[0].wasm_url, "crypto.wasm");
        assert_eq!(restored.imports[0].functions, vec!["sha256", "md5"]);
        match &restored.computed[0].expr {
            IrExpression::WasmCall { module, function, args } => {
                assert_eq!(module, "crypto");
                assert_eq!(function, "sha256");
                assert_eq!(args.len(), 1);
            }
            other => panic!("expected WasmCall, got {:?}", other),
        }
    }

    #[test]
    fn roundtrip_server_functions_and_calls() {
        let tree = RenderTree {
            title: "ServerTest".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![
                ServerFuncDecl {
                    name: "add".to_string(),
                    params: vec!["x".to_string(), "y".to_string()],
                    body: IrServerBody {
                        lets: vec![],
                        result: IrExpression::BinOp {
                            left: Box::new(IrExpression::StateRef("x".to_string())),
                            op: IrBinOp::Add,
                            right: Box::new(IrExpression::StateRef("y".to_string())),
                        },
                    },
                },
                ServerFuncDecl {
                    name: "get-config".to_string(),
                    params: vec![],
                    body: IrServerBody {
                        lets: vec![],
                        result: IrExpression::Str("hello".to_string()),
                    },
                },
            ],
            server_calls: vec![ServerCallDecl {
                name: "result".to_string(),
                func_name: "add".to_string(),
                args: vec![IrExpression::Num(1.0), IrExpression::Num(2.0)],
            }],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.server_functions.len(), 2);
        assert_eq!(restored.server_functions[0].name, "add");
        assert_eq!(restored.server_functions[0].params, vec!["x", "y"]);
        assert_eq!(restored.server_functions[1].name, "get-config");
        assert_eq!(restored.server_functions[1].params.len(), 0);
        assert_eq!(restored.server_calls.len(), 1);
        assert_eq!(restored.server_calls[0].name, "result");
        assert_eq!(restored.server_calls[0].func_name, "add");
        assert_eq!(restored.server_calls[0].args.len(), 2);
    }

    #[test]
    fn roundtrip_prompt_decls() {
        let tree = RenderTree {
            title: "AI App".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            guards: vec![],
            prompts: vec![
                PromptDecl {
                    name: "summary".to_string(),
                    provider: "openai".to_string(),
                    system: "Summarize concisely.".to_string(),
                    user: "Tell me about {topic}".to_string(),
                    model: "gpt-4o".to_string(),
                    max_tokens: 500,
                    temperature: 0.7,
                },
                PromptDecl {
                    name: "reply".to_string(),
                    provider: "anthropic".to_string(),
                    system: "Be helpful.".to_string(),
                    user: "Hello".to_string(),
                    model: "claude-sonnet-4-5-20250929".to_string(),
                    max_tokens: 1000,
                    temperature: 0.5,
                },
            ],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.prompts.len(), 2);
        assert_eq!(restored.prompts[0].name, "summary");
        assert_eq!(restored.prompts[0].provider, "openai");
        assert_eq!(restored.prompts[0].system, "Summarize concisely.");
        assert_eq!(restored.prompts[0].max_tokens, 500);
        assert_eq!(restored.prompts[0].temperature, 0.7);
        assert_eq!(restored.prompts[1].name, "reply");
        assert_eq!(restored.prompts[1].provider, "anthropic");
    }

    #[test]
    fn roundtrip_env_ref_expression() {
        let tree = RenderTree {
            title: "Env Test".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![RenderNode {
                kind: "text".to_string(),
                props: HashMap::new(),
                children: vec![],
                handlers: vec![],
                condition: Some(IrExpression::EnvRef("API_URL".to_string())),
                else_children: None,
                each_binding: None,
                span: None,
            }],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![
                ServerFuncDecl {
                    name: "get-data".to_string(),
                    params: vec![],
                    body: IrServerBody {
                        lets: vec![],
                        result: IrExpression::EnvRef("SECRET_KEY".to_string()),
                    },
                },
            ],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(
            restored.root[0].condition,
            Some(IrExpression::EnvRef("API_URL".to_string()))
        );
        assert_eq!(
            restored.server_functions[0].body.result,
            IrExpression::EnvRef("SECRET_KEY".to_string())
        );
    }

    #[test]
    fn roundtrip_dynamic_pages() {
        let tree = RenderTree {
            title: "Dynamic Routes".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![
                PageDef {
                    path: "/posts/:id".to_string(),
                    params: vec!["id".to_string()],
                    is_catch_all: false,
                    guard: None,
                    meta: vec![],
                    root: vec![],
                },
                PageDef {
                    path: "/users/:userId/posts/:postId".to_string(),
                    params: vec!["userId".to_string(), "postId".to_string()],
                    is_catch_all: false,
                    guard: None,
                    meta: vec![],
                    root: vec![],
                },
                PageDef {
                    path: "/*".to_string(),
                    params: vec![],
                    is_catch_all: true,
                    guard: None,
                    meta: vec![],
                    root: vec![],
                },
            ],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.pages.len(), 3);
        assert_eq!(restored.pages[0].params, vec!["id"]);
        assert!(!restored.pages[0].is_catch_all);
        assert_eq!(restored.pages[1].params, vec!["userId", "postId"]);
        assert!(restored.pages[2].is_catch_all);
    }

    #[test]
    fn roundtrip_page_meta() {
        let tree = RenderTree {
            title: "Meta App".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![
                PageDef {
                    path: "/about".to_string(),
                    params: vec![],
                    is_catch_all: false,
                    guard: None,
                    meta: vec![
                        ("title".to_string(), RenderValue::Str("About Us".to_string())),
                        ("description".to_string(), RenderValue::Str("Our company".to_string())),
                    ],
                    root: vec![],
                },
            ],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.pages[0].meta.len(), 2);
        assert_eq!(restored.pages[0].meta[0].0, "title");
        assert_eq!(
            restored.pages[0].meta[0].1,
            RenderValue::Str("About Us".to_string())
        );
        assert_eq!(restored.pages[0].meta[1].0, "description");
    }

    #[test]
    fn roundtrip_guards() {
        let tree = RenderTree {
            title: "Guard App".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![
                PageDef {
                    path: "/admin".to_string(),
                    params: vec![],
                    is_catch_all: false,
                    guard: Some("is-admin".to_string()),
                    meta: vec![],
                    root: vec![],
                },
                PageDef {
                    path: "/public".to_string(),
                    params: vec![],
                    is_catch_all: false,
                    guard: None,
                    meta: vec![],
                    root: vec![],
                },
            ],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            guards: vec![
                GuardDef {
                    name: "is-admin".to_string(),
                    checks: vec![
                        GuardCheck {
                            condition: IrExpression::StateRef("auth-token".to_string()),
                            redirect: "/login".to_string(),
                        },
                    ],
                },
            ],
            prompts: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.guards.len(), 1);
        assert_eq!(restored.guards[0].name, "is-admin");
        assert_eq!(restored.guards[0].checks.len(), 1);
        assert_eq!(restored.guards[0].checks[0].redirect, "/login");
        assert_eq!(restored.pages[0].guard, Some("is-admin".to_string()));
        assert!(restored.pages[1].guard.is_none());
    }

    #[test]
    fn roundtrip_data_headers() {
        let tree = RenderTree {
            title: "Auth App".to_string(),
            state: vec![],
            data: vec![DataDecl {
                name: "users".to_string(),
                url: "/api/users".to_string(),
                source_type: 0,
                method: "get".to_string(),
                cache_ms: 0,
                retry_count: 0,
                trigger_mode: 0,
                content_type: String::new(),
                watch: false,
                headers: vec![
                    ("Authorization".to_string(), RenderValue::Str("Bearer {auth-token}".to_string())),
                    ("X-Api-Key".to_string(), RenderValue::Str("my-key".to_string())),
                ],
            }],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            guards: vec![],
            prompts: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.data[0].headers.len(), 2);
        assert_eq!(restored.data[0].headers[0].0, "Authorization");
        assert_eq!(
            restored.data[0].headers[0].1,
            RenderValue::Str("Bearer {auth-token}".to_string())
        );
        assert_eq!(restored.data[0].headers[1].0, "X-Api-Key");
    }

    #[test]
    fn roundtrip_server_sql_step() {
        let tree = RenderTree {
            title: "DB App".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![ServerFuncDecl {
                name: "get-users".to_string(),
                params: vec!["limit".to_string()],
                body: IrServerBody {
                    lets: vec![
                        (
                            "rows".to_string(),
                            IrServerStep::Sql {
                                query: "SELECT id, name FROM users LIMIT $1".to_string(),
                                params: vec![IrExpression::StateRef("limit".to_string())],
                            },
                        ),
                    ],
                    result: IrExpression::StateRef("rows".to_string()),
                },
            }],
            server_calls: vec![],
            guards: vec![],
            prompts: vec![],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
        assert_eq!(restored.server_functions.len(), 1);
        match &restored.server_functions[0].body.lets[0].1 {
            IrServerStep::Sql { query, params } => {
                assert_eq!(query, "SELECT id, name FROM users LIMIT $1");
                assert_eq!(params.len(), 1);
            }
            _ => panic!("expected IrServerStep::Sql"),
        }
    }
}
