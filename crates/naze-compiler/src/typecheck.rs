use std::collections::{HashMap, HashSet};
use std::path::Path;

use naze_parser::ast::{
    Action, EventHandler, Expression, MatchPattern, Node, Param, Prop, Span, Type, Value,
};

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
    // Accessibility props accepted by all elements
    if matches!(prop, "role" | "label" | "id") {
        return Some(Expected::Text);
    }

    // Common layout props accepted by all layout containers
    let layout_prop = match prop {
        "padding" | "gap" | "width" | "height" => Some(Expected::Number),
        "color" => Some(Expected::Color),
        "columns" => Some(Expected::Number),
        _ => None,
    };

    match element {
        "row" | "column" | "stack" | "grid" => match prop {
            "padding" | "gap" | "width" | "height" | "opacity" => Some(Expected::Number),
            "min-width" | "max-width" | "min-height" | "max-height" => Some(Expected::Number),
            "flex-grow" | "flex-shrink" => Some(Expected::Number),
            "responsive" | "collapsible" => Some(Expected::Number),
            "color" => Some(Expected::Color),
            "columns" => Some(Expected::Number),
            "align" | "justify" => Some(Expected::Text),
            "wrap" => Some(Expected::Bool),
            "cursor" | "shadow" | "overflow" | "gradient" | "transform" => Some(Expected::Text),
            _ => None,
        },
        "spacer" => match prop {
            "width" | "height" | "flex-grow" | "flex-shrink" => Some(Expected::Number),
            "cursor" => Some(Expected::Text),
            _ => None,
        },
        "rect" => match prop {
            "width" | "height" | "radius" | "border" | "opacity" | "tab-index" => {
                Some(Expected::Number)
            }
            "color" | "border-color" => Some(Expected::Color),
            "cursor" | "shadow" | "gradient" | "transform" => Some(Expected::Text),
            _ => None,
        },
        "text" => match prop {
            "color" => Some(Expected::Color),
            "font-size" | "opacity" | "tab-index" | "line-height" | "letter-spacing" => {
                Some(Expected::Number)
            }
            "__text" | "cursor" | "text-decoration" | "text-align" | "text-overflow"
            | "transform" => Some(Expected::Text),
            _ => None,
        },
        "heading" => match prop {
            "color" => Some(Expected::Color),
            "font-size" | "opacity" | "tab-index" | "line-height" | "letter-spacing" => {
                Some(Expected::Number)
            }
            "__text" | "cursor" | "text-decoration" | "text-align" | "text-overflow"
            | "transform" => Some(Expected::Text),
            _ => None,
        },
        "container" => match prop {
            "padding" | "width" | "height" | "radius" | "border" | "opacity" | "collapsible" => {
                Some(Expected::Number)
            }
            "color" | "border-color" => Some(Expected::Color),
            "cursor" | "shadow" | "overflow" | "gradient" | "transform" => Some(Expected::Text),
            _ => None,
        },
        "scroll" => match prop {
            "padding" | "width" | "height" | "radius" | "border" | "opacity" => {
                Some(Expected::Number)
            }
            "color" | "border-color" => Some(Expected::Color),
            "overflow" | "cursor" => Some(Expected::Text),
            _ => None,
        },
        "image" => match prop {
            "src" | "alt" | "fit" | "cursor" | "transform" => Some(Expected::Text),
            "width" | "height" | "opacity" => Some(Expected::Number),
            _ => None,
        },
        "link" => match prop {
            "__text" | "to" | "cursor" | "text-decoration" | "text-align" => Some(Expected::Text),
            "color" => Some(Expected::Color),
            "line-height" => Some(Expected::Number),
            _ => None,
        },
        "overlay" => match prop {
            "padding" | "width" | "height" | "radius" | "border" | "opacity" => {
                Some(Expected::Number)
            }
            "color" | "border-color" => Some(Expected::Color),
            "focus-trap" | "scroll-lock" | "dismiss-on-escape" => Some(Expected::Bool),
            "anchor" | "anchor-placement" | "cursor" | "shadow" => Some(Expected::Text),
            _ => None,
        },
        "textarea" => match prop {
            "width" | "height" | "font-size" | "rows" | "max-length" | "opacity" | "tab-index"
            | "line-height" | "letter-spacing" | "border" | "radius" => Some(Expected::Number),
            "color" | "border-color" => Some(Expected::Color),
            "placeholder" | "cursor" | "text-align" | "shadow" | "transform" => {
                Some(Expected::Text)
            }
            "bind" => Some(Expected::Any),
            "validate" => Some(Expected::Any),
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
        Value::Object(_) => "object",
        Value::Bind(_) => "bind", // Two-way state binding
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

    // Collect computed variable names (read-only, cannot be set)
    let computed_names: HashSet<String> = collect_computed_names(&project.entry.nodes);

    // Collect data declaration names (for validating trigger action targets)
    let data_names: HashSet<String> = collect_data_names(&project.entry.nodes);

    // Check entry file
    check_nodes(
        &project.entry.nodes,
        &by_name,
        &[],
        &state_names,
        &computed_names,
        &data_names,
        &mut errors,
    );

    // Check component bodies (each component's body is checked with its own params in scope)
    for comp in project.components.values() {
        check_nodes(
            &comp.children,
            &by_name,
            &comp.params,
            &state_names,
            &computed_names,
            &data_names,
            &mut errors,
        );
    }

    // Validate WASM import function references
    validate_wasm_imports(project, &mut errors);

    // Validate server data calls reference declared server functions with correct arg counts
    validate_server_calls(&project.entry.nodes, &mut errors);

    errors
}

/// Collect all state variable names declared in the node tree.
/// Also collects derived state names for inputs with validation (e.g., {bind}_valid, {bind}_error).
fn collect_state_names(nodes: &[Node]) -> HashSet<String> {
    let mut names = HashSet::new();
    for node in nodes {
        match node {
            Node::State { name, .. } => {
                names.insert(name.clone());
            }
            Node::Computed { name, .. } => {
                // Computed declarations are readable as state
                names.insert(name.clone());
            }
            Node::Param { name, .. } => {
                // URL params are readable as state
                names.insert(name.clone());
            }
            Node::Storage { name, .. } => {
                // Storage declarations act like state variables
                names.insert(name.clone());
            }
            Node::Data { name, .. } | Node::ServerData { name, .. } | Node::Prompt { name, .. } => {
                // Data/prompt declarations create three derived state variables
                names.insert(format!("{}.loading", name));
                names.insert(format!("{}.error", name));
                names.insert(format!("{}.data", name));
            }
            Node::App { children, .. }
            | Node::Component { children, .. }
            | Node::Template { children, .. } => {
                names.extend(collect_state_names(children));
            }
            Node::Element {
                name,
                props,
                children,
                ..
            } => {
                // Check if this is an input with both bind and validate props
                if name == "input" {
                    let has_validate = props.iter().any(|p| p.key == "validate");
                    if has_validate {
                        // Find the bind variable
                        if let Some(bind_prop) = props.iter().find(|p| p.key == "bind") {
                            if let Value::Bind(bind_var) = &bind_prop.value {
                                // Add derived state variable names
                                names.insert(format!("{}_valid", bind_var));
                                names.insert(format!("{}_error", bind_var));
                            }
                        }
                    }
                }
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
            Node::Page {
                params, children, ..
            } => {
                // Dynamic route params are available as params.NAME inside the page
                for p in params {
                    names.insert(format!("params.{p}"));
                }
                names.extend(collect_state_names(children));
            }
            Node::Link { children, .. } => {
                names.extend(collect_state_names(children));
            }
            Node::Match { arms, .. } => {
                for arm in arms {
                    names.extend(collect_state_names(&arm.children));
                }
            }
            _ => {}
        }
    }
    names
}

/// Collect computed variable names (these are read-only and cannot be `set`).
fn collect_computed_names(nodes: &[Node]) -> HashSet<String> {
    let mut names = HashSet::new();
    for node in nodes {
        match node {
            Node::Computed { name, .. } => {
                names.insert(name.clone());
            }
            Node::App { children, .. }
            | Node::Component { children, .. }
            | Node::Template { children, .. }
            | Node::Each { children, .. }
            | Node::Fill { children, .. }
            | Node::Page { children, .. }
            | Node::Link { children, .. } => {
                names.extend(collect_computed_names(children));
            }
            Node::Element { children, .. } => {
                names.extend(collect_computed_names(children));
            }
            Node::Slot {
                default_children, ..
            } => {
                names.extend(collect_computed_names(default_children));
            }
            Node::If {
                then_children,
                else_children,
                ..
            } => {
                names.extend(collect_computed_names(then_children));
                names.extend(collect_computed_names(else_children));
            }
            _ => {}
        }
    }
    names
}

/// Collect all data declaration names (for validating `trigger` action targets).
fn collect_data_names(nodes: &[Node]) -> HashSet<String> {
    let mut names = HashSet::new();
    for node in nodes {
        match node {
            Node::Data { name, .. } => {
                names.insert(name.clone());
            }
            Node::App { children, .. }
            | Node::Component { children, .. }
            | Node::Template { children, .. }
            | Node::Each { children, .. }
            | Node::Fill { children, .. }
            | Node::Page { children, .. }
            | Node::Link { children, .. } => {
                names.extend(collect_data_names(children));
            }
            Node::Element { children, .. } => {
                names.extend(collect_data_names(children));
            }
            Node::Slot {
                default_children, ..
            } => {
                names.extend(collect_data_names(default_children));
            }
            Node::If {
                then_children,
                else_children,
                ..
            } => {
                names.extend(collect_data_names(then_children));
                names.extend(collect_data_names(else_children));
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
    computed_names: &HashSet<String>,
    data_names: &HashSet<String>,
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
                    // Check accessibility for elements with click handlers
                    check_handler_accessibility(name, props, handlers, span, errors);
                }
                // Validate event handlers
                for handler in handlers {
                    check_handler(handler, state_names, computed_names, data_names, errors);
                }
                check_nodes(
                    children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::App { children, .. } => {
                check_nodes(
                    children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::Component {
                children, params, ..
            } => {
                // Inside a component definition, its own params are in scope
                check_nodes(
                    children,
                    components,
                    params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::Template { children, .. } => {
                check_nodes(
                    children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::If {
                condition,
                then_children,
                else_children,
                span,
            } => {
                check_expression(condition, state_names, span, errors);
                check_nodes(
                    then_children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
                check_nodes(
                    else_children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::Each {
                variable,
                iterable,
                children,
                span,
            } => {
                check_expression(iterable, state_names, span, errors);
                // Add loop variable and index to scope for children
                let mut child_state = state_names.clone();
                child_state.insert(variable.clone());
                child_state.insert(format!("{}_index", variable));
                check_nodes(
                    children,
                    components,
                    in_scope_params,
                    &child_state,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::Slot {
                default_children, ..
            } => {
                check_nodes(
                    default_children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::Fill { children, .. } => {
                check_nodes(
                    children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::Page { children, .. } => {
                check_nodes(
                    children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::Link { children, .. } => {
                // Link element — children are optional nested content
                check_nodes(
                    children,
                    components,
                    in_scope_params,
                    state_names,
                    computed_names,
                    data_names,
                    errors,
                );
            }
            Node::Match {
                subject,
                arms,
                span,
            } => {
                check_expression(subject, state_names, span, errors);
                // Warn if no wildcard arm
                let has_wildcard = arms
                    .iter()
                    .any(|a| matches!(a.pattern, MatchPattern::Wildcard));
                if !has_wildcard {
                    errors.push(CompileError {
                        message:
                            "match expression should have a wildcard '_' arm for exhaustiveness"
                                .to_string(),
                        file: span.file.clone(),
                        line: span.line,
                        column: span.col,
                        severity: Severity::Warning,
                    });
                }
                for arm in arms {
                    check_nodes(
                        &arm.children,
                        components,
                        in_scope_params,
                        state_names,
                        computed_names,
                        data_names,
                        errors,
                    );
                }
            }
            Node::Function { .. } | Node::ServerFunction { .. } => {
                // Function definitions — validated during collection
            }
            Node::Let { .. }
            | Node::State { .. }
            | Node::Computed { .. }
            | Node::Storage { .. }
            | Node::Timer { .. }
            | Node::Param { .. }
            | Node::ServerData { .. }
            | Node::Prompt { .. } => {
                // Declarations — no type-checking needed beyond name collection
            }
            _ => {}
        }
    }
}

/// Check accessibility warnings for elements with event handlers.
fn check_handler_accessibility(
    element: &str,
    props: &[Prop],
    handlers: &[EventHandler],
    span: &Span,
    errors: &mut Vec<CompileError>,
) {
    // Skip if no click handlers (hover-only elements don't need full a11y)
    let has_click = handlers.iter().any(|h| h.event == "click");
    if !has_click {
        return;
    }

    let has_role = props.iter().any(|p| p.key == "role");
    let has_label = props.iter().any(|p| p.key == "label");
    let has_text = props.iter().any(|p| p.key == "__text");

    // Elements that act as buttons should have role: "button"
    if matches!(element, "rect" | "row" | "column" | "stack") && !has_role {
        errors.push(CompileError {
            message: format!(
                "clickable {} should have 'role' prop (e.g., role: \"button\") for screen readers",
                element
            ),
            file: span.file.clone(),
            line: span.line,
            column: span.col,
            severity: Severity::Warning,
        });
    }

    // Clickable elements without visible text need a label
    if !has_text && !has_label && !matches!(element, "text" | "heading" | "link") {
        errors.push(CompileError {
            message: format!(
                "clickable {} without text should have 'label' prop for screen readers",
                element
            ),
            file: span.file.clone(),
            line: span.line,
            column: span.col,
            severity: Severity::Warning,
        });
    }
}

/// Validate an event handler: check that set targets are state variables
/// and that expression state refs exist.
fn check_handler(
    handler: &EventHandler,
    state_names: &HashSet<String>,
    computed_names: &HashSet<String>,
    data_names: &HashSet<String>,
    errors: &mut Vec<CompileError>,
) {
    match &handler.action {
        Action::Set { target, expr, span } => {
            if computed_names.contains(target) {
                errors.push(CompileError {
                    message: format!("cannot set '{}': computed values are read-only", target),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            } else if !state_names.contains(target) {
                errors.push(CompileError {
                    message: format!("cannot set '{}': not a declared state variable", target),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            }
            check_expression(expr, state_names, &handler.span, errors);
        }
        Action::Navigate { .. } => {}
        Action::ScrollTo { .. } => {}
        Action::Log { expr, .. } => {
            check_expression(expr, state_names, &handler.span, errors);
        }
        Action::Trigger { data_name, span } => {
            if !data_names.contains(data_name) {
                errors.push(CompileError {
                    message: format!("cannot trigger '{}': not a declared data source", data_name),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            }
        }
        Action::Copy { expr, .. } => {
            check_expression(expr, state_names, &handler.span, errors);
        }
        Action::Send { expr, .. } => {
            check_expression(expr, state_names, &handler.span, errors);
        }
        Action::JsCall { args, .. } => {
            for arg in args {
                check_expression(arg, state_names, &handler.span, errors);
            }
        }
        Action::Notify { .. } => {}
        Action::Emit { .. } => {}
        Action::SetTheme { .. } => {}
        Action::Append { item, .. } => {
            check_expression(item, state_names, &handler.span, errors);
        }
        Action::Remove { index, .. } => {
            check_expression(index, state_names, &handler.span, errors);
        }
    }
}

/// Validate that all state refs in an expression refer to declared state variables.
fn check_expression(
    expr: &Expression,
    state_names: &HashSet<String>,
    span: &Span,
    errors: &mut Vec<CompileError>,
) {
    check_expression_inner(expr, state_names, span, errors, false);
}

fn check_expression_inner(
    expr: &Expression,
    state_names: &HashSet<String>,
    span: &Span,
    errors: &mut Vec<CompileError>,
    in_pipeline_stage: bool,
) {
    match expr {
        Expression::StateRef(name) => {
            // Env var references (env.NAME) are resolved at compile/runtime — skip state check
            if name.starts_with("env.") {
                return;
            }
            // Inside pipeline stages, bare identifiers may refer to item fields
            // rather than state variables, so skip strict validation there
            if !in_pipeline_stage && !state_names.contains(name) {
                errors.push(CompileError {
                    message: format!("unknown state variable '{}' in expression", name),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            }
        }
        Expression::BinOp { left, right, .. } => {
            check_expression_inner(left, state_names, span, errors, in_pipeline_stage);
            check_expression_inner(right, state_names, span, errors, in_pipeline_stage);
        }
        Expression::Literal(_) => {}
        Expression::Pipeline { source, stages } => {
            // Source must be a valid state reference
            check_expression_inner(source, state_names, span, errors, false);
            // Stage arguments refer to item fields, not state — skip strict validation
            for stage in stages {
                // Validate required arguments
                use naze_parser::ast::PipelineFn;
                match stage.function {
                    PipelineFn::Filter
                    | PipelineFn::Map
                    | PipelineFn::SortBy
                    | PipelineFn::Take
                    | PipelineFn::GroupBy => {
                        if stage.argument.is_none() {
                            let fn_name = match stage.function {
                                PipelineFn::Filter => "filter",
                                PipelineFn::Map => "map",
                                PipelineFn::SortBy => "sort-by",
                                PipelineFn::Take => "take",
                                PipelineFn::GroupBy => "group-by",
                                _ => unreachable!(),
                            };
                            errors.push(CompileError {
                                message: format!(
                                    "pipeline function '{}' requires an argument",
                                    fn_name
                                ),
                                file: span.file.clone(),
                                line: span.line,
                                column: span.col,
                                severity: Severity::Error,
                            });
                        }
                    }
                    PipelineFn::Reduce => {
                        if stage.argument.is_none() {
                            errors.push(CompileError {
                                message:
                                    "pipeline function 'reduce' requires an accumulator expression"
                                        .to_string(),
                                file: span.file.clone(),
                                line: span.line,
                                column: span.col,
                                severity: Severity::Error,
                            });
                        }
                        if stage.argument2.is_none() {
                            errors.push(CompileError {
                                message: "pipeline function 'reduce' requires an initial value"
                                    .to_string(),
                                file: span.file.clone(),
                                line: span.line,
                                column: span.col,
                                severity: Severity::Error,
                            });
                        }
                    }
                    PipelineFn::Sum | PipelineFn::Count | PipelineFn::Flatten => {
                        // No argument required
                    }
                    PipelineFn::Distinct => {
                        // Optional argument (field name for object lists)
                    }
                }
                if let Some(arg) = &stage.argument {
                    check_expression_inner(arg, state_names, span, errors, true);
                }
                if let Some(arg2) = &stage.argument2 {
                    check_expression_inner(arg2, state_names, span, errors, in_pipeline_stage);
                }
            }
        }
        Expression::FunctionCall { args, .. } => {
            // Validate argument expressions
            for arg in args {
                check_expression_inner(arg, state_names, span, errors, in_pipeline_stage);
            }
        }
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
    let param_map: HashMap<&str, &Param> =
        comp.params.iter().map(|p| (p.name.as_str(), p)).collect();

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
                if !matches!(&prop.value, Value::Ref(_))
                    && !value_matches_type(&prop.value, &param.ty)
                {
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
                        message: format!("component '{}' has no slot named '{}'", comp.name, name),
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

/// Check that interactive elements have required accessibility props.
/// Emits warnings (not errors) to encourage accessible apps without blocking builds.
fn check_accessibility_props(
    element: &str,
    props: &[Prop],
    span: &Span,
    errors: &mut Vec<CompileError>,
) {
    // Elements that should have explicit roles or labels for screen readers
    let needs_label = matches!(
        element,
        "rect" | "image" | "input" | "textarea" | "checkbox" | "radio" | "select"
    );
    let is_interactive = matches!(
        element,
        "rect" | "input" | "textarea" | "checkbox" | "radio" | "select"
    ) || props.iter().any(|p| p.key == "on");

    let has_label = props.iter().any(|p| p.key == "label");
    let has_role = props.iter().any(|p| p.key == "role");
    let has_text = props.iter().any(|p| p.key == "__text");
    let has_handlers = !props.is_empty()
        && props
            .iter()
            .any(|p| matches!(p.key.as_str(), "draggable" | "drop-target"));

    // Image elements should always have alt text (mapped to label)
    if element == "image" && !has_label {
        let has_alt = props.iter().any(|p| p.key == "alt");
        if !has_alt {
            errors.push(CompileError {
                message: "image element should have 'alt' or 'label' prop for accessibility"
                    .to_string(),
                file: span.file.clone(),
                line: span.line,
                column: span.col,
                severity: Severity::Warning,
            });
        }
    }

    // Interactive rect elements (with click handlers) should have a role
    if element == "rect" && is_interactive && !has_role {
        errors.push(CompileError {
            message: "interactive rect element should have 'role' prop (e.g., role: \"button\") for accessibility".to_string(),
            file: span.file.clone(),
            line: span.line,
            column: span.col,
            severity: Severity::Warning,
        });
    }

    // Interactive elements without visible text should have a label
    if needs_label && is_interactive && !has_label && !has_text {
        // Form inputs often have placeholder or visible context, so only warn for truly unlabeled elements
        if !matches!(element, "input" | "select") || !props.iter().any(|p| p.key == "placeholder") {
            errors.push(CompileError {
                message: format!(
                    "{} element should have 'label' prop for accessibility (screen reader support)",
                    element
                ),
                file: span.file.clone(),
                line: span.line,
                column: span.col,
                severity: Severity::Warning,
            });
        }
    }

    // Overlay with focus-trap should have a role (e.g., "dialog")
    if element == "overlay" {
        let has_focus_trap = props
            .iter()
            .any(|p| p.key == "focus-trap" && matches!(p.value, Value::Bool(true)));
        if has_focus_trap && !has_role {
            errors.push(CompileError {
                message: "overlay with focus-trap should have 'role' prop (e.g., role: \"dialog\") for accessibility".to_string(),
                file: span.file.clone(),
                line: span.line,
                column: span.col,
                severity: Severity::Warning,
            });
        }
    }

    // Draggable elements should have a role
    if has_handlers && !has_role {
        errors.push(CompileError {
            message: "draggable element should have 'role' prop for accessibility".to_string(),
            file: span.file.clone(),
            line: span.line,
            column: span.col,
            severity: Severity::Warning,
        });
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
    // Check for missing accessibility props on interactive elements
    check_accessibility_props(element, props, span, errors);

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
            Node::Slot { name: Some(n), .. } => {
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

// ─── WASM import validation ──────────────────────────────────────────────────

/// Read exported function names from a WASM binary.
pub fn read_wasm_exports(path: &Path) -> HashSet<String> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return HashSet::new(),
    };
    let parser = wasmparser::Parser::new(0);
    let mut exports = HashSet::new();
    for payload in parser.parse_all(&bytes) {
        if let Ok(wasmparser::Payload::ExportSection(reader)) = payload {
            for export in reader.into_iter().flatten() {
                if matches!(export.kind, wasmparser::ExternalKind::Func) {
                    exports.insert(export.name.to_string());
                }
            }
        }
    }
    exports
}

/// Validate that all qualified function calls (module.func) reference actual
/// exported functions in the imported WASM modules.
fn validate_wasm_imports(project: &ResolvedProject, errors: &mut Vec<CompileError>) {
    if project.imports.is_empty() {
        return;
    }

    // Build export map: module_name -> set of exported function names
    let mut export_map: HashMap<String, HashSet<String>> = HashMap::new();
    for imp in &project.imports {
        let exports = read_wasm_exports(&imp.wasm_path);
        export_map.insert(imp.name.clone(), exports);
    }

    // Walk all expressions in entry file looking for qualified function calls
    check_wasm_calls_in_nodes(&project.entry.nodes, &export_map, errors);
}

/// Validate that server data calls reference declared server functions with correct arg counts.
fn validate_server_calls(nodes: &[Node], errors: &mut Vec<CompileError>) {
    // Collect server function declarations: name -> param count
    let mut server_fns: HashMap<String, usize> = HashMap::new();
    collect_server_fn_decls(nodes, &mut server_fns);

    // Validate server data calls
    validate_server_data_refs(nodes, &server_fns, errors);
}

fn collect_server_fn_decls(nodes: &[Node], fns: &mut HashMap<String, usize>) {
    for node in nodes {
        match node {
            Node::ServerFunction { name, params, .. } => {
                fns.insert(name.clone(), params.len());
            }
            Node::App { children, .. } => {
                collect_server_fn_decls(children, fns);
            }
            _ => {}
        }
    }
}

fn validate_server_data_refs(
    nodes: &[Node],
    server_fns: &HashMap<String, usize>,
    errors: &mut Vec<CompileError>,
) {
    for node in nodes {
        match node {
            Node::ServerData {
                func_name,
                args,
                span,
                ..
            } => {
                if let Some(&expected) = server_fns.get(func_name) {
                    if args.len() != expected {
                        errors.push(CompileError {
                            message: format!(
                                "server function '{}' expects {} argument(s) but got {}",
                                func_name,
                                expected,
                                args.len()
                            ),
                            file: span.file.clone(),
                            line: span.line,
                            column: span.col,
                            severity: Severity::Error,
                        });
                    }
                } else {
                    errors.push(CompileError {
                        message: format!(
                            "unknown server function '{}' — declare it with `server function {}`",
                            func_name, func_name
                        ),
                        file: span.file.clone(),
                        line: span.line,
                        column: span.col,
                        severity: Severity::Error,
                    });
                }
            }
            Node::App { children, .. } | Node::Page { children, .. } => {
                validate_server_data_refs(children, server_fns, errors);
            }
            _ => {}
        }
    }
}

fn check_wasm_calls_in_nodes(
    nodes: &[Node],
    export_map: &HashMap<String, HashSet<String>>,
    errors: &mut Vec<CompileError>,
) {
    for node in nodes {
        match node {
            Node::App { children, .. } => {
                check_wasm_calls_in_nodes(children, export_map, errors);
            }
            Node::Element {
                children,
                handlers,
                props,
                ..
            } => {
                for handler in handlers {
                    check_wasm_calls_in_action(&handler.action, export_map, errors);
                }
                for prop in props {
                    check_wasm_calls_in_value(&prop.value, export_map, errors);
                }
                check_wasm_calls_in_nodes(children, export_map, errors);
            }
            Node::Computed { expr, .. } => {
                check_wasm_calls_in_expr(expr, export_map, errors);
            }
            Node::If {
                condition,
                then_children,
                else_children,
                ..
            } => {
                check_wasm_calls_in_expr(condition, export_map, errors);
                check_wasm_calls_in_nodes(then_children, export_map, errors);
                check_wasm_calls_in_nodes(else_children, export_map, errors);
            }
            Node::Each {
                iterable, children, ..
            } => {
                check_wasm_calls_in_expr(iterable, export_map, errors);
                check_wasm_calls_in_nodes(children, export_map, errors);
            }
            Node::Page { children, .. } => {
                check_wasm_calls_in_nodes(children, export_map, errors);
            }
            _ => {}
        }
    }
}

fn check_wasm_calls_in_expr(
    expr: &Expression,
    export_map: &HashMap<String, HashSet<String>>,
    errors: &mut Vec<CompileError>,
) {
    match expr {
        Expression::FunctionCall { name, args } => {
            if let Some(dot) = name.find('.') {
                let module = &name[..dot];
                let function = &name[dot + 1..];
                if let Some(exports) = export_map.get(module) {
                    if !exports.contains(function) {
                        errors.push(CompileError {
                            message: format!(
                                "WASM module '{}' does not export function '{}'",
                                module, function
                            ),
                            severity: Severity::Error,
                            file: String::new(),
                            line: 0,
                            column: 0,
                        });
                    }
                }
            }
            for arg in args {
                check_wasm_calls_in_expr(arg, export_map, errors);
            }
        }
        Expression::BinOp { left, right, .. } => {
            check_wasm_calls_in_expr(left, export_map, errors);
            check_wasm_calls_in_expr(right, export_map, errors);
        }
        Expression::Pipeline { source, stages } => {
            check_wasm_calls_in_expr(source, export_map, errors);
            for stage in stages {
                if let Some(arg) = &stage.argument {
                    check_wasm_calls_in_expr(arg, export_map, errors);
                }
                if let Some(arg) = &stage.argument2 {
                    check_wasm_calls_in_expr(arg, export_map, errors);
                }
            }
        }
        _ => {}
    }
}

fn check_wasm_calls_in_action(
    action: &Action,
    export_map: &HashMap<String, HashSet<String>>,
    errors: &mut Vec<CompileError>,
) {
    match action {
        Action::Set { expr, .. } => check_wasm_calls_in_expr(expr, export_map, errors),
        Action::Log { expr, .. } => check_wasm_calls_in_expr(expr, export_map, errors),
        Action::Copy { expr, .. } => check_wasm_calls_in_expr(expr, export_map, errors),
        Action::Send { expr, .. } => check_wasm_calls_in_expr(expr, export_map, errors),
        _ => {}
    }
}

#[allow(clippy::only_used_in_recursion)]
fn check_wasm_calls_in_value(
    value: &Value,
    export_map: &HashMap<String, HashSet<String>>,
    errors: &mut Vec<CompileError>,
) {
    match value {
        Value::List(items) => {
            for item in items {
                check_wasm_calls_in_value(item, export_map, errors);
            }
        }
        Value::Object(entries) => {
            for (_, v) in entries {
                check_wasm_calls_in_value(v, export_map, errors);
            }
        }
        _ => {}
    }
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
        let project = resolve(dir.path(), "app.naze", &[]);
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
            errs.iter()
                .any(|e| e.message.contains("missing required prop 'color'")),
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
            errs.iter()
                .any(|e| e.message.contains("type mismatch") && e.message.contains("color")),
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
            errs.iter()
                .any(|e| e.message.contains("unknown prop 'bogus'")),
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
            errs.iter()
                .any(|e| e.message.contains("type mismatch") && e.message.contains("width")),
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
            errs.iter()
                .any(|e| e.message.contains("type mismatch") && e.message.contains("width")),
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
            errors
                .iter()
                .any(|e| e.message.contains("unknown reference 'bogus'")),
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
            errs.iter()
                .any(|e| e.message.contains("type mismatch") && e.message.contains("gap")),
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
            let project = resolve(&examples_dir, name, &[]);
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
            let project = resolve(&examples_dir, name, &[]);
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
