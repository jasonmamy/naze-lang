use std::cell::RefCell;
use std::collections::HashMap;

use js_sys::RegExp;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use naze_ir::{IrAction, IrBinOp, IrExpression, PageDef, RenderNode, RenderTree, RenderValue, TextPart};
use naze_layout::{self, LayoutTree, PositionedNode, ScrollInfo};
use naze_renderer::{self, canvas::Renderer};

// ─── App state ──────────────────────────────────────────────────────────────

/// Tracks which text input is currently focused.
#[derive(Clone)]
struct FocusedInput {
    bind_var: String,   // The state variable bound to this input
    node_id: String,    // Unique identifier for the input node (x,y position)
    input_type: String, // Input type (text, password, email, number)
    change_handlers: Vec<naze_ir::IrEventHandler>, // Change handlers for this input
    validate_prop: Option<RenderValue>, // Validation rules (if any)
}

/// Tracks drag & drop state during a drag operation.
#[derive(Clone)]
struct DragState {
    source_node_id: String,         // ID of element being dragged
    source_bounds: (f32, f32, f32, f32), // (x, y, width, height) of source element
    source_color: String,           // Color of source element for ghost
    drag_data: RenderValue,         // Data attached to drag
    start_x: f32,
    start_y: f32,
    current_x: f32,
    current_y: f32,
    over_target_id: Option<String>, // Current drop target being hovered
}

/// Tracks scroll position for a scroll container.
#[derive(Clone, Default)]
struct ScrollState {
    scroll_x: f32,
    scroll_y: f32,
}

/// Easing functions for animations.
#[derive(Clone, Copy, Debug, PartialEq)]
enum EasingFn {
    Linear,
    Ease,       // cubic-bezier(0.25, 0.1, 0.25, 1.0)
    EaseIn,     // cubic-bezier(0.42, 0, 1.0, 1.0)
    EaseOut,    // cubic-bezier(0, 0, 0.58, 1.0)
    EaseInOut,  // cubic-bezier(0.42, 0, 0.58, 1.0)
}

impl EasingFn {
    fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "linear" => EasingFn::Linear,
            "ease" => EasingFn::Ease,
            "ease-in" => EasingFn::EaseIn,
            "ease-out" => EasingFn::EaseOut,
            "ease-in-out" => EasingFn::EaseInOut,
            _ => EasingFn::Ease, // Default
        }
    }

    /// Apply easing function to a progress value (0.0 to 1.0).
    fn apply(&self, t: f64) -> f64 {
        match self {
            EasingFn::Linear => t,
            EasingFn::Ease => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            EasingFn::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
            EasingFn::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
            EasingFn::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
        }
    }
}

/// Cubic bezier easing (simplified implementation using Newton-Raphson).
fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, t: f64) -> f64 {
    // For CSS cubic-bezier, t is the progress (0-1) and we need to find the y value.
    // First find the parameter t_bezier such that bezier_x(t_bezier) = t
    // Then return bezier_y(t_bezier)

    // Newton-Raphson iteration to find t_bezier
    let mut t_bezier = t; // Initial guess
    for _ in 0..8 {
        let x = bezier_sample(x1, x2, t_bezier) - t;
        if x.abs() < 1e-6 {
            break;
        }
        let dx = bezier_derivative(x1, x2, t_bezier);
        if dx.abs() < 1e-6 {
            break;
        }
        t_bezier -= x / dx;
    }

    bezier_sample(y1, y2, t_bezier)
}

/// Sample a cubic bezier curve (one coordinate).
fn bezier_sample(p1: f64, p2: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    // p0 = 0, p3 = 1 for CSS cubic-bezier
    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

/// Derivative of cubic bezier (for Newton-Raphson).
fn bezier_derivative(p1: f64, p2: f64, t: f64) -> f64 {
    let t2 = t * t;
    let mt = 1.0 - t;
    // Derivative of: 3*(1-t)^2*t*p1 + 3*(1-t)*t^2*p2 + t^3
    3.0 * mt * mt * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t2 * (1.0 - p2)
}

/// Parsed transition specification.
#[derive(Clone, Debug)]
struct TransitionSpec {
    property: String,      // e.g., "color", "opacity", "background"
    duration_ms: f64,      // Duration in milliseconds
    easing: EasingFn,
}

impl TransitionSpec {
    /// Parse transition string: "color 150ms ease" or "opacity 200ms linear"
    fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        let property = parts[0].to_string();
        let mut duration_ms = 200.0; // Default 200ms
        let mut easing = EasingFn::Ease;

        for part in parts.iter().skip(1) {
            if part.ends_with("ms") {
                if let Ok(d) = part.trim_end_matches("ms").parse::<f64>() {
                    duration_ms = d;
                }
            } else if part.ends_with('s') && !part.ends_with("ms") {
                if let Ok(d) = part.trim_end_matches('s').parse::<f64>() {
                    duration_ms = d * 1000.0;
                }
            } else {
                easing = EasingFn::from_str(part);
            }
        }

        Some(TransitionSpec { property, duration_ms, easing })
    }
}

/// An active animation for a specific element property.
#[derive(Clone, Debug)]
struct ActiveAnimation {
    node_key: String,       // Unique key for the node (based on position/id)
    property: String,       // Property being animated
    start_value: AnimValue, // Starting value
    end_value: AnimValue,   // Target value
    start_time: f64,        // Animation start time (performance.now())
    duration_ms: f64,       // Duration in milliseconds
    easing: EasingFn,
}

/// Values that can be animated.
#[derive(Clone, Debug)]
enum AnimValue {
    Number(f64),
    Color(u32),  // RRGGBB
}

impl AnimValue {
    /// Interpolate between two values based on progress (0.0 to 1.0).
    fn interpolate(&self, other: &AnimValue, progress: f64) -> AnimValue {
        match (self, other) {
            (AnimValue::Number(a), AnimValue::Number(b)) => {
                AnimValue::Number(a + (b - a) * progress)
            }
            (AnimValue::Color(a), AnimValue::Color(b)) => {
                let r1 = ((a >> 16) & 0xFF) as f64;
                let g1 = ((a >> 8) & 0xFF) as f64;
                let b1 = (a & 0xFF) as f64;
                let r2 = ((b >> 16) & 0xFF) as f64;
                let g2 = ((b >> 8) & 0xFF) as f64;
                let b2 = (b & 0xFF) as f64;
                let r = (r1 + (r2 - r1) * progress) as u32;
                let g = (g1 + (g2 - g1) * progress) as u32;
                let b_val = (b1 + (b2 - b1) * progress) as u32;
                AnimValue::Color((r << 16) | (g << 8) | b_val)
            }
            _ => other.clone(), // Mismatched types, just return target
        }
    }

    /// Convert to RenderValue.
    fn to_render_value(&self) -> RenderValue {
        match self {
            AnimValue::Number(n) => RenderValue::Num(*n, None),
            AnimValue::Color(c) => RenderValue::Color(*c),
        }
    }
}

/// Persistent app state for the render loop.
struct App {
    render_tree: RenderTree,
    state_store: HashMap<String, RenderValue>,
    renderer: Renderer,
    layout: Option<LayoutTree>,
    raf_pending: bool,
    current_path: String,  // Current page path for routing
    focused_input: Option<FocusedInput>,  // Currently focused text input
    focused_element_id: Option<String>,  // Currently focused element (for keyboard nav)
    hovered_element_id: Option<String>,  // Element currently under mouse (for hover events)
    // Animation state
    animations: Vec<ActiveAnimation>,    // Currently running animations
    prev_props: HashMap<String, HashMap<String, RenderValue>>,  // Previous prop values by node key
    open_select_id: Option<String>,  // Currently open select dropdown
    caret_visible: bool,  // Blinking caret state
    caret_interval_id: Option<i32>,  // setInterval ID for caret blinking
    drag_state: Option<DragState>,   // Active drag operation
    scroll_states: HashMap<String, ScrollState>,  // Scroll position per container
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

    // 3. Initialize data state variables (loading=true, error=null, data=null)
    for decl in &render_tree.data {
        state_store.insert(format!("{}.loading", decl.name), RenderValue::Bool(true));
        state_store.insert(format!("{}.error", decl.name), RenderValue::Str(String::new()));
        state_store.insert(format!("{}.data", decl.name), RenderValue::List(vec![]));
    }

    // 4. Set up the renderer
    let renderer = Renderer::new(canvas_id)?;

    // 5. Get initial path from URL or default to "/"
    let current_path = get_current_path();

    // 6. Collect data URLs for fetching after setup
    let data_fetches: Vec<(String, String)> = render_tree.data.iter()
        .map(|d| (d.name.clone(), d.url.clone()))
        .collect();

    // 7. Store in global for render loop and future event handlers
    APP.with(|cell| {
        *cell.borrow_mut() = Some(App {
            render_tree,
            state_store,
            renderer,
            layout: None,
            raf_pending: false,
            current_path,
            focused_input: None,
            focused_element_id: None,
            hovered_element_id: None,
            animations: Vec::new(),
            prev_props: HashMap::new(),
            open_select_id: None,
            caret_visible: true,
            caret_interval_id: None,
            drag_state: None,
            scroll_states: HashMap::new(),
        });
    });

    // 8. Create hidden input element for text input focus
    create_hidden_input(canvas_id)?;

    // 9. Create screen reader accessibility container
    create_a11y_container()?;

    // 10. Initial render (synchronous)
    do_render()?;

    // 11. Set up event listeners on the canvas
    setup_event_listeners()?;

    // 12. Set up popstate handler for browser back/forward
    setup_popstate_handler()?;

    // 13. Fire off data fetches
    for (name, url) in data_fetches {
        fetch_data(&name, &url);
    }

    Ok(())
}

/// Get the current path from the URL hash or pathname.
fn get_current_path() -> String {
    if let Some(window) = web_sys::window() {
        // Use hash-based routing for simplicity (works without server config)
        if let Ok(hash) = window.location().hash() {
            if hash.len() > 1 {
                // Remove leading # and return
                return hash[1..].to_string();
            }
        }
        // Fall back to pathname
        if let Ok(pathname) = window.location().pathname() {
            return pathname;
        }
    }
    "/".to_string()
}

/// Reload the app with new app_data without reinitializing WASM.
/// Used by the gallery to switch between examples.
#[wasm_bindgen]
pub fn reset_and_reload(app_data: &[u8]) -> Result<(), JsValue> {
    // 1. Deserialize the new render tree
    let render_tree: RenderTree = naze_ir::deserialize(app_data)
        .map_err(|e| JsValue::from_str(&format!("failed to deserialize app data: {}", e)))?;

    // 2. Initialize state store from new declarations
    let mut state_store = HashMap::new();
    for decl in &render_tree.state {
        state_store.insert(decl.name.clone(), decl.initial.clone());
    }

    // 3. Update global app state
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if let Some(app) = borrow.as_mut() {
            app.render_tree = render_tree;
            app.state_store = state_store;
            app.layout = None; // Force re-layout on next render
            app.current_path = "/".to_string(); // Reset to home page
            app.focused_input = None; // Clear focused input
            app.focused_element_id = None; // Clear focused element
            app.hovered_element_id = None; // Clear hovered element
            app.open_select_id = None; // Clear open select
            // Stop caret timer if running
            if let Some(id) = app.caret_interval_id.take() {
                if let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(id);
                }
            }
            app.caret_visible = true;
            app.caret_interval_id = None;
            app.drag_state = None; // Clear drag state
            app.scroll_states.clear(); // Clear scroll positions
        }
    });

    // Blur hidden input when switching examples
    blur_hidden_input();

    // 4. Re-render with new content
    do_render()?;

    Ok(())
}

// ─── Render loop ────────────────────────────────────────────────────────────

/// Get current time in milliseconds from performance.now().
fn get_now_ms() -> f64 {
    if let Some(window) = web_sys::window() {
        if let Ok(perf) = window.performance().ok_or("no performance") {
            return perf.now();
        }
    }
    0.0
}

/// Extract AnimValue from a RenderValue if it's animatable.
fn render_value_to_anim(value: &RenderValue) -> Option<AnimValue> {
    match value {
        RenderValue::Num(n, _) => Some(AnimValue::Number(*n)),
        RenderValue::Color(c) => Some(AnimValue::Color(*c)),
        _ => None,
    }
}

/// Parse transition specs from a node's props.
fn parse_transitions(props: &HashMap<String, RenderValue>) -> Vec<TransitionSpec> {
    let mut specs = Vec::new();
    if let Some(RenderValue::Str(transition_str)) = props.get("transition") {
        // Can have multiple transitions separated by commas
        for part in transition_str.split(',') {
            if let Some(spec) = TransitionSpec::parse(part.trim()) {
                specs.push(spec);
            }
        }
    }
    specs
}

/// Process animations for a render tree, starting new animations and interpolating values.
/// Returns (modified_props_by_node_key, has_active_animations).
fn process_animations(
    root: &[RenderNode],
    animations: &mut Vec<ActiveAnimation>,
    prev_props: &mut HashMap<String, HashMap<String, RenderValue>>,
    now: f64,
) -> (HashMap<String, HashMap<String, RenderValue>>, bool) {
    let mut animated_props: HashMap<String, HashMap<String, RenderValue>> = HashMap::new();

    // First, remove completed animations
    animations.retain(|anim| {
        let elapsed = now - anim.start_time;
        elapsed < anim.duration_ms
    });

    // Compute interpolated values for active animations
    for anim in animations.iter() {
        let elapsed = now - anim.start_time;
        let progress = (elapsed / anim.duration_ms).min(1.0);
        let eased = anim.easing.apply(progress);
        let current = anim.start_value.interpolate(&anim.end_value, eased);

        animated_props
            .entry(anim.node_key.clone())
            .or_default()
            .insert(anim.property.clone(), current.to_render_value());
    }

    // Walk tree to detect new property changes that need animations
    detect_new_animations(root, "", animations, prev_props, now);

    // Check if we have any active animations
    let has_active = !animations.is_empty();

    (animated_props, has_active)
}

/// Walk the tree to detect property changes and start new animations.
fn detect_new_animations(
    nodes: &[RenderNode],
    parent_key: &str,
    animations: &mut Vec<ActiveAnimation>,
    prev_props: &mut HashMap<String, HashMap<String, RenderValue>>,
    now: f64,
) {
    for (i, node) in nodes.iter().enumerate() {
        // Generate node key based on position and optional id
        let node_key = if let Some(RenderValue::Str(id)) = node.props.get("id") {
            format!("{}_{}", parent_key, id)
        } else {
            format!("{}_{}_{}", parent_key, node.kind, i)
        };

        // Get transitions defined on this node
        let transitions = parse_transitions(&node.props);

        if !transitions.is_empty() {
            // Get previous props for this node
            let prev = prev_props.entry(node_key.clone()).or_default();

            for spec in transitions {
                // Get current value for the transitioned property
                if let Some(current_value) = node.props.get(&spec.property) {
                    // Check if there's a previous value that's different
                    if let Some(prev_value) = prev.get(&spec.property) {
                        if current_value != prev_value {
                            // Value changed! Check if animatable
                            if let (Some(start), Some(end)) = (
                                render_value_to_anim(prev_value),
                                render_value_to_anim(current_value),
                            ) {
                                // Remove any existing animation for this property
                                animations.retain(|a| {
                                    !(a.node_key == node_key && a.property == spec.property)
                                });

                                // Start new animation
                                animations.push(ActiveAnimation {
                                    node_key: node_key.clone(),
                                    property: spec.property.clone(),
                                    start_value: start,
                                    end_value: end,
                                    start_time: now,
                                    duration_ms: spec.duration_ms,
                                    easing: spec.easing,
                                });
                            }
                        }
                    }

                    // Update previous value
                    prev.insert(spec.property.clone(), current_value.clone());
                }
            }
        }

        // Recurse into children
        detect_new_animations(&node.children, &node_key, animations, prev_props, now);
    }
}

/// Apply animated values to a render tree, creating a new tree with interpolated values.
fn apply_animated_values(
    nodes: &[RenderNode],
    animated_props: &HashMap<String, HashMap<String, RenderValue>>,
    parent_key: &str,
) -> Vec<RenderNode> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            // Generate node key
            let node_key = if let Some(RenderValue::Str(id)) = node.props.get("id") {
                format!("{}_{}", parent_key, id)
            } else {
                format!("{}_{}_{}", parent_key, node.kind, i)
            };

            // Create new props with animated values applied
            let mut new_props = node.props.clone();
            if let Some(node_anims) = animated_props.get(&node_key) {
                for (prop_name, value) in node_anims {
                    new_props.insert(prop_name.clone(), value.clone());
                }
            }

            RenderNode {
                kind: node.kind.clone(),
                props: new_props,
                children: apply_animated_values(&node.children, animated_props, &node_key),
                handlers: node.handlers.clone(),
                condition: node.condition.clone(),
                else_children: node.else_children.clone(),
                each_binding: node.each_binding.clone(),
            }
        })
        .collect()
}

/// Schedule an animation frame (for continuous animations).
/// Unlike schedule_render, this always schedules even if one is pending.
fn schedule_animation_frame() {
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if let Some(app) = borrow.as_mut() {
            app.raf_pending = true;
        }
    });

    let cb = Closure::once(|| {
        let _ = do_render();
    });

    if let Some(window) = web_sys::window() {
        let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
    }
    cb.forget();
}

/// Perform a full render: resolve state → layout → draw.
fn do_render() -> Result<(), JsValue> {
    let has_animations: Result<bool, JsValue> = APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let app = borrow
            .as_mut()
            .ok_or_else(|| JsValue::from_str("app not initialized"))?;
        app.raf_pending = false;

        // Get current time for animation processing
        let now = get_now_ms();

        // 1. Resolve interpolated strings against current state
        let resolved = resolve_tree(&app.render_tree, &app.state_store);

        // 2. Build combined tree: root content (headers, nav) + current page content
        let page_nodes = get_page_nodes(&resolved, &app.current_path);
        let combined_root: Vec<RenderNode> = if resolved.pages.is_empty() {
            // Single-page app - just use root
            resolved.root.clone()
        } else {
            // Multi-page app - combine root (shared) + page-specific content
            let mut combined = resolved.root.clone();
            combined.extend(page_nodes.iter().cloned());
            combined
        };

        // 2a. Process animations and detect new ones
        let (animated_props, has_active) = process_animations(
            &combined_root,
            &mut app.animations,
            &mut app.prev_props,
            now,
        );

        // 2b. Apply animated values to the tree
        let animated_root = apply_animated_values(&combined_root, &animated_props, "");

        let combined_tree = RenderTree {
            title: resolved.title.clone(),
            state: resolved.state.clone(),
            data: vec![],
            root: animated_root,
            pages: vec![],
        };

        // 3. Get viewport size
        let window = web_sys::window().ok_or("no window")?;
        let vw = window.inner_width()?.as_f64().unwrap_or(1024.0) as f32;
        let vh = window.inner_height()?.as_f64().unwrap_or(768.0) as f32;

        // 4. Set canvas size to viewport
        app.renderer.set_size(vw as f64, vh as f64);

        // 5. Compute layout
        let layout = {
            let renderer = &app.renderer;
            let text_measure = |text: &str, font_size: f32| -> (f32, f32) {
                let is_heading = font_size > 20.0;
                let (w, h) = renderer.measure_text(text, font_size as f64, is_heading);
                (w as f32, h as f32)
            };
            naze_layout::compute_layout_with_measure(&combined_tree, vw, vh, text_measure)
        };

        // 6. Set document title
        if let Some(document) = window.document() {
            document.set_title(&layout.title);
        }

        // 7. Clear and draw
        app.renderer.clear();
        let focused_input_id: Option<String> = app.focused_input.as_ref().map(|f| f.node_id.clone());
        let focused_element_id: Option<String> = app.focused_element_id.clone();
        let open_select: Option<String> = app.open_select_id.clone();
        let caret_visible = app.caret_visible;
        draw_tree(&app.renderer, &layout, &app.state_store, focused_input_id.as_deref(), focused_element_id.as_deref(), open_select.as_deref(), caret_visible, &app.scroll_states);

        // 8. Draw drag ghost and drop zone highlighting if dragging
        if let Some(ref drag) = app.drag_state {
            // Draw drop zone highlight if over a target
            if let Some(ref target_id) = drag.over_target_id {
                // Find the target's bounds
                if let Some(target_info) = find_drop_target_at_point(&layout.root, drag.current_x, drag.current_y) {
                    if target_info.node_id == *target_id {
                        let (tx, ty, tw, th) = target_info.bounds;
                        app.renderer.draw_drop_highlight(tx as f64, ty as f64, tw as f64, th as f64);
                    }
                }
            }

            // Draw ghost element at current position
            let (_, _, sw, sh) = drag.source_bounds;
            let ghost_x = drag.current_x - (sw / 2.0);
            let ghost_y = drag.current_y - (sh / 2.0);
            app.renderer.draw_drag_ghost(ghost_x as f64, ghost_y as f64, sw as f64, sh as f64, &drag.source_color);
        }

        // 9. Update screen reader accessibility DOM
        update_a11y_dom(&layout);

        // 10. Store layout for hit testing
        app.layout = Some(layout);

        Ok(has_active)
    });

    // If there are active animations, schedule another render
    if let Ok(true) = has_animations {
        schedule_animation_frame();
    }

    has_animations.map(|_| ())
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
        data: tree.data.clone(),
        root: resolve_nodes(&tree.root, state),
        pages: tree.pages.iter().map(|page| PageDef {
            path: page.path.clone(),
            root: resolve_nodes(&page.root, state),
        }).collect(),
    }
}

/// Get the content nodes for the current page.
/// If the app has pages, returns the matching page's content.
/// Otherwise returns the root nodes.
fn get_page_nodes<'a>(tree: &'a RenderTree, current_path: &str) -> &'a [RenderNode] {
    if tree.pages.is_empty() {
        return &tree.root;
    }

    // Find matching page
    for page in &tree.pages {
        if page.path == current_path {
            return &page.root;
        }
    }

    // Try to find "/" page as fallback
    for page in &tree.pages {
        if page.path == "/" {
            return &page.root;
        }
    }

    // Final fallback to root nodes (non-page content like navigation bars)
    &tree.root
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
fn draw_tree(
    renderer: &Renderer,
    layout: &LayoutTree,
    state: &HashMap<String, RenderValue>,
    focused_input_id: Option<&str>,
    focused_element_id: Option<&str>,
    open_select_id: Option<&str>,
    caret_visible: bool,
    scroll_states: &HashMap<String, ScrollState>,
) {
    for node in &layout.root {
        draw_node(renderer, node, state, focused_input_id, focused_element_id, open_select_id, caret_visible, scroll_states);
    }
}

/// Recursively draw a positioned node and its children.
fn draw_node(
    renderer: &Renderer,
    node: &PositionedNode,
    state: &HashMap<String, RenderValue>,
    focused_input_id: Option<&str>,
    focused_element_id: Option<&str>,
    open_select_id: Option<&str>,
    caret_visible: bool,
    scroll_states: &HashMap<String, ScrollState>,
) {
    let x = node.x as f64;
    let y = node.y as f64;
    let w = node.width as f64;
    let h = node.height as f64;

    // Check if this element is focused (for focus ring)
    let this_element_id = format!("focus_{}_{}_{}", node.kind, node.x as i32, node.y as i32);
    let is_focused = focused_element_id == Some(this_element_id.as_str());

    // Handle opacity - wrap in save/restore if not 1.0
    let opacity = naze_renderer::get_num_prop(&node.props, "opacity", 1.0);
    let needs_opacity = opacity < 1.0;
    if needs_opacity {
        renderer.save();
        renderer.set_global_alpha(opacity);
    }

    match node.kind.as_str() {
        "rect" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            let border = naze_renderer::get_num_prop(&node.props, "border", 0.0);
            let border_color = naze_renderer::get_color_prop(&node.props, "border-color", "#000000");
            renderer.draw_rect_with_border(x, y, w, h, &color, radius, border, &border_color);
        }
        "container" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            let border = naze_renderer::get_num_prop(&node.props, "border", 0.0);
            let border_color = naze_renderer::get_color_prop(&node.props, "border-color", "#000000");
            if !color.is_empty() || border > 0.0 {
                renderer.draw_rect_with_border(x, y, w, h, &color, radius, border, &border_color);
            }
            for child in &node.children {
                draw_node(renderer, child, state, focused_input_id, focused_element_id, open_select_id, caret_visible, scroll_states);
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
        "image" => {
            let src = naze_renderer::get_str_prop(&node.props, "src", "");
            let fit = naze_renderer::get_str_prop(&node.props, "fit", "contain");
            if !src.is_empty() {
                // Try to draw the image; if not loaded yet, start loading
                if !renderer.draw_image(&src, x, y, w, h, &fit) {
                    // Image not ready - start loading and schedule re-render
                    renderer.load_image(&src, || {
                        schedule_render();
                    });
                    // Draw placeholder
                    renderer.draw_rect(x, y, w, h, "#e5e5e5", 0.0);
                }
            }
        }
        "link" => {
            // Link element renders as clickable text
            let text = naze_renderer::get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = naze_renderer::get_font_size(&node.props, false);
                let color = naze_renderer::get_color_prop(&node.props, "color", "#3b82f6"); // Blue default
                renderer.draw_text(&text, x, y, font_size, false, &color);
            }
            for child in &node.children {
                draw_node(renderer, child, state, focused_input_id, focused_element_id, open_select_id, caret_visible, scroll_states);
            }
        }
        "checkbox" => {
            let label = naze_renderer::get_text_content(&node.props);
            let checked = match node.props.get("bind") {
                Some(RenderValue::Bind(var)) => {
                    match state.get(var) {
                        Some(RenderValue::Bool(b)) => *b,
                        _ => false,
                    }
                }
                _ => false,
            };
            renderer.draw_checkbox(x, y, checked, &label);
        }
        "radio" => {
            let label = naze_renderer::get_text_content(&node.props);
            // Radio is selected when state[bind] == value
            let selected = match (node.props.get("bind"), node.props.get("value")) {
                (Some(RenderValue::Bind(var)), Some(value)) => {
                    match state.get(var) {
                        Some(state_val) => state_val == value,
                        None => false,
                    }
                }
                _ => false,
            };
            renderer.draw_radio(x, y, selected, &label);
        }
        "input" => {
            let placeholder = naze_renderer::get_str_prop(&node.props, "placeholder", "");
            let input_type = naze_renderer::get_str_prop(&node.props, "type", "text");
            // Get current value from bind
            let value = match node.props.get("bind") {
                Some(RenderValue::Bind(var)) => {
                    match state.get(var) {
                        Some(RenderValue::Str(s)) => s.clone(),
                        _ => String::new(),
                    }
                }
                _ => String::new(),
            };
            // Check if this input is focused using node position as ID
            let node_id = format!("input_{}_{}", x as i32, y as i32);
            let focused = focused_input_id == Some(node_id.as_str());
            // Only show caret when focused AND caret is in visible phase
            let show_caret = focused && caret_visible;
            renderer.draw_input(x, y, w as f64, h as f64, &value, &placeholder, focused, &input_type, show_caret);
        }
        "select" => {
            let placeholder = naze_renderer::get_str_prop(&node.props, "placeholder", "Select...");
            // Get current value from bind
            let current_value = match node.props.get("bind") {
                Some(RenderValue::Bind(var)) => {
                    match state.get(var) {
                        Some(RenderValue::Str(s)) => s.clone(),
                        _ => String::new(),
                    }
                }
                _ => String::new(),
            };
            // Extract options from children
            let options = extract_select_options(&node.children);
            // Find display text for current value
            let display_text = options.iter()
                .find(|(_, v)| v == &current_value)
                .map(|(label, _)| label.as_str())
                .unwrap_or("");
            // Check if this select is open
            let select_id = format!("select_{}_{}", x as i32, y as i32);
            let is_open = open_select_id == Some(select_id.as_str());
            renderer.draw_select(x, y, w, h, display_text, &placeholder, is_open, &options, &current_value);
        }
        "option" => {
            // Options are rendered by the parent select, not directly
        }
        "scroll" => {
            // Scroll container: clip children, apply scroll offset, draw scrollbar
            let color = naze_renderer::get_color_prop(&node.props, "color", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            let border = naze_renderer::get_num_prop(&node.props, "border", 0.0);
            let border_color = naze_renderer::get_color_prop(&node.props, "border-color", "#000000");

            // Draw background
            if !color.is_empty() || border > 0.0 {
                renderer.draw_rect_with_border(x, y, w, h, &color, radius, border, &border_color);
            }

            // Get scroll state
            let scroll_id = format!("scroll_{}_{}", node.x as i32, node.y as i32);
            let scroll_state = scroll_states.get(&scroll_id).cloned().unwrap_or_default();
            let scroll_x = scroll_state.scroll_x as f64;
            let scroll_y = scroll_state.scroll_y as f64;

            // Get scroll info
            let (content_w, content_h, overflow_x, overflow_y) = match &node.scroll_info {
                Some(info) => (
                    info.content_width as f64,
                    info.content_height as f64,
                    info.overflow_x,
                    info.overflow_y,
                ),
                None => (w, h, false, false),
            };

            // Begin clipping
            renderer.begin_clip(x, y, w, h, radius);

            // Translate by scroll offset
            renderer.translate(-scroll_x, -scroll_y);

            // Draw children
            for child in &node.children {
                draw_node(renderer, child, state, focused_input_id, focused_element_id, open_select_id, caret_visible, scroll_states);
            }

            // End clipping (restores transform)
            renderer.end_clip();

            // Draw scrollbars
            if overflow_y && content_h > h {
                let max_scroll = content_h - h;
                let thumb_size = (h / content_h) * h;
                let thumb_pos = (scroll_y / max_scroll) * (h - thumb_size);
                renderer.draw_scrollbar_vertical(x + w - 8.0, y, h, thumb_pos, thumb_size);
            }
            if overflow_x && content_w > w {
                let max_scroll = content_w - w;
                let thumb_size = (w / content_w) * w;
                let thumb_pos = (scroll_x / max_scroll) * (w - thumb_size);
                renderer.draw_scrollbar_horizontal(x, y + h - 8.0, w, thumb_pos, thumb_size);
            }
        }
        "row" | "column" | "stack" | "grid" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            if !color.is_empty() {
                renderer.draw_rect(x, y, w, h, &color, radius);
            }
            for child in &node.children {
                draw_node(renderer, child, state, focused_input_id, focused_element_id, open_select_id, caret_visible, scroll_states);
            }
        }
        "spacer" => {}
        _ => {
            for child in &node.children {
                draw_node(renderer, child, state, focused_input_id, focused_element_id, open_select_id, caret_visible, scroll_states);
            }
        }
    }

    // Draw focus ring if this element is focused
    if is_focused {
        let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
        renderer.draw_focus_ring(x, y, w, h, radius);
    }

    if needs_opacity {
        renderer.restore();
    }
}

/// Extract options from select children. Returns vec of (label, value).
fn extract_select_options(children: &[PositionedNode]) -> Vec<(String, String)> {
    let mut options = Vec::new();
    for child in children {
        if child.kind == "option" {
            let label = naze_renderer::get_text_content(&child.props);
            let value = naze_renderer::get_str_prop(&child.props, "value", &label);
            options.push((label, value));
        }
    }
    options
}

// ─── Event handling ──────────────────────────────────────────────────────────

/// Set up event listeners on the canvas for clicks, drags, and mouse movement.
fn setup_event_listeners() -> Result<(), JsValue> {
    // Get canvas element from the app
    let canvas = APP.with(|cell| {
        let borrow = cell.borrow();
        let app = borrow.as_ref().ok_or("app not initialized")?;
        Ok::<_, &str>(app.renderer.canvas_element().clone())
    }).map_err(|e| JsValue::from_str(e))?;

    // Mousedown handler — start drag if on draggable element
    let mousedown_cb = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let needs_render = handle_mousedown(x, y);
        if needs_render {
            schedule_render();
        }
    });
    canvas.add_event_listener_with_callback("mousedown", mousedown_cb.as_ref().unchecked_ref())?;
    mousedown_cb.forget();

    // Mouseup handler — complete drop or trigger click
    let mouseup_cb = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let needs_render = handle_mouseup(x, y);
        if needs_render {
            schedule_render();
        }
    });
    canvas.add_event_listener_with_callback("mouseup", mouseup_cb.as_ref().unchecked_ref())?;
    mouseup_cb.forget();

    // Mousemove handler — update drag position, check drop targets, set cursor
    let move_cb = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let needs_render = handle_mousemove(x, y);
        if needs_render {
            schedule_render();
        }
    });
    canvas.add_event_listener_with_callback("mousemove", move_cb.as_ref().unchecked_ref())?;
    move_cb.forget();

    // Wheel handler — scroll content in scroll containers
    let wheel_cb = Closure::<dyn Fn(web_sys::WheelEvent)>::new(move |event: web_sys::WheelEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let delta_x = event.delta_x() as f32;
        let delta_y = event.delta_y() as f32;
        let needs_render = handle_wheel(x, y, delta_x, delta_y);
        if needs_render {
            event.prevent_default();
            schedule_render();
        }
    });
    canvas.add_event_listener_with_callback("wheel", wheel_cb.as_ref().unchecked_ref())?;
    wheel_cb.forget();

    // Resize handler — re-layout and re-render when window size changes
    let window = web_sys::window().ok_or("no window")?;
    let resize_cb = Closure::<dyn Fn()>::new(|| {
        schedule_render();
    });
    window.add_event_listener_with_callback("resize", resize_cb.as_ref().unchecked_ref())?;
    resize_cb.forget();

    // Keydown handler — handle keyboard events for focused elements and tab navigation
    let keydown_cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
        let key = event.key();
        let needs_render = handle_keydown(&key, event.shift_key());
        if needs_render {
            schedule_render();
        }
    });
    window.add_event_listener_with_callback("keydown", keydown_cb.as_ref().unchecked_ref())?;
    keydown_cb.forget();

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

        // Check if clicking on an input element first
        if let Some((bind_var, node_id, current_value, input_type, change_handlers, validate_prop)) = find_input_at_point(&layout.root, x, y, &app.state_store) {
            // Close any open select when clicking an input
            app.open_select_id = None;
            // Drop borrow before calling focus_input which needs to borrow again
            drop(borrow);
            focus_input(&bind_var, &node_id, &current_value, &input_type, change_handlers, validate_prop);
            return true; // Needs re-render to show focus
        }

        // Check if clicking on a select dropdown option (when dropdown is open)
        if let Some(open_id) = &app.open_select_id.clone() {
            if let Some((bind_var, value, change_handlers)) = find_option_at_point(&layout.root, x, y, open_id) {
                // Set the value and close the dropdown
                app.state_store.insert(bind_var, RenderValue::Str(value));
                app.open_select_id = None;
                // Execute change handlers
                for handler in &change_handlers {
                    execute_action(&handler.action, &mut app.state_store);
                }
                return true;
            }
        }

        // Check if clicking on a select element
        if let Some((_bind_var, select_id)) = find_select_at_point(&layout.root, x, y) {
            // Toggle open/close
            if app.open_select_id.as_ref() == Some(&select_id) {
                app.open_select_id = None;
            } else {
                app.open_select_id = Some(select_id);
            }
            return true;
        }

        // If clicking elsewhere with an open select, close it
        if app.open_select_id.is_some() {
            app.open_select_id = None;
            return true;
        }

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
        // Input and select elements are clickable
        if event == "click" && (node.kind == "input" || node.kind == "select") {
            return true;
        }
        if node.handlers.iter().any(|h| h.event == event) {
            return true;
        }
    }
    false
}

/// Find click handlers on the deepest node at (x, y).
/// For form elements (checkbox, radio), also includes change handlers.
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
        let mut handlers: Vec<_> = node
            .handlers
            .iter()
            .filter(|h| h.event == "click")
            .cloned()
            .collect();
        // For form elements, also include change handlers (clicking triggers change)
        if node.kind == "checkbox" || node.kind == "radio" {
            handlers.extend(
                node.handlers.iter()
                    .filter(|h| h.event == "change")
                    .cloned()
            );
        }
        if !handlers.is_empty() {
            return handlers;
        }
    }
    Vec::new()
}

/// Check if a point is inside a node's bounding box.
fn point_in_node(node: &PositionedNode, x: f32, y: f32) -> bool {
    x >= node.x && x <= node.x + node.width && y >= node.y && y <= node.y + node.height
}

/// Find the deepest element with hover handlers at (x, y).
/// Returns (element_id, hover_handlers) if found.
fn find_hover_element(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
) -> Option<(String, Vec<naze_ir::IrEventHandler>)> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first (deeper wins)
        if let Some(result) = find_hover_element(&node.children, x, y) {
            return Some(result);
        }
        // Check if this node has hover handlers
        let handlers: Vec<_> = node
            .handlers
            .iter()
            .filter(|h| h.event == "hover")
            .cloned()
            .collect();
        if !handlers.is_empty() {
            let node_id = format!("hover_{}_{}", node.x as i32, node.y as i32);
            return Some((node_id, handlers));
        }
    }
    None
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
        IrAction::Navigate { path } => {
            navigate_to(path);
            true
        }
        IrAction::ScrollTo { element_id } => {
            scroll_to_element(element_id);
            true
        }
        IrAction::Log { expr } => {
            let value = evaluate_expr(expr, state);
            log_to_console(&value);
            false // no re-render needed
        }
    }
}

/// Log a value to the browser console.
fn log_to_console(value: &RenderValue) {
    let msg = match value {
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
        RenderValue::InterpolatedStr(parts) => {
            parts.iter().map(|p| match p {
                naze_ir::TextPart::Literal(s) => s.clone(),
                naze_ir::TextPart::StateRef(name) => format!("{{{}}}", name),
            }).collect()
        }
        RenderValue::List(items) => format!("[{} items]", items.len()),
        RenderValue::Object(entries) => format!("{{{}...}}", entries.len()),
        RenderValue::Bind(name) => format!("bind:{}", name),
    };
    web_sys::console::log_1(&msg.into());
}

/// Navigate to a new path using the History API.
fn navigate_to(path: &str) {
    if let Some(window) = web_sys::window() {
        // Update URL hash (hash-based routing for simplicity)
        let _ = window.location().set_hash(path);

        // Update current path in app state
        APP.with(|cell| {
            let mut borrow = cell.borrow_mut();
            if let Some(app) = borrow.as_mut() {
                app.current_path = path.to_string();
            }
        });
    }
}

/// Set up popstate handler for browser back/forward buttons.
fn setup_popstate_handler() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;

    let popstate_cb = Closure::<dyn Fn()>::new(|| {
        let new_path = get_current_path();
        APP.with(|cell| {
            let mut borrow = cell.borrow_mut();
            if let Some(app) = borrow.as_mut() {
                app.current_path = new_path;
            }
        });
        schedule_render();
    });

    window.add_event_listener_with_callback("hashchange", popstate_cb.as_ref().unchecked_ref())?;
    popstate_cb.forget();

    Ok(())
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

// ─── Input Validation ────────────────────────────────────────────────────────

/// Validation rules extracted from the validate prop.
struct ValidationRules {
    required: bool,
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<String>,
    min: Option<f64>,
    max: Option<f64>,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            required: false,
            min_length: None,
            max_length: None,
            pattern: None,
            min: None,
            max: None,
        }
    }
}

/// Extract validation rules from a validate prop (Object value).
fn extract_validation_rules(validate_prop: &RenderValue) -> ValidationRules {
    let mut rules = ValidationRules::default();

    if let RenderValue::Object(entries) = validate_prop {
        for (key, value) in entries {
            match key.as_str() {
                "required" => {
                    if let RenderValue::Bool(b) = value {
                        rules.required = *b;
                    }
                }
                "min-length" | "minLength" => {
                    if let RenderValue::Num(n, _) = value {
                        rules.min_length = Some(*n as usize);
                    }
                }
                "max-length" | "maxLength" => {
                    if let RenderValue::Num(n, _) = value {
                        rules.max_length = Some(*n as usize);
                    }
                }
                "pattern" => {
                    if let RenderValue::Str(s) = value {
                        rules.pattern = Some(s.clone());
                    }
                }
                "min" => {
                    if let RenderValue::Num(n, _) = value {
                        rules.min = Some(*n);
                    }
                }
                "max" => {
                    if let RenderValue::Num(n, _) = value {
                        rules.max = Some(*n);
                    }
                }
                _ => {}
            }
        }
    }

    rules
}

/// Validate a value against rules. Returns (is_valid, error_message).
fn validate_value(value: &str, rules: &ValidationRules, input_type: &str) -> (bool, String) {
    // Required check
    if rules.required && value.is_empty() {
        return (false, "This field is required".to_string());
    }

    // If empty and not required, skip other validations
    if value.is_empty() {
        return (true, String::new());
    }

    // Min length check
    if let Some(min_len) = rules.min_length {
        if value.len() < min_len {
            return (false, format!("Must be at least {} characters", min_len));
        }
    }

    // Max length check
    if let Some(max_len) = rules.max_length {
        if value.len() > max_len {
            return (false, format!("Must be at most {} characters", max_len));
        }
    }

    // Pattern check (basic regex matching via web-sys)
    if let Some(pattern) = &rules.pattern {
        if !match_pattern(value, pattern) {
            return (false, "Invalid format".to_string());
        }
    }

    // Number validations for number type inputs
    if input_type == "number" {
        if let Ok(num) = value.parse::<f64>() {
            if let Some(min) = rules.min {
                if num < min {
                    return (false, format!("Must be at least {}", min));
                }
            }
            if let Some(max) = rules.max {
                if num > max {
                    return (false, format!("Must be at most {}", max));
                }
            }
        } else {
            return (false, "Must be a valid number".to_string());
        }
    }

    // Email validation for email type inputs
    if input_type == "email" && !value.is_empty() {
        // Basic email pattern check
        if !match_pattern(value, r"^[^\s@]+@[^\s@]+\.[^\s@]+$") {
            return (false, "Invalid email address".to_string());
        }
    }

    (true, String::new())
}

/// Match a value against a regex pattern using JavaScript's RegExp.
fn match_pattern(value: &str, pattern: &str) -> bool {
    let re = RegExp::new(pattern, "");
    re.test(value)
}

/// Run validation for an input and update derived state variables.
/// Updates {bind_var}_valid (bool) and {bind_var}_error (string).
fn run_validation(
    state: &mut HashMap<String, RenderValue>,
    bind_var: &str,
    value: &str,
    validate_prop: Option<&RenderValue>,
    input_type: &str,
) {
    let (is_valid, error_msg) = if let Some(validate) = validate_prop {
        let rules = extract_validation_rules(validate);
        validate_value(value, &rules, input_type)
    } else {
        // No validation rules - always valid
        (true, String::new())
    };

    // Update derived state variables
    let valid_key = format!("{}_valid", bind_var);
    let error_key = format!("{}_error", bind_var);

    state.insert(valid_key, RenderValue::Bool(is_valid));
    state.insert(error_key, RenderValue::Str(error_msg));
}

// ─── Text Input DOM Overlay ──────────────────────────────────────────────────

const HIDDEN_INPUT_ID: &str = "__naze_hidden_input";

/// Create a hidden HTML input element for capturing keyboard input.
fn create_hidden_input(_canvas_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    // Check if already exists
    if document.get_element_by_id(HIDDEN_INPUT_ID).is_some() {
        return Ok(());
    }

    // Create hidden input
    let input = document.create_element("input")?;
    let input: web_sys::HtmlInputElement = input.dyn_into()?;
    input.set_id(HIDDEN_INPUT_ID);
    input.set_type("text");

    // Style to be invisible but focusable
    input.style().set_property("position", "absolute")?;
    input.style().set_property("left", "-9999px")?;
    input.style().set_property("top", "0")?;
    input.style().set_property("opacity", "0")?;
    input.style().set_property("pointer-events", "none")?;

    // Append to body
    document.body().ok_or("no body")?.append_child(&input)?;

    // Set up input event listener
    let input_cb = Closure::<dyn Fn(web_sys::Event)>::new(move |_event: web_sys::Event| {
        // Get current value from hidden input
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(el) = document.get_element_by_id(HIDDEN_INPUT_ID) {
                    if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                        let value = input.value();
                        // Update state with new value and execute change handlers
                        let changed = APP.with(|cell| {
                            let mut borrow = cell.borrow_mut();
                            if let Some(app) = borrow.as_mut() {
                                if let Some(ref focused) = app.focused_input.clone() {
                                    // Update the bound state variable
                                    app.state_store.insert(
                                        focused.bind_var.clone(),
                                        RenderValue::Str(value.clone()),
                                    );
                                    // Run validation and update derived state
                                    run_validation(
                                        &mut app.state_store,
                                        &focused.bind_var,
                                        &value,
                                        focused.validate_prop.as_ref(),
                                        &focused.input_type,
                                    );
                                    // Execute change handlers
                                    for handler in &focused.change_handlers {
                                        execute_action(&handler.action, &mut app.state_store);
                                    }
                                    return true;
                                }
                            }
                            false
                        });
                        if changed {
                            schedule_render();
                        }
                    }
                }
            }
        }
    });

    input.add_event_listener_with_callback("input", input_cb.as_ref().unchecked_ref())?;
    input_cb.forget();

    // Set up blur event listener to clear focus state and stop caret timer
    let blur_cb = Closure::<dyn Fn(web_sys::Event)>::new(move |_event: web_sys::Event| {
        APP.with(|cell| {
            let mut borrow = cell.borrow_mut();
            if let Some(app) = borrow.as_mut() {
                app.focused_input = None;
                // Stop caret blink timer
                if let Some(id) = app.caret_interval_id.take() {
                    if let Some(window) = web_sys::window() {
                        window.clear_interval_with_handle(id);
                    }
                }
                app.caret_visible = true; // Reset to visible state
            }
        });
        schedule_render();
    });

    // Get input again for blur listener
    if let Some(el) = document.get_element_by_id(HIDDEN_INPUT_ID) {
        el.add_event_listener_with_callback("blur", blur_cb.as_ref().unchecked_ref())?;
    }
    blur_cb.forget();

    Ok(())
}

/// Blur the hidden input element.
fn blur_hidden_input() {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(el) = document.get_element_by_id(HIDDEN_INPUT_ID) {
                if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                    let _ = input.blur();
                }
            }
        }
    }
}

/// Focus an input element, setting up the hidden input with its current value.
fn focus_input(bind_var: &str, node_id: &str, current_value: &str, input_type: &str, change_handlers: Vec<naze_ir::IrEventHandler>, validate_prop: Option<RenderValue>) {
    // Update focus state and start caret blink timer
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if let Some(app) = borrow.as_mut() {
            // Stop existing timer if any
            if let Some(id) = app.caret_interval_id.take() {
                if let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(id);
                }
            }

            app.focused_input = Some(FocusedInput {
                bind_var: bind_var.to_string(),
                node_id: node_id.to_string(),
                input_type: input_type.to_string(),
                change_handlers,
                validate_prop,
            });
            app.caret_visible = true; // Start with caret visible
        }
    });

    // Start caret blink timer (toggle every 500ms)
    if let Some(window) = web_sys::window() {
        let toggle_caret = Closure::<dyn Fn()>::new(|| {
            APP.with(|cell| {
                let mut borrow = cell.borrow_mut();
                if let Some(app) = borrow.as_mut() {
                    app.caret_visible = !app.caret_visible;
                }
            });
            schedule_render();
        });

        if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
            toggle_caret.as_ref().unchecked_ref(),
            500, // 500ms interval
        ) {
            APP.with(|cell| {
                let mut borrow = cell.borrow_mut();
                if let Some(app) = borrow.as_mut() {
                    app.caret_interval_id = Some(id);
                }
            });
        }
        toggle_caret.forget();
    }

    // Focus the hidden input and set its value and type
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(el) = document.get_element_by_id(HIDDEN_INPUT_ID) {
                if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                    // Set the input type (text, password, email, number)
                    input.set_type(input_type);
                    input.set_value(current_value);
                    let _ = input.focus();
                }
            }
        }
    }
}

/// Find an input node at the given point. Returns (bind_var, node_id, current_value, input_type, change_handlers, validate_prop) if found.
fn find_input_at_point(nodes: &[PositionedNode], x: f32, y: f32, state: &HashMap<String, RenderValue>) -> Option<(String, String, String, String, Vec<naze_ir::IrEventHandler>, Option<RenderValue>)> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first
        if let Some(result) = find_input_at_point(&node.children, x, y, state) {
            return Some(result);
        }
        // Check if this is an input
        if node.kind == "input" {
            if let Some(RenderValue::Bind(bind_var)) = node.props.get("bind") {
                let node_id = format!("input_{}_{}", node.x as i32, node.y as i32);
                let current_value = match state.get(bind_var) {
                    Some(RenderValue::Str(s)) => s.clone(),
                    _ => String::new(),
                };
                let input_type = naze_renderer::get_str_prop(&node.props, "type", "text");
                // Extract change handlers
                let change_handlers: Vec<_> = node.handlers.iter()
                    .filter(|h| h.event == "change")
                    .cloned()
                    .collect();
                // Extract validate prop
                let validate_prop = node.props.get("validate").cloned();
                return Some((bind_var.clone(), node_id, current_value, input_type, change_handlers, validate_prop));
            }
        }
    }
    None
}

/// Find a select node at the given point. Returns (bind_var, select_id) if found.
fn find_select_at_point(nodes: &[PositionedNode], x: f32, y: f32) -> Option<(String, String)> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first (but not options which are inside the select)
        let child_result = find_select_at_point(&node.children, x, y);
        if child_result.is_some() {
            return child_result;
        }
        // Check if this is a select
        if node.kind == "select" {
            if let Some(RenderValue::Bind(bind_var)) = node.props.get("bind") {
                let select_id = format!("select_{}_{}", node.x as i32, node.y as i32);
                return Some((bind_var.clone(), select_id));
            }
        }
    }
    None
}

/// Find an option in an open select's dropdown at the given point.
/// Returns (bind_var, value) if found.
/// Find an option in an open select's dropdown. Returns (bind_var, value, change_handlers) if found.
fn find_option_at_point(nodes: &[PositionedNode], x: f32, y: f32, open_select_id: &str) -> Option<(String, String, Vec<naze_ir::IrEventHandler>)> {
    for node in nodes.iter().rev() {
        // Recurse into children
        if let Some(result) = find_option_at_point(&node.children, x, y, open_select_id) {
            return Some(result);
        }
        // Check if this is the open select
        if node.kind == "select" {
            let select_id = format!("select_{}_{}", node.x as i32, node.y as i32);
            if select_id == open_select_id {
                // Get bind var
                let bind_var = match node.props.get("bind") {
                    Some(RenderValue::Bind(var)) => var.clone(),
                    _ => continue,
                };
                // Get change handlers from the select element
                let change_handlers: Vec<_> = node.handlers.iter()
                    .filter(|h| h.event == "change")
                    .cloned()
                    .collect();
                // Check if point is in the dropdown area (below the select box)
                let dropdown_y = node.y + node.height;
                let option_height = 36.0_f32;
                for (i, child) in node.children.iter().enumerate() {
                    if child.kind == "option" {
                        let opt_y = dropdown_y + (i as f32 * option_height);
                        if x >= node.x && x <= node.x + node.width &&
                           y >= opt_y && y <= opt_y + option_height {
                            let label = naze_renderer::get_text_content(&child.props);
                            let value = naze_renderer::get_str_prop(&child.props, "value", &label);
                            return Some((bind_var, value, change_handlers));
                        }
                    }
                }
            }
        }
    }
    None
}

// ─── Drag & Drop ─────────────────────────────────────────────────────────────

/// Information about a draggable element at a point.
struct DraggableInfo {
    node_id: String,
    bounds: (f32, f32, f32, f32), // x, y, width, height
    color: String,
    drag_data: RenderValue,
    drag_start_handlers: Vec<naze_ir::IrEventHandler>,
}

/// Information about a drop target at a point.
struct DropTargetInfo {
    node_id: String,
    bounds: (f32, f32, f32, f32), // x, y, width, height
    drag_over_handlers: Vec<naze_ir::IrEventHandler>,
    drop_handlers: Vec<naze_ir::IrEventHandler>,
}

/// Handle mousedown event. Starts drag if on draggable element, otherwise does nothing.
fn handle_mousedown(x: f32, y: f32) -> bool {
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

        // Find draggable element at point
        if let Some(info) = find_draggable_at_point(&layout.root, x, y) {
            // Start drag operation
            app.drag_state = Some(DragState {
                source_node_id: info.node_id,
                source_bounds: info.bounds,
                source_color: info.color,
                drag_data: info.drag_data,
                start_x: x,
                start_y: y,
                current_x: x,
                current_y: y,
                over_target_id: None,
            });

            // Execute drag-start handlers
            for handler in &info.drag_start_handlers {
                execute_action(&handler.action, &mut app.state_store);
            }

            return true; // Needs render for visual feedback
        }

        false
    })
}

/// Handle mousemove event. Updates drag position, hover state, or cursor.
fn handle_mousemove(x: f32, y: f32) -> bool {
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return false,
        };

        // Check if we're dragging
        if let Some(ref mut drag) = app.drag_state {
            // Update position
            drag.current_x = x;
            drag.current_y = y;

            // Find drop target at current position
            let target = find_drop_target_at_point(&layout.root, x, y);
            let new_target_id = target.as_ref().map(|t| t.node_id.clone());

            // If we entered a new target, fire drag-over handlers
            if new_target_id != drag.over_target_id {
                if let Some(info) = target {
                    for handler in &info.drag_over_handlers {
                        execute_action(&handler.action, &mut app.state_store);
                    }
                }
                drag.over_target_id = new_target_id;
            }

            // Set cursor to grabbing
            set_cursor("grabbing");
            return true; // Needs render to update ghost position
        }

        // Track hover state - find deepest element with hover handlers
        let mut needs_render = false;
        let hover_info = find_hover_element(&layout.root, x, y);
        let new_hover_id = hover_info.as_ref().map(|(id, _)| id.clone());

        if new_hover_id != app.hovered_element_id {
            // Fire hover handlers when entering a new element
            if let Some((_, handlers)) = hover_info {
                for handler in &handlers {
                    if execute_action(&handler.action, &mut app.state_store) {
                        needs_render = true;
                    }
                }
            }
            app.hovered_element_id = new_hover_id;
        }

        // Update cursor based on what's under the mouse
        let has_draggable = find_draggable_at_point(&layout.root, x, y).is_some();
        let has_clickable = hit_test_any_handler(&layout.root, x, y, "click");

        if has_draggable {
            set_cursor("grab");
        } else if has_clickable {
            set_cursor("pointer");
        } else {
            set_cursor("default");
        }

        needs_render
    })
}

/// Handle mouseup event. Completes drop or triggers click.
fn handle_mouseup(x: f32, y: f32) -> bool {
    let was_dragging = APP.with(|cell| {
        let borrow = cell.borrow();
        if let Some(app) = borrow.as_ref() {
            return app.drag_state.is_some();
        }
        false
    });

    if was_dragging {
        // Complete drag operation
        return APP.with(|cell| {
            let mut borrow = cell.borrow_mut();
            let app = match borrow.as_mut() {
                Some(a) => a,
                None => return false,
            };
            let layout = match &app.layout {
                Some(l) => l.clone(),
                None => return false,
            };

            // Take drag state to end the drag
            let drag = app.drag_state.take();
            if let Some(_drag) = drag {
                // Find drop target at release point
                if let Some(info) = find_drop_target_at_point(&layout.root, x, y) {
                    // Execute drop handlers
                    for handler in &info.drop_handlers {
                        execute_action(&handler.action, &mut app.state_store);
                    }
                }
            }

            set_cursor("default");
            true // Needs render to clear ghost
        });
    }

    // Not dragging - handle as click
    handle_click(x, y)
}

/// Find a draggable element at the given point.
fn find_draggable_at_point(nodes: &[PositionedNode], x: f32, y: f32) -> Option<DraggableInfo> {
    // Walk depth-first; deepest match wins
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first
        if let Some(result) = find_draggable_at_point(&node.children, x, y) {
            return Some(result);
        }
        // Check if this node is draggable
        let is_draggable = match node.props.get("draggable") {
            Some(RenderValue::Bool(true)) => true,
            _ => false,
        };
        if is_draggable {
            let node_id = format!("drag_{}_{}", node.x as i32, node.y as i32);
            let color = naze_renderer::get_color_prop(&node.props, "color", "#888888");
            let drag_data = node.props.get("drag-data")
                .cloned()
                .unwrap_or(RenderValue::Str(String::new()));
            let drag_start_handlers: Vec<_> = node.handlers.iter()
                .filter(|h| h.event == "drag-start")
                .cloned()
                .collect();
            return Some(DraggableInfo {
                node_id,
                bounds: (node.x, node.y, node.width, node.height),
                color,
                drag_data,
                drag_start_handlers,
            });
        }
    }
    None
}

/// Find a drop target at the given point.
fn find_drop_target_at_point(nodes: &[PositionedNode], x: f32, y: f32) -> Option<DropTargetInfo> {
    // Walk depth-first; deepest match wins
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first
        if let Some(result) = find_drop_target_at_point(&node.children, x, y) {
            return Some(result);
        }
        // Check if this node is a drop target
        let is_drop_target = match node.props.get("drop-target") {
            Some(RenderValue::Bool(true)) => true,
            _ => false,
        };
        if is_drop_target {
            let node_id = format!("drop_{}_{}", node.x as i32, node.y as i32);
            let drag_over_handlers: Vec<_> = node.handlers.iter()
                .filter(|h| h.event == "drag-over")
                .cloned()
                .collect();
            let drop_handlers: Vec<_> = node.handlers.iter()
                .filter(|h| h.event == "drop")
                .cloned()
                .collect();
            return Some(DropTargetInfo {
                node_id,
                bounds: (node.x, node.y, node.width, node.height),
                drag_over_handlers,
                drop_handlers,
            });
        }
    }
    None
}

/// Set the cursor style on the canvas.
fn set_cursor(cursor: &str) {
    APP.with(|cell| {
        let borrow = cell.borrow();
        if let Some(app) = borrow.as_ref() {
            let _ = app.renderer.canvas_element().style().set_property("cursor", cursor);
        }
    });
}

// ─── Scroll Handling ─────────────────────────────────────────────────────────

/// Handle wheel event for scrolling.
fn handle_wheel(x: f32, y: f32, delta_x: f32, delta_y: f32) -> bool {
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return false,
        };

        // Find scroll container at point
        if let Some((scroll_id, scroll_info, bounds)) = find_scroll_at_point(&layout.root, x, y) {
            let (_, _, container_w, container_h) = bounds;

            // Calculate max scroll values
            let max_scroll_x = (scroll_info.content_width - container_w).max(0.0);
            let max_scroll_y = (scroll_info.content_height - container_h).max(0.0);

            // Get or create scroll state
            let state = app.scroll_states.entry(scroll_id.clone()).or_default();

            let mut changed = false;

            // Apply scroll deltas (with scroll multiplier for smoother feel)
            let scroll_speed = 1.0;
            if scroll_info.overflow_y && max_scroll_y > 0.0 {
                let new_scroll_y = (state.scroll_y + delta_y * scroll_speed)
                    .max(0.0)
                    .min(max_scroll_y);
                if (new_scroll_y - state.scroll_y).abs() > 0.001 {
                    state.scroll_y = new_scroll_y;
                    changed = true;
                }
            }
            if scroll_info.overflow_x && max_scroll_x > 0.0 {
                let new_scroll_x = (state.scroll_x + delta_x * scroll_speed)
                    .max(0.0)
                    .min(max_scroll_x);
                if (new_scroll_x - state.scroll_x).abs() > 0.001 {
                    state.scroll_x = new_scroll_x;
                    changed = true;
                }
            }

            // Fire scroll handlers on the scroll container
            if changed {
                if let Some(handlers) = find_scroll_handlers(&layout.root, &scroll_id) {
                    for handler in &handlers {
                        execute_action(&handler.action, &mut app.state_store);
                    }
                }
            }

            return changed;
        }

        false
    })
}

/// Find scroll handlers for a scroll container by its ID.
fn find_scroll_handlers(nodes: &[PositionedNode], scroll_id: &str) -> Option<Vec<naze_ir::IrEventHandler>> {
    for node in nodes {
        // Check if this is the scroll container
        if node.kind == "scroll" {
            let this_id = format!("scroll_{}_{}", node.x as i32, node.y as i32);
            if this_id == scroll_id {
                let handlers: Vec<_> = node.handlers.iter()
                    .filter(|h| h.event == "scroll")
                    .cloned()
                    .collect();
                if !handlers.is_empty() {
                    return Some(handlers);
                }
                return None;
            }
        }
        // Recurse into children
        if let Some(handlers) = find_scroll_handlers(&node.children, scroll_id) {
            return Some(handlers);
        }
    }
    None
}

/// Find a scroll container at the given point.
/// Returns (scroll_id, scroll_info, (x, y, width, height)) if found.
fn find_scroll_at_point(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
) -> Option<(String, ScrollInfo, (f32, f32, f32, f32))> {
    // Walk depth-first; deepest match wins
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first
        if let Some(result) = find_scroll_at_point(&node.children, x, y) {
            return Some(result);
        }
        // Check if this is a scroll container
        if node.kind == "scroll" {
            if let Some(ref info) = node.scroll_info {
                let scroll_id = format!("scroll_{}_{}", node.x as i32, node.y as i32);
                return Some((scroll_id, info.clone(), (node.x, node.y, node.width, node.height)));
            }
        }
    }
    None
}

/// Scroll to bring an element with the given ID into view.
/// The element_id should match an element's `id` prop.
fn scroll_to_element(element_id: &str) {
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return,
        };

        // Find the element by ID and its containing scroll container
        if let Some((element_y, scroll_id, scroll_info, container_y)) =
            find_element_and_scroll_container(&layout.root, element_id, None) {
            // Calculate the scroll offset to bring element into view
            let relative_y = element_y - container_y;
            let container_h = scroll_info.content_height - scroll_info.content_height; // This should be viewport height

            // Get or create scroll state
            let state = app.scroll_states.entry(scroll_id).or_default();

            // Scroll so the element is at the top of the container
            let max_scroll = (scroll_info.content_height - container_h).max(0.0);
            state.scroll_y = relative_y.min(max_scroll).max(0.0);
        }
    });
}

/// Find an element by ID and its containing scroll container.
/// Returns (element_y, scroll_id, scroll_info, container_y) if found.
fn find_element_and_scroll_container(
    nodes: &[PositionedNode],
    element_id: &str,
    current_scroll: Option<(String, ScrollInfo, f32)>,
) -> Option<(f32, String, ScrollInfo, f32)> {
    for node in nodes {
        // Update current scroll container if this is a scroll node
        let scroll_context = if node.kind == "scroll" {
            if let Some(ref info) = node.scroll_info {
                let scroll_id = format!("scroll_{}_{}", node.x as i32, node.y as i32);
                Some((scroll_id, info.clone(), node.y))
            } else {
                current_scroll.clone()
            }
        } else {
            current_scroll.clone()
        };

        // Check if this node has the target ID
        if let Some(RenderValue::Str(id)) = node.props.get("id") {
            if id == element_id {
                if let Some((scroll_id, scroll_info, container_y)) = scroll_context {
                    return Some((node.y, scroll_id, scroll_info, container_y));
                }
            }
        }

        // Recurse into children
        if let Some(result) = find_element_and_scroll_container(&node.children, element_id, scroll_context.clone()) {
            return Some(result);
        }
    }
    None
}

// ─── Keyboard Handling ───────────────────────────────────────────────────────

/// Handle keydown event. Returns true if state changed (needs re-render).
fn handle_keydown(key: &str, shift: bool) -> bool {
    APP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return false,
        };

        // Handle Tab key for focus navigation
        if key == "Tab" {
            let focusable = collect_focusable_elements(&layout.root);
            if focusable.is_empty() {
                return false;
            }

            // Find current focus index
            let current_idx = app.focused_element_id.as_ref().and_then(|id| {
                focusable.iter().position(|(fid, _, _)| fid == id)
            });

            // Calculate next focus index
            let next_idx = if shift {
                // Shift+Tab: go backwards
                match current_idx {
                    Some(0) => focusable.len() - 1,
                    Some(idx) => idx - 1,
                    None => focusable.len() - 1,
                }
            } else {
                // Tab: go forwards
                match current_idx {
                    Some(idx) if idx + 1 < focusable.len() => idx + 1,
                    _ => 0,
                }
            };

            // Update focus
            let (new_id, node_kind, _) = &focusable[next_idx];
            app.focused_element_id = Some(new_id.clone());

            // If it's an input, also set focused_input
            if node_kind == "input" {
                // Find the input node and focus it
                if let Some((bind_var, node_id, value, input_type, handlers, validate)) =
                    find_input_by_id(&layout.root, new_id, &app.state_store) {
                    // Drop borrow before calling focus_input
                    let bind_var = bind_var.clone();
                    let node_id = node_id.clone();
                    let value = value.clone();
                    let input_type = input_type.clone();
                    drop(borrow);
                    focus_input(&bind_var, &node_id, &value, &input_type, handlers, validate);
                    return true;
                }
            } else {
                // Clear text input focus if focusing non-input
                app.focused_input = None;
                blur_hidden_input();
            }

            return true;
        }

        // Handle Enter key - activate focused element
        if key == "Enter" {
            if let Some(ref focused_id) = app.focused_element_id.clone() {
                // Find the focused node and execute its click handlers
                if let Some(handlers) = find_handlers_by_element_id(&layout.root, focused_id, "click") {
                    let mut changed = false;
                    for handler in &handlers {
                        if execute_action(&handler.action, &mut app.state_store) {
                            changed = true;
                        }
                    }
                    return changed;
                }
            }
        }

        // Handle Escape key - blur focused element
        if key == "Escape" {
            if app.focused_element_id.is_some() || app.focused_input.is_some() {
                app.focused_element_id = None;
                app.focused_input = None;
                blur_hidden_input();
                return true;
            }
        }

        // Execute keypress handlers on focused element
        if let Some(ref focused_id) = app.focused_element_id.clone() {
            if let Some(handlers) = find_handlers_by_element_id(&layout.root, focused_id, "keypress") {
                let mut changed = false;
                for handler in &handlers {
                    if execute_action(&handler.action, &mut app.state_store) {
                        changed = true;
                    }
                }
                return changed;
            }
        }

        false
    })
}

/// Collect all focusable elements in tab order.
/// Returns vec of (element_id, node_kind, tab_index).
fn collect_focusable_elements(nodes: &[PositionedNode]) -> Vec<(String, String, i32)> {
    let mut focusable = Vec::new();
    collect_focusable_recursive(nodes, &mut focusable);
    // Sort by tab-index (elements with explicit tab-index come first, then DOM order)
    focusable.sort_by(|a, b| {
        match (a.2, b.2) {
            (ta, tb) if ta > 0 && tb > 0 => ta.cmp(&tb),
            (ta, _) if ta > 0 => std::cmp::Ordering::Less,
            (_, tb) if tb > 0 => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal, // Preserve DOM order
        }
    });
    focusable
}

fn collect_focusable_recursive(nodes: &[PositionedNode], out: &mut Vec<(String, String, i32)>) {
    for node in nodes {
        // Check if this node is focusable
        let is_focusable = matches!(
            node.kind.as_str(),
            "input" | "checkbox" | "radio" | "select" | "link"
        ) || node.handlers.iter().any(|h| h.event == "click")
          || node.props.get("tab-index").is_some();

        if is_focusable {
            let element_id = format!("focus_{}_{}_{}", node.kind, node.x as i32, node.y as i32);
            let tab_index = match node.props.get("tab-index") {
                Some(RenderValue::Num(n, _)) => *n as i32,
                _ => 0,
            };
            out.push((element_id, node.kind.clone(), tab_index));
        }

        // Recurse into children
        collect_focusable_recursive(&node.children, out);
    }
}

/// Find an input node by its focus element ID.
fn find_input_by_id(
    nodes: &[PositionedNode],
    element_id: &str,
    state: &HashMap<String, RenderValue>,
) -> Option<(String, String, String, String, Vec<naze_ir::IrEventHandler>, Option<RenderValue>)> {
    for node in nodes {
        let this_id = format!("focus_{}_{}_{}", node.kind, node.x as i32, node.y as i32);
        if node.kind == "input" && this_id == element_id {
            if let Some(RenderValue::Bind(bind_var)) = node.props.get("bind") {
                let node_id = format!("input_{}_{}", node.x as i32, node.y as i32);
                let value = match state.get(bind_var) {
                    Some(RenderValue::Str(s)) => s.clone(),
                    _ => String::new(),
                };
                let input_type = naze_renderer::get_str_prop(&node.props, "type", "text");
                let handlers: Vec<_> = node.handlers.iter()
                    .filter(|h| h.event == "change")
                    .cloned()
                    .collect();
                let validate = node.props.get("validate").cloned();
                return Some((bind_var.clone(), node_id, value, input_type, handlers, validate));
            }
        }
        // Recurse
        if let Some(result) = find_input_by_id(&node.children, element_id, state) {
            return Some(result);
        }
    }
    None
}

/// Find handlers for an event on an element by its focus ID.
fn find_handlers_by_element_id(
    nodes: &[PositionedNode],
    element_id: &str,
    event: &str,
) -> Option<Vec<naze_ir::IrEventHandler>> {
    for node in nodes {
        let this_id = format!("focus_{}_{}_{}", node.kind, node.x as i32, node.y as i32);
        if this_id == element_id {
            let handlers: Vec<_> = node.handlers.iter()
                .filter(|h| h.event == event)
                .cloned()
                .collect();
            if !handlers.is_empty() {
                return Some(handlers);
            }
            // For form elements, clicking also triggers change
            if event == "click" && matches!(node.kind.as_str(), "checkbox" | "radio") {
                let change_handlers: Vec<_> = node.handlers.iter()
                    .filter(|h| h.event == "change")
                    .cloned()
                    .collect();
                if !change_handlers.is_empty() {
                    return Some(change_handlers);
                }
            }
            return None;
        }
        // Recurse
        if let Some(result) = find_handlers_by_element_id(&node.children, element_id, event) {
            return Some(result);
        }
    }
    None
}

// ─── Screen Reader Accessibility DOM ─────────────────────────────────────────

const A11Y_CONTAINER_ID: &str = "__naze_a11y";

/// Create a hidden container for screen reader content.
/// This DOM mirrors the canvas content with ARIA attributes.
fn create_a11y_container() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    // Check if already exists
    if document.get_element_by_id(A11Y_CONTAINER_ID).is_some() {
        return Ok(());
    }

    // Create container
    let container = document.create_element("div")?;
    container.set_id(A11Y_CONTAINER_ID);

    // Style to be visually hidden but accessible to screen readers
    // (sr-only pattern)
    let style = container
        .dyn_ref::<web_sys::HtmlElement>()
        .ok_or("not an HtmlElement")?
        .style();
    style.set_property("position", "absolute")?;
    style.set_property("width", "1px")?;
    style.set_property("height", "1px")?;
    style.set_property("padding", "0")?;
    style.set_property("margin", "-1px")?;
    style.set_property("overflow", "hidden")?;
    style.set_property("clip", "rect(0, 0, 0, 0)")?;
    style.set_property("white-space", "nowrap")?;
    style.set_property("border", "0")?;

    // Set ARIA attributes on container
    container.set_attribute("aria-hidden", "false")?;
    container.set_attribute("role", "application")?;

    // Append to body
    document.body().ok_or("no body")?.append_child(&container)?;

    Ok(())
}

/// Update the screen reader DOM to mirror the current layout.
fn update_a11y_dom(layout: &LayoutTree) {
    let result = (|| -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;

        let container = document
            .get_element_by_id(A11Y_CONTAINER_ID)
            .ok_or("a11y container not found")?;

        // Clear existing content
        container.set_inner_html("");

        // Build accessible DOM from layout tree
        build_a11y_nodes(&document, &container, &layout.root)?;

        Ok(())
    })();

    if let Err(e) = result {
        web_sys::console::warn_1(&format!("a11y update failed: {:?}", e).into());
    }
}

/// Recursively build accessible DOM elements from positioned nodes.
fn build_a11y_nodes(
    document: &web_sys::Document,
    parent: &web_sys::Element,
    nodes: &[PositionedNode],
) -> Result<(), JsValue> {
    for node in nodes {
        // Determine ARIA role from node kind and props
        let role = get_a11y_role(node);
        let label = get_a11y_label(node);

        // Skip nodes that don't contribute to accessibility
        if role.is_none() && label.is_none() && node.kind != "text" && node.kind != "heading" {
            // Just recurse into children
            build_a11y_nodes(document, parent, &node.children)?;
            continue;
        }

        // Create appropriate HTML element
        let el = create_a11y_element(document, node, role.as_deref())?;

        // Set accessible label
        if let Some(lbl) = label {
            el.set_attribute("aria-label", &lbl)?;
        }

        // Set text content for text/heading nodes
        if matches!(node.kind.as_str(), "text" | "heading" | "link") {
            let text = naze_renderer::get_text_content(&node.props);
            el.set_text_content(Some(&text));
        }

        // Handle form elements
        match node.kind.as_str() {
            "input" => {
                let input_type = naze_renderer::get_str_prop(&node.props, "type", "text");
                el.set_attribute("type", &input_type)?;
                if let Some(placeholder) = node.props.get("placeholder") {
                    if let RenderValue::Str(s) = placeholder {
                        el.set_attribute("placeholder", s)?;
                    }
                }
            }
            "checkbox" => {
                el.set_attribute("type", "checkbox")?;
                // Set checked state
                if let Some(RenderValue::Bind(bind)) = node.props.get("bind") {
                    // We'd need state here - for now just set the structure
                    el.set_attribute("data-bind", bind)?;
                }
            }
            "radio" => {
                el.set_attribute("type", "radio")?;
                if let Some(RenderValue::Str(name)) = node.props.get("name") {
                    el.set_attribute("name", name)?;
                }
            }
            "select" => {
                // Add options
                for child in &node.children {
                    if child.kind == "option" {
                        let opt = document.create_element("option")?;
                        let text = naze_renderer::get_text_content(&child.props);
                        opt.set_text_content(Some(&text));
                        if let Some(RenderValue::Str(val)) = child.props.get("value") {
                            opt.set_attribute("value", val)?;
                        }
                        el.append_child(&opt)?;
                    }
                }
            }
            _ => {}
        }

        // Handle interactive elements
        if node.handlers.iter().any(|h| h.event == "click") {
            el.set_attribute("tabindex", "0")?;
        }

        // Append to parent
        parent.append_child(&el)?;

        // Recurse into children (except for handled cases like select)
        if !matches!(node.kind.as_str(), "select") {
            build_a11y_nodes(document, &el, &node.children)?;
        }
    }

    Ok(())
}

/// Get ARIA role for a node based on its kind and role prop.
fn get_a11y_role(node: &PositionedNode) -> Option<String> {
    // First check for explicit role prop
    if let Some(RenderValue::Str(role)) = node.props.get("role") {
        return Some(role.clone());
    }

    // Infer role from element kind
    match node.kind.as_str() {
        "heading" => Some("heading".to_string()),
        "link" => Some("link".to_string()),
        "input" => None, // HTML input has implicit role
        "checkbox" => Some("checkbox".to_string()),
        "radio" => Some("radio".to_string()),
        "select" => Some("listbox".to_string()),
        "image" => Some("img".to_string()),
        "row" | "column" if node.handlers.iter().any(|h| h.event == "click") => {
            Some("button".to_string())
        }
        "scroll" => Some("region".to_string()),
        _ => None,
    }
}

/// Get accessible label for a node.
fn get_a11y_label(node: &PositionedNode) -> Option<String> {
    // Check for explicit label prop
    if let Some(RenderValue::Str(label)) = node.props.get("label") {
        return Some(label.clone());
    }

    // For images, check alt prop
    if node.kind == "image" {
        if let Some(RenderValue::Str(alt)) = node.props.get("alt") {
            return Some(alt.clone());
        }
    }

    // For form elements with labels
    if matches!(node.kind.as_str(), "checkbox" | "radio") {
        let text = naze_renderer::get_text_content(&node.props);
        if !text.is_empty() {
            return Some(text);
        }
    }

    None
}

/// Create the appropriate HTML element for accessibility.
fn create_a11y_element(
    document: &web_sys::Document,
    node: &PositionedNode,
    role: Option<&str>,
) -> Result<web_sys::Element, JsValue> {
    let tag = match node.kind.as_str() {
        "heading" => {
            // Use h1-h6 based on font size or level prop
            let level = node.props.get("level")
                .and_then(|v| if let RenderValue::Num(n, _) = v { Some(*n as i32) } else { None })
                .unwrap_or(1)
                .clamp(1, 6);
            format!("h{}", level)
        }
        "text" => "span".to_string(),
        "link" => "a".to_string(),
        "input" => "input".to_string(),
        "checkbox" | "radio" => "input".to_string(),
        "select" => "select".to_string(),
        "image" => "img".to_string(),
        _ => "div".to_string(),
    };

    let el = document.create_element(&tag)?;

    // Set role if provided and not implicit
    if let Some(r) = role {
        // Don't set role if it matches the implicit role
        let needs_explicit = !matches!(
            (tag.as_str(), r),
            ("a", "link") | ("select", "listbox") | ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", "heading")
        );
        if needs_explicit {
            el.set_attribute("role", r)?;
        }
    }

    Ok(el)
}

// ─── Data Fetching ───────────────────────────────────────────────────────────

/// Fetch data from a URL and populate the state variables.
/// Creates {name}.loading, {name}.error, {name}.data state variables.
fn fetch_data(name: &str, url: &str) {
    use wasm_bindgen_futures::spawn_local;

    let name = name.to_string();
    let url = url.to_string();

    spawn_local(async move {
        let result = do_fetch(&url).await;

        APP.with(|cell| {
            let mut borrow = cell.borrow_mut();
            if let Some(app) = borrow.as_mut() {
                // Set loading to false
                app.state_store.insert(
                    format!("{}.loading", name),
                    RenderValue::Bool(false),
                );

                match result {
                    Ok(data) => {
                        // Success: populate data, clear error
                        app.state_store.insert(
                            format!("{}.data", name),
                            data,
                        );
                        app.state_store.insert(
                            format!("{}.error", name),
                            RenderValue::Str(String::new()),
                        );
                    }
                    Err(err) => {
                        // Error: set error message, keep data empty
                        app.state_store.insert(
                            format!("{}.error", name),
                            RenderValue::Str(err),
                        );
                    }
                }
            }
        });

        // Trigger re-render
        schedule_render();
    });
}

/// Perform the actual HTTP fetch and parse JSON response.
async fn do_fetch(url: &str) -> Result<RenderValue, String> {
    let window = web_sys::window().ok_or("no window")?;

    // Create request
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(web_sys::RequestMode::Cors);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("failed to create request: {:?}", e))?;

    // Add accept header for JSON
    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|e| format!("failed to set header: {:?}", e))?;

    // Fetch
    let resp_promise = window.fetch_with_request(&request);
    let resp_value = wasm_bindgen_futures::JsFuture::from(resp_promise)
        .await
        .map_err(|e| format!("fetch failed: {:?}", e))?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "response is not a Response")?;

    // Check status
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    // Get JSON
    let json_promise = resp
        .json()
        .map_err(|e| format!("failed to get JSON: {:?}", e))?;

    let json_value = wasm_bindgen_futures::JsFuture::from(json_promise)
        .await
        .map_err(|e| format!("JSON parse failed: {:?}", e))?;

    // Convert JS value to RenderValue
    Ok(js_to_render_value(&json_value))
}

/// Convert a JS value to a RenderValue.
fn js_to_render_value(value: &JsValue) -> RenderValue {
    if value.is_null() || value.is_undefined() {
        return RenderValue::Str(String::new());
    }

    if let Some(b) = value.as_bool() {
        return RenderValue::Bool(b);
    }

    if let Some(n) = value.as_f64() {
        return RenderValue::Num(n, None);
    }

    if let Some(s) = value.as_string() {
        return RenderValue::Str(s);
    }

    // Check if it's an array
    if js_sys::Array::is_array(value) {
        let arr: js_sys::Array = value.clone().unchecked_into();
        let items: Vec<RenderValue> = arr
            .iter()
            .map(|item| js_to_render_value(&item))
            .collect();
        return RenderValue::List(items);
    }

    // Check if it's an object
    if value.is_object() {
        let obj: js_sys::Object = value.clone().unchecked_into();
        let entries = js_sys::Object::entries(&obj);
        let mut pairs = Vec::new();
        for entry in entries.iter() {
            let entry_arr: js_sys::Array = entry.unchecked_into();
            if let Some(key) = entry_arr.get(0).as_string() {
                let val = js_to_render_value(&entry_arr.get(1));
                pairs.push((key, val));
            }
        }
        return RenderValue::Object(pairs);
    }

    // Fallback: convert to string
    RenderValue::Str(format!("{:?}", value))
}
