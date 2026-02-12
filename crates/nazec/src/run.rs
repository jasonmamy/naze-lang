use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

use naze_ir::{
    IrAction, IrBinOp, IrExpression, IrPipelineStage, RenderNode, RenderTree, RenderValue, TextPart,
};
use naze_layout::{LayoutTree, PositionedNode};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{CursorIcon, Window, WindowId};

use naze_compiler::resolve::BuildCache;

use crate::build;
use crate::diagnostic::Format;
use crate::manifest::Manifest;
use crate::native_renderer;

#[derive(Debug)]
enum AppEvent {
    SourceChanged,
}

/// Tracks which text input is currently focused in native mode.
#[derive(Clone)]
struct FocusedInput {
    bind_var: String,
    node_id: String,
    #[allow(dead_code)]
    input_type: String,
    change_handlers: Vec<naze_ir::IrEventHandler>,
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
    focused_input: Option<FocusedInput>,
    build_cache: BuildCache,
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        text,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if self.focused_input.is_some() {
                    let mut changed = false;

                    match logical_key {
                        Key::Named(NamedKey::Backspace) => {
                            // Remove last character
                            if let Some(ref focus) = self.focused_input {
                                if let Some(RenderValue::Str(current)) =
                                    self.state_store.get(&focus.bind_var)
                                {
                                    let mut chars: Vec<char> = current.chars().collect();
                                    if !chars.is_empty() {
                                        chars.pop();
                                        let new_value: String = chars.into_iter().collect();
                                        self.state_store.insert(
                                            focus.bind_var.clone(),
                                            RenderValue::Str(new_value),
                                        );
                                        changed = true;
                                    }
                                }
                            }
                        }
                        Key::Named(NamedKey::Enter) => {
                            // For textarea, Enter inserts a newline
                            if let Some(ref focus) = self.focused_input {
                                if focus.input_type == "textarea" {
                                    if let Some(RenderValue::Str(current)) =
                                        self.state_store.get(&focus.bind_var)
                                    {
                                        let new_value = format!("{}\n", current);
                                        self.state_store.insert(
                                            focus.bind_var.clone(),
                                            RenderValue::Str(new_value),
                                        );
                                        changed = true;
                                    }
                                } else {
                                    // For regular inputs, unfocus
                                    for handler in &focus.change_handlers {
                                        execute_action(&handler.action, &mut self.state_store);
                                    }
                                    self.focused_input = None;
                                    changed = true;
                                }
                            }
                        }
                        Key::Named(NamedKey::Escape) => {
                            // Execute change handlers before unfocusing
                            if let Some(ref focus) = self.focused_input {
                                for handler in &focus.change_handlers {
                                    execute_action(&handler.action, &mut self.state_store);
                                }
                            }
                            self.focused_input = None;
                            changed = true;
                        }
                        Key::Named(NamedKey::Tab) => {
                            // TODO: Move to next/prev input
                            self.focused_input = None;
                            changed = true;
                        }
                        _ => {
                            // Handle text input
                            if let Some(ref text) = text {
                                if let Some(ref focus) = self.focused_input {
                                    if let Some(RenderValue::Str(current)) =
                                        self.state_store.get(&focus.bind_var)
                                    {
                                        let new_value = format!("{}{}", current, text.as_str());
                                        self.state_store.insert(
                                            focus.bind_var.clone(),
                                            RenderValue::Str(new_value),
                                        );
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }

                    if changed {
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
        let start = std::time::Instant::now();
        match build::run_incremental(
            &self.manifest,
            Format::Text,
            &mut self.build_cache,
            &[],
            false,
        ) {
            Ok(()) => {
                let bin_path = Path::new(&self.manifest.build.output).join("app_data.bin");
                match std::fs::read(&bin_path)
                    .and_then(|bytes| naze_ir::deserialize(&bytes).map_err(std::io::Error::other))
                {
                    Ok(tree) => {
                        let mut state_store = HashMap::new();
                        for decl in &tree.state {
                            state_store.insert(decl.name.clone(), decl.initial.clone());
                        }
                        self.render_tree = tree;
                        self.state_store = state_store;
                        self.render();
                        eprintln!("  rebuilt in {}ms", start.elapsed().as_millis());
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

        let focused_input_id = self.focused_input.as_ref().map(|f| f.node_id.as_str());
        let resolved = resolve_tree(&self.render_tree, &self.state_store);
        let layout = naze_layout::compute_layout(&resolved, w as f32, h as f32);
        self.layout = Some(layout.clone());

        let mut pixmap = match tiny_skia::Pixmap::new(w, h) {
            Some(p) => p,
            None => return,
        };
        pixmap.fill(tiny_skia::Color::WHITE);
        native_renderer::draw_tree(&mut pixmap, &layout, &self.font, focused_input_id);

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

        let mut changed = false;

        // Check if clicking on an input element
        if let Some((bind_var, node_id, input_type, change_handlers)) =
            find_input_at_point(&layout.root, x, y)
        {
            // Execute change handlers for previously focused input before switching
            if let Some(ref old_focus) = self.focused_input {
                if old_focus.node_id != node_id {
                    for handler in &old_focus.change_handlers {
                        execute_action(&handler.action, &mut self.state_store);
                    }
                }
            }

            self.focused_input = Some(FocusedInput {
                bind_var,
                node_id,
                input_type,
                change_handlers,
            });
            return true; // Input focus counts as a change, needs re-render
        }

        // Clicked outside any input - unfocus if something was focused
        if self.focused_input.is_some() {
            if let Some(ref focus) = self.focused_input {
                for handler in &focus.change_handlers {
                    execute_action(&handler.action, &mut self.state_store);
                }
            }
            self.focused_input = None;
            changed = true;
        }

        // Handle click handlers (buttons, checkbox, radio, etc.)
        let handlers = find_click_handlers(&layout.root, x, y, &self.state_store);
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
        focused_input: None,
        build_cache: BuildCache::new(),
    };
    event_loop.run_app(&mut app)?;

    Ok(())
}

// ─── State resolution ───────────────────────────────────────────────────────

fn resolve_tree(tree: &RenderTree, state: &HashMap<String, RenderValue>) -> RenderTree {
    RenderTree {
        title: tree.title.clone(),
        state: tree.state.clone(),
        data: tree.data.clone(),
        computed: tree.computed.clone(),
        storage: tree.storage.clone(),
        timers: tree.timers.clone(),
        params: tree.params.clone(),
        root: resolve_nodes(&tree.root, state),
        pages: tree.pages.clone(),
        themes: tree.themes.clone(),
        imports: tree.imports.clone(),
        server_functions: tree.server_functions.clone(),
        server_calls: tree.server_calls.clone(),
        prompts: tree.prompts.clone(),
        guards: tree.guards.clone(),
    }
}

fn resolve_nodes(nodes: &[RenderNode], state: &HashMap<String, RenderValue>) -> Vec<RenderNode> {
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
                // Add change handlers
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
                // Add change handlers
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

/// Find an input element at the given point. Returns (bind_var, node_id, input_type, change_handlers).
fn find_input_at_point(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
) -> Option<(String, String, String, Vec<naze_ir::IrEventHandler>)> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first
        if let Some(result) = find_input_at_point(&node.children, x, y) {
            return Some(result);
        }
        // Check if this is an input or textarea
        if node.kind == "input" || node.kind == "textarea" {
            if let Some(RenderValue::Bind(bind_var)) = node.props.get("bind") {
                let prefix = if node.kind == "textarea" {
                    "textarea"
                } else {
                    "input"
                };
                let node_id = format!("{}_{}_{}", prefix, node.x as i32, node.y as i32);
                let input_type = if node.kind == "textarea" {
                    "textarea".to_string()
                } else {
                    match node.props.get("type") {
                        Some(RenderValue::Str(s)) => s.clone(),
                        _ => "text".to_string(),
                    }
                };
                let change_handlers: Vec<_> = node
                    .handlers
                    .iter()
                    .filter(|h| h.event == "change")
                    .cloned()
                    .collect();
                return Some((bind_var.clone(), node_id, input_type, change_handlers));
            }
        }
    }
    None
}

fn point_in_node(node: &PositionedNode, x: f32, y: f32) -> bool {
    x >= node.x && x <= node.x + node.width && y >= node.y && y <= node.y + node.height
}

// ─── Action execution ────────────────────────────────────────────────────────

fn execute_action(action: &IrAction, state: &mut HashMap<String, RenderValue>) -> bool {
    match action {
        IrAction::Set { target, expr } => {
            let value = evaluate_expr(expr, state);
            state.insert(target.clone(), value);
            true
        }
        IrAction::Navigate { .. } => false,
        IrAction::ScrollTo { .. } => {
            // Scrolling not yet implemented in native runner
            false
        }
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
        // Trigger, Copy, Send not supported in native runner
        _ => false,
    }
}

fn evaluate_expr(expr: &IrExpression, state: &HashMap<String, RenderValue>) -> RenderValue {
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
            // WASM imports not supported in native runner
            RenderValue::Num(0.0, None)
        }
        IrExpression::EnvRef(_) => {
            // Env vars resolved at compile time; should not appear at runtime
            RenderValue::Str(String::new())
        }
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
