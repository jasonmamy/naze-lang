use serde::{Deserialize, Serialize};

/// Source location for error reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub offset: usize,
    pub len: usize,
}

/// Top-level AST node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    App {
        title: String,
        children: Vec<Node>,
        span: Span,
    },
    Component {
        name: String,
        params: Vec<Param>,
        children: Vec<Node>,
        span: Span,
    },
    UseStmt {
        path: Vec<String>,
        span: Span,
    },
    Element {
        name: String,
        props: Vec<Prop>,
        children: Vec<Node>,
        handlers: Vec<EventHandler>,
        span: Span,
    },
    Let {
        name: String,
        value: Value,
        span: Span,
    },
    State {
        name: String,
        value: Value,
        shared: bool,
        span: Span,
    },
    Data {
        name: String,
        url: String,
        source: DataSource,
        config: DataConfig,
        span: Span,
    },
    Computed {
        name: String,
        expr: Expression,
        span: Span,
    },
    Storage {
        name: String,
        storage_type: StorageType,
        key: String,
        default: Value,
        span: Span,
    },
    Timer {
        name: String,
        kind: TimerKind,
        duration_ms: u64,
        action: Action,
        span: Span,
    },
    Param {
        name: String,
        ty: Type,
        default: Value,
        span: Span,
    },
    If {
        condition: Expression,
        then_children: Vec<Node>,
        else_children: Vec<Node>, // empty if no else clause
        span: Span,
    },
    Each {
        variable: String,
        iterable: Expression, // StateRef to a list variable
        children: Vec<Node>,
        span: Span,
    },
    Slot {
        name: Option<String>,       // None = default slot, Some("x") = named slot
        default_children: Vec<Node>, // fallback content if caller provides nothing
        span: Span,
    },
    Fill {
        name: String,      // slot name to fill
        children: Vec<Node>,
        span: Span,
    },
    Theme {
        colors: Vec<(String, u32)>,            // "primary" -> 0x2563eb
        spacing: Vec<(String, f64, Option<Unit>)>, // "md" -> (16.0, Some(Px))
        span: Span,
    },
    Page {
        path: String,           // URL path like "/" or "/about"
        children: Vec<Node>,
        span: Span,
    },
    Link {
        text: Value,            // Link text (may be interpolated)
        to: String,             // Target path
        children: Vec<Node>,    // Optional nested elements
        span: Span,
    },
    Comment(String),
}

/// An event handler attached to the parent element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    pub event: String,
    pub action: Action,
    pub modifier: Option<EventModifier>,
    pub span: Span,
}

/// Event handler modifier (debounce or throttle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventModifier {
    pub kind: ModifierKind,
    pub duration_ms: u64,
}

/// Kind of event modifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifierKind {
    Debounce,
    Throttle,
}

/// Configuration for enhanced data fetch declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub method: Option<String>,         // "get", "post", "put", "delete", "patch"
    pub headers: Vec<(String, String)>, // static headers
    pub body: Option<Value>,            // request body
    pub cache_ms: Option<u64>,          // cache TTL in milliseconds
    pub retry: Option<u32>,             // retry count
    pub trigger: Option<String>,        // "auto" (default) or "manual"
    pub content_type: Option<String>,   // e.g. "application/json"
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            method: None,
            headers: vec![],
            body: None,
            cache_ms: None,
            retry: None,
            trigger: None,
            content_type: None,
        }
    }
}

/// Storage type for persistent state declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    Local,
    Session,
}

/// Data source type: HTTP fetch or real-time stream (WebSocket/SSE).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    Fetch,
    Stream,
}

/// Timer kind: one-shot (after) or repeating (every).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerKind {
    After,
    Every,
}

/// An action triggered by an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Set {
        target: String,
        expr: Expression,
        span: Span,
    },
    Navigate {
        path: String,
        span: Span,
    },
    ScrollTo {
        element_id: String,
        span: Span,
    },
    Log {
        expr: Expression,
        span: Span,
    },
    Trigger {
        data_name: String,
        span: Span,
    },
    Copy {
        expr: Expression,
        span: Span,
    },
    Send {
        stream_name: String,
        expr: Expression,
        span: Span,
    },
}

/// An expression used in event handler actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Literal(Value),
    StateRef(String),
    BinOp {
        left: Box<Expression>,
        op: BinOp,
        right: Box<Expression>,
    },
}

/// Binary operators.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BinOp {
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

/// A segment of an interpolated string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringPart {
    Literal(String),
    Interpolation(Vec<String>), // ref path segments, e.g. ["count"] or ["theme", "primary"]
}

/// Component parameter declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub default: Option<Value>,
}

/// Property on an element or component invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prop {
    pub key: String,
    pub value: Value,
}

/// Value types in the language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Str(String),
    InterpolatedStr(Vec<StringPart>),
    Num(f64, Option<Unit>),
    Color(u32),
    Bool(bool),
    Ref(Vec<String>),
    List(Vec<Value>),
    Object(Vec<(String, Value)>), // Object literal: { key: value, ... }
    Bind(String), // Two-way state binding for form elements
}

/// Dimension units.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Unit {
    Px,
    Percent,
    Em,
}

/// Type annotations for component params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    Text,
    Number,
    Bool,
    Color,
}
