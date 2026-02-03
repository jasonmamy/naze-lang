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
}

/// A state variable declaration with its initial value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StateDecl {
    pub name: String,
    pub initial: RenderValue,
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
}

/// An action triggered by an event handler.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IrAction {
    Set { target: String, expr: IrExpression },
    Navigate { path: String },
}

/// An event handler on a render node.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IrEventHandler {
    pub event: String,
    pub action: IrAction,
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

/// The serializable render tree for the entire app.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderTree {
    pub title: String,
    pub root: Vec<RenderNode>,
    pub state: Vec<StateDecl>,
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
    }
    write_u32(&mut buf, tree.root.len() as u32);
    for node in &tree.root {
        write_node(&mut buf, node);
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
        state.push(StateDecl { name, initial });
    }
    let count = cursor.read_u32()? as usize;
    let mut root = Vec::with_capacity(count);
    for _ in 0..count {
        root.push(cursor.read_node()?);
    }
    Ok(RenderTree { title, root, state })
}

// ─── Writer ─────────────────────────────────────────────────────────────────

fn write_u32(buf: &mut Vec<u8>, val: u32) {
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
        Ok(IrEventHandler { event, action })
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
            root: vec![RenderNode {
                kind: "rect".to_string(),
                props: {
                    let mut m = HashMap::new();
                    m.insert("width".to_string(), RenderValue::Num(100.0, Some("px".to_string())));
                    m.insert("height".to_string(), RenderValue::Num(50.0, None));
                    m.insert("color".to_string(), RenderValue::Color(0xff0000));
                    m.insert("visible".to_string(), RenderValue::Bool(true));
                    m.insert("label".to_string(), RenderValue::Str("box".to_string()));
                    m
                },
                children: vec![],
                handlers: vec![],
                condition: None,
                else_children: None,
                each_binding: None,
            }],
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
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }
}
