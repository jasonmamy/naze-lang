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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderNode {
    pub kind: String,
    pub props: HashMap<String, RenderValue>,
    pub children: Vec<RenderNode>,
    pub handlers: Vec<IrEventHandler>,
    pub condition: Option<IrExpression>,
    pub else_children: Option<Vec<RenderNode>>,
    pub each_binding: Option<(String, IrExpression)>,
}

/// A page definition with path and content.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PageDef {
    pub path: String,
    pub root: Vec<RenderNode>,
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
        write_u32(&mut buf, page.root.len() as u32);
        for node in &page.root {
            write_node(&mut buf, node);
        }
    }
    buf
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
            let node_count = cursor.read_u32()? as usize;
            let mut page_root = Vec::with_capacity(node_count);
            for _ in 0..node_count {
                page_root.push(cursor.read_node()?);
            }
            pages.push(PageDef {
                path,
                root: page_root,
            });
        }
        pages
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
            }],
            pages: vec![],
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
            }],
            pages: vec![],
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
            }],
            pages: vec![],
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
                    },
                ],
                handlers: vec![],
                condition: None,
                else_children: None,
                each_binding: None,
            }],
            pages: vec![],
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
            }],
            pages: vec![],
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
                    }],
                },
                PageDef {
                    path: "/about".to_string(),
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
                    }],
                },
            ],
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
            }],
            pages: vec![],
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
}
