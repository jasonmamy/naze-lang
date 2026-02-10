use std::collections::HashMap;

use naze_ir::{IrEventHandler, RenderNode, RenderTree, RenderValue};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Information about scroll container content bounds.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ScrollInfo {
    pub content_width: f32,
    pub content_height: f32,
    pub overflow_x: bool,
    pub overflow_y: bool,
}

/// A positioned node with absolute coordinates, ready for rendering.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PositionedNode {
    pub kind: String,
    pub props: HashMap<String, RenderValue>,
    pub handlers: Vec<IrEventHandler>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub children: Vec<PositionedNode>,
    pub scroll_info: Option<ScrollInfo>,
}

/// The result of layout computation: positioned tree + app title.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayoutTree {
    pub title: String,
    pub root: Vec<PositionedNode>,
    /// Overlay nodes, rendered after root content (last = topmost).
    pub overlays: Vec<PositionedNode>,
}

/// Default text measurement: estimates width at ~0.6 * font_size per character.
fn default_text_measure(text: &str, font_size: f32) -> (f32, f32) {
    let width = text.len() as f32 * font_size * 0.6;
    let height = font_size * 1.2;
    (width, height)
}

/// Compute layout for a render tree using the given viewport size.
/// Uses a rough character-width estimate for text measurement.
pub fn compute_layout(tree: &RenderTree, viewport_width: f32, viewport_height: f32) -> LayoutTree {
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
    // Separate overlay nodes from normal flow
    let mut normal_nodes = Vec::new();
    let mut overlay_nodes = Vec::new();
    for node in &tree.root {
        if node.kind == "overlay" {
            overlay_nodes.push(node);
        } else {
            normal_nodes.push(node.clone());
        }
    }

    let positioned: Vec<PositionedNode> = layout_children_column(
        &normal_nodes,
        0.0,
        0.0,
        viewport_width,
        viewport_height,
        &text_measure,
    );

    // Position overlay nodes
    let overlays = layout_overlays(
        &overlay_nodes,
        &positioned,
        viewport_width,
        viewport_height,
        &text_measure,
    );

    LayoutTree {
        title: tree.title.clone(),
        root: positioned,
        overlays,
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

        let positioned = layout_node(
            node,
            x,
            cursor_y,
            w,
            h,
            available_w,
            available_h,
            text_measure,
        );
        cursor_y += positioned.height;
        out.push(positioned);
    }

    out
}

/// Apply min/max constraints to a dimension.
fn apply_constraints(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let mut result = value;
    if let Some(min_val) = min {
        result = result.max(min_val);
    }
    if let Some(max_val) = max {
        result = result.min(max_val);
    }
    result
}

/// Measure a node's intrinsic size (width, height) without positioning.
fn measure_node<F: Fn(&str, f32) -> (f32, f32)>(
    node: &RenderNode,
    available_w: f32,
    available_h: f32,
    text_measure: &F,
) -> (f32, f32) {
    // Resolve dimensions with percentage support
    let explicit_w = resolve_dimension(node, "width", available_w);
    let explicit_h = resolve_dimension(node, "height", available_h);
    let min_w = resolve_dimension(node, "min-width", available_w);
    let max_w = resolve_dimension(node, "max-width", available_w);
    let min_h = resolve_dimension(node, "min-height", available_h);
    let max_h = resolve_dimension(node, "max-height", available_h);
    let padding = get_num_prop(node, "padding").unwrap_or(0.0) as f32;
    let gap = get_num_prop(node, "gap").unwrap_or(0.0) as f32;

    let (w, h) = match node.kind.as_str() {
        "text" | "heading" => {
            let text = get_text_content(node);
            let font_size = get_font_size(node);
            let (tw, th) = text_measure(&text, font_size);
            let line_height = get_num_prop(node, "line-height")
                .map(|lh| lh as f32 * font_size)
                .unwrap_or(th);
            (explicit_w.unwrap_or(tw), explicit_h.unwrap_or(line_height))
        }
        "rect" => (explicit_w.unwrap_or(0.0), explicit_h.unwrap_or(0.0)),
        "image" => {
            // Images default to 100x100 if no explicit size
            (explicit_w.unwrap_or(100.0), explicit_h.unwrap_or(100.0))
        }
        "checkbox" => {
            // Checkbox: 20x20 box + 8px gap + label
            let label = get_text_content(node);
            let font_size = get_font_size(node);
            let (label_w, label_h) = text_measure(&label, font_size);
            let w = 20.0 + 8.0 + label_w;
            let h = label_h.max(20.0);
            (explicit_w.unwrap_or(w), explicit_h.unwrap_or(h))
        }
        "radio" => {
            // Radio: 20x20 circle + 8px gap + label
            let label = get_text_content(node);
            let font_size = get_font_size(node);
            let (label_w, label_h) = text_measure(&label, font_size);
            let w = 20.0 + 8.0 + label_w;
            let h = label_h.max(20.0);
            (explicit_w.unwrap_or(w), explicit_h.unwrap_or(h))
        }
        "input" => {
            // Text input: default width 200px, height based on font size + padding
            let font_size = get_font_size(node);
            let h = font_size + 16.0; // 8px padding top + bottom
            (explicit_w.unwrap_or(200.0), explicit_h.unwrap_or(h))
        }
        "select" => {
            // Dropdown select: default 200px wide, 32px tall
            (explicit_w.unwrap_or(200.0), explicit_h.unwrap_or(32.0))
        }
        "textarea" => {
            // Multi-line text input: default 200px wide, height based on rows
            let font_size = get_font_size(node);
            let rows = get_num_prop(node, "rows").unwrap_or(4.0) as f32;
            let line_height = get_num_prop(node, "line-height")
                .map(|lh| lh as f32 * font_size)
                .unwrap_or(font_size * 1.4);
            let h: f32 = rows * line_height + 16.0; // 8px padding top + bottom
            (explicit_w.unwrap_or(200.0), explicit_h.unwrap_or(h))
        }
        "option" => {
            // Options are rendered in dropdown overlay, not laid out directly
            (0.0, 0.0)
        }
        "overlay" => {
            // Overlays don't consume space in normal flow — positioned separately
            (0.0, 0.0)
        }
        "spacer" => (explicit_w.unwrap_or(0.0), explicit_h.unwrap_or(0.0)),
        "row" => {
            let wrap = get_bool_prop(node, "wrap").unwrap_or(false);
            let inner_w = explicit_w.unwrap_or(available_w) - padding * 2.0;
            let inner_h = explicit_h.map(|h| h - padding * 2.0).unwrap_or(available_h);

            if wrap {
                // Wrapping row: calculate height based on wrapped lines
                let mut row_w: f32 = 0.0;
                let mut row_h: f32 = 0.0;
                let mut total_h: f32 = 0.0;
                let mut row_count = 0;

                for child in node.children.iter() {
                    let (cw, ch) = measure_node(child, inner_w, inner_h, text_measure);
                    let item_gap = if row_w > 0.0 { gap } else { 0.0 };

                    if row_w > 0.0 && row_w + item_gap + cw > inner_w {
                        // Wrap to next row
                        if row_count > 0 {
                            total_h += gap;
                        }
                        total_h += row_h;
                        row_count += 1;
                        row_w = cw;
                        row_h = ch;
                    } else {
                        row_w += item_gap + cw;
                        row_h = row_h.max(ch);
                    }
                }
                // Add final row
                if row_count > 0 {
                    total_h += gap;
                }
                total_h += row_h;

                let w = explicit_w.unwrap_or(inner_w + padding * 2.0);
                let h = explicit_h.unwrap_or(total_h + padding * 2.0);
                (w, h)
            } else {
                // Non-wrapping row
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
        "scroll" => {
            // Scroll containers use explicit dimensions or available space
            // Content is measured but doesn't affect container size
            let w = explicit_w.unwrap_or(available_w);
            let h = explicit_h.unwrap_or(available_h);
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
    };

    // Apply min/max constraints
    (
        apply_constraints(w, min_w, max_w),
        apply_constraints(h, min_h, max_h),
    )
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
    let align = get_str_prop(node, "align").unwrap_or("stretch");
    let justify = get_str_prop(node, "justify").unwrap_or("start");

    // Handle scroll containers specially
    if node.kind == "scroll" {
        return layout_scroll_node(
            node,
            x,
            y,
            width,
            height,
            padding,
            gap,
            align,
            justify,
            text_measure,
        );
    }

    let children = match node.kind.as_str() {
        "text" | "heading" | "rect" | "spacer" | "image" | "checkbox" | "radio" | "input"
        | "textarea" | "select" | "option" => Vec::new(),
        "row" => {
            let inner_x = x + padding;
            let inner_y = y + padding;
            let inner_w = width - padding * 2.0;
            let inner_h = height - padding * 2.0;
            let wrap = get_bool_prop(node, "wrap").unwrap_or(false);
            if wrap {
                layout_row_wrap(
                    &node.children,
                    inner_x,
                    inner_y,
                    inner_w,
                    inner_h,
                    gap,
                    align,
                    text_measure,
                )
            } else {
                layout_row(
                    &node.children,
                    inner_x,
                    inner_y,
                    inner_w,
                    inner_h,
                    gap,
                    align,
                    justify,
                    text_measure,
                )
            }
        }
        "grid" => {
            let inner_x = x + padding;
            let inner_y = y + padding;
            let inner_w = width - padding * 2.0;
            let cols = get_num_prop(node, "columns").unwrap_or(2.0) as usize;
            layout_grid(
                &node.children,
                inner_x,
                inner_y,
                inner_w,
                available_h,
                cols,
                gap,
                text_measure,
            )
        }
        _ => {
            // column, container, stack, unknown — vertical stacking
            let inner_x = x + padding;
            let inner_y = y + padding;
            let inner_w = width - padding * 2.0;
            let inner_h = height - padding * 2.0;
            layout_column(
                &node.children,
                inner_x,
                inner_y,
                inner_w,
                inner_h,
                gap,
                align,
                justify,
                text_measure,
            )
        }
    };

    PositionedNode {
        kind: node.kind.clone(),
        props: node.props.clone(),
        handlers: node.handlers.clone(),
        x,
        y,
        width,
        height,
        children,
        scroll_info: None,
    }
}

/// Layout a scroll container. Children are laid out with unbounded space in scroll direction,
/// and scroll_info tracks the total content size.
fn layout_scroll_node<F: Fn(&str, f32) -> (f32, f32)>(
    node: &RenderNode,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    padding: f32,
    gap: f32,
    align: &str,
    justify: &str,
    text_measure: &F,
) -> PositionedNode {
    let inner_x = x + padding;
    let inner_y = y + padding;
    let inner_w = width - padding * 2.0;
    let inner_h = height - padding * 2.0;

    // Determine overflow direction from "overflow" prop (default: "y")
    let overflow = get_str_prop(node, "overflow").unwrap_or("y");
    let overflow_x = overflow == "x" || overflow == "both";
    let overflow_y = overflow == "y" || overflow == "both" || (!overflow_x && overflow != "x");

    // Layout children with unbounded space in scroll direction
    let child_available_w = if overflow_x { f32::MAX } else { inner_w };
    let child_available_h = if overflow_y { f32::MAX } else { inner_h };

    // Use column layout for vertical scroll, row layout for horizontal
    let children = if overflow_x && !overflow_y {
        // Horizontal scroll - use row layout
        layout_row(
            &node.children,
            inner_x,
            inner_y,
            child_available_w,
            inner_h,
            gap,
            align,
            justify,
            text_measure,
        )
    } else {
        // Vertical scroll (default) or both - use column layout
        layout_column(
            &node.children,
            inner_x,
            inner_y,
            inner_w,
            child_available_h,
            gap,
            align,
            justify,
            text_measure,
        )
    };

    // Calculate content bounds from children
    let (content_width, content_height) = calculate_content_bounds(&children, inner_x, inner_y);

    PositionedNode {
        kind: node.kind.clone(),
        props: node.props.clone(),
        handlers: node.handlers.clone(),
        x,
        y,
        width,
        height,
        children,
        scroll_info: Some(ScrollInfo {
            content_width,
            content_height,
            overflow_x,
            overflow_y,
        }),
    }
}

/// Calculate the total content bounds from positioned children.
fn calculate_content_bounds(nodes: &[PositionedNode], origin_x: f32, origin_y: f32) -> (f32, f32) {
    let mut max_x: f32 = 0.0;
    let mut max_y: f32 = 0.0;
    for node in nodes {
        max_x = max_x.max(node.x + node.width - origin_x);
        max_y = max_y.max(node.y + node.height - origin_y);
    }
    (max_x, max_y)
}

/// Layout children in a column with gap, align (cross-axis), and justify (main-axis).
fn layout_column<F: Fn(&str, f32) -> (f32, f32)>(
    nodes: &[RenderNode],
    x: f32,
    y: f32,
    available_w: f32,
    available_h: f32,
    gap: f32,
    align: &str,   // cross-axis: "start", "center", "end", "stretch" (default)
    justify: &str, // main-axis: "start" (default), "center", "end", "space-between", "space-around", "space-evenly"
    text_measure: &F,
) -> Vec<PositionedNode> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(nodes.len());

    // Count spacers/flex-grow/flex-shrink and measure fixed children
    let mut child_sizes: Vec<(f32, f32)> = Vec::with_capacity(nodes.len());
    let mut total_fixed_h: f32 = 0.0;
    let mut total_flex_grow: f32 = 0.0;
    let mut total_flex_shrink: f32 = 0.0;
    let mut flex_grows: Vec<f32> = Vec::with_capacity(nodes.len());
    let mut flex_shrinks: Vec<f32> = Vec::with_capacity(nodes.len());

    for node in nodes {
        let flex_grow = get_num_prop(node, "flex-grow").unwrap_or(0.0) as f32;
        let flex_shrink = get_num_prop(node, "flex-shrink").unwrap_or(1.0) as f32; // Default 1
        let is_auto_spacer = node.kind == "spacer" && get_num_prop(node, "height").is_none();

        if is_auto_spacer {
            // Auto spacers have implicit flex-grow: 1, flex-shrink: 0 (don't shrink to negative)
            flex_grows.push(1.0);
            flex_shrinks.push(0.0);
            total_flex_grow += 1.0;
            child_sizes.push((0.0, 0.0));
        } else {
            flex_grows.push(flex_grow);
            flex_shrinks.push(flex_shrink);
            if flex_grow > 0.0 {
                total_flex_grow += flex_grow;
            }
            let (w, h) = measure_node(node, available_w, available_h, text_measure);
            child_sizes.push((w, h));
            total_fixed_h += h;
            // Track shrinkable size (flex-shrink * size)
            total_flex_shrink += flex_shrink * h;
        }
    }

    let total_gaps = if nodes.len() > 1 {
        gap * (nodes.len() as f32 - 1.0)
    } else {
        0.0
    };
    let space_diff = available_h - total_fixed_h - total_gaps;

    // Determine if we're growing or shrinking
    let (start_offset, between_gap, final_sizes) = if space_diff >= 0.0 {
        // Positive space: distribute with flex-grow
        let (start, gap) = calculate_justify_spacing(
            justify,
            space_diff.max(0.0),
            nodes.len(),
            gap,
            total_flex_grow > 0.0,
        );

        let sizes: Vec<f32> = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let (_, measured_h) = child_sizes[i];
                let flex = flex_grows[i];
                if flex > 0.0 && total_flex_grow > 0.0 {
                    let flex_share = space_diff * (flex / total_flex_grow);
                    if node.kind == "spacer" {
                        flex_share
                    } else {
                        measured_h + flex_share
                    }
                } else {
                    measured_h
                }
            })
            .collect();

        (start, gap, sizes)
    } else {
        // Negative space: shrink with flex-shrink
        let overflow = -space_diff;

        let sizes: Vec<f32> = nodes
            .iter()
            .enumerate()
            .map(|(i, _node)| {
                let (_, measured_h) = child_sizes[i];
                let shrink = flex_shrinks[i];
                if shrink > 0.0 && total_flex_shrink > 0.0 {
                    let shrink_amount = overflow * (shrink * measured_h) / total_flex_shrink;
                    (measured_h - shrink_amount).max(0.0)
                } else {
                    measured_h
                }
            })
            .collect();

        (0.0, gap, sizes)
    };

    let mut cursor_y = y + start_offset;

    for (i, node) in nodes.iter().enumerate() {
        if i > 0 {
            cursor_y += between_gap;
        }

        let (measured_w, _) = child_sizes[i];
        let h = final_sizes[i];

        // Calculate x position based on align (cross-axis)
        let w = match align {
            "stretch" => available_w,
            _ => measured_w,
        };
        let child_x = match align {
            "center" => x + (available_w - w) / 2.0,
            "end" => x + available_w - w,
            _ => x, // start, stretch
        };

        let positioned = layout_node(
            node,
            child_x,
            cursor_y,
            w,
            h,
            available_w,
            available_h,
            text_measure,
        );
        cursor_y += positioned.height;
        out.push(positioned);
    }

    out
}

/// Calculate justify spacing offsets.
/// Returns (start_offset, gap_between_items).
fn calculate_justify_spacing(
    justify: &str,
    remaining: f32,
    item_count: usize,
    gap: f32,
    has_flex: bool,
) -> (f32, f32) {
    // If there's flex content, justify doesn't add extra space (flex takes it)
    if has_flex {
        return (0.0, gap);
    }

    match justify {
        "center" => (remaining / 2.0, gap),
        "end" => (remaining, gap),
        "space-between" if item_count > 1 => {
            let extra_gap = remaining / (item_count as f32 - 1.0);
            (0.0, gap + extra_gap)
        }
        "space-around" if item_count > 0 => {
            let space = remaining / item_count as f32;
            (space / 2.0, gap + space)
        }
        "space-evenly" if item_count > 0 => {
            let space = remaining / (item_count as f32 + 1.0);
            (space, gap + space)
        }
        _ => (0.0, gap), // "start" or default
    }
}

/// Layout children in a row with gap, align (cross-axis), and justify (main-axis).
fn layout_row<F: Fn(&str, f32) -> (f32, f32)>(
    nodes: &[RenderNode],
    x: f32,
    y: f32,
    available_w: f32,
    available_h: f32,
    gap: f32,
    align: &str,   // cross-axis: "start", "center", "end", "stretch" (default)
    justify: &str, // main-axis: "start" (default), "center", "end", "space-between", etc.
    text_measure: &F,
) -> Vec<PositionedNode> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(nodes.len());

    // Count spacers/flex-grow/flex-shrink and measure fixed children
    let mut child_sizes: Vec<(f32, f32)> = Vec::with_capacity(nodes.len());
    let mut total_fixed_w: f32 = 0.0;
    let mut total_flex_grow: f32 = 0.0;
    let mut total_flex_shrink: f32 = 0.0;
    let mut flex_grows: Vec<f32> = Vec::with_capacity(nodes.len());
    let mut flex_shrinks: Vec<f32> = Vec::with_capacity(nodes.len());

    for node in nodes {
        let flex_grow = get_num_prop(node, "flex-grow").unwrap_or(0.0) as f32;
        let flex_shrink = get_num_prop(node, "flex-shrink").unwrap_or(1.0) as f32; // Default 1
        let is_auto_spacer = node.kind == "spacer" && get_num_prop(node, "width").is_none();

        if is_auto_spacer {
            flex_grows.push(1.0);
            flex_shrinks.push(0.0);
            total_flex_grow += 1.0;
            child_sizes.push((0.0, 0.0));
        } else {
            flex_grows.push(flex_grow);
            flex_shrinks.push(flex_shrink);
            if flex_grow > 0.0 {
                total_flex_grow += flex_grow;
            }
            let (w, h) = measure_node(node, available_w, available_h, text_measure);
            child_sizes.push((w, h));
            total_fixed_w += w;
            total_flex_shrink += flex_shrink * w;
        }
    }

    let total_gaps = if nodes.len() > 1 {
        gap * (nodes.len() as f32 - 1.0)
    } else {
        0.0
    };
    let space_diff = available_w - total_fixed_w - total_gaps;

    // Determine if we're growing or shrinking
    let (start_offset, between_gap, final_sizes) = if space_diff >= 0.0 {
        // Positive space: distribute with flex-grow
        let (start, gap) = calculate_justify_spacing(
            justify,
            space_diff.max(0.0),
            nodes.len(),
            gap,
            total_flex_grow > 0.0,
        );

        let sizes: Vec<f32> = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let (measured_w, _) = child_sizes[i];
                let flex = flex_grows[i];
                if flex > 0.0 && total_flex_grow > 0.0 {
                    let flex_share = space_diff * (flex / total_flex_grow);
                    if node.kind == "spacer" {
                        flex_share
                    } else {
                        measured_w + flex_share
                    }
                } else {
                    measured_w
                }
            })
            .collect();

        (start, gap, sizes)
    } else {
        // Negative space: shrink with flex-shrink
        let overflow = -space_diff;

        let sizes: Vec<f32> = nodes
            .iter()
            .enumerate()
            .map(|(i, _node)| {
                let (measured_w, _) = child_sizes[i];
                let shrink = flex_shrinks[i];
                if shrink > 0.0 && total_flex_shrink > 0.0 {
                    let shrink_amount = overflow * (shrink * measured_w) / total_flex_shrink;
                    (measured_w - shrink_amount).max(0.0)
                } else {
                    measured_w
                }
            })
            .collect();

        (0.0, gap, sizes)
    };

    let mut cursor_x = x + start_offset;

    for (i, node) in nodes.iter().enumerate() {
        if i > 0 {
            cursor_x += between_gap;
        }

        let (_, measured_h) = child_sizes[i];
        let w = final_sizes[i];

        // Calculate y position based on align (cross-axis)
        let h = match align {
            "stretch" => available_h,
            _ => measured_h,
        };
        let child_y = match align {
            "center" => y + (available_h - h) / 2.0,
            "end" => y + available_h - h,
            _ => y, // start, stretch
        };

        let positioned = layout_node(
            node,
            cursor_x,
            child_y,
            w,
            h,
            available_w,
            available_h,
            text_measure,
        );
        cursor_x += positioned.width;
        out.push(positioned);
    }

    out
}

/// Layout children in a row with wrapping.
fn layout_row_wrap<F: Fn(&str, f32) -> (f32, f32)>(
    nodes: &[RenderNode],
    x: f32,
    y: f32,
    available_w: f32,
    available_h: f32,
    gap: f32,
    align: &str,
    text_measure: &F,
) -> Vec<PositionedNode> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(nodes.len());

    // First pass: measure all children
    let child_sizes: Vec<(f32, f32)> = nodes
        .iter()
        .map(|node| measure_node(node, available_w, available_h, text_measure))
        .collect();

    // Second pass: arrange into rows
    let mut cursor_x = x;
    let mut cursor_y = y;
    let mut row_height: f32 = 0.0;
    let mut row_start_idx = 0;
    let mut row_items: Vec<(usize, f32, f32)> = Vec::new(); // (index, width, height)

    for (i, (w, h)) in child_sizes.iter().enumerate() {
        let item_x = if row_items.is_empty() { 0.0 } else { gap };

        // Check if item fits on current row
        if !row_items.is_empty() && cursor_x + item_x + w > x + available_w {
            // Emit current row
            let final_row_height = row_height;
            for (idx, _, item_h) in &row_items {
                let (item_w, _) = child_sizes[*idx];
                let child_y = match align {
                    "center" => cursor_y + (final_row_height - item_h) / 2.0,
                    "end" => cursor_y + final_row_height - item_h,
                    _ => cursor_y, // start
                };
                // We need to position the actual node
                let child_x = if *idx == row_start_idx {
                    x
                } else {
                    out.last()
                        .map(|n: &PositionedNode| n.x + n.width + gap)
                        .unwrap_or(x)
                };
                let positioned = layout_node(
                    &nodes[*idx],
                    child_x,
                    child_y,
                    item_w,
                    *item_h,
                    available_w,
                    available_h,
                    text_measure,
                );
                out.push(positioned);
            }

            // Start new row
            cursor_y += final_row_height + gap;
            cursor_x = x;
            row_height = 0.0;
            row_start_idx = i;
            row_items.clear();
        }

        row_items.push((i, *w, *h));
        cursor_x += if row_items.len() == 1 { *w } else { gap + *w };
        row_height = row_height.max(*h);
    }

    // Emit final row
    let mut item_x = x;
    for (idx, item_w, item_h) in &row_items {
        let child_y = match align {
            "center" => cursor_y + (row_height - item_h) / 2.0,
            "end" => cursor_y + row_height - item_h,
            _ => cursor_y,
        };
        let positioned = layout_node(
            &nodes[*idx],
            item_x,
            child_y,
            *item_w,
            *item_h,
            available_w,
            available_h,
            text_measure,
        );
        item_x += item_w + gap;
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

/// Get a numeric prop, resolving percentage values relative to the available dimension.
/// Returns (value, is_percentage) where is_percentage indicates if the value was a %.
fn get_num_prop_with_percent(node: &RenderNode, key: &str) -> Option<(f64, bool)> {
    match node.props.get(key) {
        Some(RenderValue::Num(n, unit)) => {
            let is_percent = unit.as_ref().map_or(false, |u| u == "%");
            Some((*n, is_percent))
        }
        _ => None,
    }
}

/// Resolve a numeric prop that may be a percentage.
/// For percentages, calculates the actual value based on the available dimension.
fn resolve_dimension(node: &RenderNode, key: &str, available: f32) -> Option<f32> {
    match get_num_prop_with_percent(node, key) {
        Some((value, true)) => Some((value / 100.0) as f32 * available),
        Some((value, false)) => Some(value as f32),
        None => None,
    }
}

fn get_str_prop<'a>(node: &'a RenderNode, key: &str) -> Option<&'a str> {
    match node.props.get(key) {
        Some(RenderValue::Str(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn get_bool_prop(node: &RenderNode, key: &str) -> Option<bool> {
    match node.props.get(key) {
        Some(RenderValue::Bool(b)) => Some(*b),
        _ => None,
    }
}

fn get_text_content(node: &RenderNode) -> String {
    match node.props.get("__text") {
        Some(RenderValue::Str(s)) => s.clone(),
        Some(RenderValue::InterpolatedStr(parts)) => {
            use naze_ir::TextPart;
            let mut result = String::new();
            for part in parts {
                match part {
                    TextPart::Literal(s) => result.push_str(s),
                    TextPart::StateRef(name) => {
                        result.push('{');
                        result.push_str(name);
                        result.push('}');
                    }
                }
            }
            result
        }
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

/// Find a positioned node by its `id` prop. Returns (x, y, width, height) if found.
fn find_node_by_id(nodes: &[PositionedNode], id: &str) -> Option<(f32, f32, f32, f32)> {
    for node in nodes {
        if let Some(RenderValue::Str(node_id)) = node.props.get("id") {
            if node_id == id {
                return Some((node.x, node.y, node.width, node.height));
            }
        }
        if let Some(found) = find_node_by_id(&node.children, id) {
            return Some(found);
        }
    }
    None
}

/// Position overlay nodes relative to their anchors or as full-viewport overlays.
fn layout_overlays<F: Fn(&str, f32) -> (f32, f32)>(
    overlay_nodes: &[&RenderNode],
    positioned_root: &[PositionedNode],
    viewport_width: f32,
    viewport_height: f32,
    text_measure: &F,
) -> Vec<PositionedNode> {
    let mut overlays = Vec::new();

    for node in overlay_nodes {
        let anchor_id = match node.props.get("anchor") {
            Some(RenderValue::Str(id)) => Some(id.clone()),
            _ => None,
        };
        let placement = match node.props.get("anchor-placement") {
            Some(RenderValue::Str(p)) => p.as_str(),
            _ => "bottom",
        };

        let padding = get_num_prop(node, "padding").unwrap_or(0.0) as f32;
        let gap = get_num_prop(node, "gap").unwrap_or(0.0) as f32;
        let align = get_str_prop(node, "align").unwrap_or("stretch");
        let justify = get_str_prop(node, "justify").unwrap_or("start");

        if let Some(ref anchor) = anchor_id {
            if let Some((ax, ay, aw, ah)) = find_node_by_id(positioned_root, anchor) {
                // Measure overlay content to determine its size
                let content_w =
                    resolve_dimension(node, "width", viewport_width).unwrap_or_else(|| {
                        // Measure children to determine intrinsic width
                        let mut max_w: f32 = 0.0;
                        for child in &node.children {
                            let (cw, _) =
                                measure_node(child, viewport_width, viewport_height, text_measure);
                            max_w = max_w.max(cw);
                        }
                        max_w + padding * 2.0
                    });
                let content_h =
                    resolve_dimension(node, "height", viewport_height).unwrap_or_else(|| {
                        let mut total_h: f32 = 0.0;
                        for (i, child) in node.children.iter().enumerate() {
                            let (_, ch) =
                                measure_node(child, content_w, viewport_height, text_measure);
                            total_h += ch;
                            if i > 0 {
                                total_h += gap;
                            }
                        }
                        total_h + padding * 2.0
                    });

                // Position based on placement, with auto-flip at viewport edge
                let (mut ox, mut oy) = match placement {
                    "top" => (ax, ay - content_h),
                    "left" => (ax - content_w, ay),
                    "right" => (ax + aw, ay),
                    _ /* "bottom" */ => (ax, ay + ah),
                };

                // Auto-flip if overlay would extend beyond viewport
                if oy + content_h > viewport_height && placement == "bottom" {
                    oy = ay - content_h; // flip to top
                }
                if oy < 0.0 && placement == "top" {
                    oy = ay + ah; // flip to bottom
                }
                if ox + content_w > viewport_width && placement == "right" {
                    ox = ax - content_w; // flip to left
                }
                if ox < 0.0 && placement == "left" {
                    ox = ax + aw; // flip to right
                }

                // Clamp to viewport
                ox = ox.max(0.0);
                oy = oy.max(0.0);

                let children = layout_column(
                    &node.children,
                    ox + padding,
                    oy + padding,
                    content_w - padding * 2.0,
                    content_h - padding * 2.0,
                    gap,
                    align,
                    justify,
                    text_measure,
                );

                overlays.push(PositionedNode {
                    kind: node.kind.clone(),
                    props: node.props.clone(),
                    handlers: node.handlers.clone(),
                    x: ox,
                    y: oy,
                    width: content_w,
                    height: content_h,
                    children,
                    scroll_info: None,
                });
            }
            // If anchor not found, skip this overlay (it's invisible)
        } else {
            // No anchor — full-viewport overlay (e.g., modal dialog)
            let children = layout_column(
                &node.children,
                padding,
                padding,
                viewport_width - padding * 2.0,
                viewport_height - padding * 2.0,
                gap,
                align,
                justify,
                text_measure,
            );

            overlays.push(PositionedNode {
                kind: node.kind.clone(),
                props: node.props.clone(),
                handlers: node.handlers.clone(),
                x: 0.0,
                y: 0.0,
                width: viewport_width,
                height: viewport_height,
                children,
                scroll_info: None,
            });
        }
    }

    overlays
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
        assert!(r2.y > r1.y, "r2.y ({}) should be > r1.y ({})", r2.y, r1.y);
        assert!(
            (r2.y - r1.y - 30.0).abs() < 1.0,
            "gap between rects: expected 30, got {}",
            r2.y - r1.y
        );
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
        assert!(r2.x > r1.x, "r2.x ({}) should be > r1.x ({})", r2.x, r1.x);
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
            "counter.naze",
            "conditional.naze",
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
    fn layout_overlay_does_not_affect_siblings() {
        // An overlay between two rects should not consume space
        let tree = RenderTree {
            title: "Overlay Test".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![
                RenderNode {
                    kind: "rect".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert(
                            "width".to_string(),
                            RenderValue::Num(100.0, Some("px".to_string())),
                        );
                        m.insert(
                            "height".to_string(),
                            RenderValue::Num(50.0, Some("px".to_string())),
                        );
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                },
                RenderNode {
                    kind: "overlay".to_string(),
                    props: HashMap::new(),
                    children: vec![RenderNode {
                        kind: "rect".to_string(),
                        props: {
                            let mut m = HashMap::new();
                            m.insert(
                                "width".to_string(),
                                RenderValue::Num(300.0, Some("px".to_string())),
                            );
                            m.insert(
                                "height".to_string(),
                                RenderValue::Num(200.0, Some("px".to_string())),
                            );
                            m
                        },
                        children: vec![],
                        handlers: vec![],
                        condition: None,
                        else_children: None,
                        each_binding: None,
                    }],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                },
                RenderNode {
                    kind: "rect".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert(
                            "width".to_string(),
                            RenderValue::Num(100.0, Some("px".to_string())),
                        );
                        m.insert(
                            "height".to_string(),
                            RenderValue::Num(50.0, Some("px".to_string())),
                        );
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                },
            ],
            pages: vec![],
        };
        let layout = compute_layout(&tree, 800.0, 600.0);

        // Root should have 2 rects (overlay extracted)
        assert_eq!(
            layout.root.len(),
            2,
            "overlay should be extracted from root"
        );
        // Second rect should be directly below the first (no gap from overlay)
        let r1 = &layout.root[0];
        let r2 = &layout.root[1];
        assert!(
            (r2.y - r1.y - 50.0).abs() < 1.0,
            "second rect y ({}) should be first.y + first.height ({})",
            r2.y,
            r1.y + r1.height
        );

        // Overlay should be in overlays vec
        assert_eq!(layout.overlays.len(), 1);
        assert_eq!(layout.overlays[0].kind, "overlay");
    }

    #[test]
    fn layout_overlay_full_viewport() {
        // Overlay without anchor should span full viewport
        let tree = RenderTree {
            title: "Modal Test".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![RenderNode {
                kind: "overlay".to_string(),
                props: HashMap::new(),
                children: vec![RenderNode {
                    kind: "text".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert("__text".to_string(), RenderValue::Str("Hello".to_string()));
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                }],
                handlers: vec![],
                condition: None,
                else_children: None,
                each_binding: None,
            }],
            pages: vec![],
        };
        let layout = compute_layout(&tree, 800.0, 600.0);

        assert!(
            layout.root.is_empty(),
            "root should be empty (only overlay)"
        );
        assert_eq!(layout.overlays.len(), 1);

        let overlay = &layout.overlays[0];
        assert!((overlay.x - 0.0).abs() < 1.0);
        assert!((overlay.y - 0.0).abs() < 1.0);
        assert!((overlay.width - 800.0).abs() < 1.0);
        assert!((overlay.height - 600.0).abs() < 1.0);
    }

    #[test]
    fn layout_overlay_anchored_bottom() {
        // Overlay anchored to a button should appear below it
        let tree = RenderTree {
            title: "Dropdown Test".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![
                RenderNode {
                    kind: "rect".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert("id".to_string(), RenderValue::Str("btn".to_string()));
                        m.insert(
                            "width".to_string(),
                            RenderValue::Num(120.0, Some("px".to_string())),
                        );
                        m.insert(
                            "height".to_string(),
                            RenderValue::Num(40.0, Some("px".to_string())),
                        );
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                },
                RenderNode {
                    kind: "overlay".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert("anchor".to_string(), RenderValue::Str("btn".to_string()));
                        m.insert(
                            "width".to_string(),
                            RenderValue::Num(200.0, Some("px".to_string())),
                        );
                        m.insert(
                            "height".to_string(),
                            RenderValue::Num(100.0, Some("px".to_string())),
                        );
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                },
            ],
            pages: vec![],
        };
        let layout = compute_layout(&tree, 800.0, 600.0);

        assert_eq!(layout.root.len(), 1); // just the button
        assert_eq!(layout.overlays.len(), 1);

        let btn = &layout.root[0];
        let overlay = &layout.overlays[0];

        // Overlay should be positioned below the button (default placement = bottom)
        assert!(
            (overlay.x - btn.x).abs() < 1.0,
            "overlay.x ({}) should equal btn.x ({})",
            overlay.x,
            btn.x
        );
        assert!(
            (overlay.y - (btn.y + btn.height)).abs() < 1.0,
            "overlay.y ({}) should equal btn.y + btn.height ({})",
            overlay.y,
            btn.y + btn.height
        );
    }

    #[test]
    fn layout_overlay_anchor_auto_flip() {
        // Button near bottom of viewport — overlay should flip to top
        let tree = RenderTree {
            title: "Flip Test".to_string(),
            state: vec![],
            data: vec![],
            computed: vec![],
            storage: vec![],
            timers: vec![],
            params: vec![],
            root: vec![
                // Push button to bottom with a spacer
                RenderNode {
                    kind: "spacer".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert(
                            "height".to_string(),
                            RenderValue::Num(500.0, Some("px".to_string())),
                        );
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                },
                RenderNode {
                    kind: "rect".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert("id".to_string(), RenderValue::Str("btn-bottom".to_string()));
                        m.insert(
                            "width".to_string(),
                            RenderValue::Num(120.0, Some("px".to_string())),
                        );
                        m.insert(
                            "height".to_string(),
                            RenderValue::Num(40.0, Some("px".to_string())),
                        );
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                },
                RenderNode {
                    kind: "overlay".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert(
                            "anchor".to_string(),
                            RenderValue::Str("btn-bottom".to_string()),
                        );
                        m.insert(
                            "anchor-placement".to_string(),
                            RenderValue::Str("bottom".to_string()),
                        );
                        m.insert(
                            "width".to_string(),
                            RenderValue::Num(200.0, Some("px".to_string())),
                        );
                        m.insert(
                            "height".to_string(),
                            RenderValue::Num(150.0, Some("px".to_string())),
                        );
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    condition: None,
                    else_children: None,
                    each_binding: None,
                },
            ],
            pages: vec![],
        };
        let layout = compute_layout(&tree, 800.0, 600.0);

        let btn = &layout.root[1]; // second node (after spacer)
        let overlay = &layout.overlays[0];

        // Button is at y=500, height=40 → bottom edge at 540
        // Overlay height=150 → would extend to 690 (past viewport 600)
        // Should auto-flip to top: overlay.y = btn.y - overlay.height = 500 - 150 = 350
        assert!(
            overlay.y < btn.y,
            "overlay.y ({}) should be above btn.y ({}) due to auto-flip",
            overlay.y,
            btn.y
        );
        assert!(
            (overlay.y - (btn.y - 150.0)).abs() < 1.0,
            "overlay.y ({}) should be btn.y - height ({})",
            overlay.y,
            btn.y - 150.0
        );
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
