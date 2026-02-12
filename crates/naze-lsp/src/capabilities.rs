//! LSP capability implementations for Naze.

use std::collections::HashMap;
use tower_lsp::lsp_types::*;

use naze_parser::ast::{Node, Span as AstSpan};
use naze_parser::parse;

// ─── Diagnostics ─────────────────────────────────────────────────────────────

/// Parse a document and return diagnostics.
/// Note: Full type-checking requires project resolution, so we only do basic
/// validation here. The typecheck module requires ResolvedProject.
pub fn get_diagnostics(content: &str, file_path: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Parse the document
    let parse_result = parse(content, file_path);

    match parse_result {
        Ok(_ast) => {
            // Parsing succeeded
            // Note: Full type-checking requires project resolution (resolving imports,
            // loading components, etc.). For now, we just validate parsing.
            // TODO: Add lightweight single-file validation for common errors.
        }
        Err(parse_error) => {
            // Parse error - convert to diagnostic
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: (parse_error.line.saturating_sub(1)) as u32,
                        character: (parse_error.column.saturating_sub(1)) as u32,
                    },
                    end: Position {
                        line: (parse_error.line.saturating_sub(1)) as u32,
                        character: parse_error.column as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: parse_error.message,
                source: Some("naze".to_string()),
                ..Default::default()
            });
        }
    }

    diagnostics
}

/// Convert a byte offset to line/column position.
fn offset_to_position(offset: usize, content: &str) -> Position {
    let mut line = 0u32;
    let mut col = 0u32;

    for (i, ch) in content.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    Position {
        line,
        character: col,
    }
}

// ─── Completions ─────────────────────────────────────────────────────────────

/// Get completion items at the given position.
pub fn get_completions(content: &str, position: Position) -> Vec<CompletionItem> {
    let offset = position_to_offset(position, content);
    let context = analyze_completion_context(content, offset);

    match context {
        CompletionContext::ElementName => element_completions(),
        CompletionContext::PropertyName(element) => property_completions(&element),
        CompletionContext::PropertyValue(element, prop) => value_completions(&element, &prop),
        CompletionContext::Keyword => keyword_completions(),
        CompletionContext::Unknown => vec![],
    }
}

#[derive(Debug)]
enum CompletionContext {
    ElementName,
    PropertyName(String),
    PropertyValue(String, String),
    Keyword,
    Unknown,
}

fn analyze_completion_context(content: &str, offset: usize) -> CompletionContext {
    // Get the text before cursor
    let before = &content[..offset.min(content.len())];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let current_line = &before[line_start..];

    // Check if we're at the start of a line (element name context)
    let trimmed = current_line.trim_start();
    if trimmed.is_empty() || !trimmed.contains(':') {
        // Could be element name or keyword
        if trimmed.is_empty() {
            return CompletionContext::Keyword;
        }
        // Check if typing an element name
        let first_word = trimmed.split_whitespace().next().unwrap_or("");
        if is_keyword(first_word) {
            return CompletionContext::Keyword;
        }
        return CompletionContext::ElementName;
    }

    // Check if we're after a colon (property value context)
    if let Some(colon_pos) = current_line.rfind(':') {
        let after_colon = &current_line[colon_pos + 1..];
        if after_colon.trim().is_empty() || !after_colon.contains(',') {
            // Find the property name before the colon
            let before_colon = &current_line[..colon_pos];
            if let Some(prop_name) = before_colon.split_whitespace().last() {
                // Find element name (first word on line)
                let element = trimmed.split_whitespace().next().unwrap_or("").to_string();
                return CompletionContext::PropertyValue(element, prop_name.to_string());
            }
        }
    }

    // After comma or space, could be property name
    if current_line.contains(' ') {
        let element = trimmed.split_whitespace().next().unwrap_or("").to_string();
        return CompletionContext::PropertyName(element);
    }

    CompletionContext::Unknown
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "app"
            | "component"
            | "use"
            | "state"
            | "let"
            | "if"
            | "else"
            | "each"
            | "in"
            | "on"
            | "slot"
            | "fill"
            | "page"
            | "link"
            | "theme"
            | "data"
    )
}

fn element_completions() -> Vec<CompletionItem> {
    let elements = [
        (
            "column",
            "Vertical layout container",
            "column gap: 16px {\n\t$0\n}",
        ),
        (
            "row",
            "Horizontal layout container",
            "row gap: 8px {\n\t$0\n}",
        ),
        ("stack", "Overlapping layout container", "stack {\n\t$0\n}"),
        (
            "grid",
            "Grid layout container",
            "grid columns: 3, gap: 8px {\n\t$0\n}",
        ),
        (
            "container",
            "Styled box container",
            "container padding: 16px, color: #f5f5f5 {\n\t$0\n}",
        ),
        (
            "rect",
            "Colored rectangle",
            "rect width: 100px, height: 50px, color: #3b82f6",
        ),
        ("text", "Body text", "text \"$0\""),
        ("heading", "Heading text", "heading \"$0\""),
        (
            "image",
            "Image element",
            "image src: \"$0\", width: 200px, height: 150px",
        ),
        ("spacer", "Invisible spacer", "spacer"),
        (
            "input",
            "Text input field",
            "input bind: $0, placeholder: \"Enter text\"",
        ),
        (
            "checkbox",
            "Checkbox input",
            "checkbox bind: $0, label: \"Option\"",
        ),
        (
            "radio",
            "Radio button",
            "radio bind: $0, value: \"option\", label: \"Option\"",
        ),
        ("select", "Dropdown select", "select bind: $0 {\n\t$1\n}"),
        (
            "scroll",
            "Scrollable container",
            "scroll height: 400px {\n\t$0\n}",
        ),
    ];

    elements
        .into_iter()
        .map(|(name, doc, snippet)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(doc.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn keyword_completions() -> Vec<CompletionItem> {
    let keywords = [
        ("app", "Application entry point", "app \"$1\" {\n\t$0\n}"),
        (
            "component",
            "Reusable component definition",
            "component $1() {\n\t$0\n}",
        ),
        ("state", "Reactive state variable", "state $1 = $0"),
        ("let", "Immutable binding", "let $1 = $0"),
        ("if", "Conditional rendering", "if $1 {\n\t$0\n}"),
        ("each", "List iteration", "each $1 in $2 {\n\t$0\n}"),
        ("page", "Route page definition", "page \"/$1\" {\n\t$0\n}"),
        (
            "theme",
            "Theme definition",
            "theme {\n\tcolors {\n\t\t$0\n\t}\n}",
        ),
        ("data", "Async data fetch", "data $1: fetch \"$0\""),
        ("use", "Import component", "use $0"),
    ];

    keywords
        .into_iter()
        .map(|(name, doc, snippet)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(doc.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn property_completions(element: &str) -> Vec<CompletionItem> {
    let props = match element {
        "column" | "row" | "stack" | "grid" | "container" => vec![
            ("padding", "Internal spacing"),
            ("gap", "Space between children"),
            ("width", "Element width"),
            ("height", "Element height"),
            ("color", "Background color"),
            ("radius", "Border radius"),
            ("align", "Cross-axis alignment"),
            ("justify", "Main-axis alignment"),
            ("wrap", "Enable wrapping"),
        ],
        "rect" => vec![
            ("width", "Rectangle width"),
            ("height", "Rectangle height"),
            ("color", "Fill color"),
            ("radius", "Corner radius"),
            ("border", "Border width"),
            ("border-color", "Border color"),
            ("opacity", "Opacity (0-1)"),
        ],
        "text" | "heading" => vec![
            ("color", "Text color"),
            ("font-size", "Font size"),
            ("opacity", "Opacity (0-1)"),
        ],
        "image" => vec![
            ("src", "Image source URL"),
            ("width", "Image width"),
            ("height", "Image height"),
            ("fit", "Object fit mode"),
            ("alt", "Alternative text"),
        ],
        "input" => vec![
            ("bind", "State variable to bind"),
            ("placeholder", "Placeholder text"),
            ("type", "Input type (text, email, password, number)"),
            ("validate", "Validation rules"),
        ],
        _ => vec![
            ("width", "Element width"),
            ("height", "Element height"),
            ("padding", "Internal spacing"),
            ("color", "Color"),
        ],
    };

    props
        .into_iter()
        .map(|(name, doc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(doc.to_string()),
            insert_text: Some(format!("{}: $0", name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

fn value_completions(element: &str, prop: &str) -> Vec<CompletionItem> {
    let values: Vec<(&str, &str)> = match prop {
        "align" | "justify" => vec![
            ("start", "Align to start"),
            ("center", "Center alignment"),
            ("end", "Align to end"),
            ("stretch", "Stretch to fill"),
            ("space-between", "Space between items"),
            ("space-around", "Space around items"),
            ("space-evenly", "Even spacing"),
        ],
        "fit" => vec![
            ("cover", "Cover container"),
            ("contain", "Fit within container"),
            ("fill", "Stretch to fill"),
        ],
        "type" if element == "input" => vec![
            ("text", "Plain text input"),
            ("email", "Email input"),
            ("password", "Password input"),
            ("number", "Numeric input"),
        ],
        "role" => vec![
            ("button", "Interactive button"),
            ("link", "Navigation link"),
            ("heading", "Section heading"),
            ("main", "Main content"),
            ("navigation", "Navigation area"),
            ("list", "List container"),
            ("listitem", "List item"),
        ],
        _ => vec![],
    };

    values
        .into_iter()
        .map(|(value, doc)| CompletionItem {
            label: value.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some(doc.to_string()),
            insert_text: Some(format!("\"{}\"", value)),
            ..Default::default()
        })
        .collect()
}

fn position_to_offset(position: Position, content: &str) -> usize {
    let mut offset = 0;
    let mut current_line = 0u32;

    for (i, ch) in content.char_indices() {
        if current_line == position.line {
            let col = (i - offset) as u32;
            if col >= position.character {
                return i;
            }
        }
        if ch == '\n' {
            current_line += 1;
            if current_line > position.line {
                return i;
            }
            offset = i + 1;
        }
    }

    content.len()
}

// ─── Hover ───────────────────────────────────────────────────────────────────

/// Get hover information at the given position.
pub fn get_hover(content: &str, position: Position) -> Option<Hover> {
    let offset = position_to_offset(position, content);
    let word = get_word_at_offset(content, offset)?;

    let documentation = get_element_documentation(&word)
        .or_else(|| get_keyword_documentation(&word))
        .or_else(|| get_property_documentation(&word))?;

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: documentation,
        }),
        range: None,
    })
}

fn get_word_at_offset(content: &str, offset: usize) -> Option<String> {
    let bytes = content.as_bytes();

    // Find word boundaries
    let mut start = offset;
    while start > 0 && is_word_char(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = offset;
    while end < bytes.len() && is_word_char(bytes[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    Some(content[start..end].to_string())
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn get_element_documentation(name: &str) -> Option<String> {
    let doc = match name {
        "column" => "**column** - Vertical layout container\n\nArranges children vertically with optional gap.\n\n```naze\ncolumn gap: 16px, padding: 20px {\n  text \"Item 1\"\n  text \"Item 2\"\n}\n```",
        "row" => "**row** - Horizontal layout container\n\nArranges children horizontally with optional gap.\n\n```naze\nrow gap: 8px {\n  rect width: 50px, height: 50px, color: #ff0000\n  rect width: 50px, height: 50px, color: #00ff00\n}\n```",
        "stack" => "**stack** - Overlapping layout container\n\nStacks children on top of each other.",
        "grid" => "**grid** - Grid layout container\n\nArranges children in a grid with configurable columns.\n\n```naze\ngrid columns: 3, gap: 8px {\n  ...\n}\n```",
        "rect" => "**rect** - Colored rectangle\n\nA simple colored rectangle element.\n\n```naze\nrect width: 100px, height: 50px, color: #3b82f6, radius: 8px\n```",
        "text" => "**text** - Body text\n\nDisplays body text with optional styling.\n\n```naze\ntext \"Hello, world!\", color: #333333\n```",
        "heading" => "**heading** - Heading text\n\nDisplays heading text (larger, bold by default).\n\n```naze\nheading \"Page Title\"\n```",
        "input" => "**input** - Text input field\n\nA text input with two-way state binding.\n\n```naze\ninput bind: username, placeholder: \"Enter name\", type: \"text\"\n```",
        _ => return None,
    };
    Some(doc.to_string())
}

fn get_keyword_documentation(name: &str) -> Option<String> {
    let doc = match name {
        "app" => "**app** - Application entry point\n\nDefines the root of a Naze application.\n\n```naze\napp \"My App\" {\n  ...\n}\n```",
        "component" => "**component** - Reusable component\n\nDefines a reusable UI component with optional parameters.\n\n```naze\ncomponent Button(label: text, color: color = #3b82f6) {\n  rect padding: 12px, color: color {\n    text \"{label}\"\n  }\n}\n```",
        "state" => "**state** - Reactive state variable\n\nDeclares a mutable state variable that triggers re-renders on change.\n\n```naze\nstate count = 0\nstate items = [\"Apple\", \"Banana\"]\n```",
        "let" => "**let** - Immutable binding\n\nDeclares a compile-time constant.\n\n```naze\nlet title = \"My App\"\n```",
        "if" => "**if** - Conditional rendering\n\nConditionally renders content based on an expression.\n\n```naze\nif count > 0 {\n  text \"Count: {count}\"\n} else {\n  text \"Empty\"\n}\n```",
        "each" => "**each** - List iteration\n\nIterates over a list, rendering content for each item.\n\n```naze\neach item in items {\n  text \"{item}\"\n}\n```",
        "data" => "**data** - Async data fetch\n\nFetches data from a URL and creates loading/error/data states.\n\n```naze\ndata users: fetch \"https://api.example.com/users\"\n\nif users.loading {\n  text \"Loading...\"\n}\n```",
        _ => return None,
    };
    Some(doc.to_string())
}

fn get_property_documentation(name: &str) -> Option<String> {
    let doc = match name {
        "padding" => "**padding** - Internal spacing\n\nAdds space inside the element's boundaries.\n\nValues: `8px`, `16px`, `1em`, `5%`",
        "gap" => "**gap** - Space between children\n\nSets the spacing between child elements.\n\nValues: `8px`, `16px`, `1em`",
        "color" => "**color** - Color value\n\nSets the fill or text color.\n\nValues: `#ff0000`, `#3b82f6`, `theme.colors.primary`",
        "radius" => "**radius** - Border radius\n\nRounds the corners of the element.\n\nValues: `4px`, `8px`, `50%`",
        "opacity" => "**opacity** - Transparency\n\nSets the element's opacity.\n\nValues: `0` (transparent) to `1` (opaque)",
        "bind" => "**bind** - Two-way state binding\n\nBinds an input to a state variable for two-way data flow.\n\n```naze\nstate username = \"\"\ninput bind: username\n```",
        "transition" => "**transition** - Property animation\n\nAnimates property changes over time.\n\nFormat: `\"property duration easing\"`\n\n```naze\ntransition: \"color 200ms ease\"\n```",
        _ => return None,
    };
    Some(doc.to_string())
}

// ─── Document Symbols ────────────────────────────────────────────────────────

/// Extract document symbols from parsed AST.
pub fn get_document_symbols(content: &str, file_path: &str) -> Vec<DocumentSymbol> {
    let nodes = match parse(content, file_path) {
        Ok(nodes) => nodes,
        Err(_) => return vec![],
    };

    nodes
        .iter()
        .filter_map(|node| node_to_symbol(node, content))
        .collect()
}

#[allow(deprecated)]
fn node_to_symbol(node: &Node, content: &str) -> Option<DocumentSymbol> {
    match node {
        Node::App {
            title,
            children,
            span,
        } => {
            let range = span_to_range(span, content);
            let name_range = Range {
                start: range.start,
                end: Position {
                    line: range.start.line,
                    character: range.start.character + 3 + title.len() as u32 + 2,
                },
            };

            let child_symbols = children
                .iter()
                .filter_map(|child| node_to_symbol(child, content))
                .collect();

            Some(DocumentSymbol {
                name: format!("app \"{}\"", title),
                detail: Some("Application entry point".to_string()),
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range,
                selection_range: name_range,
                children: Some(child_symbols),
            })
        }

        Node::Component {
            name,
            params,
            children,
            span,
        } => {
            let range = span_to_range(span, content);
            let param_str = params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");

            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some(format!("component({})", param_str)),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: Some(
                    children
                        .iter()
                        .filter_map(|child| node_to_symbol(child, content))
                        .collect(),
                ),
            })
        }

        Node::Page {
            path,
            children,
            span,
            ..
        } => {
            let range = span_to_range(span, content);

            Some(DocumentSymbol {
                name: format!("page \"{}\"", path),
                detail: Some("Route page".to_string()),
                kind: SymbolKind::CLASS,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: Some(
                    children
                        .iter()
                        .filter_map(|child| node_to_symbol(child, content))
                        .collect(),
                ),
            })
        }

        Node::State { name, span, .. } => {
            let range = span_to_range(span, content);

            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some("state".to_string()),
                kind: SymbolKind::VARIABLE,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            })
        }

        Node::Let { name, span, .. } => {
            let range = span_to_range(span, content);

            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some("let".to_string()),
                kind: SymbolKind::CONSTANT,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            })
        }

        Node::Data { name, span, .. } => {
            let range = span_to_range(span, content);

            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some("data".to_string()),
                kind: SymbolKind::VARIABLE,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            })
        }

        Node::Theme { span, .. } => {
            let range = span_to_range(span, content);

            Some(DocumentSymbol {
                name: "theme".to_string(),
                detail: Some("theme".to_string()),
                kind: SymbolKind::OBJECT,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: None,
            })
        }

        Node::Element {
            name,
            children,
            span,
            ..
        } => {
            let range = span_to_range(span, content);

            let child_symbols: Vec<DocumentSymbol> = children
                .iter()
                .filter_map(|child| node_to_symbol(child, content))
                .collect();

            Some(DocumentSymbol {
                name: name.clone(),
                detail: Some("element".to_string()),
                kind: SymbolKind::STRUCT,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if child_symbols.is_empty() {
                    None
                } else {
                    Some(child_symbols)
                },
            })
        }

        Node::If {
            then_children,
            else_children,
            span,
            ..
        } => {
            let range = span_to_range(span, content);

            let mut child_symbols: Vec<DocumentSymbol> = then_children
                .iter()
                .filter_map(|child| node_to_symbol(child, content))
                .collect();

            child_symbols.extend(
                else_children
                    .iter()
                    .filter_map(|child| node_to_symbol(child, content)),
            );

            Some(DocumentSymbol {
                name: "if".to_string(),
                detail: Some("conditional".to_string()),
                kind: SymbolKind::BOOLEAN,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: if child_symbols.is_empty() {
                    None
                } else {
                    Some(child_symbols)
                },
            })
        }

        Node::Each {
            variable,
            children,
            span,
            ..
        } => {
            let range = span_to_range(span, content);

            Some(DocumentSymbol {
                name: format!("each {}", variable),
                detail: Some("iteration".to_string()),
                kind: SymbolKind::ARRAY,
                tags: None,
                deprecated: None,
                range,
                selection_range: range,
                children: Some(
                    children
                        .iter()
                        .filter_map(|child| node_to_symbol(child, content))
                        .collect(),
                ),
            })
        }

        _ => None,
    }
}

fn span_to_range(span: &AstSpan, content: &str) -> Range {
    let start = offset_to_position(span.offset, content);
    let end = offset_to_position(span.offset + span.len, content);
    Range { start, end }
}

// ─── Go to Definition ────────────────────────────────────────────────────────

/// Find the definition of a symbol at the given position.
pub fn get_definition(
    content: &str,
    file_path: &str,
    position: Position,
    uri: &Url,
) -> Option<GotoDefinitionResponse> {
    let nodes = parse(content, file_path).ok()?;
    let offset = position_to_offset(position, content);
    let word = get_word_at_offset(content, offset)?;

    // Look for component definition
    for node in &nodes {
        if let Node::Component { name, span, .. } = node {
            if name == &word {
                let range = span_to_range(span, content);
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range,
                }));
            }
        }
    }

    // Look for state/let/data definition
    for node in &nodes {
        match node {
            Node::State { name, span, .. }
            | Node::Let { name, span, .. }
            | Node::Data { name, span, .. } => {
                if name == &word {
                    let range = span_to_range(span, content);
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: uri.clone(),
                        range,
                    }));
                }
            }
            Node::App { children, .. } | Node::Page { children, .. } => {
                if let Some(location) = find_definition_in_nodes(children, &word, content, uri) {
                    return Some(location);
                }
            }
            _ => {}
        }
    }

    None
}

fn find_definition_in_nodes(
    nodes: &[Node],
    name: &str,
    content: &str,
    uri: &Url,
) -> Option<GotoDefinitionResponse> {
    for node in nodes {
        match node {
            Node::State { name: n, span, .. }
            | Node::Let { name: n, span, .. }
            | Node::Data { name: n, span, .. } => {
                if n == name {
                    let range = span_to_range(span, content);
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: uri.clone(),
                        range,
                    }));
                }
            }
            Node::Element { children, .. } => {
                if let Some(loc) = find_definition_in_nodes(children, name, content, uri) {
                    return Some(loc);
                }
            }
            Node::If {
                then_children,
                else_children,
                ..
            } => {
                if let Some(loc) = find_definition_in_nodes(then_children, name, content, uri) {
                    return Some(loc);
                }
                if let Some(loc) = find_definition_in_nodes(else_children, name, content, uri) {
                    return Some(loc);
                }
            }
            Node::Each { children, .. } => {
                if let Some(loc) = find_definition_in_nodes(children, name, content, uri) {
                    return Some(loc);
                }
            }
            _ => {}
        }
    }
    None
}

// ─── Find References ─────────────────────────────────────────────────────────

/// Find all references to a symbol at the given position.
pub fn get_references(
    content: &str,
    _file_path: &str,
    position: Position,
    uri: &Url,
    include_declaration: bool,
) -> Vec<Location> {
    let offset = position_to_offset(position, content);
    let word = match get_word_at_offset(content, offset) {
        Some(w) => w,
        None => return vec![],
    };

    let mut locations = Vec::new();

    // Simple text-based search for references
    // This is a basic implementation; a full implementation would use AST traversal
    let mut search_offset = 0;
    while let Some(found) = content[search_offset..].find(&word) {
        let abs_offset = search_offset + found;

        // Check if it's a whole word match
        let before_ok = abs_offset == 0 || !is_word_char(content.as_bytes()[abs_offset - 1]);
        let after_ok = abs_offset + word.len() >= content.len()
            || !is_word_char(content.as_bytes()[abs_offset + word.len()]);

        if before_ok && after_ok {
            let start = offset_to_position(abs_offset, content);
            let end = offset_to_position(abs_offset + word.len(), content);

            // Check if this is a declaration
            let is_declaration = is_definition_at(content, abs_offset, &word);

            if include_declaration || !is_declaration {
                locations.push(Location {
                    uri: uri.clone(),
                    range: Range { start, end },
                });
            }
        }

        search_offset = abs_offset + 1;
    }

    locations
}

fn is_definition_at(content: &str, offset: usize, _word: &str) -> bool {
    // Look backwards for declaration keywords
    let before = &content[..offset];
    let trimmed = before.trim_end();

    trimmed.ends_with("state")
        || trimmed.ends_with("let")
        || trimmed.ends_with("component")
        || trimmed.ends_with("data")
        || trimmed.ends_with("each")
        || trimmed.ends_with(" in")
}

// ─── Code Actions ────────────────────────────────────────────────────────────

/// Get code actions at the given range.
pub fn get_code_actions(
    content: &str,
    _file_path: &str,
    range: Range,
    diagnostics: &[Diagnostic],
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // Check for diagnostic-based actions
    for diag in diagnostics {
        if let Some(action) = diagnostic_to_action(diag, content) {
            actions.push(CodeActionOrCommand::CodeAction(action));
        }
    }

    // Context-based actions
    let start_offset = position_to_offset(range.start, content);
    let word = get_word_at_offset(content, start_offset);

    if let Some(w) = word {
        // Wrap in container action
        if is_element_type(&w) {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Wrap in column".to_string(),
                kind: Some(CodeActionKind::REFACTOR),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: "Wrap in column".to_string(),
                    command: "naze.wrapInColumn".to_string(),
                    arguments: None,
                }),
                is_preferred: None,
                disabled: None,
                data: None,
            }));

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Wrap in row".to_string(),
                kind: Some(CodeActionKind::REFACTOR),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: "Wrap in row".to_string(),
                    command: "naze.wrapInRow".to_string(),
                    arguments: None,
                }),
                is_preferred: None,
                disabled: None,
                data: None,
            }));
        }

        // Extract component action
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Extract to component".to_string(),
            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
            diagnostics: None,
            edit: None,
            command: Some(Command {
                title: "Extract to component".to_string(),
                command: "naze.extractComponent".to_string(),
                arguments: None,
            }),
            is_preferred: None,
            disabled: None,
            data: None,
        }));
    }

    actions
}

fn diagnostic_to_action(diag: &Diagnostic, _content: &str) -> Option<CodeAction> {
    let msg = &diag.message;

    // Suggest adding missing closing brace
    if msg.contains("expected") && msg.contains("}") {
        return Some(CodeAction {
            title: "Add missing closing brace".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diag.clone()]),
            edit: None,
            command: Some(Command {
                title: "Add closing brace".to_string(),
                command: "naze.addClosingBrace".to_string(),
                arguments: None,
            }),
            is_preferred: Some(true),
            disabled: None,
            data: None,
        });
    }

    None
}

fn is_element_type(name: &str) -> bool {
    matches!(
        name,
        "column"
            | "row"
            | "stack"
            | "grid"
            | "container"
            | "rect"
            | "text"
            | "heading"
            | "image"
            | "input"
            | "checkbox"
            | "radio"
            | "select"
            | "scroll"
            | "spacer"
    )
}

// ─── Rename ──────────────────────────────────────────────────────────────────

/// Prepare rename at the given position.
pub fn prepare_rename(content: &str, position: Position) -> Option<PrepareRenameResponse> {
    let offset = position_to_offset(position, content);
    let word = get_word_at_offset(content, offset)?;

    // Only allow renaming identifiers (state, let, component names)
    // Not keywords or element types
    if is_keyword(&word) || is_element_type(&word) {
        return None;
    }

    let start = find_word_start(content, offset);
    let end = find_word_end(content, offset);

    let range = Range {
        start: offset_to_position(start, content),
        end: offset_to_position(end, content),
    };

    Some(PrepareRenameResponse::Range(range))
}

/// Perform rename of a symbol.
pub fn get_rename_edits(
    content: &str,
    position: Position,
    new_name: &str,
    uri: &Url,
) -> Option<WorkspaceEdit> {
    let offset = position_to_offset(position, content);
    let _word = get_word_at_offset(content, offset)?;

    // Find all references and replace
    let references = get_references(content, "", position, uri, true);

    let edits: Vec<TextEdit> = references
        .iter()
        .map(|loc| TextEdit {
            range: loc.range,
            new_text: new_name.to_string(),
        })
        .collect();

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), edits);

    Some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

fn find_word_start(content: &str, offset: usize) -> usize {
    let bytes = content.as_bytes();
    let mut start = offset;
    while start > 0 && is_word_char(bytes[start - 1]) {
        start -= 1;
    }
    start
}

fn find_word_end(content: &str, offset: usize) -> usize {
    let bytes = content.as_bytes();
    let mut end = offset;
    while end < bytes.len() && is_word_char(bytes[end]) {
        end += 1;
    }
    end
}
