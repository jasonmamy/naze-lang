use std::collections::HashMap;

use naze_ir::{RenderNode, RenderTree, RenderValue};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A positioned node with absolute coordinates, ready for rendering.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PositionedNode {
    pub kind: String,
    pub props: HashMap<String, RenderValue>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub children: Vec<PositionedNode>,
}

/// The result of layout computation: positioned tree + app title.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayoutTree {
    pub title: String,
    pub root: Vec<PositionedNode>,
}

/// Default text measurement: estimates width at ~0.6 * font_size per character.
fn default_text_measure(text: &str, font_size: f32) -> (f32, f32) {
    let width = text.len() as f32 * font_size * 0.6;
    let height = font_size * 1.2;
    (width, height)
}

/// Compute layout for a render tree using the given viewport size.
/// Uses a rough character-width estimate for text measurement.
pub fn compute_layout(
    tree: &RenderTree,
    viewport_width: f32,
    viewport_height: f32,
) -> LayoutTree {
    compute_layout_with_measure(tree, viewport_width, viewport_height, default_text_measure)
}

/// Compute layout with a custom text measurement function.
/// `text_measure(text, font_size) -> (width, height)` is called for every text/heading node.
pub fn compute_layout_with_measure<F>(
    tree: &RenderTree,
    viewport_width: f32,
    viewport_height: f32,
    text_measure: F,
) -> LayoutTree
where
    F: Fn(&str, f32) -> (f32, f32),
{
    let positioned: Vec<PositionedNode> = layout_children_column(
        &tree.root,
        0.0,
        0.0,
        viewport_width,
        viewport_height,
        &text_measure,
    );

    LayoutTree {
        title: tree.title.clone(),
        root: positioned,
    }
}

// ─── Custom minimal layout engine ────────────────────────────────────────────
// Supports: row, column, stack, grid, container, rect, text, heading, spacer.
// Handles: width/height, padding, gap. No flex-grow/shrink (except spacer).

/// Layout children in a column (vertical stacking).
fn layout_children_column<F: Fn(&str, f32) -> (f32, f32)>(
    nodes: &[RenderNode],
    x: f32,
    y: f32,
    available_w: f32,
    available_h: f32,
    text_measure: &F,
) -> Vec<PositionedNode> {
    let mut out = Vec::with_capacity(nodes.len());
    let mut cursor_y = y;

    // First pass: compute sizes of non-spacer children and count spacers
    let mut child_sizes: Vec<(f32, f32)> = Vec::with_capacity(nodes.len());
    let mut total_fixed_h: f32 = 0.0;
    let mut spacer_count: u32 = 0;
    let gap = 0.0; // top-level has no gap

    for node in nodes {
        if node.kind == "spacer" && get_num_prop(node, "height").is_none() {
            spacer_count += 1;
            child_sizes.push((0.0, 0.0));
        } else {
            let (w, h) = measure_node(node, available_w, available_h, text_measure);
            child_sizes.push((w, h));
            total_fixed_h += h;
        }
    }

    let total_gaps = if nodes.len() > 1 {
        gap * (nodes.len() as f32 - 1.0)
    } else {
        0.0
    };
    let remaining = (available_h - total_fixed_h - total_gaps).max(0.0);
    let spacer_h = if spacer_count > 0 {
        remaining / spacer_count as f32
    } else {
        0.0
    };

    for (i, node) in nodes.iter().enumerate() {
        let (w, h) = if node.kind == "spacer" && get_num_prop(node, "height").is_none() {
            (available_w, spacer_h)
        } else {
            child_sizes[i]
        };

        let positioned = layout_node(node, x, cursor_y, w, h, available_w, available_h, text_measure);
        cursor_y += positioned.height;
        out.push(positioned);
    }

    out
}

/// Measure a node's intrinsic size (width, height) without positioning.
fn measure_node<F: Fn(&str, f32) -> (f32, f32)>(
    node: &RenderNode,
    available_w: f32,
    available_h: f32,
    text_measure: &F,
) -> (f32, f32) {
    let explicit_w = get_num_prop(node, "width").map(|v| v as f32);
    let explicit_h = get_num_prop(node, "height").map(|v| v as f32);
    let padding = get_num_prop(node, "padding").unwrap_or(0.0) as f32;
    let gap = get_num_prop(node, "gap").unwrap_or(0.0) as f32;

    match node.kind.as_str() {
        "text" | "heading" => {
            let text = get_text_content(node);
            let font_size = get_font_size(node);
            let (tw, th) = text_measure(&text, font_size);
            (explicit_w.unwrap_or(tw), explicit_h.unwrap_or(th))
        }
        "rect" => {
            (explicit_w.unwrap_or(0.0), explicit_h.unwrap_or(0.0))
        }
        "spacer" => {
            (explicit_w.unwrap_or(0.0), explicit_h.unwrap_or(0.0))
        }
        "row" => {
            let inner_w = explicit_w.unwrap_or(available_w) - padding * 2.0;
            let inner_h = explicit_h.map(|h| h - padding * 2.0).unwrap_or(available_h);
            let mut total_w: f32 = 0.0;
            let mut max_h: f32 = 0.0;
            for (i, child) in node.children.iter().enumerate() {
                let (cw, ch) = measure_node(child, inner_w, inner_h, text_measure);
                total_w += cw;
                if i > 0 {
                    total_w += gap;
                }
                max_h = max_h.max(ch);
            }
            let w = explicit_w.unwrap_or(total_w + padding * 2.0);
            let h = explicit_h.unwrap_or(max_h + padding * 2.0);
            (w, h)
        }
        "column" | "container" | "stack" => {
            let inner_w = explicit_w.unwrap_or(available_w) - padding * 2.0;
            let inner_h = explicit_h.map(|h| h - padding * 2.0).unwrap_or(available_h);
            let mut max_w: f32 = 0.0;
            let mut total_h: f32 = 0.0;
            for (i, child) in node.children.iter().enumerate() {
                let (cw, ch) = measure_node(child, inner_w, inner_h, text_measure);
                max_w = max_w.max(cw);
                total_h += ch;
                if i > 0 {
                    total_h += gap;
                }
            }
            let w = explicit_w.unwrap_or(max_w + padding * 2.0);
            let h = explicit_h.unwrap_or(total_h + padding * 2.0);
            (w, h)
        }
        "grid" => {
            let cols = get_num_prop(node, "columns").unwrap_or(2.0) as usize;
            let inner_w = explicit_w.unwrap_or(available_w) - padding * 2.0;
            let col_w = if cols > 0 {
                (inner_w - gap * (cols as f32 - 1.0).max(0.0)) / cols as f32
            } else {
                inner_w
            };
            let mut row_h: f32 = 0.0;
            let mut total_h: f32 = 0.0;
            let mut rows: usize = 0;
            for (i, child) in node.children.iter().enumerate() {
                let (_cw, ch) = measure_node(child, col_w, available_h, text_measure);
                row_h = row_h.max(ch);
                if (i + 1) % cols == 0 || i == node.children.len() - 1 {
                    if rows > 0 {
                        total_h += gap;
                    }
                    total_h += row_h;
                    row_h = 0.0;
                    rows += 1;
                }
            }
            let w = explicit_w.unwrap_or(inner_w + padding * 2.0);
            let h = explicit_h.unwrap_or(total_h + padding * 2.0);
            (w, h)
        }
        _ => {
            // Unknown: treat like column
            let inner_w = explicit_w.unwrap_or(available_w) - padding * 2.0;
            let inner_h = explicit_h.map(|h| h - padding * 2.0).unwrap_or(available_h);
            let mut max_w: f32 = 0.0;
            let mut total_h: f32 = 0.0;
            for child in &node.children {
                let (cw, ch) = measure_node(child, inner_w, inner_h, text_measure);
                max_w = max_w.max(cw);
                total_h += ch;
            }
            let w = explicit_w.unwrap_or(max_w + padding * 2.0);
            let h = explicit_h.unwrap_or(total_h + padding * 2.0);
            (w, h)
        }
    }
}

/// Layout a single node at the given position with the given size.
fn layout_node<F: Fn(&str, f32) -> (f32, f32)>(
    node: &RenderNode,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    _available_w: f32,
    available_h: f32,
    text_measure: &F,
) -> PositionedNode {
    let padding = get_num_prop(node, "padding").unwrap_or(0.0) as f32;
    let gap = get_num_prop(node, "gap").unwrap_or(0.0) as f32;

    let children = match node.kind.as_str() {
        "text" | "heading" | "rect" | "spacer" => Vec::new(),
        "row" => {
            let inner_x = x + padding;
            let inner_y = y + padding;
            let inner_w = width - padding * 2.0;
            let inner_h = height - padding * 2.0;
            layout_row(&node.children, inner_x, inner_y, inner_w, inner_h, gap, text_measure)
        }
        "grid" => {
            let inner_x = x + padding;
            let inner_y = y + padding;
            let inner_w = width - padding * 2.0;
            let cols = get_num_prop(node, "columns").unwrap_or(2.0) as usize;
            layout_grid(&node.children, inner_x, inner_y, inner_w, available_h, cols, gap, text_measure)
        }
        _ => {
            // column, container, stack, unknown — vertical stacking
            let inner_x = x + padding;
            let inner_y = y + padding;
            let inner_w = width - padding * 2.0;
            let inner_h = height - padding * 2.0;
            layout_column(&node.children, inner_x, inner_y, inner_w, inner_h, gap, text_measure)
        }
    };

    PositionedNode {
        kind: node.kind.clone(),
        props: node.props.clone(),
        x,
        y,
        width,
        height,
        children,
    }
}

/// Layout children in a column with gap.
fn layout_column<F: Fn(&str, f32) -> (f32, f32)>(
    nodes: &[RenderNode],
    x: f32,
    y: f32,
    available_w: f32,
    available_h: f32,
    gap: f32,
    text_measure: &F,
) -> Vec<PositionedNode> {
    let mut out = Vec::with_capacity(nodes.len());
    let mut cursor_y = y;

    // Count spacers and measure fixed children
    let mut child_sizes: Vec<(f32, f32)> = Vec::with_capacity(nodes.len());
    let mut total_fixed_h: f32 = 0.0;
    let mut spacer_count: u32 = 0;

    for node in nodes {
        if node.kind == "spacer" && get_num_prop(node, "height").is_none() {
            spacer_count += 1;
            child_sizes.push((0.0, 0.0));
        } else {
            let (w, h) = measure_node(node, available_w, available_h, text_measure);
            child_sizes.push((w, h));
            total_fixed_h += h;
        }
    }

    let total_gaps = if nodes.len() > 1 {
        gap * (nodes.len() as f32 - 1.0)
    } else {
        0.0
    };
    let remaining = (available_h - total_fixed_h - total_gaps).max(0.0);
    let spacer_h = if spacer_count > 0 {
        remaining / spacer_count as f32
    } else {
        0.0
    };

    for (i, node) in nodes.iter().enumerate() {
        if i > 0 {
            cursor_y += gap;
        }
        let (w, h) = if node.kind == "spacer" && get_num_prop(node, "height").is_none() {
            (available_w, spacer_h)
        } else {
            child_sizes[i]
        };

        let positioned = layout_node(node, x, cursor_y, w, h, available_w, available_h, text_measure);
        cursor_y += positioned.height;
        out.push(positioned);
    }

    out
}

/// Layout children in a row with gap.
fn layout_row<F: Fn(&str, f32) -> (f32, f32)>(
    nodes: &[RenderNode],
    x: f32,
    y: f32,
    available_w: f32,
    available_h: f32,
    gap: f32,
    text_measure: &F,
) -> Vec<PositionedNode> {
    let mut out = Vec::with_capacity(nodes.len());
    let mut cursor_x = x;

    // Count spacers and measure fixed children
    let mut child_sizes: Vec<(f32, f32)> = Vec::with_capacity(nodes.len());
    let mut total_fixed_w: f32 = 0.0;
    let mut spacer_count: u32 = 0;

    for node in nodes {
        if node.kind == "spacer" && get_num_prop(node, "width").is_none() {
            spacer_count += 1;
            child_sizes.push((0.0, 0.0));
        } else {
            let (w, h) = measure_node(node, available_w, available_h, text_measure);
            child_sizes.push((w, h));
            total_fixed_w += w;
        }
    }

    let total_gaps = if nodes.len() > 1 {
        gap * (nodes.len() as f32 - 1.0)
    } else {
        0.0
    };
    let remaining = (available_w - total_fixed_w - total_gaps).max(0.0);
    let spacer_w = if spacer_count > 0 {
        remaining / spacer_count as f32
    } else {
        0.0
    };

    for (i, node) in nodes.iter().enumerate() {
        if i > 0 {
            cursor_x += gap;
        }
        let (w, h) = if node.kind == "spacer" && get_num_prop(node, "width").is_none() {
            (spacer_w, available_h)
        } else {
            child_sizes[i]
        };

        let positioned = layout_node(node, cursor_x, y, w, h, available_w, available_h, text_measure);
        cursor_x += positioned.width;
        out.push(positioned);
    }

    out
}

/// Layout children in a grid.
fn layout_grid<F: Fn(&str, f32) -> (f32, f32)>(
    nodes: &[RenderNode],
    x: f32,
    y: f32,
    available_w: f32,
    available_h: f32,
    cols: usize,
    gap: f32,
    text_measure: &F,
) -> Vec<PositionedNode> {
    if cols == 0 || nodes.is_empty() {
        return Vec::new();
    }

    let col_w = (available_w - gap * (cols as f32 - 1.0).max(0.0)) / cols as f32;
    let mut out = Vec::with_capacity(nodes.len());

    // Compute row heights
    let num_rows = (nodes.len() + cols - 1) / cols;
    let mut row_heights = vec![0.0f32; num_rows];
    for (i, node) in nodes.iter().enumerate() {
        let row = i / cols;
        let (_cw, ch) = measure_node(node, col_w, available_h, text_measure);
        row_heights[row] = row_heights[row].max(ch);
    }

    let mut cursor_y = y;
    for (i, node) in nodes.iter().enumerate() {
        let row = i / cols;
        let col = i % cols;

        if col == 0 && row > 0 {
            cursor_y += row_heights[row - 1] + gap;
        }

        let cx = x + col as f32 * (col_w + gap);
        let (w, h) = measure_node(node, col_w, available_h, text_measure);
        let positioned = layout_node(node, cx, cursor_y, w, h, col_w, available_h, text_measure);
        out.push(positioned);
    }

    out
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn get_num_prop(node: &RenderNode, key: &str) -> Option<f64> {
    match node.props.get(key) {
        Some(RenderValue::Num(n, _)) => Some(*n),
        _ => None,
    }
}

fn get_text_content(node: &RenderNode) -> String {
    match node.props.get("__text") {
        Some(RenderValue::Str(s)) => s.clone(),
        _ => String::new(),
    }
}

fn get_font_size(node: &RenderNode) -> f32 {
    match node.props.get("font-size") {
        Some(RenderValue::Num(n, _)) => *n as f32,
        _ => match node.kind.as_str() {
            "heading" => 24.0,
            _ => 16.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use naze_compiler::codegen;
    use naze_compiler::resolve::resolve;
    use std::fs;

    fn setup_and_layout(files: &[(&str, &str)], vw: f32, vh: f32) -> LayoutTree {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            let path = dir.path().join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }
        let project = resolve(dir.path(), "app.naze");
        assert!(
            project.errors.is_empty(),
            "resolve errors: {:?}",
            project.errors
        );
        let render_tree = codegen::lower(&project);
        compute_layout(&render_tree, vw, vh)
    }

    #[test]
    fn layout_simple_text() {
        let tree = setup_and_layout(
            &[(
                "app.naze",
                r#"app "Hello" {
  text "world"
}"#,
            )],
            800.0,
            600.0,
        );
        assert_eq!(tree.title, "Hello");
        assert_eq!(tree.root.len(), 1);
        let text = &tree.root[0];
        assert_eq!(text.kind, "text");
        assert!(text.width > 0.0, "text should have positive width");
        assert!(text.height > 0.0, "text should have positive height");
    }

    #[test]
    fn layout_rect_fixed_size() {
        let tree = setup_and_layout(
            &[(
                "app.naze",
                r#"app "Test" {
  rect width: 100px, height: 50px, color: #ff0000
}"#,
            )],
            800.0,
            600.0,
        );
        let rect = &tree.root[0];
        assert_eq!(rect.kind, "rect");
        assert!((rect.width - 100.0).abs() < 1.0, "width={}", rect.width);
        assert!((rect.height - 50.0).abs() < 1.0, "height={}", rect.height);
    }

    #[test]
    fn layout_column_stacks_vertically() {
        let tree = setup_and_layout(
            &[(
                "app.naze",
                r#"app "Test" {
  column {
    rect width: 100px, height: 30px, color: #ff0000
    rect width: 100px, height: 30px, color: #00ff00
  }
}"#,
            )],
            800.0,
            600.0,
        );
        let col = &tree.root[0];
        assert_eq!(col.children.len(), 2);
        let r1 = &col.children[0];
        let r2 = &col.children[1];
        // Second rect should be below the first
        assert!(
            r2.y > r1.y,
            "r2.y ({}) should be > r1.y ({})",
            r2.y,
            r1.y
        );
        assert!((r2.y - r1.y - 30.0).abs() < 1.0, "gap between rects: expected 30, got {}", r2.y - r1.y);
    }

    #[test]
    fn layout_row_stacks_horizontally() {
        let tree = setup_and_layout(
            &[(
                "app.naze",
                r#"app "Test" {
  row {
    rect width: 50px, height: 30px, color: #ff0000
    rect width: 50px, height: 30px, color: #00ff00
  }
}"#,
            )],
            800.0,
            600.0,
        );
        let row = &tree.root[0];
        assert_eq!(row.children.len(), 2);
        let r1 = &row.children[0];
        let r2 = &row.children[1];
        // Second rect should be to the right of the first
        assert!(
            r2.x > r1.x,
            "r2.x ({}) should be > r1.x ({})",
            r2.x,
            r1.x
        );
        // Same y position
        assert!(
            (r2.y - r1.y).abs() < 1.0,
            "rects should be at same y: {} vs {}",
            r1.y,
            r2.y
        );
    }

    #[test]
    fn layout_padding_offsets_children() {
        let tree = setup_and_layout(
            &[(
                "app.naze",
                r#"app "Test" {
  column padding: 20px {
    rect width: 50px, height: 30px, color: #ff0000
  }
}"#,
            )],
            800.0,
            600.0,
        );
        let col = &tree.root[0];
        let rect = &col.children[0];
        // Rect should be offset by at least the padding amount
        assert!(
            rect.x >= col.x + 20.0 - 1.0,
            "rect.x ({}) should be >= col.x ({}) + 20",
            rect.x,
            col.x
        );
        assert!(
            rect.y >= col.y + 20.0 - 1.0,
            "rect.y ({}) should be >= col.y ({}) + 20",
            rect.y,
            col.y
        );
    }

    #[test]
    fn layout_gap_between_children() {
        let tree = setup_and_layout(
            &[(
                "app.naze",
                r#"app "Test" {
  column gap: 10px {
    rect width: 50px, height: 20px, color: #ff0000
    rect width: 50px, height: 20px, color: #00ff00
  }
}"#,
            )],
            800.0,
            600.0,
        );
        let col = &tree.root[0];
        let r1 = &col.children[0];
        let r2 = &col.children[1];
        let actual_gap = r2.y - r1.y - r1.height;
        assert!(
            (actual_gap - 10.0).abs() < 1.0,
            "gap should be 10px, got {}",
            actual_gap
        );
    }

    #[test]
    fn layout_grid_basic() {
        let tree = setup_and_layout(
            &[(
                "app.naze",
                r#"app "Test" {
  grid columns: 2 {
    rect width: 50px, height: 30px, color: #ff0000
    rect width: 50px, height: 30px, color: #00ff00
    rect width: 50px, height: 30px, color: #0000ff
    rect width: 50px, height: 30px, color: #ffff00
  }
}"#,
            )],
            800.0,
            600.0,
        );
        let grid = &tree.root[0];
        assert_eq!(grid.children.len(), 4);
        // Items 0 and 1 should be on the same row (same y)
        let r0 = &grid.children[0];
        let r1 = &grid.children[1];
        assert!(
            (r0.y - r1.y).abs() < 1.0,
            "first row items should have same y: {} vs {}",
            r0.y,
            r1.y
        );
        // Item 1 should be to the right of item 0
        assert!(r1.x > r0.x, "r1.x ({}) > r0.x ({})", r1.x, r0.x);
        // Item 2 should be below item 0
        let r2 = &grid.children[2];
        assert!(r2.y > r0.y, "r2.y ({}) > r0.y ({})", r2.y, r0.y);
    }

    #[test]
    fn layout_inlined_component() {
        let tree = setup_and_layout(
            &[
                (
                    "components/box.naze",
                    "component box(color: color) {\n  rect width: 80px, height: 80px, color: color\n}\n",
                ),
                (
                    "app.naze",
                    "use components/box\n\napp \"Test\" {\n  box color: #ff0000\n}\n",
                ),
            ],
            800.0,
            600.0,
        );
        // Component inlined to rect
        assert_eq!(tree.root.len(), 1);
        let rect = &tree.root[0];
        assert_eq!(rect.kind, "rect");
        assert!((rect.width - 80.0).abs() < 1.0);
        assert!((rect.height - 80.0).abs() < 1.0);
    }

    #[test]
    fn layout_all_examples() {
        let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples");

        for name in &[
            "hello.naze",
            "boxes.naze",
            "columns.naze",
            "rows.naze",
            "nested.naze",
            "padding.naze",
            "rounded.naze",
            "colors.naze",
            "typography.naze",
            "grid.naze",
            "dashboard-static.naze",
            "app-shell.naze",
            "component-basic.naze",
            "component-props.naze",
            "multi-component.naze",
        ] {
            let project = resolve(&examples_dir, name);
            let render_tree = codegen::lower(&project);
            let layout = compute_layout(&render_tree, 1024.0, 768.0);
            assert!(!layout.root.is_empty(), "empty layout for {}", name);

            // All nodes should have non-negative coordinates
            fn check_positions(nodes: &[PositionedNode], file: &str) {
                for node in nodes {
                    assert!(
                        node.x >= 0.0 && node.y >= 0.0,
                        "negative position in {}: {:?} at ({}, {})",
                        file,
                        node.kind,
                        node.x,
                        node.y
                    );
                    check_positions(&node.children, file);
                }
            }
            check_positions(&layout.root, name);
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn layout_serializes_roundtrip() {
        let tree = setup_and_layout(
            &[(
                "app.naze",
                r#"app "Hello" {
  column padding: 10px {
    text "hi"
    rect width: 50px, height: 30px, color: #ff0000
  }
}"#,
            )],
            800.0,
            600.0,
        );
        let json = serde_json::to_string(&tree).unwrap();
        let restored: LayoutTree = serde_json::from_str(&json).unwrap();
        assert_eq!(tree, restored);
    }
}
