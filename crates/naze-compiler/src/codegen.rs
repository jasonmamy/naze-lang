use std::collections::HashMap;

use naze_parser::ast::{Node, Prop, Unit, Value};

use crate::resolve::{ComponentDef, ResolvedProject};

// Re-export IR types so existing consumers can use `naze_compiler::codegen::*`
pub use naze_ir::{RenderNode, RenderTree, RenderValue};

/// Lower a resolved project into a flattened RenderTree.
/// All component invocations are inlined with prop substitution.
pub fn lower(project: &ResolvedProject) -> RenderTree {
    let by_name: HashMap<&str, &ComponentDef> = project
        .components
        .values()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let mut title = String::new();
    let mut root = Vec::new();

    for node in &project.entry.nodes {
        match node {
            Node::App {
                title: t,
                children,
                ..
            } => {
                title = t.clone();
                root = lower_nodes(children, &by_name, &HashMap::new());
            }
            // Skip use statements, comments, component defs at top level
            _ => {}
        }
    }

    RenderTree { title, root }
}

// Re-export serialize/deserialize from naze-ir
pub use naze_ir::{deserialize, serialize};

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
                ..
            } => {
                if let Some(comp) = components.get(name.as_str()) {
                    // Component invocation: build substitution scope and inline body
                    out.extend(inline_component(comp, props, components, scope));
                } else {
                    // Built-in element
                    let resolved_props = resolve_props(props, scope);
                    let child_nodes = lower_nodes(children, components, scope);
                    out.push(RenderNode {
                        kind: name.clone(),
                        props: resolved_props,
                        children: child_nodes,
                    });
                }
            }
            Node::Comment(_) | Node::UseStmt { .. } | Node::Component { .. } => {
                // Skip non-renderable nodes
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
fn inline_component(
    comp: &ComponentDef,
    call_props: &[Prop],
    components: &HashMap<&str, &ComponentDef>,
    parent_scope: &HashMap<String, RenderValue>,
) -> Vec<RenderNode> {
    // Build scope: start with defaults, then overlay call-site values
    let mut scope = parent_scope.clone();

    // Apply defaults first
    for param in &comp.params {
        if let Some(default) = &param.default {
            scope.insert(param.name.clone(), lower_value(default, parent_scope));
        }
    }

    // Apply call-site props (overriding defaults)
    for prop in call_props {
        if prop.key == "__text" {
            continue; // components don't have text content
        }
        scope.insert(prop.key.clone(), lower_value(&prop.value, parent_scope));
    }

    lower_nodes(&comp.children, components, &scope)
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
        Value::Num(n, unit) => RenderValue::Num(*n, unit.as_ref().map(unit_str)),
        Value::Color(c) => RenderValue::Color(*c),
        Value::Bool(b) => RenderValue::Bool(*b),
        Value::Ref(parts) => {
            if parts.len() == 1 {
                // Single-segment ref: look up in scope
                if let Some(val) = scope.get(&parts[0]) {
                    val.clone()
                } else {
                    // Unresolved ref — produce a placeholder string
                    RenderValue::Str(format!("<unresolved:{}>", parts[0]))
                }
            } else {
                // Multi-segment ref (e.g., theme.primary) — not supported in Phase 1
                RenderValue::Str(format!("<unresolved:{}>", parts.join(".")))
            }
        }
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
        let project = resolve(dir.path(), "app.naze");
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
        ] {
            let project = resolve(&examples_dir, name);
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
    fn all_examples_with_components_lower() {
        let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples");

        for name in &["component-basic.naze", "component-props.naze", "multi-component.naze"] {
            let project = resolve(&examples_dir, name);
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
}
