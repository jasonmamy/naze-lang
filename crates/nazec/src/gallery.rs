//! Build and serve an example gallery for interactive browsing.

use std::collections::HashMap;
use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use naze_compiler::codegen;
use naze_compiler::error::Severity;
use naze_compiler::resolve;
use naze_compiler::typecheck;
use naze_ir::{IrAction, IrBinOp, IrExpression, RenderNode, RenderTree, RenderValue, TextPart};
use naze_layout::{LayoutTree, PositionedNode};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{CursorIcon, Window, WindowId};

use crate::native_renderer;

// Embed the pre-built runtime files from wasm-pack output.
const RUNTIME_WASM: &[u8] = include_bytes!("../../naze-runtime/pkg/naze_runtime_bg.wasm");
const RUNTIME_JS: &str = include_str!("../../naze-runtime/pkg/naze_runtime.js");
const RUNTIME_BG_JS: &str = include_str!("../../naze-runtime/pkg/naze_runtime_bg.js");

const GALLERY_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Naze Examples</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html, body { width: 100%; height: 100%; overflow: hidden; font-family: system-ui, sans-serif; }
    body { display: flex; }
    #sidebar {
      width: 220px;
      background: #1a1a2e;
      color: #fff;
      padding: 16px;
      overflow-y: auto;
      flex-shrink: 0;
    }
    #sidebar h2 { margin: 0 0 16px; font-size: 18px; font-weight: 600; }
    #sidebar button {
      display: block;
      width: 100%;
      padding: 8px 12px;
      margin: 4px 0;
      background: transparent;
      border: 1px solid #333;
      color: #fff;
      cursor: pointer;
      text-align: left;
      border-radius: 4px;
      font-size: 14px;
    }
    #sidebar button:hover { background: #333; }
    #sidebar button.active { background: #3b82f6; border-color: #3b82f6; }
    #main { flex: 1; display: flex; flex-direction: column; min-width: 0; }
    #title {
      padding: 12px 16px;
      background: #f5f5f5;
      border-bottom: 1px solid #ddd;
      font-size: 14px;
      color: #666;
    }
    #canvas-container { flex: 1; position: relative; overflow: hidden; }
    canvas { position: absolute; top: 0; left: 0; }
  </style>
</head>
<body>
  <div id="sidebar">
    <h2>Naze Examples</h2>
{{EXAMPLE_BUTTONS}}
  </div>
  <div id="main">
    <div id="title">Select an example</div>
    <div id="canvas-container">
      <canvas id="naze-canvas"></canvas>
    </div>
  </div>
  <script type="module">
    import init, { start, reset_and_reload } from './naze_runtime.js';

    let initialized = false;

    async function loadExample(name) {
      const resp = await fetch('./' + name + '/app_data.bin');
      const data = new Uint8Array(await resp.arrayBuffer());

      if (!initialized) {
        await init();
        start(data, 'naze-canvas');
        initialized = true;
      } else {
        reset_and_reload(data);
      }

      document.getElementById('title').textContent = name;
      document.querySelectorAll('#sidebar button').forEach(b =>
        b.classList.toggle('active', b.dataset.name === name));
    }

    // Expose for button onclick
    window.loadExample = loadExample;

    // Load first example on start
    loadExample('{{FIRST_EXAMPLE}}');
  </script>
</body>
</html>
"#;

/// Find the examples directory relative to cargo manifest.
fn find_examples_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // When running from workspace root or nazec crate
    let candidates = [
        PathBuf::from("examples"),
        PathBuf::from("../../examples"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("examples"))
            .unwrap_or_default(),
    ];

    for path in &candidates {
        if path.is_dir() {
            return Ok(path.clone());
        }
    }

    Err("Could not find examples directory".into())
}

/// Find all example .naze files (excluding components/ subdirectory).
fn find_examples(dir: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut examples = Vec::new();

    // Files that are config files, not runnable examples
    const EXCLUDED: &[&str] = &["theme"];

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "naze" {
                    if let Some(stem) = path.file_stem() {
                        let name = stem.to_string_lossy().to_string();
                        if !EXCLUDED.contains(&name.as_str()) {
                            examples.push(name);
                        }
                    }
                }
            }
        }
    }

    examples.sort();
    Ok(examples)
}

/// Build a single example to output_dir/{name}/app_data.bin.
fn build_example(
    examples_dir: &Path,
    name: &str,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = format!("{}.naze", name);

    // Resolve and compile
    let project = resolve::resolve(examples_dir, &entry);

    // Check for errors
    let resolve_errors: Vec<_> = project
        .errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .collect();
    if !resolve_errors.is_empty() {
        return Err(format!("resolve errors in {}: {:?}", name, resolve_errors).into());
    }

    let tc_errors = typecheck::typecheck(&project);
    let tc_hard: Vec<_> = tc_errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .collect();
    if !tc_hard.is_empty() {
        return Err(format!("type errors in {}: {:?}", name, tc_hard).into());
    }

    // Lower and serialize
    let tree = codegen::lower(&project);
    let bytes = naze_ir::serialize(&tree);

    // Write to output_dir/{name}/app_data.bin
    let example_dir = output_dir.join(name);
    fs::create_dir_all(&example_dir)?;
    fs::write(example_dir.join("app_data.bin"), &bytes)?;

    Ok(())
}

/// Copy runtime files to output directory.
fn copy_runtime(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(output_dir.join("naze_runtime_bg.wasm"), RUNTIME_WASM)?;
    fs::write(output_dir.join("naze_runtime.js"), RUNTIME_JS)?;
    fs::write(output_dir.join("naze_runtime_bg.js"), RUNTIME_BG_JS)?;
    Ok(())
}

/// Generate the gallery index.html.
fn generate_gallery_html(
    examples: &[String],
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let buttons: String = examples
        .iter()
        .map(|name| {
            format!(
                "    <button data-name=\"{}\" onclick=\"loadExample('{}')\">{}</button>\n",
                name, name, name
            )
        })
        .collect();

    let first = examples.first().map(|s| s.as_str()).unwrap_or("hello");

    let html = GALLERY_HTML_TEMPLATE
        .replace("{{EXAMPLE_BUTTONS}}", &buttons)
        .replace("{{FIRST_EXAMPLE}}", first);

    fs::write(output_dir.join("index.html"), html)?;
    Ok(())
}

/// Start a simple HTTP server and open the browser.
fn serve_and_open(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("\nGallery built to: {}", output_dir.display());
    eprintln!("\nTo view the gallery, run:");
    eprintln!(
        "  cd {} && python3 -m http.server 8000",
        output_dir.display()
    );
    eprintln!("Then open: http://localhost:8000\n");

    // Try to start python server automatically
    let port = 8000;
    let server_result = std::process::Command::new("python3")
        .args(["-m", "http.server", &port.to_string()])
        .current_dir(output_dir)
        .spawn();

    match server_result {
        Ok(mut child) => {
            eprintln!("Server started on http://localhost:{}", port);

            // Try to open browser
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open")
                .arg(format!("http://localhost:{}", port))
                .spawn();

            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open")
                .arg(format!("http://localhost:{}", port))
                .spawn();

            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", &format!("http://localhost:{}", port)])
                .spawn();

            eprintln!("Press Ctrl+C to stop the server.");

            // Wait for server
            let _ = child.wait();
        }
        Err(_) => {
            eprintln!("Could not start python server automatically.");
            eprintln!("Please run the commands above manually.");
        }
    }

    Ok(())
}

// ─── Native Gallery Window ──────────────────────────────────────────────────

const SIDEBAR_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 32.0;
const BUTTON_PADDING: f32 = 4.0;

struct NativeGallery {
    examples: Vec<(String, RenderTree)>,
    selected: usize,
    state_store: HashMap<String, RenderValue>,
    font: fontdue::Font,
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    layout: Option<LayoutTree>,
    cursor_pos: Option<(f32, f32)>,
}

impl ApplicationHandler for NativeGallery {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let title = format!("Naze Gallery - {}", self.examples[self.selected].0);
        let attrs = Window::default_attributes()
            .with_title(&title)
            .with_inner_size(LogicalSize::new(1224.0f64, 768.0));
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
                self.update_cursor();
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if let Some((x, y)) = self.cursor_pos {
                    self.handle_click(x, y);
                }
            }
            _ => {}
        }
    }
}

impl NativeGallery {
    fn select_example(&mut self, idx: usize) {
        if idx < self.examples.len() && idx != self.selected {
            self.selected = idx;
            // Reset state for new example
            self.state_store.clear();
            for decl in &self.examples[self.selected].1.state {
                self.state_store
                    .insert(decl.name.clone(), decl.initial.clone());
            }
            self.layout = None;
            if let Some(window) = &self.window {
                window.set_title(&format!(
                    "Naze Gallery - {}",
                    self.examples[self.selected].0
                ));
                window.request_redraw();
            }
        }
    }

    fn update_cursor(&self) {
        if let (Some((x, y)), Some(window)) = (self.cursor_pos, &self.window) {
            // Check sidebar buttons
            if x < SIDEBAR_WIDTH {
                let y_offset = y - 48.0; // Skip header
                if y_offset > 0.0 {
                    let btn_idx = (y_offset / (BUTTON_HEIGHT + BUTTON_PADDING)) as usize;
                    if btn_idx < self.examples.len() {
                        window.set_cursor(CursorIcon::Pointer);
                        return;
                    }
                }
            }
            // Check example content handlers
            if let Some(layout) = &self.layout {
                if hit_test_any_handler(&layout.root, x - SIDEBAR_WIDTH, y, "click") {
                    window.set_cursor(CursorIcon::Pointer);
                    return;
                }
            }
            window.set_cursor(CursorIcon::Default);
        }
    }

    fn handle_click(&mut self, x: f32, y: f32) {
        // Check sidebar clicks
        if x < SIDEBAR_WIDTH {
            let y_offset = y - 48.0; // Skip header
            if y_offset > 0.0 {
                let btn_idx = (y_offset / (BUTTON_HEIGHT + BUTTON_PADDING)) as usize;
                if btn_idx < self.examples.len() {
                    self.select_example(btn_idx);
                    return;
                }
            }
        }

        // Check example content clicks
        let content_x = x - SIDEBAR_WIDTH;
        if content_x >= 0.0 {
            if let Some(layout) = &self.layout {
                let handlers = find_click_handlers(&layout.root, content_x, y);
                let mut changed = false;
                for handler in &handlers {
                    if execute_action(&handler.action, &mut self.state_store) {
                        changed = true;
                    }
                }
                if changed {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
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

        let mut pixmap = match tiny_skia::Pixmap::new(w, h) {
            Some(p) => p,
            None => return,
        };

        // Fill background
        pixmap.fill(tiny_skia::Color::from_rgba8(248, 250, 252, 255));

        // Draw sidebar
        self.draw_sidebar(&mut pixmap, h);

        // Draw example content
        let content_w = w as f32 - SIDEBAR_WIDTH;
        let content_h = h as f32;
        if content_w > 0.0 {
            let tree = &self.examples[self.selected].1;
            let resolved = resolve_tree(tree, &self.state_store);
            let layout = naze_layout::compute_layout(&resolved, content_w, content_h);
            self.layout = Some(layout.clone());

            // Offset and draw content
            self.draw_content(&mut pixmap, &layout, SIDEBAR_WIDTH);
        }

        // Present
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

    fn draw_sidebar(&self, pixmap: &mut tiny_skia::Pixmap, height: u32) {
        use tiny_skia::{Paint, Rect, Shader, Transform};

        // Sidebar background
        let sidebar_bg = Paint {
            shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(26, 26, 46, 255)),
            ..Paint::default()
        };
        if let Some(rect) = Rect::from_xywh(0.0, 0.0, SIDEBAR_WIDTH, height as f32) {
            pixmap.fill_rect(rect, &sidebar_bg, Transform::identity(), None);
        }

        // Title
        native_renderer::draw_tree(
            pixmap,
            &LayoutTree {
                title: String::new(),
                root: vec![PositionedNode {
                    kind: "heading".to_string(),
                    x: 16.0,
                    y: 12.0,
                    width: SIDEBAR_WIDTH - 32.0,
                    height: 28.0,
                    props: {
                        let mut p = HashMap::new();
                        p.insert(
                            "__text".to_string(),
                            RenderValue::Str("Naze Gallery".to_string()),
                        );
                        p.insert("font-size".to_string(), RenderValue::Num(18.0, None));
                        p.insert("color".to_string(), RenderValue::Color(0xFFFFFF));
                        p
                    },
                    children: vec![],
                    handlers: vec![],
                    scroll_info: None,
                }],
            },
            &self.font,
            None, // Gallery doesn't track focused inputs
        );

        // Example buttons
        let mut y = 48.0;
        for (idx, (name, _)) in self.examples.iter().enumerate() {
            let is_selected = idx == self.selected;

            // Button background
            let btn_color = if is_selected {
                tiny_skia::Color::from_rgba8(59, 130, 246, 255) // Blue
            } else {
                tiny_skia::Color::from_rgba8(51, 51, 51, 255) // Dark gray
            };
            let btn_paint = Paint {
                shader: Shader::SolidColor(btn_color),
                ..Paint::default()
            };
            if let Some(rect) = Rect::from_xywh(8.0, y, SIDEBAR_WIDTH - 16.0, BUTTON_HEIGHT) {
                pixmap.fill_rect(rect, &btn_paint, Transform::identity(), None);
            }

            // Button text
            native_renderer::draw_tree(
                pixmap,
                &LayoutTree {
                    title: String::new(),
                    root: vec![PositionedNode {
                        kind: "text".to_string(),
                        x: 16.0,
                        y: y + 6.0,
                        width: SIDEBAR_WIDTH - 32.0,
                        height: 20.0,
                        props: {
                            let mut p = HashMap::new();
                            p.insert("__text".to_string(), RenderValue::Str(name.clone()));
                            p.insert("font-size".to_string(), RenderValue::Num(14.0, None));
                            p.insert("color".to_string(), RenderValue::Color(0xFFFFFF));
                            p
                        },
                        children: vec![],
                        handlers: vec![],
                        scroll_info: None,
                    }],
                },
                &self.font,
                None, // Gallery doesn't track focused inputs
            );

            y += BUTTON_HEIGHT + BUTTON_PADDING;
        }
    }

    fn draw_content(&self, pixmap: &mut tiny_skia::Pixmap, layout: &LayoutTree, x_offset: f32) {
        // Offset all nodes by x_offset
        let offset_layout = LayoutTree {
            title: layout.title.clone(),
            root: offset_nodes(&layout.root, x_offset, 0.0),
        };
        native_renderer::draw_tree(pixmap, &offset_layout, &self.font, None);
    }
}

fn offset_nodes(nodes: &[PositionedNode], dx: f32, dy: f32) -> Vec<PositionedNode> {
    nodes
        .iter()
        .map(|node| PositionedNode {
            kind: node.kind.clone(),
            x: node.x + dx,
            y: node.y + dy,
            width: node.width,
            height: node.height,
            props: node.props.clone(),
            children: offset_nodes(&node.children, dx, dy),
            handlers: node.handlers.clone(),
            scroll_info: node.scroll_info.clone(),
        })
        .collect()
}

// ─── State resolution (duplicated from run.rs to avoid circular deps) ───────

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
                } else if node.kind == "input" {
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

fn find_click_handlers(nodes: &[PositionedNode], x: f32, y: f32) -> Vec<naze_ir::IrEventHandler> {
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

fn execute_action(action: &IrAction, state: &mut HashMap<String, RenderValue>) -> bool {
    match action {
        IrAction::Set { target, expr } => {
            let value = evaluate_expr(expr, state);
            state.insert(target.clone(), value);
            true
        }
        IrAction::Navigate { .. } => false,
        IrAction::ScrollTo { .. } => {
            // Scrolling not yet implemented in gallery
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
    }
}

fn evaluate_expr(expr: &IrExpression, state: &HashMap<String, RenderValue>) -> RenderValue {
    match expr {
        IrExpression::Num(n) => RenderValue::Num(*n, None),
        IrExpression::Str(s) => RenderValue::Str(s.clone()),
        IrExpression::Bool(b) => RenderValue::Bool(*b),
        IrExpression::StateRef(name) => state
            .get(name)
            .cloned()
            .unwrap_or(RenderValue::Num(0.0, None)),
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

/// Run the native gallery window.
fn run_native(examples_dir: &Path, examples: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Loading {} examples for native gallery...", examples.len());

    // Load font
    let font_bytes = include_bytes!("../fonts/DejaVuSans.ttf");
    let font = fontdue::Font::from_bytes(font_bytes as &[u8], fontdue::FontSettings::default())
        .map_err(|e| format!("failed to load font: {}", e))?;

    // Build all examples into RenderTrees
    let mut loaded: Vec<(String, RenderTree)> = Vec::new();
    for name in examples {
        let entry = format!("{}.naze", name);
        let project = resolve::resolve(examples_dir, &entry);

        let resolve_errors: Vec<_> = project
            .errors
            .iter()
            .filter(|e| matches!(e.severity, Severity::Error))
            .collect();
        if !resolve_errors.is_empty() {
            eprintln!("  skipping {} (resolve errors)", name);
            continue;
        }

        let tc_errors = typecheck::typecheck(&project);
        let tc_hard: Vec<_> = tc_errors
            .iter()
            .filter(|e| matches!(e.severity, Severity::Error))
            .collect();
        if !tc_hard.is_empty() {
            eprintln!("  skipping {} (type errors)", name);
            continue;
        }

        let tree = codegen::lower(&project);
        loaded.push((name.clone(), tree));
        eprint!(".");
    }
    eprintln!(" done");

    if loaded.is_empty() {
        return Err("No examples could be loaded".into());
    }

    // Initialize state for first example
    let mut state_store = HashMap::new();
    for decl in &loaded[0].1.state {
        state_store.insert(decl.name.clone(), decl.initial.clone());
    }

    let event_loop = EventLoop::new()?;
    let mut gallery = NativeGallery {
        examples: loaded,
        selected: 0,
        state_store,
        font,
        window: None,
        surface: None,
        layout: None,
        cursor_pos: None,
    };

    event_loop.run_app(&mut gallery)?;

    Ok(())
}

/// Main entry point for the gallery command.
pub fn run(build_only: bool, native: bool) -> Result<(), Box<dyn std::error::Error>> {
    let examples_dir = find_examples_dir()?;

    // Find all examples
    let examples = find_examples(&examples_dir)?;

    // Native mode: skip web build, run in native window
    if native {
        eprintln!("Starting native gallery...");
        eprintln!("  examples: {}", examples_dir.display());
        eprintln!("  found {} examples", examples.len());
        return run_native(&examples_dir, &examples);
    }

    // Web mode: build to dist directory
    let output_dir = examples_dir.join("dist");

    eprintln!("Building example gallery...");
    eprintln!("  examples: {}", examples_dir.display());
    eprintln!("  output: {}", output_dir.display());

    // Clean and create output directory
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)?;
    }
    fs::create_dir_all(&output_dir)?;

    eprintln!("  found {} examples", examples.len());

    // Build each example
    for (i, name) in examples.iter().enumerate() {
        eprint!("  [{}/{}] building {}...", i + 1, examples.len(), name);
        match build_example(&examples_dir, name, &output_dir) {
            Ok(()) => eprintln!(" ok"),
            Err(e) => {
                eprintln!(" FAILED");
                eprintln!("    {}", e);
                // Continue with other examples
            }
        }
    }

    // Copy runtime files
    copy_runtime(&output_dir)?;

    // Generate gallery HTML
    generate_gallery_html(&examples, &output_dir)?;

    eprintln!("Gallery built successfully!");

    if !build_only {
        serve_and_open(&output_dir)?;
    }

    Ok(())
}
