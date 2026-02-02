use std::path::Path;

use naze_compiler::codegen;
use naze_compiler::error::Severity;
use naze_compiler::resolve;
use naze_compiler::typecheck;

const WASM_SIZE_LIMIT: usize = 100 * 1024; // 100KB

/// Embedded runtime WASM — same as what nazec embeds.
const RUNTIME_WASM: &[u8] =
    include_bytes!("../../naze-runtime/pkg/naze_runtime_bg.wasm");

fn examples_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
}

/// Full pipeline test: resolve → typecheck → lower → serialize → verify roundtrip.
fn build_example(name: &str) {
    let dir = examples_dir();
    let project = resolve::resolve(&dir, name);

    // No resolution errors
    let resolve_errors: Vec<_> = project
        .errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .collect();
    assert!(
        resolve_errors.is_empty(),
        "resolve errors in {name}: {resolve_errors:?}"
    );

    // No type errors
    let tc_errors = typecheck::typecheck(&project);
    let tc_hard: Vec<_> = tc_errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .collect();
    assert!(
        tc_hard.is_empty(),
        "type errors in {name}: {tc_hard:?}"
    );

    // Lower and serialize
    let tree = codegen::lower(&project);
    assert!(!tree.title.is_empty(), "empty title in {name}");
    assert!(!tree.root.is_empty(), "empty root in {name}");

    let bytes = naze_ir::serialize(&tree);
    assert!(bytes.len() < 64 * 1024, "app_data too large for {name}: {}B", bytes.len());

    // Roundtrip
    let restored = naze_ir::deserialize(&bytes).expect("deserialize failed");
    assert_eq!(tree, restored, "roundtrip failed for {name}");
}

// --- Self-contained examples (no component imports) ---

#[test]
fn build_hello() { build_example("hello.naze"); }

#[test]
fn build_boxes() { build_example("boxes.naze"); }

#[test]
fn build_columns() { build_example("columns.naze"); }

#[test]
fn build_rows() { build_example("rows.naze"); }

#[test]
fn build_nested() { build_example("nested.naze"); }

#[test]
fn build_padding() { build_example("padding.naze"); }

#[test]
fn build_rounded() { build_example("rounded.naze"); }

#[test]
fn build_colors() { build_example("colors.naze"); }

#[test]
fn build_typography() { build_example("typography.naze"); }

#[test]
fn build_grid() { build_example("grid.naze"); }

#[test]
fn build_dashboard_static() { build_example("dashboard-static.naze"); }

#[test]
fn build_app_shell() { build_example("app-shell.naze"); }

// --- Examples with component imports ---

#[test]
fn build_component_basic() { build_example("component-basic.naze"); }

#[test]
fn build_component_props() { build_example("component-props.naze"); }

#[test]
fn build_multi_component() { build_example("multi-component.naze"); }

// --- Runtime WASM size check ---

#[test]
fn runtime_wasm_under_100kb() {
    assert!(
        RUNTIME_WASM.len() < WASM_SIZE_LIMIT,
        "runtime WASM is {}KB, exceeds {}KB limit",
        RUNTIME_WASM.len() / 1024,
        WASM_SIZE_LIMIT / 1024
    );
}
