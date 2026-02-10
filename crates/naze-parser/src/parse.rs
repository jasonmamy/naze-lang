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
            Rule::page_block => nodes.push(parse_page(pair, file)),
            Rule::component_def => nodes.push(parse_component(pair, file)),
            Rule::theme_def => nodes.push(parse_theme(pair, file)),
            Rule::use_stmt => nodes.push(parse_use(pair, file)),
            Rule::let_stmt => nodes.push(parse_let(pair, file)),
            Rule::state_stmt => nodes.push(parse_state(pair, file)),
            Rule::shared_state_stmt => nodes.push(parse_shared_state(pair, file)),
            Rule::computed_stmt => nodes.push(parse_computed(pair, file)),
            Rule::storage_stmt => nodes.push(parse_storage(pair, file)),
            Rule::data_stmt => nodes.push(parse_data(pair, file)),
            Rule::timer_stmt => nodes.push(parse_timer(pair, file)),
            Rule::param_stmt => nodes.push(parse_param_stmt(pair, file)),
            Rule::if_stmt => nodes.push(parse_if_stmt(pair, file)),
            Rule::each_stmt => nodes.push(parse_each_stmt(pair, file)),
            Rule::on_handler => {} // on_handler at file scope is meaningless; skip
            Rule::slot_stmt => nodes.push(parse_slot_stmt(pair, file)),
            Rule::fill_stmt => nodes.push(parse_fill_stmt(pair, file)),
            Rule::link_element => nodes.push(parse_link(pair, file)),
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

fn parse_page(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let path = match parse_string_lit(inner.next().unwrap()) {
        Value::Str(s) => s,
        Value::InterpolatedStr(parts) => {
            // Page path doesn't support interpolation; flatten to literal text
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

    Node::Page {
        path,
        children: contents.nodes,
        span,
    }
}

fn parse_link(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let text = parse_string_lit(inner.next().unwrap());
    let to = match parse_string_lit(inner.next().unwrap()) {
        Value::Str(s) => s,
        _ => String::new(),
    };

    let mut children = Vec::new();
    if let Some(block) = inner.next() {
        if block.as_rule() == Rule::block {
            let contents = parse_block(block, file);
            children = contents.nodes;
        }
    }

    Node::Link {
        text,
        to,
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

fn parse_theme(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut colors = Vec::new();
    let mut spacing = Vec::new();

    // theme_def contains a theme_block
    let theme_block = pair.into_inner().next().unwrap();

    for section in theme_block.into_inner() {
        if section.as_rule() != Rule::theme_section {
            continue;
        }

        let mut section_inner = section.into_inner();
        let section_name = section_inner.next().unwrap().as_str();

        for entry in section_inner {
            if entry.as_rule() != Rule::theme_entry {
                continue;
            }

            let mut entry_inner = entry.into_inner();
            let name = entry_inner.next().unwrap().as_str().to_string();
            let value = parse_value(entry_inner.next().unwrap());

            match section_name {
                "colors" => {
                    if let Value::Color(c) = value {
                        colors.push((name, c));
                    }
                }
                "spacing" => {
                    if let Value::Num(n, unit) = value {
                        spacing.push((name, n, unit));
                    }
                }
                _ => {}
            }
        }
    }

    Node::Theme {
        colors,
        spacing,
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
    Node::State { name, value, shared: false, span }
}

fn parse_shared_state(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_value(inner.next().unwrap());
    Node::State { name, value, shared: true, span }
}

fn parse_computed(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let expr = parse_expression(inner.next().unwrap());
    Node::Computed { name, expr, span }
}

fn parse_storage(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let st = inner.next().unwrap();
    let storage_type = match st.as_str() {
        "local" => StorageType::Local,
        "session" => StorageType::Session,
        _ => StorageType::Local,
    };
    let key_pair = inner.next().unwrap();
    let key = match parse_string_lit(key_pair) {
        Value::Str(s) => s,
        _ => String::new(),
    };
    let default = parse_value(inner.next().unwrap());
    Node::Storage {
        name,
        storage_type,
        key,
        default,
        span,
    }
}

fn parse_data(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let source_pair = inner.next().unwrap();
    let source = match source_pair.as_str() {
        "stream" => DataSource::Stream,
        _ => DataSource::Fetch,
    };
    let url_pair = inner.next().unwrap();
    // Extract string content (removing quotes)
    let url = match parse_string_lit(url_pair) {
        Value::Str(s) => s,
        _ => String::new(),
    };
    let mut config = DataConfig::default();
    // Parse optional data block with config properties
    if let Some(block) = inner.next() {
        if block.as_rule() == Rule::data_block {
            for prop_pair in block.into_inner() {
                if prop_pair.as_rule() == Rule::data_prop {
                    let mut prop_inner = prop_pair.into_inner();
                    let key = prop_inner.next().unwrap().as_str();
                    let val_pair = prop_inner.next().unwrap();
                    match key {
                        "method" => {
                            config.method = Some(val_pair.as_str().to_string());
                        }
                        "cache" => {
                            config.cache_ms = Some(parse_duration_ms(val_pair));
                        }
                        "retry" => {
                            if let Value::Num(n, _) = parse_value(val_pair) {
                                config.retry = Some(n as u32);
                            }
                        }
                        "trigger" => {
                            config.trigger = Some(val_pair.as_str().trim_matches('"').to_string());
                        }
                        "content-type" => {
                            if let Value::Str(s) = parse_value(val_pair) {
                                config.content_type = Some(s);
                            }
                        }
                        "body" => {
                            config.body = Some(parse_value(val_pair));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Node::Data { name, url, source, config, span }
}

fn parse_timer(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let kind_str = inner.next().unwrap().as_str();
    let kind = match kind_str {
        "after" => TimerKind::After,
        "every" => TimerKind::Every,
        _ => TimerKind::After,
    };
    let duration_pair = inner.next().unwrap();
    let duration_ms = parse_duration_ms(duration_pair);
    let action_pair = inner.next().unwrap();
    let action = parse_action(action_pair, file);
    Node::Timer { name, kind, duration_ms, action, span }
}

fn parse_param_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let ty_str = inner.next().unwrap().as_str();
    let ty = match ty_str {
        "text" => Type::Text,
        "number" => Type::Number,
        "bool" => Type::Bool,
        "color" => Type::Color,
        _ => Type::Text,
    };
    let default = parse_value(inner.next().unwrap());
    Node::Param { name, ty, default, span }
}

fn parse_duration_ms(pair: pest::iterators::Pair<Rule>) -> u64 {
    let text = pair.as_str();
    if let Some(n) = text.strip_suffix("ms") {
        n.parse::<f64>().unwrap_or(0.0) as u64
    } else if let Some(n) = text.strip_suffix("min") {
        (n.parse::<f64>().unwrap_or(0.0) * 60_000.0) as u64
    } else if let Some(n) = text.strip_suffix('h') {
        (n.parse::<f64>().unwrap_or(0.0) * 3_600_000.0) as u64
    } else if let Some(n) = text.strip_suffix('s') {
        (n.parse::<f64>().unwrap_or(0.0) * 1_000.0) as u64
    } else {
        // Try parsing as plain number (milliseconds)
        text.parse::<f64>().unwrap_or(0.0) as u64
    }
}

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let condition = parse_expression(inner.next().unwrap());
    let then_block = inner.next().unwrap();
    let then_contents = parse_block(then_block, file);

    let else_children = if let Some(next) = inner.next() {
        match next.as_rule() {
            Rule::if_stmt => {
                // else if — wrap the nested if in a single-element vec
                vec![parse_if_stmt(next, file)]
            }
            Rule::block => {
                parse_block(next, file).nodes
            }
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };

    Node::If {
        condition,
        then_children: then_contents.nodes,
        else_children,
        span,
    }
}

fn parse_each_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let variable = inner.next().unwrap().as_str().to_string();
    let iterable_pair = inner.next().unwrap();
    let iterable = match iterable_pair.as_rule() {
        Rule::ref_path => {
            let segments: Vec<String> = iterable_pair.as_str().split('.').map(String::from).collect();
            Expression::StateRef(segments.join("."))
        }
        Rule::ident => Expression::StateRef(iterable_pair.as_str().to_string()),
        _ => Expression::StateRef(iterable_pair.as_str().to_string()),
    };
    let body_block = inner.next().unwrap();
    let contents = parse_block(body_block, file);

    Node::Each {
        variable,
        iterable,
        children: contents.nodes,
        span,
    }
}

fn parse_slot_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut name = None;
    let mut default_children = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::string_lit => {
                name = match parse_string_lit(p) {
                    Value::Str(s) => Some(s),
                    _ => None,
                };
            }
            Rule::block => {
                let contents = parse_block(p, file);
                default_children = contents.nodes;
            }
            _ => {}
        }
    }

    Node::Slot {
        name,
        default_children,
        span,
    }
}

fn parse_fill_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let name = match parse_string_lit(name_pair) {
        Value::Str(s) => s,
        _ => String::new(),
    };

    let block_pair = inner.next().unwrap();
    let contents = parse_block(block_pair, file);

    Node::Fill {
        name,
        children: contents.nodes,
        span,
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

    // Convert bind: stateVar to Value::Bind for two-way binding
    if key == "bind" {
        if let Value::Ref(ref parts) = value {
            if parts.len() == 1 {
                return Prop {
                    key,
                    value: Value::Bind(parts[0].clone()),
                };
            }
        }
    }

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
        Rule::list_lit => {
            let items: Vec<Value> = inner.into_inner().map(parse_value).collect();
            Value::List(items)
        }
        Rule::object_lit => {
            let entries: Vec<(String, Value)> = inner.into_inner()
                .map(|entry| {
                    let mut entry_inner = entry.into_inner();
                    let key = entry_inner.next().unwrap().as_str().to_string();
                    let value = parse_value(entry_inner.next().unwrap());
                    (key, value)
                })
                .collect();
            Value::Object(entries)
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
            Rule::page_block => nodes.push(parse_page(p, file)),
            Rule::component_def => nodes.push(parse_component(p, file)),
            Rule::use_stmt => nodes.push(parse_use(p, file)),
            Rule::let_stmt => nodes.push(parse_let(p, file)),
            Rule::state_stmt => nodes.push(parse_state(p, file)),
            Rule::shared_state_stmt => nodes.push(parse_shared_state(p, file)),
            Rule::computed_stmt => nodes.push(parse_computed(p, file)),
            Rule::storage_stmt => nodes.push(parse_storage(p, file)),
            Rule::data_stmt => nodes.push(parse_data(p, file)),
            Rule::timer_stmt => nodes.push(parse_timer(p, file)),
            Rule::param_stmt => nodes.push(parse_param_stmt(p, file)),
            Rule::if_stmt => nodes.push(parse_if_stmt(p, file)),
            Rule::each_stmt => nodes.push(parse_each_stmt(p, file)),
            Rule::on_handler => handlers.push(parse_on_handler(p, file)),
            Rule::slot_stmt => nodes.push(parse_slot_stmt(p, file)),
            Rule::fill_stmt => nodes.push(parse_fill_stmt(p, file)),
            Rule::link_element => nodes.push(parse_link(p, file)),
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
    let next = inner.next().unwrap();
    let (modifier, action_pair) = if next.as_rule() == Rule::event_modifier {
        let mut mod_inner = next.into_inner();
        let kind_str = mod_inner.next().unwrap().as_str();
        let kind = match kind_str {
            "debounce" => ModifierKind::Debounce,
            "throttle" => ModifierKind::Throttle,
            _ => ModifierKind::Debounce,
        };
        let duration_ms = parse_duration_ms(mod_inner.next().unwrap());
        (Some(EventModifier { kind, duration_ms }), inner.next().unwrap())
    } else {
        (None, next)
    };
    let action = parse_action(action_pair, file);
    EventHandler { event, action, modifier, span }
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
        Rule::scroll_to_action => {
            let mut inner = pair.into_inner();
            let element_id = match parse_string_lit(inner.next().unwrap()) {
                Value::Str(s) => s,
                _ => String::new(),
            };
            Action::ScrollTo { element_id, span }
        }
        Rule::log_action => {
            let mut inner = pair.into_inner();
            let expr = parse_expression(inner.next().unwrap());
            Action::Log { expr, span }
        }
        Rule::trigger_action => {
            let mut inner = pair.into_inner();
            let data_name = inner.next().unwrap().as_str().to_string();
            Action::Trigger { data_name, span }
        }
        Rule::copy_action => {
            let mut inner = pair.into_inner();
            let expr = parse_expression(inner.next().unwrap());
            Action::Copy { expr, span }
        }
        Rule::send_action => {
            let mut inner = pair.into_inner();
            let stream_name = inner.next().unwrap().as_str().to_string();
            let expr = parse_expression(inner.next().unwrap());
            Action::Send { stream_name, expr, span }
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
    fn parse_log_action() {
        let source = r#"app "Test" {
  state count = 0
  rect {
    on click: log count
  }
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    assert_eq!(handlers[0].event, "click");
                    match &handlers[0].action {
                        Action::Log { expr, .. } => {
                            assert!(matches!(expr, Expression::StateRef(s) if s == "count"));
                        }
                        _ => panic!("expected Log action"),
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

    #[test]
    fn parse_computed_stmt() {
        let source = "computed total = count * price\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Computed { name, expr, .. } => {
                assert_eq!(name, "total");
                assert!(matches!(expr, Expression::BinOp { .. }));
            }
            _ => panic!("expected Computed node"),
        }
    }

    #[test]
    fn parse_computed_in_app_block() {
        let source = r#"app "Shop" {
  state count = 1
  state price = 10
  computed total = count * price
  text "{total}"
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                assert_eq!(children.len(), 4); // state, state, computed, text
                assert!(matches!(&children[2], Node::Computed { name, .. } if name == "total"));
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_storage_stmt() {
        let source = "storage theme: local \"theme-pref\" default: \"light\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Storage {
                name,
                storage_type,
                key,
                default,
                ..
            } => {
                assert_eq!(name, "theme");
                assert!(matches!(storage_type, StorageType::Local));
                assert_eq!(key, "theme-pref");
                assert!(matches!(default, Value::Str(s) if s == "light"));
            }
            _ => panic!("expected Storage node"),
        }
    }

    #[test]
    fn parse_storage_session() {
        let source = "storage token: session \"auth-token\" default: \"\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Storage {
                name,
                storage_type,
                ..
            } => {
                assert_eq!(name, "token");
                assert!(matches!(storage_type, StorageType::Session));
            }
            _ => panic!("expected Storage node"),
        }
    }

    #[test]
    fn parse_data_with_config() {
        let source = "data users: fetch \"/api/users\" {\n  method: post\n  cache: 5min\n  retry: 3\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Data { name, url, config, .. } => {
                assert_eq!(name, "users");
                assert_eq!(url, "/api/users");
                assert_eq!(config.method.as_deref(), Some("post"));
                assert_eq!(config.cache_ms, Some(300_000)); // 5min = 300000ms
                assert_eq!(config.retry, Some(3));
            }
            _ => panic!("expected Data node"),
        }
    }

    #[test]
    fn parse_timer_after() {
        let source = "timer dismiss: after 5s {\n  set visible = false\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Timer { name, kind, duration_ms, action, .. } => {
                assert_eq!(name, "dismiss");
                assert!(matches!(kind, TimerKind::After));
                assert_eq!(*duration_ms, 5000);
                assert!(matches!(action, Action::Set { .. }));
            }
            _ => panic!("expected Timer node"),
        }
    }

    #[test]
    fn parse_timer_every() {
        let source = "timer tick: every 1s {\n  set count = count + 1\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Timer { name, kind, duration_ms, .. } => {
                assert_eq!(name, "tick");
                assert!(matches!(kind, TimerKind::Every));
                assert_eq!(*duration_ms, 1000);
            }
            _ => panic!("expected Timer node"),
        }
    }

    #[test]
    fn parse_data_simple() {
        let source = "data items: fetch \"/api/items\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Data { name, config, .. } => {
                assert_eq!(name, "items");
                assert!(config.method.is_none());
                assert!(config.cache_ms.is_none());
            }
            _ => panic!("expected Data node"),
        }
    }

    #[test]
    fn parse_if_stmt() {
        let source = r#"app "Test" {
  state count = 0
  if count > 0 {
    text "positive"
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                // state + if
                assert!(children.len() >= 2);
                match &children[1] {
                    Node::If {
                        condition,
                        then_children,
                        else_children,
                        ..
                    } => {
                        assert!(matches!(condition, Expression::BinOp { .. }));
                        assert_eq!(then_children.len(), 1);
                        assert!(else_children.is_empty());
                    }
                    other => panic!("expected If, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_if_else() {
        let source = r#"app "Test" {
  state count = 0
  if count > 0 {
    text "positive"
  } else {
    text "zero"
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::If {
                        then_children,
                        else_children,
                        ..
                    } => {
                        assert_eq!(then_children.len(), 1);
                        assert_eq!(else_children.len(), 1);
                    }
                    other => panic!("expected If, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_if_elseif_else() {
        let source = r#"app "Test" {
  state count = 0
  if count > 10 {
    text "big"
  } else if count > 0 {
    text "small"
  } else {
    text "zero"
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::If {
                        then_children,
                        else_children,
                        ..
                    } => {
                        assert_eq!(then_children.len(), 1);
                        // else_children contains a nested If node
                        assert_eq!(else_children.len(), 1);
                        assert!(matches!(&else_children[0], Node::If { .. }));
                    }
                    other => panic!("expected If, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_each_stmt() {
        let source = r#"app "Test" {
  state items = ["Apple", "Banana"]
  each item in items {
    text "{item}"
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Each {
                        variable,
                        iterable,
                        children,
                        ..
                    } => {
                        assert_eq!(variable, "item");
                        assert!(matches!(iterable, Expression::StateRef(name) if name == "items"));
                        assert_eq!(children.len(), 1);
                    }
                    other => panic!("expected Each, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_slot_default() {
        let source = "component card(title: text) {\n  heading \"Title\"\n  slot\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Component { children, .. } => {
                assert_eq!(children.len(), 2); // heading + slot
                match &children[1] {
                    Node::Slot {
                        name,
                        default_children,
                        ..
                    } => {
                        assert!(name.is_none());
                        assert!(default_children.is_empty());
                    }
                    other => panic!("expected Slot, got {:?}", other),
                }
            }
            _ => panic!("expected Component"),
        }
    }

    #[test]
    fn parse_slot_named() {
        let source = "component page(title: text) {\n  slot \"header\"\n  slot\n  slot \"footer\"\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Component { children, .. } => {
                assert_eq!(children.len(), 3);
                match &children[0] {
                    Node::Slot { name, .. } => assert_eq!(name.as_deref(), Some("header")),
                    other => panic!("expected Slot, got {:?}", other),
                }
                match &children[1] {
                    Node::Slot { name, .. } => assert!(name.is_none()),
                    other => panic!("expected Slot, got {:?}", other),
                }
                match &children[2] {
                    Node::Slot { name, .. } => assert_eq!(name.as_deref(), Some("footer")),
                    other => panic!("expected Slot, got {:?}", other),
                }
            }
            _ => panic!("expected Component"),
        }
    }

    #[test]
    fn parse_slot_with_fallback() {
        let source = "component panel(title: text) {\n  slot {\n    text \"default content\"\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Component { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Slot {
                        name,
                        default_children,
                        ..
                    } => {
                        assert!(name.is_none());
                        assert_eq!(default_children.len(), 1);
                    }
                    other => panic!("expected Slot, got {:?}", other),
                }
            }
            _ => panic!("expected Component"),
        }
    }

    #[test]
    fn parse_fill() {
        let source = r#"app "Test" {
  fill "header" {
    heading "My Header"
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Fill {
                        name, children, ..
                    } => {
                        assert_eq!(name, "header");
                        assert_eq!(children.len(), 1);
                    }
                    other => panic!("expected Fill, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_list_literal() {
        let source = r#"app "Test" {
  state items = [1, 2, "hello"]
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[0] {
                    Node::State { name, value, .. } => {
                        assert_eq!(name, "items");
                        match value {
                            Value::List(items) => {
                                assert_eq!(items.len(), 3);
                                assert!(matches!(&items[0], Value::Num(1.0, None)));
                                assert!(matches!(&items[1], Value::Num(2.0, None)));
                                assert!(matches!(&items[2], Value::Str(s) if s == "hello"));
                            }
                            other => panic!("expected List, got {:?}", other),
                        }
                    }
                    other => panic!("expected State, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_theme_def() {
        let source = r#"theme {
  colors {
    primary: #2563eb
    danger: #dc2626
  }
  spacing {
    sm: 8px
    md: 16px
  }
}
"#;
        let nodes = parse(source, "theme.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Theme {
                colors, spacing, ..
            } => {
                assert_eq!(colors.len(), 2);
                assert_eq!(colors[0].0, "primary");
                assert_eq!(colors[0].1, 0x2563eb);
                assert_eq!(colors[1].0, "danger");
                assert_eq!(colors[1].1, 0xdc2626);

                assert_eq!(spacing.len(), 2);
                assert_eq!(spacing[0].0, "sm");
                assert_eq!(spacing[0].1, 8.0);
                assert!(matches!(spacing[0].2, Some(Unit::Px)));
                assert_eq!(spacing[1].0, "md");
                assert_eq!(spacing[1].1, 16.0);
            }
            other => panic!("expected Theme, got {:?}", other),
        }
    }

    #[test]
    fn parse_page_block() {
        let source = r#"page "/about" {
  heading "About Page"
  text "This is the about page."
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Page { path, children, .. } => {
                assert_eq!(path, "/about");
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Page, got {:?}", other),
        }
    }

    #[test]
    fn parse_link_element() {
        let source = r#"link "Go to About", to: "/about"
"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Link { text, to, children, .. } => {
                match text {
                    Value::Str(s) => assert_eq!(s, "Go to About"),
                    _ => panic!("expected Str for link text"),
                }
                assert_eq!(to, "/about");
                assert!(children.is_empty());
            }
            other => panic!("expected Link, got {:?}", other),
        }
    }

    #[test]
    fn parse_overlay_element() {
        let source = r#"app "Test" {
  state dialog-open = false
  overlay focus-trap: true, scroll-lock: true {
    rect width: 480px, height: 300px, color: #ffffff {
      text "Dialog content"
    }
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                // state + overlay
                assert_eq!(children.len(), 2);
                match &children[1] {
                    Node::Element {
                        name, props, children, ..
                    } => {
                        assert_eq!(name, "overlay");
                        assert_eq!(props.len(), 2);
                        assert_eq!(props[0].key, "focus-trap");
                        assert!(matches!(&props[0].value, Value::Bool(true)));
                        assert_eq!(props[1].key, "scroll-lock");
                        assert!(matches!(&props[1].value, Value::Bool(true)));
                        assert_eq!(children.len(), 1); // rect
                    }
                    other => panic!("expected overlay Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_overlay_with_anchor() {
        let source = r#"app "Test" {
  state menu-open = false
  rect id: "menu-btn" {
    on click: set menu-open = true
  }
  if menu-open {
    overlay anchor: "menu-btn", anchor-placement: "bottom" {
      text "Menu item 1"
      text "Menu item 2"
    }
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                // state, rect, if
                assert_eq!(children.len(), 3);
                match &children[2] {
                    Node::If { then_children, .. } => {
                        assert_eq!(then_children.len(), 1);
                        match &then_children[0] {
                            Node::Element { name, props, .. } => {
                                assert_eq!(name, "overlay");
                                assert_eq!(props[0].key, "anchor");
                                assert!(matches!(&props[0].value, Value::Str(s) if s == "menu-btn"));
                                assert_eq!(props[1].key, "anchor-placement");
                                assert!(matches!(&props[1].value, Value::Str(s) if s == "bottom"));
                            }
                            other => panic!("expected overlay Element, got {:?}", other),
                        }
                    }
                    other => panic!("expected If, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_click_outside_event() {
        let source = r#"app "Test" {
  state open = false
  overlay {
    on click-outside: set open = false
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { name, handlers, .. } => {
                        assert_eq!(name, "overlay");
                        assert_eq!(handlers.len(), 1);
                        assert_eq!(handlers[0].event, "click-outside");
                        match &handlers[0].action {
                            Action::Set { target, .. } => assert_eq!(target, "open"),
                            other => panic!("expected Set action, got {:?}", other),
                        }
                    }
                    other => panic!("expected overlay Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_context_menu_event() {
        let source = r#"app "Test" {
  state menu-open = false
  rect {
    on context-menu: set menu-open = true
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers[0].event, "context-menu");
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_pointer_move_event() {
        let source = r#"app "Test" {
  state x = 0
  rect {
    on pointer-move: set x = 1
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers[0].event, "pointer-move");
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_arrow_key_events() {
        let source = r#"app "Test" {
  state index = 0
  rect {
    on arrow-up: set index = index - 1
    on arrow-down: set index = index + 1
    on arrow-left: set index = 0
    on arrow-right: set index = 10
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers.len(), 4);
                        assert_eq!(handlers[0].event, "arrow-up");
                        assert_eq!(handlers[1].event, "arrow-down");
                        assert_eq!(handlers[2].event, "arrow-left");
                        assert_eq!(handlers[3].event, "arrow-right");
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_multiple_pages() {
        let source = r#"app "My App" {
  page "/" {
    heading "Home"
  }
  page "/about" {
    heading "About"
  }
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::App { children, .. } => {
                assert_eq!(children.len(), 2);
                match &children[0] {
                    Node::Page { path, .. } => assert_eq!(path, "/"),
                    other => panic!("expected Page, got {:?}", other),
                }
                match &children[1] {
                    Node::Page { path, .. } => assert_eq!(path, "/about"),
                    other => panic!("expected Page, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_trigger_action() {
        let source = "app \"Test\" {\n  data items: fetch \"/api/items\"\n  rect {\n    on click: trigger items\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers.len(), 1);
                        match &handlers[0].action {
                            Action::Trigger { data_name, .. } => assert_eq!(data_name, "items"),
                            other => panic!("expected Trigger action, got {:?}", other),
                        }
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_copy_action() {
        let source = "app \"Test\" {\n  state url = \"https://example.com\"\n  rect {\n    on click: copy url\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers.len(), 1);
                        match &handlers[0].action {
                            Action::Copy { expr, .. } => {
                                assert!(matches!(expr, Expression::StateRef(n) if n == "url"));
                            }
                            other => panic!("expected Copy action, got {:?}", other),
                        }
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_data_stream() {
        let source = "data chat: stream \"wss://api.example.com/chat\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Data { name, url, source, .. } => {
                assert_eq!(name, "chat");
                assert_eq!(url, "wss://api.example.com/chat");
                assert!(matches!(source, DataSource::Stream));
            }
            other => panic!("expected Data node, got {:?}", other),
        }
    }

    #[test]
    fn parse_send_action() {
        let source = "app \"Test\" {\n  state msg = \"hello\"\n  rect {\n    on click: send chat msg\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers.len(), 1);
                        match &handlers[0].action {
                            Action::Send { stream_name, expr, .. } => {
                                assert_eq!(stream_name, "chat");
                                assert!(matches!(expr, Expression::StateRef(n) if n == "msg"));
                            }
                            other => panic!("expected Send action, got {:?}", other),
                        }
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_param_stmt_test() {
        let source = "param page: number default: 1\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Param { name, ty, default, .. } => {
                assert_eq!(name, "page");
                assert!(matches!(ty, Type::Number));
                match default {
                    Value::Num(n, _) => assert_eq!(*n, 1.0),
                    other => panic!("expected Num default, got {:?}", other),
                }
            }
            other => panic!("expected Param node, got {:?}", other),
        }
    }

    #[test]
    fn parse_param_text_default() {
        let source = "param query: text default: \"hello\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Param { name, ty, default, .. } => {
                assert_eq!(name, "query");
                assert!(matches!(ty, Type::Text));
                match default {
                    Value::Str(s) => assert_eq!(s, "hello"),
                    other => panic!("expected Str default, got {:?}", other),
                }
            }
            other => panic!("expected Param node, got {:?}", other),
        }
    }

    #[test]
    fn parse_shared_state_stmt() {
        let source = "shared state auth = false\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::State { name, shared, .. } => {
                assert_eq!(name, "auth");
                assert!(*shared);
            }
            other => panic!("expected State node, got {:?}", other),
        }
    }

    #[test]
    fn parse_regular_state_not_shared() {
        let source = "state count = 0\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::State { name, shared, .. } => {
                assert_eq!(name, "count");
                assert!(!*shared);
            }
            other => panic!("expected State node, got {:?}", other),
        }
    }

    #[test]
    fn parse_debounce_modifier() {
        let source = "app \"Test\" {\n  state q = \"\"\n  rect {\n    on change debounce 300ms: set q = q\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers.len(), 1);
                        let m = handlers[0].modifier.as_ref().unwrap();
                        assert!(matches!(m.kind, ModifierKind::Debounce));
                        assert_eq!(m.duration_ms, 300);
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_throttle_modifier() {
        let source = "app \"Test\" {\n  state x = 0\n  rect {\n    on scroll throttle 100ms: set x = x + 1\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers.len(), 1);
                        let m = handlers[0].modifier.as_ref().unwrap();
                        assert!(matches!(m.kind, ModifierKind::Throttle));
                        assert_eq!(m.duration_ms, 100);
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_no_modifier() {
        let source = "app \"Test\" {\n  state c = 0\n  rect {\n    on click: set c = c + 1\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => {
                match &children[1] {
                    Node::Element { handlers, .. } => {
                        assert_eq!(handlers.len(), 1);
                        assert!(handlers[0].modifier.is_none());
                    }
                    other => panic!("expected Element, got {:?}", other),
                }
            }
            _ => panic!("expected App"),
        }
    }
}
