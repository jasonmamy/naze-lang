//! Native build: produces standalone executable with embedded app data.

use std::path::Path;
use std::process::Command;

use crate::build;
use crate::diagnostic::Format;
use crate::manifest::Manifest;

/// Build a standalone native binary.
pub fn run(manifest: &Manifest, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = Path::new(&manifest.build.output);
    let app_name = &manifest.app.name;

    // Step 1: Build app_data.bin using existing web build
    build::run(manifest, format)?;

    let app_data_path = output_dir.join("app_data.bin");
    if !app_data_path.exists() {
        return Err("app_data.bin not found after build".into());
    }

    // Step 2: Create temporary build directory
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("naze")
        .join("native-build");
    std::fs::create_dir_all(&cache_dir)?;

    let build_dir = cache_dir.join(app_name);
    std::fs::create_dir_all(&build_dir)?;
    std::fs::create_dir_all(build_dir.join("src"))?;

    // Step 3: Copy app_data.bin to build dir
    std::fs::copy(&app_data_path, build_dir.join("app_data.bin"))?;

    // Step 4: Copy font
    let font_bytes = include_bytes!("../fonts/DejaVuSans.ttf");
    std::fs::write(build_dir.join("DejaVuSans.ttf"), font_bytes)?;

    // Step 5: Get paths to naze-ir and naze-layout for Cargo.toml
    // Look for workspace relative to the nazec executable, not current dir
    let workspace_root = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|dir| {
            // Walk up from the executable to find the workspace
            dir.ancestors()
                .find(|p| {
                    p.join("Cargo.toml").exists()
                        && p.join("crates/naze-ir").exists()
                        && std::fs::read_to_string(p.join("Cargo.toml"))
                            .map(|s| s.contains("[workspace]"))
                            .unwrap_or(false)
                })
                .map(|p| p.to_path_buf())
        });

    // Step 6: Write Cargo.toml
    let cargo_toml = generate_cargo_toml(app_name, workspace_root.as_deref());
    std::fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;

    // Step 7: Write main.rs
    std::fs::write(build_dir.join("src").join("main.rs"), MAIN_RS_TEMPLATE)?;

    // Step 8: Write renderer.rs
    std::fs::write(
        build_dir.join("src").join("renderer.rs"),
        RENDERER_RS_TEMPLATE,
    )?;

    // Step 9: Build with cargo
    if format == Format::Text {
        eprintln!("compiling native binary...");
    }

    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&build_dir)
        .status()?;

    if !status.success() {
        return Err("cargo build failed".into());
    }

    // Step 10: Copy binary to output directory
    let binary_name = if cfg!(windows) {
        format!("{}.exe", app_name)
    } else {
        app_name.clone()
    };

    let built_binary = build_dir.join("target").join("release").join(&binary_name);

    let output_binary = output_dir.join(&binary_name);
    std::fs::copy(&built_binary, &output_binary)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&output_binary)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&output_binary, perms)?;
    }

    if format == Format::Text {
        eprintln!("  created: {}", output_binary.display());
    }

    Ok(())
}

fn generate_cargo_toml(app_name: &str, workspace_root: Option<&Path>) -> String {
    let deps = if let Some(root) = workspace_root {
        // Use local workspace paths during development
        let ir_path = root.join("crates").join("naze-ir");
        let layout_path = root.join("crates").join("naze-layout");
        format!(
            r#"naze-ir = {{ path = "{}" }}
naze-layout = {{ path = "{}" }}"#,
            ir_path.display(),
            layout_path.display()
        )
    } else {
        // Fall back to crates.io versions (for end users)
        r#"naze-ir = "0.1"
naze-layout = "0.1""#
            .to_string()
    };

    format!(
        r#"[package]
name = "{app_name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{app_name}"
path = "src/main.rs"

[dependencies]
{deps}
winit = "0.30"
softbuffer = "0.4"
tiny-skia = "0.11"
fontdue = "0.9"

[profile.release]
opt-level = "z"
lto = true
strip = true
"#
    )
}

const MAIN_RS_TEMPLATE: &str = r##"mod renderer;

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use naze_ir::{IrAction, IrBinOp, IrExpression, IrPipelineStage, RenderNode, RenderTree, RenderValue, TextPart};
use naze_layout::{LayoutTree, PositionedNode};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{CursorIcon, Window, WindowId};

// Embed app data and font at compile time
static APP_DATA: &[u8] = include_bytes!("../app_data.bin");
static FONT_DATA: &[u8] = include_bytes!("../DejaVuSans.ttf");

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

        let resolved = resolve_tree(&self.render_tree, &self.state_store);
        let layout = naze_layout::compute_layout(&resolved, w as f32, h as f32);
        self.layout = Some(layout.clone());

        let mut pixmap = match tiny_skia::Pixmap::new(w, h) {
            Some(p) => p,
            None => return,
        };
        pixmap.fill(tiny_skia::Color::WHITE);
        renderer::draw_tree(&mut pixmap, &layout, &self.font, None);

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
    let render_tree = naze_ir::deserialize(APP_DATA).expect("invalid app data");
    let font = fontdue::Font::from_bytes(FONT_DATA as &[u8], fontdue::FontSettings::default())
        .expect("invalid font");

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
        computed: tree.computed.clone(),
        storage: tree.storage.clone(),
        timers: tree.timers.clone(),
        params: tree.params.clone(),
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
        IrAction::ScrollTo { .. } => false,
        IrAction::Log { expr } => {
            let value = evaluate_expr(expr, state);
            println!("[log] {:?}", value);
            false
        }
        // Trigger, Copy, Send, JsCall not supported in native build
        _ => false,
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
    }
}

fn eval_pipeline(source: RenderValue, stages: &[IrPipelineStage], state: &HashMap<String, RenderValue>) -> RenderValue {
    let mut current = source;
    for stage in stages {
        current = eval_pipeline_stage(current, stage, state);
    }
    current
}

fn eval_pipeline_stage(input: RenderValue, stage: &IrPipelineStage, state: &HashMap<String, RenderValue>) -> RenderValue {
    let items = match &input {
        RenderValue::List(items) => items.clone(),
        _ => return input,
    };
    match stage.function {
        0 => { // filter
            let arg = match &stage.argument { Some(a) => a, None => return RenderValue::List(items) };
            RenderValue::List(items.into_iter().filter(|item| {
                let mut s = state.clone();
                s.insert("__it".to_string(), item.clone());
                if let RenderValue::Object(entries) = item { for (k, v) in entries { s.insert(k.clone(), v.clone()); } }
                matches!(evaluate_expr(arg, &s), RenderValue::Bool(true))
            }).collect())
        }
        1 => { // map
            let arg = match &stage.argument { Some(a) => a, None => return RenderValue::List(items) };
            RenderValue::List(items.into_iter().map(|item| {
                let mut s = state.clone();
                s.insert("__it".to_string(), item.clone());
                if let RenderValue::Object(entries) = &item { for (k, v) in entries { s.insert(k.clone(), v.clone()); } }
                evaluate_expr(arg, &s)
            }).collect())
        }
        2 => { // sort-by
            let arg = match &stage.argument { Some(a) => a, None => return RenderValue::List(items) };
            let mut sorted = items;
            sorted.sort_by(|a, b| {
                let mut sa = state.clone(); sa.insert("__it".to_string(), a.clone());
                if let RenderValue::Object(e) = a { for (k, v) in e { sa.insert(k.clone(), v.clone()); } }
                let mut sb = state.clone(); sb.insert("__it".to_string(), b.clone());
                if let RenderValue::Object(e) = b { for (k, v) in e { sb.insert(k.clone(), v.clone()); } }
                let ak = evaluate_expr(arg, &sa); let bk = evaluate_expr(arg, &sb);
                match (&ak, &bk) {
                    (RenderValue::Num(an, _), RenderValue::Num(bn, _)) => an.partial_cmp(bn).unwrap_or(std::cmp::Ordering::Equal),
                    (RenderValue::Str(a), RenderValue::Str(b)) => a.cmp(b),
                    _ => std::cmp::Ordering::Equal,
                }
            });
            RenderValue::List(sorted)
        }
        3 => { // take
            let n = match &stage.argument { Some(a) => match evaluate_expr(a, state) { RenderValue::Num(n, _) => n as usize, _ => items.len() }, None => items.len() };
            RenderValue::List(items.into_iter().take(n).collect())
        }
        4 => { // sum
            let total: f64 = items.iter().filter_map(|i| if let RenderValue::Num(n, _) = i { Some(n) } else { None }).sum();
            RenderValue::Num(total, None)
        }
        5 => RenderValue::Num(items.len() as f64, None), // count
        6 => { // reduce
            let acc_expr = match &stage.argument { Some(a) => a, None => return RenderValue::List(items) };
            let initial = match &stage.argument2 { Some(init) => evaluate_expr(init, state), None => RenderValue::Num(0.0, None) };
            let mut acc = initial;
            for item in &items {
                let mut s = state.clone(); s.insert("__it".to_string(), item.clone());
                if let RenderValue::Object(entries) = item { for (k, v) in entries { s.insert(k.clone(), v.clone()); } }
                s.insert("acc".to_string(), acc.clone());
                acc = evaluate_expr(acc_expr, &s);
            }
            acc
        }
        7 => { // group-by
            let arg = match &stage.argument { Some(a) => a, None => return RenderValue::List(items) };
            let mut groups: Vec<(String, Vec<RenderValue>)> = Vec::new();
            for item in items {
                let mut s = state.clone(); s.insert("__it".to_string(), item.clone());
                if let RenderValue::Object(entries) = &item { for (k, v) in entries { s.insert(k.clone(), v.clone()); } }
                let key = render_value_to_string(&evaluate_expr(arg, &s));
                if let Some(group) = groups.iter_mut().find(|(k, _)| k == &key) { group.1.push(item); }
                else { groups.push((key, vec![item])); }
            }
            RenderValue::Object(groups.into_iter().map(|(k, v)| (k, RenderValue::List(v))).collect())
        }
        8 => { // flatten
            let mut flattened = Vec::new();
            for item in items { match item { RenderValue::List(inner) => flattened.extend(inner), other => flattened.push(other) } }
            RenderValue::List(flattened)
        }
        9 => { // distinct
            let mut seen = Vec::new();
            let mut result = Vec::new();
            for item in items {
                let key = match &stage.argument {
                    Some(arg) => {
                        let mut s = state.clone(); s.insert("__it".to_string(), item.clone());
                        if let RenderValue::Object(entries) = &item { for (k, v) in entries { s.insert(k.clone(), v.clone()); } }
                        render_value_to_string(&evaluate_expr(arg, &s))
                    }
                    None => render_value_to_string(&item),
                };
                if !seen.contains(&key) { seen.push(key); result.push(item); }
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
"##;

const RENDERER_RS_TEMPLATE: &str = r##"use std::collections::HashMap;

use naze_ir::RenderValue;
use naze_layout::{LayoutTree, PositionedNode};
use tiny_skia::{Paint, PathBuilder, Pixmap, Rect, Shader, Transform};

const DEFAULT_TEXT_SIZE: f64 = 16.0;
const DEFAULT_HEADING_SIZE: f64 = 24.0;

fn get_color_u32(props: &HashMap<String, RenderValue>, key: &str, default: u32) -> u32 {
    match props.get(key) {
        Some(RenderValue::Color(c)) => *c,
        _ => default,
    }
}

fn get_color_u32_opt(props: &HashMap<String, RenderValue>, key: &str) -> Option<u32> {
    match props.get(key) {
        Some(RenderValue::Color(c)) => Some(*c),
        _ => None,
    }
}

fn get_num_prop(props: &HashMap<String, RenderValue>, key: &str, default: f64) -> f64 {
    match props.get(key) {
        Some(RenderValue::Num(n, _)) => *n,
        _ => default,
    }
}

fn get_text_content(props: &HashMap<String, RenderValue>) -> String {
    match props.get("__text") {
        Some(RenderValue::Str(s)) => s.clone(),
        Some(RenderValue::InterpolatedStr(parts)) => {
            use naze_ir::TextPart;
            let mut result = String::new();
            for part in parts {
                match part {
                    TextPart::Literal(s) => result.push_str(s),
                    TextPart::StateRef(name) => {
                        result.push('{');
                        result.push_str(name);
                        result.push('}');
                    }
                }
            }
            result
        }
        _ => String::new(),
    }
}

fn get_font_size(props: &HashMap<String, RenderValue>, is_heading: bool) -> f64 {
    match props.get("font-size") {
        Some(RenderValue::Num(n, _)) => *n,
        _ => {
            if is_heading {
                DEFAULT_HEADING_SIZE
            } else {
                DEFAULT_TEXT_SIZE
            }
        }
    }
}

fn make_paint(color: u32) -> Paint<'static> {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    Paint {
        shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(r, g, b, 255)),
        anti_alias: true,
        ..Paint::default()
    }
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: u32, radius: f32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let paint = make_paint(color);

    if radius > 0.0 {
        if let Some(path) = rounded_rect_path(x, y, w, h, radius) {
            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    } else if let Some(rect) = Rect::from_xywh(x, y, w, h) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.0).min(h / 2.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish()
}

fn stroke_rounded_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, r: f32, color: u32, stroke_width: f32) {
    if let Some(path) = rounded_rect_path(x, y, w, h, r) {
        let paint = make_paint(color);
        let stroke = tiny_skia::Stroke {
            width: stroke_width,
            ..tiny_skia::Stroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn draw_checkbox(pixmap: &mut Pixmap, x: f32, y: f32, checked: bool, label: &str, font: &fontdue::Font) {
    let box_size = 20.0_f32;
    fill_rect(pixmap, x, y, box_size, box_size, 0xffffff, 3.0);
    stroke_rounded_rect(pixmap, x, y, box_size, box_size, 3.0, 0x9ca3af, 2.0);

    if checked {
        let mut pb = PathBuilder::new();
        pb.move_to(x + 4.0, y + 10.0);
        pb.line_to(x + 8.0, y + 15.0);
        pb.line_to(x + 16.0, y + 5.0);
        if let Some(path) = pb.finish() {
            let paint = make_paint(0x2563eb);
            let stroke = tiny_skia::Stroke {
                width: 2.5,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..tiny_skia::Stroke::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    if !label.is_empty() {
        draw_text(pixmap, label, x + 28.0, y + 2.0, 16.0, font, 0x374151);
    }
}

fn draw_radio(pixmap: &mut Pixmap, x: f32, y: f32, selected: bool, label: &str, font: &fontdue::Font) {
    let radius = 10.0_f32;
    let center_x = x + radius;
    let center_y = y + radius;

    let mut pb = PathBuilder::new();
    pb.push_circle(center_x, center_y, radius);
    if let Some(path) = pb.finish() {
        let paint = make_paint(0xffffff);
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    let mut pb = PathBuilder::new();
    pb.push_circle(center_x, center_y, radius - 1.0);
    if let Some(path) = pb.finish() {
        let paint = make_paint(0x9ca3af);
        let stroke = tiny_skia::Stroke {
            width: 2.0,
            ..tiny_skia::Stroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    if selected {
        let mut pb = PathBuilder::new();
        pb.push_circle(center_x, center_y, 5.0);
        if let Some(path) = pb.finish() {
            let paint = make_paint(0x2563eb);
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    if !label.is_empty() {
        draw_text(pixmap, label, x + 28.0, y + 2.0, 16.0, font, 0x374151);
    }
}

fn draw_input(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, value: &str, placeholder: &str, focused: bool, input_type: &str, show_caret: bool, font: &fontdue::Font) {
    fill_rect(pixmap, x, y, w, h, 0xffffff, 4.0);

    let border_color = if focused { 0x2563eb } else { 0xd1d5db };
    let border_width = if focused { 2.0 } else { 1.0 };
    stroke_rounded_rect(pixmap, x, y, w, h, 4.0, border_color, border_width);

    let display_value = if input_type == "password" {
        "•".repeat(value.chars().count())
    } else {
        value.to_string()
    };

    let text_x = x + 8.0;
    let text_y = y + 4.0;
    if !display_value.is_empty() {
        draw_text(pixmap, &display_value, text_x, text_y, 16.0, font, 0x111827);
    } else if !placeholder.is_empty() {
        draw_text(pixmap, placeholder, text_x, text_y, 16.0, font, 0x9ca3af);
    }

    if show_caret {
        let text_width: f32 = display_value.chars()
            .map(|ch| font.metrics(ch, 16.0).advance_width)
            .sum();
        let cursor_x = text_x + text_width;

        let mut pb = PathBuilder::new();
        pb.move_to(cursor_x, y + 6.0);
        pb.line_to(cursor_x, y + h - 6.0);
        if let Some(path) = pb.finish() {
            let paint = make_paint(0x111827);
            let stroke = tiny_skia::Stroke {
                width: 1.0,
                ..tiny_skia::Stroke::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

fn draw_select(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    display_text: &str,
    is_open: bool,
    font: &fontdue::Font,
) {
    fill_rect(pixmap, x, y, w, h, 0xffffff, 4.0);

    let border_color = if is_open { 0x2563eb } else { 0xd1d5db };
    let border_width = if is_open { 2.0 } else { 1.0 };
    stroke_rounded_rect(pixmap, x, y, w, h, 4.0, border_color, border_width);

    if !display_text.is_empty() {
        draw_text(pixmap, display_text, x + 12.0, y + 8.0, 16.0, font, 0x111827);
    }

    let arrow_x = x + w - 24.0;
    let arrow_y = y + h / 2.0;
    let mut pb = PathBuilder::new();
    if is_open {
        pb.move_to(arrow_x, arrow_y + 2.0);
        pb.line_to(arrow_x + 6.0, arrow_y - 4.0);
        pb.line_to(arrow_x + 12.0, arrow_y + 2.0);
    } else {
        pb.move_to(arrow_x, arrow_y - 2.0);
        pb.line_to(arrow_x + 6.0, arrow_y + 4.0);
        pb.line_to(arrow_x + 12.0, arrow_y - 2.0);
    }
    if let Some(path) = pb.finish() {
        let paint = make_paint(0x6b7280);
        let stroke = tiny_skia::Stroke {
            width: 2.0,
            line_cap: tiny_skia::LineCap::Round,
            line_join: tiny_skia::LineJoin::Round,
            ..tiny_skia::Stroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn draw_text(
    pixmap: &mut Pixmap,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    font: &fontdue::Font,
    color: u32,
) {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;

    let mut cursor_x = x;
    let baseline_y = y + font_size * 0.8;

    let pw = pixmap.width() as i32;
    let ph = pixmap.height() as i32;

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        let gx = cursor_x as i32 + metrics.xmin;
        let gy = baseline_y as i32 - metrics.height as i32 - metrics.ymin;

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha == 0 {
                    continue;
                }
                let px = gx + col as i32;
                let py = gy + row as i32;
                if px < 0 || py < 0 || px >= pw || py >= ph {
                    continue;
                }
                let di = ((py as u32 * pixmap.width() + px as u32) * 4) as usize;
                let data = pixmap.data_mut();
                let a = alpha as f32 / 255.0;
                let inv_a = 1.0 - a;
                data[di] = (r as f32 * a + data[di] as f32 * inv_a) as u8;
                data[di + 1] = (g as f32 * a + data[di + 1] as f32 * inv_a) as u8;
                data[di + 2] = (b as f32 * a + data[di + 2] as f32 * inv_a) as u8;
                data[di + 3] = 255;
            }
        }
        cursor_x += metrics.advance_width;
    }
}

pub fn draw_tree(pixmap: &mut Pixmap, layout: &LayoutTree, font: &fontdue::Font, focused_input_id: Option<&str>) {
    for node in &layout.root {
        draw_node(pixmap, node, font, focused_input_id);
    }
}

fn draw_node(pixmap: &mut Pixmap, node: &PositionedNode, font: &fontdue::Font, focused_input_id: Option<&str>) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    match node.kind.as_str() {
        "rect" => {
            let color = get_color_u32(&node.props, "color", 0x000000);
            let radius = get_num_prop(&node.props, "radius", 0.0) as f32;
            fill_rect(pixmap, x, y, w, h, color, radius);
        }
        "container" => {
            let color = get_color_u32_opt(&node.props, "color");
            let radius = get_num_prop(&node.props, "radius", 0.0) as f32;
            if let Some(c) = color {
                fill_rect(pixmap, x, y, w, h, c, radius);
            }
            for child in &node.children {
                draw_node(pixmap, child, font, focused_input_id);
            }
        }
        "text" => {
            let text = get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = get_font_size(&node.props, false) as f32;
                let color = get_color_u32(&node.props, "color", 0x000000);
                draw_text(pixmap, &text, x, y, font_size, font, color);
            }
        }
        "heading" => {
            let text = get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = get_font_size(&node.props, true) as f32;
                let color = get_color_u32(&node.props, "color", 0x000000);
                draw_text(pixmap, &text, x, y, font_size, font, color);
            }
        }
        "row" | "column" | "stack" | "grid" => {
            let color = get_color_u32_opt(&node.props, "color");
            if let Some(c) = color {
                fill_rect(pixmap, x, y, w, h, c, 0.0);
            }
            for child in &node.children {
                draw_node(pixmap, child, font, focused_input_id);
            }
        }
        "spacer" => {}
        "checkbox" => {
            let label = get_text_content(&node.props);
            let checked = match node.props.get("checked") {
                Some(RenderValue::Bool(b)) => *b,
                _ => false,
            };
            draw_checkbox(pixmap, x, y, checked, &label, font);
        }
        "radio" => {
            let label = get_text_content(&node.props);
            let selected = match node.props.get("selected") {
                Some(RenderValue::Bool(b)) => *b,
                _ => false,
            };
            draw_radio(pixmap, x, y, selected, &label, font);
        }
        "input" => {
            let placeholder = match node.props.get("placeholder") {
                Some(RenderValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            let value = match node.props.get("value") {
                Some(RenderValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            let node_id = format!("input_{}_{}", x as i32, y as i32);
            let focused = focused_input_id == Some(node_id.as_str());
            let input_type = match node.props.get("type") {
                Some(RenderValue::Str(s)) => s.as_str(),
                _ => "text",
            };
            let show_caret = focused;
            draw_input(pixmap, x, y, w, h, &value, &placeholder, focused, input_type, show_caret, font);
        }
        "textarea" => {
            let placeholder = match node.props.get("placeholder") {
                Some(RenderValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            let value = match node.props.get("value") {
                Some(RenderValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            let node_id = format!("textarea_{}_{}", x as i32, y as i32);
            let focused = focused_input_id == Some(node_id.as_str());
            draw_input(pixmap, x, y, w, h, &value, &placeholder, focused, "text", focused, font);
        }
        "select" => {
            let selected_value = match node.props.get("selected") {
                Some(RenderValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            let mut display_text = String::new();
            for child in &node.children {
                if child.kind == "option" {
                    let value = match child.props.get("value") {
                        Some(RenderValue::Str(s)) => s.clone(),
                        _ => get_text_content(&child.props),
                    };
                    if value == selected_value {
                        display_text = get_text_content(&child.props);
                        break;
                    }
                }
            }
            draw_select(pixmap, x, y, w, h, &display_text, false, font);
        }
        "option" => {}
        _ => {
            for child in &node.children {
                draw_node(pixmap, child, font, focused_input_id);
            }
        }
    }
}
"##;
