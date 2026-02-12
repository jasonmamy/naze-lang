//! Interactive playground server for live Naze editing and preview.

use std::collections::HashMap;

use naze_compiler::codegen;
use naze_compiler::error::{CompileError, Severity};
use naze_compiler::resolve;
use naze_compiler::typecheck;

pub fn run(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move { run_async(port).await })
}

async fn run_async(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use axum::{routing::post, Router};
    use tower_http::services::ServeDir;

    eprintln!("starting playground on http://localhost:{port}");

    // Create a temporary directory for playground assets
    let tmp_dir = std::env::temp_dir().join("naze-playground");
    std::fs::create_dir_all(&tmp_dir)?;

    // Write runtime WASM and JS (same as build.rs does)
    let runtime_wasm = include_bytes!("../../naze-runtime/pkg/naze_runtime_bg.wasm");
    let runtime_js = include_str!("../../naze-runtime/pkg/naze_runtime.js");
    std::fs::write(tmp_dir.join("naze_runtime_bg.wasm"), runtime_wasm)?;
    std::fs::write(tmp_dir.join("naze_runtime.js"), runtime_js)?;

    // Write playground HTML
    std::fs::write(tmp_dir.join("index.html"), PLAYGROUND_HTML)?;

    // Write playground compiler WASM + JS if available
    write_playground_wasm(&tmp_dir)?;

    let app = Router::new()
        .route("/compile", post(compile_handler))
        .fallback_service(ServeDir::new(&tmp_dir));

    let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Open browser
    let _ = open::that(format!("http://localhost:{port}"));

    axum::serve(listener, app).await?;
    Ok(())
}

/// Server-side compile endpoint: POST /compile with text/plain body.
/// Returns JSON matching the playground WASM compile() format.
async fn compile_handler(body: String) -> axum::Json<serde_json::Value> {
    let source = body;

    // 1. Parse
    let nodes = match naze_parser::parse(&source, "playground.naze") {
        Ok(n) => n,
        Err(e) => {
            return axum::Json(serde_json::json!({
                "success": false,
                "errors": [{
                    "file": "playground.naze",
                    "line": 0,
                    "column": 0,
                    "message": e.to_string()
                }]
            }));
        }
    };

    // 2. Build minimal ResolvedProject
    let project = resolve::ResolvedProject {
        entry: resolve::SourceFile {
            path: "playground.naze".into(),
            nodes,
        },
        components: HashMap::new(),
        themes: vec![],
        imports: vec![],
        errors: vec![],
    };

    // 3. Typecheck
    let errors = typecheck::typecheck(&project);
    let real_errors: Vec<&CompileError> = errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .collect();
    if !real_errors.is_empty() {
        let err_json: Vec<serde_json::Value> = real_errors
            .iter()
            .map(|e| {
                serde_json::json!({
                    "file": e.file,
                    "line": e.line,
                    "column": e.column,
                    "message": e.message
                })
            })
            .collect();
        return axum::Json(serde_json::json!({
            "success": false,
            "errors": err_json
        }));
    }

    // 4. Codegen
    let tree = codegen::lower(&project);

    // 5. Serialize
    let binary = naze_ir::serialize(&tree);

    // 6. Base64 encode
    let b64 = base64_encode(&binary);

    axum::Json(serde_json::json!({
        "success": true,
        "binary": b64,
        "errors": []
    }))
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Write playground compiler WASM assets if the crate has been built.
fn write_playground_wasm(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // The playground WASM is optional — the playground falls back to server-side
    // compilation if the WASM isn't available. Check if the pkg/ directory exists.
    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../naze-playground/pkg");

    if pkg_dir.exists() {
        let wasm_path = pkg_dir.join("naze_playground_bg.wasm");
        let js_path = pkg_dir.join("naze_playground.js");
        if wasm_path.exists() && js_path.exists() {
            std::fs::copy(&wasm_path, dir.join("naze_playground_bg.wasm"))?;
            std::fs::copy(&js_path, dir.join("naze_playground.js"))?;
        }
    }
    Ok(())
}

/// The playground HTML with editor, preview, and controls.
const PLAYGROUND_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Naze Playground</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html, body { width: 100%; height: 100%; overflow: hidden; font-family: system-ui, -apple-system, sans-serif; }
    .toolbar {
      height: 40px; background: #1e1e2e; color: #cdd6f4;
      display: flex; align-items: center; padding: 0 12px; gap: 12px;
      border-bottom: 1px solid #313244;
    }
    .toolbar h1 { font-size: 14px; font-weight: 600; }
    .toolbar select, .toolbar button {
      background: #313244; color: #cdd6f4; border: 1px solid #45475a;
      border-radius: 4px; padding: 4px 8px; font-size: 12px; cursor: pointer;
    }
    .toolbar button:hover { background: #45475a; }
    .toolbar .spacer { flex: 1; }
    .main { display: flex; height: calc(100% - 40px); }
    .editor-pane {
      width: 50%; display: flex; flex-direction: column;
      border-right: 1px solid #313244;
    }
    .preview-pane { width: 50%; position: relative; background: #fff; }
    .editor-wrap { flex: 1; position: relative; overflow: hidden; }
    .editor-textarea {
      position: absolute; top: 0; left: 0; width: 100%; height: 100%;
      background: #1e1e2e; color: #cdd6f4;
      font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
      font-size: 13px; line-height: 1.5; padding: 12px;
      border: none; outline: none; resize: none;
      tab-size: 2; white-space: pre; overflow: auto;
    }
    .error-panel {
      background: #1e1e2e; border-top: 1px solid #313244;
      padding: 8px 12px; font-size: 12px; max-height: 120px; overflow-y: auto;
    }
    .error-panel.ok { color: #a6e3a1; }
    .error-panel.err { color: #f38ba8; }
    .error-item { padding: 2px 0; }
    .error-item .loc { color: #89b4fa; }
    #preview-canvas { width: 100%; height: 100%; display: block; }
    .preview-placeholder {
      display: flex; align-items: center; justify-content: center;
      width: 100%; height: 100%; color: #6c7086; font-size: 14px;
    }
  </style>
</head>
<body>
  <div class="toolbar">
    <h1>Naze Playground</h1>
    <select id="examples"><option value="">-- Select Example --</option></select>
    <div class="spacer"></div>
    <button id="share-btn" title="Copy share URL">Share</button>
    <button id="compile-btn" title="Ctrl+Enter">Compile</button>
  </div>
  <div class="main">
    <div class="editor-pane">
      <div class="editor-wrap">
        <textarea id="editor" class="editor-textarea" spellcheck="false" autocomplete="off">app "Hello" {
  state count = 0
  column gap: 16px, padding: 20px {
    heading "Counter: {count}"
    row gap: 8px {
      button "+" { on click { set count = count + 1 } }
      button "-" { on click { set count = count - 1 } }
      button "Reset" { on click { set count = 0 } }
    }
  }
}</textarea>
      </div>
      <div id="errors" class="error-panel ok">Ready</div>
    </div>
    <div class="preview-pane">
      <canvas id="preview-canvas"></canvas>
      <div id="preview-placeholder" class="preview-placeholder" style="display:none">
        Compiling...
      </div>
    </div>
  </div>

  <script type="module">
    import init as initRuntime, { start, reset_and_reload } from './naze_runtime.js';

    const editor = document.getElementById('editor');
    const errPanel = document.getElementById('errors');
    const exSelect = document.getElementById('examples');
    const shareBtn = document.getElementById('share-btn');
    const compileBtn = document.getElementById('compile-btn');

    let runtimeReady = false;
    let appInitialized = false;
    let compileTimer = null;
    let playgroundCompile = null; // WASM compile function (if available)

    // ─── Examples ──────────────────────────────────────────────────────
    const EXAMPLES = [
      { name: "Counter", category: "Basics", source: `app "Counter" {\n  state count = 0\n  column gap: 16px, padding: 20px {\n    heading "Count: {count}"\n    row gap: 8px {\n      button "+" { on click { set count = count + 1 } }\n      button "-" { on click { set count = count - 1 } }\n      button "Reset" { on click { set count = 0 } }\n    }\n  }\n}` },
      { name: "Greeting", category: "Basics", source: `app "Greeting" {\n  state name = "World"\n  column gap: 12px, padding: 20px {\n    heading "Hello, {name}!"\n    input bind: name, placeholder: "Enter name"\n  }\n}` },
      { name: "Dashboard", category: "Layout", source: `app "Dashboard" {\n  column padding: 20px, gap: 16px {\n    heading "Dashboard"\n    row gap: 16px {\n      rect width: 200px, height: 100px, color: #e0f2fe, radius: 8px, padding: 16px {\n        text "Users", font-size: 12px, color: #0369a1\n        heading "1,234", font-size: 24px\n      }\n      rect width: 200px, height: 100px, color: #fce7f3, radius: 8px, padding: 16px {\n        text "Revenue", font-size: 12px, color: #be185d\n        heading "$45.6K", font-size: 24px\n      }\n    }\n  }\n}` },
      { name: "Todo List", category: "State", source: `app "Todo" {\n  state task = ""\n  state items = []\n  column gap: 12px, padding: 20px {\n    heading "Todo List"\n    row gap: 8px {\n      input bind: task, placeholder: "New task"\n      button "Add" {\n        on click {\n          set items = items + [task]\n          set task = ""\n        }\n      }\n    }\n    each item in items {\n      text "- {item}"\n    }\n  }\n}` },
      { name: "Theme", category: "Theming", source: `theme default {\n  colors {\n    primary: #6366f1\n    surface: #f8fafc\n    text: #1e293b\n  }\n  spacing {\n    sm: 8px\n    md: 16px\n    lg: 24px\n  }\n}\n\napp "Themed" {\n  column gap: @md, padding: @lg, color: @surface {\n    heading "Themed App", color: @primary\n    text "Uses theme tokens for consistent styling", color: @text\n  }\n}` },
    ];

    // Populate examples dropdown
    let lastCat = '';
    for (const ex of EXAMPLES) {
      if (ex.category !== lastCat) {
        const optGroup = document.createElement('optgroup');
        optGroup.label = ex.category;
        exSelect.appendChild(optGroup);
        lastCat = ex.category;
      }
      const opt = document.createElement('option');
      opt.value = ex.name;
      opt.textContent = ex.name;
      exSelect.lastElementChild.appendChild(opt);
    }

    exSelect.addEventListener('change', () => {
      const ex = EXAMPLES.find(e => e.name === exSelect.value);
      if (ex) {
        editor.value = ex.source;
        doCompile();
      }
    });

    // ─── Share URL ─────────────────────────────────────────────────────
    shareBtn.addEventListener('click', () => {
      const encoded = btoa(unescape(encodeURIComponent(editor.value)));
      const url = location.origin + location.pathname + '#' + encoded;
      navigator.clipboard.writeText(url).then(() => {
        shareBtn.textContent = 'Copied!';
        setTimeout(() => { shareBtn.textContent = 'Share'; }, 1500);
      });
    });

    // Compile button
    compileBtn.addEventListener('click', () => {
      clearTimeout(compileTimer);
      doCompile();
    });

    // Load from URL hash
    if (location.hash.length > 1) {
      try {
        const decoded = decodeURIComponent(escape(atob(location.hash.slice(1))));
        editor.value = decoded;
      } catch (e) { /* ignore bad hash */ }
    }

    // ─── Compile ───────────────────────────────────────────────────────
    async function doCompile() {
      const source = editor.value;
      if (!source.trim()) {
        errPanel.className = 'error-panel ok';
        errPanel.textContent = 'Empty';
        return;
      }

      // Try WASM-based compilation first (if playground WASM loaded)
      if (playgroundCompile) {
        try {
          const resultJson = playgroundCompile(source);
          const result = JSON.parse(resultJson);
          if (result.success) {
            errPanel.className = 'error-panel ok';
            errPanel.textContent = '\u2713 No errors';
            const binaryStr = atob(result.binary);
            const data = new Uint8Array(binaryStr.length);
            for (let i = 0; i < binaryStr.length; i++) data[i] = binaryStr.charCodeAt(i);
            await renderPreview(data);
          } else {
            showErrors(result.errors || []);
          }
        } catch (e) {
          errPanel.className = 'error-panel err';
          errPanel.textContent = 'Compile error: ' + e;
        }
        return;
      }

      // Fallback: server-side compilation via POST /compile
      try {
        const resp = await fetch('/compile', {
          method: 'POST',
          headers: { 'Content-Type': 'text/plain' },
          body: source,
        });
        if (resp.ok) {
          const result = await resp.json();
          if (result.success) {
            errPanel.className = 'error-panel ok';
            errPanel.textContent = '\u2713 No errors';
            const binaryStr = atob(result.binary);
            const data = new Uint8Array(binaryStr.length);
            for (let i = 0; i < binaryStr.length; i++) data[i] = binaryStr.charCodeAt(i);
            await renderPreview(data);
          } else {
            showErrors(result.errors || []);
          }
        } else {
          errPanel.className = 'error-panel err';
          errPanel.textContent = 'Server error: ' + resp.status;
        }
      } catch (e) {
        errPanel.className = 'error-panel err';
        errPanel.textContent = 'Connection error: ' + e;
      }
    }

    function showErrors(errors) {
      errPanel.className = 'error-panel err';
      if (errors.length === 0) {
        errPanel.textContent = 'Unknown error';
        return;
      }
      errPanel.innerHTML = errors.map(e =>
        `<div class="error-item"><span class="loc">${e.file || ''}:${e.line || 0}:${e.column || 0}</span> ${escHtml(e.message || '')}</div>`
      ).join('');
    }

    function escHtml(s) {
      return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    async function renderPreview(data) {
      if (!runtimeReady) return;
      try {
        if (appInitialized) {
          reset_and_reload(data);
        } else {
          start(data, 'preview-canvas');
          appInitialized = true;
        }
      } catch (e) {
        errPanel.className = 'error-panel err';
        errPanel.textContent = 'Runtime error: ' + e;
      }
    }

    // ─── Debounced compile on type ─────────────────────────────────────
    editor.addEventListener('input', () => {
      clearTimeout(compileTimer);
      compileTimer = setTimeout(doCompile, 500);
    });

    // Ctrl+Enter = force compile
    editor.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        clearTimeout(compileTimer);
        doCompile();
      }
      // Tab key inserts spaces
      if (e.key === 'Tab') {
        e.preventDefault();
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.value = editor.value.substring(0, start) + '  ' + editor.value.substring(end);
        editor.selectionStart = editor.selectionEnd = start + 2;
      }
    });

    // ─── Init ──────────────────────────────────────────────────────────
    async function main() {
      // Load runtime WASM
      await initRuntime();
      runtimeReady = true;

      // Try to load playground compiler WASM
      try {
        const mod = await import('./naze_playground.js');
        await mod.default();
        playgroundCompile = mod.compile;
        console.log('[playground] compiler WASM loaded');
      } catch (e) {
        console.log('[playground] compiler WASM not available, using server-side compilation');
      }

      // Initial compile
      await doCompile();
    }

    main().catch(e => console.error('Playground init error:', e));
  </script>
</body>
</html>
"#;
