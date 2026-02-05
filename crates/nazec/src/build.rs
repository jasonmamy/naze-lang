use std::fs;
use std::path::Path;

use naze_compiler::codegen;
use naze_compiler::error::{CompileError, Severity};
use naze_compiler::resolve;
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
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html, body { width: 100%; height: 100%; overflow: hidden; background: #fff; }
    canvas { display: block; }
  </style>
</head>
<body>
  <canvas id="naze-canvas"></canvas>
  <script type="module">
    import init, { start } from './naze_runtime.js';
    async function main() {
      await init();
      const resp = await fetch('./app_data.bin');
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

/// Run the full build pipeline: parse -> resolve -> typecheck -> serialize -> write dist/
pub fn run(manifest: &Manifest, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = Path::new(".");
    let entry = &manifest.build.entry;
    let output_dir = Path::new(&manifest.build.output);

    let mut diag = DiagnosticPrinter::new(format);

    // 1. Resolve: parse all .naze files, resolve imports
    if format == Format::Text {
        eprintln!("  resolving...");
    }
    let project = resolve::resolve(project_dir, entry);

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

    // 3. Lower to render tree and serialize
    if format == Format::Text {
        eprintln!("  compiling...");
    }
    let render_tree = codegen::lower(&project);
    let app_data = naze_ir::serialize(&render_tree);

    // 4. Write dist/
    if format == Format::Text {
        eprintln!("  writing {}...", output_dir.display());
    }
    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("app_data.bin"), &app_data)?;
    fs::write(output_dir.join("naze_runtime_bg.wasm"), RUNTIME_WASM)?;
    fs::write(output_dir.join("naze_runtime.js"), RUNTIME_JS)?;

    let html = INDEX_HTML_TEMPLATE.replace("{{TITLE}}", &render_tree.title);
    fs::write(output_dir.join("index.html"), html)?;

    if format == Format::Text {
        let wasm_kb = RUNTIME_WASM.len() / 1024;
        let data_bytes = app_data.len();
        eprintln!("  done: runtime {}KB + app data {}B", wasm_kb, data_bytes);
    }

    Ok(())
}

/// Type-check only (no output).
pub fn check(manifest: &Manifest, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = Path::new(".");
    let entry = &manifest.build.entry;

    let mut diag = DiagnosticPrinter::new(format);

    // 1. Resolve
    let project = resolve::resolve(project_dir, entry);
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
