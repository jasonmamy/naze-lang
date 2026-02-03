//! Build and serve an example gallery for interactive browsing.

use std::fs;
use std::path::{Path, PathBuf};

use naze_compiler::codegen;
use naze_compiler::error::Severity;
use naze_compiler::resolve;
use naze_compiler::typecheck;

// Embed the pre-built runtime files from wasm-pack output.
const RUNTIME_WASM: &[u8] = include_bytes!("../../naze-runtime/pkg/naze_runtime_bg.wasm");
const RUNTIME_JS: &str = include_str!("../../naze-runtime/pkg/naze_runtime.js");

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

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "naze" {
                    if let Some(stem) = path.file_stem() {
                        examples.push(stem.to_string_lossy().to_string());
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
    eprintln!("  cd {} && python3 -m http.server 8000", output_dir.display());
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

/// Main entry point for the gallery command.
pub fn run(build_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    let examples_dir = find_examples_dir()?;
    let output_dir = examples_dir.join("dist");

    eprintln!("Building example gallery...");
    eprintln!("  examples: {}", examples_dir.display());
    eprintln!("  output: {}", output_dir.display());

    // Clean and create output directory
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)?;
    }
    fs::create_dir_all(&output_dir)?;

    // Find all examples
    let examples = find_examples(&examples_dir)?;
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
