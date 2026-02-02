use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "naze.pest"]
struct NazeParser;

/// Parse a .naze source string into a list of AST nodes.
pub fn parse(source: &str, file: &str) -> Result<Vec<Node>, ParseError> {
    let pairs = NazeParser::parse(Rule::file, source).map_err(|e| {
        let (line, column) = match e.line_col {
            pest::error::LineColLocation::Pos((l, c)) => (l, c),
            pest::error::LineColLocation::Span((l, c), _) => (l, c),
        };
        // Extract just the semantic message from pest (e.g., "expected block")
        // by looking for the "= expected ..." line, or fall back to the variant description.
        let message = match &e.variant {
            pest::error::ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                let mut parts = Vec::new();
                if !positives.is_empty() {
                    let names: Vec<_> = positives.iter().map(|r| format!("{:?}", r)).collect();
                    parts.push(format!("expected {}", names.join(", ")));
                }
                if !negatives.is_empty() {
                    let names: Vec<_> = negatives.iter().map(|r| format!("{:?}", r)).collect();
                    parts.push(format!("unexpected {}", names.join(", ")));
                }
                if parts.is_empty() {
                    "unexpected input".to_string()
                } else {
                    parts.join("; ")
                }
            }
            pest::error::ErrorVariant::CustomError { message } => message.clone(),
        };
        ParseError {
            message,
            file: file.to_string(),
            line,
            column,
        }
    })?;

    // `pairs` contains a single `file` pair; iterate its inner pairs
    let file_pair = pairs.into_iter().next().unwrap();
    let mut nodes = Vec::new();
    for pair in file_pair.into_inner() {
        match pair.as_rule() {
            Rule::app_block => nodes.push(parse_app(pair, file)),
            Rule::component_def => nodes.push(parse_component(pair, file)),
            Rule::use_stmt => nodes.push(parse_use(pair, file)),
            Rule::element => nodes.push(parse_element(pair, file)),
            Rule::comment => nodes.push(Node::Comment(
                pair.as_str().trim_start_matches("--").trim().to_string(),
            )),
            Rule::EOI => {}
            _ => {}
        }
    }
    Ok(nodes)
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error in {}: {}", self.file, self.message)
    }
}

impl std::error::Error for ParseError {}

fn span_from_pair(pair: &pest::iterators::Pair<Rule>, file: &str) -> Span {
    let pest_span = pair.as_span();
    let (line, col) = pest_span.start_pos().line_col();
    Span {
        file: file.to_string(),
        line,
        col,
        offset: pest_span.start(),
        len: pest_span.end() - pest_span.start(),
    }
}

fn parse_app(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let title = parse_string_lit(inner.next().unwrap());
    let block = inner.next().unwrap();
    let children = parse_block(block, file);

    Node::App {
        title,
        children,
        span,
    }
}

fn parse_component(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut params = Vec::new();
    let mut children = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::param_list => {
                params = parse_param_list(p);
            }
            Rule::block => {
                children = parse_block(p, file);
            }
            _ => {}
        }
    }

    Node::Component {
        name,
        params,
        children,
        span,
    }
}

fn parse_param_list(pair: pest::iterators::Pair<Rule>) -> Vec<Param> {
    pair.into_inner().map(parse_param).collect()
}

fn parse_param(pair: pest::iterators::Pair<Rule>) -> Param {
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let ty = parse_type(inner.next().unwrap());

    let default = inner.next().map(|p| parse_value(p));

    Param { name, ty, default }
}

fn parse_type(pair: pest::iterators::Pair<Rule>) -> Type {
    match pair.as_str() {
        "text" => Type::Text,
        "number" => Type::Number,
        "bool" => Type::Bool,
        "color" => Type::Color,
        other => panic!("unknown type: {other}"),
    }
}

fn parse_use(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let path_pair = pair.into_inner().next().unwrap();
    let path: Vec<String> = path_pair.as_str().split('/').map(String::from).collect();
    Node::UseStmt { path, span }
}

fn parse_element(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut props = Vec::new();
    let mut children = Vec::new();
    let mut text_content: Option<String> = None;

    for p in inner {
        match p.as_rule() {
            Rule::string_lit => {
                text_content = Some(parse_string_lit(p));
            }
            Rule::inline_props => {
                props = parse_inline_props(p);
            }
            Rule::block => {
                children = parse_block(p, file);
            }
            _ => {}
        }
    }

    // If the element has a string literal (e.g., heading "Hello"), add it as a
    // __text prop so codegen can extract it.
    if let Some(text) = text_content {
        props.insert(
            0,
            Prop {
                key: "__text".to_string(),
                value: Value::Str(text),
            },
        );
    }

    Node::Element {
        name,
        props,
        children,
        span,
    }
}

fn parse_inline_props(pair: pest::iterators::Pair<Rule>) -> Vec<Prop> {
    pair.into_inner().map(parse_prop).collect()
}

fn parse_prop(pair: pest::iterators::Pair<Rule>) -> Prop {
    let mut inner = pair.into_inner();
    let key = inner.next().unwrap().as_str().to_string();
    let value = parse_value(inner.next().unwrap());
    Prop { key, value }
}

fn parse_value(pair: pest::iterators::Pair<Rule>) -> Value {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::string_lit => Value::Str(parse_string_lit(inner)),
        Rule::color_lit => {
            let hex = inner.as_str().trim_start_matches('#');
            let val = u32::from_str_radix(hex, 16).unwrap_or(0);
            // Normalize 3-digit hex (#abc -> #aabbcc)
            if hex.len() == 3 {
                let r = (val >> 8) & 0xF;
                let g = (val >> 4) & 0xF;
                let b = val & 0xF;
                Value::Color(((r << 4 | r) << 16) | ((g << 4 | g) << 8) | (b << 4 | b))
            } else {
                Value::Color(val)
            }
        }
        Rule::number_lit => {
            let mut num_inner = inner.into_inner();
            let raw = num_inner.next().unwrap().as_str();
            let num: f64 = raw.parse().unwrap_or(0.0);
            let unit = num_inner.next().map(|u| match u.as_str() {
                "px" => Unit::Px,
                "%" => Unit::Percent,
                "em" => Unit::Em,
                _ => Unit::Px,
            });
            Value::Num(num, unit)
        }
        Rule::bool_lit => Value::Bool(inner.as_str() == "true"),
        Rule::ref_path => {
            let parts: Vec<String> = inner.as_str().split('.').map(String::from).collect();
            Value::Ref(parts)
        }
        Rule::ident => {
            // Bare identifier used as a value (e.g., component prop reference)
            Value::Ref(vec![inner.as_str().to_string()])
        }
        _ => panic!("unexpected value rule: {:?}", inner.as_rule()),
    }
}

fn parse_string_lit(pair: pest::iterators::Pair<Rule>) -> String {
    let mut inner = pair.into_inner();
    match inner.next() {
        Some(s) => s.as_str().replace("\\\"", "\"").replace("\\\\", "\\"),
        None => String::new(),
    }
}

fn parse_block(pair: pest::iterators::Pair<Rule>, file: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::app_block => nodes.push(parse_app(p, file)),
            Rule::component_def => nodes.push(parse_component(p, file)),
            Rule::use_stmt => nodes.push(parse_use(p, file)),
            Rule::element => nodes.push(parse_element(p, file)),
            Rule::comment => nodes.push(Node::Comment(
                p.as_str().trim_start_matches("--").trim().to_string(),
            )),
            _ => {}
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_app() {
        let source = r#"app "Hello" {
  text "world"
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::App { title, children, .. } => {
                assert_eq!(title, "Hello");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected App node"),
        }
    }

    #[test]
    fn parse_comment() {
        let source = "-- this is a comment\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Comment(text) => assert_eq!(text, "this is a comment"),
            _ => panic!("expected Comment node"),
        }
    }

    #[test]
    fn parse_use_stmt() {
        let source = "use components/toolbar\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::UseStmt { path, .. } => {
                assert_eq!(path, &["components", "toolbar"]);
            }
            _ => panic!("expected UseStmt"),
        }
    }

    #[test]
    fn parse_element_with_props() {
        let source = "rect width: 100px, height: 50px, color: #ff0000\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element { name, props, .. } => {
                assert_eq!(name, "rect");
                assert_eq!(props.len(), 3);
                assert_eq!(props[0].key, "width");
                assert_eq!(props[2].key, "color");
                match &props[2].value {
                    Value::Color(c) => assert_eq!(*c, 0xff0000),
                    _ => panic!("expected Color"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_element_with_text_and_props() {
        let source = r#"heading "Hello" color: #333333
"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element { name, props, .. } => {
                assert_eq!(name, "heading");
                // __text prop + color prop
                assert_eq!(props.len(), 2);
                assert_eq!(props[0].key, "__text");
                match &props[0].value {
                    Value::Str(s) => assert_eq!(s, "Hello"),
                    _ => panic!("expected Str"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_component_def() {
        let source = r#"component box(color: color, size: number = 80px) {
  rect width: size, height: size, color: color
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Component {
                name, params, children, ..
            } => {
                assert_eq!(name, "box");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "color");
                assert!(params[0].default.is_none());
                assert_eq!(params[1].name, "size");
                assert!(params[1].default.is_some());
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected Component"),
        }
    }

    #[test]
    fn parse_nested_layout() {
        let source = r#"app "Layout" {
  column padding: 20px {
    heading "Title"
    row gap: 12px {
      rect width: 80px, height: 80px, color: #ff0000
      rect width: 80px, height: 80px, color: #00ff00
    }
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::App { children, .. } => {
                assert_eq!(children.len(), 1); // column
                match &children[0] {
                    Node::Element {
                        name, children: col_children, ..
                    } => {
                        assert_eq!(name, "column");
                        assert_eq!(col_children.len(), 2); // heading + row
                    }
                    _ => panic!("expected column Element"),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_color_shorthand() {
        let source = "rect color: #abc\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Element { props, .. } => match &props[0].value {
                Value::Color(c) => assert_eq!(*c, 0xaabbcc),
                _ => panic!("expected Color"),
            },
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_bool_prop() {
        let source = "toggle visible: true\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Element { props, .. } => match &props[0].value {
                Value::Bool(b) => assert!(*b),
                _ => panic!("expected Bool"),
            },
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_ref_value() {
        let source = "component box(color: color, size: number = 80px) {\n  rect width: size, color: theme.primary\n}\n";
        // This tests that `size` alone is an ident (not a ref — refs need dots).
        // But `theme.primary` is a ref.
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Component { children, .. } => match &children[0] {
                Node::Element { props, .. } => {
                    // `size` is a bare ident — currently parsed as ref_path won't match
                    // (ref_path requires dots). It'll fail if the grammar doesn't handle it.
                    // Let's verify theme.primary
                    let color_prop = &props[1];
                    assert_eq!(color_prop.key, "color");
                    match &color_prop.value {
                        Value::Ref(parts) => {
                            assert_eq!(parts, &["theme", "primary"]);
                        }
                        _ => panic!("expected Ref for theme.primary"),
                    }
                }
                _ => panic!("expected Element"),
            },
            _ => panic!("expected Component"),
        }
    }
}
