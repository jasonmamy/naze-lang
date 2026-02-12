//! Static HTML renderer: converts resolved RenderNode trees to semantic HTML + CSS.
//! Used by `nazec build --static` for SSG (static site generation).

use naze_ir::{RenderNode, RenderTree, RenderValue};

use crate::exec;

// ─── Public API ──────────────────────────────────────────────────────────────

/// Generate static HTML content from a RenderTree by resolving initial state
/// and converting the node tree to semantic HTML with inline CSS.
pub fn generate_static_html(tree: &RenderTree) -> String {
    let state = exec::init_state(tree);
    let resolved = exec::resolve_nodes(&tree.root, &state);
    render_to_html(&resolved)
}

/// Generate static HTML for a specific page's root nodes.
pub fn generate_static_html_for_page(
    tree: &RenderTree,
    page_root: &[RenderNode],
) -> String {
    let state = exec::init_state(tree);
    let resolved = exec::resolve_nodes(page_root, &state);
    render_to_html(&resolved)
}

/// Convert resolved RenderNodes to an HTML string.
pub fn render_to_html(nodes: &[RenderNode]) -> String {
    let mut html = String::new();
    for node in nodes {
        html.push_str(&render_node(node));
    }
    html
}

// ─── Node rendering ─────────────────────────────────────────────────────────

fn render_node(node: &RenderNode) -> String {
    match node.kind.as_str() {
        "text" => render_text(node),
        "heading" => render_heading(node),
        "row" => render_flex_container(node, "row"),
        "column" | "container" => render_flex_container(node, "column"),
        "stack" => render_stack(node),
        "grid" => render_grid(node),
        "rect" => render_rect(node),
        "image" => render_image(node),
        "spacer" => render_spacer(node),
        "divider" => render_divider(node),
        "link" => render_link(node),
        "button" => render_button(node),
        "input" => render_input(node),
        "textarea" => render_textarea(node),
        "checkbox" => render_checkbox(node),
        "radio" => render_radio(node),
        "select" => render_select(node),
        "option" => render_option(node),
        "scroll" => render_scroll(node),
        "nav" => render_nav(node),
        "overlay" => render_overlay(node),
        _ => {
            // Unknown kind — render as div with children
            let style = build_style(node, false);
            let children = render_children(&node.children);
            format!("<div{}>{}</div>", style_attr(&style), children)
        }
    }
}

fn render_children(children: &[RenderNode]) -> String {
    let mut html = String::new();
    for child in children {
        html.push_str(&render_node(child));
    }
    html
}

fn render_text(node: &RenderNode) -> String {
    let text = get_text_content(node);
    let style = build_style(node, true);
    format!("<p{}>{}</p>", style_attr(&style), escape_html(&text))
}

fn render_heading(node: &RenderNode) -> String {
    let text = get_text_content(node);
    let level = heading_level(node);
    let style = build_style(node, true);
    format!(
        "<h{level}{style}>{text}</h{level}>",
        level = level,
        style = style_attr(&style),
        text = escape_html(&text)
    )
}

fn render_flex_container(node: &RenderNode, direction: &str) -> String {
    let mut styles = Vec::new();
    styles.push("display:flex".to_string());
    if direction == "column" {
        styles.push("flex-direction:column".to_string());
    }
    add_common_styles(node, false, &mut styles);
    let children = render_children(&node.children);
    format!(
        "<div style=\"{}\">{}</div>",
        styles.join(";"),
        children
    )
}

fn render_stack(node: &RenderNode) -> String {
    let mut styles = Vec::new();
    styles.push("position:relative".to_string());
    add_common_styles(node, false, &mut styles);
    // Stack children need position:absolute
    let mut children_html = String::new();
    for child in &node.children {
        let child_html = render_node(child);
        // Wrap in absolute-positioned container
        children_html.push_str(&format!(
            "<div style=\"position:absolute;inset:0\">{}</div>",
            child_html
        ));
    }
    format!(
        "<div style=\"{}\">{}</div>",
        styles.join(";"),
        children_html
    )
}

fn render_grid(node: &RenderNode) -> String {
    let mut styles = Vec::new();
    styles.push("display:grid".to_string());
    if let Some(cols) = get_num_prop(node, "columns") {
        styles.push(format!(
            "grid-template-columns:repeat({},1fr)",
            cols as i64
        ));
    }
    add_common_styles(node, false, &mut styles);
    let children = render_children(&node.children);
    format!(
        "<div style=\"{}\">{}</div>",
        styles.join(";"),
        children
    )
}

fn render_rect(node: &RenderNode) -> String {
    let style = build_style(node, false);
    let children = render_children(&node.children);
    format!("<div{}>{}</div>", style_attr(&style), children)
}

fn render_image(node: &RenderNode) -> String {
    let src = get_str_prop(node, "src").unwrap_or_default();
    let alt = get_str_prop(node, "alt").unwrap_or_default();
    let style = build_style(node, false);
    format!(
        "<img src=\"{}\" alt=\"{}\"{}/>",
        escape_attr(&src),
        escape_attr(&alt),
        style_attr(&style)
    )
}

fn render_spacer(node: &RenderNode) -> String {
    let style = build_style(node, false);
    format!("<div{}></div>", style_attr(&style))
}

fn render_divider(node: &RenderNode) -> String {
    let mut styles = Vec::new();
    if let Some(color) = get_color_prop(node, "color") {
        styles.push(format!("border-color:{}", color));
    }
    if !styles.is_empty() {
        format!("<hr style=\"{}\"/>", styles.join(";"))
    } else {
        "<hr/>".to_string()
    }
}

fn render_link(node: &RenderNode) -> String {
    let href = get_str_prop(node, "href").unwrap_or_default();
    let style = build_style(node, false);
    let children = render_children(&node.children);
    let text = if children.is_empty() {
        let t = get_text_content(node);
        escape_html(&t)
    } else {
        children
    };
    format!(
        "<a href=\"{}\"{}>{}</a>",
        escape_attr(&href),
        style_attr(&style),
        text
    )
}

fn render_button(node: &RenderNode) -> String {
    let label = get_str_prop(node, "label").unwrap_or_default();
    let style = build_style(node, false);
    format!(
        "<button{}>{}</button>",
        style_attr(&style),
        escape_html(&label)
    )
}

fn render_input(node: &RenderNode) -> String {
    let input_type = get_str_prop(node, "type").unwrap_or_else(|| "text".to_string());
    let placeholder = get_str_prop(node, "placeholder").unwrap_or_default();
    let style = build_style(node, false);
    format!(
        "<input type=\"{}\" placeholder=\"{}\"{}/>",
        escape_attr(&input_type),
        escape_attr(&placeholder),
        style_attr(&style)
    )
}

fn render_textarea(node: &RenderNode) -> String {
    let placeholder = get_str_prop(node, "placeholder").unwrap_or_default();
    let style = build_style(node, false);
    format!(
        "<textarea placeholder=\"{}\"{}></textarea>",
        escape_attr(&placeholder),
        style_attr(&style)
    )
}

fn render_checkbox(node: &RenderNode) -> String {
    let label = get_str_prop(node, "label").unwrap_or_default();
    format!(
        "<label><input type=\"checkbox\"/>{}</label>",
        escape_html(&label)
    )
}

fn render_radio(node: &RenderNode) -> String {
    let label = get_str_prop(node, "label").unwrap_or_default();
    let value = get_str_prop(node, "value").unwrap_or_default();
    format!(
        "<label><input type=\"radio\" value=\"{}\"/>{}</label>",
        escape_attr(&value),
        escape_html(&label)
    )
}

fn render_select(node: &RenderNode) -> String {
    let style = build_style(node, false);
    let children = render_children(&node.children);
    format!("<select{}>{}</select>", style_attr(&style), children)
}

fn render_option(node: &RenderNode) -> String {
    let text = get_text_content(node);
    let value = get_str_prop(node, "value").unwrap_or_else(|| text.clone());
    format!(
        "<option value=\"{}\">{}</option>",
        escape_attr(&value),
        escape_html(&text)
    )
}

fn render_scroll(node: &RenderNode) -> String {
    let mut styles = Vec::new();
    styles.push("overflow:auto".to_string());
    add_common_styles(node, false, &mut styles);
    let children = render_children(&node.children);
    format!(
        "<div style=\"{}\">{}</div>",
        styles.join(";"),
        children
    )
}

fn render_nav(node: &RenderNode) -> String {
    let mut styles = Vec::new();
    styles.push("display:flex".to_string());
    add_common_styles(node, false, &mut styles);
    let children = render_children(&node.children);
    format!(
        "<nav style=\"{}\">{}</nav>",
        styles.join(";"),
        children
    )
}

fn render_overlay(node: &RenderNode) -> String {
    let mut styles = Vec::new();
    styles.push("position:fixed".to_string());
    styles.push("inset:0".to_string());
    add_common_styles(node, false, &mut styles);
    let children = render_children(&node.children);
    format!(
        "<div style=\"{}\">{}</div>",
        styles.join(";"),
        children
    )
}

// ─── Style building ─────────────────────────────────────────────────────────

fn build_style(node: &RenderNode, is_text: bool) -> String {
    let mut styles = Vec::new();
    add_common_styles(node, is_text, &mut styles);
    styles.join(";")
}

fn add_common_styles(node: &RenderNode, is_text: bool, styles: &mut Vec<String>) {
    // Color: on text/heading = text color, on containers = background-color
    if let Some(color) = get_color_prop(node, "color") {
        if is_text {
            styles.push(format!("color:{}", color));
        } else {
            styles.push(format!("background-color:{}", color));
        }
    }

    // Font properties
    if let Some(size) = get_num_prop(node, "font-size") {
        styles.push(format!("font-size:{}px", size));
    }
    if let Some(weight) = get_str_prop(node, "font-weight") {
        styles.push(format!("font-weight:{}", weight));
    } else if let Some(weight) = get_str_prop(node, "weight") {
        styles.push(format!("font-weight:{}", weight));
    }

    // Text alignment
    if let Some(align) = get_str_prop(node, "align") {
        if is_text {
            styles.push(format!("text-align:{}", align));
        } else {
            styles.push(format!("align-items:{}", css_align(&align)));
        }
    }

    // Justify
    if let Some(justify) = get_str_prop(node, "justify") {
        styles.push(format!("justify-content:{}", css_justify(&justify)));
    }

    // Spacing
    if let Some(gap) = get_num_prop(node, "gap") {
        styles.push(format!("gap:{}px", gap));
    }
    if let Some(padding) = get_num_prop(node, "padding") {
        styles.push(format!("padding:{}px", padding));
    }

    // Dimensions
    if let Some(w) = get_dimension_prop(node, "width") {
        styles.push(format!("width:{}", w));
    }
    if let Some(h) = get_dimension_prop(node, "height") {
        styles.push(format!("height:{}", h));
    }
    if let Some(v) = get_dimension_prop(node, "min-width") {
        styles.push(format!("min-width:{}", v));
    }
    if let Some(v) = get_dimension_prop(node, "max-width") {
        styles.push(format!("max-width:{}", v));
    }
    if let Some(v) = get_dimension_prop(node, "min-height") {
        styles.push(format!("min-height:{}", v));
    }
    if let Some(v) = get_dimension_prop(node, "max-height") {
        styles.push(format!("max-height:{}", v));
    }

    // Border
    if let Some(radius) = get_num_prop(node, "radius") {
        styles.push(format!("border-radius:{}px", radius));
    }
    if let Some(border) = get_num_prop(node, "border") {
        let border_color = get_color_prop(node, "border-color")
            .unwrap_or_else(|| "#000000".to_string());
        styles.push(format!("border:{}px solid {}", border, border_color));
    }

    // Shadow presets (matching naze-renderer)
    if let Some(shadow) = get_str_prop(node, "shadow") {
        if let Some(css) = shadow_to_css(&shadow) {
            styles.push(format!("box-shadow:{}", css));
        }
    }

    // Opacity
    if let Some(opacity) = get_num_prop(node, "opacity") {
        styles.push(format!("opacity:{}", opacity));
    }
}

fn shadow_to_css(preset: &str) -> Option<String> {
    match preset {
        "sm" => Some("0 1px 2px rgba(0,0,0,0.1)".to_string()),
        "md" => Some("0 4px 6px rgba(0,0,0,0.1)".to_string()),
        "lg" => Some("0 10px 15px rgba(0,0,0,0.1)".to_string()),
        "xl" => Some("0 20px 25px rgba(0,0,0,0.1)".to_string()),
        _ => None,
    }
}

fn css_align(align: &str) -> &str {
    match align {
        "center" => "center",
        "right" | "end" => "flex-end",
        "left" | "start" => "flex-start",
        _ => align,
    }
}

fn css_justify(justify: &str) -> &str {
    match justify {
        "center" => "center",
        "end" => "flex-end",
        "start" => "flex-start",
        "between" | "space-between" => "space-between",
        "around" | "space-around" => "space-around",
        "evenly" | "space-evenly" => "space-evenly",
        _ => justify,
    }
}

fn style_attr(style: &str) -> String {
    if style.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", style)
    }
}

// ─── Property helpers ────────────────────────────────────────────────────────

fn get_text_content(node: &RenderNode) -> String {
    match node.props.get("__text") {
        Some(RenderValue::Str(s)) => s.clone(),
        _ => String::new(),
    }
}

fn get_str_prop(node: &RenderNode, key: &str) -> Option<String> {
    match node.props.get(key) {
        Some(RenderValue::Str(s)) => Some(s.clone()),
        _ => None,
    }
}

fn get_num_prop(node: &RenderNode, key: &str) -> Option<f64> {
    match node.props.get(key) {
        Some(RenderValue::Num(n, _)) => Some(*n),
        _ => None,
    }
}

fn get_color_prop(node: &RenderNode, key: &str) -> Option<String> {
    match node.props.get(key) {
        Some(RenderValue::Color(c)) => Some(color_to_css(*c)),
        _ => None,
    }
}

fn get_dimension_prop(node: &RenderNode, key: &str) -> Option<String> {
    match node.props.get(key) {
        Some(RenderValue::Num(n, Some(unit))) => Some(format!("{}{}", n, unit)),
        Some(RenderValue::Num(n, None)) => Some(format!("{}px", n)),
        Some(RenderValue::Str(s)) => Some(s.clone()),
        _ => None,
    }
}

fn heading_level(node: &RenderNode) -> u8 {
    // Use level prop if present, otherwise infer from font-size
    if let Some(level) = get_num_prop(node, "level") {
        return (level as u8).clamp(1, 6);
    }
    match get_num_prop(node, "font-size") {
        Some(size) if size >= 32.0 => 1,
        Some(size) if size >= 24.0 => 2,
        Some(size) if size >= 20.0 => 3,
        Some(size) if size >= 16.0 => 4,
        _ => 1,
    }
}

fn color_to_css(c: u32) -> String {
    format!("#{:06x}", c & 0xFFFFFF)
}

// ─── HTML escaping ──────────────────────────────────────────────────────────

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn text_node(text: &str) -> RenderNode {
        let mut props = HashMap::new();
        props.insert("__text".to_string(), RenderValue::Str(text.to_string()));
        RenderNode {
            kind: "text".to_string(),
            props,
            children: vec![],
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        }
    }

    fn heading_node(text: &str, font_size: f64) -> RenderNode {
        let mut props = HashMap::new();
        props.insert("__text".to_string(), RenderValue::Str(text.to_string()));
        props.insert("font-size".to_string(), RenderValue::Num(font_size, None));
        RenderNode {
            kind: "heading".to_string(),
            props,
            children: vec![],
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        }
    }

    fn container_node(kind: &str, props: HashMap<String, RenderValue>, children: Vec<RenderNode>) -> RenderNode {
        RenderNode {
            kind: kind.to_string(),
            props,
            children,
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        }
    }

    #[test]
    fn render_text_node() {
        let html = render_node(&text_node("Hello"));
        assert_eq!(html, "<p>Hello</p>");
    }

    #[test]
    fn render_text_escapes_html() {
        let html = render_node(&text_node("<script>alert('xss')</script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn render_heading_node() {
        let html = render_node(&heading_node("Title", 24.0));
        assert!(html.starts_with("<h2"));
        assert!(html.contains("Title"));
        assert!(html.contains("font-size:24px"));
    }

    #[test]
    fn render_heading_h1() {
        let html = render_node(&heading_node("Big", 32.0));
        assert!(html.starts_with("<h1"));
    }

    #[test]
    fn render_row_layout() {
        let row = container_node("row", {
            let mut p = HashMap::new();
            p.insert("gap".to_string(), RenderValue::Num(16.0, None));
            p
        }, vec![text_node("A"), text_node("B")]);
        let html = render_node(&row);
        assert!(html.contains("display:flex"));
        assert!(!html.contains("flex-direction:column"));
        assert!(html.contains("gap:16px"));
        assert!(html.contains("<p>A</p>"));
        assert!(html.contains("<p>B</p>"));
    }

    #[test]
    fn render_column_layout() {
        let col = container_node("column", HashMap::new(), vec![text_node("X")]);
        let html = render_node(&col);
        assert!(html.contains("display:flex"));
        assert!(html.contains("flex-direction:column"));
        assert!(html.contains("<p>X</p>"));
    }

    #[test]
    fn render_nested_tree() {
        let tree = container_node("column", HashMap::new(), vec![
            container_node("row", HashMap::new(), vec![
                text_node("Hello"),
                text_node("World"),
            ]),
        ]);
        let html = render_node(&tree);
        assert!(html.contains("flex-direction:column"));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("<p>World</p>"));
    }

    #[test]
    fn render_background_color() {
        let node = container_node("container", {
            let mut p = HashMap::new();
            p.insert("color".to_string(), RenderValue::Color(0x1e293b));
            p
        }, vec![]);
        let html = render_node(&node);
        assert!(html.contains("background-color:#1e293b"));
    }

    #[test]
    fn render_text_color() {
        let mut props = HashMap::new();
        props.insert("__text".to_string(), RenderValue::Str("Hi".to_string()));
        props.insert("color".to_string(), RenderValue::Color(0xff0000));
        let node = RenderNode {
            kind: "text".to_string(),
            props,
            children: vec![],
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        };
        let html = render_node(&node);
        assert!(html.contains("color:#ff0000"));
    }

    #[test]
    fn render_image_node() {
        let mut props = HashMap::new();
        props.insert("src".to_string(), RenderValue::Str("logo.png".to_string()));
        props.insert("alt".to_string(), RenderValue::Str("Logo".to_string()));
        props.insert("width".to_string(), RenderValue::Num(100.0, None));
        let node = RenderNode {
            kind: "image".to_string(),
            props,
            children: vec![],
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        };
        let html = render_node(&node);
        assert!(html.contains("src=\"logo.png\""));
        assert!(html.contains("alt=\"Logo\""));
        assert!(html.contains("width:100px"));
    }

    #[test]
    fn render_input_node() {
        let mut props = HashMap::new();
        props.insert("placeholder".to_string(), RenderValue::Str("Enter name".to_string()));
        let node = RenderNode {
            kind: "input".to_string(),
            props,
            children: vec![],
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        };
        let html = render_node(&node);
        assert!(html.contains("<input"));
        assert!(html.contains("placeholder=\"Enter name\""));
    }

    #[test]
    fn render_checkbox_node() {
        let mut props = HashMap::new();
        props.insert("label".to_string(), RenderValue::Str("Accept terms".to_string()));
        let node = RenderNode {
            kind: "checkbox".to_string(),
            props,
            children: vec![],
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        };
        let html = render_node(&node);
        assert!(html.contains("<label>"));
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("Accept terms"));
    }

    #[test]
    fn render_grid_node() {
        let mut props = HashMap::new();
        props.insert("columns".to_string(), RenderValue::Num(3.0, None));
        props.insert("gap".to_string(), RenderValue::Num(8.0, None));
        let node = container_node("grid", props, vec![text_node("A"), text_node("B"), text_node("C")]);
        let html = render_node(&node);
        assert!(html.contains("display:grid"));
        assert!(html.contains("grid-template-columns:repeat(3,1fr)"));
        assert!(html.contains("gap:8px"));
    }

    #[test]
    fn render_shadow_preset() {
        let mut props = HashMap::new();
        props.insert("shadow".to_string(), RenderValue::Str("md".to_string()));
        let node = container_node("rect", props, vec![]);
        let html = render_node(&node);
        assert!(html.contains("box-shadow:0 4px 6px rgba(0,0,0,0.1)"));
    }

    #[test]
    fn render_border_radius() {
        let mut props = HashMap::new();
        props.insert("radius".to_string(), RenderValue::Num(8.0, None));
        let node = container_node("rect", props, vec![]);
        let html = render_node(&node);
        assert!(html.contains("border-radius:8px"));
    }

    #[test]
    fn render_padding() {
        let mut props = HashMap::new();
        props.insert("padding".to_string(), RenderValue::Num(16.0, None));
        let node = container_node("column", props, vec![]);
        let html = render_node(&node);
        assert!(html.contains("padding:16px"));
    }

    #[test]
    fn generate_static_html_from_example() {
        use std::path::Path;

        let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples");

        let project = naze_compiler::resolve::resolve(&examples_dir, "dashboard-static.naze", &[]);
        assert!(
            project.errors.iter().all(|e| !matches!(e.severity, naze_compiler::error::Severity::Error)),
            "resolve errors: {:?}", project.errors
        );

        let tree = naze_compiler::codegen::lower(&project);
        let html = generate_static_html(&tree);

        // Should contain semantic HTML elements
        assert!(html.contains("<p>"), "should contain <p> elements");
        assert!(html.contains("<h"), "should contain heading elements");
        assert!(html.contains("display:flex"), "should contain flex layout");

        // Should contain actual text content from the example
        assert!(html.contains("Dashboard"), "should contain Dashboard title");
        assert!(html.contains("Revenue"), "should contain Revenue text");
        assert!(html.contains("Overview"), "should contain Overview text");

        // Should contain CSS styles
        assert!(html.contains("background-color:"), "should contain background colors");
        assert!(html.contains("padding:"), "should contain padding");
    }
}
