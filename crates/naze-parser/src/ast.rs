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
    Comment(String),
}

/// An event handler attached to the parent element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    pub event: String,
    pub action: Action,
    pub span: Span,
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
