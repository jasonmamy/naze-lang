//! Production SSR server: per-request HTML rendering with server function pre-evaluation.
//! Started via `nazec serve` after `nazec build` has produced `dist/`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

use naze_ir::{RenderNode, RenderTree, RenderValue};

use crate::html_renderer;
use crate::manifest::Manifest;
use crate::seo;
use crate::{exec, server_fns};

// ─── Server State ────────────────────────────────────────────────────────────

/// Shared immutable state for the SSR server.
#[derive(Clone)]
struct SsrState {
    render_tree: Arc<RenderTree>,
    manifest: Arc<Manifest>,
    script_tags: String,
    wasm_imports_tag: String,
    wasm_imports_load: String,
}

// ─── HTML Template ───────────────────────────────────────────────────────────

const SSR_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{TITLE}}</title>
{{META_TAGS}}
{{JSON_LD}}
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    .naze-static { font-family: system-ui, sans-serif; }
    .naze-canvas { display: none; }
    .naze-canvas.active { display: block; }
    .naze-static.hidden { display: none; }
  </style>
</head>
<body>
  <div class="naze-static" id="naze-static">
    {{STATIC_CONTENT}}
  </div>
  <canvas id="naze-canvas" class="naze-canvas"></canvas>
  <script id="__naze_state" type="application/json">{{HYDRATION_STATE}}</script>
  {{SCRIPTS}}
{{WASM_IMPORTS}}
  <script type="module">
    import init, { start } from '{{ASSET_PREFIX}}/naze_runtime.js';
    async function main() {
      {{WASM_IMPORTS_LOAD}}
      await init();
      const resp = await fetch('{{ASSET_PREFIX}}/app_data.bin');
      const data = new Uint8Array(await resp.arrayBuffer());
      start(data, 'naze-canvas');
      document.getElementById('naze-canvas').classList.add('active');
      document.getElementById('naze-static').classList.add('hidden');
    }
    main().catch(console.error);
  </script>
</body>
</html>
"#;

// ─── Entry Point ─────────────────────────────────────────────────────────────

/// Start the production SSR server.
pub fn run(manifest: &Manifest, port: u16, host: &str) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move { run_async(manifest, port, host).await })
}

async fn run_async(
    manifest: &Manifest,
    port: u16,
    host: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let dist_dir = Path::new(&manifest.build.output);

    // Verify dist/ exists
    let bin_path = dist_dir.join("app_data.bin");
    if !bin_path.exists() {
        return Err("dist/app_data.bin not found — run `nazec build` first".into());
    }

    // Load and deserialize the render tree
    let bytes = std::fs::read(&bin_path)?;
    let render_tree = naze_ir::deserialize(&bytes)?;

    // Pre-compute script tags and WASM import parts
    let script_tags: String = manifest
        .scripts
        .values()
        .map(|url| format!("  <script src=\"{}\"></script>", url))
        .collect::<Vec<_>>()
        .join("\n");

    let (wasm_imports_tag, wasm_imports_load) = if render_tree.imports.is_empty() {
        (String::new(), String::new())
    } else {
        (
            "  <script src=\"wasm_imports.js\"></script>".to_string(),
            "await loadWasmImports();".to_string(),
        )
    };

    let state = SsrState {
        render_tree: Arc::new(render_tree),
        manifest: Arc::new(manifest.clone()),
        script_tags,
        wasm_imports_tag,
        wasm_imports_load,
    };

    // Build router
    let app = Router::new()
        .route("/", get(ssr_root_handler))
        .route("/api/{name}", post(api_handler))
        .route("/api/prompt/{name}", post(prompt_handler))
        .route("/{*path}", get(ssr_or_static_handler))
        .fallback_service(ServeDir::new(dist_dir))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    eprintln!();
    eprintln!("  ┌─────────────────────────────────────────┐");
    eprintln!("  │  SSR server running at:                  │");
    eprintln!("  │  http://{}:{:<22}│", host, port);
    eprintln!("  │                                         │");
    eprintln!("  │  Press Ctrl+C to stop                   │");
    eprintln!("  └─────────────────────────────────────────┘");
    eprintln!();
    let _ = std::io::stderr().flush();

    axum::serve(listener, app).await?;
    Ok(())
}

// ─── Route Handlers ──────────────────────────────────────────────────────────

/// SSR handler for the root page.
async fn ssr_root_handler(State(state): State<SsrState>) -> Response {
    render_page(&state, "/")
}

/// SSR handler for sub-pages; falls through to static assets if path has a file extension.
async fn ssr_or_static_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
    State(state): State<SsrState>,
) -> Response {
    // If the path looks like a static asset, let the fallback ServeDir handle it
    if has_file_extension(&path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let request_path = format!("/{}", path);
    render_page(&state, &request_path).into_response()
}

/// Handle POST /api/{name} — evaluate a server function and return JSON result.
/// Auth-related headers (Authorization, Cookie) are forwarded to outgoing fetches.
async fn api_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
    State(state): State<SsrState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let func = match state
        .render_tree
        .server_functions
        .iter()
        .find(|f| f.name == name)
    {
        Some(f) => f,
        None => {
            return axum::Json(serde_json::json!({
                "error": format!("unknown server function '{}'", name)
            }));
        }
    };

    let args = match body.get("args").and_then(|a| a.as_array()) {
        Some(arr) => arr.clone(),
        None => vec![],
    };

    // Forward auth-related headers to server function fetches
    let forwarded: Vec<(String, String)> = ["authorization", "cookie", "x-api-key"]
        .iter()
        .filter_map(|&key| {
            headers
                .get(key)
                .and_then(|v| v.to_str().ok().map(|s| (key.to_string(), s.to_string())))
        })
        .collect();

    let data = server_fns::evaluate_server_fn_with_headers(func, &args, &forwarded);
    axum::Json(serde_json::json!({ "data": data }))
}

/// Handle POST /api/prompt/{name} — execute an AI prompt and return the result.
async fn prompt_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(state): State<SsrState>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let prompt = match state.render_tree.prompts.iter().find(|p| p.name == name) {
        Some(p) => p.clone(),
        None => {
            return axum::Json(serde_json::json!({
                "error": format!("unknown prompt '{}'", name)
            }));
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

// ─── SSR Rendering ───────────────────────────────────────────────────────────

/// Render a page to HTML with server function pre-evaluation.
fn render_page(state: &SsrState, path: &str) -> Response {
    let tree = &state.render_tree;
    let manifest = &state.manifest;

    // 1. Initialize fresh state for this request
    let mut app_state = exec::init_state(tree);

    // 2. Pre-evaluate server function calls
    for call in &tree.server_calls {
        if let Some(func) = tree
            .server_functions
            .iter()
            .find(|f| f.name == call.func_name)
        {
            // Evaluate call arguments, then delegate to server_fns
            let args_json: Vec<serde_json::Value> = call
                .args
                .iter()
                .map(|a| {
                    let val = exec::evaluate_expr(a, &app_state);
                    crate::server_fns::render_value_to_json(&val)
                })
                .collect();
            let result_json = crate::server_fns::evaluate_server_fn(func, &args_json);
            let result = crate::server_fns::json_to_render_value(&result_json);

            // Inject three-state pattern: loading=false, data=result, error=""
            app_state.insert(format!("{}.loading", call.name), RenderValue::Bool(false));
            app_state.insert(format!("{}.data", call.name), result);
            app_state.insert(
                format!("{}.error", call.name),
                RenderValue::Str(String::new()),
            );
        }
    }

    // 3. Select page content (with dynamic route params and meta)
    let (page_nodes, route_params, page_meta, page_guard) = find_page_nodes(tree, path);
    for (name, value) in &route_params {
        app_state.insert(format!("params.{name}"), RenderValue::Str(value.clone()));
    }

    // 3b. Evaluate guard (if any) — redirect on failed check
    if let Some(guard_name) = page_guard {
        if let Some(guard) = tree.guards.iter().find(|g| g.name == guard_name) {
            for check in &guard.checks {
                let val = exec::evaluate_expr(&check.condition, &app_state);
                let triggered = matches!(&val, RenderValue::Bool(true));
                if triggered {
                    return axum::response::Redirect::temporary(&check.redirect).into_response();
                }
            }
        }
    }

    // 4. Resolve nodes (evaluate __if, __each, interpolations)
    let resolved = exec::resolve_nodes(page_nodes, &app_state);

    // 5. Render to HTML
    let static_content = html_renderer::render_to_html(&resolved);

    // 6. Generate SEO tags (with page-level meta overrides)
    let mut page_title = route_to_title(path, &tree.title);
    let mut extra_meta = String::new();
    for (key, val) in page_meta {
        let resolved_val = exec::resolve_value(val, &app_state);
        let text = render_value_text(&resolved_val);
        match key.as_str() {
            "title" => page_title = text,
            "description" => {
                extra_meta.push_str(&format!(
                    "  <meta name=\"description\" content=\"{}\">\n",
                    seo::escape_html(&text)
                ));
                extra_meta.push_str(&format!(
                    "  <meta property=\"og:description\" content=\"{}\">\n",
                    seo::escape_html(&text)
                ));
            }
            "image" => {
                extra_meta.push_str(&format!(
                    "  <meta property=\"og:image\" content=\"{}\">\n",
                    seo::escape_html(&text)
                ));
            }
            "canonical" => {
                extra_meta.push_str(&format!(
                    "  <link rel=\"canonical\" href=\"{}\">\n",
                    seo::escape_html(&text)
                ));
            }
            "robots" => {
                extra_meta.push_str(&format!(
                    "  <meta name=\"robots\" content=\"{}\">\n",
                    seo::escape_html(&text)
                ));
            }
            _ => {}
        }
    }
    let mut meta_tags = seo::generate_meta_tags(manifest, &tree.title, Some(path));
    if !extra_meta.is_empty() {
        meta_tags.push('\n');
        meta_tags.push_str(&extra_meta);
    }
    let json_ld = seo::generate_json_ld(manifest, &page_title);

    // 7. Serialize pre-evaluated state for hydration
    let hydration_state = serialize_state_for_hydration(&app_state);

    // 8. Assemble HTML
    let html = SSR_HTML_TEMPLATE
        .replace("{{TITLE}}", &page_title)
        .replace("{{META_TAGS}}", &meta_tags)
        .replace("{{JSON_LD}}", &json_ld)
        .replace("{{STATIC_CONTENT}}", &static_content)
        .replace("{{HYDRATION_STATE}}", &hydration_state)
        .replace("{{SCRIPTS}}", &state.script_tags)
        .replace("{{WASM_IMPORTS}}", &state.wasm_imports_tag)
        .replace("{{WASM_IMPORTS_LOAD}}", &state.wasm_imports_load)
        .replace("{{ASSET_PREFIX}}", ".");

    Html(html).into_response()
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Match a URL path against a route pattern with `:param` segments.
fn match_route(
    pattern: &str,
    actual: &str,
    params: &[String],
    is_catch_all: bool,
) -> Option<Vec<(String, String)>> {
    if is_catch_all {
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
        if let Some(stripped) = p.strip_prefix(':') {
            let name = if param_idx < params.len() {
                params[param_idx].clone()
            } else {
                stripped.to_string()
            };
            extracted.push((name, a.to_string()));
            param_idx += 1;
        } else if p != a {
            return None;
        }
    }
    Some(extracted)
}

/// Find page nodes for a given path with dynamic route matching.
/// Returns (page_nodes, extracted_params).
#[allow(clippy::type_complexity)]
fn find_page_nodes<'a>(
    tree: &'a RenderTree,
    path: &str,
) -> (
    &'a [RenderNode],
    Vec<(String, String)>,
    &'a [(String, RenderValue)],
    Option<&'a str>,
) {
    // 1. Exact match (static routes)
    for page in &tree.pages {
        if !page.is_catch_all && page.params.is_empty() && page.path == path {
            return (&page.root, vec![], &page.meta, page.guard.as_deref());
        }
    }
    // 2. Dynamic routes
    for page in &tree.pages {
        if !page.params.is_empty() {
            if let Some(extracted) = match_route(&page.path, path, &page.params, false) {
                return (&page.root, extracted, &page.meta, page.guard.as_deref());
            }
        }
    }
    // 3. Catch-all
    for page in &tree.pages {
        if page.is_catch_all {
            return (&page.root, vec![], &page.meta, page.guard.as_deref());
        }
    }
    // 4. Fallback to root (no meta, no guard)
    (&tree.root, vec![], &[], None)
}

/// Convert a RenderValue to a plain text string for meta tag content.
fn render_value_text(val: &RenderValue) -> String {
    match val {
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

/// Check if a path looks like a static file (has a file extension).
fn has_file_extension(path: &str) -> bool {
    let last_segment = path.rsplit('/').next().unwrap_or(path);
    last_segment.contains('.') && !last_segment.starts_with('.')
}

/// Derive a page title from a route path: "/about" -> "About - My App"
fn route_to_title(path: &str, app_title: &str) -> String {
    let segment = path.trim_matches('/').rsplit('/').next().unwrap_or("");
    if segment.is_empty() {
        return app_title.to_string();
    }
    let name: String = segment
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => first.to_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!("{} - {}", name, app_title)
}

/// Serialize app state as JSON for the hydration script tag.
fn serialize_state_for_hydration(state: &HashMap<String, RenderValue>) -> String {
    let json_map: serde_json::Map<String, serde_json::Value> = state
        .iter()
        .map(|(k, v)| (k.clone(), server_fns::render_value_to_json(v)))
        .collect();
    serde_json::to_string(&json_map).unwrap_or_else(|_| "{}".to_string())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use naze_ir::{
        IrExpression, IrServerBody, PageDef, RenderNode, RenderTree, ServerCallDecl, ServerFuncDecl,
    };

    fn empty_tree() -> RenderTree {
        RenderTree {
            title: "Test App".to_string(),
            state: vec![],
            computed: vec![],
            data: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![text_node("Hello from root")],
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        }
    }

    fn text_node(text: &str) -> RenderNode {
        let mut props = HashMap::new();
        props.insert("__text".to_string(), RenderValue::Str(text.to_string()));
        RenderNode {
            kind: "text".to_string(),
            props,
            children: vec![],
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        }
    }

    #[test]
    fn test_find_page_nodes_root() {
        let tree = empty_tree();
        let (nodes, params, _meta, _guard) = find_page_nodes(&tree, "/");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].kind, "text");
        assert!(params.is_empty());
    }

    #[test]
    fn test_find_page_nodes_named() {
        let mut tree = empty_tree();
        tree.pages.push(PageDef {
            path: "/about".to_string(),
            params: vec![],
            is_catch_all: false,
            guard: None,
            meta: vec![],
            root: vec![text_node("About page")],
        });
        let (nodes, params, _meta, _guard) = find_page_nodes(&tree, "/about");
        assert_eq!(nodes.len(), 1);
        assert!(params.is_empty());
        match nodes[0].props.get("__text") {
            Some(RenderValue::Str(s)) => assert_eq!(s, "About page"),
            _ => panic!("expected text prop"),
        }
    }

    #[test]
    fn test_find_page_nodes_fallback() {
        let tree = empty_tree();
        let (nodes, params, _meta, _guard) = find_page_nodes(&tree, "/nonexistent");
        // Falls back to root
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].kind, "text");
        assert!(params.is_empty());
    }

    #[test]
    fn test_find_page_nodes_dynamic_route() {
        let mut tree = empty_tree();
        tree.pages.push(PageDef {
            path: "/posts/:id".to_string(),
            params: vec!["id".to_string()],
            is_catch_all: false,
            guard: None,
            meta: vec![],
            root: vec![text_node("Post detail")],
        });
        let (nodes, params, _meta, _guard) = find_page_nodes(&tree, "/posts/42");
        assert_eq!(nodes.len(), 1);
        assert_eq!(params, vec![("id".to_string(), "42".to_string())]);
    }

    #[test]
    fn test_find_page_nodes_catch_all() {
        let mut tree = empty_tree();
        tree.pages.push(PageDef {
            path: "/*".to_string(),
            params: vec![],
            is_catch_all: true,
            guard: None,
            meta: vec![],
            root: vec![text_node("Not found")],
        });
        let (nodes, _, _meta, _guard) = find_page_nodes(&tree, "/anything/here");
        assert_eq!(nodes.len(), 1);
        match nodes[0].props.get("__text") {
            Some(RenderValue::Str(s)) => assert_eq!(s, "Not found"),
            _ => panic!("expected text prop"),
        }
    }

    #[test]
    fn test_has_file_extension() {
        assert!(has_file_extension("style.css"));
        assert!(has_file_extension("naze_runtime.js"));
        assert!(has_file_extension("app_data.bin"));
        assert!(has_file_extension("images/logo.png"));
        assert!(!has_file_extension("about"));
        assert!(!has_file_extension("docs/intro"));
        assert!(!has_file_extension(""));
    }

    #[test]
    fn test_route_to_title() {
        assert_eq!(route_to_title("/", "My App"), "My App");
        assert_eq!(route_to_title("/about", "My App"), "About - My App");
        assert_eq!(
            route_to_title("/user-profile", "My App"),
            "User Profile - My App"
        );
    }

    #[test]
    fn test_server_fn_pre_evaluation() {
        let mut tree = empty_tree();
        tree.server_functions.push(ServerFuncDecl {
            name: "get-data".to_string(),
            params: vec![],
            body: IrServerBody {
                lets: vec![],
                result: IrExpression::Str("pre-evaluated result".to_string()),
            },
        });
        tree.server_calls.push(ServerCallDecl {
            name: "mydata".to_string(),
            func_name: "get-data".to_string(),
            args: vec![],
        });
        // Add a conditional node that checks mydata.loading
        tree.root = vec![text_node("static content")];

        // Simulate what render_page does
        let mut app_state = exec::init_state(&tree);

        // Pre-evaluate server calls
        for call in &tree.server_calls {
            if let Some(func) = tree
                .server_functions
                .iter()
                .find(|f| f.name == call.func_name)
            {
                let args_json: Vec<serde_json::Value> = call
                    .args
                    .iter()
                    .map(|a| {
                        let val = exec::evaluate_expr(a, &app_state);
                        crate::server_fns::render_value_to_json(&val)
                    })
                    .collect();
                let result_json = crate::server_fns::evaluate_server_fn(func, &args_json);
                let result = crate::server_fns::json_to_render_value(&result_json);
                app_state.insert(format!("{}.loading", call.name), RenderValue::Bool(false));
                app_state.insert(format!("{}.data", call.name), result);
                app_state.insert(
                    format!("{}.error", call.name),
                    RenderValue::Str(String::new()),
                );
            }
        }

        // Verify pre-evaluated state
        assert_eq!(
            app_state.get("mydata.loading"),
            Some(&RenderValue::Bool(false))
        );
        assert_eq!(
            app_state.get("mydata.data"),
            Some(&RenderValue::Str("pre-evaluated result".to_string()))
        );
        assert_eq!(
            app_state.get("mydata.error"),
            Some(&RenderValue::Str(String::new()))
        );
    }

    #[test]
    fn test_hydration_state_serialized() {
        let mut state = HashMap::new();
        state.insert("count".to_string(), RenderValue::Num(42.0, None));
        state.insert("name".to_string(), RenderValue::Str("test".to_string()));

        let json_str = serialize_state_for_hydration(&state);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert!(parsed.is_object());
        assert_eq!(parsed["count"], serde_json::json!(42.0));
        assert_eq!(parsed["name"], serde_json::json!("test"));
    }
}
