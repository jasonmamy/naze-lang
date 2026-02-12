//! Development server with hot reload via WebSocket.

use std::path::Path;
use std::time::{Duration, Instant};

use axum::http::header::{HeaderValue, CACHE_CONTROL};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use naze_compiler::resolve::BuildCache;

use crate::build;
use crate::diagnostic::Format;
use crate::manifest::Manifest;

/// Shared state for the dev server.
#[derive(Clone)]
struct AppState {
    reload_tx: broadcast::Sender<()>,
    server_fns: std::sync::Arc<std::sync::RwLock<Vec<naze_ir::ServerFuncDecl>>>,
    prompts: std::sync::Arc<std::sync::RwLock<Vec<naze_ir::PromptDecl>>>,
}

/// HTML template with hot reload WebSocket client.
const DEV_INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{TITLE}} - Dev</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html, body { width: 100%; height: 100%; overflow: hidden; background: #fff; font-family: system-ui, -apple-system, sans-serif; }
    .app-wrap { display: flex; width: 100%; height: 100%; }
    .canvas-wrap { flex: 1; position: relative; overflow: hidden; }
    canvas { display: block; touch-action: none; }
    .dev-banner {
      position: fixed; bottom: 8px; right: 8px;
      background: #22c55e; color: white; padding: 4px 8px;
      border-radius: 4px; font-size: 12px; z-index: 9999; opacity: 0.8;
    }
    .dev-banner.disconnected { background: #ef4444; }
    .dev-banner.reloading { background: #f59e0b; }

    /* Inspector Panel */
    .inspector { display: none; width: 320px; height: 100%; background: #1e1e2e; color: #cdd6f4; border-left: 1px solid #313244; flex-direction: column; font-size: 12px; overflow: hidden; }
    .inspector.open { display: flex; }
    .inspector-header { display: flex; align-items: center; padding: 6px 10px; background: #181825; border-bottom: 1px solid #313244; gap: 4px; }
    .inspector-header h2 { font-size: 12px; font-weight: 600; margin-right: auto; }
    .inspector-header button { background: none; border: none; color: #6c7086; cursor: pointer; font-size: 14px; padding: 2px 4px; }
    .inspector-header button:hover { color: #cdd6f4; }
    .inspector-tabs { display: flex; background: #181825; border-bottom: 1px solid #313244; }
    .inspector-tabs button { flex: 1; background: none; border: none; color: #6c7086; padding: 6px 4px; font-size: 11px; cursor: pointer; border-bottom: 2px solid transparent; }
    .inspector-tabs button.active { color: #89b4fa; border-bottom-color: #89b4fa; }
    .inspector-tabs button:hover { color: #bac2de; }
    .tab-content { flex: 1; overflow-y: auto; padding: 6px; }
    .tab-content::-webkit-scrollbar { width: 6px; }
    .tab-content::-webkit-scrollbar-thumb { background: #45475a; border-radius: 3px; }

    /* Elements Tab */
    .tree-node { padding: 1px 0; }
    .tree-row { display: flex; align-items: center; padding: 2px 4px; cursor: pointer; border-radius: 3px; white-space: nowrap; }
    .tree-row:hover { background: #313244; }
    .tree-row.selected { background: #45475a; }
    .tree-toggle { width: 14px; text-align: center; color: #6c7086; flex-shrink: 0; user-select: none; }
    .tree-kind { color: #f38ba8; }
    .tree-prop { color: #a6e3a1; margin-left: 4px; }
    .tree-children { padding-left: 14px; }
    .node-detail { border-top: 1px solid #313244; margin-top: 6px; padding-top: 6px; }
    .node-detail-row { display: flex; padding: 1px 4px; }
    .node-detail-key { color: #89b4fa; min-width: 70px; }
    .node-detail-val { color: #a6e3a1; word-break: break-all; }

    /* State Tab */
    .state-row { display: flex; padding: 3px 4px; border-radius: 3px; }
    .state-row.changed { animation: flash-yellow 0.5s; }
    @keyframes flash-yellow { 0% { background: #f9e2af33; } 100% { background: transparent; } }
    .state-name { color: #89b4fa; min-width: 100px; flex-shrink: 0; }
    .state-val { color: #a6e3a1; word-break: break-all; }

    /* Events Tab */
    .event-entry { padding: 3px 4px; border-bottom: 1px solid #31324488; font-size: 11px; }
    .event-time { color: #6c7086; }
    .event-type { color: #f9e2af; font-weight: 600; }
    .event-target { color: #89b4fa; }
    .event-change { color: #a6e3a1; }
    .event-section-header { color: #6c7086; font-size: 10px; text-transform: uppercase; letter-spacing: 1px; padding: 6px 4px 2px; }
    .net-entry { padding: 3px 4px; border-bottom: 1px solid #31324488; font-size: 11px; }
    .net-method { color: #f9e2af; font-weight: 600; }
    .net-url { color: #89b4fa; }
    .net-status { font-weight: 600; }
    .net-status.ok { color: #a6e3a1; }
    .net-status.err { color: #f38ba8; }
    .net-dur { color: #6c7086; }

    /* A11y Tab */
    .a11y-row { display: flex; padding: 2px 4px; }
    .a11y-kind { color: #f38ba8; min-width: 80px; }
    .a11y-map { color: #6c7086; margin: 0 6px; }
    .a11y-html { color: #a6e3a1; }
  </style>
</head>
<body>
  <div class="app-wrap">
    <div class="canvas-wrap">
      <canvas id="naze-canvas"></canvas>
    </div>
    <div class="inspector" id="inspector">
      <div class="inspector-header">
        <h2>Inspector</h2>
        <button id="insp-close" title="Close (Ctrl+Shift+I)">&times;</button>
      </div>
      <div class="inspector-tabs">
        <button class="active" data-tab="elements">Elements</button>
        <button data-tab="state">State</button>
        <button data-tab="events">Events</button>
        <button data-tab="a11y">A11y</button>
      </div>
      <div class="tab-content" id="tab-elements"></div>
      <div class="tab-content" id="tab-state" style="display:none"></div>
      <div class="tab-content" id="tab-events" style="display:none"></div>
      <div class="tab-content" id="tab-a11y" style="display:none"></div>
    </div>
  </div>
  <div class="dev-banner" id="dev-status">Connected</div>
  {{SCRIPTS}}
{{WASM_IMPORTS}}
  <script type="module">
    import init, { start, reset_and_reload, inspector_get_tree, inspector_get_state,
      inspector_node_at, inspector_set_highlight, inspector_get_event_log,
      inspector_get_network_log } from './naze_runtime.js';

    const statusEl = document.getElementById('dev-status');
    const inspectorEl = document.getElementById('inspector');
    const canvasEl = document.getElementById('naze-canvas');
    let initialized = false;

    async function loadApp() {
      const resp = await fetch('./app_data.bin?t=' + Date.now());
      const data = new Uint8Array(await resp.arrayBuffer());
      if (initialized) {
        console.log('[naze-dev] calling reset_and_reload');
        try {
          reset_and_reload(data);
          statusEl.textContent = 'Reloaded';
          statusEl.className = 'dev-banner';
          setTimeout(() => { statusEl.textContent = 'Connected'; }, 1000);
          if (inspectorEl.classList.contains('open')) refreshInspector();
        } catch (e) {
          console.error('[naze-dev] reset_and_reload failed:', e);
          statusEl.textContent = 'Error';
          statusEl.className = 'dev-banner disconnected';
        }
      } else {
        console.log('[naze-dev] calling start');
        start(data, 'naze-canvas');
        initialized = true;
      }
    }

    // ─── Inspector ────────────────────────────────────────────────────
    let inspOpen = false;
    let activeTab = 'elements';
    let selectedPath = null;
    let prevState = {};
    let stateTimer = null;
    let eventTimer = null;

    function toggleInspector() {
      inspOpen = !inspOpen;
      inspectorEl.classList.toggle('open', inspOpen);
      if (inspOpen) {
        refreshInspector();
        startPolling();
      } else {
        stopPolling();
        inspector_set_highlight('');
      }
    }

    function startPolling() {
      stateTimer = setInterval(refreshState, 500);
      eventTimer = setInterval(refreshEvents, 1000);
    }
    function stopPolling() {
      clearInterval(stateTimer);
      clearInterval(eventTimer);
    }

    function refreshInspector() {
      if (activeTab === 'elements') refreshElements();
      else if (activeTab === 'state') refreshState();
      else if (activeTab === 'events') refreshEvents();
      else if (activeTab === 'a11y') refreshA11y();
    }

    // Tab switching
    document.querySelectorAll('.inspector-tabs button').forEach(btn => {
      btn.addEventListener('click', () => {
        document.querySelectorAll('.inspector-tabs button').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        activeTab = btn.dataset.tab;
        document.querySelectorAll('.tab-content').forEach(tc => tc.style.display = 'none');
        document.getElementById('tab-' + activeTab).style.display = '';
        refreshInspector();
      });
    });
    document.getElementById('insp-close').addEventListener('click', toggleInspector);

    // Ctrl+Shift+I
    document.addEventListener('keydown', e => {
      if (e.ctrlKey && e.shiftKey && e.key === 'I') { e.preventDefault(); toggleInspector(); }
    });

    // ─── Elements Tab ─────────────────────────────────────────────────
    function refreshElements() {
      if (!initialized) return;
      try {
        const json = inspector_get_tree();
        const tree = JSON.parse(json);
        const container = document.getElementById('tab-elements');
        container.innerHTML = '';
        if (tree.children) {
          for (const child of tree.children) {
            container.appendChild(buildTreeNode(child));
          }
        }
      } catch(e) { console.warn('inspector tree error:', e); }
    }

    function buildTreeNode(node) {
      const div = document.createElement('div');
      div.className = 'tree-node';
      const row = document.createElement('div');
      row.className = 'tree-row' + (node.path === selectedPath ? ' selected' : '');
      const hasKids = node.children && node.children.length > 0;
      const toggle = document.createElement('span');
      toggle.className = 'tree-toggle';
      toggle.textContent = hasKids ? '\u25B8' : ' ';
      row.appendChild(toggle);
      const kind = document.createElement('span');
      kind.className = 'tree-kind';
      kind.textContent = node.kind;
      row.appendChild(kind);
      // Show first text prop if present
      if (node.props) {
        const textProp = node.props.text || node.props.content || node.props.label;
        if (textProp) {
          const prop = document.createElement('span');
          prop.className = 'tree-prop';
          const t = String(textProp);
          prop.textContent = '"' + (t.length > 20 ? t.slice(0,20)+'...' : t) + '"';
          row.appendChild(prop);
        }
      }
      div.appendChild(row);

      const childWrap = document.createElement('div');
      childWrap.className = 'tree-children';
      let expanded = true;
      if (hasKids) {
        for (const c of node.children) childWrap.appendChild(buildTreeNode(c));
        div.appendChild(childWrap);
      }

      toggle.addEventListener('click', e => {
        e.stopPropagation();
        if (!hasKids) return;
        expanded = !expanded;
        toggle.textContent = expanded ? '\u25BE' : '\u25B8';
        childWrap.style.display = expanded ? '' : 'none';
      });

      row.addEventListener('click', () => {
        selectedPath = node.path;
        inspector_set_highlight(node.path || '');
        // Update selection highlight
        document.querySelectorAll('.tree-row.selected').forEach(r => r.classList.remove('selected'));
        row.classList.add('selected');
        // Show detail
        showNodeDetail(node);
      });

      return div;
    }

    function showNodeDetail(node) {
      // Remove existing detail panel
      const container = document.getElementById('tab-elements');
      const old = container.querySelector('.node-detail');
      if (old) old.remove();
      const detail = document.createElement('div');
      detail.className = 'node-detail';
      const entries = [
        ['kind', node.kind],
        ['path', node.path || ''],
      ];
      if (node.layout) {
        entries.push(['position', `${Math.round(node.layout.x)}, ${Math.round(node.layout.y)}`]);
        entries.push(['size', `${Math.round(node.layout.w)} \u00D7 ${Math.round(node.layout.h)}`]);
      }
      if (node.handlers > 0) entries.push(['handlers', String(node.handlers)]);
      if (node.props) {
        for (const [k,v] of Object.entries(node.props)) {
          entries.push([k, String(v)]);
        }
      }
      for (const [k,v] of entries) {
        const r = document.createElement('div');
        r.className = 'node-detail-row';
        r.innerHTML = '<span class="node-detail-key">' + esc(k) + '</span><span class="node-detail-val">' + esc(v) + '</span>';
        detail.appendChild(r);
      }
      container.appendChild(detail);
    }

    // ─── State Tab ────────────────────────────────────────────────────
    function refreshState() {
      if (!initialized) return;
      try {
        const json = inspector_get_state();
        const state = JSON.parse(json);
        const container = document.getElementById('tab-state');
        container.innerHTML = '';
        for (const [name, val] of Object.entries(state)) {
          const row = document.createElement('div');
          row.className = 'state-row';
          const prev = prevState[name];
          if (prev !== undefined && prev !== String(val)) {
            row.classList.add('changed');
          }
          row.innerHTML = '<span class="state-name">' + esc(name) + '</span><span class="state-val">' + esc(String(val)) + '</span>';
          container.appendChild(row);
        }
        // Update prev snapshot
        const snap = {};
        for (const [k,v] of Object.entries(state)) snap[k] = String(v);
        prevState = snap;
      } catch(e) { console.warn('inspector state error:', e); }
    }

    // ─── Events Tab ───────────────────────────────────────────────────
    function refreshEvents() {
      if (!initialized) return;
      try {
        const container = document.getElementById('tab-events');
        container.innerHTML = '';
        // Event log
        const evJson = inspector_get_event_log();
        const events = JSON.parse(evJson);
        if (events.length > 0) {
          const hdr = document.createElement('div');
          hdr.className = 'event-section-header';
          hdr.textContent = 'events';
          container.appendChild(hdr);
          for (const ev of events.slice().reverse()) {
            const d = document.createElement('div');
            d.className = 'event-entry';
            const t = new Date(ev.timestamp_ms).toLocaleTimeString('en', {hour12:false,hour:'2-digit',minute:'2-digit',second:'2-digit',fractionalSecondDigits:3});
            let html = '<span class="event-time">' + t + '</span> <span class="event-type">' + esc(ev.event_type) + '</span>';
            if (ev.target_kind) html += ' <span class="event-target">' + esc(ev.target_kind) + '</span>';
            if (ev.state_changes && ev.state_changes.length > 0) {
              for (const [v,o,n] of ev.state_changes) {
                html += ' <span class="event-change">' + esc(v) + ': ' + esc(o) + ' \u2192 ' + esc(n) + '</span>';
              }
            }
            d.innerHTML = html;
            container.appendChild(d);
          }
        }
        // Network log
        const netJson = inspector_get_network_log();
        const net = JSON.parse(netJson);
        if (net.length > 0) {
          const hdr2 = document.createElement('div');
          hdr2.className = 'event-section-header';
          hdr2.textContent = 'network';
          container.appendChild(hdr2);
          for (const n of net.slice().reverse()) {
            const d = document.createElement('div');
            d.className = 'net-entry';
            const statusClass = n.status >= 200 && n.status < 400 ? 'ok' : 'err';
            d.innerHTML = '<span class="net-method">' + esc(n.method) + '</span> <span class="net-url">' + esc(n.url) + '</span> <span class="net-status ' + statusClass + '">' + n.status + '</span> <span class="net-dur">' + Math.round(n.duration_ms) + 'ms</span>';
            container.appendChild(d);
          }
        }
        if (events.length === 0 && net.length === 0) {
          container.innerHTML = '<div style="color:#6c7086;padding:8px">No events yet. Interact with the app to see events here.</div>';
        }
      } catch(e) { console.warn('inspector events error:', e); }
    }

    // ─── A11y Tab ─────────────────────────────────────────────────────
    const A11Y_MAP = {
      text: 'p', heading: 'h1-h6', button: 'button', input: 'input',
      image: 'img', link: 'a', nav: 'nav', column: 'div', row: 'div',
      select: 'select', checkbox: 'input[checkbox]', radio: 'input[radio]',
      list: 'ul', table: 'table', form: 'form', dialog: 'dialog',
      rect: 'div', stack: 'div', grid: 'div', canvas: 'canvas',
      video: 'video', audio: 'audio', progress: 'progress', slider: 'input[range]',
    };
    function refreshA11y() {
      if (!initialized) return;
      try {
        const json = inspector_get_tree();
        const tree = JSON.parse(json);
        const container = document.getElementById('tab-a11y');
        container.innerHTML = '<div style="color:#6c7086;padding:4px 4px 8px">Semantic HTML mapping for accessibility</div>';
        function walk(node, depth) {
          const htmlTag = A11Y_MAP[node.kind] || 'div';
          const row = document.createElement('div');
          row.className = 'a11y-row';
          row.style.paddingLeft = (depth * 12 + 4) + 'px';
          row.innerHTML = '<span class="a11y-kind">' + esc(node.kind) + '</span><span class="a11y-map">\u2192</span><span class="a11y-html">&lt;' + htmlTag + '&gt;</span>';
          container.appendChild(row);
          if (node.children) { for (const c of node.children) walk(c, depth+1); }
        }
        if (tree.children) { for (const c of tree.children) walk(c, 0); }
      } catch(e) { console.warn('inspector a11y error:', e); }
    }

    // ─── Canvas hover → hit test ──────────────────────────────────────
    canvasEl.addEventListener('mousemove', e => {
      if (!inspOpen || !initialized) return;
      try {
        const rect = canvasEl.getBoundingClientRect();
        const x = (e.clientX - rect.left) * (canvasEl.width / rect.width);
        const y = (e.clientY - rect.top) * (canvasEl.height / rect.height);
        const json = inspector_node_at(x, y);
        if (json && json !== '{}') {
          const node = JSON.parse(json);
          if (node.path) inspector_set_highlight(node.path);
        }
      } catch(ex) { /* ignore */ }
    });

    function esc(s) { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }

    // ─── Main ─────────────────────────────────────────────────────────
    async function main() {
      {{WASM_IMPORTS_LOAD}}
      await init();
      await loadApp();

      function connectWs() {
        const ws = new WebSocket(`ws://${location.host}/ws`);
        ws.onopen = () => { statusEl.textContent = 'Connected'; statusEl.className = 'dev-banner'; };
        ws.onmessage = async (e) => {
          if (e.data === 'reload') {
            statusEl.textContent = 'Reloading...';
            statusEl.className = 'dev-banner reloading';
            await loadApp();
          }
        };
        ws.onclose = () => {
          statusEl.textContent = 'Disconnected';
          statusEl.className = 'dev-banner disconnected';
          setTimeout(connectWs, 1000);
        };
        ws.onerror = () => { ws.close(); };
      }
      connectWs();
    }

    main().catch(e => {
      document.body.innerHTML = '<pre style="color:red;padding:20px">' + e + '</pre>';
    });
  </script>
</body>
</html>
"#;

/// Run the dev server with hot reload.
pub fn run(
    manifest: &Manifest,
    port: u16,
    open_browser: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move { run_async(manifest, port, open_browser).await })
}

async fn run_async(
    manifest: &Manifest,
    port: u16,
    open_browser: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let output_dir = Path::new(&manifest.build.output);

    // Load .env file and set process env vars for server-side resolution
    let dotenv = crate::manifest::load_dotenv(".env");
    for (key, value) in &dotenv {
        if std::env::var(key).is_err() {
            std::env::set_var(key, value);
        }
    }

    // Resolve dependencies once at dev server startup
    let resolved_deps = crate::deps::resolve_deps(manifest, Path::new("."))?;

    // Initial build
    eprintln!("building...");
    build::run(manifest, Format::Text, &resolved_deps, false)?;

    // Write dev index.html (overwrites the normal one)
    let title = get_app_title(manifest)?;
    let script_tags: String = manifest
        .scripts
        .iter()
        .map(|(_, url)| format!("  <script src=\"{}\"></script>", url))
        .collect::<Vec<_>>()
        .join("\n");
    let dev_html = DEV_INDEX_HTML
        .replace("{{TITLE}}", &title)
        .replace("{{SCRIPTS}}", &script_tags)
        .replace("{{WASM_IMPORTS}}", "")
        .replace("{{WASM_IMPORTS_LOAD}}", "");
    std::fs::write(output_dir.join("index.html"), dev_html)?;

    // Load server functions and prompts from initial build
    let (initial_server_fns, initial_prompts) = {
        let bin_path = output_dir.join("app_data.bin");
        let bytes = std::fs::read(&bin_path)?;
        let tree = naze_ir::deserialize(&bytes)?;
        (tree.server_functions, tree.prompts)
    };

    // Create broadcast channel for reload notifications
    let (reload_tx, _) = broadcast::channel::<()>(16);
    let server_fns = std::sync::Arc::new(std::sync::RwLock::new(initial_server_fns));
    let prompts = std::sync::Arc::new(std::sync::RwLock::new(initial_prompts));
    let state = AppState {
        reload_tx: reload_tx.clone(),
        server_fns: server_fns.clone(),
        prompts: prompts.clone(),
    };

    // Spawn file watcher
    let manifest_clone = manifest.clone();
    let output_dir_clone = output_dir.to_path_buf();
    let deps_clone = resolved_deps.clone();
    let server_fns_watcher = server_fns.clone();
    let prompts_watcher = prompts.clone();
    tokio::spawn(async move {
        watch_and_rebuild(manifest_clone, reload_tx, output_dir_clone, deps_clone, server_fns_watcher, prompts_watcher).await;
    });

    // Build router with no-cache headers for dev mode
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/{name}", post(api_handler))
        .route("/api/prompt/{name}", post(prompt_handler))
        .fallback_service(ServeDir::new(output_dir))
        .layer(SetResponseHeaderLayer::if_not_present(
            CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Print server info and flush to ensure it's visible
    eprintln!();
    eprintln!("  ┌─────────────────────────────────────────┐");
    eprintln!("  │  Dev server running at:                 │");
    eprintln!("  │  http://localhost:{:<22}│", port);
    eprintln!("  │                                         │");
    eprintln!("  │  Watching for changes...                │");
    eprintln!("  │  Press Ctrl+C to stop                   │");
    eprintln!("  │  Use --port to change port              │");
    eprintln!("  └─────────────────────────────────────────┘");
    eprintln!();
    let _ = std::io::stderr().flush();

    // Open browser if requested
    if open_browser {
        let url = format!("http://localhost:{}", port);
        if let Err(e) = open::that(&url) {
            eprintln!("  warning: couldn't open browser: {}", e);
        }
    }

    axum::serve(listener, app).await?;
    Ok(())
}

/// Get the app title from a fresh build.
fn get_app_title(manifest: &Manifest) -> Result<String, Box<dyn std::error::Error>> {
    let output_dir = Path::new(&manifest.build.output);
    let bin_path = output_dir.join("app_data.bin");
    let bytes = std::fs::read(&bin_path)?;
    let render_tree = naze_ir::deserialize(&bytes)?;
    Ok(render_tree.title)
}

/// WebSocket handler for hot reload connections.
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a single WebSocket connection.
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut reload_rx = state.reload_tx.subscribe();

    // Spawn task to forward reload messages to this client
    let send_task = tokio::spawn(async move {
        eprintln!("  client subscribed to reload notifications");
        while let Ok(()) = reload_rx.recv().await {
            eprintln!("  sending reload to client...");
            if sender.send(Message::Text("reload".into())).await.is_err() {
                eprintln!("  failed to send reload to client");
                break;
            }
            eprintln!("  reload sent successfully");
        }
    });

    // Keep connection alive by reading (and ignoring) incoming messages
    while let Some(Ok(_)) = receiver.next().await {}

    // Clean up
    send_task.abort();
}

/// Watch for file changes and rebuild on change.
async fn watch_and_rebuild(
    manifest: Manifest,
    reload_tx: broadcast::Sender<()>,
    output_dir: std::path::PathBuf,
    resolved_deps: Vec<naze_compiler::resolve::ResolvedDep>,
    server_fns: std::sync::Arc<std::sync::RwLock<Vec<naze_ir::ServerFuncDecl>>>,
    prompts: std::sync::Arc<std::sync::RwLock<Vec<naze_ir::PromptDecl>>>,
) {
    use notify::{RecursiveMode, Watcher};

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("  error: failed to create file watcher: {}", e);
            return;
        }
    };

    let project_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("  error: failed to get current dir: {}", e);
            return;
        }
    };

    if let Err(e) = watcher.watch(&project_dir, RecursiveMode::Recursive) {
        eprintln!("  error: failed to watch directory: {}", e);
        return;
    }

    let debounce = Duration::from_millis(300);
    let mut last_event: Option<Instant> = None;
    let mut build_cache = BuildCache::new();

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
                last_event = None;

                // Rebuild incrementally (reuse cached ASTs for unchanged files)
                let start = Instant::now();
                match build::run_incremental(&manifest, Format::Text, &mut build_cache, &resolved_deps, false) {
                    Ok(()) => {
                        let elapsed = start.elapsed().as_millis();
                        eprintln!("  rebuilt in {}ms", elapsed);

                        // Refresh server functions and prompts from rebuilt app_data.bin
                        if let Ok(bytes) = std::fs::read(output_dir.join("app_data.bin")) {
                            if let Ok(tree) = naze_ir::deserialize(&bytes) {
                                if let Ok(mut fns) = server_fns.write() {
                                    *fns = tree.server_functions;
                                }
                                if let Ok(mut p) = prompts.write() {
                                    *p = tree.prompts;
                                }
                            }
                        }

                        // Write dev index.html again
                        if let Ok(title) = get_app_title(&manifest) {
                            let script_tags: String = manifest
                                .scripts
                                .iter()
                                .map(|(_, url)| format!("  <script src=\"{}\"></script>", url))
                                .collect::<Vec<_>>()
                                .join("\n");
                            let dev_html = DEV_INDEX_HTML
                                .replace("{{TITLE}}", &title)
                                .replace("{{SCRIPTS}}", &script_tags)
                                .replace("{{WASM_IMPORTS}}", "")
                                .replace("{{WASM_IMPORTS_LOAD}}", "");
                            let _ = std::fs::write(output_dir.join("index.html"), dev_html);
                        }

                        // Notify all connected clients
                        let subscribers = reload_tx.receiver_count();
                        eprintln!("  notifying {} client(s)...", subscribers);
                        match reload_tx.send(()) {
                            Ok(_) => eprintln!("  broadcast sent"),
                            Err(_) => eprintln!("  broadcast failed (no receivers)"),
                        }
                    }
                    Err(e) => {
                        eprintln!("  build failed: {}", e);
                    }
                }
            }
        }
    }
}

// ─── Server Function API ────────────────────────────────────────────────────

/// Handle POST /api/{name} — evaluate a server function and return JSON result.
async fn api_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let fns = match state.server_fns.read() {
        Ok(fns) => fns,
        Err(_) => {
            return axum::Json(serde_json::json!({ "error": "internal lock error" }));
        }
    };

    let func = match fns.iter().find(|f| f.name == name) {
        Some(f) => f,
        None => {
            return axum::Json(serde_json::json!({
                "error": format!("unknown server function '{}'", name)
            }));
        }
    };

    // Parse positional args from request body
    let args = match body.get("args").and_then(|a| a.as_array()) {
        Some(arr) => arr.clone(),
        None => vec![],
    };

    let data = crate::server_fns::evaluate_server_fn(func, &args);
    axum::Json(serde_json::json!({ "data": data }))
}

/// Handle POST /api/prompt/{name} — execute an AI prompt and return the result.
async fn prompt_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(state): State<AppState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    // Extract prompt in a block so the RwLockReadGuard is dropped before .await
    let prompt = {
        let prompts = match state.prompts.read() {
            Ok(p) => p,
            Err(_) => {
                return axum::Json(serde_json::json!({ "error": "internal lock error" }));
            }
        };
        match prompts.iter().find(|p| p.name == name) {
            Some(p) => p.clone(),
            None => {
                return axum::Json(serde_json::json!({
                    "error": format!("unknown prompt '{}'", name)
                }));
            }
        }
    };

    // Extract interpolation variables from request body
    let vars: std::collections::HashMap<String, String> = body
        .get("vars")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Resolve interpolations in system/user prompts
    let system = crate::prompt_handlers::resolve_interpolations(&prompt.system, &vars);
    let user = crate::prompt_handlers::resolve_interpolations(&prompt.user, &vars);

    let req = crate::prompt_handlers::PromptRequest {
        provider: prompt.provider.clone(),
        system,
        user,
        model: prompt.model.clone(),
        max_tokens: prompt.max_tokens,
        temperature: prompt.temperature,
    };

    match crate::prompt_handlers::execute_prompt(&req).await {
        Ok(resp) => axum::Json(serde_json::json!({ "data": resp.text })),
        Err(e) => axum::Json(serde_json::json!({ "error": e })),
    }
}
