//! Intermediate representation types shared between the compiler and runtime.
//! Uses a simple custom binary format to minimize WASM size (no serde in WASM).

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A property value in the render tree, stripped of AST-specific details.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RenderValue {
    Str(String),
    Num(f64, Option<String>), // value + optional unit ("px", "%", "em")
    Color(u32),
    Bool(bool),
}

/// A node in the flattened render tree.
/// Components have been inlined — only built-in elements remain.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderNode {
    pub kind: String,
    pub props: HashMap<String, RenderValue>,
    pub children: Vec<RenderNode>,
}

/// The serializable render tree for the entire app.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderTree {
    pub title: String,
    pub root: Vec<RenderNode>,
}

// ─── Simple binary encoding ─────────────────────────────────────────────────
// Format:
//   String: u32 len + utf8 bytes
//   RenderValue: u8 tag + payload
//     0 = Str(String)
//     1 = Num(f64, Option<String>)
//     2 = Color(u32)
//     3 = Bool(bool)
//   RenderNode: String kind + u32 prop_count + props + u32 child_count + children
//   RenderTree: String title + u32 root_count + root nodes

/// Serialize a RenderTree to compact binary bytes.
pub fn serialize(tree: &RenderTree) -> Vec<u8> {
    let mut buf = Vec::new();
    write_string(&mut buf, &tree.title);
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
    let count = cursor.read_u32()? as usize;
    let mut root = Vec::with_capacity(count);
    for _ in 0..count {
        root.push(cursor.read_node()?);
    }
    Ok(RenderTree { title, root })
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
        Ok(RenderNode {
            kind,
            props,
            children,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simple() {
        let tree = RenderTree {
            title: "Hello".to_string(),
            root: vec![RenderNode {
                kind: "text".to_string(),
                props: {
                    let mut m = HashMap::new();
                    m.insert("__text".to_string(), RenderValue::Str("world".to_string()));
                    m
                },
                children: vec![],
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
                    },
                    RenderNode {
                        kind: "rect".to_string(),
                        props: {
                            let mut m = HashMap::new();
                            m.insert("color".to_string(), RenderValue::Color(0x00ff00));
                            m
                        },
                        children: vec![],
                    },
                ],
            }],
        };
        let bytes = serialize(&tree);
        let restored = deserialize(&bytes).unwrap();
        assert_eq!(tree, restored);
    }
}
