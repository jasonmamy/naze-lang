use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use naze_ir::{IrAction, IrBinOp, IrExpression, RenderNode, RenderTree, RenderValue, TextPart};
use naze_layout::{self, LayoutTree, PositionedNode};
use naze_renderer::{self, canvas::Renderer};

// ─── App state ──────────────────────────────────────────────────────────────

/// Persistent app state for the render loop.
struct App {
    render_tree: RenderTree,
    state_store: HashMap<String, RenderValue>,
    renderer: Renderer,
    layout: Option<LayoutTree>,
    raf_pending: bool,
}

thread_local! {
    static APP: RefCell<Option<App>> = RefCell::new(None);
}

// ─── Entry point ────────────────────────────────────────────────────────────

/// Entry point called from JavaScript.
/// `app_data` is a binary-encoded RenderTree.
/// `canvas_id` is the HTML id of the canvas element to render into.
#[wasm_bindgen]
pub fn start(app_data: &[u8], canvas_id: &str) -> Result<(), JsValue> {
    // 1. Deserialize the render tree
    let render_tree: RenderTree = naze_ir::deserialize(app_data)
        .map_err(|e| JsValue::from_str(&format!("failed to deserialize app data: {}", e)))?;

    // 2. Initialize state store from declarations
    let mut state_store = HashMap::new();
    for decl in &render_tree.state {
        state_store.insert(decl.name.clone(), decl.initial.clone());
    }

    // 3. Set up the renderer
    let renderer = Renderer::new(canvas_id)?;

    // 4. Store in global for render loop and future event handlers
    APP.with(|cell| {
        *cell.borrow_mut() = Some(App {
            render_tree,
            state_store,
            renderer,
            layout: None,
            raf_pending: false,
        });
    });

    // 5. Initial render (synchronous)
    do_render()?;

    // 6. Set up event listeners on the canvas
    setup_event_listeners()?;

    Ok(())
}

// ─── Render loop ────────────────────────────────────────────────────────────

/// Perform a full render: resolve state → layout → draw.
fn do_render() -> Result<(), JsValue> {
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let app = borrow
            .as_mut()
            .ok_or_else(|| JsValue::from_str("app not initialized"))?;
        app.raf_pending = false;

        // 1. Resolve interpolated strings against current state
        let resolved = resolve_tree(&app.render_tree, &app.state_store);

        // 2. Get viewport size
        let window = web_sys::window().ok_or("no window")?;
        let vw = window.inner_width()?.as_f64().unwrap_or(1024.0) as f32;
        let vh = window.inner_height()?.as_f64().unwrap_or(768.0) as f32;

        // 3. Set canvas size to viewport
        app.renderer.set_size(vw as f64, vh as f64);

        // 4. Compute layout
        let layout = {
            let renderer = &app.renderer;
            let text_measure = |text: &str, font_size: f32| -> (f32, f32) {
                let is_heading = font_size > 20.0;
                let (w, h) = renderer.measure_text(text, font_size as f64, is_heading);
                (w as f32, h as f32)
            };
            naze_layout::compute_layout_with_measure(&resolved, vw, vh, text_measure)
        };

        // 5. Set document title
        if let Some(document) = window.document() {
            document.set_title(&layout.title);
        }

        // 6. Clear and draw
        app.renderer.clear();
        draw_tree(&app.renderer, &layout);

        // 7. Store layout for hit testing
        app.layout = Some(layout);

        Ok(())
    })
}

/// Schedule a render on the next animation frame.
/// Coalesces multiple calls — only one frame is scheduled at a time.
fn schedule_render() {
    let already_pending = APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if let Some(app) = borrow.as_mut() {
            if app.raf_pending {
                return true;
            }
            app.raf_pending = true;
        }
        false
    });

    if already_pending {
        return;
    }

    let cb = Closure::once(|| {
        let _ = do_render();
    });

    if let Some(window) = web_sys::window() {
        let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
    }
    cb.forget(); // Runs once per state change; acceptable for M1
}

// ─── State resolution ───────────────────────────────────────────────────────

/// Create a copy of the render tree with all InterpolatedStr values resolved
/// to plain Str values using the current state store.
fn resolve_tree(tree: &RenderTree, state: &HashMap<String, RenderValue>) -> RenderTree {
    RenderTree {
        title: tree.title.clone(),
        state: tree.state.clone(),
        root: resolve_nodes(&tree.root, state),
    }
}

fn resolve_nodes(nodes: &[RenderNode], state: &HashMap<String, RenderValue>) -> Vec<RenderNode> {
    nodes
        .iter()
        .map(|node| RenderNode {
            kind: node.kind.clone(),
            props: node
                .props
                .iter()
                .map(|(k, v)| (k.clone(), resolve_value(v, state)))
                .collect(),
            children: resolve_nodes(&node.children, state),
            handlers: node.handlers.clone(),
        })
        .collect()
}

/// Resolve a single value. InterpolatedStr parts are concatenated into a plain Str.
fn resolve_value(value: &RenderValue, state: &HashMap<String, RenderValue>) -> RenderValue {
    match value {
        RenderValue::InterpolatedStr(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    TextPart::Literal(s) => result.push_str(s),
                    TextPart::StateRef(name) => match state.get(name.as_str()) {
                        Some(RenderValue::Str(s)) => result.push_str(s),
                        Some(RenderValue::Num(n, _)) => {
                            // Format integers without trailing .0
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
                            // Unresolved ref — show placeholder
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

// ─── Drawing ────────────────────────────────────────────────────────────────

/// Walk the positioned tree and draw all nodes.
fn draw_tree(renderer: &Renderer, layout: &LayoutTree) {
    for node in &layout.root {
        draw_node(renderer, node);
    }
}

/// Recursively draw a positioned node and its children.
fn draw_node(renderer: &Renderer, node: &PositionedNode) {
    let x = node.x as f64;
    let y = node.y as f64;
    let w = node.width as f64;
    let h = node.height as f64;

    match node.kind.as_str() {
        "rect" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            renderer.draw_rect(x, y, w, h, &color, radius);
        }
        "container" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            if !color.is_empty() {
                renderer.draw_rect(x, y, w, h, &color, radius);
            }
            for child in &node.children {
                draw_node(renderer, child);
            }
        }
        "text" => {
            let text = naze_renderer::get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = naze_renderer::get_font_size(&node.props, false);
                let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
                renderer.draw_text(&text, x, y, font_size, false, &color);
            }
        }
        "heading" => {
            let text = naze_renderer::get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = naze_renderer::get_font_size(&node.props, true);
                let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
                renderer.draw_text(&text, x, y, font_size, true, &color);
            }
        }
        "row" | "column" | "stack" | "grid" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "");
            if !color.is_empty() {
                renderer.draw_rect(x, y, w, h, &color, 0.0);
            }
            for child in &node.children {
                draw_node(renderer, child);
            }
        }
        "spacer" => {}
        _ => {
            for child in &node.children {
                draw_node(renderer, child);
            }
        }
    }
}

// ─── Event handling ──────────────────────────────────────────────────────────

/// Set up click and mousemove event listeners on the canvas.
fn setup_event_listeners() -> Result<(), JsValue> {
    // Get canvas element from the app
    let canvas = APP.with(|cell| {
        let borrow = cell.borrow();
        let app = borrow.as_ref().ok_or("app not initialized")?;
        Ok::<_, &str>(app.renderer.canvas_element().clone())
    }).map_err(|e| JsValue::from_str(e))?;

    // Click handler
    let click_cb = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let needs_render = handle_click(x, y);
        if needs_render {
            schedule_render();
        }
    });
    canvas.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref())?;
    click_cb.forget();

    // Mousemove handler — set cursor to pointer over clickable nodes
    let move_cb = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let is_clickable = APP.with(|cell| {
            let borrow = cell.borrow();
            if let Some(app) = borrow.as_ref() {
                if let Some(layout) = &app.layout {
                    return hit_test_any_handler(&layout.root, x, y, "click");
                }
            }
            false
        });
        APP.with(|cell| {
            let borrow = cell.borrow();
            if let Some(app) = borrow.as_ref() {
                app.renderer.set_cursor(if is_clickable { "pointer" } else { "default" });
            }
        });
    });
    canvas.add_event_listener_with_callback("mousemove", move_cb.as_ref().unchecked_ref())?;
    move_cb.forget();

    Ok(())
}

/// Handle a click at (x, y). Walks the layout tree, finds the deepest node
/// at that point with click handlers, executes them, and returns whether
/// the state was changed (needs re-render).
fn handle_click(x: f32, y: f32) -> bool {
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l,
            None => return false,
        };

        // Find deepest node at (x, y) with click handlers
        let handlers = find_click_handlers(&layout.root, x, y);
        if handlers.is_empty() {
            return false;
        }

        let mut changed = false;
        for handler in &handlers {
            if execute_action(&handler.action, &mut app.state_store) {
                changed = true;
            }
        }
        changed
    })
}

/// Check if any node at (x, y) has a handler for the given event.
fn hit_test_any_handler(nodes: &[PositionedNode], x: f32, y: f32, event: &str) -> bool {
    // Walk depth-first; check deepest children first
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        if hit_test_any_handler(&node.children, x, y, event) {
            return true;
        }
        if node.handlers.iter().any(|h| h.event == event) {
            return true;
        }
    }
    false
}

/// Find click handlers on the deepest node at (x, y).
fn find_click_handlers(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
) -> Vec<naze_ir::IrEventHandler> {
    // Walk depth-first; deepest match wins
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first (deeper wins)
        let child_handlers = find_click_handlers(&node.children, x, y);
        if !child_handlers.is_empty() {
            return child_handlers;
        }
        // This node's click handlers
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

/// Check if a point is inside a node's bounding box.
fn point_in_node(node: &PositionedNode, x: f32, y: f32) -> bool {
    x >= node.x && x <= node.x + node.width && y >= node.y && y <= node.y + node.height
}

// ─── Action execution ────────────────────────────────────────────────────────

/// Execute an action, mutating the state store. Returns true if state was changed.
fn execute_action(
    action: &IrAction,
    state: &mut HashMap<String, RenderValue>,
) -> bool {
    match action {
        IrAction::Set { target, expr } => {
            let value = evaluate_expr(expr, state);
            state.insert(target.clone(), value);
            true
        }
        IrAction::Navigate { .. } => {
            // Navigate not implemented in M2
            false
        }
    }
}

// ─── Expression evaluation ───────────────────────────────────────────────────

/// Evaluate an expression against the current state.
fn evaluate_expr(
    expr: &IrExpression,
    state: &HashMap<String, RenderValue>,
) -> RenderValue {
    match expr {
        IrExpression::Num(n) => RenderValue::Num(*n, None),
        IrExpression::Str(s) => RenderValue::Str(s.clone()),
        IrExpression::Bool(b) => RenderValue::Bool(*b),
        IrExpression::StateRef(name) => {
            state.get(name).cloned().unwrap_or(RenderValue::Num(0.0, None))
        }
        IrExpression::BinOp { left, op, right } => {
            let lval = evaluate_expr(left, state);
            let rval = evaluate_expr(right, state);
            eval_binop(&lval, op, &rval)
        }
    }
}

/// Evaluate a binary operation on two RenderValues.
fn eval_binop(left: &RenderValue, op: &IrBinOp, right: &RenderValue) -> RenderValue {
    // Extract numeric values
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
        // Arithmetic ops — produce Num
        IrBinOp::Add => {
            if let (Some(l), Some(r)) = (left_num, right_num) {
                RenderValue::Num(l + r, None)
            } else {
                // String concatenation fallback
                let ls = render_value_to_string(left);
                let rs = render_value_to_string(right);
                RenderValue::Str(format!("{}{}", ls, rs))
            }
        }
        IrBinOp::Sub => RenderValue::Num(
            left_num.unwrap_or(0.0) - right_num.unwrap_or(0.0),
            None,
        ),
        IrBinOp::Mul => RenderValue::Num(
            left_num.unwrap_or(0.0) * right_num.unwrap_or(0.0),
            None,
        ),
        IrBinOp::Div => {
            let r = right_num.unwrap_or(1.0);
            let r = if r == 0.0 { 1.0 } else { r };
            RenderValue::Num(left_num.unwrap_or(0.0) / r, None)
        }
        // Comparison ops — produce Bool
        IrBinOp::Eq => RenderValue::Bool(left_num == right_num),
        IrBinOp::Neq => RenderValue::Bool(left_num != right_num),
        IrBinOp::Gt => RenderValue::Bool(left_num.unwrap_or(0.0) > right_num.unwrap_or(0.0)),
        IrBinOp::Lt => RenderValue::Bool(left_num.unwrap_or(0.0) < right_num.unwrap_or(0.0)),
        IrBinOp::Gte => RenderValue::Bool(left_num.unwrap_or(0.0) >= right_num.unwrap_or(0.0)),
        IrBinOp::Lte => RenderValue::Bool(left_num.unwrap_or(0.0) <= right_num.unwrap_or(0.0)),
        // Boolean ops — produce Bool
        IrBinOp::And => {
            let l = match left { RenderValue::Bool(b) => *b, _ => left_num.unwrap_or(0.0) != 0.0 };
            let r = match right { RenderValue::Bool(b) => *b, _ => right_num.unwrap_or(0.0) != 0.0 };
            RenderValue::Bool(l && r)
        }
        IrBinOp::Or => {
            let l = match left { RenderValue::Bool(b) => *b, _ => left_num.unwrap_or(0.0) != 0.0 };
            let r = match right { RenderValue::Bool(b) => *b, _ => right_num.unwrap_or(0.0) != 0.0 };
            RenderValue::Bool(l || r)
        }
    }
}

fn render_value_to_string(v: &RenderValue) -> String {
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
