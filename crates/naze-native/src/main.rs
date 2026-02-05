mod renderer;

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use naze_ir::{IrAction, IrBinOp, IrExpression, RenderNode, RenderTree, RenderValue, TextPart};
use naze_layout::{LayoutTree, PositionedNode};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{CursorIcon, Window, WindowId};

struct App {
    render_tree: RenderTree,
    state_store: HashMap<String, RenderValue>,
    font: fontdue::Font,
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    layout: Option<LayoutTree>,
    cursor_pos: Option<(f32, f32)>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(&self.render_tree.title)
            .with_inner_size(LogicalSize::new(1024.0f64, 768.0));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        self.window = Some(window.clone());
        self.surface = Some(surface);
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Some((position.x as f32, position.y as f32));
                // Update cursor icon based on whether we're over a clickable node
                if let (Some(layout), Some(window)) = (&self.layout, &self.window) {
                    let is_clickable = hit_test_any_handler(
                        &layout.root,
                        position.x as f32,
                        position.y as f32,
                        "click",
                    );
                    window.set_cursor(if is_clickable {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::Default
                    });
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if let Some((x, y)) = self.cursor_pos {
                    if self.handle_click(x, y) {
                        self.render();
                    }
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn render(&mut self) {
        let window = match &self.window {
            Some(w) => w,
            None => return,
        };
        let size = window.inner_size();
        let w = size.width;
        let h = size.height;
        if w == 0 || h == 0 {
            return;
        }

        // Resolve interpolated strings against state, then compute layout
        let resolved = resolve_tree(&self.render_tree, &self.state_store);
        let layout = naze_layout::compute_layout(&resolved, w as f32, h as f32);

        // Store layout for hit testing
        self.layout = Some(layout.clone());

        // Rasterize into a pixel buffer
        let mut pixmap = match tiny_skia::Pixmap::new(w, h) {
            Some(p) => p,
            None => return,
        };
        pixmap.fill(tiny_skia::Color::WHITE);
        renderer::draw_tree(&mut pixmap, &layout, &self.font);

        // Blit to window via softbuffer
        let surface = match &mut self.surface {
            Some(s) => s,
            None => return,
        };
        surface
            .resize(
                NonZeroU32::new(w).unwrap(),
                NonZeroU32::new(h).unwrap(),
            )
            .unwrap();
        let mut buffer = surface.buffer_mut().unwrap();

        // Convert tiny-skia premultiplied RGBA to softbuffer 0x00RRGGBB
        let pixels = pixmap.data();
        for i in 0..(w * h) as usize {
            let r = pixels[i * 4] as u32;
            let g = pixels[i * 4 + 1] as u32;
            let b = pixels[i * 4 + 2] as u32;
            buffer[i] = (r << 16) | (g << 8) | b;
        }
        buffer.present().unwrap();
    }

    fn handle_click(&mut self, x: f32, y: f32) -> bool {
        let layout = match &self.layout {
            Some(l) => l,
            None => return false,
        };
        let handlers = find_click_handlers(&layout.root, x, y);
        if handlers.is_empty() {
            return false;
        }
        let mut changed = false;
        for handler in &handlers {
            if execute_action(&handler.action, &mut self.state_store) {
                changed = true;
            }
        }
        changed
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "dist/app_data.bin".to_string());

    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {}", path, e);
        std::process::exit(1);
    });

    let render_tree = naze_ir::deserialize(&bytes).unwrap_or_else(|e| {
        eprintln!("error: cannot deserialize app data: {}", e);
        std::process::exit(1);
    });

    let font_bytes = include_bytes!("../fonts/DejaVuSans.ttf");
    let font = fontdue::Font::from_bytes(font_bytes as &[u8], fontdue::FontSettings::default())
        .unwrap_or_else(|e| {
            eprintln!("error: cannot load font: {}", e);
            std::process::exit(1);
        });

    // Initialize state store from declarations
    let mut state_store = HashMap::new();
    for decl in &render_tree.state {
        state_store.insert(decl.name.clone(), decl.initial.clone());
    }

    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        render_tree,
        state_store,
        font,
        window: None,
        surface: None,
        layout: None,
        cursor_pos: None,
    };
    event_loop.run_app(&mut app).unwrap();
}

// ─── State resolution ───────────────────────────────────────────────────────

fn resolve_tree(tree: &RenderTree, state: &HashMap<String, RenderValue>) -> RenderTree {
    RenderTree {
        title: tree.title.clone(),
        state: tree.state.clone(),
        data: tree.data.clone(),
        root: resolve_nodes(&tree.root, state),
        pages: tree.pages.clone(),
    }
}

fn resolve_nodes(nodes: &[RenderNode], state: &HashMap<String, RenderValue>) -> Vec<RenderNode> {
    let mut out = Vec::new();
    for node in nodes {
        match node.kind.as_str() {
            "__if" => {
                let show_then = node.condition.as_ref().map_or(false, |cond| {
                    match evaluate_expr(cond, state) {
                        RenderValue::Bool(b) => b,
                        RenderValue::Num(n, _) => n != 0.0,
                        _ => false,
                    }
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
                out.push(RenderNode {
                    kind: node.kind.clone(),
                    props: node
                        .props
                        .iter()
                        .map(|(k, v)| (k.clone(), resolve_value(v, state)))
                        .collect(),
                    children: resolve_nodes(&node.children, state),
                    handlers: node.handlers.clone(),
                    condition: None,
                    else_children: None,
                    each_binding: None,
                });
            }
        }
    }
    out
}

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

// ─── Hit testing ─────────────────────────────────────────────────────────────

fn hit_test_any_handler(nodes: &[PositionedNode], x: f32, y: f32, event: &str) -> bool {
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

fn find_click_handlers(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
) -> Vec<naze_ir::IrEventHandler> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        let child_handlers = find_click_handlers(&node.children, x, y);
        if !child_handlers.is_empty() {
            return child_handlers;
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

fn point_in_node(node: &PositionedNode, x: f32, y: f32) -> bool {
    x >= node.x && x <= node.x + node.width && y >= node.y && y <= node.y + node.height
}

// ─── Action execution ────────────────────────────────────────────────────────

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
        IrAction::Navigate { .. } => false,
        IrAction::ScrollTo { .. } => {
            // Native scrolling not yet implemented
            false
        }
        IrAction::Log { expr } => {
            let value = evaluate_expr(expr, state);
            println!("[log] {:?}", value);
            false
        }
    }
}

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
            } else {
                RenderValue::Str(format!(
                    "{}{}",
                    render_value_to_string(left),
                    render_value_to_string(right)
                ))
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
        IrBinOp::Eq => RenderValue::Bool(left_num == right_num),
        IrBinOp::Neq => RenderValue::Bool(left_num != right_num),
        IrBinOp::Gt => RenderValue::Bool(left_num.unwrap_or(0.0) > right_num.unwrap_or(0.0)),
        IrBinOp::Lt => RenderValue::Bool(left_num.unwrap_or(0.0) < right_num.unwrap_or(0.0)),
        IrBinOp::Gte => RenderValue::Bool(left_num.unwrap_or(0.0) >= right_num.unwrap_or(0.0)),
        IrBinOp::Lte => RenderValue::Bool(left_num.unwrap_or(0.0) <= right_num.unwrap_or(0.0)),
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
            if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{}", n) }
        }
        RenderValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        RenderValue::Color(c) => format!("#{:06x}", c),
        _ => String::new(),
    }
}
