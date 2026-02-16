//! Shared execution engine: state resolution, expression evaluation, action execution.
//! Used by test_runner.rs. The same logic exists in run.rs and gallery.rs (historical duplication).

use std::collections::HashMap;

use naze_ir::{
    IrAction, IrBinOp, IrExpression, IrPipelineStage, RenderNode, RenderTree, RenderValue, TextPart,
};
use naze_layout::PositionedNode;

// ─── State initialization ───────────────────────────────────────────────────

pub(crate) fn init_state(tree: &RenderTree) -> HashMap<String, RenderValue> {
    let mut store = HashMap::new();
    for decl in &tree.state {
        store.insert(decl.name.clone(), decl.initial.clone());
    }
    // Initialize computed values
    for decl in &tree.computed {
        let val = evaluate_expr(&decl.expr, &store);
        store.insert(decl.name.clone(), val);
    }
    store
}

// ─── Tree / node resolution ─────────────────────────────────────────────────

pub(crate) fn resolve_nodes(
    nodes: &[RenderNode],
    state: &HashMap<String, RenderValue>,
) -> Vec<RenderNode> {
    let mut out = Vec::new();
    for node in nodes {
        match node.kind.as_str() {
            "__if" => {
                let show_then =
                    node.condition
                        .as_ref()
                        .is_some_and(|cond| match evaluate_expr(cond, state) {
                            RenderValue::Bool(b) => b,
                            RenderValue::Num(n, _) => n != 0.0,
                            _ => false,
                        });
                if show_then {
                    out.extend(resolve_nodes(&node.children, state));
                } else if let Some(else_nodes) = &node.else_children {
                    out.extend(resolve_nodes(else_nodes, state));
                }
            }
            "__each" => {
                if let Some((var, iterable_expr)) = &node.each_binding {
                    if let RenderValue::List(items) = evaluate_expr(iterable_expr, state) {
                        for item in &items {
                            let mut child_state = state.clone();
                            child_state.insert(var.clone(), item.clone());
                            out.extend(resolve_nodes(&node.children, &child_state));
                        }
                    }
                }
            }
            _ => {
                let mut props: HashMap<String, RenderValue> = node
                    .props
                    .iter()
                    .map(|(k, v)| (k.clone(), resolve_value(v, state)))
                    .collect();

                // Resolve bind props for form elements
                if node.kind == "checkbox" {
                    if let Some(RenderValue::Bind(var)) = node.props.get("bind") {
                        let checked = match state.get(var) {
                            Some(RenderValue::Bool(b)) => *b,
                            _ => false,
                        };
                        props.insert("checked".to_string(), RenderValue::Bool(checked));
                    }
                } else if node.kind == "radio" {
                    if let Some(RenderValue::Bind(var)) = node.props.get("bind") {
                        let selected = match (state.get(var), node.props.get("value")) {
                            (Some(state_val), Some(prop_val)) => state_val == prop_val,
                            _ => false,
                        };
                        props.insert("selected".to_string(), RenderValue::Bool(selected));
                    }
                } else if node.kind == "input" || node.kind == "textarea" {
                    if let Some(RenderValue::Bind(var)) = node.props.get("bind") {
                        let value = match state.get(var) {
                            Some(RenderValue::Str(s)) => s.clone(),
                            _ => String::new(),
                        };
                        props.insert("value".to_string(), RenderValue::Str(value));
                    }
                } else if node.kind == "select" {
                    if let Some(RenderValue::Bind(var)) = node.props.get("bind") {
                        let value = match state.get(var) {
                            Some(RenderValue::Str(s)) => s.clone(),
                            _ => String::new(),
                        };
                        props.insert("selected".to_string(), RenderValue::Str(value));
                    }
                }

                out.push(RenderNode {
                    kind: node.kind.clone(),
                    props,
                    children: resolve_nodes(&node.children, state),
                    handlers: node.handlers.clone(),
                    condition: None,
                    else_children: None,
                    each_binding: None,
                    span: None,
                });
            }
        }
    }
    out
}

pub(crate) fn resolve_value(
    value: &RenderValue,
    state: &HashMap<String, RenderValue>,
) -> RenderValue {
    match value {
        RenderValue::InterpolatedStr(parts) => {
            // Single state ref → return raw value to preserve Color/Num types
            if parts.len() == 1 {
                if let TextPart::StateRef(name) = &parts[0] {
                    if let Some(val) = state.get(name.as_str()) {
                        return val.clone();
                    }
                }
            }
            let mut result = String::new();
            for part in parts {
                match part {
                    TextPart::Literal(s) => result.push_str(s),
                    TextPart::StateRef(name) => match state.get(name.as_str()) {
                        Some(RenderValue::Str(s)) => result.push_str(s),
                        Some(RenderValue::Num(n, _)) => {
                            if n.fract() == 0.0 {
                                result.push_str(&format!("{}", *n as i64));
                            } else {
                                result.push_str(&format!("{}", n));
                            }
                        }
                        Some(RenderValue::Bool(b)) => {
                            result.push_str(if *b { "true" } else { "false" });
                        }
                        Some(RenderValue::Color(c)) => {
                            result.push_str(&format!("#{:06x}", c));
                        }
                        _ => {
                            result.push('{');
                            result.push_str(name);
                            result.push('}');
                        }
                    },
                }
            }
            RenderValue::Str(result)
        }
        other => other.clone(),
    }
}

// ─── Expression evaluation ──────────────────────────────────────────────────

pub(crate) fn evaluate_expr(
    expr: &IrExpression,
    state: &HashMap<String, RenderValue>,
) -> RenderValue {
    match expr {
        IrExpression::Num(n) => RenderValue::Num(*n, None),
        IrExpression::Str(s) => RenderValue::Str(s.clone()),
        IrExpression::Bool(b) => RenderValue::Bool(*b),
        IrExpression::StateRef(name) => {
            if let Some(val) = state.get(name) {
                return val.clone();
            }
            if let Some(dot) = name.find('.') {
                let root = &name[..dot];
                let field = &name[dot + 1..];
                if let Some(RenderValue::Object(entries)) = state.get(root) {
                    for (k, v) in entries {
                        if k == field {
                            return v.clone();
                        }
                    }
                }
            }
            RenderValue::Num(0.0, None)
        }
        IrExpression::BinOp { left, op, right } => {
            let lval = evaluate_expr(left, state);
            let rval = evaluate_expr(right, state);
            eval_binop(&lval, op, &rval)
        }
        IrExpression::Pipeline { source, stages } => {
            let source_val = evaluate_expr(source, state);
            eval_pipeline(source_val, stages, state)
        }
        IrExpression::WasmCall { .. } => {
            // WASM imports not supported in CLI executor
            RenderValue::Num(0.0, None)
        }
        IrExpression::EnvRef(name) => {
            // Server-side env var resolution at runtime
            match std::env::var(name) {
                Ok(val) => RenderValue::Str(val),
                Err(_) => RenderValue::Str(String::new()),
            }
        }
        IrExpression::List(items) => {
            RenderValue::List(items.iter().map(|e| evaluate_expr(e, state)).collect())
        }
        IrExpression::Object(entries) => RenderValue::Object(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), evaluate_expr(v, state)))
                .collect(),
        ),
    }
}

fn eval_pipeline(
    source: RenderValue,
    stages: &[IrPipelineStage],
    state: &HashMap<String, RenderValue>,
) -> RenderValue {
    let mut current = source;
    for stage in stages {
        current = eval_pipeline_stage(current, stage, state);
    }
    current
}

fn eval_pipeline_stage(
    input: RenderValue,
    stage: &IrPipelineStage,
    state: &HashMap<String, RenderValue>,
) -> RenderValue {
    let items = match &input {
        RenderValue::List(items) => items.clone(),
        _ => return input,
    };
    match stage.function {
        0 => {
            // filter
            let arg = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            RenderValue::List(
                items
                    .into_iter()
                    .filter(|item| {
                        let mut s = state.clone();
                        s.insert("__it".to_string(), item.clone());
                        if let RenderValue::Object(entries) = item {
                            for (k, v) in entries {
                                s.insert(k.clone(), v.clone());
                            }
                        }
                        matches!(evaluate_expr(arg, &s), RenderValue::Bool(true))
                    })
                    .collect(),
            )
        }
        1 => {
            // map
            let arg = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            RenderValue::List(
                items
                    .into_iter()
                    .map(|item| {
                        let mut s = state.clone();
                        s.insert("__it".to_string(), item.clone());
                        if let RenderValue::Object(entries) = &item {
                            for (k, v) in entries {
                                s.insert(k.clone(), v.clone());
                            }
                        }
                        evaluate_expr(arg, &s)
                    })
                    .collect(),
            )
        }
        2 => {
            // sort-by
            let arg = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            let mut sorted = items;
            sorted.sort_by(|a, b| {
                let mut sa = state.clone();
                sa.insert("__it".to_string(), a.clone());
                if let RenderValue::Object(e) = a {
                    for (k, v) in e {
                        sa.insert(k.clone(), v.clone());
                    }
                }
                let mut sb = state.clone();
                sb.insert("__it".to_string(), b.clone());
                if let RenderValue::Object(e) = b {
                    for (k, v) in e {
                        sb.insert(k.clone(), v.clone());
                    }
                }
                let ak = evaluate_expr(arg, &sa);
                let bk = evaluate_expr(arg, &sb);
                match (&ak, &bk) {
                    (RenderValue::Num(an, _), RenderValue::Num(bn, _)) => {
                        an.partial_cmp(bn).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (RenderValue::Str(a), RenderValue::Str(b)) => a.cmp(b),
                    _ => std::cmp::Ordering::Equal,
                }
            });
            RenderValue::List(sorted)
        }
        3 => {
            // take
            let n = match &stage.argument {
                Some(a) => match evaluate_expr(a, state) {
                    RenderValue::Num(n, _) => n as usize,
                    _ => items.len(),
                },
                None => items.len(),
            };
            RenderValue::List(items.into_iter().take(n).collect())
        }
        4 => {
            // sum
            let total: f64 = items
                .iter()
                .filter_map(|i| {
                    if let RenderValue::Num(n, _) = i {
                        Some(n)
                    } else {
                        None
                    }
                })
                .sum();
            RenderValue::Num(total, None)
        }
        5 => RenderValue::Num(items.len() as f64, None), // count
        6 => {
            // reduce
            let acc_expr = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            let initial = match &stage.argument2 {
                Some(init) => evaluate_expr(init, state),
                None => RenderValue::Num(0.0, None),
            };
            let mut acc = initial;
            for item in &items {
                let mut s = state.clone();
                s.insert("__it".to_string(), item.clone());
                if let RenderValue::Object(entries) = item {
                    for (k, v) in entries {
                        s.insert(k.clone(), v.clone());
                    }
                }
                s.insert("acc".to_string(), acc.clone());
                acc = evaluate_expr(acc_expr, &s);
            }
            acc
        }
        7 => {
            // group-by
            let arg = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            let mut groups: Vec<(String, Vec<RenderValue>)> = Vec::new();
            for item in items {
                let mut s = state.clone();
                s.insert("__it".to_string(), item.clone());
                if let RenderValue::Object(entries) = &item {
                    for (k, v) in entries {
                        s.insert(k.clone(), v.clone());
                    }
                }
                let key = render_value_to_string(&evaluate_expr(arg, &s));
                if let Some(group) = groups.iter_mut().find(|(k, _)| k == &key) {
                    group.1.push(item);
                } else {
                    groups.push((key, vec![item]));
                }
            }
            RenderValue::Object(
                groups
                    .into_iter()
                    .map(|(k, v)| (k, RenderValue::List(v)))
                    .collect(),
            )
        }
        8 => {
            // flatten
            let mut flattened = Vec::new();
            for item in items {
                match item {
                    RenderValue::List(inner) => flattened.extend(inner),
                    other => flattened.push(other),
                }
            }
            RenderValue::List(flattened)
        }
        9 => {
            // distinct
            let mut seen = Vec::new();
            let mut result = Vec::new();
            for item in items {
                let key = match &stage.argument {
                    Some(arg) => {
                        let mut s = state.clone();
                        s.insert("__it".to_string(), item.clone());
                        if let RenderValue::Object(entries) = &item {
                            for (k, v) in entries {
                                s.insert(k.clone(), v.clone());
                            }
                        }
                        render_value_to_string(&evaluate_expr(arg, &s))
                    }
                    None => render_value_to_string(&item),
                };
                if !seen.contains(&key) {
                    seen.push(key);
                    result.push(item);
                }
            }
            RenderValue::List(result)
        }
        _ => RenderValue::List(items),
    }
}

fn eval_binop(left: &RenderValue, op: &IrBinOp, right: &RenderValue) -> RenderValue {
    let left_num = match left {
        RenderValue::Num(n, _) => Some(*n),
        RenderValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    };
    let right_num = match right {
        RenderValue::Num(n, _) => Some(*n),
        RenderValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    };

    match op {
        IrBinOp::Add => {
            if let (Some(l), Some(r)) = (left_num, right_num) {
                RenderValue::Num(l + r, None)
            } else if let (RenderValue::List(ll), RenderValue::List(rl)) = (left, right) {
                let mut result = ll.clone();
                result.extend(rl.iter().cloned());
                RenderValue::List(result)
            } else {
                RenderValue::Str(format!(
                    "{}{}",
                    render_value_to_string(left),
                    render_value_to_string(right)
                ))
            }
        }
        IrBinOp::Sub => RenderValue::Num(left_num.unwrap_or(0.0) - right_num.unwrap_or(0.0), None),
        IrBinOp::Mul => RenderValue::Num(left_num.unwrap_or(0.0) * right_num.unwrap_or(0.0), None),
        IrBinOp::Div => {
            let r = right_num.unwrap_or(1.0);
            let r = if r == 0.0 { 1.0 } else { r };
            RenderValue::Num(left_num.unwrap_or(0.0) / r, None)
        }
        IrBinOp::Eq => RenderValue::Bool(left_num == right_num),
        IrBinOp::Neq => RenderValue::Bool(left_num != right_num),
        IrBinOp::Gt => RenderValue::Bool(left_num.unwrap_or(0.0) > right_num.unwrap_or(0.0)),
        IrBinOp::Lt => RenderValue::Bool(left_num.unwrap_or(0.0) < right_num.unwrap_or(0.0)),
        IrBinOp::Gte => RenderValue::Bool(left_num.unwrap_or(0.0) >= right_num.unwrap_or(0.0)),
        IrBinOp::Lte => RenderValue::Bool(left_num.unwrap_or(0.0) <= right_num.unwrap_or(0.0)),
        IrBinOp::And => {
            let l = match left {
                RenderValue::Bool(b) => *b,
                _ => left_num.unwrap_or(0.0) != 0.0,
            };
            let r = match right {
                RenderValue::Bool(b) => *b,
                _ => right_num.unwrap_or(0.0) != 0.0,
            };
            RenderValue::Bool(l && r)
        }
        IrBinOp::Or => {
            let l = match left {
                RenderValue::Bool(b) => *b,
                _ => left_num.unwrap_or(0.0) != 0.0,
            };
            let r = match right {
                RenderValue::Bool(b) => *b,
                _ => right_num.unwrap_or(0.0) != 0.0,
            };
            RenderValue::Bool(l || r)
        }
    }
}

// ─── Action execution ───────────────────────────────────────────────────────

pub(crate) fn execute_action(action: &IrAction, state: &mut HashMap<String, RenderValue>, themes: &[naze_ir::ThemeDef]) -> bool {
    match action {
        IrAction::Set { target, expr } => {
            let value = evaluate_expr(expr, state);
            state.insert(target.clone(), value);
            true
        }
        IrAction::Navigate { .. } => false,
        IrAction::ScrollTo { .. } => false,
        IrAction::Log { expr } => {
            let value = evaluate_expr(expr, state);
            let msg = match &value {
                RenderValue::Str(s) => s.clone(),
                RenderValue::Num(n, _) => {
                    if n.fract() == 0.0 {
                        format!("{}", *n as i64)
                    } else {
                        format!("{}", n)
                    }
                }
                RenderValue::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
                RenderValue::Color(c) => format!("#{:06x}", c),
                _ => format!("{:?}", value),
            };
            eprintln!("[log] {}", msg);
            false
        }
        IrAction::Append { item, target } => {
            let item_value = evaluate_expr(item, state);
            if let Some(RenderValue::List(list)) = state.get_mut(target) {
                list.push(item_value);
                true
            } else {
                false
            }
        }
        IrAction::Remove { index, target } => {
            let idx_value = evaluate_expr(index, state);
            if let RenderValue::Num(idx, _) = idx_value {
                let idx = idx as usize;
                if let Some(RenderValue::List(list)) = state.get_mut(target) {
                    if idx < list.len() {
                        list.remove(idx);
                        return true;
                    }
                }
            }
            false
        }
        IrAction::SetTheme { name } => {
            if let Some(theme) = themes.iter().find(|t| t.name == *name) {
                for (token, color) in &theme.colors {
                    state.insert(format!("theme.colors.{}", token), RenderValue::Color(*color));
                }
                for (token, value) in &theme.spacing {
                    state.insert(format!("theme.spacing.{}", token), RenderValue::Num(*value, Some("px".into())));
                }
            }
            state.insert("active-theme".to_string(), RenderValue::Str(name.clone()));
            true
        }
        _ => false,
    }
}

// ─── Hit testing ────────────────────────────────────────────────────────────

pub(crate) fn find_click_handlers(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
    state: &HashMap<String, RenderValue>,
) -> Vec<naze_ir::IrEventHandler> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        let child_handlers = find_click_handlers(&node.children, x, y, state);
        if !child_handlers.is_empty() {
            return child_handlers;
        }

        // For checkbox/radio, generate toggle handlers
        if node.kind == "checkbox" {
            if let Some(RenderValue::Bind(var)) = node.props.get("bind") {
                let current = match state.get(var) {
                    Some(RenderValue::Bool(b)) => *b,
                    _ => false,
                };
                let mut handlers: Vec<naze_ir::IrEventHandler> = vec![naze_ir::IrEventHandler {
                    event: "click".to_string(),
                    action: IrAction::Set {
                        target: var.clone(),
                        expr: IrExpression::Bool(!current),
                    },
                    modifier_kind: 0,
                    modifier_ms: 0,
                }];
                handlers.extend(
                    node.handlers
                        .iter()
                        .filter(|h| h.event == "change")
                        .cloned(),
                );
                return handlers;
            }
        } else if node.kind == "radio" {
            if let (Some(RenderValue::Bind(var)), Some(value)) =
                (node.props.get("bind"), node.props.get("value"))
            {
                let value_str = match value {
                    RenderValue::Str(s) => s.clone(),
                    _ => continue,
                };
                let mut handlers: Vec<naze_ir::IrEventHandler> = vec![naze_ir::IrEventHandler {
                    event: "click".to_string(),
                    action: IrAction::Set {
                        target: var.clone(),
                        expr: IrExpression::Str(value_str),
                    },
                    modifier_kind: 0,
                    modifier_ms: 0,
                }];
                handlers.extend(
                    node.handlers
                        .iter()
                        .filter(|h| h.event == "change")
                        .cloned(),
                );
                return handlers;
            }
        }

        let click_handlers: Vec<_> = node
            .handlers
            .iter()
            .filter(|h| h.event == "click")
            .cloned()
            .collect();
        if !click_handlers.is_empty() {
            return click_handlers;
        }
    }
    Vec::new()
}

pub(crate) fn point_in_node(node: &PositionedNode, x: f32, y: f32) -> bool {
    x >= node.x && x <= node.x + node.width && y >= node.y && y <= node.y + node.height
}

// ─── Element search (for test assertions) ───────────────────────────────────

/// Find a PositionedNode whose visible text matches the given string.
pub(crate) fn find_element_by_text<'a>(
    nodes: &'a [PositionedNode],
    target_text: &str,
) -> Option<&'a PositionedNode> {
    for node in nodes {
        // Check __text prop (resolved text content)
        if let Some(RenderValue::Str(text)) = node.props.get("__text") {
            if text == target_text {
                return Some(node);
            }
        }
        // Check label prop (for checkbox/radio/button-like elements)
        if let Some(RenderValue::Str(label)) = node.props.get("label") {
            if label == target_text {
                return Some(node);
            }
        }
        // Recurse into children
        if let Some(found) = find_element_by_text(&node.children, target_text) {
            return Some(found);
        }
    }
    None
}

/// Find an input/textarea element by its placeholder or label text.
pub(crate) fn find_input_by_label<'a>(
    nodes: &'a [PositionedNode],
    label: &str,
) -> Option<&'a PositionedNode> {
    for node in nodes {
        if node.kind == "input" || node.kind == "textarea" {
            if let Some(RenderValue::Str(placeholder)) = node.props.get("placeholder") {
                if placeholder == label {
                    return Some(node);
                }
            }
            if let Some(RenderValue::Str(lbl)) = node.props.get("label") {
                if lbl == label {
                    return Some(node);
                }
            }
        }
        if let Some(found) = find_input_by_label(&node.children, label) {
            return Some(found);
        }
    }
    None
}

/// Check if any element with the given text is visible in the layout tree.
pub(crate) fn is_text_visible(nodes: &[PositionedNode], target_text: &str) -> bool {
    find_element_by_text(nodes, target_text).is_some()
}

// ─── RenderNode text search (for test runner) ───────────────────────────────
// Layout drops children of leaf nodes (rect, text, etc). These functions
// search the resolved RenderNode tree which preserves all children.

/// Check if any RenderNode in the tree has text matching the target.
pub(crate) fn is_text_in_render_nodes(nodes: &[RenderNode], target_text: &str) -> bool {
    for node in nodes {
        if let Some(RenderValue::Str(text)) = node.props.get("__text") {
            if text == target_text {
                return true;
            }
        }
        if let Some(RenderValue::Str(label)) = node.props.get("label") {
            if label == target_text {
                return true;
            }
        }
        if is_text_in_render_nodes(&node.children, target_text) {
            return true;
        }
    }
    false
}

/// Find a RenderNode in the layout tree that contains the target text as a
/// child, and that has a corresponding PositionedNode (for click coordinates).
/// Returns the deepest PositionedNode whose render-tree counterpart contains the text.
pub(crate) fn find_clickable_for_text<'a>(
    layout_nodes: &'a [PositionedNode],
    render_nodes: &[RenderNode],
    target_text: &str,
) -> Option<&'a PositionedNode> {
    // Walk layout and render nodes in parallel
    for (layout_node, render_node) in layout_nodes.iter().zip(render_nodes.iter()) {
        if !render_node_contains_text(render_node, target_text) {
            continue;
        }
        // Prefer deeper matches: recurse into children first
        if let Some(found) =
            find_clickable_for_text(&layout_node.children, &render_node.children, target_text)
        {
            return Some(found);
        }
        // No deeper match — this node is the most specific container
        return Some(layout_node);
    }
    None
}

/// Check if a RenderNode or any of its children contain the target text.
fn render_node_contains_text(node: &RenderNode, target_text: &str) -> bool {
    if let Some(RenderValue::Str(text)) = node.props.get("__text") {
        if text == target_text {
            return true;
        }
    }
    if let Some(RenderValue::Str(label)) = node.props.get("label") {
        if label == target_text {
            return true;
        }
    }
    for child in &node.children {
        if render_node_contains_text(child, target_text) {
            return true;
        }
    }
    false
}

/// Find an input RenderNode by placeholder/label, returning its PositionedNode counterpart.
pub(crate) fn find_input_in_render_nodes<'a>(
    layout_nodes: &'a [PositionedNode],
    render_nodes: &[RenderNode],
    label: &str,
) -> Option<&'a PositionedNode> {
    for (layout_node, render_node) in layout_nodes.iter().zip(render_nodes.iter()) {
        if render_node.kind == "input" || render_node.kind == "textarea" {
            if let Some(RenderValue::Str(placeholder)) = render_node.props.get("placeholder") {
                if placeholder == label {
                    return Some(layout_node);
                }
            }
            if let Some(RenderValue::Str(lbl)) = render_node.props.get("label") {
                if lbl == label {
                    return Some(layout_node);
                }
            }
        }
        if let Some(found) =
            find_input_in_render_nodes(&layout_node.children, &render_node.children, label)
        {
            return Some(found);
        }
    }
    None
}

// ─── Utility ────────────────────────────────────────────────────────────────

pub(crate) fn render_value_to_string(v: &RenderValue) -> String {
    match v {
        RenderValue::Str(s) => s.clone(),
        RenderValue::Num(n, _) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        RenderValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        RenderValue::Color(c) => format!("#{:06x}", c),
        _ => String::new(),
    }
}
