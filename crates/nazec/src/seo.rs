use crate::manifest::Manifest;
use naze_ir::{RenderNode, RenderValue, TextPart};

/// Generate HTML meta tags for SEO (description, Open Graph, Twitter Card).
pub fn generate_meta_tags(manifest: &Manifest, title: &str, page_path: Option<&str>) -> String {
    let mut tags = Vec::new();
    let page_title = page_title(title, page_path);

    // Standard meta tags
    if let Some(desc) = &manifest.seo.description {
        tags.push(format!(
            "  <meta name=\"description\" content=\"{}\">",
            escape_attr(desc)
        ));
    }
    if let Some(author) = &manifest.seo.author {
        tags.push(format!(
            "  <meta name=\"author\" content=\"{}\">",
            escape_attr(author)
        ));
    }
    if let Some(keywords) = &manifest.seo.keywords {
        tags.push(format!(
            "  <meta name=\"keywords\" content=\"{}\">",
            escape_attr(keywords)
        ));
    }

    // Open Graph
    tags.push(format!(
        "  <meta property=\"og:title\" content=\"{}\">",
        escape_attr(&page_title)
    ));
    tags.push("  <meta property=\"og:type\" content=\"website\">".to_string());
    if let Some(desc) = &manifest.seo.description {
        tags.push(format!(
            "  <meta property=\"og:description\" content=\"{}\">",
            escape_attr(desc)
        ));
    }
    if let Some(image) = &manifest.seo.image {
        let image_url = resolve_image_url(image, manifest.seo.canonical.as_deref());
        tags.push(format!(
            "  <meta property=\"og:image\" content=\"{}\">",
            escape_attr(&image_url)
        ));
    }
    if let Some(canonical) = &manifest.seo.canonical {
        let url = resolve_page_url(canonical, page_path);
        tags.push(format!(
            "  <meta property=\"og:url\" content=\"{}\">",
            escape_attr(&url)
        ));
    }
    if let Some(locale) = &manifest.seo.locale {
        tags.push(format!(
            "  <meta property=\"og:locale\" content=\"{}\">",
            escape_attr(locale)
        ));
    }

    // Twitter Card
    tags.push("  <meta name=\"twitter:card\" content=\"summary_large_image\">".to_string());
    tags.push(format!(
        "  <meta name=\"twitter:title\" content=\"{}\">",
        escape_attr(&page_title)
    ));
    if let Some(desc) = &manifest.seo.description {
        tags.push(format!(
            "  <meta name=\"twitter:description\" content=\"{}\">",
            escape_attr(desc)
        ));
    }
    if let Some(image) = &manifest.seo.image {
        tags.push(format!(
            "  <meta name=\"twitter:image\" content=\"{}\">",
            escape_attr(image)
        ));
    }
    if let Some(twitter) = &manifest.seo.twitter {
        tags.push(format!(
            "  <meta name=\"twitter:site\" content=\"{}\">",
            escape_attr(twitter)
        ));
    }

    // Canonical link
    if let Some(canonical) = &manifest.seo.canonical {
        let url = resolve_page_url(canonical, page_path);
        tags.push(format!(
            "  <link rel=\"canonical\" href=\"{}\">",
            escape_attr(&url)
        ));
    }

    tags.join("\n")
}

/// Generate JSON-LD structured data (WebApplication schema).
pub fn generate_json_ld(manifest: &Manifest, title: &str) -> String {
    let mut props = Vec::new();
    props.push("    \"@context\": \"https://schema.org\"".to_string());
    props.push("    \"@type\": \"WebApplication\"".to_string());
    props.push(format!("    \"name\": \"{}\"", escape_json(title)));

    if let Some(desc) = &manifest.seo.description {
        props.push(format!("    \"description\": \"{}\"", escape_json(desc)));
    }
    if let Some(canonical) = &manifest.seo.canonical {
        props.push(format!("    \"url\": \"{}\"", escape_json(canonical)));
    }
    if let Some(author) = &manifest.seo.author {
        props.push(format!(
            "    \"author\": {{\n      \"@type\": \"Person\",\n      \"name\": \"{}\"\n    }}",
            escape_json(author)
        ));
    }
    props.push("    \"applicationCategory\": \"WebApplication\"".to_string());
    props.push("    \"operatingSystem\": \"Any\"".to_string());

    format!(
        "  <script type=\"application/ld+json\">\n  {{\n{}\n  }}\n  </script>",
        props.join(",\n")
    )
}

/// Extract visible text content from render tree nodes for noscript fallback.
pub fn extract_text_content(nodes: &[RenderNode]) -> String {
    let mut texts = Vec::new();
    collect_text(nodes, &mut texts);
    let content = texts.join(" ");
    if content.len() > 1000 {
        format!("{}...", &content[..997])
    } else {
        content
    }
}

/// Escape HTML special characters for safe insertion into HTML content.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn collect_text(nodes: &[RenderNode], texts: &mut Vec<String>) {
    for node in nodes {
        if let Some(text_val) = node.props.get("__text") {
            match text_val {
                RenderValue::Str(s) if !s.is_empty() => {
                    texts.push(s.clone());
                }
                RenderValue::InterpolatedStr(parts) => {
                    let literal: String = parts
                        .iter()
                        .filter_map(|p| match p {
                            TextPart::Literal(s) => Some(s.as_str()),
                            TextPart::StateRef(_) => None,
                        })
                        .collect();
                    if !literal.is_empty() {
                        texts.push(literal);
                    }
                }
                _ => {}
            }
        }
        collect_text(&node.children, texts);
        if let Some(else_children) = &node.else_children {
            collect_text(else_children, texts);
        }
    }
}

fn page_title(app_title: &str, page_path: Option<&str>) -> String {
    match page_path {
        Some(path) if path != "/" => {
            let segment = path.trim_matches('/').rsplit('/').next().unwrap_or("");
            if segment.is_empty() {
                return app_title.to_string();
            }
            let name: String = segment
                .split('-')
                .map(capitalize_word)
                .collect::<Vec<_>>()
                .join(" ");
            format!("{} - {}", name, app_title)
        }
        _ => app_title.to_string(),
    }
}

fn capitalize_word(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn resolve_image_url(image: &str, canonical: Option<&str>) -> String {
    if image.starts_with("http://") || image.starts_with("https://") {
        return image.to_string();
    }
    if let Some(base) = canonical {
        if image.starts_with('/') {
            return format!("{}{}", base.trim_end_matches('/'), image);
        }
    }
    image.to_string()
}

fn resolve_page_url(canonical: &str, page_path: Option<&str>) -> String {
    match page_path {
        Some(path) if path != "/" => {
            format!("{}{}", canonical.trim_end_matches('/'), path)
        }
        _ => canonical.to_string(),
    }
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Compute the relative asset prefix for a route path.
/// "/" → ".", "/about" → "..", "/docs/intro" → "../.."
pub fn asset_prefix_for_route(path: &str) -> String {
    let depth = path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .count();
    if depth == 0 {
        ".".to_string()
    } else {
        (0..depth).map(|_| "..").collect::<Vec<_>>().join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{App, Build, Seo};
    use std::collections::HashMap;

    fn test_manifest(seo: Seo) -> Manifest {
        Manifest {
            app: App {
                name: "test-app".to_string(),
                version: "0.1.0".to_string(),
            },
            build: Build::default(),
            scripts: HashMap::new(),
            seo,
            dependencies: HashMap::new(),
            env: HashMap::new(),
        }
    }

    #[test]
    fn meta_tags_minimal() {
        let manifest = test_manifest(Seo::default());
        let tags = generate_meta_tags(&manifest, "My App", None);
        assert!(tags.contains("og:title"));
        assert!(tags.contains("My App"));
        assert!(tags.contains("twitter:card"));
    }

    #[test]
    fn meta_tags_full() {
        let manifest = test_manifest(Seo {
            description: Some("A test app".to_string()),
            image: Some("/og.png".to_string()),
            author: Some("Jane".to_string()),
            keywords: Some("naze,app".to_string()),
            canonical: Some("https://example.com".to_string()),
            twitter: Some("@jane".to_string()),
            locale: Some("en_US".to_string()),
        });
        let tags = generate_meta_tags(&manifest, "My App", None);
        assert!(tags.contains("name=\"description\""));
        assert!(tags.contains("A test app"));
        assert!(tags.contains("og:image"));
        assert!(tags.contains("https://example.com/og.png"));
        assert!(tags.contains("og:url"));
        assert!(tags.contains("og:locale"));
        assert!(tags.contains("twitter:site"));
        assert!(tags.contains("@jane"));
        assert!(tags.contains("rel=\"canonical\""));
    }

    #[test]
    fn meta_tags_page_specific() {
        let manifest = test_manifest(Seo {
            canonical: Some("https://example.com".to_string()),
            ..Seo::default()
        });
        let tags = generate_meta_tags(&manifest, "My App", Some("/about"));
        assert!(tags.contains("About - My App"));
        assert!(tags.contains("https://example.com/about"));
    }

    #[test]
    fn json_ld_output() {
        let manifest = test_manifest(Seo {
            description: Some("A test app".to_string()),
            canonical: Some("https://example.com".to_string()),
            author: Some("Jane".to_string()),
            ..Seo::default()
        });
        let ld = generate_json_ld(&manifest, "My App");
        assert!(ld.contains("application/ld+json"));
        assert!(ld.contains("\"WebApplication\""));
        assert!(ld.contains("\"My App\""));
        assert!(ld.contains("\"A test app\""));
        assert!(ld.contains("\"Jane\""));
    }

    #[test]
    fn text_extraction() {
        let nodes = vec![
            RenderNode {
                kind: "text".to_string(),
                props: {
                    let mut m = HashMap::new();
                    m.insert("__text".to_string(), RenderValue::Str("Hello".to_string()));
                    m
                },
                children: vec![],
                handlers: vec![],
                span: None,
                condition: None,
                else_children: None,
                each_binding: None,
            },
            RenderNode {
                kind: "column".to_string(),
                props: HashMap::new(),
                children: vec![RenderNode {
                    kind: "text".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert("__text".to_string(), RenderValue::Str("World".to_string()));
                        m
                    },
                    children: vec![],
                    handlers: vec![],
                    span: None,
                    condition: None,
                    else_children: None,
                    each_binding: None,
                }],
                handlers: vec![],
                span: None,
                condition: None,
                else_children: None,
                each_binding: None,
            },
        ];
        assert_eq!(extract_text_content(&nodes), "Hello World");
    }

    #[test]
    fn text_extraction_interpolated() {
        let nodes = vec![RenderNode {
            kind: "text".to_string(),
            props: {
                let mut m = HashMap::new();
                m.insert(
                    "__text".to_string(),
                    RenderValue::InterpolatedStr(vec![
                        TextPart::Literal("Count: ".to_string()),
                        TextPart::StateRef("count".to_string()),
                    ]),
                );
                m
            },
            children: vec![],
            handlers: vec![],
            span: None,
            condition: None,
            else_children: None,
            each_binding: None,
        }];
        assert_eq!(extract_text_content(&nodes), "Count: ");
    }

    #[test]
    fn asset_prefix() {
        assert_eq!(asset_prefix_for_route("/"), ".");
        assert_eq!(asset_prefix_for_route("/about"), "..");
        assert_eq!(asset_prefix_for_route("/docs/intro"), "../..");
        assert_eq!(asset_prefix_for_route("/a/b/c"), "../../..");
    }

    #[test]
    fn page_title_derivation() {
        assert_eq!(page_title("My App", None), "My App");
        assert_eq!(page_title("My App", Some("/")), "My App");
        assert_eq!(page_title("My App", Some("/about")), "About - My App");
        assert_eq!(
            page_title("My App", Some("/getting-started")),
            "Getting Started - My App"
        );
        assert_eq!(
            page_title("My App", Some("/docs/api-ref")),
            "Api Ref - My App"
        );
    }

    #[test]
    fn escape_html_chars() {
        assert_eq!(escape_html("<b>test</b>"), "&lt;b&gt;test&lt;/b&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }

    #[test]
    fn image_url_resolution() {
        assert_eq!(
            resolve_image_url("/og.png", Some("https://example.com")),
            "https://example.com/og.png"
        );
        assert_eq!(
            resolve_image_url("/og.png", Some("https://example.com/")),
            "https://example.com/og.png"
        );
        assert_eq!(
            resolve_image_url(
                "https://cdn.example.com/og.png",
                Some("https://example.com")
            ),
            "https://cdn.example.com/og.png"
        );
        assert_eq!(resolve_image_url("/og.png", None), "/og.png");
    }
}
