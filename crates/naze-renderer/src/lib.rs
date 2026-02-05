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

/// Extract a string prop with fallback.
pub fn get_str_prop(
    props: &std::collections::HashMap<String, RenderValue>,
    key: &str,
    default: &str,
) -> String {
    match props.get(key) {
        Some(RenderValue::Str(s)) => s.clone(),
        _ => default.to_string(),
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
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

    /// Image load state
    #[derive(Clone)]
    pub enum ImageState {
        Loading,
        Loaded(HtmlImageElement),
        Failed,
    }

    /// A Canvas2D renderer that draws to an HTML canvas element.
    pub struct Renderer {
        ctx: CanvasRenderingContext2d,
        canvas: HtmlCanvasElement,
        dpr: f64,
        image_cache: Rc<RefCell<HashMap<String, ImageState>>>,
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

            Ok(Self {
                ctx,
                canvas,
                dpr,
                image_cache: Rc::new(RefCell::new(HashMap::new())),
            })
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

        /// Set the global alpha (opacity) for subsequent draw operations.
        pub fn set_global_alpha(&self, alpha: f64) {
            self.ctx.set_global_alpha(alpha);
        }

        /// Save the current canvas state (including global alpha).
        pub fn save(&self) {
            self.ctx.save();
        }

        /// Restore the previously saved canvas state.
        pub fn restore(&self) {
            self.ctx.restore();
        }

        /// Load an image from URL if not already cached.
        /// Returns true if image is ready, false if still loading.
        pub fn load_image<F: FnOnce() + 'static>(&self, src: &str, on_load: F) -> bool {
            let mut cache = self.image_cache.borrow_mut();

            if let Some(state) = cache.get(src) {
                return matches!(state, ImageState::Loaded(_));
            }

            // Start loading
            cache.insert(src.to_string(), ImageState::Loading);

            let image = HtmlImageElement::new().unwrap();
            image.set_cross_origin(Some("anonymous"));

            let cache_clone = Rc::clone(&self.image_cache);

            let onload = Closure::once(Box::new(move || {
                // Note: we capture image in this closure to keep it alive
                on_load();
            }) as Box<dyn FnOnce()>);

            let image_clone = image.clone();
            let src_for_success = src.to_string();
            let cache_for_success = Rc::clone(&self.image_cache);
            let success_cb = Closure::once(Box::new(move || {
                cache_for_success
                    .borrow_mut()
                    .insert(src_for_success, ImageState::Loaded(image_clone));
                onload.forget(); // Transfer ownership, will be called
            }) as Box<dyn FnOnce()>);

            let src_for_error = src.to_string();
            let cache_for_error = Rc::clone(&cache_clone);
            let error_cb = Closure::once(Box::new(move || {
                cache_for_error
                    .borrow_mut()
                    .insert(src_for_error, ImageState::Failed);
            }) as Box<dyn FnOnce()>);

            image.set_onload(Some(success_cb.as_ref().unchecked_ref()));
            image.set_onerror(Some(error_cb.as_ref().unchecked_ref()));
            success_cb.forget();
            error_cb.forget();

            image.set_src(src);

            false
        }

        /// Draw an image at the given position. Returns true if drawn, false if still loading.
        pub fn draw_image(&self, src: &str, x: f64, y: f64, w: f64, h: f64, fit: &str) -> bool {
            let cache = self.image_cache.borrow();

            if let Some(ImageState::Loaded(img)) = cache.get(src) {
                let img_w = img.natural_width() as f64;
                let img_h = img.natural_height() as f64;

                if img_w == 0.0 || img_h == 0.0 {
                    return false;
                }

                match fit {
                    "fill" => {
                        // Stretch to fill
                        let _ = self.ctx.draw_image_with_html_image_element_and_dw_and_dh(
                            img, x, y, w, h,
                        );
                    }
                    "cover" => {
                        // Scale to cover, crop overflow
                        let scale = (w / img_w).max(h / img_h);
                        let sw = w / scale;
                        let sh = h / scale;
                        let sx = (img_w - sw) / 2.0;
                        let sy = (img_h - sh) / 2.0;
                        let _ = self.ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                            img, sx, sy, sw, sh, x, y, w, h,
                        );
                    }
                    _ => {
                        // "contain" (default) - scale to fit, letterbox
                        let scale = (w / img_w).min(h / img_h);
                        let dw = img_w * scale;
                        let dh = img_h * scale;
                        let dx = x + (w - dw) / 2.0;
                        let dy = y + (h - dh) / 2.0;
                        let _ = self.ctx.draw_image_with_html_image_element_and_dw_and_dh(
                            img, dx, dy, dw, dh,
                        );
                    }
                }
                true
            } else {
                false
            }
        }

        /// Draw a checkbox with optional label.
        pub fn draw_checkbox(&self, x: f64, y: f64, checked: bool, label: &str) {
            // Box dimensions
            let box_size = 20.0;
            let box_x = x;
            let box_y = y;

            // Draw box background
            self.ctx.set_fill_style_str("#ffffff");
            self.draw_rounded_rect_path(box_x, box_y, box_size, box_size, 3.0);
            self.ctx.fill();

            // Draw box border
            self.ctx.set_stroke_style_str("#9ca3af");
            self.ctx.set_line_width(2.0);
            self.draw_rounded_rect_path(box_x, box_y, box_size, box_size, 3.0);
            self.ctx.stroke();

            // Draw checkmark if checked
            if checked {
                self.ctx.begin_path();
                self.ctx.move_to(box_x + 4.0, box_y + 10.0);
                self.ctx.line_to(box_x + 8.0, box_y + 15.0);
                self.ctx.line_to(box_x + 16.0, box_y + 5.0);
                self.ctx.set_stroke_style_str("#2563eb");
                self.ctx.set_line_width(2.5);
                self.ctx.stroke();
            }

            // Draw label
            if !label.is_empty() {
                self.draw_text(label, x + 28.0, y + 2.0, 16.0, false, "#374151");
            }
        }

        /// Draw a radio button with optional label.
        pub fn draw_radio(&self, x: f64, y: f64, selected: bool, label: &str) {
            let size = 20.0;
            let radius = size / 2.0;

            // Draw outer circle as a rounded rect with full radius (makes a circle)
            // Background
            self.ctx.set_fill_style_str("#ffffff");
            self.draw_rounded_rect_path(x, y, size, size, radius);
            self.ctx.fill();

            // Border
            self.ctx.set_stroke_style_str("#9ca3af");
            self.ctx.set_line_width(2.0);
            self.draw_rounded_rect_path(x, y, size, size, radius);
            self.ctx.stroke();

            // Draw inner filled circle if selected
            if selected {
                let inner_size = 10.0;
                let inner_radius = inner_size / 2.0;
                let inner_x = x + (size - inner_size) / 2.0;
                let inner_y = y + (size - inner_size) / 2.0;
                self.ctx.set_fill_style_str("#2563eb");
                self.draw_rounded_rect_path(inner_x, inner_y, inner_size, inner_size, inner_radius);
                self.ctx.fill();
            }

            // Draw label
            if !label.is_empty() {
                self.draw_text(label, x + 28.0, y + 2.0, 16.0, false, "#374151");
            }
        }

        /// Draw a text input field.
        pub fn draw_input(&self, x: f64, y: f64, w: f64, h: f64, value: &str, placeholder: &str, focused: bool, input_type: &str, show_caret: bool) {
            // Background
            self.ctx.set_fill_style_str("#ffffff");
            self.draw_rounded_rect_path(x, y, w, h, 4.0);
            self.ctx.fill();

            // Border - blue when focused, gray otherwise
            let border_color = if focused { "#2563eb" } else { "#d1d5db" };
            self.ctx.set_stroke_style_str(border_color);
            self.ctx.set_line_width(if focused { 2.0 } else { 1.0 });
            self.draw_rounded_rect_path(x, y, w, h, 4.0);
            self.ctx.stroke();

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
                self.draw_text(&display_value, text_x, text_y, 16.0, false, "#111827");
            } else if !placeholder.is_empty() {
                self.draw_text(placeholder, text_x, text_y, 16.0, false, "#9ca3af");
            }

            // Draw cursor when show_caret is true (handles blinking)
            if show_caret {
                let (text_width, _) = self.measure_text(&display_value, 16.0, false);
                let cursor_x = text_x + text_width;
                self.ctx.set_stroke_style_str("#111827");
                self.ctx.set_line_width(1.0);
                self.ctx.begin_path();
                self.ctx.move_to(cursor_x, y + 6.0);
                self.ctx.line_to(cursor_x, y + h - 6.0);
                self.ctx.stroke();
            }
        }

        /// Draw a select/dropdown element.
        pub fn draw_select(
            &self,
            x: f64,
            y: f64,
            w: f64,
            h: f64,
            display_text: &str,
            placeholder: &str,
            is_open: bool,
            options: &[(String, String)], // (label, value)
            selected_value: &str,
        ) {
            // Background
            self.ctx.set_fill_style_str("#ffffff");
            self.draw_rounded_rect_path(x, y, w, h, 4.0);
            self.ctx.fill();

            // Border
            let border_color = if is_open { "#2563eb" } else { "#d1d5db" };
            self.ctx.set_stroke_style_str(border_color);
            self.ctx.set_line_width(if is_open { 2.0 } else { 1.0 });
            self.draw_rounded_rect_path(x, y, w, h, 4.0);
            self.ctx.stroke();

            // Display text or placeholder
            let text_x = x + 12.0;
            let text_y = y + 8.0;
            if !display_text.is_empty() {
                self.draw_text(display_text, text_x, text_y, 16.0, false, "#111827");
            } else if !placeholder.is_empty() {
                self.draw_text(placeholder, text_x, text_y, 16.0, false, "#9ca3af");
            }

            // Dropdown arrow
            let arrow_x = x + w - 24.0;
            let arrow_y = y + h / 2.0;
            self.ctx.set_stroke_style_str("#6b7280");
            self.ctx.set_line_width(2.0);
            self.ctx.begin_path();
            if is_open {
                // Up arrow when open
                self.ctx.move_to(arrow_x, arrow_y + 2.0);
                self.ctx.line_to(arrow_x + 6.0, arrow_y - 4.0);
                self.ctx.line_to(arrow_x + 12.0, arrow_y + 2.0);
            } else {
                // Down arrow when closed
                self.ctx.move_to(arrow_x, arrow_y - 2.0);
                self.ctx.line_to(arrow_x + 6.0, arrow_y + 4.0);
                self.ctx.line_to(arrow_x + 12.0, arrow_y - 2.0);
            }
            self.ctx.stroke();

            // Draw dropdown options if open
            if is_open && !options.is_empty() {
                let option_h = 32.0;
                let dropdown_h = options.len() as f64 * option_h;
                let dropdown_y = y + h + 4.0;

                // Dropdown background with shadow effect
                self.ctx.set_fill_style_str("#ffffff");
                self.draw_rounded_rect_path(x, dropdown_y, w, dropdown_h, 4.0);
                self.ctx.fill();

                // Dropdown border
                self.ctx.set_stroke_style_str("#e5e7eb");
                self.ctx.set_line_width(1.0);
                self.draw_rounded_rect_path(x, dropdown_y, w, dropdown_h, 4.0);
                self.ctx.stroke();

                // Draw each option
                for (i, (label, value)) in options.iter().enumerate() {
                    let opt_y = dropdown_y + (i as f64 * option_h);
                    let is_selected = value == selected_value;

                    // Highlight selected option
                    if is_selected {
                        self.ctx.set_fill_style_str("#eff6ff");
                        self.ctx.fill_rect(x + 1.0, opt_y, w - 2.0, option_h);
                    }

                    // Option text
                    let color = if is_selected { "#2563eb" } else { "#374151" };
                    self.draw_text(label, x + 12.0, opt_y + 8.0, 16.0, false, color);
                }
            }
        }

        /// Draw a filled rectangle with optional border.
        pub fn draw_rect_with_border(
            &self,
            x: f64,
            y: f64,
            w: f64,
            h: f64,
            color: &str,
            radius: f64,
            border: f64,
            border_color: &str,
        ) {
            // Draw fill
            if !color.is_empty() {
                self.ctx.set_fill_style_str(color);
                if radius > 0.0 {
                    self.draw_rounded_rect_path(x, y, w, h, radius);
                    self.ctx.fill();
                } else {
                    self.ctx.fill_rect(x, y, w, h);
                }
            }

            // Draw border if specified
            if border > 0.0 && !border_color.is_empty() {
                self.ctx.set_stroke_style_str(border_color);
                self.ctx.set_line_width(border);
                if radius > 0.0 {
                    self.draw_rounded_rect_path(x, y, w, h, radius);
                    self.ctx.stroke();
                } else {
                    self.ctx.stroke_rect(x, y, w, h);
                }
            }
        }

        /// Draw a semi-transparent ghost element for drag operations.
        pub fn draw_drag_ghost(&self, x: f64, y: f64, w: f64, h: f64, color: &str) {
            self.save();
            self.set_global_alpha(0.6);
            self.ctx.set_fill_style_str(color);
            self.draw_rounded_rect_path(x, y, w, h, 8.0);
            self.ctx.fill();
            // Add a slight shadow/border effect
            self.ctx.set_stroke_style_str("#000000");
            self.ctx.set_line_width(1.0);
            self.set_global_alpha(0.2);
            self.draw_rounded_rect_path(x, y, w, h, 8.0);
            self.ctx.stroke();
            self.restore();
        }

        /// Draw a dashed highlight border around a drop target.
        pub fn draw_drop_highlight(&self, x: f64, y: f64, w: f64, h: f64) {
            self.save();
            self.ctx.set_stroke_style_str("#3b82f6"); // Blue highlight
            self.ctx.set_line_width(3.0);
            // Create dashed line effect
            let dash_array = js_sys::Array::new();
            dash_array.push(&8.0.into());
            dash_array.push(&4.0.into());
            self.ctx.set_line_dash(&dash_array).unwrap();
            self.draw_rounded_rect_path(x, y, w, h, 4.0);
            self.ctx.stroke();
            // Reset line dash
            self.ctx.set_line_dash(&js_sys::Array::new()).unwrap();
            self.restore();
        }

        /// Begin a clipping region. Content drawn after this will be clipped to the rectangle.
        /// Must be paired with end_clip().
        pub fn begin_clip(&self, x: f64, y: f64, w: f64, h: f64, radius: f64) {
            self.ctx.save();
            self.ctx.begin_path();
            if radius > 0.0 {
                self.draw_rounded_rect_path(x, y, w, h, radius);
            } else {
                self.ctx.rect(x, y, w, h);
            }
            self.ctx.clip();
        }

        /// End a clipping region started with begin_clip().
        pub fn end_clip(&self) {
            self.ctx.restore();
        }

        /// Translate the canvas origin for drawing scrolled content.
        pub fn translate(&self, x: f64, y: f64) {
            let _ = self.ctx.translate(x, y);
        }

        /// Draw a vertical scrollbar.
        pub fn draw_scrollbar_vertical(&self, x: f64, y: f64, height: f64, thumb_pos: f64, thumb_size: f64) {
            let bar_width = 8.0;
            let bar_x = x - bar_width - 2.0; // 2px margin from edge

            // Track background
            self.ctx.set_fill_style_str("#f1f5f9");
            self.draw_rounded_rect_path(bar_x, y, bar_width, height, 4.0);
            self.ctx.fill();

            // Thumb
            self.ctx.set_fill_style_str("#94a3b8");
            self.draw_rounded_rect_path(bar_x, y + thumb_pos, bar_width, thumb_size, 4.0);
            self.ctx.fill();
        }

        /// Draw a horizontal scrollbar.
        pub fn draw_scrollbar_horizontal(&self, x: f64, y: f64, width: f64, thumb_pos: f64, thumb_size: f64) {
            let bar_height = 8.0;
            let bar_y = y - bar_height - 2.0; // 2px margin from edge

            // Track background
            self.ctx.set_fill_style_str("#f1f5f9");
            self.draw_rounded_rect_path(x, bar_y, width, bar_height, 4.0);
            self.ctx.fill();

            // Thumb
            self.ctx.set_fill_style_str("#94a3b8");
            self.draw_rounded_rect_path(x + thumb_pos, bar_y, thumb_size, bar_height, 4.0);
            self.ctx.fill();
        }

        /// Draw a focus ring around an element for keyboard navigation visibility.
        pub fn draw_focus_ring(&self, x: f64, y: f64, w: f64, h: f64, radius: f64) {
            self.save();
            // Offset the ring slightly outside the element
            let offset = 2.0;
            let ring_x = x - offset;
            let ring_y = y - offset;
            let ring_w = w + offset * 2.0;
            let ring_h = h + offset * 2.0;
            let ring_radius = if radius > 0.0 { radius + offset } else { 4.0 };

            // Blue focus ring (matches browser default focus style)
            self.ctx.set_stroke_style_str("#2563eb");
            self.ctx.set_line_width(2.0);
            self.draw_rounded_rect_path(ring_x, ring_y, ring_w, ring_h, ring_radius);
            self.ctx.stroke();

            // Outer glow effect for better visibility
            self.set_global_alpha(0.3);
            self.ctx.set_stroke_style_str("#3b82f6");
            self.ctx.set_line_width(4.0);
            self.draw_rounded_rect_path(ring_x - 1.0, ring_y - 1.0, ring_w + 2.0, ring_h + 2.0, ring_radius + 1.0);
            self.ctx.stroke();

            self.restore();
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
