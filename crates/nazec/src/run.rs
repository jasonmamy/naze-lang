use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

use naze_ir::{IrAction, IrBinOp, IrExpression, RenderNode, RenderTree, RenderValue, TextPart};
use naze_layout::{LayoutTree, PositionedNode};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{CursorIcon, Window, WindowId};

use crate::build;
use crate::diagnostic::Format;
use crate::manifest::Manifest;
use crate::native_renderer;

#[derive(Debug)]
enum AppEvent {
    SourceChanged,
}

struct App {
    manifest: Manifest,
    render_tree: RenderTree,
    state_store: HashMap<String, RenderValue>,
    font: fontdue::Font,
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    layout: Option<LayoutTree>,
    cursor_pos: Option<(f32, f32)>,
}

impl ApplicationHandler<AppEvent> for App {
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

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::SourceChanged => {
                self.rebuild_and_reload();
            }
        }
    }
}

impl App {
    fn rebuild_and_reload(&mut self) {
        eprintln!("change detected, rebuilding...");
        match build::run(&self.manifest, Format::Text) {
            Ok(()) => {
                let bin_path =
                    Path::new(&self.manifest.build.output).join("app_data.bin");
                match std::fs::read(&bin_path).and_then(|bytes| {
                    naze_ir::deserialize(&bytes)
                        .map_err(std::io::Error::other)
                }) {
                    Ok(tree) => {
                        let mut state_store = HashMap::new();
                        for decl in &tree.state {
                            state_store
                                .insert(decl.name.clone(), decl.initial.clone());
                        }
                        self.render_tree = tree;
                        self.state_store = state_store;
                        self.render();
                        eprintln!("reloaded");
                    }
                    Err(e) => eprintln!("reload error: {e}"),
                }
            }
            Err(e) => eprintln!("build error: {e}"),
        }
    }

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

        let resolved = resolve_tree(&self.render_tree, &self.state_store);
        let layout = naze_layout::compute_layout(&resolved, w as f32, h as f32);
        self.layout = Some(layout.clone());

        let mut pixmap = match tiny_skia::Pixmap::new(w, h) {
            Some(p) => p,
            None => return,
        };
        pixmap.fill(tiny_skia::Color::WHITE);
        native_renderer::draw_tree(&mut pixmap, &layout, &self.font);

        let surface = match &mut self.surface {
            Some(s) => s,
            None => return,
        };
        surface
            .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
            .unwrap();
        let mut buffer = surface.buffer_mut().unwrap();

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

/// Run the native desktop preview with live reload.
pub fn run(manifest: &Manifest) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new(&manifest.build.output);
    let bin_path = output_dir.join("app_data.bin");

    if !bin_path.exists() {
        return Err(format!(
            "no build output found at {}. Run `nazec build` first.",
            bin_path.display()
        )
        .into());
    }

    let bytes = std::fs::read(&bin_path)?;
    let render_tree = naze_ir::deserialize(&bytes)
        .map_err(|e| format!("failed to deserialize {}: {}", bin_path.display(), e))?;

    let font_bytes = include_bytes!("../fonts/DejaVuSans.ttf");
    let font = fontdue::Font::from_bytes(font_bytes as &[u8], fontdue::FontSettings::default())
        .map_err(|e| format!("failed to load font: {}", e))?;

    eprintln!("running {} (native preview)", render_tree.title);
    eprintln!("watching for changes... (press Ctrl+C to stop)");

    let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    // Spawn file watcher thread with debouncing.
    // Waits for 300ms of quiet after the last .naze file change before
    // sending a single SourceChanged event, coalescing multiple editor
    // events (data write + metadata update) into one rebuild.
    let project_dir = std::env::current_dir()?;
    std::thread::spawn(move || {
        use notify::{RecursiveMode, Watcher};
        use std::time::{Duration, Instant};

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();
        watcher
            .watch(&project_dir, RecursiveMode::Recursive)
            .unwrap();

        let debounce = Duration::from_millis(300);
        let mut last_event: Option<Instant> = None;

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(event)) => {
                    let dominated = event.paths.iter().any(|p| {
                        let is_naze = p.extension().is_some_and(|e| e == "naze");
                        let in_dist = p.components().any(|c| c.as_os_str() == "dist");
                        is_naze && !in_dist
                    });
                    if dominated && event.kind.is_modify() {
                        last_event = Some(Instant::now());
                    }
                }
                Ok(Err(_)) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if let Some(t) = last_event {
                if t.elapsed() >= debounce {
                    let _ = proxy.send_event(AppEvent::SourceChanged);
                    last_event = None;
                }
            }
        }
    });

    let mut state_store = HashMap::new();
    for decl in &render_tree.state {
        state_store.insert(decl.name.clone(), decl.initial.clone());
    }

    let mut app = App {
        manifest: manifest.clone(),
        render_tree,
        state_store,
        font,
        window: None,
        surface: None,
        layout: None,
        cursor_pos: None,
    };
    event_loop.run_app(&mut app)?;

    Ok(())
}

// ─── State resolution ───────────────────────────────────────────────────────

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
