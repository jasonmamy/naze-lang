use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use js_sys::RegExp;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use naze_ir::{
    ComputedDecl, IrAction, IrBinOp, IrEventHandler, IrExpression, IrPipelineStage, PageDef,
    ParamDecl, RenderNode, RenderTree, RenderValue, StorageDecl, TextPart, TimerDecl,
};
use naze_layout::{self, LayoutTree, PositionedNode, ScrollInfo};
use naze_renderer::{self, canvas::Renderer};

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Build a dotted state key like "name.loading" without format! overhead.
#[inline]
fn state_key(base: &str, suffix: &str) -> String {
    let mut s = String::with_capacity(base.len() + 1 + suffix.len());
    s.push_str(base);
    s.push('.');
    s.push_str(suffix);
    s
}

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
    source_node_id: String,              // ID of element being dragged
    source_bounds: (f32, f32, f32, f32), // (x, y, width, height) of source element
    source_color: String,                // Color of source element for ghost
    drag_data: RenderValue,              // Data attached to drag
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
#[derive(Clone, Copy, Debug)]
enum EasingFn {
    Linear,
    Ease,                            // cubic-bezier(0.25, 0.1, 0.25, 1.0)
    EaseIn,                          // cubic-bezier(0.42, 0, 1.0, 1.0)
    EaseOut,                         // cubic-bezier(0, 0, 0.58, 1.0)
    EaseInOut,                       // cubic-bezier(0.42, 0, 0.58, 1.0)
    CubicBezier(f64, f64, f64, f64), // custom cubic-bezier(x1, y1, x2, y2)
}

impl EasingFn {
    fn from_str(s: &str) -> Self {
        let trimmed = s.trim();
        let lower = trimmed.to_lowercase();
        // Parse cubic-bezier(x1, y1, x2, y2)
        if lower.starts_with("cubic-bezier(") && lower.ends_with(')') {
            let inner = &trimmed[13..trimmed.len() - 1];
            let params: Vec<f64> = inner
                .split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect();
            if params.len() == 4 {
                return EasingFn::CubicBezier(params[0], params[1], params[2], params[3]);
            }
        }
        match lower.as_str() {
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
            EasingFn::CubicBezier(x1, y1, x2, y2) => cubic_bezier(*x1, *y1, *x2, *y2, t),
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

/// Spring physics state for damped oscillation animations.
#[derive(Clone, Debug)]
struct SpringState {
    position: f64, // 0.0 = start, 1.0 = target
    velocity: f64,
}

impl SpringState {
    fn new() -> Self {
        SpringState {
            position: 0.0,
            velocity: 0.0,
        }
    }

    /// Advance the spring simulation by dt_ms milliseconds.
    /// Returns (clamped_position, settled).
    fn step(&mut self, dt_ms: f64, stiffness: f64, damping: f64) -> (f64, bool) {
        let dt = dt_ms / 1000.0; // Convert to seconds
        let force = -stiffness * (self.position - 1.0) - damping * self.velocity;
        self.velocity += force * dt;
        self.position += self.velocity * dt;
        let settled = (self.position - 1.0).abs() < 0.001 && self.velocity.abs() < 0.001;
        (self.position.clamp(-0.5, 2.0), settled)
    }
}

/// Animation driver — determines how progress is computed.
#[derive(Clone, Debug)]
enum AnimDriver {
    /// Fixed-duration with easing curve.
    Timed { duration_ms: f64, easing: EasingFn },
    /// Spring physics — runs until settled.
    Spring {
        stiffness: f64,
        damping: f64,
        state: SpringState,
        last_time: f64,
    },
    /// Multi-value keyframe sequence.
    Keyframe {
        values: Vec<AnimValue>,
        duration_ms: f64,
        easing: EasingFn,
    },
}

/// Spec for how an animation is driven.
#[derive(Clone, Debug)]
enum AnimDriverSpec {
    Timed { duration_ms: f64, easing: EasingFn },
    Spring { stiffness: f64, damping: f64 },
}

/// Parsed transition specification.
#[derive(Clone, Debug)]
struct TransitionSpec {
    property: String, // e.g., "color", "opacity", "background"
    driver: AnimDriverSpec,
}

impl TransitionSpec {
    /// Parse transition string.
    /// Formats:
    ///   "color 150ms ease"
    ///   "opacity 200ms cubic-bezier(0.4, 0, 0.2, 1)"
    ///   "color spring(300, 20)"
    fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        // Extract property name (first token)
        let first_space = s.find(' ')?;
        let property = s[..first_space].to_string();
        let rest = s[first_space..].trim();

        // Check for spring(stiffness, damping)
        if let Some(spring_start) = rest.find("spring(") {
            let after_spring = &rest[spring_start + 7..];
            if let Some(paren_end) = after_spring.find(')') {
                let params: Vec<f64> = after_spring[..paren_end]
                    .split(',')
                    .filter_map(|p| p.trim().parse().ok())
                    .collect();
                if params.len() == 2 {
                    return Some(TransitionSpec {
                        property,
                        driver: AnimDriverSpec::Spring {
                            stiffness: params[0],
                            damping: params[1],
                        },
                    });
                }
            }
        }

        // Timed animation: parse duration + easing
        let mut duration_ms = 200.0;
        let mut easing = EasingFn::Ease;

        // Reassemble rest to handle cubic-bezier(...) as a single token
        // Split on whitespace but rejoin cubic-bezier tokens
        let tokens = tokenize_transition(rest);
        for token in &tokens {
            if token.ends_with("ms") {
                if let Ok(d) = token.trim_end_matches("ms").parse::<f64>() {
                    duration_ms = d;
                }
            } else if token.ends_with('s') && !token.ends_with("ms") {
                if let Ok(d) = token.trim_end_matches('s').parse::<f64>() {
                    duration_ms = d * 1000.0;
                }
            } else {
                easing = EasingFn::from_str(token);
            }
        }

        Some(TransitionSpec {
            property,
            driver: AnimDriverSpec::Timed {
                duration_ms,
                easing,
            },
        })
    }
}

/// Tokenize a transition string, keeping `cubic-bezier(...)` as a single token.
fn tokenize_transition(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '(' {
            // Consume until matching ')'
            current.push(ch);
            chars.next();
            while let Some(&inner) = chars.peek() {
                current.push(inner);
                chars.next();
                if inner == ')' {
                    break;
                }
            }
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            chars.next();
        } else {
            current.push(ch);
            chars.next();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Parsed keyframe animation specification.
#[derive(Clone, Debug)]
struct KeyframeSpec {
    property: String,       // e.g., "scale", "opacity", "color"
    values: Vec<AnimValue>, // Keyframe stops (min 2)
    duration_ms: f64,
    easing: EasingFn,
}

impl KeyframeSpec {
    /// Parse: "scale [1, 1.2, 0.95, 1] 400ms ease-in-out"
    /// or:   "opacity [0, 1] 200ms ease"
    /// or:   "color [#ff0000, #00ff00, #0000ff] 1s linear"
    fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        // Extract property name (first token before space or '[')
        let prop_end = s.find(|c: char| c.is_whitespace() || c == '[')?;
        let property = s[..prop_end].trim().to_string();
        let rest = s[prop_end..].trim();

        // Extract values between [ and ]
        let bracket_start = rest.find('[')?;
        let bracket_end = rest.find(']')?;
        let values_str = &rest[bracket_start + 1..bracket_end];
        let values: Vec<AnimValue> = values_str
            .split(',')
            .filter_map(|v| {
                let v = v.trim();
                if v.starts_with('#') && v.len() == 7 {
                    u32::from_str_radix(&v[1..], 16).ok().map(AnimValue::Color)
                } else {
                    v.parse::<f64>().ok().map(AnimValue::Number)
                }
            })
            .collect();

        if values.len() < 2 {
            return None;
        }

        // Parse duration + easing from remainder after ']'
        let after_bracket = rest[bracket_end + 1..].trim();
        let mut duration_ms = 400.0; // Default
        let mut easing = EasingFn::Ease;

        let tokens = tokenize_transition(after_bracket);
        for token in &tokens {
            if token.ends_with("ms") {
                if let Ok(d) = token.trim_end_matches("ms").parse::<f64>() {
                    duration_ms = d;
                }
            } else if token.ends_with('s') && !token.ends_with("ms") {
                if let Ok(d) = token.trim_end_matches('s').parse::<f64>() {
                    duration_ms = d * 1000.0;
                }
            } else {
                easing = EasingFn::from_str(token);
            }
        }

        Some(KeyframeSpec {
            property,
            values,
            duration_ms,
            easing,
        })
    }
}

/// Split an animate prop value on commas that are outside brackets.
/// e.g., "scale [1, 1.2, 1] 400ms, opacity [0, 1] 200ms" → ["scale [1, 1.2, 1] 400ms", "opacity [0, 1] 200ms"]
fn split_animate_specs(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => depth -= 1,
            ',' if depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        result.push(&s[start..]);
    }
    result
}

/// An active animation for a specific element property.
#[derive(Clone, Debug)]
struct ActiveAnimation {
    node_key: String,       // Unique key for the node (based on position/id)
    property: String,       // Property being animated
    start_value: AnimValue, // Starting value
    end_value: AnimValue,   // Target value
    start_time: f64,        // Animation start time (performance.now())
    driver: AnimDriver,
}

/// Values that can be animated.
#[derive(Clone, Debug)]
enum AnimValue {
    Number(f64),
    Color(u32), // RRGGBB
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
/// Inspector event log entry.
#[derive(Clone)]
struct EventRecord {
    timestamp_ms: f64,
    event_type: String,
    target_kind: String,
    target_path: String,
    state_changes: Vec<(String, String, String)>, // (var, old_val, new_val)
}

/// Inspector network log entry.
#[derive(Clone)]
struct NetworkRecord {
    timestamp_ms: f64,
    url: String,
    method: String,
    status: u16,
    duration_ms: f64,
    preview: String,
}

const MAX_EVENT_LOG: usize = 200;
const MAX_NETWORK_LOG: usize = 100;

struct App {
    render_tree: RenderTree,
    state_store: HashMap<String, RenderValue>,
    renderer: Renderer,
    layout: Option<LayoutTree>,
    raf_pending: bool,
    current_path: String,                // Current page path for routing
    focused_input: Option<FocusedInput>, // Currently focused text input
    focused_element_id: Option<String>,  // Currently focused element (for keyboard nav)
    hovered_element_id: Option<String>,  // Element currently under mouse (for hover events)
    // Animation state
    animations: Vec<ActiveAnimation>, // Currently running animations
    prev_props: HashMap<String, HashMap<String, RenderValue>>, // Previous prop values by node key
    completed_keyframes: HashSet<String>, // Keyframe animations that have already played (node_key::property)
    open_select_id: Option<String>,   // Currently open select dropdown
    caret_visible: bool,              // Blinking caret state
    caret_interval_id: Option<i32>,   // setInterval ID for caret blinking
    drag_state: Option<DragState>,    // Active drag operation
    scroll_states: HashMap<String, ScrollState>, // Scroll position per container
    // Touch scroll state
    touch_start_x: f32,
    touch_start_y: f32,
    touch_scroll_id: Option<String>,
    touch_identifier: Option<i32>,
    // Accessibility: previous text content for live region announcements
    prev_a11y_texts: Vec<String>,
    // Inspector state
    highlight_path: Option<String>,
    event_log: Vec<EventRecord>,
    network_log: Vec<NetworkRecord>,
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
    // 0. Set up panic hook for better error messages
    console_error_panic_hook::set_once();

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
        state_store.insert(state_key(&decl.name, "loading"), RenderValue::Bool(true));
        state_store.insert(
            state_key(&decl.name, "error"),
            RenderValue::Str(String::new()),
        );
        state_store.insert(state_key(&decl.name, "data"), RenderValue::List(vec![]));
    }

    // 3a2. Initialize server call state variables (same three-state pattern)
    for call in &render_tree.server_calls {
        state_store.insert(state_key(&call.name, "loading"), RenderValue::Bool(true));
        state_store.insert(
            state_key(&call.name, "error"),
            RenderValue::Str(String::new()),
        );
        state_store.insert(state_key(&call.name, "data"), RenderValue::List(vec![]));
    }

    // 3a3. Initialize prompt state variables (same three-state pattern)
    for prompt in &render_tree.prompts {
        state_store.insert(state_key(&prompt.name, "loading"), RenderValue::Bool(true));
        state_store.insert(
            state_key(&prompt.name, "error"),
            RenderValue::Str(String::new()),
        );
        state_store.insert(
            state_key(&prompt.name, "data"),
            RenderValue::Str(String::new()),
        );
    }

    // 3b. Initialize storage-backed state (read from localStorage/sessionStorage)
    init_storage(&render_tree.storage, &mut state_store);

    // 3b2. Initialize theme state from default theme (for runtime theme switching)
    // First apply built-in "default" tokens, then overlay the first user-defined theme
    // so all theme.colors.* / theme.spacing.* refs resolve at runtime.
    {
        let mut init_theme_name = String::new();
        for theme in &render_tree.themes {
            // Apply "default" base, then stop at first user-defined theme
            let is_builtin = theme.name == "default" || theme.name.is_empty();
            for (name, color) in &theme.colors {
                state_store.insert(
                    format!("theme.colors.{}", name),
                    RenderValue::Color(*color),
                );
            }
            for (name, value) in &theme.spacing {
                state_store.insert(
                    format!("theme.spacing.{}", name),
                    RenderValue::Num(*value, Some("px".into())),
                );
            }
            if !is_builtin && init_theme_name.is_empty() {
                init_theme_name = theme.name.clone();
                break; // Stop after first user-defined theme
            }
        }
        if init_theme_name.is_empty() {
            if let Some(t) = render_tree.themes.first() {
                init_theme_name = t.name.clone();
            }
        }
        state_store.insert(
            "active-theme".to_string(),
            RenderValue::Str(init_theme_name),
        );
    }

    // 3c. Initialize URL params (read from query string, fallback to defaults)
    init_params(&render_tree.params, &mut state_store);

    // 3d. Evaluate computed values (derived from initial state)
    recompute_computed(&render_tree.computed, &mut state_store);

    // 4. Set up the renderer
    let renderer = Renderer::new(canvas_id)?;

    // 5. Get initial path from URL or default to "/"
    let current_path = get_current_path();

    // 6. Collect data declarations for fetching after setup (skip manual triggers and streams)
    let data_fetches: Vec<(String, String, String)> = render_tree
        .data
        .iter()
        .filter(|d| d.trigger_mode == 0 && d.source_type == 0) // skip manual trigger and streams
        .map(|d| (d.name.clone(), d.url.clone(), d.method.clone()))
        .collect();

    // 6b. Collect stream declarations for WebSocket/SSE connections
    let stream_connects: Vec<(String, String)> = render_tree
        .data
        .iter()
        .filter(|d| d.source_type == 1) // stream sources
        .map(|d| (d.name.clone(), d.url.clone()))
        .collect();

    // 6c. Collect server call declarations (evaluate args against current state)
    let server_calls: Vec<(String, String, Vec<RenderValue>)> = render_tree
        .server_calls
        .iter()
        .map(|call| {
            let args: Vec<RenderValue> = call
                .args
                .iter()
                .map(|a| evaluate_expr(a, &state_store))
                .collect();
            (call.name.clone(), call.func_name.clone(), args)
        })
        .collect();

    // 6d. Collect device API declarations
    let device_data: Vec<(String, String, bool)> = render_tree
        .data
        .iter()
        .filter(|d| d.source_type == 4) // device sources
        .map(|d| (d.name.clone(), d.url.clone(), d.watch))
        .collect();

    // 6e. Collect JS call data declarations
    let js_call_data: Vec<(String, String)> = render_tree
        .data
        .iter()
        .filter(|d| d.source_type == 3) // JS call sources
        .map(|d| (d.name.clone(), d.url.clone()))
        .collect();

    // 6f. Collect prompt declarations with interpolation vars from current state
    let prompt_calls: Vec<(String, HashMap<String, String>)> = render_tree
        .prompts
        .iter()
        .map(|p| {
            let vars = collect_prompt_vars(&p.system, &p.user, &state_store);
            (p.name.clone(), vars)
        })
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
            completed_keyframes: HashSet::new(),
            open_select_id: None,
            caret_visible: true,
            caret_interval_id: None,
            drag_state: None,
            scroll_states: HashMap::new(),
            touch_start_x: 0.0,
            touch_start_y: 0.0,
            touch_scroll_id: None,
            touch_identifier: None,
            prev_a11y_texts: Vec::new(),
            highlight_path: None,
            event_log: Vec::new(),
            network_log: Vec::new(),
        });
    });

    // 8. Create hidden input element for text input focus
    create_hidden_input(canvas_id)?;
    create_hidden_textarea(canvas_id)?;

    // 9. Create screen reader accessibility container
    create_a11y_container()?;

    // 10. Initial render (synchronous)
    do_render()?;

    // 11. Set up event listeners on the canvas
    setup_event_listeners()?;

    // 12. Set up popstate handler for browser back/forward
    setup_popstate_handler()?;

    // 13. Fire off data fetches
    for (name, url, method) in data_fetches {
        fetch_data(&name, &url, &method);
    }

    // 13b. Connect WebSocket/SSE streams
    for (name, url) in stream_connects {
        connect_stream(&name, &url);
    }

    // 13c. Fire off server function calls
    for (name, func_name, args) in server_calls {
        call_server_function(&name, &func_name, args);
    }

    // 13d. Fire off prompt calls
    for (name, vars) in prompt_calls {
        call_prompt(&name, vars);
    }

    // 13e. Initialize device API data sources
    for (name, api, watch) in device_data {
        init_device_data(&name, &api, watch);
    }

    // 13f. Initialize JS call data sources
    for (name, func_name) in js_call_data {
        init_js_call_data(&name, &func_name);
    }

    // 14. Set up timers
    setup_timers();

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

    // 2b. Initialize data state variables (loading=true, error=null, data=null)
    for decl in &render_tree.data {
        state_store.insert(state_key(&decl.name, "loading"), RenderValue::Bool(true));
        state_store.insert(
            state_key(&decl.name, "error"),
            RenderValue::Str(String::new()),
        );
        state_store.insert(state_key(&decl.name, "data"), RenderValue::List(vec![]));
    }

    // 2b2. Initialize server call state variables
    for call in &render_tree.server_calls {
        state_store.insert(state_key(&call.name, "loading"), RenderValue::Bool(true));
        state_store.insert(
            state_key(&call.name, "error"),
            RenderValue::Str(String::new()),
        );
        state_store.insert(state_key(&call.name, "data"), RenderValue::List(vec![]));
    }

    // 2b3. Initialize prompt state variables
    for prompt in &render_tree.prompts {
        state_store.insert(state_key(&prompt.name, "loading"), RenderValue::Bool(true));
        state_store.insert(
            state_key(&prompt.name, "error"),
            RenderValue::Str(String::new()),
        );
        state_store.insert(
            state_key(&prompt.name, "data"),
            RenderValue::Str(String::new()),
        );
    }

    // 2c. Initialize storage-backed state
    init_storage(&render_tree.storage, &mut state_store);

    // 2d. Initialize URL params
    init_params(&render_tree.params, &mut state_store);

    // 2e. Evaluate computed values
    recompute_computed(&render_tree.computed, &mut state_store);

    // 2f. Collect server calls to fire after state update
    let server_calls: Vec<(String, String, Vec<RenderValue>)> = render_tree
        .server_calls
        .iter()
        .map(|call| {
            let args: Vec<RenderValue> = call
                .args
                .iter()
                .map(|a| evaluate_expr(a, &state_store))
                .collect();
            (call.name.clone(), call.func_name.clone(), args)
        })
        .collect();

    // 2g. Collect prompt declarations with interpolation vars
    let prompt_calls: Vec<(String, HashMap<String, String>)> = render_tree
        .prompts
        .iter()
        .map(|p| {
            let vars = collect_prompt_vars(&p.system, &p.user, &state_store);
            (p.name.clone(), vars)
        })
        .collect();

    // 3. Update global app state
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
            app.animations.clear(); // Clear running animations
            app.prev_props.clear(); // Clear previous prop values for animations
            app.touch_scroll_id = None; // Clear touch scroll state
            app.touch_identifier = None;
            app.prev_a11y_texts.clear(); // Reset live region tracking
        }
    });

    // Blur hidden input when switching examples
    blur_hidden_input();

    // 4. Re-render with new content
    do_render()?;

    // 5. Fire off server function calls
    for (name, func_name, args) in server_calls {
        call_server_function(&name, &func_name, args);
    }

    // 5b. Fire off prompt calls
    for (name, vars) in prompt_calls {
        call_prompt(&name, vars);
    }

    Ok(())
}

// ─── Inspector exports ──────────────────────────────────────────────────────

/// Return the node tree as JSON for the inspector panel.
#[wasm_bindgen]
pub fn inspector_get_tree() -> String {
    APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return "null".into() };
        let app = match borrow.as_ref() {
            Some(a) => a,
            None => return "null".into(),
        };

        // Use the resolved + layout tree for accurate display
        let resolved = resolve_tree(&app.render_tree, &app.state_store);
        let (page_nodes, _params) = get_page_nodes(&resolved, &app.current_path);
        let combined: Vec<RenderNode> = if resolved.pages.is_empty() {
            resolved.root.clone()
        } else {
            let mut c = resolved.root.clone();
            c.extend(page_nodes.iter().cloned());
            c
        };

        let mut json = String::with_capacity(4096);
        json.push('[');
        for (i, node) in combined.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            node_to_json(node, &mut json, "".into(), i);
        }
        json.push(']');
        json
    })
}

/// Return all state variables as JSON for the inspector panel.
#[wasm_bindgen]
pub fn inspector_get_state() -> String {
    APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return "{}".into() };
        let app = match borrow.as_ref() {
            Some(a) => a,
            None => return "{}".into(),
        };

        let mut json = String::with_capacity(2048);
        json.push('{');
        let mut first = true;
        let mut keys: Vec<&String> = app.state_store.keys().collect();
        keys.sort();
        for key in keys {
            let val = &app.state_store[key];
            if !first {
                json.push(',');
            }
            first = false;
            json.push('"');
            json_escape_into(key, &mut json);
            json.push_str("\":");
            render_value_to_json(val, &mut json);
        }
        json.push('}');
        json
    })
}

/// Hit-test at canvas coordinates and return node info as JSON.
#[wasm_bindgen]
pub fn inspector_node_at(x: f32, y: f32) -> String {
    APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return "null".into() };
        let app = match borrow.as_ref() {
            Some(a) => a,
            None => return "null".into(),
        };

        let layout = match &app.layout {
            Some(l) => l,
            None => return "null".into(),
        };

        // Walk positioned nodes to find deepest hit
        if let Some(hit) = find_node_at(&layout.root, x, y, String::new()) {
            let mut json = String::with_capacity(512);
            json.push_str("{\"kind\":\"");
            json_escape_into(&hit.kind, &mut json);
            json.push_str("\",\"path\":\"");
            json_escape_into(&hit.path, &mut json);
            json.push_str("\",\"layout\":{\"x\":");
            json.push_str(&hit.bounds.0.to_string());
            json.push_str(",\"y\":");
            json.push_str(&hit.bounds.1.to_string());
            json.push_str(",\"w\":");
            json.push_str(&hit.bounds.2.to_string());
            json.push_str(",\"h\":");
            json.push_str(&hit.bounds.3.to_string());
            json.push_str("},\"handlers\":");
            json.push_str(&hit.handlers_count.to_string());
            json.push_str(",\"props\":{");
            let mut first = true;
            for (k, v) in &hit.props {
                if !first {
                    json.push(',');
                }
                first = false;
                json.push('"');
                json_escape_into(k, &mut json);
                json.push_str("\":");
                render_value_to_json(v, &mut json);
            }
            json.push_str("}}");
            json
        } else {
            "null".into()
        }
    })
}

/// Set or clear the highlighted node path for the inspector overlay.
#[wasm_bindgen]
pub fn inspector_set_highlight(path: &str) {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        if let Some(app) = borrow.as_mut() {
            if path.is_empty() {
                app.highlight_path = None;
            } else {
                app.highlight_path = Some(path.to_string());
            }
            app.raf_pending = false; // Force re-render
        }
    });
    schedule_render();
}

/// Return the event log as JSON for the inspector debugger.
#[wasm_bindgen]
pub fn inspector_get_event_log() -> String {
    APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return "[]".into() };
        let app = match borrow.as_ref() {
            Some(a) => a,
            None => return "[]".into(),
        };

        let mut json = String::with_capacity(2048);
        json.push('[');
        for (i, evt) in app.event_log.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str("{\"t\":");
            json.push_str(&evt.timestamp_ms.to_string());
            json.push_str(",\"type\":\"");
            json_escape_into(&evt.event_type, &mut json);
            json.push_str("\",\"target\":\"");
            json_escape_into(&evt.target_kind, &mut json);
            json.push_str("\",\"path\":\"");
            json_escape_into(&evt.target_path, &mut json);
            json.push_str("\",\"changes\":[");
            for (j, (var, old, new)) in evt.state_changes.iter().enumerate() {
                if j > 0 {
                    json.push(',');
                }
                json.push_str("{\"var\":\"");
                json_escape_into(var, &mut json);
                json.push_str("\",\"old\":\"");
                json_escape_into(old, &mut json);
                json.push_str("\",\"new\":\"");
                json_escape_into(new, &mut json);
                json.push_str("\"}");
            }
            json.push_str("]}");
        }
        json.push(']');
        json
    })
}

/// Return the network log as JSON for the inspector debugger.
#[wasm_bindgen]
pub fn inspector_get_network_log() -> String {
    APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return "[]".into() };
        let app = match borrow.as_ref() {
            Some(a) => a,
            None => return "[]".into(),
        };

        let mut json = String::with_capacity(1024);
        json.push('[');
        for (i, net) in app.network_log.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str("{\"t\":");
            json.push_str(&net.timestamp_ms.to_string());
            json.push_str(",\"url\":\"");
            json_escape_into(&net.url, &mut json);
            json.push_str("\",\"method\":\"");
            json_escape_into(&net.method, &mut json);
            json.push_str("\",\"status\":");
            json.push_str(&net.status.to_string());
            json.push_str(",\"ms\":");
            json.push_str(&net.duration_ms.to_string());
            json.push_str(",\"preview\":\"");
            json_escape_into(&net.preview, &mut json);
            json.push_str("\"}");
        }
        json.push(']');
        json
    })
}

// ─── Inspector helpers ─────────────────────────────────────────────────────

/// Convert a RenderNode to JSON for the inspector tree view.
fn node_to_json(node: &RenderNode, json: &mut String, prefix: String, index: usize) {
    let path = if prefix.is_empty() {
        index.to_string()
    } else {
        format!("{}.{}", prefix, index)
    };

    json.push_str("{\"kind\":\"");
    json_escape_into(&node.kind, json);
    json.push_str("\",\"path\":\"");
    json_escape_into(&path, json);
    json.push_str("\",\"props\":{");
    let mut first = true;
    for (k, v) in &node.props {
        if !first {
            json.push(',');
        }
        first = false;
        json.push('"');
        json_escape_into(k, json);
        json.push_str("\":");
        render_value_to_json(v, json);
    }
    json.push_str("},\"handlers\":");
    json.push_str(&node.handlers.len().to_string());
    json.push_str(",\"children\":[");
    for (i, child) in node.children.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        node_to_json(child, json, path.clone(), i);
    }
    json.push_str("]}");
}

/// Convert a RenderValue to JSON string.
fn render_value_to_json(val: &RenderValue, json: &mut String) {
    match val {
        RenderValue::Str(s) => {
            json.push('"');
            json_escape_into(s, json);
            json.push('"');
        }
        RenderValue::Num(n, unit) => {
            if let Some(u) = unit {
                json.push('"');
                json.push_str(&n.to_string());
                json_escape_into(u, json);
                json.push('"');
            } else {
                json.push_str(&n.to_string());
            }
        }
        RenderValue::Color(c) => {
            json.push_str(&format!("\"#{:06x}\"", c));
        }
        RenderValue::Bool(b) => {
            json.push_str(if *b { "true" } else { "false" });
        }
        RenderValue::InterpolatedStr(parts) => {
            json.push('"');
            for part in parts {
                match part {
                    naze_ir::TextPart::Literal(s) => json_escape_into(s, json),
                    naze_ir::TextPart::StateRef(s) => {
                        json.push_str("{");
                        json_escape_into(s, json);
                        json.push_str("}");
                    }
                }
            }
            json.push('"');
        }
        RenderValue::List(items) => {
            json.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                render_value_to_json(item, json);
            }
            json.push(']');
        }
        RenderValue::Object(fields) => {
            json.push('{');
            for (i, (k, v)) in fields.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                json.push('"');
                json_escape_into(k, json);
                json.push_str("\":");
                render_value_to_json(v, json);
            }
            json.push('}');
        }
        RenderValue::Bind(name) => {
            json.push_str("\"bind:");
            json_escape_into(name, json);
            json.push('"');
        }
    }
}

/// Escape a string for JSON output.
fn json_escape_into(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
}

/// Hit info from find_node_at.
struct HitInfo {
    kind: String,
    props: HashMap<String, RenderValue>,
    handlers_count: usize,
    path: String,
    bounds: (f32, f32, f32, f32),
}

/// Find the deepest node at (x, y) in the positioned node tree.
fn find_node_at(nodes: &[PositionedNode], x: f32, y: f32, prefix: String) -> Option<HitInfo> {
    let mut result = None;
    for (i, pn) in nodes.iter().enumerate() {
        let path = if prefix.is_empty() {
            i.to_string()
        } else {
            format!("{}.{}", prefix, i)
        };
        let (nx, ny, nw, nh) = (pn.x, pn.y, pn.width, pn.height);
        if x >= nx && x <= nx + nw && y >= ny && y <= ny + nh {
            result = Some(HitInfo {
                kind: pn.kind.clone(),
                props: pn.props.clone(),
                handlers_count: pn.handlers.len(),
                path: path.clone(),
                bounds: (nx, ny, nw, nh),
            });
            // Check children for deeper hit
            if let Some(deeper) = find_node_at(&pn.children, x, y, path) {
                result = Some(deeper);
            }
        }
    }
    result
}

/// Log an event to the inspector event log (standalone — borrows APP internally).
#[allow(dead_code)]
fn log_event(
    event_type: &str,
    target_kind: &str,
    target_path: &str,
    state_changes: Vec<(String, String, String)>,
) {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        if let Some(app) = borrow.as_mut() {
            log_event_direct(app, event_type, target_kind, target_path, state_changes);
        }
    });
}

/// Log an event directly on a borrowed App (for use within existing APP.with closures).
fn log_event_direct(
    app: &mut App,
    event_type: &str,
    target_kind: &str,
    target_path: &str,
    state_changes: Vec<(String, String, String)>,
) {
    let ts = get_now_ms();
    app.event_log.push(EventRecord {
        timestamp_ms: ts,
        event_type: event_type.into(),
        target_kind: target_kind.into(),
        target_path: target_path.into(),
        state_changes,
    });
    if app.event_log.len() > MAX_EVENT_LOG {
        app.event_log.remove(0);
    }
}

/// Log a network request to the inspector network log.
fn log_network(url: &str, method: &str, status: u16, duration_ms: f64, preview: &str) {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        if let Some(app) = borrow.as_mut() {
            let ts = get_now_ms();
            app.network_log.push(NetworkRecord {
                timestamp_ms: ts,
                url: url.into(),
                method: method.into(),
                status,
                duration_ms,
                preview: if preview.len() > 200 {
                    format!("{}...", &preview[..200])
                } else {
                    preview.into()
                },
            });
            if app.network_log.len() > MAX_NETWORK_LOG {
                app.network_log.remove(0);
            }
        }
    });
}

/// Snapshot state variables for diff tracking.
#[allow(dead_code)]
fn snapshot_state_for_diff(vars: &[&str]) -> HashMap<String, String> {
    APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return HashMap::new() };
        let app = match borrow.as_ref() {
            Some(a) => a,
            None => return HashMap::new(),
        };
        let mut snap = HashMap::new();
        for &var in vars {
            if let Some(val) = app.state_store.get(var) {
                snap.insert(var.into(), render_value_brief(val));
            }
        }
        snap
    })
}

/// Brief display of a RenderValue for inspector logs.
#[allow(dead_code)]
fn render_value_brief(val: &RenderValue) -> String {
    match val {
        RenderValue::Str(s) => {
            if s.len() > 50 {
                format!("\"{}...\"", &s[..50])
            } else {
                format!("\"{}\"", s)
            }
        }
        RenderValue::Num(n, _) => n.to_string(),
        RenderValue::Bool(b) => b.to_string(),
        RenderValue::Color(c) => format!("#{:06x}", c),
        RenderValue::List(items) => format!("[{} items]", items.len()),
        RenderValue::Object(fields) => format!("{{{} fields}}", fields.len()),
        _ => "...".into(),
    }
}

/// Find node layout bounds by tree path (e.g., "0.1.2").
fn find_node_bounds_by_path(
    nodes: &[PositionedNode],
    target_path: &str,
    prefix: &String,
    _depth: usize,
) -> Option<(f32, f32, f32, f32)> {
    for (i, pn) in nodes.iter().enumerate() {
        let path = if prefix.is_empty() {
            i.to_string()
        } else {
            format!("{}.{}", prefix, i)
        };
        if path == target_path {
            return Some((pn.x, pn.y, pn.width, pn.height));
        }
        if target_path.starts_with(&format!("{}.", path)) {
            if let Some(result) =
                find_node_bounds_by_path(&pn.children, target_path, &path, _depth + 1)
            {
                return Some(result);
            }
        }
    }
    None
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
/// Returns (modified_props_by_node_key, has_active_animations, all_layout_invariant).
fn process_animations(
    root: &[RenderNode],
    animations: &mut Vec<ActiveAnimation>,
    prev_props: &mut HashMap<String, HashMap<String, RenderValue>>,
    completed_keyframes: &mut HashSet<String>,
    now: f64,
) -> (HashMap<String, HashMap<String, RenderValue>>, bool, bool, bool) {
    // 1. Detect new animations FIRST — so newly appearing elements get their
    //    initial animated value computed on the same frame they appear.
    let anim_count_before = animations.len();
    detect_new_animations(root, "", animations, prev_props, completed_keyframes, now);
    let has_new = animations.len() > anim_count_before;

    // 2. Record completed keyframe animations before removing them
    for anim in animations.iter() {
        if let AnimDriver::Keyframe { duration_ms, .. } = &anim.driver {
            if now - anim.start_time >= *duration_ms {
                completed_keyframes.insert(format!("{}::{}", anim.node_key, anim.property));
            }
        }
    }

    // 3. Remove completed animations
    animations.retain(|anim| match &anim.driver {
        AnimDriver::Timed { duration_ms, .. } => {
            let elapsed = now - anim.start_time;
            elapsed < *duration_ms
        }
        AnimDriver::Spring { state, .. } => {
            let settled = (state.position - 1.0).abs() < 0.001 && state.velocity.abs() < 0.001;
            let timed_out = (now - anim.start_time) > 5000.0;
            !settled && !timed_out
        }
        AnimDriver::Keyframe { duration_ms, .. } => {
            let elapsed = now - anim.start_time;
            elapsed < *duration_ms
        }
    });

    // 4. Compute interpolated values for all active animations
    let mut animated_props: HashMap<String, HashMap<String, RenderValue>> = HashMap::new();
    for anim in animations.iter_mut() {
        let (current, target_prop) = match &mut anim.driver {
            AnimDriver::Timed {
                duration_ms,
                easing,
            } => {
                let elapsed = now - anim.start_time;
                let progress = (elapsed / *duration_ms).min(1.0);
                let eased = easing.apply(progress);
                let val = anim.start_value.interpolate(&anim.end_value, eased);
                (val.to_render_value(), anim.property.clone())
            }
            AnimDriver::Spring {
                stiffness,
                damping,
                state,
                last_time,
            } => {
                let dt = now - *last_time;
                *last_time = now;
                let (pos, _settled) = state.step(dt, *stiffness, *damping);
                let val = anim.start_value.interpolate(&anim.end_value, pos);
                (val.to_render_value(), anim.property.clone())
            }
            AnimDriver::Keyframe {
                values,
                duration_ms,
                easing,
            } => {
                let elapsed = now - anim.start_time;
                let progress = (elapsed / *duration_ms).min(1.0);
                let eased = easing.apply(progress);
                let val = interpolate_keyframes(values, eased);
                let rv = keyframe_to_render_value(&anim.property, &val);
                let prop = keyframe_target_prop(&anim.property);
                (rv, prop)
            }
        };

        animated_props
            .entry(anim.node_key.clone())
            .or_default()
            .insert(target_prop, current);
    }

    let has_active = !animations.is_empty();
    let layout_invariant = !animations.is_empty()
        && animations.iter().all(|a| {
            matches!(
                a.property.as_str(),
                "transform" | "opacity" | "color" | "border-color" | "shadow" | "scale" | "rotate"
            )
        });

    (animated_props, has_active, layout_invariant, has_new)
}

/// Interpolate between keyframe values at a given progress (0.0-1.0).
fn interpolate_keyframes(values: &[AnimValue], progress: f64) -> AnimValue {
    if values.len() < 2 {
        return values.first().cloned().unwrap_or(AnimValue::Number(0.0));
    }
    let n = values.len() - 1;
    let segment_progress = progress * n as f64;
    let segment_index = (segment_progress.floor() as usize).min(n - 1);
    let local_progress = segment_progress - segment_index as f64;
    values[segment_index].interpolate(&values[segment_index + 1], local_progress)
}

/// Map keyframe property names to their render prop target.
fn keyframe_target_prop(property: &str) -> String {
    match property {
        "scale" | "rotate" => "transform".to_string(),
        other => other.to_string(),
    }
}

/// Convert an interpolated keyframe AnimValue to a RenderValue, applying
/// property-specific formatting (e.g., scale → "scale(N)").
fn keyframe_to_render_value(property: &str, val: &AnimValue) -> RenderValue {
    match (property, val) {
        ("scale", AnimValue::Number(n)) => RenderValue::Str(format!("scale({})", n)),
        ("rotate", AnimValue::Number(n)) => RenderValue::Str(format!("rotate({}deg)", n)),
        _ => val.to_render_value(),
    }
}

/// Walk the tree to detect property changes and start new animations.
fn detect_new_animations(
    nodes: &[RenderNode],
    parent_key: &str,
    animations: &mut Vec<ActiveAnimation>,
    prev_props: &mut HashMap<String, HashMap<String, RenderValue>>,
    completed_keyframes: &mut HashSet<String>,
    now: f64,
) {
    for (i, node) in nodes.iter().enumerate() {
        // Generate node key based on position and optional id
        let node_key = if let Some(RenderValue::Str(id)) = node.props.get("id") {
            format!("{}_{}", parent_key, id)
        } else {
            format!("{}_{}_{}", parent_key, node.kind, i)
        };

        // --- Transition-based animations (value-change driven) ---
        let transitions = parse_transitions(&node.props);

        if !transitions.is_empty() {
            let prev = prev_props.entry(node_key.clone()).or_default();

            for spec in transitions {
                if let Some(current_value) = node.props.get(&spec.property) {
                    if let Some(prev_value) = prev.get(&spec.property) {
                        if current_value != prev_value {
                            if let (Some(start), Some(end)) = (
                                render_value_to_anim(prev_value),
                                render_value_to_anim(current_value),
                            ) {
                                // Remove any existing animation for this property
                                animations.retain(|a| {
                                    !(a.node_key == node_key && a.property == spec.property)
                                });

                                let driver = match spec.driver {
                                    AnimDriverSpec::Timed {
                                        duration_ms,
                                        easing,
                                    } => AnimDriver::Timed {
                                        duration_ms,
                                        easing,
                                    },
                                    AnimDriverSpec::Spring { stiffness, damping } => {
                                        AnimDriver::Spring {
                                            stiffness,
                                            damping,
                                            state: SpringState::new(),
                                            last_time: now,
                                        }
                                    }
                                };

                                animations.push(ActiveAnimation {
                                    node_key: node_key.clone(),
                                    property: spec.property.clone(),
                                    start_value: start,
                                    end_value: end,
                                    start_time: now,
                                    driver,
                                });
                            }
                        }
                    }

                    prev.insert(spec.property.clone(), current_value.clone());
                }
            }
        }

        // --- Keyframe animations (trigger on node appearance) ---
        if let Some(RenderValue::Str(animate_str)) = node.props.get("animate") {
            // Split on commas that are OUTSIDE brackets
            for part in split_animate_specs(animate_str) {
                if let Some(kf) = KeyframeSpec::parse(part.trim()) {
                    let kf_key = format!("{}::{}", node_key, kf.property);
                    let already_running = animations
                        .iter()
                        .any(|a| a.node_key == node_key && a.property == kf.property);
                    let already_completed = completed_keyframes.contains(&kf_key);
                    if !already_running && !already_completed {
                        let first = kf.values.first().cloned().unwrap_or(AnimValue::Number(0.0));
                        let last = kf.values.last().cloned().unwrap_or(AnimValue::Number(1.0));
                        animations.push(ActiveAnimation {
                            node_key: node_key.clone(),
                            property: kf.property,
                            start_value: first,
                            end_value: last,
                            start_time: now,
                            driver: AnimDriver::Keyframe {
                                values: kf.values,
                                duration_ms: kf.duration_ms,
                                easing: kf.easing,
                            },
                        });
                    }
                }
            }
        }

        // Recurse into children
        detect_new_animations(&node.children, &node_key, animations, prev_props, completed_keyframes, now);
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
                span: node.span.clone(),
                condition: node.condition.clone(),
                else_children: node.else_children.clone(),
                each_binding: node.each_binding.clone(),
            }
        })
        .collect()
}

/// Apply animated values directly to a positioned layout tree (fast path — no re-layout).
fn apply_animated_values_to_layout(
    nodes: &mut Vec<naze_layout::PositionedNode>,
    animated_props: &HashMap<String, HashMap<String, RenderValue>>,
    parent_key: &str,
) {
    for (i, node) in nodes.iter_mut().enumerate() {
        let node_key = if let Some(RenderValue::Str(id)) = node.props.get("id") {
            format!("{}_{}", parent_key, id)
        } else {
            format!("{}_{}_{}", parent_key, node.kind, i)
        };
        if let Some(node_anims) = animated_props.get(&node_key) {
            for (prop_name, value) in node_anims {
                node.props.insert(prop_name.clone(), value.clone());
            }
        }
        apply_animated_values_to_layout(&mut node.children, animated_props, &node_key);
    }
}

/// Schedule an animation frame (for continuous animations).
/// Unlike schedule_render, this always schedules even if one is pending.
fn schedule_animation_frame() {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Ok(false), };
        let app = borrow
            .as_mut()
            .ok_or_else(|| JsValue::from_str("app not initialized"))?;
        app.raf_pending = false;

        // Get current time for animation processing
        let now = get_now_ms();

        // 0. Re-evaluate computed values and persist storage before resolving
        recompute_computed(&app.render_tree.computed, &mut app.state_store);
        persist_storage(&app.render_tree.storage, &app.state_store);

        // 1. Resolve interpolated strings against current state
        let resolved = resolve_tree(&app.render_tree, &app.state_store);

        // 2. Build combined tree: root content (headers, nav) + current page content
        let (page_nodes, route_params) = get_page_nodes(&resolved, &app.current_path);
        // Inject route params into state store so {params.id} etc. resolve
        for (name, value) in &route_params {
            app.state_store
                .insert(format!("params.{name}"), RenderValue::Str(value.clone()));
        }
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
        let (animated_props, has_active, layout_invariant, has_new) = process_animations(
            &combined_root,
            &mut app.animations,
            &mut app.prev_props,
            &mut app.completed_keyframes,
            now,
        );

        // 2b. Apply animated values to the tree
        let animated_root = apply_animated_values(&combined_root, &animated_props, "");

        let combined_tree = RenderTree {
            title: resolved.title.clone(),
            state: resolved.state.clone(),
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: animated_root,
            pages: vec![],
            themes: resolved.themes.clone(),
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
            models: vec![],
        };

        // 3. Get viewport size
        let window = web_sys::window().ok_or("no window")?;
        let vw = window.inner_width()?.as_f64().unwrap_or(1024.0) as f32;
        let vh = window.inner_height()?.as_f64().unwrap_or(768.0) as f32;

        // 4. Set canvas size to viewport
        app.renderer.set_size(vw as f64, vh as f64);

        // 5. Compute layout (or reuse cached layout for layout-invariant animations)
        //    Skip fast path when new animations were detected — the tree structure
        //    may have changed (new items added) so cached layout would be stale.
        let layout = if has_active && layout_invariant && !has_new && app.layout.is_some() {
            // Fast path: only transform/opacity/color changed — skip layout, patch props
            let mut cached = app.layout.as_ref().unwrap().clone();
            apply_animated_values_to_layout(&mut cached.root, &animated_props, "");
            apply_animated_values_to_layout(&mut cached.overlays, &animated_props, "");
            cached
        } else {
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
        let focused_input_id: Option<String> =
            app.focused_input.as_ref().map(|f| f.node_id.clone());
        let focused_element_id: Option<String> = app.focused_element_id.clone();
        let open_select: Option<String> = app.open_select_id.clone();
        let caret_visible = app.caret_visible;
        draw_tree(
            &app.renderer,
            &layout,
            &app.state_store,
            focused_input_id.as_deref(),
            focused_element_id.as_deref(),
            open_select.as_deref(),
            caret_visible,
            &app.scroll_states,
        );

        // 8. Draw drag ghost and drop zone highlighting if dragging
        if let Some(ref drag) = app.drag_state {
            // Draw drop zone highlight if over a target
            if let Some(ref target_id) = drag.over_target_id {
                // Find the target's bounds
                if let Some(target_info) =
                    find_drop_target_at_point(&layout.root, drag.current_x, drag.current_y)
                {
                    if target_info.node_id == *target_id {
                        let (tx, ty, tw, th) = target_info.bounds;
                        app.renderer
                            .draw_drop_highlight(tx as f64, ty as f64, tw as f64, th as f64);
                    }
                }
            }

            // Draw ghost element at current position
            let (_, _, sw, sh) = drag.source_bounds;
            let ghost_x = drag.current_x - (sw / 2.0);
            let ghost_y = drag.current_y - (sh / 2.0);
            app.renderer.draw_drag_ghost(
                ghost_x as f64,
                ghost_y as f64,
                sw as f64,
                sh as f64,
                &drag.source_color,
            );
        }

        // 9. Update screen reader accessibility DOM
        update_a11y_dom(&layout, &mut app.prev_a11y_texts);

        // 9a. Draw inspector highlight overlay if active
        if let Some(ref highlight_path) = app.highlight_path {
            if let Some(bounds) =
                find_node_bounds_by_path(&layout.root, highlight_path, &String::new(), 0)
            {
                let (hx, hy, hw, hh) = bounds;
                // Draw translucent blue fill + blue dashed border
                app.renderer.draw_rect(
                    hx as f64,
                    hy as f64,
                    hw as f64,
                    hh as f64,
                    "rgba(59,130,246,0.1)",
                    0.0,
                );
                app.renderer
                    .draw_drop_highlight(hx as f64, hy as f64, hw as f64, hh as f64);
            }
        }

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
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
        computed: tree.computed.clone(),
        storage: tree.storage.clone(),
        timers: tree.timers.clone(),
        params: tree.params.clone(),
        root: resolve_nodes(&tree.root, state),
        pages: tree
            .pages
            .iter()
            .map(|page| PageDef {
                path: page.path.clone(),
                params: page.params.clone(),
                is_catch_all: page.is_catch_all,
                guard: page.guard.clone(),
                meta: page.meta.clone(),
                root: resolve_nodes(&page.root, state),
            })
            .collect(),
        themes: tree.themes.clone(),
        imports: tree.imports.clone(),
        server_functions: tree.server_functions.clone(),
        server_calls: tree.server_calls.clone(),
        guards: tree.guards.clone(),
        prompts: tree.prompts.clone(),
        models: tree.models.clone(),
    }
}

/// Get the content nodes for the current page.
/// If the app has pages, returns the matching page's content.
/// Otherwise returns the root nodes.
/// Match a URL path against a route pattern with `:param` segments and `/*` catch-all.
/// Returns extracted param values as (name, value) pairs if the route matches.
fn match_route(
    pattern: &str,
    actual: &str,
    params: &[String],
    is_catch_all: bool,
) -> Option<Vec<(String, String)>> {
    if is_catch_all {
        // Catch-all matches everything — no params extracted
        return Some(vec![]);
    }

    let pat_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let act_segs: Vec<&str> = actual.split('/').filter(|s| !s.is_empty()).collect();

    if pat_segs.len() != act_segs.len() {
        return None;
    }

    let mut extracted = Vec::new();
    let mut param_idx = 0;

    for (p, a) in pat_segs.iter().zip(act_segs.iter()) {
        if p.starts_with(':') {
            // Dynamic segment — extract value
            let name = if param_idx < params.len() {
                params[param_idx].clone()
            } else {
                p[1..].to_string()
            };
            extracted.push((name, a.to_string()));
            param_idx += 1;
        } else if p != a {
            return None;
        }
    }

    Some(extracted)
}

fn get_page_nodes<'a>(
    tree: &'a RenderTree,
    current_path: &str,
) -> (&'a [RenderNode], Vec<(String, String)>) {
    if tree.pages.is_empty() {
        return (&tree.root, vec![]);
    }

    // 1. Try exact match first (fastest path)
    for page in &tree.pages {
        if !page.is_catch_all && page.params.is_empty() && page.path == current_path {
            return (&page.root, vec![]);
        }
    }

    // 2. Try dynamic routes (pattern matching)
    for page in &tree.pages {
        if !page.params.is_empty() {
            if let Some(extracted) = match_route(&page.path, current_path, &page.params, false) {
                return (&page.root, extracted);
            }
        }
    }

    // 3. Try catch-all route
    for page in &tree.pages {
        if page.is_catch_all {
            return (&page.root, vec![]);
        }
    }

    // 4. Try "/" page as fallback
    for page in &tree.pages {
        if page.path == "/" {
            return (&page.root, vec![]);
        }
    }

    // Final fallback to root nodes (non-page content like navigation bars)
    (&tree.root, vec![])
}

/// Generate a stable ID for an `each` loop item based on its content.
fn generate_stable_id(var_name: &str, item: &RenderValue, index: usize) -> String {
    match item {
        RenderValue::Object(entries) => {
            for key in &["id", "text", "name"] {
                if let Some((_, val)) = entries.iter().find(|(k, _)| k == key) {
                    return format!("{}_{}", var_name, render_value_to_id(val));
                }
            }
            format!("{}_idx_{}", var_name, index)
        }
        RenderValue::Str(s) => format!("{}_{}", var_name, sanitize_id(s)),
        RenderValue::Num(n, _) => format!("{}_{}", var_name, *n as i64),
        _ => format!("{}_idx_{}", var_name, index),
    }
}

fn render_value_to_id(val: &RenderValue) -> String {
    match val {
        RenderValue::Num(n, _) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        RenderValue::Str(s) => sanitize_id(s),
        RenderValue::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
        _ => "x".to_string(),
    }
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .take(40)
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
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
                        let index_key = format!("{}_index", var);
                        for (i, item) in items.iter().enumerate() {
                            let mut child_state = state.clone();
                            child_state.insert(var.clone(), item.clone());
                            child_state.insert(index_key.clone(), RenderValue::Num(i as f64, None));
                            let mut resolved = resolve_nodes(&node.children, &child_state);
                            // Inject stable id for animation identity
                            let stable_id = generate_stable_id(var, item, i);
                            for child in &mut resolved {
                                if !child.props.contains_key("id") {
                                    child.props.insert(
                                        "id".to_string(),
                                        RenderValue::Str(stable_id.clone()),
                                    );
                                }
                            }
                            out.extend(resolved);
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
                    handlers: resolve_handlers(&node.handlers, state),
                    span: node.span.clone(),
                    condition: None,
                    else_children: None,
                    each_binding: None,
                });
            }
        }
    }
    out
}

/// Convert a RenderValue to an IrExpression (for each-binding resolution).
fn render_value_to_ir(val: &RenderValue) -> IrExpression {
    match val {
        RenderValue::Num(n, _) => IrExpression::Num(*n),
        RenderValue::Str(s) => IrExpression::Str(s.clone()),
        RenderValue::Bool(b) => IrExpression::Bool(*b),
        RenderValue::Object(entries) => IrExpression::Object(
            entries.iter().map(|(k, v)| (k.clone(), render_value_to_ir(v))).collect(),
        ),
        RenderValue::List(items) => {
            IrExpression::List(items.iter().map(render_value_to_ir).collect())
        }
        _ => IrExpression::Str(String::new()),
    }
}

/// Resolve state refs in an IrExpression, substituting known values from state.
fn resolve_expr_state(expr: &IrExpression, state: &HashMap<String, RenderValue>) -> IrExpression {
    match expr {
        IrExpression::StateRef(name) => {
            // Direct lookup — handles all types including Object/List
            if let Some(val) = state.get(name.as_str()) {
                render_value_to_ir(val)
            }
            // Dotted path: "card.r" → lookup "card", extract field "r"
            else if let Some(dot) = name.find('.') {
                let root = &name[..dot];
                let field = &name[dot + 1..];
                if let Some(RenderValue::Object(entries)) = state.get(root) {
                    for (k, v) in entries {
                        if k == field {
                            return render_value_to_ir(v);
                        }
                    }
                }
                expr.clone()
            } else {
                expr.clone()
            }
        }
        IrExpression::BinOp { left, op, right } => IrExpression::BinOp {
            left: Box::new(resolve_expr_state(left, state)),
            op: *op,
            right: Box::new(resolve_expr_state(right, state)),
        },
        IrExpression::Object(entries) => IrExpression::Object(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), resolve_expr_state(v, state)))
                .collect(),
        ),
        IrExpression::List(items) => {
            IrExpression::List(items.iter().map(|v| resolve_expr_state(v, state)).collect())
        }
        IrExpression::Index { list, index } => IrExpression::Index {
            list: Box::new(resolve_expr_state(list, state)),
            index: Box::new(resolve_expr_state(index, state)),
        },
        IrExpression::FunctionCall { name, args } => IrExpression::FunctionCall {
            name: name.clone(),
            args: args.iter().map(|a| resolve_expr_state(a, state)).collect(),
        },
        _ => expr.clone(),
    }
}

/// Resolve state refs in event handlers, substituting loop-scoped variables.
fn resolve_handlers(
    handlers: &[IrEventHandler],
    state: &HashMap<String, RenderValue>,
) -> Vec<IrEventHandler> {
    handlers
        .iter()
        .map(|h| IrEventHandler {
            event: h.event.clone(),
            actions: h.actions.iter().map(|a| resolve_action_state(a, state)).collect(),
            modifier_kind: h.modifier_kind,
            modifier_ms: h.modifier_ms,
        })
        .collect()
}

/// Resolve state refs in an IrAction.
fn resolve_action_state(action: &IrAction, state: &HashMap<String, RenderValue>) -> IrAction {
    match action {
        IrAction::Set { target, expr } => IrAction::Set {
            target: target.clone(),
            expr: resolve_expr_state(expr, state),
        },
        IrAction::Append { item, target } => IrAction::Append {
            item: resolve_expr_state(item, state),
            target: target.clone(),
        },
        IrAction::Remove { index, target } => IrAction::Remove {
            index: resolve_expr_state(index, state),
            target: target.clone(),
        },
        IrAction::SetIndex { target, index, expr } => IrAction::SetIndex {
            target: target.clone(),
            index: resolve_expr_state(index, state),
            expr: resolve_expr_state(expr, state),
        },
        IrAction::Conditional { condition, then_actions, else_actions } => IrAction::Conditional {
            condition: resolve_expr_state(condition, state),
            then_actions: then_actions.iter().map(|a| resolve_action_state(a, state)).collect(),
            else_actions: else_actions.iter().map(|a| resolve_action_state(a, state)).collect(),
        },
        _ => action.clone(),
    }
}

/// Resolve a single value. InterpolatedStr parts are concatenated into a plain Str.
fn resolve_value(value: &RenderValue, state: &HashMap<String, RenderValue>) -> RenderValue {
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
                    TextPart::StateRef(name) => {
                        // Resolve value: direct lookup, then dotted path into objects
                        let resolved = state.get(name.as_str()).cloned().or_else(|| {
                            let dot = name.find('.')?;
                            let root = &name[..dot];
                            let field = &name[dot + 1..];
                            if let Some(RenderValue::Object(entries)) = state.get(root) {
                                for (k, v) in entries {
                                    if k == field {
                                        return Some(v.clone());
                                    }
                                }
                            }
                            None
                        });
                        match resolved.as_ref() {
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
                        }
                    }
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
        draw_node(
            renderer,
            node,
            state,
            focused_input_id,
            focused_element_id,
            open_select_id,
            caret_visible,
            scroll_states,
        );
    }
    // Draw overlays on top of root content (later index = higher z-order)
    for node in &layout.overlays {
        draw_node(
            renderer,
            node,
            state,
            focused_input_id,
            focused_element_id,
            open_select_id,
            caret_visible,
            scroll_states,
        );
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

    // Handle transform - wrap in save/restore
    let transform = naze_renderer::get_str_prop(&node.props, "transform", "");
    let needs_transform = !transform.is_empty();
    if needs_transform {
        renderer.save();
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;
        renderer.apply_transform(&transform, cx, cy);
    }

    match node.kind.as_str() {
        "rect" => {
            let shadow = naze_renderer::get_str_prop(&node.props, "shadow", "");
            let has_shadow = !shadow.is_empty();
            if has_shadow {
                renderer.save();
                renderer.apply_shadow(&shadow);
            }
            let gradient = naze_renderer::get_str_prop(&node.props, "gradient", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            if !gradient.is_empty() {
                renderer.fill_rect_with_gradient(x, y, w, h, &gradient, radius);
            } else {
                let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
                let border = naze_renderer::get_num_prop(&node.props, "border", 0.0);
                let border_color =
                    naze_renderer::get_color_prop(&node.props, "border-color", "#000000");
                renderer.draw_rect_with_border(x, y, w, h, &color, radius, border, &border_color);
            }
            if has_shadow {
                renderer.clear_shadow();
                renderer.restore();
            }
            for child in &node.children {
                draw_node(
                    renderer,
                    child,
                    state,
                    focused_input_id,
                    focused_element_id,
                    open_select_id,
                    caret_visible,
                    scroll_states,
                );
            }
        }
        "container" => {
            let shadow = naze_renderer::get_str_prop(&node.props, "shadow", "");
            let has_shadow = !shadow.is_empty();
            if has_shadow {
                renderer.save();
                renderer.apply_shadow(&shadow);
            }
            let gradient = naze_renderer::get_str_prop(&node.props, "gradient", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            if !gradient.is_empty() {
                renderer.fill_rect_with_gradient(x, y, w, h, &gradient, radius);
            } else {
                let color = naze_renderer::get_color_prop(&node.props, "color", "");
                let border = naze_renderer::get_num_prop(&node.props, "border", 0.0);
                let border_color =
                    naze_renderer::get_color_prop(&node.props, "border-color", "#000000");
                if !color.is_empty() || border > 0.0 {
                    renderer.draw_rect_with_border(
                        x,
                        y,
                        w,
                        h,
                        &color,
                        radius,
                        border,
                        &border_color,
                    );
                }
            }
            if has_shadow {
                renderer.clear_shadow();
                renderer.restore();
            }
            for child in &node.children {
                draw_node(
                    renderer,
                    child,
                    state,
                    focused_input_id,
                    focused_element_id,
                    open_select_id,
                    caret_visible,
                    scroll_states,
                );
            }
        }
        "text" => {
            let text = naze_renderer::get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = naze_renderer::get_font_size(&node.props, false);
                let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
                let text_align = naze_renderer::get_str_prop(&node.props, "text-align", "");
                let text_overflow = naze_renderer::get_str_prop(&node.props, "text-overflow", "");
                let letter_spacing =
                    naze_renderer::get_num_prop(&node.props, "letter-spacing", 0.0);
                if letter_spacing != 0.0 {
                    renderer.set_letter_spacing(letter_spacing);
                }
                if text_overflow == "ellipsis" {
                    renderer.draw_text_ellipsis(&text, x, y, font_size, false, &color, w);
                } else if !text_align.is_empty() {
                    renderer.draw_text_aligned(
                        &text,
                        x,
                        y,
                        font_size,
                        false,
                        &color,
                        &text_align,
                        w,
                    );
                } else {
                    renderer.draw_text(&text, x, y, font_size, false, &color);
                }
                let decoration = naze_renderer::get_str_prop(&node.props, "text-decoration", "");
                if !decoration.is_empty() {
                    let (tw, _) = renderer.measure_text(&text, font_size, false);
                    let draw_x = match text_align.as_str() {
                        "center" => x + (w - tw) / 2.0,
                        "right" | "end" => x + w - tw,
                        _ => x,
                    };
                    renderer.draw_text_decoration(draw_x, y, tw, font_size, &decoration, &color);
                }
                if letter_spacing != 0.0 {
                    renderer.clear_letter_spacing();
                }
            }
        }
        "heading" => {
            let text = naze_renderer::get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = naze_renderer::get_font_size(&node.props, true);
                let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
                let text_align = naze_renderer::get_str_prop(&node.props, "text-align", "");
                let text_overflow = naze_renderer::get_str_prop(&node.props, "text-overflow", "");
                let letter_spacing =
                    naze_renderer::get_num_prop(&node.props, "letter-spacing", 0.0);
                if letter_spacing != 0.0 {
                    renderer.set_letter_spacing(letter_spacing);
                }
                if text_overflow == "ellipsis" {
                    renderer.draw_text_ellipsis(&text, x, y, font_size, true, &color, w);
                } else if !text_align.is_empty() {
                    renderer.draw_text_aligned(
                        &text,
                        x,
                        y,
                        font_size,
                        true,
                        &color,
                        &text_align,
                        w,
                    );
                } else {
                    renderer.draw_text(&text, x, y, font_size, true, &color);
                }
                let decoration = naze_renderer::get_str_prop(&node.props, "text-decoration", "");
                if !decoration.is_empty() {
                    let (tw, _) = renderer.measure_text(&text, font_size, true);
                    let draw_x = match text_align.as_str() {
                        "center" => x + (w - tw) / 2.0,
                        "right" | "end" => x + w - tw,
                        _ => x,
                    };
                    renderer.draw_text_decoration(draw_x, y, tw, font_size, &decoration, &color);
                }
                if letter_spacing != 0.0 {
                    renderer.clear_letter_spacing();
                }
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
        "path" => {
            let d = naze_renderer::get_str_prop(&node.props, "d", "");
            let fill = naze_renderer::get_color_prop(&node.props, "fill", "");
            let stroke = naze_renderer::get_color_prop(&node.props, "stroke", "");
            let sw = naze_renderer::get_num_prop(&node.props, "stroke-width", 1.0);
            if !d.is_empty() {
                renderer.draw_path(x, y, &d, &fill, &stroke, sw);
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
                draw_node(
                    renderer,
                    child,
                    state,
                    focused_input_id,
                    focused_element_id,
                    open_select_id,
                    caret_visible,
                    scroll_states,
                );
            }
        }
        "checkbox" => {
            let label = naze_renderer::get_text_content(&node.props);
            let checked = match node.props.get("bind") {
                Some(RenderValue::Bind(var)) => match state.get(var) {
                    Some(RenderValue::Bool(b)) => *b,
                    _ => false,
                },
                _ => false,
            };
            renderer.draw_checkbox(x, y, checked, &label);
        }
        "radio" => {
            let label = naze_renderer::get_text_content(&node.props);
            // Radio is selected when state[bind] == value
            let selected = match (node.props.get("bind"), node.props.get("value")) {
                (Some(RenderValue::Bind(var)), Some(value)) => match state.get(var) {
                    Some(state_val) => state_val == value,
                    None => false,
                },
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
                        // For file inputs, state is an Object — extract the "name" field
                        Some(RenderValue::Object(obj)) => obj
                            .iter()
                            .find(|(k, _)| k == "name")
                            .and_then(|(_, v)| {
                                if let RenderValue::Str(s) = v {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default(),
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
            renderer.draw_input(
                x,
                y,
                w as f64,
                h as f64,
                &value,
                &placeholder,
                focused,
                &input_type,
                show_caret,
            );
        }
        "textarea" => {
            let placeholder = naze_renderer::get_str_prop(&node.props, "placeholder", "");
            // Get current value from bind
            let value = match node.props.get("bind") {
                Some(RenderValue::Bind(var)) => match state.get(var) {
                    Some(RenderValue::Str(s)) => s.clone(),
                    _ => String::new(),
                },
                _ => String::new(),
            };
            let node_id = format!("input_{}_{}", x as i32, y as i32);
            let focused = focused_input_id == Some(node_id.as_str());
            let show_caret = focused && caret_visible;
            renderer.draw_textarea(
                x,
                y,
                w as f64,
                h as f64,
                &value,
                &placeholder,
                focused,
                show_caret,
            );
        }
        "select" => {
            let placeholder = naze_renderer::get_str_prop(&node.props, "placeholder", "Select...");
            // Get current value from bind
            let current_value = match node.props.get("bind") {
                Some(RenderValue::Bind(var)) => match state.get(var) {
                    Some(RenderValue::Str(s)) => s.clone(),
                    _ => String::new(),
                },
                _ => String::new(),
            };
            // Extract options from children
            let options = extract_select_options(&node.children);
            // Find display text for current value
            let display_text = options
                .iter()
                .find(|(_, v)| v == &current_value)
                .map(|(label, _)| label.as_str())
                .unwrap_or("");
            // Check if this select is open
            let select_id = format!("select_{}_{}", x as i32, y as i32);
            let is_open = open_select_id == Some(select_id.as_str());
            renderer.draw_select(
                x,
                y,
                w,
                h,
                display_text,
                &placeholder,
                is_open,
                &options,
                &current_value,
            );
        }
        "option" => {
            // Options are rendered by the parent select, not directly
        }
        "scroll" => {
            // Scroll container: clip children, apply scroll offset, draw scrollbar
            let color = naze_renderer::get_color_prop(&node.props, "color", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            let border = naze_renderer::get_num_prop(&node.props, "border", 0.0);
            let border_color =
                naze_renderer::get_color_prop(&node.props, "border-color", "#000000");

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
                draw_node(
                    renderer,
                    child,
                    state,
                    focused_input_id,
                    focused_element_id,
                    open_select_id,
                    caret_visible,
                    scroll_states,
                );
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
            let shadow = naze_renderer::get_str_prop(&node.props, "shadow", "");
            let has_shadow = !shadow.is_empty();
            if has_shadow {
                renderer.save();
                renderer.apply_shadow(&shadow);
            }
            let gradient = naze_renderer::get_str_prop(&node.props, "gradient", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            if !gradient.is_empty() {
                renderer.fill_rect_with_gradient(x, y, w, h, &gradient, radius);
            } else {
                let color = naze_renderer::get_color_prop(&node.props, "color", "");
                if !color.is_empty() {
                    renderer.draw_rect(x, y, w, h, &color, radius);
                }
            }
            if has_shadow {
                renderer.clear_shadow();
                renderer.restore();
            }
            let overflow = naze_renderer::get_str_prop(&node.props, "overflow", "");
            let clip = overflow == "hidden" || overflow == "clip";
            if clip {
                renderer.begin_clip(x, y, w, h, radius);
            }
            for child in &node.children {
                draw_node(
                    renderer,
                    child,
                    state,
                    focused_input_id,
                    focused_element_id,
                    open_select_id,
                    caret_visible,
                    scroll_states,
                );
            }
            if clip {
                renderer.end_clip();
            }
        }
        "spacer" => {}
        _ => {
            for child in &node.children {
                draw_node(
                    renderer,
                    child,
                    state,
                    focused_input_id,
                    focused_element_id,
                    open_select_id,
                    caret_visible,
                    scroll_states,
                );
            }
        }
    }

    // Draw focus ring if this element is focused
    if is_focused {
        let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
        renderer.draw_focus_ring(x, y, w, h, radius);
    }

    if needs_transform {
        renderer.restore();
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
    let canvas = APP
        .with(|cell| {
            let borrow = cell.try_borrow().map_err(|_| "app busy")?;
            let app = borrow.as_ref().ok_or("app not initialized")?;
            Ok::<_, &str>(app.renderer.canvas_element().clone())
        })
        .map_err(|e| JsValue::from_str(e))?;

    // Mousedown handler — start drag if on draggable element
    let mousedown_cb =
        Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
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
    let mouseup_cb =
        Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
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
    let wheel_cb =
        Closure::<dyn Fn(web_sys::WheelEvent)>::new(move |event: web_sys::WheelEvent| {
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
    let keydown_cb =
        Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            let key = event.key();
            let needs_render = handle_keydown(&key, event.shift_key());
            if needs_render {
                schedule_render();
            }
        });
    window.add_event_listener_with_callback("keydown", keydown_cb.as_ref().unchecked_ref())?;
    keydown_cb.forget();

    // Context menu handler — fire context-menu event handlers, suppress browser menu
    let contextmenu_cb =
        Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            let x = event.offset_x() as f32;
            let y = event.offset_y() as f32;
            let handled = handle_contextmenu(x, y);
            if handled {
                event.prevent_default();
                schedule_render();
            }
        });
    canvas
        .add_event_listener_with_callback("contextmenu", contextmenu_cb.as_ref().unchecked_ref())?;
    contextmenu_cb.forget();

    // Touch handlers — enable scroll containers on mobile/touch devices
    let canvas_for_touch = canvas.clone();
    let touchstart_cb =
        Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
            let touches = event.changed_touches();
            if touches.length() == 0 {
                return;
            }
            let touch = touches.get(0).unwrap();
            let rect = canvas_for_touch.get_bounding_client_rect();
            let x = touch.client_x() as f32 - rect.left() as f32;
            let y = touch.client_y() as f32 - rect.top() as f32;
            let captured = handle_touchstart(x, y, touch.identifier());
            if captured {
                event.prevent_default();
            }
        });
    canvas
        .add_event_listener_with_callback("touchstart", touchstart_cb.as_ref().unchecked_ref())?;
    touchstart_cb.forget();

    let canvas_for_touchmove = canvas.clone();
    let touchmove_cb =
        Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
            let touches = event.changed_touches();
            if touches.length() == 0 {
                return;
            }
            let touch = touches.get(0).unwrap();
            let rect = canvas_for_touchmove.get_bounding_client_rect();
            let x = touch.client_x() as f32 - rect.left() as f32;
            let y = touch.client_y() as f32 - rect.top() as f32;
            let needs_render = handle_touchmove(x, y, touch.identifier());
            if needs_render {
                event.prevent_default();
                schedule_render();
            }
        });
    canvas.add_event_listener_with_callback("touchmove", touchmove_cb.as_ref().unchecked_ref())?;
    touchmove_cb.forget();

    let touchend_cb =
        Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
            let touches = event.changed_touches();
            if touches.length() == 0 {
                return;
            }
            let touch = touches.get(0).unwrap();
            handle_touchend(touch.identifier());
        });
    canvas.add_event_listener_with_callback("touchend", touchend_cb.as_ref().unchecked_ref())?;
    canvas.add_event_listener_with_callback("touchcancel", touchend_cb.as_ref().unchecked_ref())?;
    touchend_cb.forget();

    Ok(())
}

/// Handle a click at (x, y). Walks the layout tree, finds the deepest node
/// at that point with click handlers, executes them, and returns whether
/// the state was changed (needs re-render).
fn handle_click(x: f32, y: f32) -> bool {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l,
            None => return false,
        };

        // Check overlays first (topmost = last in vec, check in reverse)
        // If click lands inside an overlay, handle it there; if outside, fire click-outside
        let mut overlay_consumed = false;
        for overlay in layout.overlays.iter().rev() {
            if point_in_node(overlay, x, y) {
                // Click is inside this overlay — check for inputs/selects/click handlers within it
                if let Some((
                    bind_var,
                    node_id,
                    current_value,
                    input_type,
                    change_handlers,
                    validate_prop,
                )) = find_input_at_point(&overlay.children, x, y, &app.state_store)
                {
                    app.open_select_id = None;
                    if input_type == "file" {
                        let accept = find_input_prop_at_point(&overlay.children, x, y, "accept");
                        let max_size = parse_max_size(&find_input_prop_at_point(
                            &overlay.children,
                            x,
                            y,
                            "max-size",
                        ));
                        drop(borrow);
                        open_file_picker(&bind_var, &accept, max_size, change_handlers);
                    } else {
                        drop(borrow);
                        focus_input(
                            &bind_var,
                            &node_id,
                            &current_value,
                            &input_type,
                            change_handlers,
                            validate_prop,
                        );
                    }
                    return true;
                }
                let handlers = find_click_handlers(&overlay.children, x, y);
                if !handlers.is_empty() {
                    let mut changed = false;
                    for handler in &handlers {
                        if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                            changed = true;
                        }
                    }
                    return changed;
                }
                overlay_consumed = true;
                break;
            } else {
                // Click is outside this overlay — fire click-outside handlers
                let outside_handlers: Vec<_> = overlay
                    .handlers
                    .iter()
                    .filter(|h| h.event == "click-outside")
                    .cloned()
                    .collect();
                if !outside_handlers.is_empty() {
                    let mut changed = false;
                    for handler in &outside_handlers {
                        if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                            changed = true;
                        }
                    }
                    return changed;
                }
            }
        }

        // If an overlay consumed the click but had no specific handler, block click-through
        if overlay_consumed {
            return false;
        }

        // Check if clicking on an input element first
        if let Some((
            bind_var,
            node_id,
            current_value,
            input_type,
            change_handlers,
            validate_prop,
        )) = find_input_at_point(&layout.root, x, y, &app.state_store)
        {
            // Close any open select when clicking an input
            app.open_select_id = None;
            if input_type == "file" {
                let accept = find_input_prop_at_point(&layout.root, x, y, "accept");
                let max_size =
                    parse_max_size(&find_input_prop_at_point(&layout.root, x, y, "max-size"));
                drop(borrow);
                open_file_picker(&bind_var, &accept, max_size, change_handlers);
            } else {
                // Drop borrow before calling focus_input which needs to borrow again
                drop(borrow);
                focus_input(
                    &bind_var,
                    &node_id,
                    &current_value,
                    &input_type,
                    change_handlers,
                    validate_prop,
                );
            }
            return true; // Needs re-render to show focus
        }

        // Check if clicking on a select dropdown option (when dropdown is open)
        if let Some(open_id) = &app.open_select_id.clone() {
            if let Some((bind_var, value, change_handlers)) =
                find_option_at_point(&layout.root, x, y, open_id)
            {
                // Set the value and close the dropdown
                app.state_store.insert(bind_var, RenderValue::Str(value));
                app.open_select_id = None;
                // Execute change handlers
                for handler in &change_handlers {
                    execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes);
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

        // Snapshot state before executing actions for event logging
        let state_before: Vec<(String, String)> = app
            .state_store
            .iter()
            .map(|(k, v)| (k.clone(), render_value_brief(v)))
            .collect();

        let mut changed = false;
        for handler in &handlers {
            if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                changed = true;
            }
        }

        // Reset animation on clicked node so it replays
        reset_click_animation(
            &layout.root, x, y, "",
            &mut app.animations, &mut app.completed_keyframes,
        );

        // Log the click event with state changes
        let mut state_changes = Vec::new();
        for (k, old_val) in &state_before {
            if let Some(new_v) = app.state_store.get(k) {
                let new_val = render_value_brief(new_v);
                if *old_val != new_val {
                    state_changes.push((k.clone(), old_val.clone(), new_val));
                }
            }
        }
        log_event_direct(app, "click", "", "", state_changes);

        changed
    })
}

/// Reset the animation on a clicked node so it replays.
/// Walks the tree to find the deepest node with click handlers at (x,y),
/// computes its animation key, and removes it from completed_keyframes.
fn reset_click_animation(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
    parent_key: &str,
    animations: &mut Vec<ActiveAnimation>,
    completed_keyframes: &mut HashSet<String>,
) {
    for (i, node) in nodes.iter().enumerate() {
        if !point_in_node(node, x, y) {
            continue;
        }
        let node_key = if let Some(RenderValue::Str(id)) = node.props.get("id") {
            format!("{}_{}", parent_key, id)
        } else {
            format!("{}_{}_{}", parent_key, node.kind, i)
        };
        // Recurse into children first (deepest wins)
        reset_click_animation(
            &node.children, x, y, &node_key,
            animations, completed_keyframes,
        );
        // If this node has click handlers AND an animate prop, reset its animation
        let has_click = node.handlers.iter().any(|h| h.event == "click");
        if has_click {
            if let Some(RenderValue::Str(animate_str)) = node.props.get("animate") {
                for part in split_animate_specs(animate_str) {
                    if let Some(kf) = KeyframeSpec::parse(part.trim()) {
                        let kf_key = format!("{}::{}", node_key, kf.property);
                        completed_keyframes.remove(&kf_key);
                        animations.retain(|a| !(a.node_key == node_key && a.property == kf.property));
                    }
                }
            }
        }
    }
}

/// Check if any node at (x, y) has a handler for the given event.
/// Checks overlays first when a layout is available.
fn hit_test_any_handler(nodes: &[PositionedNode], x: f32, y: f32, event: &str) -> bool {
    // Walk depth-first; check deepest children first
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        if hit_test_any_handler(&node.children, x, y, event) {
            return true;
        }
        // Input, textarea, and select elements are clickable
        if event == "click"
            && (node.kind == "input" || node.kind == "textarea" || node.kind == "select")
        {
            return true;
        }
        if node.handlers.iter().any(|h| h.event == event) {
            return true;
        }
    }
    false
}

/// Check overlays then root for any handler at (x, y).
fn hit_test_any_handler_with_overlays(layout: &LayoutTree, x: f32, y: f32, event: &str) -> bool {
    // Check overlays first (topmost = last)
    for overlay in layout.overlays.iter().rev() {
        if point_in_node(overlay, x, y) {
            return hit_test_any_handler(&overlay.children, x, y, event);
        }
    }
    hit_test_any_handler(&layout.root, x, y, event)
}

/// Find click handlers on the deepest node at (x, y).
/// For form elements (checkbox, radio), also includes change handlers.
fn find_click_handlers(nodes: &[PositionedNode], x: f32, y: f32) -> Vec<naze_ir::IrEventHandler> {
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
                node.handlers
                    .iter()
                    .filter(|h| h.event == "change")
                    .cloned(),
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

// ─── Overlay helpers ─────────────────────────────────────────────────────────

/// Find event handlers at a point for a specific event type (e.g., "pointer-move", "context-menu").
fn find_event_handlers_at_point(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
    event: &str,
) -> Vec<naze_ir::IrEventHandler> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first (deeper wins)
        let child_handlers = find_event_handlers_at_point(&node.children, x, y, event);
        if !child_handlers.is_empty() {
            return child_handlers;
        }
        let handlers: Vec<_> = node
            .handlers
            .iter()
            .filter(|h| h.event == event)
            .cloned()
            .collect();
        if !handlers.is_empty() {
            return handlers;
        }
    }
    Vec::new()
}

/// Handle right-click / context menu. Returns true if a handler was found.
fn handle_contextmenu(x: f32, y: f32) -> bool {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return false,
        };

        // Check overlays first
        for overlay in layout.overlays.iter().rev() {
            if point_in_node(overlay, x, y) {
                let handlers =
                    find_event_handlers_at_point(&overlay.children, x, y, "context-menu");
                if !handlers.is_empty() {
                    let mut changed = false;
                    for handler in &handlers {
                        if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                            changed = true;
                        }
                    }
                    return changed;
                }
                return false; // Inside overlay but no handler
            }
        }

        // Check root
        let handlers = find_event_handlers_at_point(&layout.root, x, y, "context-menu");
        if handlers.is_empty() {
            return false;
        }
        let mut changed = false;
        for handler in &handlers {
            if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                changed = true;
            }
        }
        changed
    })
}

/// Get the topmost overlay with focus-trap: true, if any.
fn get_focus_trap_overlay(layout: &LayoutTree) -> Option<PositionedNode> {
    for overlay in layout.overlays.iter().rev() {
        if let Some(RenderValue::Bool(true)) = overlay.props.get("focus-trap") {
            return Some(overlay.clone());
        }
    }
    None
}

/// Find handlers by element ID, searching overlays first then root.
fn find_handlers_in_layout(
    layout: &LayoutTree,
    element_id: &str,
    event: &str,
) -> Option<Vec<naze_ir::IrEventHandler>> {
    // Search overlays first
    for overlay in layout.overlays.iter().rev() {
        if let Some(handlers) = find_handlers_by_element_id(&overlay.children, element_id, event) {
            return Some(handlers);
        }
    }
    find_handlers_by_element_id(&layout.root, element_id, event)
}

// ─── Action execution ────────────────────────────────────────────────────────

/// Execute all actions in an event handler. Returns true if any action changed state.
fn execute_handler_actions(handler: &IrEventHandler, state: &mut HashMap<String, RenderValue>, themes: &[naze_ir::ThemeDef]) -> bool {
    let mut changed = false;
    for action in &handler.actions {
        if execute_action(action, state, themes) {
            changed = true;
        }
    }
    changed
}

/// Execute an action, mutating the state store. Returns true if state was changed.
fn execute_action(action: &IrAction, state: &mut HashMap<String, RenderValue>, themes: &[naze_ir::ThemeDef]) -> bool {
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
        IrAction::Trigger { data_name } => {
            // Re-fetch the named data source (server call or fetch)
            APP.with(|cell| {
                if let Ok(borrow) = cell.try_borrow() {
                    if let Some(app) = borrow.as_ref() {
                        // Check server calls first
                        for call in &app.render_tree.server_calls {
                            if call.name == *data_name {
                                let args: Vec<RenderValue> = call
                                    .args
                                    .iter()
                                    .map(|a| evaluate_expr(a, state))
                                    .collect();
                                call_server_function(
                                    &call.name,
                                    &call.func_name,
                                    args,
                                );
                                return;
                            }
                        }
                        // Check data fetch declarations
                        for decl in &app.render_tree.data {
                            if decl.name == *data_name && decl.source_type == 0 {
                                fetch_data(
                                    &decl.name,
                                    &decl.url,
                                    &decl.method,
                                );
                                return;
                            }
                        }
                    }
                }
            });
            true
        }
        IrAction::Copy { expr } => {
            let value = evaluate_expr(expr, state);
            let text = render_value_to_string(&value);
            // Use the Clipboard API
            if let Some(window) = web_sys::window() {
                let clipboard = window.navigator().clipboard();
                let _ = clipboard.write_text(&text);
            }
            false // no re-render needed
        }
        IrAction::Send { stream_name, expr } => {
            let value = evaluate_expr(expr, state);
            let _text = render_value_to_string(&value);
            // WebSocket send — stub until Batch 6 (data streams)
            web_sys::console::log_1(&format!("send to {}: (stub)", stream_name).into());
            false
        }
        IrAction::JsCall {
            function_name,
            args,
            target,
        } => {
            // JS interop: call window-scoped function with args
            let window = web_sys::window().unwrap();

            // Resolve function — supports dotted paths like "Math.random"
            let js_fn = {
                let parts: Vec<&str> = function_name.split('.').collect();
                let mut obj: JsValue = window.into();
                let mut found = true;
                for part in &parts[..parts.len().saturating_sub(1)] {
                    match js_sys::Reflect::get(&obj, &JsValue::from_str(part)) {
                        Ok(next) if !next.is_undefined() => obj = next,
                        _ => {
                            found = false;
                            break;
                        }
                    }
                }
                if found {
                    let fn_name = parts.last().unwrap_or(&"");
                    js_sys::Reflect::get(&obj, &JsValue::from_str(fn_name))
                        .ok()
                        .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                        .map(|f| (f, obj))
                } else {
                    None
                }
            };

            match js_fn {
                Some((func, this_val)) => {
                    // Convert Naze args to JsValue array
                    let js_args = js_sys::Array::new();
                    for arg in args {
                        let val = evaluate_expr(arg, state);
                        match val {
                            RenderValue::Str(s) => js_args.push(&JsValue::from_str(&s)),
                            RenderValue::Num(n, _) => js_args.push(&JsValue::from_f64(n)),
                            RenderValue::Bool(b) => js_args.push(&JsValue::from_bool(b)),
                            _ => js_args.push(&JsValue::NULL),
                        };
                    }

                    match func.apply(&this_val, &js_args) {
                        Ok(result) => {
                            if let Some(target_var) = target {
                                // Convert JS result back to RenderValue
                                let naze_val = if let Some(s) = result.as_string() {
                                    RenderValue::Str(s)
                                } else if let Some(n) = result.as_f64() {
                                    RenderValue::Num(n, None)
                                } else if let Some(b) = result.as_bool() {
                                    RenderValue::Bool(b)
                                } else {
                                    RenderValue::Str("null".to_string())
                                };
                                state.insert(target_var.clone(), naze_val);
                                return true; // Re-render to show updated state
                            }
                        }
                        Err(err) => {
                            web_sys::console::error_1(&err);
                        }
                    }
                    false
                }
                None => {
                    web_sys::console::error_1(
                        &format!("JS function not found: {}", function_name).into(),
                    );
                    false
                }
            }
        }
        IrAction::Notify { title, body, icon } => {
            // Browser Notifications API
            let permission = web_sys::Notification::permission();
            match permission {
                web_sys::NotificationPermission::Granted => {
                    let options = web_sys::NotificationOptions::new();
                    if !body.is_empty() {
                        options.set_body(body);
                    }
                    if !icon.is_empty() {
                        let _ = js_sys::Reflect::set(
                            &options,
                            &JsValue::from_str("icon"),
                            &JsValue::from_str(icon),
                        );
                    }
                    let _ = web_sys::Notification::new_with_options(title, &options);
                }
                web_sys::NotificationPermission::Denied => {
                    web_sys::console::warn_1(&"Notification permission denied".into());
                }
                _ => {
                    // Request permission, then show notification
                    let title = title.clone();
                    let body = body.clone();
                    let icon = icon.clone();
                    let cb = Closure::once(move |perm: JsValue| {
                        if perm.as_string().as_deref() == Some("granted") {
                            let options = web_sys::NotificationOptions::new();
                            if !body.is_empty() {
                                options.set_body(&body);
                            }
                            if !icon.is_empty() {
                                let _ = js_sys::Reflect::set(
                                    &options,
                                    &JsValue::from_str("icon"),
                                    &JsValue::from_str(&icon),
                                );
                            }
                            let _ = web_sys::Notification::new_with_options(&title, &options);
                        }
                    });
                    if let Ok(promise) = web_sys::Notification::request_permission() {
                        let _ = promise.then(&cb);
                    }
                    cb.forget();
                }
            }
            false
        }
        IrAction::SetTheme { name } => {
            // Find the requested theme and update all theme.* state entries
            if let Some(theme) = themes.iter().find(|t| t.name == *name) {
                for (token, color) in &theme.colors {
                    state.insert(
                        format!("theme.colors.{}", token),
                        RenderValue::Color(*color),
                    );
                }
                for (token, value) in &theme.spacing {
                    state.insert(
                        format!("theme.spacing.{}", token),
                        RenderValue::Num(*value, Some("px".into())),
                    );
                }
            }
            state.insert("active-theme".to_string(), RenderValue::Str(name.clone()));
            true
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
        IrAction::SetIndex { target, index, expr } => {
            let idx_val = evaluate_expr(index, state);
            let new_val = evaluate_expr(expr, state);
            let idx = match &idx_val {
                RenderValue::Num(n, _) => *n as usize,
                _ => return false,
            };
            if let Some(RenderValue::List(items)) = state.get(target) {
                let mut new_items = items.clone();
                if idx < new_items.len() {
                    new_items[idx] = new_val;
                    state.insert(target.clone(), RenderValue::List(new_items));
                }
            }
            false
        }
        IrAction::Conditional { condition, then_actions, else_actions } => {
            let cond = evaluate_expr(condition, state);
            let truthy = match &cond {
                RenderValue::Bool(b) => *b,
                RenderValue::Str(s) => !s.is_empty(),
                RenderValue::Num(n, _) => *n != 0.0,
                _ => false,
            };
            let actions = if truthy { then_actions } else { else_actions };
            let mut navigated = false;
            for action in actions {
                if execute_action(action, state, themes) {
                    navigated = true;
                }
            }
            navigated
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
        RenderValue::InterpolatedStr(parts) => parts
            .iter()
            .map(|p| match p {
                naze_ir::TextPart::Literal(s) => s.clone(),
                naze_ir::TextPart::StateRef(name) => format!("{{{}}}", name),
            })
            .collect(),
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
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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

// ─── Storage persistence ─────────────────────────────────────────────────────

/// Initialize storage-backed state variables by reading from web storage.
/// Falls back to the declared default value if storage is empty or unreadable.
/// Initialize URL parameter state from query string, falling back to defaults.
fn init_params(param_decls: &[ParamDecl], state: &mut HashMap<String, RenderValue>) {
    if param_decls.is_empty() {
        return;
    }

    // Parse query string from URL
    let query_params: HashMap<String, String> = if let Some(window) = web_sys::window() {
        if let Ok(search) = window.location().search() {
            let search = search.trim_start_matches('?');
            search
                .split('&')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next()?;
                    let val = parts.next().unwrap_or("");
                    if key.is_empty() {
                        None
                    } else {
                        Some((key.to_string(), val.to_string()))
                    }
                })
                .collect()
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    for decl in param_decls {
        let value = if let Some(raw) = query_params.get(&decl.name) {
            // Parse URL value according to declared type
            match decl.param_type.as_str() {
                "number" => {
                    if let Ok(n) = raw.parse::<f64>() {
                        RenderValue::Num(n, None)
                    } else {
                        decl.default.clone()
                    }
                }
                "bool" => RenderValue::Bool(raw == "true" || raw == "1"),
                _ => RenderValue::Str(raw.clone()),
            }
        } else {
            decl.default.clone()
        };
        state.insert(decl.name.clone(), value);
    }
}

fn init_storage(storage_decls: &[StorageDecl], state: &mut HashMap<String, RenderValue>) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    for decl in storage_decls {
        let store = if decl.storage_type == 0 {
            window.local_storage().ok().flatten()
        } else {
            window.session_storage().ok().flatten()
        };
        let value = store
            .and_then(|s| s.get_item(&decl.key).ok().flatten())
            .and_then(|raw| parse_storage_value(&raw))
            .unwrap_or_else(|| decl.default.clone());
        state.insert(decl.name.clone(), value);
    }
}

/// Persist storage-backed state variables after a state change.
fn persist_storage(storage_decls: &[StorageDecl], state: &HashMap<String, RenderValue>) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    for decl in storage_decls {
        if let Some(value) = state.get(&decl.name) {
            let store = if decl.storage_type == 0 {
                window.local_storage().ok().flatten()
            } else {
                window.session_storage().ok().flatten()
            };
            if let Some(s) = store {
                let raw = storage_value_to_string(value);
                let _ = s.set_item(&decl.key, &raw);
            }
        }
    }
}

fn parse_storage_value(raw: &str) -> Option<RenderValue> {
    if raw == "true" {
        Some(RenderValue::Bool(true))
    } else if raw == "false" {
        Some(RenderValue::Bool(false))
    } else if let Ok(n) = raw.parse::<f64>() {
        Some(RenderValue::Num(n, None))
    } else {
        Some(RenderValue::Str(raw.to_string()))
    }
}

fn storage_value_to_string(value: &RenderValue) -> String {
    match value {
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
        _ => String::new(),
    }
}

// ─── Timers ──────────────────────────────────────────────────────────────────

/// Initialize a device API data source (geolocation, accelerometer).
fn init_device_data(name: &str, api: &str, watch: bool) {
    match api {
        "geolocation" => init_geolocation(name, watch),
        "accelerometer" => init_accelerometer(name, watch),
        _ => {
            // Unknown device API — set error state
            let err_key = state_key(&name, "error");
            let loading_key = state_key(&name, "loading");
            APP.with(|cell| {
                let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                if let Some(app) = borrow.as_mut() {
                    app.state_store.insert(
                        err_key,
                        RenderValue::Str(format!("Unsupported device API: {}", api)),
                    );
                    app.state_store
                        .insert(loading_key, RenderValue::Bool(false));
                }
            });
            schedule_render();
        }
    }
}

/// Initialize geolocation data source.
fn init_geolocation(name: &str, watch: bool) {
    let name = name.to_string();
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let navigator = window.navigator();
    let geolocation = match navigator.geolocation() {
        Ok(g) => g,
        Err(_) => {
            let err_key = state_key(&name, "error");
            let loading_key = state_key(&name, "loading");
            APP.with(|cell| {
                let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                if let Some(app) = borrow.as_mut() {
                    app.state_store.insert(
                        err_key,
                        RenderValue::Str("Geolocation not supported".into()),
                    );
                    app.state_store
                        .insert(loading_key, RenderValue::Bool(false));
                }
            });
            schedule_render();
            return;
        }
    };

    let name_ok = name.clone();
    let success_cb = Closure::<dyn Fn(web_sys::Position)>::new(move |pos: web_sys::Position| {
        let coords = pos.coords();
        let data = RenderValue::Object(vec![
            (
                "latitude".to_string(),
                RenderValue::Num(coords.latitude(), None),
            ),
            (
                "longitude".to_string(),
                RenderValue::Num(coords.longitude(), None),
            ),
            (
                "accuracy".to_string(),
                RenderValue::Num(coords.accuracy(), None),
            ),
        ]);
        let data_key = state_key(&name_ok, "data");
        let loading_key = state_key(&name_ok, "loading");
        let err_key = state_key(&name_ok, "error");
        APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
            if let Some(app) = borrow.as_mut() {
                app.state_store.insert(data_key, data);
                app.state_store
                    .insert(loading_key, RenderValue::Bool(false));
                app.state_store
                    .insert(err_key, RenderValue::Str(String::new()));
            }
        });
        schedule_render();
    });

    let name_err = name.clone();
    let error_cb =
        Closure::<dyn Fn(web_sys::PositionError)>::new(move |err: web_sys::PositionError| {
            let msg = match err.code() {
                1 => "Geolocation permission denied".to_string(),
                2 => "Position unavailable".to_string(),
                3 => "Geolocation timeout".to_string(),
                _ => format!("Geolocation error: {}", err.message()),
            };
            let err_key = state_key(&name_err, "error");
            let loading_key = state_key(&name_err, "loading");
            APP.with(|cell| {
                let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                if let Some(app) = borrow.as_mut() {
                    app.state_store.insert(err_key, RenderValue::Str(msg));
                    app.state_store
                        .insert(loading_key, RenderValue::Bool(false));
                }
            });
            schedule_render();
        });

    if watch {
        let _ = geolocation.watch_position_with_error_callback(
            success_cb.as_ref().unchecked_ref(),
            Some(error_cb.as_ref().unchecked_ref()),
        );
    } else {
        let _ = geolocation.get_current_position_with_error_callback(
            success_cb.as_ref().unchecked_ref(),
            Some(error_cb.as_ref().unchecked_ref()),
        );
    }
    success_cb.forget();
    error_cb.forget();
}

/// Initialize accelerometer/device motion data source.
fn init_accelerometer(name: &str, watch: bool) {
    let name = name.to_string();
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    let name_cb = name.clone();
    let once = !watch;
    let motion_cb = Closure::<dyn Fn(web_sys::DeviceMotionEvent)>::new(
        move |event: web_sys::DeviceMotionEvent| {
            // Access accelerationIncludingGravity via Reflect for broad browser compat
            let accel_val = js_sys::Reflect::get(
                event.as_ref(),
                &JsValue::from_str("accelerationIncludingGravity"),
            )
            .unwrap_or(JsValue::NULL);
            let (x, y, z) = if !accel_val.is_null() && !accel_val.is_undefined() {
                let gx = js_sys::Reflect::get(&accel_val, &JsValue::from_str("x"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let gy = js_sys::Reflect::get(&accel_val, &JsValue::from_str("y"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let gz = js_sys::Reflect::get(&accel_val, &JsValue::from_str("z"))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                (gx, gy, gz)
            } else {
                (0.0, 0.0, 0.0)
            };

            let data = RenderValue::Object(vec![
                ("x".to_string(), RenderValue::Num(x, None)),
                ("y".to_string(), RenderValue::Num(y, None)),
                ("z".to_string(), RenderValue::Num(z, None)),
            ]);
            let data_key = state_key(&name_cb, "data");
            let loading_key = state_key(&name_cb, "loading");
            let err_key = state_key(&name_cb, "error");
            APP.with(|cell| {
                let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                if let Some(app) = borrow.as_mut() {
                    app.state_store.insert(data_key, data);
                    app.state_store
                        .insert(loading_key, RenderValue::Bool(false));
                    app.state_store
                        .insert(err_key, RenderValue::Str(String::new()));
                }
            });
            schedule_render();

            // For one-shot mode, remove the listener after first event
            if once {
                // For one-shot mode, we rely on the closure not scheduling
                // further renders. Removing the listener would require storing
                // the closure reference, which isn't worth the complexity.
            }
        },
    );

    let _ =
        window.add_event_listener_with_callback("devicemotion", motion_cb.as_ref().unchecked_ref());
    motion_cb.forget();
}

/// Initialize a JS call data source (source_type == 3).
fn init_js_call_data(name: &str, func_name: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    // Resolve dotted function path (e.g. "Math.random")
    let parts: Vec<&str> = func_name.split('.').collect();
    let mut obj: JsValue = window.into();
    let mut found = true;
    for part in &parts[..parts.len().saturating_sub(1)] {
        match js_sys::Reflect::get(&obj, &JsValue::from_str(part)) {
            Ok(next) if !next.is_undefined() => obj = next,
            _ => {
                found = false;
                break;
            }
        }
    }

    let data_key = state_key(&name, "data");
    let loading_key = state_key(&name, "loading");
    let err_key = state_key(&name, "error");

    if found {
        let fn_name = parts.last().unwrap_or(&"");
        if let Ok(val) = js_sys::Reflect::get(&obj, &JsValue::from_str(fn_name)) {
            if let Ok(func) = val.dyn_into::<js_sys::Function>() {
                match func.apply(&obj, &js_sys::Array::new()) {
                    Ok(result) => {
                        let naze_val = if let Some(s) = result.as_string() {
                            RenderValue::Str(s)
                        } else if let Some(n) = result.as_f64() {
                            RenderValue::Num(n, None)
                        } else if let Some(b) = result.as_bool() {
                            RenderValue::Bool(b)
                        } else {
                            RenderValue::Str("null".to_string())
                        };
                        APP.with(|cell| {
                            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                            if let Some(app) = borrow.as_mut() {
                                app.state_store.insert(data_key, naze_val);
                                app.state_store
                                    .insert(loading_key, RenderValue::Bool(false));
                                app.state_store
                                    .insert(err_key, RenderValue::Str(String::new()));
                            }
                        });
                        schedule_render();
                        return;
                    }
                    Err(e) => {
                        let msg = format!("JS call error: {:?}", e);
                        APP.with(|cell| {
                            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                            if let Some(app) = borrow.as_mut() {
                                app.state_store.insert(err_key, RenderValue::Str(msg));
                                app.state_store
                                    .insert(loading_key, RenderValue::Bool(false));
                            }
                        });
                        schedule_render();
                        return;
                    }
                }
            }
        }
    }

    // Function not found
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        if let Some(app) = borrow.as_mut() {
            app.state_store.insert(
                err_key,
                RenderValue::Str(format!("JS function not found: {}", func_name)),
            );
            app.state_store
                .insert(loading_key, RenderValue::Bool(false));
        }
    });
    schedule_render();
}

/// Set up all timer declarations (setTimeout / setInterval).
fn setup_timers() {
    let timer_decls: Vec<TimerDecl> = APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return vec![] };
        borrow
            .as_ref()
            .map(|app| app.render_tree.timers.clone())
            .unwrap_or_default()
    });

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    for decl in timer_decls {
        let action = decl.action.clone();
        let cb = Closure::<dyn Fn()>::new(move || {
            let needs_render = APP.with(|cell| {
                let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                if let Some(app) = borrow.as_mut() {
                    execute_action(&action, &mut app.state_store, &app.render_tree.themes)
                } else {
                    false
                }
            });
            if needs_render {
                schedule_render();
            }
        });

        if decl.kind == 0 {
            // after → setTimeout
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                decl.duration_ms as i32,
            );
        } else {
            // every → setInterval
            let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                decl.duration_ms as i32,
            );
        }
        cb.forget(); // Leak the closure so it persists
    }
}

// ─── Computed values ─────────────────────────────────────────────────────────

/// Re-evaluate all computed declarations and update the state store.
/// Computed values are evaluated in declaration order; a computed value
/// can reference state variables and earlier computed values.
fn recompute_computed(computed: &[ComputedDecl], state: &mut HashMap<String, RenderValue>) {
    for decl in computed {
        let value = evaluate_expr(&decl.expr, state);
        state.insert(decl.name.clone(), value);
    }
}

// ─── Expression evaluation ───────────────────────────────────────────────────

/// Evaluate an expression against the current state.
fn evaluate_expr(expr: &IrExpression, state: &HashMap<String, RenderValue>) -> RenderValue {
    match expr {
        IrExpression::Num(n) => RenderValue::Num(*n, None),
        IrExpression::Str(s) => RenderValue::Str(s.clone()),
        IrExpression::Bool(b) => RenderValue::Bool(*b),
        IrExpression::StateRef(name) => {
            if let Some(val) = state.get(name) {
                return val.clone();
            }
            // Dotted field access: "obj.field" -> lookup obj, extract field
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
        IrExpression::WasmCall {
            module,
            function,
            args,
        } => {
            let arg_vals: Vec<RenderValue> = args.iter().map(|a| evaluate_expr(a, state)).collect();
            call_wasm_import(module, function, &arg_vals)
        }
        IrExpression::EnvRef(_) => {
            // Env refs are resolved at compile time for client code;
            // this branch should not be reached in the WASM runtime.
            RenderValue::Str(String::new())
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
        IrExpression::Index { list, index } => {
            let list_val = evaluate_expr(list, state);
            let idx_val = evaluate_expr(index, state);
            let idx = match &idx_val {
                RenderValue::Num(n, _) => *n as usize,
                _ => return RenderValue::Str(String::new()),
            };
            match &list_val {
                RenderValue::List(items) => {
                    items.get(idx).cloned().unwrap_or(RenderValue::Str(String::new()))
                }
                _ => RenderValue::Str(String::new()),
            }
        }
        IrExpression::FunctionCall { name, args } => {
            match name.as_str() {
                "length" => {
                    if let Some(arg) = args.first() {
                        let val = evaluate_expr(arg, state);
                        match &val {
                            RenderValue::List(items) => RenderValue::Num(items.len() as f64, None),
                            RenderValue::Str(s) => RenderValue::Num(s.len() as f64, None),
                            _ => RenderValue::Num(0.0, None),
                        }
                    } else {
                        RenderValue::Num(0.0, None)
                    }
                }
                "random" => {
                    let min = args.first().map(|a| {
                        match evaluate_expr(a, state) {
                            RenderValue::Num(n, _) => n,
                            _ => 0.0,
                        }
                    }).unwrap_or(0.0);
                    let max = args.get(1).map(|a| {
                        match evaluate_expr(a, state) {
                            RenderValue::Num(n, _) => n,
                            _ => 1.0,
                        }
                    }).unwrap_or(1.0);
                    let r = js_sys::Math::random();
                    let val = min + (r * (max - min + 1.0)).floor();
                    let val = val.min(max); // clamp to max
                    RenderValue::Num(val, None)
                }
                _ => RenderValue::Str(String::new()),
            }
        }
    }
}

/// Call an imported WASM module function via the JS bridge.
/// Invokes `window.__naze_wasm_call(module, func, args)` which is set up
/// by the generated `wasm_imports.js` loader script.
fn call_wasm_import(module: &str, func: &str, args: &[RenderValue]) -> RenderValue {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return RenderValue::Num(0.0, None),
    };
    let call_fn = js_sys::Reflect::get(&window, &JsValue::from_str("__naze_wasm_call"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Function>().ok());
    let Some(call_fn) = call_fn else {
        web_sys::console::error_1(&JsValue::from_str("WASM import bridge not found"));
        return RenderValue::Num(0.0, None);
    };
    let js_args = js_sys::Array::new();
    js_args.push(&JsValue::from_str(module));
    js_args.push(&JsValue::from_str(func));
    let js_arg_array = js_sys::Array::new();
    for arg in args {
        match arg {
            RenderValue::Num(n, _) => {
                js_arg_array.push(&JsValue::from_f64(*n));
            }
            RenderValue::Str(s) => {
                js_arg_array.push(&JsValue::from_str(s));
            }
            RenderValue::Bool(b) => {
                js_arg_array.push(&JsValue::from_bool(*b));
            }
            _ => {
                js_arg_array.push(&JsValue::from_f64(0.0));
            }
        }
    }
    js_args.push(&js_arg_array);
    match call_fn.apply(&JsValue::NULL, &js_args) {
        Ok(result) => {
            if let Some(n) = result.as_f64() {
                RenderValue::Num(n, None)
            } else if let Some(s) = result.as_string() {
                RenderValue::Str(s)
            } else {
                RenderValue::Num(0.0, None)
            }
        }
        Err(_) => RenderValue::Num(0.0, None),
    }
}

/// Evaluate a pipeline by applying stages sequentially to a source value.
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

/// Evaluate a single pipeline stage.
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
            // filter: keep items where predicate is true
            let arg = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            let filtered: Vec<RenderValue> = items
                .into_iter()
                .filter(|item| {
                    let item_state = build_item_state(item, state);
                    matches!(evaluate_expr(arg, &item_state), RenderValue::Bool(true))
                })
                .collect();
            RenderValue::List(filtered)
        }
        1 => {
            // map: transform each item
            let arg = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            let mapped: Vec<RenderValue> = items
                .into_iter()
                .map(|item| {
                    let item_state = build_item_state(&item, state);
                    evaluate_expr(arg, &item_state)
                })
                .collect();
            RenderValue::List(mapped)
        }
        2 => {
            // sort-by: sort items by key expression
            let arg = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            let mut sorted = items;
            sorted.sort_by(|a, b| {
                let a_state = build_item_state(a, state);
                let b_state = build_item_state(b, state);
                let a_key = evaluate_expr(arg, &a_state);
                let b_key = evaluate_expr(arg, &b_state);
                compare_render_values(&a_key, &b_key)
            });
            RenderValue::List(sorted)
        }
        3 => {
            // take: keep first N items
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
            // sum: reduce to numeric total
            let mut total = 0.0f64;
            for item in &items {
                match item {
                    RenderValue::Num(n, _) => total += n,
                    _ => {}
                }
            }
            RenderValue::Num(total, None)
        }
        5 => {
            // count: return list length
            RenderValue::Num(items.len() as f64, None)
        }
        6 => {
            // reduce: fold list with accumulator expression and initial value
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
                let mut item_state = build_item_state(item, state);
                item_state.insert("acc".to_string(), acc.clone());
                acc = evaluate_expr(acc_expr, &item_state);
            }
            acc
        }
        7 => {
            // group-by: group items into object of lists keyed by field value
            let arg = match &stage.argument {
                Some(a) => a,
                None => return RenderValue::List(items),
            };
            let mut groups: Vec<(String, Vec<RenderValue>)> = Vec::new();
            for item in items {
                let item_state = build_item_state(&item, state);
                let key = render_value_to_string(&evaluate_expr(arg, &item_state));
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
            // flatten: flatten nested lists one level
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
            // distinct: unique items, optionally by field
            let mut seen = Vec::new();
            let mut result = Vec::new();
            for item in items {
                let key = match &stage.argument {
                    Some(arg) => {
                        let item_state = build_item_state(&item, state);
                        render_value_to_string(&evaluate_expr(arg, &item_state))
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
        10 => {
            // shuffle: Fisher-Yates using js_sys::Math::random()
            let mut shuffled = items;
            let len = shuffled.len();
            for i in (1..len).rev() {
                let j = (js_sys::Math::random() * (i as f64 + 1.0)).floor() as usize;
                shuffled.swap(i, j);
            }
            RenderValue::List(shuffled)
        }
        _ => RenderValue::List(items),
    }
}

/// Build a state map for evaluating expressions within a pipeline stage.
/// Object fields are injected as top-level keys so `score` resolves to `item.score`.
fn build_item_state(
    item: &RenderValue,
    parent_state: &HashMap<String, RenderValue>,
) -> HashMap<String, RenderValue> {
    let mut item_state = parent_state.clone();
    item_state.insert("__it".to_string(), item.clone());
    if let RenderValue::Object(entries) = item {
        for (k, v) in entries {
            item_state.insert(k.clone(), v.clone());
        }
    }
    // For non-object items, also make __it accessible as the value
    item_state
}

/// Compare two RenderValues for sorting purposes.
fn compare_render_values(a: &RenderValue, b: &RenderValue) -> std::cmp::Ordering {
    match (a, b) {
        (RenderValue::Num(an, _), RenderValue::Num(bn, _)) => {
            an.partial_cmp(bn).unwrap_or(std::cmp::Ordering::Equal)
        }
        (RenderValue::Str(as_), RenderValue::Str(bs)) => as_.cmp(bs),
        (RenderValue::Bool(ab), RenderValue::Bool(bb)) => ab.cmp(bb),
        _ => std::cmp::Ordering::Equal,
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
            } else if let (RenderValue::List(ll), RenderValue::List(rl)) = (left, right) {
                let mut result = ll.clone();
                result.extend(rl.iter().cloned());
                RenderValue::List(result)
            } else {
                // String concatenation fallback
                let ls = render_value_to_string(left);
                let rs = render_value_to_string(right);
                RenderValue::Str(format!("{}{}", ls, rs))
            }
        }
        IrBinOp::Sub => RenderValue::Num(left_num.unwrap_or(0.0) - right_num.unwrap_or(0.0), None),
        IrBinOp::Mul => RenderValue::Num(left_num.unwrap_or(0.0) * right_num.unwrap_or(0.0), None),
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
const HIDDEN_TEXTAREA_ID: &str = "__naze_hidden_textarea";

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
                            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
                                        execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes);
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
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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

/// Create a hidden HTML textarea element for capturing multi-line keyboard input.
fn create_hidden_textarea(_canvas_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    if document.get_element_by_id(HIDDEN_TEXTAREA_ID).is_some() {
        return Ok(());
    }

    let textarea = document.create_element("textarea")?;
    let textarea: web_sys::HtmlTextAreaElement = textarea.dyn_into()?;
    textarea.set_id(HIDDEN_TEXTAREA_ID);

    // Style to be invisible but focusable
    textarea.style().set_property("position", "absolute")?;
    textarea.style().set_property("left", "-9999px")?;
    textarea.style().set_property("top", "0")?;
    textarea.style().set_property("opacity", "0")?;
    textarea.style().set_property("pointer-events", "none")?;

    document.body().ok_or("no body")?.append_child(&textarea)?;

    // Input event listener — same logic as hidden input
    let input_cb = Closure::<dyn Fn(web_sys::Event)>::new(move |_event: web_sys::Event| {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(el) = document.get_element_by_id(HIDDEN_TEXTAREA_ID) {
                    if let Ok(ta) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                        let value = ta.value();
                        let changed = APP.with(|cell| {
                            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                            if let Some(app) = borrow.as_mut() {
                                if let Some(ref focused) = app.focused_input.clone() {
                                    app.state_store.insert(
                                        focused.bind_var.clone(),
                                        RenderValue::Str(value.clone()),
                                    );
                                    run_validation(
                                        &mut app.state_store,
                                        &focused.bind_var,
                                        &value,
                                        focused.validate_prop.as_ref(),
                                        &focused.input_type,
                                    );
                                    for handler in &focused.change_handlers {
                                        execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes);
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
    textarea.add_event_listener_with_callback("input", input_cb.as_ref().unchecked_ref())?;
    input_cb.forget();

    // Blur event listener
    let blur_cb = Closure::<dyn Fn(web_sys::Event)>::new(move |_event: web_sys::Event| {
        APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
            if let Some(app) = borrow.as_mut() {
                app.focused_input = None;
                if let Some(id) = app.caret_interval_id.take() {
                    if let Some(window) = web_sys::window() {
                        window.clear_interval_with_handle(id);
                    }
                }
                app.caret_visible = true;
            }
        });
        schedule_render();
    });
    if let Some(el) = document.get_element_by_id(HIDDEN_TEXTAREA_ID) {
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
            // Also blur hidden textarea
            if let Some(el) = document.get_element_by_id(HIDDEN_TEXTAREA_ID) {
                if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                    let _ = textarea.blur();
                }
            }
        }
    }
}

/// Focus an input element, setting up the hidden input with its current value.
fn focus_input(
    bind_var: &str,
    node_id: &str,
    current_value: &str,
    input_type: &str,
    change_handlers: Vec<naze_ir::IrEventHandler>,
    validate_prop: Option<RenderValue>,
) {
    // Update focus state and start caret blink timer
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
                let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
                let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                if let Some(app) = borrow.as_mut() {
                    app.caret_interval_id = Some(id);
                }
            });
        }
        toggle_caret.forget();
    }

    // Focus the appropriate hidden element (textarea or input)
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if input_type == "textarea" {
                // Use hidden textarea for multi-line input
                if let Some(el) = document.get_element_by_id(HIDDEN_TEXTAREA_ID) {
                    if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                        textarea.set_value(current_value);
                        let _ = textarea.focus();
                    }
                }
            } else if let Some(el) = document.get_element_by_id(HIDDEN_INPUT_ID) {
                if let Ok(input) = el.dyn_into::<web_sys::HtmlInputElement>() {
                    input.set_type(input_type);
                    input.set_value(current_value);
                    let _ = input.focus();
                }
            }
        }
    }
}

/// Open a native file picker for file-type inputs.
fn open_file_picker(
    bind_var: &str,
    accept: &str,
    max_size_bytes: u64,
    change_handlers: Vec<naze_ir::IrEventHandler>,
) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    // Create a hidden <input type="file">
    let el = match document.create_element("input") {
        Ok(e) => e,
        Err(_) => return,
    };
    let file_input: web_sys::HtmlInputElement = match el.dyn_into() {
        Ok(i) => i,
        Err(_) => return,
    };
    file_input.set_type("file");
    if !accept.is_empty() {
        file_input.set_attribute("accept", accept).ok();
    }
    let _ = file_input.style().set_property("display", "none");
    if let Some(body) = document.body() {
        let _ = body.append_child(&file_input);
    }

    let bind_var = bind_var.to_string();
    let file_input_clone = file_input.clone();
    let on_change = Closure::<dyn Fn()>::new(move || {
        let files = match file_input_clone.files() {
            Some(f) => f,
            None => return,
        };
        let file = match files.get(0) {
            Some(f) => f,
            None => return,
        };

        let name = file.name();
        let size = file.size() as u64;
        let file_type = file.type_();

        // Check max-size
        if max_size_bytes > 0 && size > max_size_bytes {
            web_sys::console::log_1(
                &format!("File too large: {} bytes (max {})", size, max_size_bytes).into(),
            );
            return;
        }

        // Store file name in bound state variable
        let bind = bind_var.clone();
        let change_h = change_handlers.clone();
        APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
            if let Some(app) = borrow.as_mut() {
                // Store as object with name, size, type
                let obj = vec![
                    ("name".to_string(), RenderValue::Str(name)),
                    ("size".to_string(), RenderValue::Num(size as f64, None)),
                    ("type".to_string(), RenderValue::Str(file_type)),
                ];
                app.state_store
                    .insert(bind.clone(), RenderValue::Object(obj));
                // Execute change handlers
                for handler in &change_h {
                    execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes);
                }
            }
        });
        schedule_render();

        // Clean up the hidden input
        if let Some(parent) = file_input_clone.parent_node() {
            let _ = parent.remove_child(&file_input_clone);
        }
    });

    file_input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
    on_change.forget();

    // Trigger the file dialog
    file_input.click();
}

/// Find an input node at the given point. Returns (bind_var, node_id, current_value, input_type, change_handlers, validate_prop) if found.
fn find_input_at_point(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
    state: &HashMap<String, RenderValue>,
) -> Option<(
    String,
    String,
    String,
    String,
    Vec<naze_ir::IrEventHandler>,
    Option<RenderValue>,
)> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first
        if let Some(result) = find_input_at_point(&node.children, x, y, state) {
            return Some(result);
        }
        // Check if this is an input or textarea
        if node.kind == "input" || node.kind == "textarea" {
            if let Some(RenderValue::Bind(bind_var)) = node.props.get("bind") {
                let node_id = format!("input_{}_{}", node.x as i32, node.y as i32);
                let current_value = match state.get(bind_var) {
                    Some(RenderValue::Str(s)) => s.clone(),
                    _ => String::new(),
                };
                let input_type = if node.kind == "textarea" {
                    "textarea".to_string()
                } else {
                    naze_renderer::get_str_prop(&node.props, "type", "text")
                };
                // Extract change handlers
                let change_handlers: Vec<_> = node
                    .handlers
                    .iter()
                    .filter(|h| h.event == "change")
                    .cloned()
                    .collect();
                // Extract validate prop
                let validate_prop = node.props.get("validate").cloned();
                return Some((
                    bind_var.clone(),
                    node_id,
                    current_value,
                    input_type,
                    change_handlers,
                    validate_prop,
                ));
            }
        }
    }
    None
}

/// Find a string prop on the input node at a given point.
fn find_input_prop_at_point(nodes: &[PositionedNode], x: f32, y: f32, prop: &str) -> String {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        if let Some(result) = find_input_prop_at_point_inner(&node.children, x, y, prop) {
            return result;
        }
        if node.kind == "input" {
            return naze_renderer::get_str_prop(&node.props, prop, "").to_string();
        }
    }
    String::new()
}

fn find_input_prop_at_point_inner(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
    prop: &str,
) -> Option<String> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        if let Some(r) = find_input_prop_at_point_inner(&node.children, x, y, prop) {
            return Some(r);
        }
        if node.kind == "input" {
            return Some(naze_renderer::get_str_prop(&node.props, prop, "").to_string());
        }
    }
    None
}

/// Parse a max-size string like "5mb", "500kb" into bytes.
fn parse_max_size(s: &str) -> u64 {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return 0;
    }
    if let Some(rest) = s.strip_suffix("gb") {
        rest.trim().parse::<u64>().unwrap_or(0) * 1024 * 1024 * 1024
    } else if let Some(rest) = s.strip_suffix("mb") {
        rest.trim().parse::<u64>().unwrap_or(0) * 1024 * 1024
    } else if let Some(rest) = s.strip_suffix("kb") {
        rest.trim().parse::<u64>().unwrap_or(0) * 1024
    } else if let Some(rest) = s.strip_suffix('b') {
        rest.trim().parse::<u64>().unwrap_or(0)
    } else {
        s.parse::<u64>().unwrap_or(0)
    }
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
fn find_option_at_point(
    nodes: &[PositionedNode],
    x: f32,
    y: f32,
    open_select_id: &str,
) -> Option<(String, String, Vec<naze_ir::IrEventHandler>)> {
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
                let change_handlers: Vec<_> = node
                    .handlers
                    .iter()
                    .filter(|h| h.event == "change")
                    .cloned()
                    .collect();
                // Check if point is in the dropdown area (below the select box)
                let dropdown_y = node.y + node.height;
                let option_height = 36.0_f32;
                for (i, child) in node.children.iter().enumerate() {
                    if child.kind == "option" {
                        let opt_y = dropdown_y + (i as f32 * option_height);
                        if x >= node.x
                            && x <= node.x + node.width
                            && y >= opt_y
                            && y <= opt_y + option_height
                        {
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
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
                execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes);
            }

            return true; // Needs render for visual feedback
        }

        false
    })
}

/// Handle mousemove event. Updates drag position, hover state, or cursor.
fn handle_mousemove(x: f32, y: f32) -> bool {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
                        execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes);
                    }
                }
                drag.over_target_id = new_target_id;
            }

            // Set cursor to grabbing
            set_cursor("grabbing");
            return true; // Needs render to update ghost position
        }

        // Fire pointer-move handlers on nodes under the mouse
        let mut needs_render = false;
        // Check overlays first, then root
        let pointer_move_nodes: &[PositionedNode] = {
            let mut found_in_overlay = false;
            for overlay in layout.overlays.iter().rev() {
                if point_in_node(overlay, x, y) {
                    found_in_overlay = true;
                    break;
                }
            }
            if found_in_overlay {
                // For pointer-move in overlays, search overlay children
                &[] // handled below per-overlay
            } else {
                &layout.root
            }
        };
        // Fire pointer-move on overlay children if applicable
        for overlay in layout.overlays.iter().rev() {
            if point_in_node(overlay, x, y) {
                let pm_handlers =
                    find_event_handlers_at_point(&overlay.children, x, y, "pointer-move");
                for handler in &pm_handlers {
                    if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                        needs_render = true;
                    }
                }
                break;
            }
        }
        // Fire pointer-move on root nodes
        if !pointer_move_nodes.is_empty() {
            let pm_handlers = find_event_handlers_at_point(&layout.root, x, y, "pointer-move");
            for handler in &pm_handlers {
                if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                    needs_render = true;
                }
            }
        }

        // Track hover state - find deepest element with hover handlers
        // Check overlays first
        let hover_info = {
            let mut result = None;
            for overlay in layout.overlays.iter().rev() {
                if point_in_node(overlay, x, y) {
                    result = find_hover_element(&overlay.children, x, y);
                    break;
                }
            }
            if result.is_none() {
                result = find_hover_element(&layout.root, x, y);
            }
            result
        };
        let new_hover_id = hover_info.as_ref().map(|(id, _)| id.clone());

        if new_hover_id != app.hovered_element_id {
            // Fire hover handlers when entering a new element
            if let Some((_, handlers)) = hover_info {
                for handler in &handlers {
                    if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                        needs_render = true;
                    }
                }
            }
            app.hovered_element_id = new_hover_id;
        }

        // Update cursor based on what's under the mouse
        // Priority: explicit cursor prop > draggable > clickable > default
        let explicit_cursor = find_cursor_prop_at_point(&layout.overlays, x, y)
            .or_else(|| find_cursor_prop_at_point(&layout.root, x, y));

        if let Some(cursor) = explicit_cursor {
            set_cursor(&cursor);
        } else {
            let has_draggable = find_draggable_at_point(&layout.root, x, y).is_some();
            let has_clickable = hit_test_any_handler_with_overlays(&layout, x, y, "click");

            if has_draggable {
                set_cursor("grab");
            } else if has_clickable {
                set_cursor("pointer");
            } else {
                set_cursor("default");
            }
        }

        needs_render
    })
}

/// Handle mouseup event. Completes drop or triggers click.
fn handle_mouseup(x: f32, y: f32) -> bool {
    let was_dragging = APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return false };
        if let Some(app) = borrow.as_ref() {
            return app.drag_state.is_some();
        }
        false
    });

    if was_dragging {
        // Complete drag operation
        return APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
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
                        execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes);
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
            let drag_data = node
                .props
                .get("drag-data")
                .cloned()
                .unwrap_or(RenderValue::Str(String::new()));
            let drag_start_handlers: Vec<_> = node
                .handlers
                .iter()
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
            let drag_over_handlers: Vec<_> = node
                .handlers
                .iter()
                .filter(|h| h.event == "drag-over")
                .cloned()
                .collect();
            let drop_handlers: Vec<_> = node
                .handlers
                .iter()
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

/// Find the cursor prop on the deepest node at the given point.
fn find_cursor_prop_at_point(nodes: &[PositionedNode], x: f32, y: f32) -> Option<String> {
    for node in nodes.iter().rev() {
        if !point_in_node(node, x, y) {
            continue;
        }
        // Check children first (deeper wins)
        if let Some(result) = find_cursor_prop_at_point(&node.children, x, y) {
            return Some(result);
        }
        // Check this node's cursor prop
        if let Some(RenderValue::Str(cursor)) = node.props.get("cursor") {
            return Some(cursor.clone());
        }
    }
    None
}

/// Set the cursor style on the canvas.
fn set_cursor(cursor: &str) {
    APP.with(|cell| {
        let borrow = match cell.try_borrow() { Ok(b) => b, Err(_) => return };
        if let Some(app) = borrow.as_ref() {
            let _ = app
                .renderer
                .canvas_element()
                .style()
                .set_property("cursor", cursor);
        }
    });
}

// ─── Scroll Handling ─────────────────────────────────────────────────────────

/// Apply scroll delta to a named scroll container. Returns true if scroll changed.
/// Shared by wheel and touch handlers.
fn apply_scroll_delta(
    app: &mut App,
    layout: &LayoutTree,
    scroll_id: &str,
    delta_x: f32,
    delta_y: f32,
) -> bool {
    // Find the scroll container's info by walking the layout tree
    let scroll_info_bounds = find_scroll_by_id(&layout.root, scroll_id);
    let (scroll_info, bounds) = match scroll_info_bounds {
        Some(v) => v,
        None => return false,
    };
    let (_, _, container_w, container_h) = bounds;

    let max_scroll_x = (scroll_info.content_width - container_w).max(0.0);
    let max_scroll_y = (scroll_info.content_height - container_h).max(0.0);

    let state = app.scroll_states.entry(scroll_id.to_string()).or_default();
    let mut changed = false;

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

    if changed {
        if let Some(handlers) = find_scroll_handlers(&layout.root, scroll_id) {
            for handler in &handlers {
                execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes);
            }
        }
    }

    changed
}

/// Find a scroll container by its ID, returning (ScrollInfo, bounds).
fn find_scroll_by_id(
    nodes: &[PositionedNode],
    target_id: &str,
) -> Option<(ScrollInfo, (f32, f32, f32, f32))> {
    for node in nodes {
        if node.kind == "scroll" {
            if let Some(ref info) = node.scroll_info {
                let scroll_id = format!("scroll_{}_{}", node.x as i32, node.y as i32);
                if scroll_id == target_id {
                    return Some((info.clone(), (node.x, node.y, node.width, node.height)));
                }
            }
        }
        if let Some(result) = find_scroll_by_id(&node.children, target_id) {
            return Some(result);
        }
    }
    None
}

/// Handle wheel event for scrolling.
fn handle_wheel(x: f32, y: f32, delta_x: f32, delta_y: f32) -> bool {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return false,
        };

        // Check if any visible overlay has scroll-lock: true — suppress scrolling
        for overlay in layout.overlays.iter().rev() {
            if let Some(RenderValue::Bool(true)) = overlay.props.get("scroll-lock") {
                return false;
            }
        }

        if let Some((scroll_id, _scroll_info, _bounds)) = find_scroll_at_point(&layout.root, x, y) {
            return apply_scroll_delta(app, &layout, &scroll_id, delta_x, delta_y);
        }

        false
    })
}

/// Handle touchstart: find scroll container at touch point, store state.
fn handle_touchstart(x: f32, y: f32, identifier: i32) -> bool {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return false,
        };

        // Check scroll-lock
        for overlay in layout.overlays.iter().rev() {
            if let Some(RenderValue::Bool(true)) = overlay.props.get("scroll-lock") {
                return false;
            }
        }

        if let Some((scroll_id, _info, _bounds)) = find_scroll_at_point(&layout.root, x, y) {
            app.touch_start_x = x;
            app.touch_start_y = y;
            app.touch_scroll_id = Some(scroll_id);
            app.touch_identifier = Some(identifier);
            return true; // We're capturing this touch
        }

        // Clear touch state if not in a scroll container
        app.touch_scroll_id = None;
        app.touch_identifier = None;
        false
    })
}

/// Handle touchmove: compute delta, apply scroll.
fn handle_touchmove(x: f32, y: f32, identifier: i32) -> bool {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };

        // Only handle the touch we're tracking
        if app.touch_identifier != Some(identifier) || app.touch_scroll_id.is_none() {
            return false;
        }

        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return false,
        };

        let scroll_id = app.touch_scroll_id.clone().unwrap();
        // Touch scroll is inverted: dragging down scrolls up (negative delta)
        let delta_x = app.touch_start_x - x;
        let delta_y = app.touch_start_y - y;

        // Update start position for next move event
        app.touch_start_x = x;
        app.touch_start_y = y;

        apply_scroll_delta(app, &layout, &scroll_id, delta_x, delta_y)
    })
}

/// Handle touchend/touchcancel: clear touch state.
fn handle_touchend(identifier: i32) {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        if let Some(app) = borrow.as_mut() {
            if app.touch_identifier == Some(identifier) {
                app.touch_scroll_id = None;
                app.touch_identifier = None;
            }
        }
    });
}

/// Find scroll handlers for a scroll container by its ID.
fn find_scroll_handlers(
    nodes: &[PositionedNode],
    scroll_id: &str,
) -> Option<Vec<naze_ir::IrEventHandler>> {
    for node in nodes {
        // Check if this is the scroll container
        if node.kind == "scroll" {
            let this_id = format!("scroll_{}_{}", node.x as i32, node.y as i32);
            if this_id == scroll_id {
                let handlers: Vec<_> = node
                    .handlers
                    .iter()
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
                return Some((
                    scroll_id,
                    info.clone(),
                    (node.x, node.y, node.width, node.height),
                ));
            }
        }
    }
    None
}

/// Scroll to bring an element with the given ID into view.
/// The element_id should match an element's `id` prop.
fn scroll_to_element(element_id: &str) {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return,
        };

        // Find the element by ID and its containing scroll container
        if let Some((element_y, scroll_id, scroll_info, container_y, container_h)) =
            find_element_and_scroll_container(&layout.root, element_id, None)
        {
            // Calculate the scroll offset to bring element into view
            let relative_y = element_y - container_y;

            // Get or create scroll state
            let state = app.scroll_states.entry(scroll_id).or_default();

            // Scroll so the element is at the top of the container
            let max_scroll = (scroll_info.content_height - container_h).max(0.0);
            state.scroll_y = relative_y.min(max_scroll).max(0.0);
        }
    });
}

/// Find an element by ID and its containing scroll container.
/// Returns (element_y, scroll_id, scroll_info, container_y, container_h) if found.
fn find_element_and_scroll_container(
    nodes: &[PositionedNode],
    element_id: &str,
    current_scroll: Option<(String, ScrollInfo, f32, f32)>,
) -> Option<(f32, String, ScrollInfo, f32, f32)> {
    for node in nodes {
        // Update current scroll container if this is a scroll node
        let scroll_context = if node.kind == "scroll" {
            if let Some(ref info) = node.scroll_info {
                let scroll_id = format!("scroll_{}_{}", node.x as i32, node.y as i32);
                Some((scroll_id, info.clone(), node.y, node.height))
            } else {
                current_scroll.clone()
            }
        } else {
            current_scroll.clone()
        };

        // Check if this node has the target ID
        if let Some(RenderValue::Str(id)) = node.props.get("id") {
            if id == element_id {
                if let Some((scroll_id, scroll_info, container_y, container_h)) = scroll_context {
                    return Some((node.y, scroll_id, scroll_info, container_y, container_h));
                }
            }
        }

        // Recurse into children
        if let Some(result) =
            find_element_and_scroll_container(&node.children, element_id, scroll_context.clone())
        {
            return Some(result);
        }
    }
    None
}

// ─── Keyboard Handling ───────────────────────────────────────────────────────

/// Handle keydown event. Returns true if state changed (needs re-render).
fn handle_keydown(key: &str, shift: bool) -> bool {
    APP.with(|cell| {
        let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
        let app = match borrow.as_mut() {
            Some(a) => a,
            None => return false,
        };
        let layout = match &app.layout {
            Some(l) => l.clone(),
            None => return false,
        };

        // Handle Tab key for focus navigation
        // If an overlay has focus-trap, restrict focusable elements to that overlay
        let focusable_source = get_focus_trap_overlay(&layout);
        let focusable_nodes = match &focusable_source {
            Some(overlay) => &overlay.children,
            None => layout.root.as_slice(),
        };
        if key == "Tab" {
            let focusable = collect_focusable_elements(focusable_nodes);
            if focusable.is_empty() {
                return false;
            }

            // Find current focus index
            let current_idx = app
                .focused_element_id
                .as_ref()
                .and_then(|id| focusable.iter().position(|(fid, _, _)| fid == id));

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
                    find_input_by_id(&layout.root, new_id, &app.state_store)
                {
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
                if let Some(handlers) =
                    find_handlers_by_element_id(&layout.root, focused_id, "click")
                {
                    let mut changed = false;
                    for handler in &handlers {
                        if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                            changed = true;
                        }
                    }
                    return changed;
                }
            }
        }

        // Handle Escape key - dismiss topmost overlay first, then blur focused element
        if key == "Escape" {
            // Check if any overlay should be dismissed (topmost first)
            for overlay in layout.overlays.iter().rev() {
                // dismiss-on-escape defaults to true; opt out with dismiss-on-escape: false
                let dismiss = match overlay.props.get("dismiss-on-escape") {
                    Some(RenderValue::Bool(false)) => false,
                    _ => true,
                };
                if dismiss {
                    // Fire click-outside handlers to dismiss the overlay
                    let outside_handlers: Vec<_> = overlay
                        .handlers
                        .iter()
                        .filter(|h| h.event == "click-outside")
                        .cloned()
                        .collect();
                    if !outside_handlers.is_empty() {
                        let mut changed = false;
                        for handler in &outside_handlers {
                            if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                                changed = true;
                            }
                        }
                        return changed;
                    }
                }
            }
            // No overlay to dismiss — blur focused element
            if app.focused_element_id.is_some() || app.focused_input.is_some() {
                app.focused_element_id = None;
                app.focused_input = None;
                blur_hidden_input();
                return true;
            }
        }

        // Handle arrow key events on focused element
        let arrow_event = match key {
            "ArrowUp" => Some("arrow-up"),
            "ArrowDown" => Some("arrow-down"),
            "ArrowLeft" => Some("arrow-left"),
            "ArrowRight" => Some("arrow-right"),
            _ => None,
        };
        if let Some(event_name) = arrow_event {
            if let Some(ref focused_id) = app.focused_element_id.clone() {
                // Search overlays first, then root
                let handlers = find_handlers_in_layout(&layout, focused_id, event_name);
                if let Some(handlers) = handlers {
                    let mut changed = false;
                    for handler in &handlers {
                        if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
                            changed = true;
                        }
                    }
                    return changed;
                }
            }
        }

        // Execute keypress handlers on focused element
        if let Some(ref focused_id) = app.focused_element_id.clone() {
            if let Some(handlers) =
                find_handlers_by_element_id(&layout.root, focused_id, "keypress")
            {
                let mut changed = false;
                for handler in &handlers {
                    if execute_handler_actions(handler, &mut app.state_store, &app.render_tree.themes) {
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
) -> Option<(
    String,
    String,
    String,
    String,
    Vec<naze_ir::IrEventHandler>,
    Option<RenderValue>,
)> {
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
                let handlers: Vec<_> = node
                    .handlers
                    .iter()
                    .filter(|h| h.event == "change")
                    .cloned()
                    .collect();
                let validate = node.props.get("validate").cloned();
                return Some((
                    bind_var.clone(),
                    node_id,
                    value,
                    input_type,
                    handlers,
                    validate,
                ));
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
            let handlers: Vec<_> = node
                .handlers
                .iter()
                .filter(|h| h.event == event)
                .cloned()
                .collect();
            if !handlers.is_empty() {
                return Some(handlers);
            }
            // For form elements, clicking also triggers change
            if event == "click" && matches!(node.kind.as_str(), "checkbox" | "radio") {
                let change_handlers: Vec<_> = node
                    .handlers
                    .iter()
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
const A11Y_LIVE_REGION_ID: &str = "__naze_a11y_live";

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
    let body = document.body().ok_or("no body")?;
    body.append_child(&container)?;

    // Create live region for dynamic content announcements
    if document.get_element_by_id(A11Y_LIVE_REGION_ID).is_none() {
        let live = document.create_element("div")?;
        live.set_id(A11Y_LIVE_REGION_ID);
        live.set_attribute("aria-live", "polite")?;
        live.set_attribute("aria-atomic", "true")?;
        // Same sr-only styling
        let live_style = live
            .dyn_ref::<web_sys::HtmlElement>()
            .ok_or("not an HtmlElement")?
            .style();
        live_style.set_property("position", "absolute")?;
        live_style.set_property("width", "1px")?;
        live_style.set_property("height", "1px")?;
        live_style.set_property("padding", "0")?;
        live_style.set_property("margin", "-1px")?;
        live_style.set_property("overflow", "hidden")?;
        live_style.set_property("clip", "rect(0, 0, 0, 0)")?;
        live_style.set_property("white-space", "nowrap")?;
        live_style.set_property("border", "0")?;
        body.append_child(&live)?;
    }

    Ok(())
}

/// Update the screen reader DOM to mirror the current layout.
fn update_a11y_dom(layout: &LayoutTree, prev_a11y_texts: &mut Vec<String>) {
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

        // Collect current text content for live region diff
        let mut current_texts = Vec::new();
        collect_visible_texts(&layout.root, &mut current_texts);

        // Diff against previous texts and announce new ones
        let new_texts: Vec<&String> = current_texts
            .iter()
            .filter(|t| !prev_a11y_texts.contains(t))
            .collect();

        if !new_texts.is_empty() {
            // Announce the first new text via the live region
            if let Some(live_el) = document.get_element_by_id(A11Y_LIVE_REGION_ID) {
                live_el.set_text_content(Some(new_texts[0]));
            }
        }

        *prev_a11y_texts = current_texts;

        Ok(())
    })();

    if let Err(e) = result {
        web_sys::console::warn_1(&format!("a11y update failed: {:?}", e).into());
    }
}

/// Collect all visible text content from the layout tree.
fn collect_visible_texts(nodes: &[PositionedNode], texts: &mut Vec<String>) {
    for node in nodes {
        if matches!(node.kind.as_str(), "text" | "heading" | "link") {
            let text = naze_renderer::get_text_content(&node.props);
            if !text.is_empty() {
                texts.push(text);
            }
        }
        collect_visible_texts(&node.children, texts);
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

        // Set aria-live for status/alert roles
        if let Some(RenderValue::Str(role_str)) = node.props.get("role") {
            match role_str.as_str() {
                "status" => {
                    el.set_attribute("aria-live", "polite")?;
                }
                "alert" => {
                    el.set_attribute("aria-live", "assertive")?;
                }
                _ => {}
            }
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
            let level = node
                .props
                .get("level")
                .and_then(|v| {
                    if let RenderValue::Num(n, _) = v {
                        Some(*n as i32)
                    } else {
                        None
                    }
                })
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
            ("a", "link")
                | ("select", "listbox")
                | ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", "heading")
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
fn fetch_data(name: &str, url: &str, method: &str) {
    use wasm_bindgen_futures::spawn_local;

    let name = name.to_string();
    let url = url.to_string();
    let method = method.to_string();

    spawn_local(async move {
        let result = do_fetch(&url, &method).await;

        APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
            if let Some(app) = borrow.as_mut() {
                // Set loading to false
                app.state_store
                    .insert(state_key(&name, "loading"), RenderValue::Bool(false));

                match result {
                    Ok(data) => {
                        // Success: populate data, clear error
                        app.state_store.insert(state_key(&name, "data"), data);
                        app.state_store
                            .insert(state_key(&name, "error"), RenderValue::Str(String::new()));
                    }
                    Err(err) => {
                        // Error: set error message, keep data empty
                        app.state_store
                            .insert(state_key(&name, "error"), RenderValue::Str(err));
                    }
                }
            }
        });

        // Trigger re-render
        schedule_render();
    });
}

/// Call a server function via POST /api/{func_name} and update three-state variables.
fn call_server_function(name: &str, func_name: &str, args: Vec<RenderValue>) {
    use wasm_bindgen_futures::spawn_local;

    let name = name.to_string();
    let func_name = func_name.to_string();

    spawn_local(async move {
        let result = do_server_call(&func_name, &args).await;

        APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
            if let Some(app) = borrow.as_mut() {
                // Set loading to false
                app.state_store
                    .insert(state_key(&name, "loading"), RenderValue::Bool(false));

                match result {
                    Ok(data) => {
                        app.state_store.insert(state_key(&name, "data"), data);
                        app.state_store
                            .insert(state_key(&name, "error"), RenderValue::Str(String::new()));
                    }
                    Err(err) => {
                        app.state_store
                            .insert(state_key(&name, "error"), RenderValue::Str(err));
                    }
                }
            }
        });

        schedule_render();
    });
}

/// POST to /api/{func_name} with JSON body { "args": [...] }.
async fn do_server_call(func_name: &str, args: &[RenderValue]) -> Result<RenderValue, String> {
    let start_ms = get_now_ms();
    let window = web_sys::window().ok_or("no window")?;

    // Build JSON body: { "args": [arg1, arg2, ...] }
    let args_array = js_sys::Array::new();
    for arg in args {
        args_array.push(&render_value_to_jsvalue(arg));
    }
    let body_obj = js_sys::Object::new();
    js_sys::Reflect::set(&body_obj, &"args".into(), &args_array)
        .map_err(|_| "failed to set args")?;
    let body_str = js_sys::JSON::stringify(&body_obj)
        .map_err(|_| "failed to stringify body")?
        .as_string()
        .unwrap_or_default();

    let url = format!("/api/{}", func_name);
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&body_str));

    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("failed to create request: {:?}", e))?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("failed to set header: {:?}", e))?;

    let resp_promise = window.fetch_with_request(&request);
    let resp_value = wasm_bindgen_futures::JsFuture::from(resp_promise)
        .await
        .map_err(|e| {
            log_network(
                &url,
                "POST",
                0,
                get_now_ms() - start_ms,
                "server call failed",
            );
            format!("server call failed: {:?}", e)
        })?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "response is not a Response")?;

    let status = resp.status();

    if !resp.ok() {
        log_network(&url, "POST", status, get_now_ms() - start_ms, "");
        return Err(format!("server error: {}", status));
    }

    let json_promise = resp
        .json()
        .map_err(|e| format!("failed to get JSON: {:?}", e))?;
    let json_value = wasm_bindgen_futures::JsFuture::from(json_promise)
        .await
        .map_err(|e| format!("JSON parse failed: {:?}", e))?;

    // Extract "data" field from response { "data": ... }
    let data = js_sys::Reflect::get(&json_value, &"data".into()).unwrap_or(JsValue::NULL);

    // Check for "error" field
    if let Ok(err_val) = js_sys::Reflect::get(&json_value, &"error".into()) {
        if let Some(err_str) = err_val.as_string() {
            if !err_str.is_empty() {
                log_network(&url, "POST", status, get_now_ms() - start_ms, &err_str);
                return Err(err_str);
            }
        }
    }

    log_network(&url, "POST", status, get_now_ms() - start_ms, "ok");
    Ok(js_to_render_value(&data))
}

/// Extract `{variable}` references from prompt system/user templates and
/// collect their current values from the state store as strings.
fn collect_prompt_vars(
    system: &str,
    user: &str,
    state_store: &HashMap<String, RenderValue>,
) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for template in [system, user] {
        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut var_name = String::new();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    var_name.push(c);
                }
                if !var_name.is_empty() && !vars.contains_key(&var_name) {
                    let val = state_store
                        .get(&var_name)
                        .map(render_value_to_string)
                        .unwrap_or_default();
                    vars.insert(var_name, val);
                }
            }
        }
    }
    vars
}

/// Call a prompt via POST /api/prompt/{name} and update three-state variables.
fn call_prompt(name: &str, vars: HashMap<String, String>) {
    use wasm_bindgen_futures::spawn_local;

    let name = name.to_string();

    spawn_local(async move {
        let result = do_prompt_call(&name, &vars).await;

        APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
            if let Some(app) = borrow.as_mut() {
                app.state_store
                    .insert(state_key(&name, "loading"), RenderValue::Bool(false));

                match result {
                    Ok(text) => {
                        app.state_store
                            .insert(state_key(&name, "data"), RenderValue::Str(text));
                        app.state_store
                            .insert(state_key(&name, "error"), RenderValue::Str(String::new()));
                    }
                    Err(err) => {
                        app.state_store
                            .insert(state_key(&name, "error"), RenderValue::Str(err));
                    }
                }
            }
        });

        schedule_render();
    });
}

/// POST to /api/prompt/{name} with JSON body { "vars": { "key": "value" } }.
async fn do_prompt_call(name: &str, vars: &HashMap<String, String>) -> Result<String, String> {
    let start_ms = get_now_ms();
    let window = web_sys::window().ok_or("no window")?;

    // Build JSON body: { "vars": { "key": "value", ... } }
    let vars_obj = js_sys::Object::new();
    for (k, v) in vars {
        js_sys::Reflect::set(&vars_obj, &JsValue::from_str(k), &JsValue::from_str(v))
            .map_err(|_| "failed to set var")?;
    }
    let body_obj = js_sys::Object::new();
    js_sys::Reflect::set(&body_obj, &"vars".into(), &vars_obj).map_err(|_| "failed to set vars")?;
    let body_str = js_sys::JSON::stringify(&body_obj)
        .map_err(|_| "failed to stringify body")?
        .as_string()
        .unwrap_or_default();

    let url = format!("/api/prompt/{}", name);
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&body_str));

    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("failed to create request: {:?}", e))?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("failed to set header: {:?}", e))?;

    let resp_promise = window.fetch_with_request(&request);
    let resp_value = wasm_bindgen_futures::JsFuture::from(resp_promise)
        .await
        .map_err(|e| {
            log_network(
                &url,
                "POST",
                0,
                get_now_ms() - start_ms,
                "prompt call failed",
            );
            format!("prompt call failed: {:?}", e)
        })?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "response is not a Response")?;

    let status = resp.status();

    if !resp.ok() {
        log_network(&url, "POST", status, get_now_ms() - start_ms, "");
        return Err(format!("prompt error: {}", status));
    }

    let json_promise = resp
        .json()
        .map_err(|e| format!("failed to get JSON: {:?}", e))?;
    let json_value = wasm_bindgen_futures::JsFuture::from(json_promise)
        .await
        .map_err(|e| format!("JSON parse failed: {:?}", e))?;

    // Check for "error" field first
    if let Ok(err_val) = js_sys::Reflect::get(&json_value, &"error".into()) {
        if let Some(err_str) = err_val.as_string() {
            if !err_str.is_empty() {
                log_network(&url, "POST", status, get_now_ms() - start_ms, &err_str);
                return Err(err_str);
            }
        }
    }

    // Extract "data" field (string response from AI provider)
    let data = js_sys::Reflect::get(&json_value, &"data".into()).unwrap_or(JsValue::NULL);
    let result = data.as_string().unwrap_or_default();
    let preview = if result.len() > 80 {
        &result[..80]
    } else {
        &result
    };
    log_network(&url, "POST", status, get_now_ms() - start_ms, preview);
    Ok(result)
}

/// Convert a RenderValue to a JsValue for use in fetch body serialization.
fn render_value_to_jsvalue(v: &RenderValue) -> JsValue {
    match v {
        RenderValue::Str(s) => JsValue::from_str(s),
        RenderValue::Num(n, _) => JsValue::from_f64(*n),
        RenderValue::Bool(b) => JsValue::from_bool(*b),
        RenderValue::Color(c) => JsValue::from_str(&format!("#{:06x}", c)),
        RenderValue::List(items) => {
            let arr = js_sys::Array::new();
            for item in items {
                arr.push(&render_value_to_jsvalue(item));
            }
            arr.into()
        }
        RenderValue::Object(entries) => {
            let obj = js_sys::Object::new();
            for (k, v) in entries {
                let _ =
                    js_sys::Reflect::set(&obj, &JsValue::from_str(k), &render_value_to_jsvalue(v));
            }
            obj.into()
        }
        _ => JsValue::NULL,
    }
}

/// Connect to a WebSocket stream and append incoming messages to the data list.
fn connect_stream(name: &str, url: &str) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let ws = match web_sys::WebSocket::new(url) {
        Ok(ws) => ws,
        Err(e) => {
            web_sys::console::log_1(&format!("WebSocket connect failed: {:?}", e).into());
            return;
        }
    };

    let name_clone = name.to_string();

    // Set loading to false on open
    let name_open = name_clone.clone();
    let onopen = Closure::wrap(Box::new(move |_: web_sys::Event| {
        APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
            if let Some(app) = borrow.as_mut() {
                app.state_store
                    .insert(state_key(&name_open, "loading"), RenderValue::Bool(false));
            }
        });
        schedule_render();
    }) as Box<dyn Fn(web_sys::Event)>);
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    // Append incoming messages to data list
    let name_msg = name_clone.clone();
    let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
        if let Some(text) = e.data().as_string() {
            APP.with(|cell| {
                let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
                if let Some(app) = borrow.as_mut() {
                    let key = state_key(&name_msg, "data");
                    let current = app
                        .state_store
                        .get(&key)
                        .cloned()
                        .unwrap_or(RenderValue::List(vec![]));
                    let mut items = match current {
                        RenderValue::List(items) => items,
                        _ => vec![],
                    };
                    items.push(RenderValue::Str(text));
                    app.state_store.insert(key, RenderValue::List(items));
                }
            });
            schedule_render();
        }
    }) as Box<dyn Fn(web_sys::MessageEvent)>);
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // Set error on close/error
    let name_err = name_clone.clone();
    let onerror = Closure::wrap(Box::new(move |_: web_sys::Event| {
        APP.with(|cell| {
            let mut borrow = match cell.try_borrow_mut() { Ok(b) => b, Err(_) => return Default::default(), };
            if let Some(app) = borrow.as_mut() {
                app.state_store.insert(
                    state_key(&name_err, "error"),
                    RenderValue::Str("WebSocket error".to_string()),
                );
                app.state_store
                    .insert(state_key(&name_err, "loading"), RenderValue::Bool(false));
            }
        });
        schedule_render();
    }) as Box<dyn Fn(web_sys::Event)>);
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();
}

/// Perform the actual HTTP fetch and parse JSON response.
async fn do_fetch(url: &str, method: &str) -> Result<RenderValue, String> {
    let start_ms = get_now_ms();
    let window = web_sys::window().ok_or("no window")?;

    // Create request with configured method
    let opts = web_sys::RequestInit::new();
    opts.set_method(&method.to_uppercase());
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
        .map_err(|e| {
            log_network(url, method, 0, get_now_ms() - start_ms, "fetch failed");
            format!("fetch failed: {:?}", e)
        })?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "response is not a Response")?;

    let status = resp.status();

    // Check status
    if !resp.ok() {
        log_network(url, method, status, get_now_ms() - start_ms, "");
        return Err(format!("HTTP error: {}", status));
    }

    // Get JSON
    let json_promise = resp
        .json()
        .map_err(|e| format!("failed to get JSON: {:?}", e))?;

    let json_value = wasm_bindgen_futures::JsFuture::from(json_promise)
        .await
        .map_err(|e| format!("JSON parse failed: {:?}", e))?;

    log_network(url, method, status, get_now_ms() - start_ms, "ok");

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
        let items: Vec<RenderValue> = arr.iter().map(|item| js_to_render_value(&item)).collect();
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
