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
            Rule::let_stmt => nodes.push(parse_let(pair, file)),
            Rule::state_stmt => nodes.push(parse_state(pair, file)),
            Rule::on_handler => {} // on_handler at file scope is meaningless; skip
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

    let title = match parse_string_lit(inner.next().unwrap()) {
        Value::Str(s) => s,
        Value::InterpolatedStr(parts) => {
            // App title doesn't support interpolation; flatten to literal text
            parts
                .into_iter()
                .map(|p| match p {
                    StringPart::Literal(s) => s,
                    StringPart::Interpolation(segs) => format!("{{{}}}", segs.join(".")),
                })
                .collect()
        }
        _ => String::new(),
    };
    let block = inner.next().unwrap();
    let contents = parse_block(block, file);
    // App blocks ignore handlers (events only make sense on elements)

    Node::App {
        title,
        children: contents.nodes,
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
                let contents = parse_block(p, file);
                children = contents.nodes;
                // Component definitions ignore handlers
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

fn parse_let(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_value(inner.next().unwrap());
    Node::Let { name, value, span }
}

fn parse_state(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_value(inner.next().unwrap());
    Node::State { name, value, span }
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
    let mut handlers = Vec::new();
    let mut text_value: Option<Value> = None;

    for p in inner {
        match p.as_rule() {
            Rule::string_lit => {
                text_value = Some(parse_string_lit(p));
            }
            Rule::inline_props => {
                props = parse_inline_props(p);
            }
            Rule::block => {
                let contents = parse_block(p, file);
                children = contents.nodes;
                handlers = contents.handlers;
            }
            _ => {}
        }
    }

    // If the element has a string literal (e.g., heading "Hello"), add it as a
    // __text prop so codegen can extract it. Supports both plain and interpolated strings.
    if let Some(value) = text_value {
        props.insert(
            0,
            Prop {
                key: "__text".to_string(),
                value,
            },
        );
    }

    Node::Element {
        name,
        props,
        children,
        handlers,
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
        Rule::string_lit => parse_string_lit(inner),
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

fn parse_string_lit(pair: pest::iterators::Pair<Rule>) -> Value {
    let mut parts: Vec<StringPart> = Vec::new();
    let mut has_interpolation = false;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::string_chars => {
                let text = p.as_str().replace("\\\"", "\"").replace("\\\\", "\\");
                parts.push(StringPart::Literal(text));
            }
            Rule::interpolation => {
                has_interpolation = true;
                let inner = p.into_inner().next().unwrap();
                let segments: Vec<String> = match inner.as_rule() {
                    Rule::ref_path => inner.as_str().split('.').map(String::from).collect(),
                    Rule::ident => vec![inner.as_str().to_string()],
                    _ => vec![inner.as_str().to_string()],
                };
                parts.push(StringPart::Interpolation(segments));
            }
            _ => {}
        }
    }

    if has_interpolation {
        Value::InterpolatedStr(parts)
    } else {
        // Plain string — collapse literal parts into a single String
        let s: String = parts
            .into_iter()
            .map(|p| match p {
                StringPart::Literal(s) => s,
                StringPart::Interpolation(_) => unreachable!(),
            })
            .collect();
        Value::Str(s)
    }
}

struct BlockContents {
    nodes: Vec<Node>,
    handlers: Vec<EventHandler>,
}

fn parse_block(pair: pest::iterators::Pair<Rule>, file: &str) -> BlockContents {
    let mut nodes = Vec::new();
    let mut handlers = Vec::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::app_block => nodes.push(parse_app(p, file)),
            Rule::component_def => nodes.push(parse_component(p, file)),
            Rule::use_stmt => nodes.push(parse_use(p, file)),
            Rule::let_stmt => nodes.push(parse_let(p, file)),
            Rule::state_stmt => nodes.push(parse_state(p, file)),
            Rule::on_handler => handlers.push(parse_on_handler(p, file)),
            Rule::element => nodes.push(parse_element(p, file)),
            Rule::comment => nodes.push(Node::Comment(
                p.as_str().trim_start_matches("--").trim().to_string(),
            )),
            _ => {}
        }
    }
    BlockContents { nodes, handlers }
}

fn parse_on_handler(pair: pest::iterators::Pair<Rule>, file: &str) -> EventHandler {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let event = inner.next().unwrap().as_str().to_string();
    let action_pair = inner.next().unwrap();
    let action = parse_action(action_pair, file);
    EventHandler { event, action, span }
}

fn parse_action(pair: pest::iterators::Pair<Rule>, file: &str) -> Action {
    let span = span_from_pair(&pair, file);
    match pair.as_rule() {
        Rule::set_action => {
            let mut inner = pair.into_inner();
            let target = inner.next().unwrap().as_str().to_string();
            let expr = parse_expression(inner.next().unwrap());
            Action::Set { target, expr, span }
        }
        Rule::navigate_action => {
            let mut inner = pair.into_inner();
            let path = match parse_string_lit(inner.next().unwrap()) {
                Value::Str(s) => s,
                _ => String::new(),
            };
            Action::Navigate { path, span }
        }
        _ => panic!("unexpected action rule: {:?}", pair.as_rule()),
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Expression {
    let items: Vec<_> = pair.into_inner().collect();
    let mut atoms = Vec::new();
    let mut ops = Vec::new();

    for (i, item) in items.iter().enumerate() {
        if i % 2 == 0 {
            atoms.push(parse_expr_atom(item.clone()));
        } else {
            ops.push(parse_bin_op(item.as_str()));
        }
    }

    build_expr_tree(&atoms, &ops)
}

fn parse_expr_atom(pair: pest::iterators::Pair<Rule>) -> Expression {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
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
            Expression::Literal(Value::Num(num, unit))
        }
        Rule::bool_lit => Expression::Literal(Value::Bool(inner.as_str() == "true")),
        Rule::string_lit => Expression::Literal(parse_string_lit(inner)),
        Rule::ref_path => {
            let segments: Vec<String> = inner.as_str().split('.').map(String::from).collect();
            Expression::StateRef(segments.join("."))
        }
        Rule::ident => Expression::StateRef(inner.as_str().to_string()),
        Rule::expression => parse_expression(inner), // parenthesized
        _ => panic!("unexpected expr_atom rule: {:?}", inner.as_rule()),
    }
}

fn parse_bin_op(s: &str) -> BinOp {
    match s {
        "+" => BinOp::Add,
        "-" => BinOp::Sub,
        "*" => BinOp::Mul,
        "/" => BinOp::Div,
        "==" => BinOp::Eq,
        "!=" => BinOp::Neq,
        ">" => BinOp::Gt,
        "<" => BinOp::Lt,
        ">=" => BinOp::Gte,
        "<=" => BinOp::Lte,
        "&&" => BinOp::And,
        "||" => BinOp::Or,
        _ => panic!("unknown binary operator: {}", s),
    }
}

fn op_precedence(op: BinOp) -> u8 {
    match op {
        BinOp::Or => 1,
        BinOp::And => 2,
        BinOp::Eq | BinOp::Neq => 3,
        BinOp::Gt | BinOp::Lt | BinOp::Gte | BinOp::Lte => 4,
        BinOp::Add | BinOp::Sub => 5,
        BinOp::Mul | BinOp::Div => 6,
    }
}

/// Build an expression tree from a flat list of atoms and operators,
/// respecting operator precedence. Uses rightmost-lowest-precedence split
/// for left associativity.
fn build_expr_tree(atoms: &[Expression], ops: &[BinOp]) -> Expression {
    assert!(!atoms.is_empty());
    if atoms.len() == 1 {
        return atoms[0].clone();
    }

    // Find rightmost operator with lowest precedence
    let mut min_prec = u8::MAX;
    let mut split_at = 0;
    for (i, op) in ops.iter().enumerate() {
        let prec = op_precedence(*op);
        if prec <= min_prec {
            min_prec = prec;
            split_at = i;
        }
    }

    let left = build_expr_tree(&atoms[..=split_at], &ops[..split_at]);
    let right = build_expr_tree(&atoms[split_at + 1..], &ops[split_at + 1..]);
    Expression::BinOp {
        left: Box::new(left),
        op: ops[split_at],
        right: Box::new(right),
    }
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

    #[test]
    fn parse_let_stmt() {
        let source = "let label = \"Hello\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Let { name, value, .. } => {
                assert_eq!(name, "label");
                match value {
                    Value::Str(s) => assert_eq!(s, "Hello"),
                    _ => panic!("expected Str value"),
                }
            }
            _ => panic!("expected Let node"),
        }
    }

    #[test]
    fn parse_state_stmt() {
        let source = "state count = 0\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::State { name, value, .. } => {
                assert_eq!(name, "count");
                match value {
                    Value::Num(n, _) => assert_eq!(*n, 0.0),
                    _ => panic!("expected Num value"),
                }
            }
            _ => panic!("expected State node"),
        }
    }

    #[test]
    fn parse_string_interpolation() {
        let source = "text \"Count: {count}\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element { props, .. } => {
                assert_eq!(props[0].key, "__text");
                match &props[0].value {
                    Value::InterpolatedStr(parts) => {
                        assert_eq!(parts.len(), 2);
                        match &parts[0] {
                            StringPart::Literal(s) => assert_eq!(s, "Count: "),
                            _ => panic!("expected Literal"),
                        }
                        match &parts[1] {
                            StringPart::Interpolation(segs) => assert_eq!(segs, &["count"]),
                            _ => panic!("expected Interpolation"),
                        }
                    }
                    _ => panic!("expected InterpolatedStr"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_plain_string_not_interpolated() {
        let source = "text \"Hello world\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Element { props, .. } => match &props[0].value {
                Value::Str(s) => assert_eq!(s, "Hello world"),
                _ => panic!("plain string should be Value::Str, not InterpolatedStr"),
            },
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_interpolation_with_ref_path() {
        let source = "text \"Color: {theme.primary}\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Element { props, .. } => match &props[0].value {
                Value::InterpolatedStr(parts) => {
                    assert_eq!(parts.len(), 2);
                    match &parts[1] {
                        StringPart::Interpolation(segs) => {
                            assert_eq!(segs, &["theme", "primary"]);
                        }
                        _ => panic!("expected Interpolation"),
                    }
                }
                _ => panic!("expected InterpolatedStr"),
            },
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_on_handler() {
        let source = r#"app "Test" {
  state count = 0
  rect width: 100px, height: 50px, color: #ff0000 {
    on click: set count = count + 1
  }
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                assert_eq!(children.len(), 2); // state + rect
                match &children[1] {
                    Node::Element { name, handlers, .. } => {
                        assert_eq!(name, "rect");
                        assert_eq!(handlers.len(), 1);
                        assert_eq!(handlers[0].event, "click");
                        match &handlers[0].action {
                            Action::Set { target, expr, .. } => {
                                assert_eq!(target, "count");
                                // expr should be count + 1
                                match expr {
                                    Expression::BinOp { left, op, right } => {
                                        assert!(matches!(**left, Expression::StateRef(ref s) if s == "count"));
                                        assert_eq!(*op, BinOp::Add);
                                        assert!(matches!(**right, Expression::Literal(Value::Num(n, _)) if n == 1.0));
                                    }
                                    _ => panic!("expected BinOp expression"),
                                }
                            }
                            _ => panic!("expected Set action"),
                        }
                    }
                    _ => panic!("expected Element"),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_expression_precedence() {
        // Test that * binds tighter than +: a + b * c => a + (b * c)
        let source = r#"app "Test" {
  state x = 0
  rect width: 100px, height: 50px, color: #ff0000 {
    on click: set x = x + 2 * 3
  }
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => match &handlers[0].action {
                    Action::Set { expr, .. } => match expr {
                        Expression::BinOp { left, op, right } => {
                            // Top-level should be Add: x + (2 * 3)
                            assert_eq!(*op, BinOp::Add);
                            assert!(matches!(**left, Expression::StateRef(ref s) if s == "x"));
                            match &**right {
                                Expression::BinOp { op: inner_op, .. } => {
                                    assert_eq!(*inner_op, BinOp::Mul);
                                }
                                _ => panic!("expected nested BinOp for 2 * 3"),
                            }
                        }
                        _ => panic!("expected BinOp"),
                    },
                    _ => panic!("expected Set"),
                },
                _ => panic!("expected Element"),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_on_handler_literal() {
        // on click: set count = 0 (literal value, no binop)
        let source = r#"app "Test" {
  state count = 0
  rect width: 100px, height: 50px, color: #ff0000 {
    on click: set count = 0
  }
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    match &handlers[0].action {
                        Action::Set { target, expr, .. } => {
                            assert_eq!(target, "count");
                            assert!(matches!(expr, Expression::Literal(Value::Num(n, _)) if *n == 0.0));
                        }
                        _ => panic!("expected Set action"),
                    }
                }
                _ => panic!("expected Element"),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_text_with_comma_props() {
        // text "Hello", color: #ff0000 — comma between string and props
        let source = "text \"Hello\", color: #ff0000\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Element { name, props, .. } => {
                assert_eq!(name, "text");
                assert_eq!(props.len(), 2);
                assert_eq!(props[0].key, "__text");
                assert_eq!(props[1].key, "color");
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_state_in_app_block() {
        let source = r#"app "Counter" {
  state count = 0
  let label = "Count"
  heading "{label}: {count}"
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                assert_eq!(children.len(), 3); // state, let, heading
                assert!(matches!(&children[0], Node::State { name, .. } if name == "count"));
                assert!(matches!(&children[1], Node::Let { name, .. } if name == "label"));
            }
            _ => panic!("expected App"),
        }
    }
}
