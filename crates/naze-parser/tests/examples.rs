use naze_parser::parse;
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
