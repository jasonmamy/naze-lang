use wasm_bindgen::prelude::*;

use naze_ir::RenderTree;
use naze_layout::{self, LayoutTree, PositionedNode};
use naze_renderer::{self, canvas::Renderer};

/// Entry point called from JavaScript.
/// `app_data` is a binary-encoded RenderTree.
/// `canvas_id` is the HTML id of the canvas element to render into.
#[wasm_bindgen]
pub fn start(app_data: &[u8], canvas_id: &str) -> Result<(), JsValue> {
    // 1. Deserialize the render tree
    let render_tree: RenderTree = naze_ir::deserialize(app_data)
        .map_err(|e| JsValue::from_str(&format!("failed to deserialize app data: {}", e)))?;

    // 2. Set up the renderer
    let renderer = Renderer::new(canvas_id)?;

    // 3. Get viewport size from window
    let window = web_sys::window().ok_or("no window")?;
    let vw = window.inner_width()?.as_f64().unwrap_or(1024.0) as f32;
    let vh = window.inner_height()?.as_f64().unwrap_or(768.0) as f32;

    // 4. Set canvas size to viewport
    renderer.set_size(vw as f64, vh as f64);

    // 5. Compute layout (borrows renderer for text measurement)
    let layout = {
        let text_measure = |text: &str, font_size: f32| -> (f32, f32) {
            let is_heading = font_size > 20.0;
            let (w, h) = renderer.measure_text(text, font_size as f64, is_heading);
            (w as f32, h as f32)
        };
        naze_layout::compute_layout_with_measure(&render_tree, vw, vh, text_measure)
    };

    // 6. Set document title
    if let Some(document) = window.document() {
        document.set_title(&layout.title);
    }

    // 7. Clear and draw
    renderer.clear();
    draw_tree(&renderer, &layout);

    Ok(())
}

/// Walk the positioned tree and draw all nodes.
fn draw_tree(renderer: &Renderer, layout: &LayoutTree) {
    for node in &layout.root {
        draw_node(renderer, node);
    }
}

/// Recursively draw a positioned node and its children.
fn draw_node(renderer: &Renderer, node: &PositionedNode) {
    let x = node.x as f64;
    let y = node.y as f64;
    let w = node.width as f64;
    let h = node.height as f64;

    match node.kind.as_str() {
        "rect" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            renderer.draw_rect(x, y, w, h, &color, radius);
        }
        "container" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "");
            let radius = naze_renderer::get_num_prop(&node.props, "radius", 0.0);
            if !color.is_empty() {
                renderer.draw_rect(x, y, w, h, &color, radius);
            }
            for child in &node.children {
                draw_node(renderer, child);
            }
        }
        "text" => {
            let text = naze_renderer::get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = naze_renderer::get_font_size(&node.props, false);
                let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
                renderer.draw_text(&text, x, y, font_size, false, &color);
            }
        }
        "heading" => {
            let text = naze_renderer::get_text_content(&node.props);
            if !text.is_empty() {
                let font_size = naze_renderer::get_font_size(&node.props, true);
                let color = naze_renderer::get_color_prop(&node.props, "color", "#000000");
                renderer.draw_text(&text, x, y, font_size, true, &color);
            }
        }
        "row" | "column" | "stack" | "grid" => {
            let color = naze_renderer::get_color_prop(&node.props, "color", "");
            if !color.is_empty() {
                renderer.draw_rect(x, y, w, h, &color, 0.0);
            }
            for child in &node.children {
                draw_node(renderer, child);
            }
        }
        "spacer" => {}
        _ => {
            for child in &node.children {
                draw_node(renderer, child);
            }
        }
    }
}
