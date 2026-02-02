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
        span: Span,
    },
    Comment(String),
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
    Num(f64, Option<Unit>),
    Color(u32),
    Bool(bool),
    Ref(Vec<String>),
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
