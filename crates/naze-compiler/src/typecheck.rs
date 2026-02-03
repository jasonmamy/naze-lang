use std::collections::{HashMap, HashSet};

use naze_parser::ast::{Action, EventHandler, Expression, Node, Param, Prop, Span, Type, Value};

use crate::error::{CompileError, Severity};
use crate::resolve::{ComponentDef, ResolvedProject};

/// Expected type for a built-in element property.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Expected {
    Number,
    Color,
    Text,
    Bool,
    Any,
}

/// Built-in element property schemas: (element, prop_name) -> expected type.
fn builtin_prop_type(element: &str, prop: &str) -> Option<Expected> {
    // Common layout props accepted by all layout containers
    let layout_prop = match prop {
        "padding" | "gap" | "width" | "height" => Some(Expected::Number),
        "color" => Some(Expected::Color),
        "columns" => Some(Expected::Number),
        _ => None,
    };

    match element {
        "row" | "column" | "stack" | "grid" => match prop {
            "padding" | "gap" | "width" | "height" => Some(Expected::Number),
            "color" => Some(Expected::Color),
            "columns" => Some(Expected::Number),
            "align" | "justify" => Some(Expected::Any), // TODO: enum type
            _ => None,
        },
        "spacer" => match prop {
            "width" | "height" => Some(Expected::Number),
            _ => None,
        },
        "rect" => match prop {
            "width" | "height" | "radius" => Some(Expected::Number),
            "color" => Some(Expected::Color),
            _ => None,
        },
        "text" => match prop {
            "color" => Some(Expected::Color),
            "font-size" => Some(Expected::Number),
            "__text" => Some(Expected::Text),
            _ => None,
        },
        "heading" => match prop {
            "color" => Some(Expected::Color),
            "font-size" => Some(Expected::Number),
            "__text" => Some(Expected::Text),
            _ => None,
        },
        "container" => match prop {
            "padding" | "width" | "height" | "radius" => Some(Expected::Number),
            "color" => Some(Expected::Color),
            _ => None,
        },
        _ => layout_prop,
    }
}

/// Check whether a value matches an expected type.
fn value_matches(value: &Value, expected: Expected) -> bool {
    match expected {
        Expected::Any => true,
        Expected::Number => matches!(value, Value::Num(_, _)),
        Expected::Color => matches!(value, Value::Color(_)),
        Expected::Text => matches!(value, Value::Str(_) | Value::InterpolatedStr(_)),
        Expected::Bool => matches!(value, Value::Bool(_)),
    }
}

fn expected_name(expected: Expected) -> &'static str {
    match expected {
        Expected::Number => "number",
        Expected::Color => "color",
        Expected::Text => "text",
        Expected::Bool => "bool",
        Expected::Any => "any",
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Str(_) => "text",
        Value::InterpolatedStr(_) => "text",
        Value::Num(_, _) => "number",
        Value::Color(_) => "color",
        Value::Bool(_) => "bool",
        Value::Ref(_) => "reference",
        Value::List(_) => "list",
    }
}

/// Type-check a resolved project. Returns a list of errors/warnings.
pub fn typecheck(project: &ResolvedProject) -> Vec<CompileError> {
    let mut errors = Vec::new();

    // Build a lookup: component short name -> component def
    let by_name: HashMap<&str, &ComponentDef> = project
        .components
        .values()
        .map(|c| (c.name.as_str(), c))
        .collect();

    // Collect state variable names from the entry file
    let state_names: HashSet<String> = collect_state_names(&project.entry.nodes);

    // Check entry file
    check_nodes(&project.entry.nodes, &by_name, &[], &state_names, &mut errors);

    // Check component bodies (each component's body is checked with its own params in scope)
    for comp in project.components.values() {
        check_nodes(&comp.children, &by_name, &comp.params, &state_names, &mut errors);
    }

    errors
}

/// Collect all state variable names declared in the node tree.
fn collect_state_names(nodes: &[Node]) -> HashSet<String> {
    let mut names = HashSet::new();
    for node in nodes {
        match node {
            Node::State { name, .. } => {
                names.insert(name.clone());
            }
            Node::App { children, .. } | Node::Component { children, .. } => {
                names.extend(collect_state_names(children));
            }
            Node::Element { children, .. } => {
                names.extend(collect_state_names(children));
            }
            Node::If {
                then_children,
                else_children,
                ..
            } => {
                names.extend(collect_state_names(then_children));
                names.extend(collect_state_names(else_children));
            }
            Node::Each { children, .. } => {
                names.extend(collect_state_names(children));
            }
            Node::Slot {
                default_children, ..
            } => {
                names.extend(collect_state_names(default_children));
            }
            Node::Fill { children, .. } => {
                names.extend(collect_state_names(children));
            }
            _ => {}
        }
    }
    names
}

/// Recursively type-check a list of nodes.
/// `in_scope_params` are component parameters available as ref values in the current scope.
fn check_nodes(
    nodes: &[Node],
    components: &HashMap<&str, &ComponentDef>,
    in_scope_params: &[Param],
    state_names: &HashSet<String>,
    errors: &mut Vec<CompileError>,
) {
    for node in nodes {
        match node {
            Node::Element {
                name,
                props,
                children,
                handlers,
                span,
                ..
            } => {
                // Is this a component invocation?
                if let Some(comp) = components.get(name.as_str()) {
                    check_component_call(comp, props, children, span, in_scope_params, errors);
                } else {
                    // Built-in element — check prop types
                    check_builtin_props(name, props, span, in_scope_params, errors);
                }
                // Validate event handlers
                for handler in handlers {
                    check_handler(handler, state_names, errors);
                }
                check_nodes(children, components, in_scope_params, state_names, errors);
            }
            Node::App { children, .. } => {
                check_nodes(children, components, in_scope_params, state_names, errors);
            }
            Node::Component {
                children, params, ..
            } => {
                // Inside a component definition, its own params are in scope
                check_nodes(children, components, params, state_names, errors);
            }
            Node::If {
                condition,
                then_children,
                else_children,
                span,
            } => {
                check_expression(condition, state_names, span, errors);
                check_nodes(then_children, components, in_scope_params, state_names, errors);
                check_nodes(else_children, components, in_scope_params, state_names, errors);
            }
            Node::Each {
                iterable,
                children,
                span,
                ..
            } => {
                check_expression(iterable, state_names, span, errors);
                check_nodes(children, components, in_scope_params, state_names, errors);
            }
            Node::Slot {
                default_children, ..
            } => {
                check_nodes(default_children, components, in_scope_params, state_names, errors);
            }
            Node::Fill { children, .. } => {
                check_nodes(children, components, in_scope_params, state_names, errors);
            }
            Node::Let { .. } | Node::State { .. } => {
                // Declarations — no type-checking needed in Phase 2 M1
                // (Future: validate names don't shadow builtins)
            }
            _ => {}
        }
    }
}

/// Validate an event handler: check that set targets are state variables
/// and that expression state refs exist.
fn check_handler(
    handler: &EventHandler,
    state_names: &HashSet<String>,
    errors: &mut Vec<CompileError>,
) {
    match &handler.action {
        Action::Set { target, expr, span } => {
            if !state_names.contains(target) {
                errors.push(CompileError {
                    message: format!(
                        "cannot set '{}': not a declared state variable",
                        target
                    ),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            }
            check_expression(expr, state_names, &handler.span, errors);
        }
        Action::Navigate { .. } => {}
    }
}

/// Validate that all state refs in an expression refer to declared state variables.
fn check_expression(
    expr: &Expression,
    state_names: &HashSet<String>,
    span: &Span,
    errors: &mut Vec<CompileError>,
) {
    match expr {
        Expression::StateRef(name) => {
            if !state_names.contains(name) {
                errors.push(CompileError {
                    message: format!(
                        "unknown state variable '{}' in expression",
                        name
                    ),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            }
        }
        Expression::BinOp { left, right, .. } => {
            check_expression(left, state_names, span, errors);
            check_expression(right, state_names, span, errors);
        }
        Expression::Literal(_) => {}
    }
}

/// Type-check a component invocation against its definition.
fn check_component_call(
    comp: &ComponentDef,
    props: &[Prop],
    call_children: &[Node],
    call_span: &Span,
    _in_scope_params: &[Param],
    errors: &mut Vec<CompileError>,
) {
    let param_map: HashMap<&str, &Param> = comp.params.iter().map(|p| (p.name.as_str(), p)).collect();

    // Check each provided prop
    for prop in props {
        if prop.key == "__text" {
            // Text content shorthand — components don't accept this
            errors.push(CompileError {
                message: format!(
                    "component '{}' does not accept inline text content",
                    comp.name
                ),
                file: call_span.file.clone(),
                line: call_span.line,
                column: call_span.col,
                severity: Severity::Error,
            });
            continue;
        }

        match param_map.get(prop.key.as_str()) {
            Some(param) => {
                // Skip type checking for refs — they're resolved at runtime or codegen
                if !matches!(&prop.value, Value::Ref(_)) {
                    if !value_matches_type(&prop.value, &param.ty) {
                        errors.push(CompileError {
                            message: format!(
                                "type mismatch for prop '{}' on component '{}': expected {}, got {}",
                                prop.key,
                                comp.name,
                                type_name(&param.ty),
                                value_type_name(&prop.value),
                            ),
                            file: call_span.file.clone(),
                            line: call_span.line,
                            column: call_span.col,
                            severity: Severity::Error,
                        });
                    }
                }
            }
            None => {
                errors.push(CompileError {
                    message: format!(
                        "unknown prop '{}' on component '{}'. Available: {}",
                        prop.key,
                        comp.name,
                        comp.params
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    file: call_span.file.clone(),
                    line: call_span.line,
                    column: call_span.col,
                    severity: Severity::Error,
                });
            }
        }
    }

    // Check required props (no default) are provided
    for param in &comp.params {
        if param.default.is_none() {
            let provided = props.iter().any(|p| p.key == param.name);
            if !provided {
                errors.push(CompileError {
                    message: format!(
                        "missing required prop '{}' on component '{}'",
                        param.name, comp.name
                    ),
                    file: call_span.file.clone(),
                    line: call_span.line,
                    column: call_span.col,
                    severity: Severity::Error,
                });
            }
        }
    }

    // Check slot/fill usage
    let has_slots = has_slot_in_tree(&comp.children);

    if !call_children.is_empty() && !has_slots {
        errors.push(CompileError {
            message: format!(
                "component '{}' does not declare any slots and cannot accept children",
                comp.name
            ),
            file: call_span.file.clone(),
            line: call_span.line,
            column: call_span.col,
            severity: Severity::Error,
        });
    }

    if has_slots {
        let declared_slots = collect_slot_names(&comp.children);
        for child in call_children {
            if let Node::Fill { name, span, .. } = child {
                if !declared_slots.contains(name.as_str()) {
                    errors.push(CompileError {
                        message: format!(
                            "component '{}' has no slot named '{}'",
                            comp.name, name
                        ),
                        file: span.file.clone(),
                        line: span.line,
                        column: span.col,
                        severity: Severity::Error,
                    });
                }
            }
        }
    }
}

/// Type-check props on a built-in element.
fn check_builtin_props(
    element: &str,
    props: &[Prop],
    span: &Span,
    in_scope_params: &[Param],
    errors: &mut Vec<CompileError>,
) {
    for prop in props {
        // Skip interpolated strings — they contain state refs resolved at runtime
        if matches!(&prop.value, Value::InterpolatedStr(_)) {
            continue;
        }

        // Skip ref values — they reference component params, checked elsewhere
        if matches!(&prop.value, Value::Ref(parts) if parts.len() == 1) {
            // Single-segment ref: check it's a valid in-scope param
            if let Value::Ref(parts) = &prop.value {
                let name = &parts[0];
                let is_param = in_scope_params.iter().any(|p| p.name == *name);
                if !is_param && !name.contains('.') {
                    // Not a known param — could be a forward reference or error.
                    // For Phase 1, only warn if we're inside a component body.
                    if !in_scope_params.is_empty() {
                        errors.push(CompileError {
                            message: format!(
                                "unknown reference '{}' on element '{}': not a component parameter",
                                name, element
                            ),
                            file: span.file.clone(),
                            line: span.line,
                            column: span.col,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            continue;
        }

        // Multi-segment ref (e.g., theme.primary) — skip type checking
        if matches!(&prop.value, Value::Ref(parts) if parts.len() > 1) {
            continue;
        }

        if let Some(expected) = builtin_prop_type(element, &prop.key) {
            if !value_matches(&prop.value, expected) {
                errors.push(CompileError {
                    message: format!(
                        "type mismatch for prop '{}' on '{}': expected {}, got {}",
                        prop.key,
                        element,
                        expected_name(expected),
                        value_type_name(&prop.value),
                    ),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            }
        }
        // Unknown props on builtins are silently ignored for Phase 1 — the set will grow.
    }
}

/// Check if a value matches a declared component parameter type.
fn value_matches_type(value: &Value, ty: &Type) -> bool {
    match ty {
        Type::Text => matches!(value, Value::Str(_) | Value::InterpolatedStr(_)),
        Type::Number => matches!(value, Value::Num(_, _)),
        Type::Bool => matches!(value, Value::Bool(_)),
        Type::Color => matches!(value, Value::Color(_)),
    }
}

fn type_name(ty: &Type) -> &'static str {
    match ty {
        Type::Text => "text",
        Type::Number => "number",
        Type::Bool => "bool",
        Type::Color => "color",
    }
}

/// Check if a node tree contains any Slot nodes.
fn has_slot_in_tree(nodes: &[Node]) -> bool {
    for node in nodes {
        match node {
            Node::Slot { .. } => return true,
            Node::Element { children, .. } => {
                if has_slot_in_tree(children) {
                    return true;
                }
            }
            Node::If {
                then_children,
                else_children,
                ..
            } => {
                if has_slot_in_tree(then_children) || has_slot_in_tree(else_children) {
                    return true;
                }
            }
            Node::Each { children, .. } => {
                if has_slot_in_tree(children) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Collect all named slot names from a component's body.
fn collect_slot_names(nodes: &[Node]) -> HashSet<&str> {
    let mut names = HashSet::new();
    for node in nodes {
        match node {
            Node::Slot {
                name: Some(n), ..
            } => {
                names.insert(n.as_str());
            }
            Node::Element { children, .. } => {
                names.extend(collect_slot_names(children));
            }
            Node::If {
                then_children,
                else_children,
                ..
            } => {
                names.extend(collect_slot_names(then_children));
                names.extend(collect_slot_names(else_children));
            }
            Node::Each { children, .. } => {
                names.extend(collect_slot_names(children));
            }
            _ => {}
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::resolve;
    use std::fs;
    use std::path::Path;

    fn setup_and_check(files: &[(&str, &str)]) -> Vec<CompileError> {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            let path = dir.path().join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }
        let project = resolve(dir.path(), "app.naze");
        let mut errors = typecheck(&project);
        errors.extend(project.errors);
        errors
    }

    fn errors_only(errors: &[CompileError]) -> Vec<&CompileError> {
        errors
            .iter()
            .filter(|e| matches!(e.severity, Severity::Error))
            .collect()
    }

    #[test]
    fn valid_app_no_errors() {
        let errors = setup_and_check(&[(
            "app.naze",
            r#"app "Hello" {
  column padding: 20px, gap: 16px {
    heading "Title"
    rect width: 100px, height: 50px, color: #ff0000, radius: 8px
    text "body"
  }
}
"#,
        )]);
        let errs = errors_only(&errors);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn valid_component_call() {
        let errors = setup_and_check(&[
            (
                "components/box.naze",
                "component box(color: color, size: number = 80px) {\n  rect width: size, height: size, color: color\n}\n",
            ),
            (
                "app.naze",
                "use components/box\n\napp \"Test\" {\n  box color: #ff0000\n  box color: #00ff00, size: 120px\n}\n",
            ),
        ]);
        let errs = errors_only(&errors);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn missing_required_prop() {
        let errors = setup_and_check(&[
            (
                "components/box.naze",
                "component box(color: color) {\n  rect width: 80px, height: 80px, color: color\n}\n",
            ),
            (
                "app.naze",
                "use components/box\n\napp \"Test\" {\n  box size: 100px\n}\n",
            ),
        ]);
        let errs = errors_only(&errors);
        assert!(
            errs.iter().any(|e| e.message.contains("missing required prop 'color'")),
            "expected missing prop error, got: {:?}",
            errs
        );
    }

    #[test]
    fn wrong_prop_type_on_component() {
        let errors = setup_and_check(&[
            (
                "components/box.naze",
                "component box(color: color) {\n  rect width: 80px, height: 80px, color: color\n}\n",
            ),
            (
                "app.naze",
                "use components/box\n\napp \"Test\" {\n  box color: 42px\n}\n",
            ),
        ]);
        let errs = errors_only(&errors);
        assert!(
            errs.iter().any(|e| e.message.contains("type mismatch") && e.message.contains("color")),
            "expected type mismatch, got: {:?}",
            errs
        );
    }

    #[test]
    fn unknown_prop_on_component() {
        let errors = setup_and_check(&[
            (
                "components/box.naze",
                "component box(color: color) {\n  rect width: 80px, height: 80px, color: color\n}\n",
            ),
            (
                "app.naze",
                "use components/box\n\napp \"Test\" {\n  box color: #ff0000, bogus: 10px\n}\n",
            ),
        ]);
        let errs = errors_only(&errors);
        assert!(
            errs.iter().any(|e| e.message.contains("unknown prop 'bogus'")),
            "expected unknown prop error, got: {:?}",
            errs
        );
    }

    #[test]
    fn wrong_type_on_builtin() {
        let errors = setup_and_check(&[(
            "app.naze",
            "app \"Test\" {\n  rect width: \"not a number\", color: #ff0000\n}\n",
        )]);
        let errs = errors_only(&errors);
        assert!(
            errs.iter().any(|e| e.message.contains("type mismatch") && e.message.contains("width")),
            "expected type mismatch on width, got: {:?}",
            errs
        );
    }

    #[test]
    fn color_where_number_expected() {
        let errors = setup_and_check(&[(
            "app.naze",
            "app \"Test\" {\n  rect width: #ff0000, height: 50px, color: #000000\n}\n",
        )]);
        let errs = errors_only(&errors);
        assert!(
            errs.iter().any(|e| e.message.contains("type mismatch") && e.message.contains("width")),
            "expected type mismatch, got: {:?}",
            errs
        );
    }

    #[test]
    fn ref_to_valid_param_ok() {
        let errors = setup_and_check(&[
            (
                "components/box.naze",
                "component box(color: color, size: number = 80px) {\n  rect width: size, height: size, color: color\n}\n",
            ),
            (
                "app.naze",
                "use components/box\n\napp \"Test\" {\n  box color: #ff0000\n}\n",
            ),
        ]);
        // The refs inside the component body (size, color) should not produce errors
        let errs = errors_only(&errors);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }

    #[test]
    fn ref_to_unknown_param_warns() {
        let errors = setup_and_check(&[(
            "components/box.naze",
            "component box(color: color) {\n  rect width: bogus, height: 80px, color: color\n}\n",
        ),
        (
            "app.naze",
            "use components/box\n\napp \"Test\" {\n  box color: #ff0000\n}\n",
        )]);
        assert!(
            errors.iter().any(|e| e.message.contains("unknown reference 'bogus'")),
            "expected unknown ref warning, got: {:?}",
            errors
        );
    }

    #[test]
    fn nested_elements_checked() {
        let errors = setup_and_check(&[(
            "app.naze",
            r#"app "Test" {
  column padding: 20px {
    row gap: "bad" {
      rect width: 50px, height: 50px, color: #000000
    }
  }
}
"#,
        )]);
        let errs = errors_only(&errors);
        assert!(
            errs.iter().any(|e| e.message.contains("type mismatch") && e.message.contains("gap")),
            "expected gap type error, got: {:?}",
            errs
        );
    }

    #[test]
    fn all_examples_typecheck() {
        let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples");

        // Parse examples that are self-contained apps (no use statements to external dirs)
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
            let project = resolve(&examples_dir, name);
            let tc_errors = typecheck(&project);
            let all_errors: Vec<_> = project
                .errors
                .iter()
                .chain(tc_errors.iter())
                .filter(|e| matches!(e.severity, Severity::Error))
                .collect();
            assert!(
                all_errors.is_empty(),
                "errors in {}: {:?}",
                name,
                all_errors
            );
        }

        // Examples with component imports (resolved from examples dir)
        for name in &[
            "component-basic.naze",
            "component-props.naze",
            "multi-component.naze",
            "slots.naze",
        ] {
            let project = resolve(&examples_dir, name);
            let tc_errors = typecheck(&project);
            let all_errors: Vec<_> = project
                .errors
                .iter()
                .chain(tc_errors.iter())
                .filter(|e| matches!(e.severity, Severity::Error))
                .collect();
            assert!(
                all_errors.is_empty(),
                "errors in {}: {:?}",
                name,
                all_errors
            );
        }
    }

    #[test]
    fn component_without_slots_rejects_children() {
        let errors = setup_and_check(&[(
            "app.naze",
            r#"component box(color: color) {
  rect width: 80px, height: 80px, color: color
}

app "Test" {
  box color: #ff0000 {
    text "should not be here"
  }
}
"#,
        )]);
        let errs = errors_only(&errors);
        assert!(
            errs.iter()
                .any(|e| e.message.contains("does not declare any slots")),
            "expected slot error, got: {:?}",
            errs
        );
    }

    #[test]
    fn fill_unknown_slot_name_errors() {
        let errors = setup_and_check(&[(
            "app.naze",
            r#"component card(title: text) {
  rect color: #f0f0f0 {
    heading "{title}"
    slot
  }
}

app "Test" {
  card title: "Hello" {
    fill "nonexistent" {
      text "bad"
    }
  }
}
"#,
        )]);
        let errs = errors_only(&errors);
        assert!(
            errs.iter()
                .any(|e| e.message.contains("has no slot named 'nonexistent'")),
            "expected unknown slot error, got: {:?}",
            errs
        );
    }

    #[test]
    fn valid_slot_usage_no_errors() {
        let errors = setup_and_check(&[(
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
  }
}
"#,
        )]);
        let errs = errors_only(&errors);
        assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    }
}
