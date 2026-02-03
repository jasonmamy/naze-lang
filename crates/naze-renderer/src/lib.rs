use naze_ir::RenderValue;

/// Convert a 24-bit RGB color integer to a CSS color string.
pub fn color_to_css(color: u32) -> String {
    let r = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = color & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Extract a color prop as CSS string, with a fallback.
pub fn get_color_prop(
    props: &std::collections::HashMap<String, RenderValue>,
    key: &str,
    default: &str,
) -> String {
    match props.get(key) {
        Some(RenderValue::Color(c)) => color_to_css(*c),
        _ => default.to_string(),
    }
}

/// Extract a numeric prop with fallback.
pub fn get_num_prop(
    props: &std::collections::HashMap<String, RenderValue>,
    key: &str,
    default: f64,
) -> f64 {
    match props.get(key) {
        Some(RenderValue::Num(n, _)) => *n,
        _ => default,
    }
}

/// Extract text content from a `__text` prop.
/// Handles both plain strings and interpolated strings (fallback: joins parts).
pub fn get_text_content(
    props: &std::collections::HashMap<String, RenderValue>,
) -> String {
    match props.get("__text") {
        Some(RenderValue::Str(s)) => s.clone(),
        Some(RenderValue::InterpolatedStr(parts)) => {
            // Fallback: runtime should resolve these before layout/render,
            // but handle gracefully if not.
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

/// Build a CSS font string like "16px sans-serif" or "bold 24px sans-serif".
pub fn font_string(size: f64, bold: bool) -> String {
    if bold {
        format!("bold {}px sans-serif", size)
    } else {
        format!("{}px sans-serif", size)
    }
}

/// Default font size for text elements.
pub const DEFAULT_TEXT_SIZE: f64 = 16.0;
/// Default font size for heading elements.
pub const DEFAULT_HEADING_SIZE: f64 = 24.0;

/// Get the font size from props, using element-appropriate defaults.
pub fn get_font_size(
    props: &std::collections::HashMap<String, RenderValue>,
    is_heading: bool,
) -> f64 {
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

/// Canvas2D renderer — only available on WASM targets.
#[cfg(target_arch = "wasm32")]
pub mod canvas {
    use super::*;
    use wasm_bindgen::prelude::*;
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

    /// A Canvas2D renderer that draws to an HTML canvas element.
    pub struct Renderer {
        ctx: CanvasRenderingContext2d,
        canvas: HtmlCanvasElement,
        dpr: f64,
    }

    impl Renderer {
        /// Create a new renderer attached to the canvas element with the given ID.
        /// Handles devicePixelRatio for crisp rendering on high-DPI screens.
        pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
            let window = web_sys::window().ok_or("no window")?;
            let document = window.document().ok_or("no document")?;
            let canvas_el = document
                .get_element_by_id(canvas_id)
                .ok_or_else(|| JsValue::from_str(&format!("no element with id '{}'", canvas_id)))?;
            let canvas: HtmlCanvasElement = canvas_el
                .dyn_into::<HtmlCanvasElement>()
                .map_err(|_| "element is not a canvas")?;

            let ctx = canvas
                .get_context("2d")?
                .ok_or("failed to get 2d context")?
                .dyn_into::<CanvasRenderingContext2d>()?;

            let dpr = window.device_pixel_ratio();

            Ok(Self { ctx, canvas, dpr })
        }

        /// Set the canvas to the given CSS size, scaling the backing store for DPR.
        pub fn set_size(&self, css_width: f64, css_height: f64) {
            let backing_w = (css_width * self.dpr) as u32;
            let backing_h = (css_height * self.dpr) as u32;

            self.canvas.set_width(backing_w);
            self.canvas.set_height(backing_h);

            // Set CSS size
            self.canvas
                .style()
                .set_property("width", &format!("{}px", css_width))
                .unwrap();
            self.canvas
                .style()
                .set_property("height", &format!("{}px", css_height))
                .unwrap();

            // Scale context to match DPR
            self.ctx.scale(self.dpr, self.dpr).unwrap();
        }

        /// Clear the entire canvas.
        pub fn clear(&self) {
            let w = self.canvas.width() as f64 / self.dpr;
            let h = self.canvas.height() as f64 / self.dpr;
            self.ctx.clear_rect(0.0, 0.0, w, h);
        }

        /// Draw a filled rectangle, optionally with rounded corners.
        pub fn draw_rect(&self, x: f64, y: f64, w: f64, h: f64, color: &str, radius: f64) {
            self.ctx.set_fill_style_str(color);

            if radius > 0.0 {
                self.draw_rounded_rect_path(x, y, w, h, radius);
                self.ctx.fill();
            } else {
                self.ctx.fill_rect(x, y, w, h);
            }
        }

        /// Draw text at the given position.
        /// `x`, `y` are the top-left corner of the text bounding box.
        /// The text baseline is offset from y by the font size (approximate ascent).
        pub fn draw_text(
            &self,
            text: &str,
            x: f64,
            y: f64,
            font_size: f64,
            bold: bool,
            color: &str,
        ) {
            let font = font_string(font_size, bold);
            self.ctx.set_font(&font);
            self.ctx.set_fill_style_str(color);
            // Approximate baseline: ~80% of font size from the top
            let baseline_y = y + font_size * 0.8;
            self.ctx.fill_text(text, x, baseline_y).unwrap();
        }

        /// Measure text width using the Canvas2D measureText API.
        /// Returns (width, height) where height is estimated from the font size.
        pub fn measure_text(&self, text: &str, font_size: f64, bold: bool) -> (f64, f64) {
            let font = font_string(font_size, bold);
            self.ctx.set_font(&font);
            let metrics = self.ctx.measure_text(text).unwrap();
            let width = metrics.width();
            let height = font_size * 1.2; // line-height estimate
            (width, height)
        }

        /// Trace a rounded rectangle path using arc curves.
        fn draw_rounded_rect_path(&self, x: f64, y: f64, w: f64, h: f64, r: f64) {
            // Clamp radius to half the smallest dimension
            let r = r.min(w / 2.0).min(h / 2.0);

            self.ctx.begin_path();
            self.ctx.move_to(x + r, y);
            self.ctx.line_to(x + w - r, y);
            self.ctx
                .arc_to(x + w, y, x + w, y + r, r)
                .unwrap();
            self.ctx.line_to(x + w, y + h - r);
            self.ctx
                .arc_to(x + w, y + h, x + w - r, y + h, r)
                .unwrap();
            self.ctx.line_to(x + r, y + h);
            self.ctx
                .arc_to(x, y + h, x, y + h - r, r)
                .unwrap();
            self.ctx.line_to(x, y + r);
            self.ctx
                .arc_to(x, y, x + r, y, r)
                .unwrap();
            self.ctx.close_path();
        }

        /// Get a reference to the underlying 2D context.
        pub fn context(&self) -> &CanvasRenderingContext2d {
            &self.ctx
        }

        /// Get the device pixel ratio.
        pub fn dpr(&self) -> f64 {
            self.dpr
        }

        /// Get a reference to the underlying canvas element.
        pub fn canvas_element(&self) -> &HtmlCanvasElement {
            &self.canvas
        }

        /// Set the cursor style on the canvas element.
        pub fn set_cursor(&self, cursor: &str) {
            let _ = self.canvas.style().set_property("cursor", cursor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn color_to_css_black() {
        assert_eq!(color_to_css(0x000000), "#000000");
    }

    #[test]
    fn color_to_css_red() {
        assert_eq!(color_to_css(0xff0000), "#ff0000");
    }

    #[test]
    fn color_to_css_mixed() {
        assert_eq!(color_to_css(0x1e293b), "#1e293b");
    }

    #[test]
    fn get_color_prop_found() {
        let mut props = HashMap::new();
        props.insert("color".to_string(), RenderValue::Color(0xff0000));
        assert_eq!(get_color_prop(&props, "color", "#000000"), "#ff0000");
    }

    #[test]
    fn get_color_prop_missing() {
        let props = HashMap::new();
        assert_eq!(get_color_prop(&props, "color", "#000000"), "#000000");
    }

    #[test]
    fn get_num_prop_found() {
        let mut props = HashMap::new();
        props.insert(
            "radius".to_string(),
            RenderValue::Num(8.0, Some("px".to_string())),
        );
        assert!((get_num_prop(&props, "radius", 0.0) - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn get_num_prop_missing() {
        let props = HashMap::new();
        assert!((get_num_prop(&props, "radius", 0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn font_string_normal() {
        assert_eq!(font_string(16.0, false), "16px sans-serif");
    }

    #[test]
    fn font_string_bold() {
        assert_eq!(font_string(24.0, true), "bold 24px sans-serif");
    }

    #[test]
    fn font_string_custom_size() {
        assert_eq!(font_string(18.0, false), "18px sans-serif");
    }

    #[test]
    fn get_text_content_found() {
        let mut props = HashMap::new();
        props.insert("__text".to_string(), RenderValue::Str("hello".to_string()));
        assert_eq!(get_text_content(&props), "hello");
    }

    #[test]
    fn get_text_content_missing() {
        let props = HashMap::new();
        assert_eq!(get_text_content(&props), "");
    }

    #[test]
    fn get_font_size_default_text() {
        let props = HashMap::new();
        assert!((get_font_size(&props, false) - DEFAULT_TEXT_SIZE).abs() < f64::EPSILON);
    }

    #[test]
    fn get_font_size_default_heading() {
        let props = HashMap::new();
        assert!((get_font_size(&props, true) - DEFAULT_HEADING_SIZE).abs() < f64::EPSILON);
    }

    #[test]
    fn get_font_size_custom() {
        let mut props = HashMap::new();
        props.insert(
            "font-size".to_string(),
            RenderValue::Num(18.0, Some("px".to_string())),
        );
        assert!((get_font_size(&props, false) - 18.0).abs() < f64::EPSILON);
    }
}
