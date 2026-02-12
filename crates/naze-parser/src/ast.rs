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
    Template {
        name: String,
        slots: Vec<String>,
        children: Vec<Node>,
        span: Span,
    },
    UseStmt {
        path: Vec<String>,
        span: Span,
    },
    Import {
        name: String,   // local binding name: "crypto"
        source: String, // source path: "./lib/crypto.wasm" or "@naze/crypto"
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
    Boundary {
        children: Vec<Node>,       // Normal content (may contain data declarations)
        catch_children: Vec<Node>, // Fallback content shown when data fails
        span: Span,
    },
    Each {
        variable: String,
        iterable: Expression, // StateRef to a list variable
        children: Vec<Node>,
        span: Span,
    },
    Slot {
        name: Option<String>,        // None = default slot, Some("x") = named slot
        default_children: Vec<Node>, // fallback content if caller provides nothing
        span: Span,
    },
    Fill {
        name: String, // slot name to fill
        children: Vec<Node>,
        span: Span,
    },
    Theme {
        name: Option<String>,                      // None = unnamed (default), Some("dark") = named
        extends: Option<String>,                    // parent theme name for inheritance
        colors: Vec<(String, u32)>,                // "primary" -> 0x2563eb
        spacing: Vec<(String, f64, Option<Unit>)>, // "md" -> (16.0, Some(Px))
        span: Span,
    },
    Page {
        path: String,          // URL path like "/" or "/posts/:id"
        params: Vec<String>,   // Extracted param names (e.g., ["id"] from "/posts/:id")
        guard: Option<String>, // Optional guard name (e.g., "is-admin")
        children: Vec<Node>,
        span: Span,
    },
    Guard {
        name: String,
        checks: Vec<GuardCheckAst>, // condition + redirect pairs
        span: Span,
    },
    Model {
        name: String,
        fields: Vec<ModelField>,
        span: Span,
    },
    Link {
        text: Value,         // Link text (may be interpolated)
        to: String,          // Target path
        children: Vec<Node>, // Optional nested elements
        span: Span,
    },
    Function {
        name: String,
        params: Vec<FuncParam>,
        return_type: Type,
        body: Expression,
        span: Span,
    },
    ServerFunction {
        name: String,
        params: Vec<FuncParam>,
        body: ServerBody,
        span: Span,
    },
    ServerData {
        name: String,      // data binding name: "user"
        func_name: String, // server function name: "get-user"
        args: Vec<Expression>,
        span: Span,
    },
    Prompt {
        name: String,                  // binding name: "summary"
        provider: String,              // provider name: "openai", "anthropic", "ollama"
        props: Vec<(String, Value)>,   // key-value properties from block
        span: Span,
    },
    Meta {
        key: String,        // "title", "description", "image", "canonical", "robots"
        value: Value,
        span: Span,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataConfig {
    pub method: Option<String>, // "get", "post", "put", "delete", "patch"
    pub headers: Vec<(String, Value)>, // headers with interpolation support
    pub body: Option<Value>,    // request body
    pub cache_ms: Option<u64>,  // cache TTL in milliseconds
    pub retry: Option<u32>,     // retry count
    pub trigger: Option<String>, // "auto" (default) or "manual"
    pub content_type: Option<String>, // e.g. "application/json"
    pub watch: bool,                  // for device APIs: continuously watch vs one-shot
}

/// Storage type for persistent state declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    Local,
    Session,
}

/// Data source type: HTTP fetch, real-time stream (WebSocket/SSE), JS function call, or device API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    Fetch,
    Stream,
    JsCall,
    Device,
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
    JsCall {
        function_name: String,
        args: Vec<Expression>,
        target: Option<String>, // state var to store return value
        span: Span,
    },
    Notify {
        title: String,
        body: Option<String>,
        icon: Option<String>,
        span: Span,
    },
    Emit {
        event_name: String,
        span: Span,
    },
    SetTheme {
        theme_name: String,
        span: Span,
    },
}

/// Pipeline function identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineFn {
    Filter,
    Map,
    SortBy,
    Take,
    Sum,
    Count,
    Reduce,
    GroupBy,
    Flatten,
    Distinct,
}

/// A single stage in a pipeline expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub function: PipelineFn,
    pub argument: Option<Expression>,
    pub argument2: Option<Expression>,
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
    Pipeline {
        source: Box<Expression>,
        stages: Vec<PipelineStage>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
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

/// Function parameter (name: type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncParam {
    pub name: String,
    pub ty: Type,
}

/// Server function body: a sequence of let bindings + a final result expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerBody {
    pub lets: Vec<(String, ServerExpr)>,
    pub result: Expression,
}

/// A single guard check: condition + redirect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardCheckAst {
    pub condition: Expression,
    pub redirect: String,
}

/// A field in a model definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelField {
    pub name: String,
    pub field_type: String,       // "number", "text", "bool", "timestamp"
    pub constraints: Vec<String>, // "primary", "unique", "default:now", etc.
}

/// A single condition in a where clause: field op value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCondition {
    pub field: String,
    pub op: String,        // "==", "!=", ">", "<", ">=", "<="
    pub value: Expression,
}

/// Expression types allowed in server function let bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerExpr {
    Fetch(String),                            // fetch "url"
    Sql { query: String, params: Vec<Expression> }, // sql "SELECT ..." [param1, param2]
    Expr(Expression),                         // any regular expression
    Find {
        model: String,
        conditions: Vec<QueryCondition>,
        order: Option<(String, bool)>,  // (field_name, ascending)
        limit: Option<Expression>,
    },
    Insert {
        model: String,
        fields: Vec<(String, Value)>,
    },
    Update {
        model: String,
        set_fields: Vec<(String, Value)>,
        conditions: Vec<QueryCondition>,
    },
    Delete {
        model: String,
        conditions: Vec<QueryCondition>,
    },
}

/// A single arm in a match expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub children: Vec<Node>,
}

/// Pattern in a match arm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchPattern {
    Wildcard,
    StringLit(String),
    NumberLit(f64),
    BoolLit(bool),
    Ident(String),
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
    Bind(String),                 // Two-way state binding for form elements
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

// ─── Test file AST types ─────────────────────────────────────────────────────

/// A parsed `.test.naze` file containing test and flow blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFile {
    pub uses: Vec<String>,
    pub tests: Vec<TestBlock>,
    pub flows: Vec<FlowBlock>,
}

/// A single test block: `test "description" { steps }`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestBlock {
    pub name: String,
    pub steps: Vec<TestStep>,
    pub span: Span,
}

/// A flow block: `flow "description" { steps }`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowBlock {
    pub name: String,
    pub steps: Vec<TestStep>,
    pub span: Span,
}

/// A single step in a test or flow block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStep {
    Render {
        component: String,
        props: Vec<Prop>,
        span: Span,
    },
    Click {
        text: String,
        span: Span,
    },
    Fill {
        target: String,
        value: String,
        span: Span,
    },
    Navigate {
        path: String,
        span: Span,
    },
    Wait {
        duration_ms: u64,
        span: Span,
    },
    Assert {
        kind: AssertKind,
        span: Span,
    },
}

/// The different assertion types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssertKind {
    TextVisible(String),
    TextNotVisible(String),
    PageIs(String),
    StateIs { name: String, value: Value },
    Emitted(String),
    NoA11yViolations,
}
