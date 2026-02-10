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
    // Top edge
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    // Top-right corner
    pb.quad_to(x + w, y, x + w, y + r);
    // Right edge
    pb.line_to(x + w, y + h - r);
    // Bottom-right corner
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    // Bottom edge
    pb.line_to(x + r, y + h);
    // Bottom-left corner
    pb.quad_to(x, y + h, x, y + h - r);
    // Left edge
    pb.line_to(x, y + r);
    // Top-left corner
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish()
}

fn get_str_prop(props: &HashMap<String, RenderValue>, key: &str, default: &str) -> String {
    match props.get(key) {
        Some(RenderValue::Str(s)) => s.clone(),
        _ => default.to_string(),
    }
}

fn draw_text_decoration(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    font_size: f32,
    decoration: &str,
    color: u32,
) {
    let line_y = match decoration {
        "underline" => y + font_size * 0.95,
        "line-through" => y + font_size * 0.5,
        "overline" => y + font_size * 0.05,
        _ => return,
    };
    let thickness = 1.0_f32.max(font_size / 16.0);
    let paint = make_paint(color);
    if let Some(rect) = Rect::from_xywh(x, line_y, width, thickness) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
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
                // Alpha-blend onto pixmap (premultiplied RGBA)
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

pub fn draw_tree(pixmap: &mut Pixmap, layout: &LayoutTree, font: &fontdue::Font) {
    for node in &layout.root {
        draw_node(pixmap, node, font);
    }
    // Draw overlays on top of root content
    for node in &layout.overlays {
        draw_node(pixmap, node, font);
    }
}

fn draw_node(pixmap: &mut Pixmap, node: &PositionedNode, font: &fontdue::Font) {
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
                draw_node(pixmap, child, font);
            }
        }
        "text" => {
            let text = get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = get_font_size(&node.props, false) as f32;
                let color = get_color_u32(&node.props, "color", 0x000000);
                let text_align = get_str_prop(&node.props, "text-align", "");
                let text_width = text.len() as f32 * font_size * 0.6;
                let draw_x = match text_align.as_str() {
                    "center" => x + (w - text_width) / 2.0,
                    "right" | "end" => x + w - text_width,
                    _ => x,
                };
                draw_text(pixmap, &text, draw_x, y, font_size, font, color);
                let decoration = get_str_prop(&node.props, "text-decoration", "");
                if !decoration.is_empty() {
                    draw_text_decoration(
                        pixmap,
                        draw_x,
                        y,
                        text_width,
                        font_size,
                        &decoration,
                        color,
                    );
                }
            }
        }
        "heading" => {
            let text = get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = get_font_size(&node.props, true) as f32;
                let color = get_color_u32(&node.props, "color", 0x000000);
                let text_align = get_str_prop(&node.props, "text-align", "");
                let text_width = text.len() as f32 * font_size * 0.6;
                let draw_x = match text_align.as_str() {
                    "center" => x + (w - text_width) / 2.0,
                    "right" | "end" => x + w - text_width,
                    _ => x,
                };
                draw_text(pixmap, &text, draw_x, y, font_size, font, color);
                let decoration = get_str_prop(&node.props, "text-decoration", "");
                if !decoration.is_empty() {
                    draw_text_decoration(
                        pixmap,
                        draw_x,
                        y,
                        text_width,
                        font_size,
                        &decoration,
                        color,
                    );
                }
            }
        }
        "row" | "column" | "stack" | "grid" => {
            let color = get_color_u32_opt(&node.props, "color");
            if let Some(c) = color {
                fill_rect(pixmap, x, y, w, h, c, 0.0);
            }
            for child in &node.children {
                draw_node(pixmap, child, font);
            }
        }
        "spacer" => {}
        _ => {
            for child in &node.children {
                draw_node(pixmap, child, font);
            }
        }
    }
}
