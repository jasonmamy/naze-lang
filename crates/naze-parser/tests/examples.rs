use naze_parser::{parse, parse_test_file};
use std::fs;
use std::path::Path;

fn parse_example(name: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join(name);
    let source = fs::read_to_string(&path).unwrap_or_else(|e| panic!("can't read {name}: {e}"));
    let nodes = parse(&source, name).unwrap_or_else(|e| panic!("parse failed for {name}: {e}"));
    assert!(!nodes.is_empty(), "{name} produced no AST nodes");

    // Verify JSON serialization round-trips
    let json = serde_json::to_string_pretty(&nodes).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn parse_hello() {
    parse_example("hello.naze");
}

#[test]
fn parse_boxes() {
    parse_example("boxes.naze");
}

#[test]
fn parse_columns() {
    parse_example("columns.naze");
}

#[test]
fn parse_rows() {
    parse_example("rows.naze");
}

#[test]
fn parse_nested() {
    parse_example("nested.naze");
}

#[test]
fn parse_padding() {
    parse_example("padding.naze");
}

#[test]
fn parse_rounded() {
    parse_example("rounded.naze");
}

#[test]
fn parse_colors() {
    parse_example("colors.naze");
}

#[test]
fn parse_typography() {
    parse_example("typography.naze");
}

#[test]
fn parse_grid() {
    parse_example("grid.naze");
}

#[test]
fn parse_component_color_box() {
    parse_example("components/color-box.naze");
}

#[test]
fn parse_component_card() {
    parse_example("components/card.naze");
}

#[test]
fn parse_component_basic() {
    parse_example("component-basic.naze");
}

#[test]
fn parse_component_props() {
    parse_example("component-props.naze");
}

#[test]
fn parse_dashboard_static() {
    parse_example("dashboard-static.naze");
}

#[test]
fn parse_app_shell() {
    parse_example("app-shell.naze");
}

#[test]
fn parse_multi_component() {
    parse_example("multi-component.naze");
}

#[test]
fn parse_counter() {
    parse_example("counter.naze");
}

#[test]
fn parse_overlay_dialog() {
    parse_example("overlay-dialog.naze");
}

#[test]
fn parse_overlay_dropdown() {
    parse_example("overlay-dropdown.naze");
}

#[test]
fn parse_computed() {
    parse_example("computed.naze");
}

#[test]
fn parse_storage() {
    parse_example("storage.naze");
}

#[test]
fn parse_data_enhanced() {
    parse_example("data-enhanced.naze");
}

#[test]
fn parse_timer() {
    parse_example("timer.naze");
}

#[test]
fn parse_actions() {
    parse_example("actions.naze");
}

#[test]
fn parse_stream() {
    parse_example("stream.naze");
}

#[test]
fn parse_params() {
    parse_example("params.naze");
}

#[test]
fn parse_shared_state() {
    parse_example("shared-state.naze");
}

#[test]
fn parse_debounce() {
    parse_example("debounce.naze");
}

#[test]
fn parse_file_input() {
    parse_example("file-input.naze");
}

#[test]
fn parse_text_decoration() {
    parse_example("text-decoration.naze");
}

#[test]
fn parse_shadow() {
    parse_example("shadow.naze");
}

#[test]
fn parse_text_alignment() {
    parse_example("text-alignment.naze");
}

#[test]
fn parse_text_overflow() {
    parse_example("text-overflow.naze");
}

#[test]
fn parse_gradient() {
    parse_example("gradient.naze");
}

#[test]
fn parse_transform() {
    parse_example("transform.naze");
}

#[test]
fn parse_visual_properties() {
    parse_example("visual-properties.naze");
}

#[test]
fn parse_pipeline() {
    parse_example("pipeline.naze");
}

#[test]
fn parse_pipeline_advanced() {
    parse_example("pipeline-advanced.naze");
}

#[test]
fn parse_functions() {
    parse_example("functions.naze");
}

#[test]
fn parse_match() {
    parse_example("match.naze");
}

#[test]
fn parse_template_basic() {
    parse_example("template-basic.naze");
}

#[test]
fn parse_responsive_layout() {
    parse_example("responsive-layout.naze");
}

// ─── Test file parsing (.test.naze) ──────────────────────────────────────────

fn parse_test_example(name: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join(name);
    let source = fs::read_to_string(&path).unwrap_or_else(|e| panic!("can't read {name}: {e}"));
    let test_file =
        parse_test_file(&source, name).unwrap_or_else(|e| panic!("parse failed for {name}: {e}"));

    // Verify it produced test blocks
    assert!(
        !test_file.tests.is_empty() || !test_file.flows.is_empty(),
        "{name} produced no test or flow blocks"
    );

    // Verify JSON serialization round-trips
    let json = serde_json::to_string_pretty(&test_file).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn parse_test_counter() {
    parse_test_example("tests/counter.test.naze");
}

#[test]
fn parse_test_input() {
    parse_test_example("tests/input.test.naze");
}

#[test]
fn parse_test_inline() {
    // Parse a test file from an inline string to verify all step types
    let source = r#"
use counter

test "basic assertions" {
  render counter
  assert text "hello" is visible
  assert text "gone" is not visible
  click "Submit"
  fill "email" with "user@example.com"
  assert state count is 5
  navigate "/home"
  assert page is "/home"
  wait 100ms
}

flow "user journey" {
  render counter
  click "Start"
  assert text "welcome" is visible
}
"#;
    let test_file = parse_test_file(source, "inline.test.naze")
        .unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert_eq!(test_file.uses.len(), 1);
    assert_eq!(test_file.uses[0], "counter");
    assert_eq!(test_file.tests.len(), 1);
    assert_eq!(test_file.flows.len(), 1);
    assert_eq!(test_file.tests[0].name, "basic assertions");
    assert_eq!(test_file.tests[0].steps.len(), 9);
    assert_eq!(test_file.flows[0].name, "user journey");
    assert_eq!(test_file.flows[0].steps.len(), 3);
}
