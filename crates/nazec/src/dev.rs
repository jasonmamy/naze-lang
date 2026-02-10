//! Development server with hot reload via WebSocket.

use std::path::Path;
use std::time::{Duration, Instant};

use axum::http::header::{HeaderValue, CACHE_CONTROL};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::build;
use crate::diagnostic::Format;
use crate::manifest::Manifest;

/// Shared state for the dev server.
#[derive(Clone)]
struct AppState {
    reload_tx: broadcast::Sender<()>,
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
    html, body { width: 100%; height: 100%; overflow: hidden; background: #fff; }
    canvas { display: block; }
    .dev-banner {
      position: fixed;
      bottom: 8px;
      right: 8px;
      background: #22c55e;
      color: white;
      padding: 4px 8px;
      border-radius: 4px;
      font-family: system-ui, sans-serif;
      font-size: 12px;
      z-index: 9999;
      opacity: 0.8;
    }
    .dev-banner.disconnected { background: #ef4444; }
    .dev-banner.reloading { background: #f59e0b; }
  </style>
</head>
<body>
  <canvas id="naze-canvas"></canvas>
  <div class="dev-banner" id="dev-status">Connected</div>
  {{SCRIPTS}}
  <script type="module">
    import init, { start, reset_and_reload } from './naze_runtime.js';

    const statusEl = document.getElementById('dev-status');
    let initialized = false;

    async function loadApp() {
      // Add cache-busting query param to ensure fresh data
      const resp = await fetch('./app_data.bin?t=' + Date.now());
      const data = new Uint8Array(await resp.arrayBuffer());
      if (initialized) {
        // Hot reload: reset and reload with new data
        console.log('[naze-dev] calling reset_and_reload');
        try {
          reset_and_reload(data);
          statusEl.textContent = 'Reloaded';
          statusEl.className = 'dev-banner';
          setTimeout(() => { statusEl.textContent = 'Connected'; }, 1000);
        } catch (e) {
          console.error('[naze-dev] reset_and_reload failed:', e);
          statusEl.textContent = 'Error';
          statusEl.className = 'dev-banner disconnected';
        }
      } else {
        // Initial load
        console.log('[naze-dev] calling start');
        start(data, 'naze-canvas');
        initialized = true;
      }
    }

    async function main() {
      await init();
      await loadApp();

      // Connect to hot reload WebSocket
      function connectWs() {
        const ws = new WebSocket(`ws://${location.host}/ws`);

        ws.onopen = () => {
          statusEl.textContent = 'Connected';
          statusEl.className = 'dev-banner';
        };

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
          // Reconnect after 1 second
          setTimeout(connectWs, 1000);
        };

        ws.onerror = () => {
          ws.close();
        };
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

    // Initial build
    eprintln!("building...");
    build::run(manifest, Format::Text)?;

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
        .replace("{{SCRIPTS}}", &script_tags);
    std::fs::write(output_dir.join("index.html"), dev_html)?;

    // Create broadcast channel for reload notifications
    let (reload_tx, _) = broadcast::channel::<()>(16);
    let state = AppState {
        reload_tx: reload_tx.clone(),
    };

    // Spawn file watcher
    let manifest_clone = manifest.clone();
    let output_dir_clone = output_dir.to_path_buf();
    tokio::spawn(async move {
        watch_and_rebuild(manifest_clone, reload_tx, output_dir_clone).await;
    });

    // Build router with no-cache headers for dev mode
    let app = Router::new()
        .route("/ws", get(ws_handler))
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

                // Rebuild
                let start = Instant::now();
                match build::run(&manifest, Format::Text) {
                    Ok(()) => {
                        let elapsed = start.elapsed().as_millis();
                        eprintln!("  rebuilt in {}ms", elapsed);

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
                                .replace("{{SCRIPTS}}", &script_tags);
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
