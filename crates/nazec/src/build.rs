use std::fs;
use std::path::Path;
use std::time::Instant;

use naze_compiler::codegen;
use naze_compiler::error::{CompileError, Severity};
use naze_compiler::resolve::{self, BuildCache, ResolvedDep};
use naze_compiler::typecheck;

use crate::diagnostic::{DiagnosticPrinter, Format};
use crate::manifest::Manifest;

// Embed the pre-built runtime files from wasm-pack output.
// These must be built first via `wasm-pack build crates/naze-runtime --target web`.
const RUNTIME_WASM: &[u8] = include_bytes!("../../naze-runtime/pkg/naze_runtime_bg.wasm");
const RUNTIME_JS: &str = include_str!("../../naze-runtime/pkg/naze_runtime.js");

const INDEX_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{TITLE}}</title>
{{META_TAGS}}
{{JSON_LD}}
  <link rel="alternate" type="application/naze" href="{{ASSET_PREFIX}}/app_data.bin">
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html, body { width: 100%; height: 100%; overflow: hidden; background: #fff; }
    canvas { display: block; touch-action: none; }
  </style>
</head>
<body>
  <canvas id="naze-canvas"></canvas>
  <noscript>{{NOSCRIPT}}</noscript>
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
    }
    main().catch(e => {
      document.body.innerHTML = '<pre style="color:red;padding:20px">' + e + '</pre>';
    });
  </script>
</body>
</html>
"#;

const STATIC_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{TITLE}}</title>
{{META_TAGS}}
{{JSON_LD}}
  <link rel="alternate" type="application/naze" href="{{ASSET_PREFIX}}/app_data.bin">
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

/// Run the full build pipeline: parse -> resolve -> typecheck -> serialize -> write dist/
pub fn run(
    manifest: &Manifest,
    format: Format,
    deps: &[ResolvedDep],
    static_render: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let project_dir = Path::new(".");
    let entry = &manifest.build.entry;
    let output_dir = Path::new(&manifest.build.output);

    let mut diag = DiagnosticPrinter::new(format);

    // 1. Resolve: parse all .naze files, resolve imports
    if format == Format::Text {
        eprintln!("  resolving...");
    }
    let project = resolve::resolve(project_dir, entry, deps);

    // Collect all errors from resolve phase
    if !project.errors.is_empty() {
        diag.print_all(&project.errors);
        let has_errors = project
            .errors
            .iter()
            .any(|e| matches!(e.severity, Severity::Error));
        if has_errors {
            diag.print_summary(&project.errors);
            return Err("resolution failed".into());
        }
    }

    // 2. Type-check
    if format == Format::Text {
        eprintln!("  type checking...");
    }
    let tc_errors = typecheck::typecheck(&project);
    if !tc_errors.is_empty() {
        diag.print_all(&tc_errors);
        let has_errors = tc_errors
            .iter()
            .any(|e| matches!(e.severity, Severity::Error));
        if has_errors {
            diag.print_summary(&tc_errors);
            return Err("type checking failed".into());
        }
    }

    // 2b. Resolve environment variables
    let dotenv = crate::manifest::load_dotenv(".env");
    let (env_vars, missing_env) = crate::manifest::resolve_env_vars(manifest, &dotenv);
    if !missing_env.is_empty() {
        for name in &missing_env {
            diag.print_all(&[CompileError {
                message: format!(
                    "required environment variable '{}' is not set (declare in [env] section of naze.toml)",
                    name
                ),
                file: "naze.toml".to_string(),
                line: 0,
                column: 0,
                severity: Severity::Error,
            }]);
        }
        return Err("missing required environment variables".into());
    }

    // 3. Lower to render tree and serialize
    if format == Format::Text {
        eprintln!("  compiling...");
    }
    codegen::set_env_vars(env_vars);
    let render_tree = codegen::lower(&project);

    // Warn if server functions are used (require `nazec dev`, not supported in production builds)
    if !render_tree.server_functions.is_empty() && format == Format::Text {
        eprintln!("  warning: this app uses server functions — use `nazec serve` for production or `nazec dev` for development");
    }

    let (app_data, source_map) = naze_ir::serialize_with_source_map(&render_tree);

    // 4. Write dist/
    if format == Format::Text {
        eprintln!("  writing {}...", output_dir.display());
    }
    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("app_data.bin"), &app_data)?;
    fs::write(output_dir.join("naze_runtime_bg.wasm"), RUNTIME_WASM)?;
    fs::write(output_dir.join("naze_runtime.js"), RUNTIME_JS)?;

    // Write source map
    if !source_map.mappings.is_empty() {
        let map_json = serde_json::to_string_pretty(&source_map)?;
        fs::write(output_dir.join("app_data.map.json"), map_json)?;
    }

    // Copy imported .wasm files and generate JS bridge
    write_wasm_imports(output_dir, &render_tree, &project)?;

    let script_tags: String = manifest
        .scripts
        .values()
        .map(|url| format!("  <script src=\"{}\"></script>", url))
        .collect::<Vec<_>>()
        .join("\n");
    if static_render {
        if format == Format::Text {
            eprintln!("  rendering static HTML...");
        }
        write_static_html_files(output_dir, manifest, &render_tree, &script_tags)?;
    } else {
        write_html_files(output_dir, manifest, &render_tree, &script_tags)?;
    }

    if format == Format::Text {
        let wasm_kb = RUNTIME_WASM.len() / 1024;
        let data_bytes = app_data.len();
        let elapsed = start.elapsed().as_millis();
        let mode = if static_render { " (static)" } else { "" };
        eprintln!(
            "  done{}: runtime {}KB + app data {}B (built in {}ms)",
            mode, wasm_kb, data_bytes, elapsed
        );
    }

    Ok(())
}

/// Incremental build: reuses cached ASTs for unchanged files.
/// Identical to `run()` but uses `resolve_incremental()` with a persistent cache.
pub fn run_incremental(
    manifest: &Manifest,
    format: Format,
    cache: &mut BuildCache,
    deps: &[ResolvedDep],
    static_render: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let project_dir = Path::new(".");
    let entry = &manifest.build.entry;
    let output_dir = Path::new(&manifest.build.output);

    let mut diag = DiagnosticPrinter::new(format);

    if format == Format::Text {
        eprintln!("  resolving...");
    }
    let project = resolve::resolve_incremental(project_dir, entry, cache, deps);

    if !project.errors.is_empty() {
        diag.print_all(&project.errors);
        let has_errors = project
            .errors
            .iter()
            .any(|e| matches!(e.severity, Severity::Error));
        if has_errors {
            diag.print_summary(&project.errors);
            return Err("resolution failed".into());
        }
    }

    if format == Format::Text {
        eprintln!("  type checking...");
    }
    let tc_errors = typecheck::typecheck(&project);
    if !tc_errors.is_empty() {
        diag.print_all(&tc_errors);
        let has_errors = tc_errors
            .iter()
            .any(|e| matches!(e.severity, Severity::Error));
        if has_errors {
            diag.print_summary(&tc_errors);
            return Err("type checking failed".into());
        }
    }

    // Resolve env vars for incremental build
    let dotenv = crate::manifest::load_dotenv(".env");
    let (env_vars, missing_env) = crate::manifest::resolve_env_vars(manifest, &dotenv);
    if !missing_env.is_empty() {
        for name in &missing_env {
            diag.print_all(&[CompileError {
                message: format!("required environment variable '{}' is not set", name),
                file: "naze.toml".to_string(),
                line: 0,
                column: 0,
                severity: Severity::Error,
            }]);
        }
        return Err("missing required environment variables".into());
    }

    if format == Format::Text {
        eprintln!("  compiling...");
    }
    codegen::set_env_vars(env_vars);
    let render_tree = codegen::lower(&project);
    let (app_data, source_map) = naze_ir::serialize_with_source_map(&render_tree);

    if format == Format::Text {
        eprintln!("  writing {}...", output_dir.display());
    }
    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("app_data.bin"), &app_data)?;
    fs::write(output_dir.join("naze_runtime_bg.wasm"), RUNTIME_WASM)?;
    fs::write(output_dir.join("naze_runtime.js"), RUNTIME_JS)?;

    // Write source map
    if !source_map.mappings.is_empty() {
        let map_json = serde_json::to_string_pretty(&source_map)?;
        fs::write(output_dir.join("app_data.map.json"), map_json)?;
    }

    // Copy imported .wasm files and generate JS bridge
    write_wasm_imports(output_dir, &render_tree, &project)?;

    let script_tags: String = manifest
        .scripts
        .values()
        .map(|url| format!("  <script src=\"{}\"></script>", url))
        .collect::<Vec<_>>()
        .join("\n");
    if static_render {
        write_static_html_files(output_dir, manifest, &render_tree, &script_tags)?;
    } else {
        write_html_files(output_dir, manifest, &render_tree, &script_tags)?;
    }

    if format == Format::Text {
        let wasm_kb = RUNTIME_WASM.len() / 1024;
        let data_bytes = app_data.len();
        let elapsed = start.elapsed().as_millis();
        let mode = if static_render { " (static)" } else { "" };
        eprintln!(
            "  done{}: runtime {}KB + app data {}B (built in {}ms)",
            mode, wasm_kb, data_bytes, elapsed
        );
    }

    Ok(())
}

/// Write index.html and per-route HTML files with SEO metadata.
fn write_html_files(
    output_dir: &Path,
    manifest: &Manifest,
    render_tree: &naze_ir::RenderTree,
    script_tags: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::seo;

    let (wasm_imports_tag, wasm_imports_load) = wasm_import_html_parts(render_tree);

    // Root page
    let meta_tags = seo::generate_meta_tags(manifest, &render_tree.title, None);
    let json_ld = seo::generate_json_ld(manifest, &render_tree.title);
    let noscript_text = seo::extract_text_content(&render_tree.root);

    let html = INDEX_HTML_TEMPLATE
        .replace("{{TITLE}}", &render_tree.title)
        .replace("{{META_TAGS}}", &meta_tags)
        .replace("{{JSON_LD}}", &json_ld)
        .replace("{{NOSCRIPT}}", &seo::escape_html(&noscript_text))
        .replace("{{SCRIPTS}}", script_tags)
        .replace("{{WASM_IMPORTS}}", &wasm_imports_tag)
        .replace("{{WASM_IMPORTS_LOAD}}", &wasm_imports_load)
        .replace("{{ASSET_PREFIX}}", ".");
    fs::write(output_dir.join("index.html"), html)?;

    // Per-route pages
    for page in &render_tree.pages {
        if page.path == "/" {
            continue;
        }
        let page_title = route_to_title(&page.path, &render_tree.title);
        let meta_tags = seo::generate_meta_tags(manifest, &render_tree.title, Some(&page.path));
        let json_ld = seo::generate_json_ld(manifest, &page_title);
        let noscript_text = seo::extract_text_content(&page.root);
        let asset_prefix = seo::asset_prefix_for_route(&page.path);

        let html = INDEX_HTML_TEMPLATE
            .replace("{{TITLE}}", &page_title)
            .replace("{{META_TAGS}}", &meta_tags)
            .replace("{{JSON_LD}}", &json_ld)
            .replace("{{NOSCRIPT}}", &seo::escape_html(&noscript_text))
            .replace("{{SCRIPTS}}", script_tags)
            .replace("{{WASM_IMPORTS}}", &wasm_imports_tag)
            .replace("{{WASM_IMPORTS_LOAD}}", &wasm_imports_load)
            .replace("{{ASSET_PREFIX}}", &asset_prefix);

        let route_dir = output_dir.join(page.path.trim_start_matches('/'));
        fs::create_dir_all(&route_dir)?;
        fs::write(route_dir.join("index.html"), html)?;
    }

    Ok(())
}

/// Write index.html and per-route HTML files with pre-rendered static content.
fn write_static_html_files(
    output_dir: &Path,
    manifest: &Manifest,
    render_tree: &naze_ir::RenderTree,
    script_tags: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::html_renderer;
    use crate::seo;

    let (wasm_imports_tag, wasm_imports_load) = wasm_import_html_parts(render_tree);

    // Root page
    let static_content = html_renderer::generate_static_html(render_tree);
    let meta_tags = seo::generate_meta_tags(manifest, &render_tree.title, None);
    let json_ld = seo::generate_json_ld(manifest, &render_tree.title);

    let html = STATIC_HTML_TEMPLATE
        .replace("{{TITLE}}", &render_tree.title)
        .replace("{{META_TAGS}}", &meta_tags)
        .replace("{{JSON_LD}}", &json_ld)
        .replace("{{STATIC_CONTENT}}", &static_content)
        .replace("{{SCRIPTS}}", script_tags)
        .replace("{{WASM_IMPORTS}}", &wasm_imports_tag)
        .replace("{{WASM_IMPORTS_LOAD}}", &wasm_imports_load)
        .replace("{{ASSET_PREFIX}}", ".");
    fs::write(output_dir.join("index.html"), html)?;

    // Per-route pages
    for page in &render_tree.pages {
        if page.path == "/" {
            continue;
        }
        let page_title = route_to_title(&page.path, &render_tree.title);
        let static_content = html_renderer::generate_static_html_for_page(render_tree, &page.root);
        let meta_tags = seo::generate_meta_tags(manifest, &render_tree.title, Some(&page.path));
        let json_ld = seo::generate_json_ld(manifest, &page_title);
        let asset_prefix = seo::asset_prefix_for_route(&page.path);

        let html = STATIC_HTML_TEMPLATE
            .replace("{{TITLE}}", &page_title)
            .replace("{{META_TAGS}}", &meta_tags)
            .replace("{{JSON_LD}}", &json_ld)
            .replace("{{STATIC_CONTENT}}", &static_content)
            .replace("{{SCRIPTS}}", script_tags)
            .replace("{{WASM_IMPORTS}}", &wasm_imports_tag)
            .replace("{{WASM_IMPORTS_LOAD}}", &wasm_imports_load)
            .replace("{{ASSET_PREFIX}}", &asset_prefix);

        let route_dir = output_dir.join(page.path.trim_start_matches('/'));
        fs::create_dir_all(&route_dir)?;
        fs::write(route_dir.join("index.html"), html)?;
    }

    Ok(())
}

/// Derive a page title from a route path: "/about" → "About - My App"
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

/// Copy imported .wasm files to dist/ and generate the JS bridge loader.
fn write_wasm_imports(
    output_dir: &Path,
    render_tree: &naze_ir::RenderTree,
    project: &resolve::ResolvedProject,
) -> Result<(), Box<dyn std::error::Error>> {
    if render_tree.imports.is_empty() {
        return Ok(());
    }

    for (imp, resolved) in render_tree.imports.iter().zip(project.imports.iter()) {
        let dest = output_dir.join(&imp.wasm_url);
        fs::copy(&resolved.wasm_path, &dest)?;
    }

    let bridge_js = generate_wasm_bridge(&render_tree.imports);
    fs::write(output_dir.join("wasm_imports.js"), bridge_js)?;

    Ok(())
}

/// Generate the JS bridge that loads WASM modules and exposes `__naze_wasm_call`.
fn generate_wasm_bridge(imports: &[naze_ir::ImportDecl]) -> String {
    let mut js =
        String::from("// Auto-generated WASM import bridge\nconst __naze_modules = {};\n\n");

    js.push_str("async function loadWasmImports() {\n");
    for imp in imports {
        js.push_str(&format!(
            "  {{\n    const resp = await fetch('./{wasm_url}');\n    const {{ instance }} = await WebAssembly.instantiateStreaming(resp);\n    __naze_modules['{name}'] = instance.exports;\n  }}\n",
            wasm_url = imp.wasm_url,
            name = imp.name,
        ));
    }
    js.push_str("}\n\n");

    js.push_str(
        "window.__naze_wasm_call = function(module, func, args) {\n  \
         const mod = __naze_modules[module];\n  \
         if (!mod) { console.error('WASM module not loaded:', module); return 0; }\n  \
         const fn_ = mod[func];\n  \
         if (!fn_) { console.error('WASM function not found:', module + '.' + func); return 0; }\n  \
         return fn_(...args);\n\
         };\n",
    );

    js
}

/// Generate WASM import script tag and load call for HTML template.
fn wasm_import_html_parts(render_tree: &naze_ir::RenderTree) -> (String, String) {
    if render_tree.imports.is_empty() {
        return (String::new(), String::new());
    }
    let script_tag = "  <script src=\"wasm_imports.js\"></script>".to_string();
    let load_call = "await loadWasmImports();".to_string();
    (script_tag, load_call)
}

/// Type-check only (no output).
pub fn check(
    manifest: &Manifest,
    format: Format,
    deps: &[ResolvedDep],
) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = Path::new(".");
    let entry = &manifest.build.entry;

    let mut diag = DiagnosticPrinter::new(format);

    // 1. Resolve
    let project = resolve::resolve(project_dir, entry, deps);
    let mut all_errors: Vec<CompileError> = project.errors.clone();

    // 2. Type-check
    let tc_errors = typecheck::typecheck(&project);
    all_errors.extend(tc_errors);

    if !all_errors.is_empty() {
        diag.print_all(&all_errors);
        diag.print_summary(&all_errors);
    }

    let has_errors = all_errors
        .iter()
        .any(|e| matches!(e.severity, Severity::Error));

    if has_errors {
        return Err("check failed".into());
    }

    if format == Format::Text {
        eprintln!("  no errors");
    }
    Ok(())
}
