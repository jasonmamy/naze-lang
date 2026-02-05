use std::collections::HashMap;

use naze_ir::RenderValue;
use naze_layout::{LayoutTree, PositionedNode};
use tiny_skia::{Paint, PathBuilder, Pixmap, Rect, Shader, Transform};

// --- Helper functions (duplicated from naze-renderer to avoid web-sys dep) ---

const DEFAULT_TEXT_SIZE: f64 = 16.0;
const DEFAULT_HEADING_SIZE: f64 = 24.0;

fn get_color_u32(props: &HashMap<String, RenderValue>, key: &str, default: u32) -> u32 {
    match props.get(key) {
        Some(RenderValue::Color(c)) => *c,
        _ => default,
    }
}

fn get_color_u32_opt(props: &HashMap<String, RenderValue>, key: &str) -> Option<u32> {
    match props.get(key) {
        Some(RenderValue::Color(c)) => Some(*c),
        _ => None,
    }
}

fn get_num_prop(props: &HashMap<String, RenderValue>, key: &str, default: f64) -> f64 {
    match props.get(key) {
        Some(RenderValue::Num(n, _)) => *n,
        _ => default,
    }
}

fn get_text_content(props: &HashMap<String, RenderValue>) -> String {
    match props.get("__text") {
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

fn get_font_size(props: &HashMap<String, RenderValue>, is_heading: bool) -> f64 {
    match props.get("font-size") {
        Some(RenderValue::Num(n, _)) => *n,
        _ => {
            if is_heading {
                DEFAULT_HEADING_SIZE
            } else {
                DEFAULT_TEXT_SIZE
            }
        }
    }
}

// --- Drawing functions ---

fn make_paint(color: u32) -> Paint<'static> {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    Paint {
        shader: Shader::SolidColor(tiny_skia::Color::from_rgba8(r, g, b, 255)),
        anti_alias: true,
        ..Paint::default()
    }
}

fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: u32, radius: f32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let paint = make_paint(color);

    if radius > 0.0 {
        if let Some(path) = rounded_rect_path(x, y, w, h, radius) {
            pixmap.fill_path(
                &path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    } else if let Some(rect) = Rect::from_xywh(x, y, w, h) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.0).min(h / 2.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish()
}

fn stroke_rounded_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, r: f32, color: u32, stroke_width: f32) {
    if let Some(path) = rounded_rect_path(x, y, w, h, r) {
        let paint = make_paint(color);
        let stroke = tiny_skia::Stroke {
            width: stroke_width,
            ..tiny_skia::Stroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn draw_checkbox(pixmap: &mut Pixmap, x: f32, y: f32, checked: bool, label: &str, font: &fontdue::Font) {
    let box_size = 20.0_f32;
    let box_x = x;
    let box_y = y;

    // Draw box background (white)
    fill_rect(pixmap, box_x, box_y, box_size, box_size, 0xffffff, 3.0);

    // Draw box border (gray)
    stroke_rounded_rect(pixmap, box_x, box_y, box_size, box_size, 3.0, 0x9ca3af, 2.0);

    // Draw checkmark if checked
    if checked {
        let mut pb = PathBuilder::new();
        pb.move_to(box_x + 4.0, box_y + 10.0);
        pb.line_to(box_x + 8.0, box_y + 15.0);
        pb.line_to(box_x + 16.0, box_y + 5.0);
        if let Some(path) = pb.finish() {
            let paint = make_paint(0x2563eb); // Blue checkmark
            let stroke = tiny_skia::Stroke {
                width: 2.5,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..tiny_skia::Stroke::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    // Draw label
    if !label.is_empty() {
        draw_text(pixmap, label, x + 28.0, y + 2.0, 16.0, font, 0x374151);
    }
}

fn draw_radio(pixmap: &mut Pixmap, x: f32, y: f32, selected: bool, label: &str, font: &fontdue::Font) {
    let radius = 10.0_f32;
    let center_x = x + radius;
    let center_y = y + radius;

    // Draw outer circle background (white)
    let mut pb = PathBuilder::new();
    pb.push_circle(center_x, center_y, radius);
    if let Some(path) = pb.finish() {
        let paint = make_paint(0xffffff);
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }

    // Draw outer circle border (gray)
    let mut pb = PathBuilder::new();
    pb.push_circle(center_x, center_y, radius - 1.0);
    if let Some(path) = pb.finish() {
        let paint = make_paint(0x9ca3af);
        let stroke = tiny_skia::Stroke {
            width: 2.0,
            ..tiny_skia::Stroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // Draw inner filled circle if selected
    if selected {
        let mut pb = PathBuilder::new();
        pb.push_circle(center_x, center_y, 5.0);
        if let Some(path) = pb.finish() {
            let paint = make_paint(0x2563eb); // Blue
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    // Draw label
    if !label.is_empty() {
        draw_text(pixmap, label, x + 28.0, y + 2.0, 16.0, font, 0x374151);
    }
}

fn draw_input(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, value: &str, placeholder: &str, focused: bool, input_type: &str, show_caret: bool, font: &fontdue::Font) {
    // Background
    fill_rect(pixmap, x, y, w, h, 0xffffff, 4.0);

    // Border - blue when focused, gray otherwise
    let border_color = if focused { 0x2563eb } else { 0xd1d5db };
    let border_width = if focused { 2.0 } else { 1.0 };
    stroke_rounded_rect(pixmap, x, y, w, h, 4.0, border_color, border_width);

    // For password type, show dots instead of actual characters
    let display_value = if input_type == "password" {
        "•".repeat(value.chars().count())
    } else {
        value.to_string()
    };

    // Text content or placeholder
    let text_x = x + 8.0;
    let text_y = y + 4.0;
    if !display_value.is_empty() {
        draw_text(pixmap, &display_value, text_x, text_y, 16.0, font, 0x111827);
    } else if !placeholder.is_empty() {
        draw_text(pixmap, placeholder, text_x, text_y, 16.0, font, 0x9ca3af);
    }

    // Draw cursor when show_caret is true (handles blinking)
    if show_caret {
        // Measure text width to position cursor
        let text_width: f32 = display_value.chars()
            .map(|ch| font.metrics(ch, 16.0).advance_width)
            .sum();
        let cursor_x = text_x + text_width;

        // Draw cursor line
        let mut pb = PathBuilder::new();
        pb.move_to(cursor_x, y + 6.0);
        pb.line_to(cursor_x, y + h - 6.0);
        if let Some(path) = pb.finish() {
            let paint = make_paint(0x111827);
            let stroke = tiny_skia::Stroke {
                width: 1.0,
                ..tiny_skia::Stroke::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

fn draw_select(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    display_text: &str,
    is_open: bool,
    font: &fontdue::Font,
) {
    // Background
    fill_rect(pixmap, x, y, w, h, 0xffffff, 4.0);

    // Border
    let border_color = if is_open { 0x2563eb } else { 0xd1d5db };
    let border_width = if is_open { 2.0 } else { 1.0 };
    stroke_rounded_rect(pixmap, x, y, w, h, 4.0, border_color, border_width);

    // Display text
    if !display_text.is_empty() {
        draw_text(pixmap, display_text, x + 12.0, y + 8.0, 16.0, font, 0x111827);
    }

    // Dropdown arrow (simple triangle using lines)
    let arrow_x = x + w - 24.0;
    let arrow_y = y + h / 2.0;
    let mut pb = PathBuilder::new();
    if is_open {
        pb.move_to(arrow_x, arrow_y + 2.0);
        pb.line_to(arrow_x + 6.0, arrow_y - 4.0);
        pb.line_to(arrow_x + 12.0, arrow_y + 2.0);
    } else {
        pb.move_to(arrow_x, arrow_y - 2.0);
        pb.line_to(arrow_x + 6.0, arrow_y + 4.0);
        pb.line_to(arrow_x + 12.0, arrow_y - 2.0);
    }
    if let Some(path) = pb.finish() {
        let paint = make_paint(0x6b7280);
        let stroke = tiny_skia::Stroke {
            width: 2.0,
            line_cap: tiny_skia::LineCap::Round,
            line_join: tiny_skia::LineJoin::Round,
            ..tiny_skia::Stroke::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn draw_text(
    pixmap: &mut Pixmap,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    font: &fontdue::Font,
    color: u32,
) {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;

    let mut cursor_x = x;
    let baseline_y = y + font_size * 0.8;

    let pw = pixmap.width() as i32;
    let ph = pixmap.height() as i32;

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        let gx = cursor_x as i32 + metrics.xmin;
        let gy = baseline_y as i32 - metrics.height as i32 - metrics.ymin;

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha == 0 {
                    continue;
                }
                let px = gx + col as i32;
                let py = gy + row as i32;
                if px < 0 || py < 0 || px >= pw || py >= ph {
                    continue;
                }
                let di = ((py as u32 * pixmap.width() + px as u32) * 4) as usize;
                let data = pixmap.data_mut();
                let a = alpha as f32 / 255.0;
                let inv_a = 1.0 - a;
                data[di] = (r as f32 * a + data[di] as f32 * inv_a) as u8;
                data[di + 1] = (g as f32 * a + data[di + 1] as f32 * inv_a) as u8;
                data[di + 2] = (b as f32 * a + data[di + 2] as f32 * inv_a) as u8;
                data[di + 3] = 255;
            }
        }
        cursor_x += metrics.advance_width;
    }
}

// --- Tree drawing (mirrors naze-runtime draw_node logic) ---

pub fn draw_tree(pixmap: &mut Pixmap, layout: &LayoutTree, font: &fontdue::Font, focused_input_id: Option<&str>) {
    for node in &layout.root {
        draw_node(pixmap, node, font, focused_input_id);
    }
}

fn draw_node(pixmap: &mut Pixmap, node: &PositionedNode, font: &fontdue::Font, focused_input_id: Option<&str>) {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    match node.kind.as_str() {
        "rect" => {
            let color = get_color_u32(&node.props, "color", 0x000000);
            let radius = get_num_prop(&node.props, "radius", 0.0) as f32;
            fill_rect(pixmap, x, y, w, h, color, radius);
        }
        "container" => {
            let color = get_color_u32_opt(&node.props, "color");
            let radius = get_num_prop(&node.props, "radius", 0.0) as f32;
            if let Some(c) = color {
                fill_rect(pixmap, x, y, w, h, c, radius);
            }
            for child in &node.children {
                draw_node(pixmap, child, font, focused_input_id);
            }
        }
        "text" => {
            let text = get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = get_font_size(&node.props, false) as f32;
                let color = get_color_u32(&node.props, "color", 0x000000);
                draw_text(pixmap, &text, x, y, font_size, font, color);
            }
        }
        "heading" => {
            let text = get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = get_font_size(&node.props, true) as f32;
                let color = get_color_u32(&node.props, "color", 0x000000);
                draw_text(pixmap, &text, x, y, font_size, font, color);
            }
        }
        "row" | "column" | "stack" | "grid" => {
            let color = get_color_u32_opt(&node.props, "color");
            if let Some(c) = color {
                fill_rect(pixmap, x, y, w, h, c, 0.0);
            }
            for child in &node.children {
                draw_node(pixmap, child, font, focused_input_id);
            }
        }
        "spacer" => {}
        "checkbox" => {
            let label = get_text_content(&node.props);
            let checked = match node.props.get("checked") {
                Some(RenderValue::Bool(b)) => *b,
                _ => false,
            };
            draw_checkbox(pixmap, x, y, checked, &label, font);
        }
        "radio" => {
            let label = get_text_content(&node.props);
            let selected = match node.props.get("selected") {
                Some(RenderValue::Bool(b)) => *b,
                _ => false,
            };
            draw_radio(pixmap, x, y, selected, &label, font);
        }
        "input" => {
            let placeholder = match node.props.get("placeholder") {
                Some(RenderValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            let value = match node.props.get("value") {
                Some(RenderValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            // Check if this input is focused based on position-derived node_id
            let node_id = format!("input_{}_{}", x as i32, y as i32);
            let focused = focused_input_id == Some(node_id.as_str());
            let input_type = match node.props.get("type") {
                Some(RenderValue::Str(s)) => s.as_str(),
                _ => "text",
            };
            // Show caret when focused (native mode doesn't have blinking yet)
            let show_caret = focused;
            draw_input(pixmap, x, y, w, h, &value, &placeholder, focused, input_type, show_caret, font);
        }
        "select" => {
            // Get current value from selected prop (resolved in run.rs/gallery.rs)
            let selected_value = match node.props.get("selected") {
                Some(RenderValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            // Find display text from children options
            let mut display_text = String::new();
            for child in &node.children {
                if child.kind == "option" {
                    let value = match child.props.get("value") {
                        Some(RenderValue::Str(s)) => s.clone(),
                        _ => get_text_content(&child.props),
                    };
                    if value == selected_value {
                        display_text = get_text_content(&child.props);
                        break;
                    }
                }
            }
            // For native rendering, always show closed state
            draw_select(pixmap, x, y, w, h, &display_text, false, font);
        }
        "option" => {
            // Options are rendered by the parent select
        }
        _ => {
            for child in &node.children {
                draw_node(pixmap, child, font, focused_input_id);
            }
        }
    }
}
