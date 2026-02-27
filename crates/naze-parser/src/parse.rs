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
            Rule::template_def => nodes.push(parse_template(pair, file)),
            Rule::theme_def => nodes.push(parse_theme(pair, file)),
            Rule::import_stmt => nodes.push(parse_import(pair, file)),
            Rule::use_stmt => nodes.push(parse_use(pair, file)),
            Rule::let_stmt => nodes.push(parse_let(pair, file)),
            Rule::state_stmt => nodes.push(parse_state(pair, file)),
            Rule::shared_state_stmt => nodes.push(parse_shared_state(pair, file)),
            Rule::computed_stmt => nodes.push(parse_computed(pair, file)),
            Rule::storage_stmt => nodes.push(parse_storage(pair, file)),
            Rule::server_data_stmt => nodes.push(parse_server_data(pair, file)),
            Rule::prompt_stmt => nodes.push(parse_prompt_stmt(pair, file)),
            Rule::data_stmt => nodes.push(parse_data(pair, file)),
            Rule::timer_stmt => nodes.push(parse_timer(pair, file)),
            Rule::param_stmt => nodes.push(parse_param_stmt(pair, file)),
            Rule::server_function_def => nodes.push(parse_server_function(pair, file)),
            Rule::function_def => nodes.push(parse_function_def(pair, file)),
            Rule::boundary_stmt => nodes.push(parse_boundary_stmt(pair, file)),
            Rule::meta_stmt => nodes.push(parse_meta_stmt(pair, file)),
            Rule::guard_def => nodes.push(parse_guard_def(pair, file)),
            Rule::model_def => nodes.push(parse_model_def(pair, file)),
            Rule::if_stmt => nodes.push(parse_if_stmt(pair, file)),
            Rule::match_stmt => nodes.push(parse_match_stmt(pair, file)),
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

    // Check for optional guard_ref and block
    let mut guard = None;
    let mut block_pair = None;
    for p in inner {
        match p.as_rule() {
            Rule::guard_ref => {
                guard = Some(p.into_inner().next().unwrap().as_str().to_string());
            }
            Rule::block => {
                block_pair = Some(p);
            }
            _ => {}
        }
    }
    let contents = parse_block(block_pair.unwrap(), file);

    // Extract dynamic param names from `:param` segments
    let params: Vec<String> = path
        .split('/')
        .filter_map(|seg| seg.strip_prefix(':').map(|s| s.to_string()))
        .collect();

    Node::Page {
        path,
        params,
        guard,
        children: contents.nodes,
        span,
    }
}

fn parse_guard_def(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let mut checks = Vec::new();
    for check_pair in inner {
        if check_pair.as_rule() == Rule::guard_check {
            let mut check_inner = check_pair.into_inner();
            let condition = parse_pipe_expression(check_inner.next().unwrap());
            let redirect = match parse_string_lit(check_inner.next().unwrap()) {
                Value::Str(s) => s,
                _ => String::new(),
            };
            checks.push(GuardCheckAst {
                condition,
                redirect,
            });
        }
    }

    Node::Guard { name, checks, span }
}

fn parse_model_def(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut fields = Vec::new();
    for field_pair in inner {
        if field_pair.as_rule() == Rule::model_field {
            let mut field_inner = field_pair.into_inner();
            let field_name = field_inner.next().unwrap().as_str().to_string();
            let field_type = field_inner.next().unwrap().as_str().to_string();
            let mut constraints = Vec::new();
            for constraint_pair in field_inner {
                if constraint_pair.as_rule() == Rule::field_constraint {
                    let text = constraint_pair.as_str().trim().to_string();
                    let mut inner_parts = constraint_pair.into_inner();
                    if let Some(default_val) = inner_parts.next() {
                        // "default X" — inner has the default value (ident/number/string)
                        constraints.push(format!("default:{}", default_val.as_str()));
                    } else {
                        // Simple keyword: "primary" or "unique"
                        constraints.push(text);
                    }
                }
            }
            fields.push(ModelField {
                name: field_name,
                field_type,
                constraints,
            });
        }
    }
    Node::Model { name, fields, span }
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

fn parse_template(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut slots = Vec::new();
    let mut children = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::template_slot_list => {
                for slot_ident in p.into_inner() {
                    slots.push(slot_ident.as_str().to_string());
                }
            }
            Rule::block => {
                let contents = parse_block(p, file);
                children = contents.nodes;
            }
            _ => {}
        }
    }

    Node::Template {
        name,
        slots,
        children,
        span,
    }
}

fn parse_theme(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut colors = Vec::new();
    let mut spacing = Vec::new();
    let mut name = None;
    let mut extends = None;

    // theme_def contains optional theme_name, optional theme_extends, then theme_block
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::theme_name => {
                name = Some(inner_pair.into_inner().next().unwrap().as_str().to_string());
            }
            Rule::theme_extends => {
                extends = Some(inner_pair.into_inner().next().unwrap().as_str().to_string());
            }
            Rule::theme_block => {
                for section in inner_pair.into_inner() {
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
                        let token_name = entry_inner.next().unwrap().as_str().to_string();
                        let value = parse_value(entry_inner.next().unwrap());

                        match section_name {
                            "colors" => {
                                if let Value::Color(c) = value {
                                    colors.push((token_name, c));
                                }
                            }
                            "spacing" => {
                                if let Value::Num(n, unit) = value {
                                    spacing.push((token_name, n, unit));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Node::Theme {
        name,
        extends,
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
    Node::State {
        name,
        value,
        shared: false,
        span,
    }
}

fn parse_shared_state(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_value(inner.next().unwrap());
    Node::State {
        name,
        value,
        shared: true,
        span,
    }
}

fn parse_computed(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let expr_pair = inner.next().unwrap();
    let expr = parse_pipe_expression(expr_pair);
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
        "js" => DataSource::JsCall,
        "device" => DataSource::Device,
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
                        "headers" => {
                            if let Value::Object(entries) = parse_value(val_pair) {
                                for (k, v) in entries {
                                    config.headers.push((k, v));
                                }
                            }
                        }
                        "watch" => {
                            if let Value::Bool(b) = parse_value(val_pair) {
                                config.watch = b;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Node::Data {
        name,
        url,
        source,
        config,
        span,
    }
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
    Node::Timer {
        name,
        kind,
        duration_ms,
        action,
        span,
    }
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
    Node::Param {
        name,
        ty,
        default,
        span,
    }
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

fn parse_boundary_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let body_block = inner.next().unwrap();
    let catch_block = inner.next().unwrap();

    Node::Boundary {
        children: parse_block(body_block, file).nodes,
        catch_children: parse_block(catch_block, file).nodes,
        span,
    }
}

fn parse_meta_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();

    let key = inner.next().unwrap().as_str().to_string();
    let value = parse_value(inner.next().unwrap());

    Node::Meta { key, value, span }
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
            Rule::block => parse_block(next, file).nodes,
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
    let iterable = parse_pipe_expression(iterable_pair);
    let body_block = inner.next().unwrap();
    let contents = parse_block(body_block, file);

    Node::Each {
        variable,
        iterable,
        children: contents.nodes,
        span,
    }
}

fn parse_function_def(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let param_list = inner.next().unwrap();
    let params: Vec<FuncParam> = param_list
        .into_inner()
        .map(|fp| {
            let mut fp_inner = fp.into_inner();
            let pname = fp_inner.next().unwrap().as_str().to_string();
            let ty = parse_type(fp_inner.next().unwrap());
            FuncParam { name: pname, ty }
        })
        .collect();
    let return_type = parse_type(inner.next().unwrap());
    let body_pair = inner.next().unwrap();
    let body = parse_pipe_expression(body_pair);
    Node::Function {
        name,
        params,
        return_type,
        body,
        span,
    }
}

fn parse_server_function(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    // Collect func_param pairs and server_body
    let mut params = Vec::new();
    let mut body = ServerBody {
        lets: vec![],
        result: Expression::Literal(Value::Num(0.0, None)),
    };
    for p in inner {
        match p.as_rule() {
            Rule::func_param => {
                let mut fp_inner = p.into_inner();
                let pname = fp_inner.next().unwrap().as_str().to_string();
                let ty = parse_type(fp_inner.next().unwrap());
                params.push(FuncParam { name: pname, ty });
            }
            Rule::server_body => {
                body = parse_server_body(p);
            }
            Rule::pipe_expression | Rule::expression => {
                // Fallback for backward compatibility
                body = ServerBody {
                    lets: vec![],
                    result: parse_pipe_expression(p),
                };
            }
            _ => {}
        }
    }
    Node::ServerFunction {
        name,
        params,
        body,
        span,
    }
}

fn parse_server_body(pair: pest::iterators::Pair<Rule>) -> ServerBody {
    let mut lets = Vec::new();
    let mut result = Expression::Literal(Value::Num(0.0, None));
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::server_let => {
                let mut inner = p.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let expr_pair = inner.next().unwrap();
                let server_expr = match expr_pair.as_rule() {
                    Rule::server_fetch_expr => {
                        let url = parse_string_lit(expr_pair.into_inner().next().unwrap());
                        match url {
                            Value::Str(s) => ServerExpr::Fetch(s),
                            Value::InterpolatedStr(parts) => {
                                // Flatten to string for now
                                let s = parts
                                    .into_iter()
                                    .map(|p| match p {
                                        StringPart::Literal(s) => s,
                                        StringPart::Interpolation(segs) => {
                                            format!("{{{}}}", segs.join("."))
                                        }
                                    })
                                    .collect();
                                ServerExpr::Fetch(s)
                            }
                            _ => ServerExpr::Fetch(String::new()),
                        }
                    }
                    Rule::server_sql_expr => {
                        let mut sql_inner = expr_pair.into_inner();
                        let query = match parse_string_lit(sql_inner.next().unwrap()) {
                            Value::Str(s) => s,
                            _ => String::new(),
                        };
                        let params = sql_inner
                            .next()
                            .map(|params_pair| {
                                params_pair
                                    .into_inner()
                                    .map(parse_pipe_expression)
                                    .collect()
                            })
                            .unwrap_or_default();
                        ServerExpr::Sql { query, params }
                    }
                    Rule::server_find_expr => parse_server_find(expr_pair),
                    Rule::server_insert_expr => parse_server_insert(expr_pair),
                    Rule::server_update_expr => parse_server_update(expr_pair),
                    Rule::server_delete_expr => parse_server_delete(expr_pair),
                    _ => ServerExpr::Expr(parse_pipe_expression(expr_pair)),
                };
                lets.push((name, server_expr));
            }
            Rule::server_find_expr => {
                // find as final result: wrap in a let + use that as result
                lets.push(("__result".to_string(), parse_server_find(p)));
                result = Expression::StateRef("__result".to_string());
            }
            Rule::server_insert_expr => {
                lets.push(("__result".to_string(), parse_server_insert(p)));
                result = Expression::StateRef("__result".to_string());
            }
            Rule::server_update_expr => {
                lets.push(("__result".to_string(), parse_server_update(p)));
                result = Expression::StateRef("__result".to_string());
            }
            Rule::server_delete_expr => {
                lets.push(("__result".to_string(), parse_server_delete(p)));
                result = Expression::StateRef("__result".to_string());
            }
            Rule::pipe_expression | Rule::expression => {
                result = parse_pipe_expression(p);
            }
            _ => {}
        }
    }
    ServerBody { lets, result }
}

fn parse_server_find(pair: pest::iterators::Pair<Rule>) -> ServerExpr {
    let mut inner = pair.into_inner();
    let model = inner.next().unwrap().as_str().to_string();
    let mut conditions = Vec::new();
    let mut order = None;
    let mut limit = None;
    for clause in inner {
        match clause.as_rule() {
            Rule::query_where => {
                conditions = parse_query_conditions(clause);
            }
            Rule::query_order => {
                let mut order_inner = clause.into_inner();
                let field = order_inner.next().unwrap().as_str().to_string();
                let ascending = order_inner
                    .next()
                    .map(|p| p.as_str() == "asc")
                    .unwrap_or(true);
                order = Some((field, ascending));
            }
            Rule::query_limit => {
                let limit_expr = clause.into_inner().next().unwrap();
                limit = Some(parse_pipe_expression(limit_expr));
            }
            _ => {}
        }
    }
    ServerExpr::Find {
        model,
        conditions,
        order,
        limit,
    }
}

fn parse_server_insert(pair: pest::iterators::Pair<Rule>) -> ServerExpr {
    let mut inner = pair.into_inner();
    let model = inner.next().unwrap().as_str().to_string();
    let obj = inner.next().unwrap(); // object_lit
    let fields: Vec<(String, Value)> = obj
        .into_inner()
        .map(|entry| {
            let mut entry_inner = entry.into_inner();
            let key = entry_inner.next().unwrap().as_str().to_string();
            let value = parse_value(entry_inner.next().unwrap());
            (key, value)
        })
        .collect();
    ServerExpr::Insert { model, fields }
}

fn parse_server_update(pair: pest::iterators::Pair<Rule>) -> ServerExpr {
    let mut inner = pair.into_inner();
    let model = inner.next().unwrap().as_str().to_string();
    let obj = inner.next().unwrap(); // object_lit for "set" fields
    let set_fields: Vec<(String, Value)> = obj
        .into_inner()
        .map(|entry| {
            let mut entry_inner = entry.into_inner();
            let key = entry_inner.next().unwrap().as_str().to_string();
            let value = parse_value(entry_inner.next().unwrap());
            (key, value)
        })
        .collect();
    let conditions = inner
        .next()
        .map(|where_pair| parse_query_conditions(where_pair))
        .unwrap_or_default();
    ServerExpr::Update {
        model,
        set_fields,
        conditions,
    }
}

fn parse_server_delete(pair: pest::iterators::Pair<Rule>) -> ServerExpr {
    let mut inner = pair.into_inner();
    let model = inner.next().unwrap().as_str().to_string();
    let conditions = inner
        .next()
        .map(|where_pair| parse_query_conditions(where_pair))
        .unwrap_or_default();
    ServerExpr::Delete { model, conditions }
}

fn parse_query_conditions(where_pair: pest::iterators::Pair<Rule>) -> Vec<QueryCondition> {
    where_pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::query_condition)
        .map(|cond_pair| {
            let mut cond_inner = cond_pair.into_inner();
            let field = cond_inner.next().unwrap().as_str().to_string();
            let op = cond_inner.next().unwrap().as_str().to_string();
            let value = parse_pipe_expression(cond_inner.next().unwrap());
            QueryCondition { field, op, value }
        })
        .collect()
}

fn parse_server_data(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let func_name = inner.next().unwrap().as_str().to_string();
    let mut args = Vec::new();
    for p in inner {
        if p.as_rule() == Rule::call_args {
            args = p.into_inner().map(parse_expression).collect();
        }
    }
    Node::ServerData {
        name,
        func_name,
        args,
        span,
    }
}

fn parse_prompt_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let provider = inner.next().unwrap().as_str().to_string();
    let mut props = Vec::new();
    if let Some(block) = inner.next() {
        if block.as_rule() == Rule::prompt_block {
            for prop_pair in block.into_inner() {
                if prop_pair.as_rule() == Rule::prompt_prop {
                    let mut prop_inner = prop_pair.into_inner();
                    let key = prop_inner.next().unwrap().as_str().to_string();
                    let val = parse_value(prop_inner.next().unwrap());
                    props.push((key, val));
                }
            }
        }
    }
    Node::Prompt {
        name,
        provider,
        props,
        span,
    }
}

fn parse_match_stmt(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let subject = parse_expression(inner.next().unwrap());
    let mut arms = Vec::new();
    for arm_pair in inner {
        if arm_pair.as_rule() == Rule::match_arm {
            let mut arm_inner = arm_pair.into_inner();
            let pattern_pair = arm_inner.next().unwrap();
            let pattern = parse_match_pattern(pattern_pair);
            let body_pair = arm_inner.next().unwrap();
            let children = match body_pair.as_rule() {
                Rule::match_arm_body => {
                    let contents = parse_block(body_pair, file);
                    contents.nodes
                }
                _ => {
                    // Single element (element rule)
                    vec![parse_element(body_pair, file)]
                }
            };
            arms.push(MatchArm { pattern, children });
        }
    }
    Node::Match {
        subject,
        arms,
        span,
    }
}

fn parse_match_pattern(pair: pest::iterators::Pair<Rule>) -> MatchPattern {
    // "_" is a literal in the grammar, not a rule — no inner child
    match pair.into_inner().next() {
        None => MatchPattern::Wildcard,
        Some(inner) => match inner.as_rule() {
            Rule::string_lit => match parse_string_lit(inner) {
                Value::Str(s) => MatchPattern::StringLit(s),
                _ => MatchPattern::StringLit(String::new()),
            },
            Rule::number_lit => {
                let mut num_inner = inner.into_inner();
                let raw = num_inner.next().unwrap().as_str();
                MatchPattern::NumberLit(raw.parse().unwrap_or(0.0))
            }
            Rule::bool_lit => MatchPattern::BoolLit(inner.as_str() == "true"),
            Rule::ident => {
                let s = inner.as_str();
                if s == "_" {
                    MatchPattern::Wildcard
                } else {
                    MatchPattern::Ident(s.to_string())
                }
            }
            _ => MatchPattern::Wildcard,
        },
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

fn parse_import(pair: pest::iterators::Pair<Rule>, file: &str) -> Node {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let source = match parse_string_lit(inner.next().unwrap()) {
        Value::Str(s) => s,
        _ => String::new(),
    };
    Node::Import { name, source, span }
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
            let entries: Vec<(String, Value)> = inner
                .into_inner()
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
            Rule::template_def => nodes.push(parse_template(p, file)),
            Rule::import_stmt => nodes.push(parse_import(p, file)),
            Rule::use_stmt => nodes.push(parse_use(p, file)),
            Rule::let_stmt => nodes.push(parse_let(p, file)),
            Rule::state_stmt => nodes.push(parse_state(p, file)),
            Rule::shared_state_stmt => nodes.push(parse_shared_state(p, file)),
            Rule::computed_stmt => nodes.push(parse_computed(p, file)),
            Rule::storage_stmt => nodes.push(parse_storage(p, file)),
            Rule::server_data_stmt => nodes.push(parse_server_data(p, file)),
            Rule::prompt_stmt => nodes.push(parse_prompt_stmt(p, file)),
            Rule::data_stmt => nodes.push(parse_data(p, file)),
            Rule::timer_stmt => nodes.push(parse_timer(p, file)),
            Rule::param_stmt => nodes.push(parse_param_stmt(p, file)),
            Rule::server_function_def => nodes.push(parse_server_function(p, file)),
            Rule::function_def => nodes.push(parse_function_def(p, file)),
            Rule::boundary_stmt => nodes.push(parse_boundary_stmt(p, file)),
            Rule::meta_stmt => nodes.push(parse_meta_stmt(p, file)),
            Rule::guard_def => nodes.push(parse_guard_def(p, file)),
            Rule::model_def => nodes.push(parse_model_def(p, file)),
            Rule::if_stmt => nodes.push(parse_if_stmt(p, file)),
            Rule::match_stmt => nodes.push(parse_match_stmt(p, file)),
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
        (
            Some(EventModifier { kind, duration_ms }),
            inner.next().unwrap(),
        )
    } else {
        (None, next)
    };
    let actions = parse_action_list(action_pair, file);
    EventHandler {
        event,
        actions,
        modifier,
        span,
    }
}

fn parse_action_list(pair: pest::iterators::Pair<Rule>, file: &str) -> Vec<Action> {
    pair.into_inner().map(|p| parse_action(p, file)).collect()
}

fn parse_action(pair: pest::iterators::Pair<Rule>, file: &str) -> Action {
    let span = span_from_pair(&pair, file);
    match pair.as_rule() {
        Rule::set_action => {
            let mut inner = pair.into_inner();
            let target = inner.next().unwrap().as_str().to_string();
            let expr = parse_pipe_expression(inner.next().unwrap());
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
            Action::Send {
                stream_name,
                expr,
                span,
            }
        }
        Rule::js_action => {
            let mut inner = pair.into_inner();
            let function_name = match parse_string_lit(inner.next().unwrap()) {
                Value::Str(s) => s,
                _ => String::new(),
            };
            let mut args = Vec::new();
            let mut target = None;
            for p in inner {
                match p.as_rule() {
                    Rule::call_args => {
                        args = p.into_inner().map(parse_expression).collect();
                    }
                    Rule::ident => {
                        target = Some(p.as_str().to_string());
                    }
                    _ => {}
                }
            }
            Action::JsCall {
                function_name,
                args,
                target,
                span,
            }
        }
        Rule::notify_action => {
            let mut inner = pair.into_inner();
            let title = match parse_string_lit(inner.next().unwrap()) {
                Value::Str(s) => s,
                _ => String::new(),
            };
            let mut body = None;
            let mut icon = None;
            if let Some(block) = inner.next() {
                if block.as_rule() == Rule::notify_block {
                    for prop_pair in block.into_inner() {
                        if prop_pair.as_rule() == Rule::notify_prop {
                            let mut prop_inner = prop_pair.into_inner();
                            let key = prop_inner.next().unwrap().as_str();
                            let val_pair = prop_inner.next().unwrap();
                            match key {
                                "body" => {
                                    if let Value::Str(s) = parse_value(val_pair) {
                                        body = Some(s);
                                    }
                                }
                                "icon" => {
                                    if let Value::Str(s) = parse_value(val_pair) {
                                        icon = Some(s);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Action::Notify {
                title,
                body,
                icon,
                span,
            }
        }
        Rule::emit_action => {
            let mut inner = pair.into_inner();
            let event_name = inner.next().unwrap().as_str().to_string();
            Action::Emit { event_name, span }
        }
        Rule::set_theme_action => {
            let mut inner = pair.into_inner();
            let theme_name = match parse_string_lit(inner.next().unwrap()) {
                Value::Str(s) => s,
                _ => String::new(),
            };
            Action::SetTheme { theme_name, span }
        }
        Rule::append_action => {
            let mut inner = pair.into_inner();
            let item = parse_expression(inner.next().unwrap());
            let target = inner.next().unwrap().as_str().to_string();
            Action::Append { item, target, span }
        }
        Rule::remove_action => {
            let mut inner = pair.into_inner();
            let index = parse_expression(inner.next().unwrap());
            let target = inner.next().unwrap().as_str().to_string();
            Action::Remove { index, target, span }
        }
        Rule::set_index_action => {
            let mut inner = pair.into_inner();
            let target = inner.next().unwrap().as_str().to_string();
            let index = parse_expression(inner.next().unwrap());
            let expr = parse_pipe_expression(inner.next().unwrap());
            Action::SetIndex {
                target,
                index,
                expr,
                span,
            }
        }
        Rule::conditional_action => {
            let mut inner = pair.into_inner();
            let condition = parse_expression(inner.next().unwrap());
            let then_actions = parse_action_list(inner.next().unwrap(), file);
            let else_actions = match inner.next() {
                Some(else_pair) => parse_action_list(else_pair, file),
                None => vec![],
            };
            Action::Conditional {
                condition,
                then_actions,
                else_actions,
                span,
            }
        }
        _ => panic!("unexpected action rule: {:?}", pair.as_rule()),
    }
}

fn parse_pipe_expression(pair: pest::iterators::Pair<Rule>) -> Expression {
    match pair.as_rule() {
        Rule::pipe_expression => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            let source = parse_expression(first);
            let stages: Vec<PipelineStage> = inner.map(parse_pipe_stage).collect();
            if stages.is_empty() {
                source
            } else {
                Expression::Pipeline {
                    source: Box::new(source),
                    stages,
                }
            }
        }
        Rule::expression => parse_expression(pair),
        Rule::ref_path => {
            let segments: Vec<String> = pair.as_str().split('.').map(String::from).collect();
            Expression::StateRef(segments.join("."))
        }
        Rule::ident => Expression::StateRef(pair.as_str().to_string()),
        _ => Expression::StateRef(pair.as_str().to_string()),
    }
}

fn parse_pipe_stage(pair: pest::iterators::Pair<Rule>) -> PipelineStage {
    let mut inner = pair.into_inner();
    let fn_pair = inner.next().unwrap();
    let function = match fn_pair.as_str() {
        "filter" => PipelineFn::Filter,
        "map" => PipelineFn::Map,
        "sort-by" => PipelineFn::SortBy,
        "take" => PipelineFn::Take,
        "sum" => PipelineFn::Sum,
        "count" => PipelineFn::Count,
        "reduce" => PipelineFn::Reduce,
        "group-by" => PipelineFn::GroupBy,
        "flatten" => PipelineFn::Flatten,
        "distinct" => PipelineFn::Distinct,
        "shuffle" => PipelineFn::Shuffle,
        _ => panic!("unknown pipeline function: {}", fn_pair.as_str()),
    };
    let argument = inner.next().map(|arg_pair| {
        let expr_pair = arg_pair.into_inner().next().unwrap();
        parse_expression(expr_pair)
    });
    let argument2 = inner.next().map(|arg_pair| {
        let expr_pair = arg_pair.into_inner().next().unwrap();
        parse_expression(expr_pair)
    });
    PipelineStage {
        function,
        argument,
        argument2,
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
        Rule::function_call => {
            let mut call_inner = inner.into_inner();
            let name = call_inner.next().unwrap().as_str().to_string();
            let args = match call_inner.next() {
                Some(args_pair) => args_pair.into_inner().map(parse_expression).collect(),
                None => vec![],
            };
            Expression::FunctionCall { name, args }
        }
        Rule::ref_path => {
            let segments: Vec<String> = inner.as_str().split('.').map(String::from).collect();
            Expression::StateRef(segments.join("."))
        }
        Rule::object_lit => {
            let entries: Vec<(String, Value)> = inner
                .into_inner()
                .map(|entry| {
                    let mut entry_inner = entry.into_inner();
                    let key = entry_inner.next().unwrap().as_str().to_string();
                    let val = parse_value(entry_inner.next().unwrap());
                    (key, val)
                })
                .collect();
            Expression::Literal(Value::Object(entries))
        }
        Rule::list_lit => {
            let items: Vec<Value> = inner.into_inner().map(parse_value).collect();
            Expression::Literal(Value::List(items))
        }
        Rule::index_access => {
            let mut idx_inner = inner.into_inner();
            let list = idx_inner.next().unwrap().as_str().to_string();
            let index = Box::new(parse_expression(idx_inner.next().unwrap()));
            Expression::Index { list, index }
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

// ─── Test file parser (for .test.naze files) ────────────────────────────────

/// Parse a .test.naze source string into a TestFile AST.
pub fn parse_test_file(source: &str, file: &str) -> Result<TestFile, ParseError> {
    let pairs = NazeParser::parse(Rule::test_file, source).map_err(|e| {
        let (line, column) = match e.line_col {
            pest::error::LineColLocation::Pos((l, c)) => (l, c),
            pest::error::LineColLocation::Span((l, c), _) => (l, c),
        };
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

    let file_pair = pairs.into_iter().next().unwrap();
    let mut uses = Vec::new();
    let mut tests = Vec::new();
    let mut flows = Vec::new();

    for pair in file_pair.into_inner() {
        match pair.as_rule() {
            Rule::test_use_stmt => {
                let path_pair = pair.into_inner().next().unwrap();
                uses.push(path_pair.as_str().to_string());
            }
            Rule::test_block => tests.push(parse_test_block(pair, file)),
            Rule::flow_block => flows.push(parse_flow_block(pair, file)),
            Rule::comment | Rule::EOI => {}
            _ => {}
        }
    }

    Ok(TestFile { uses, tests, flows })
}

fn parse_test_block(pair: pest::iterators::Pair<Rule>, file: &str) -> TestBlock {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = extract_plain_string(inner.next().unwrap());
    let mut steps = Vec::new();
    for p in inner {
        if let Some(step) = parse_test_step(p, file) {
            steps.push(step);
        }
    }
    TestBlock { name, steps, span }
}

fn parse_flow_block(pair: pest::iterators::Pair<Rule>, file: &str) -> FlowBlock {
    let span = span_from_pair(&pair, file);
    let mut inner = pair.into_inner();
    let name = extract_plain_string(inner.next().unwrap());
    let mut steps = Vec::new();
    for p in inner {
        if let Some(step) = parse_test_step(p, file) {
            steps.push(step);
        }
    }
    FlowBlock { name, steps, span }
}

fn parse_test_step(pair: pest::iterators::Pair<Rule>, file: &str) -> Option<TestStep> {
    let span = span_from_pair(&pair, file);
    match pair.as_rule() {
        Rule::test_render => {
            let mut inner = pair.into_inner();
            let component = inner.next().unwrap().as_str().to_string();
            let props = match inner.next() {
                Some(p) if p.as_rule() == Rule::inline_props => parse_inline_props(p),
                _ => Vec::new(),
            };
            Some(TestStep::Render {
                component,
                props,
                span,
            })
        }
        Rule::test_click => {
            let text = extract_plain_string(pair.into_inner().next().unwrap());
            Some(TestStep::Click { text, span })
        }
        Rule::test_fill => {
            let mut inner = pair.into_inner();
            let target = extract_plain_string(inner.next().unwrap());
            let value = extract_plain_string(inner.next().unwrap());
            Some(TestStep::Fill {
                target,
                value,
                span,
            })
        }
        Rule::test_navigate => {
            let path = extract_plain_string(pair.into_inner().next().unwrap());
            Some(TestStep::Navigate { path, span })
        }
        Rule::test_wait => {
            let duration_ms = parse_duration_ms(pair.into_inner().next().unwrap());
            Some(TestStep::Wait { duration_ms, span })
        }
        Rule::test_assert => {
            let kind_pair = pair.into_inner().next().unwrap();
            let kind = parse_assert_kind(kind_pair);
            Some(TestStep::Assert { kind, span })
        }
        Rule::comment => None,
        _ => None,
    }
}

fn parse_assert_kind(pair: pest::iterators::Pair<Rule>) -> AssertKind {
    match pair.as_rule() {
        Rule::assert_text_visible => {
            let text = extract_plain_string(pair.into_inner().next().unwrap());
            AssertKind::TextVisible(text)
        }
        Rule::assert_text_not_visible => {
            let text = extract_plain_string(pair.into_inner().next().unwrap());
            AssertKind::TextNotVisible(text)
        }
        Rule::assert_page => {
            let path = extract_plain_string(pair.into_inner().next().unwrap());
            AssertKind::PageIs(path)
        }
        Rule::assert_state => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let value = parse_value(inner.next().unwrap());
            AssertKind::StateIs { name, value }
        }
        Rule::assert_emitted => {
            let name = pair.into_inner().next().unwrap().as_str().to_string();
            AssertKind::Emitted(name)
        }
        Rule::assert_a11y => AssertKind::NoA11yViolations,
        _ => panic!("unexpected assert kind: {:?}", pair.as_rule()),
    }
}

/// Extract a plain string from a string_lit pair (ignoring interpolation).
fn extract_plain_string(pair: pest::iterators::Pair<Rule>) -> String {
    match parse_string_lit(pair) {
        Value::Str(s) => s,
        Value::InterpolatedStr(parts) => parts
            .into_iter()
            .map(|p| match p {
                StringPart::Literal(s) => s,
                StringPart::Interpolation(segs) => format!("{{{}}}", segs.join(".")),
            })
            .collect(),
        _ => String::new(),
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
            Node::App {
                title, children, ..
            } => {
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
    fn parse_use_scoped_package() {
        let source = "use @naze/ui-kit/button\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::UseStmt { path, .. } => {
                assert_eq!(path, &["@naze", "ui-kit", "button"]);
            }
            _ => panic!("expected UseStmt"),
        }
    }

    #[test]
    fn parse_use_scoped_package_minimal() {
        let source = "use @org/lib\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::UseStmt { path, .. } => {
                assert_eq!(path, &["@org", "lib"]);
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
                name,
                params,
                children,
                ..
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
    fn parse_template_def() {
        let source = r#"template my-layout(header, main, footer) {
  column {
    slot "header"
    slot "main"
    slot "footer"
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Template {
                name,
                slots,
                children,
                ..
            } => {
                assert_eq!(name, "my-layout");
                assert_eq!(slots, &["header", "main", "footer"]);
                assert_eq!(children.len(), 1); // the column element
            }
            _ => panic!("expected Template"),
        }
    }

    #[test]
    fn parse_template_no_slots() {
        let source = r#"template centered {
  column max-width: 800px {
    slot
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Template {
                name,
                slots,
                children,
                ..
            } => {
                assert_eq!(name, "centered");
                assert!(slots.is_empty());
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected Template"),
        }
    }

    #[test]
    fn parse_emit_action() {
        let source = "app \"Test\" {\n  rect {\n    on click: emit toggled\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[0] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    assert_eq!(handlers[0].event, "click");
                    match &handlers[0].actions[0] {
                        Action::Emit { event_name, .. } => {
                            assert_eq!(event_name, "toggled");
                        }
                        _ => panic!("expected Emit action"),
                    }
                }
                _ => panic!("expected Element"),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_custom_event_name() {
        let source = "app \"Test\" {\n  rect {\n    on toggle-sidebar: set open = true\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[0] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers[0].event, "toggle-sidebar");
                }
                _ => panic!("expected Element"),
            },
            _ => panic!("expected App"),
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
                        name,
                        children: col_children,
                        ..
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
                        match &handlers[0].actions[0] {
                            Action::Set { target, expr, .. } => {
                                assert_eq!(target, "count");
                                // expr should be count + 1
                                match expr {
                                    Expression::BinOp { left, op, right } => {
                                        assert!(
                                            matches!(**left, Expression::StateRef(ref s) if s == "count")
                                        );
                                        assert_eq!(*op, BinOp::Add);
                                        assert!(
                                            matches!(**right, Expression::Literal(Value::Num(n, _)) if n == 1.0)
                                        );
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
                Node::Element { handlers, .. } => match &handlers[0].actions[0] {
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
                    match &handlers[0].actions[0] {
                        Action::Set { target, expr, .. } => {
                            assert_eq!(target, "count");
                            assert!(
                                matches!(expr, Expression::Literal(Value::Num(n, _)) if *n == 0.0)
                            );
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
                    match &handlers[0].actions[0] {
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
                name, storage_type, ..
            } => {
                assert_eq!(name, "token");
                assert!(matches!(storage_type, StorageType::Session));
            }
            _ => panic!("expected Storage node"),
        }
    }

    #[test]
    fn parse_data_with_config() {
        let source =
            "data users: fetch \"/api/users\" {\n  method: post\n  cache: 5min\n  retry: 3\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Data {
                name, url, config, ..
            } => {
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
    fn parse_data_with_headers() {
        let source = "data users: fetch \"/api/users\" {\n  headers: { Authorization: \"Bearer abc123\", X-Api-Key: \"key\" }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Data { name, config, .. } => {
                assert_eq!(name, "users");
                assert_eq!(config.headers.len(), 2);
                assert_eq!(config.headers[0].0, "Authorization");
                assert!(matches!(&config.headers[0].1, Value::Str(s) if s == "Bearer abc123"));
                assert_eq!(config.headers[1].0, "X-Api-Key");
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
            Node::Timer {
                name,
                kind,
                duration_ms,
                action,
                ..
            } => {
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
            Node::Timer {
                name,
                kind,
                duration_ms,
                ..
            } => {
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
            Node::App { children, .. } => match &children[1] {
                Node::If {
                    then_children,
                    else_children,
                    ..
                } => {
                    assert_eq!(then_children.len(), 1);
                    assert_eq!(else_children.len(), 1);
                }
                other => panic!("expected If, got {:?}", other),
            },
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
            Node::App { children, .. } => match &children[1] {
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
            },
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
        let source =
            "component page(title: text) {\n  slot \"header\"\n  slot\n  slot \"footer\"\n}\n";
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
        let source =
            "component panel(title: text) {\n  slot {\n    text \"default content\"\n  }\n}\n";
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
                    Node::Fill { name, children, .. } => {
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
            Node::App { children, .. } => match &children[0] {
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
            },
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
                name,
                extends,
                colors,
                spacing,
                ..
            } => {
                assert!(name.is_none());
                assert!(extends.is_none());
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
    fn parse_named_theme() {
        let source = r#"theme dark {
  colors {
    primary: #60a5fa
    bg: #1e293b
  }
}
"#;
        let nodes = parse(source, "theme.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Theme {
                name,
                extends,
                colors,
                ..
            } => {
                assert_eq!(name.as_deref(), Some("dark"));
                assert!(extends.is_none());
                assert_eq!(colors.len(), 2);
                assert_eq!(colors[0].0, "primary");
            }
            other => panic!("expected Theme, got {:?}", other),
        }
    }

    #[test]
    fn parse_theme_extends() {
        let source = r#"theme dark extends light {
  colors {
    bg: #1e293b
  }
}
"#;
        let nodes = parse(source, "theme.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Theme { name, extends, .. } => {
                assert_eq!(name.as_deref(), Some("dark"));
                assert_eq!(extends.as_deref(), Some("light"));
            }
            other => panic!("expected Theme, got {:?}", other),
        }
    }

    #[test]
    fn parse_set_theme_action() {
        let source = "app \"Test\" {\n  rect {\n    on click: set-theme \"dark\"\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[0] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    match &handlers[0].actions[0] {
                        Action::SetTheme { theme_name, .. } => {
                            assert_eq!(theme_name, "dark");
                        }
                        _ => panic!("expected SetTheme action"),
                    }
                }
                _ => panic!("expected Element"),
            },
            _ => panic!("expected App"),
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
    fn parse_dynamic_page_block() {
        let source = r#"page "/posts/:id" {
  heading "Post Detail"
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Page {
                path,
                params,
                children,
                ..
            } => {
                assert_eq!(path, "/posts/:id");
                assert_eq!(params, &vec!["id".to_string()]);
                assert_eq!(children.len(), 1);
            }
            other => panic!("expected Page, got {:?}", other),
        }
    }

    #[test]
    fn parse_catch_all_page() {
        let source = r#"page "/*" {
  text "Not found"
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Page { path, params, .. } => {
                assert_eq!(path, "/*");
                assert!(params.is_empty());
            }
            other => panic!("expected Page, got {:?}", other),
        }
    }

    #[test]
    fn parse_multi_param_page() {
        let source = r#"page "/users/:userId/posts/:postId" {
  text "User post"
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Page { path, params, .. } => {
                assert_eq!(path, "/users/:userId/posts/:postId");
                assert_eq!(params, &vec!["userId".to_string(), "postId".to_string()]);
            }
            other => panic!("expected Page, got {:?}", other),
        }
    }

    #[test]
    fn parse_boundary_stmt() {
        let source = r#"boundary {
  data users: fetch "https://api.example.com/users"
  text "Users loaded"
} catch {
  text "Something went wrong"
}
"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Boundary {
                children,
                catch_children,
                ..
            } => {
                assert_eq!(children.len(), 2); // data + text
                assert_eq!(catch_children.len(), 1); // text
            }
            other => panic!("expected Boundary, got {:?}", other),
        }
    }

    #[test]
    fn parse_link_element() {
        let source = r#"link "Go to About", to: "/about"
"#;
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Link {
                text, to, children, ..
            } => {
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
                        name,
                        props,
                        children,
                        ..
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
                                assert!(
                                    matches!(&props[0].value, Value::Str(s) if s == "menu-btn")
                                );
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
            Node::App { children, .. } => match &children[1] {
                Node::Element { name, handlers, .. } => {
                    assert_eq!(name, "overlay");
                    assert_eq!(handlers.len(), 1);
                    assert_eq!(handlers[0].event, "click-outside");
                    match &handlers[0].actions[0] {
                        Action::Set { target, .. } => assert_eq!(target, "open"),
                        other => panic!("expected Set action, got {:?}", other),
                    }
                }
                other => panic!("expected overlay Element, got {:?}", other),
            },
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
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers[0].event, "context-menu");
                }
                other => panic!("expected Element, got {:?}", other),
            },
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
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers[0].event, "pointer-move");
                }
                other => panic!("expected Element, got {:?}", other),
            },
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
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 4);
                    assert_eq!(handlers[0].event, "arrow-up");
                    assert_eq!(handlers[1].event, "arrow-down");
                    assert_eq!(handlers[2].event, "arrow-left");
                    assert_eq!(handlers[3].event, "arrow-right");
                }
                other => panic!("expected Element, got {:?}", other),
            },
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
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    match &handlers[0].actions[0] {
                        Action::Trigger { data_name, .. } => assert_eq!(data_name, "items"),
                        other => panic!("expected Trigger action, got {:?}", other),
                    }
                }
                other => panic!("expected Element, got {:?}", other),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_copy_action() {
        let source = "app \"Test\" {\n  state url = \"https://example.com\"\n  rect {\n    on click: copy url\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    match &handlers[0].actions[0] {
                        Action::Copy { expr, .. } => {
                            assert!(matches!(expr, Expression::StateRef(n) if n == "url"));
                        }
                        other => panic!("expected Copy action, got {:?}", other),
                    }
                }
                other => panic!("expected Element, got {:?}", other),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_data_stream() {
        let source = "data chat: stream \"wss://api.example.com/chat\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Data {
                name, url, source, ..
            } => {
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
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    match &handlers[0].actions[0] {
                        Action::Send {
                            stream_name, expr, ..
                        } => {
                            assert_eq!(stream_name, "chat");
                            assert!(matches!(expr, Expression::StateRef(n) if n == "msg"));
                        }
                        other => panic!("expected Send action, got {:?}", other),
                    }
                }
                other => panic!("expected Element, got {:?}", other),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_param_stmt_test() {
        let source = "param page: number default: 1\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Param {
                name, ty, default, ..
            } => {
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
            Node::Param {
                name, ty, default, ..
            } => {
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
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    let m = handlers[0].modifier.as_ref().unwrap();
                    assert!(matches!(m.kind, ModifierKind::Debounce));
                    assert_eq!(m.duration_ms, 300);
                }
                other => panic!("expected Element, got {:?}", other),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_throttle_modifier() {
        let source = "app \"Test\" {\n  state x = 0\n  rect {\n    on scroll throttle 100ms: set x = x + 1\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    let m = handlers[0].modifier.as_ref().unwrap();
                    assert!(matches!(m.kind, ModifierKind::Throttle));
                    assert_eq!(m.duration_ms, 100);
                }
                other => panic!("expected Element, got {:?}", other),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_no_modifier() {
        let source =
            "app \"Test\" {\n  state c = 0\n  rect {\n    on click: set c = c + 1\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[1] {
                Node::Element { handlers, .. } => {
                    assert_eq!(handlers.len(), 1);
                    assert!(handlers[0].modifier.is_none());
                }
                other => panic!("expected Element, got {:?}", other),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_computed_pipeline() {
        let source = "computed total = items | map price | sum\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Computed { name, expr, .. } => {
                assert_eq!(name, "total");
                match expr {
                    Expression::Pipeline { source, stages } => {
                        assert!(matches!(**source, Expression::StateRef(ref s) if s == "items"));
                        assert_eq!(stages.len(), 2);
                        assert!(matches!(stages[0].function, PipelineFn::Map));
                        assert!(stages[0].argument.is_some());
                        assert!(matches!(stages[1].function, PipelineFn::Sum));
                        assert!(stages[1].argument.is_none());
                    }
                    _ => panic!("expected Pipeline expression"),
                }
            }
            _ => panic!("expected Computed node"),
        }
    }

    #[test]
    fn parse_pipeline_filter() {
        let source = "computed passing = students | filter score > 60\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Computed { expr, .. } => {
                match expr {
                    Expression::Pipeline { source, stages } => {
                        assert!(matches!(**source, Expression::StateRef(ref s) if s == "students"));
                        assert_eq!(stages.len(), 1);
                        assert!(matches!(stages[0].function, PipelineFn::Filter));
                        // Argument is "score > 60" — a BinOp expression
                        match &stages[0].argument {
                            Some(Expression::BinOp { left, op, right }) => {
                                assert!(
                                    matches!(**left, Expression::StateRef(ref s) if s == "score")
                                );
                                assert_eq!(*op, BinOp::Gt);
                                assert!(
                                    matches!(**right, Expression::Literal(Value::Num(n, _)) if n == 60.0)
                                );
                            }
                            other => panic!("expected BinOp argument, got {:?}", other),
                        }
                    }
                    _ => panic!("expected Pipeline expression"),
                }
            }
            _ => panic!("expected Computed node"),
        }
    }

    #[test]
    fn parse_pipeline_sort_by_take() {
        let source = "computed top = items | sort-by name | take 3\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Computed { expr, .. } => match expr {
                Expression::Pipeline { stages, .. } => {
                    assert_eq!(stages.len(), 2);
                    assert!(matches!(stages[0].function, PipelineFn::SortBy));
                    assert!(matches!(stages[1].function, PipelineFn::Take));
                }
                _ => panic!("expected Pipeline expression"),
            },
            _ => panic!("expected Computed node"),
        }
    }

    #[test]
    fn parse_pipeline_count() {
        let source = "computed n = items | count\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Computed { expr, .. } => match expr {
                Expression::Pipeline { stages, .. } => {
                    assert_eq!(stages.len(), 1);
                    assert!(matches!(stages[0].function, PipelineFn::Count));
                    assert!(stages[0].argument.is_none());
                }
                _ => panic!("expected Pipeline expression"),
            },
            _ => panic!("expected Computed node"),
        }
    }

    #[test]
    fn parse_each_with_pipeline() {
        let source = r#"app "Test" {
  state items = [1, 2, 3]
  each item in items | take 2 {
    text "{item}"
  }
}"#;
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::App { children, .. } => match &children[1] {
                Node::Each {
                    variable,
                    iterable,
                    children,
                    ..
                } => {
                    assert_eq!(variable, "item");
                    assert!(matches!(iterable, Expression::Pipeline { .. }));
                    assert_eq!(children.len(), 1);
                }
                other => panic!("expected Each, got {:?}", other),
            },
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_computed_no_pipeline() {
        // Verify non-pipeline computed still works
        let source = "computed total = count * price\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Computed { expr, .. } => {
                assert!(matches!(expr, Expression::BinOp { .. }));
            }
            _ => panic!("expected Computed node"),
        }
    }

    #[test]
    fn parse_import_stmt() {
        let source = "import crypto from \"./lib/crypto.wasm\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Import { name, source, .. } => {
                assert_eq!(name, "crypto");
                assert_eq!(source, "./lib/crypto.wasm");
            }
            _ => panic!("expected Import, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_import_scoped_package() {
        let source = "import math from \"@naze/math\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Import { name, source, .. } => {
                assert_eq!(name, "math");
                assert_eq!(source, "@naze/math");
            }
            _ => panic!("expected Import"),
        }
    }

    #[test]
    fn parse_qualified_function_call() {
        let source = "computed total = crypto.hash(x)\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::Computed { name, expr, .. } => {
                assert_eq!(name, "total");
                match expr {
                    Expression::FunctionCall { name, args } => {
                        assert_eq!(name, "crypto.hash");
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("expected FunctionCall, got {:?}", expr),
                }
            }
            _ => panic!("expected Computed"),
        }
    }

    #[test]
    fn parse_server_function_def() {
        let source = "server function add(x: number, y: number) {\n  x + y\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::ServerFunction {
                name, params, body, ..
            } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "x");
                assert_eq!(params[1].name, "y");
                assert!(body.lets.is_empty());
                match &body.result {
                    Expression::BinOp { op, .. } => {
                        assert!(matches!(op, BinOp::Add));
                    }
                    _ => panic!("expected BinOp, got {:?}", body.result),
                }
            }
            _ => panic!("expected ServerFunction, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_server_function_no_params() {
        let source = "server function get-config() {\n  42\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::ServerFunction {
                name, params, body, ..
            } => {
                assert_eq!(name, "get-config");
                assert_eq!(params.len(), 0);
                assert!(body.lets.is_empty());
                match &body.result {
                    Expression::Literal(Value::Num(n, _)) => assert_eq!(*n, 42.0),
                    _ => panic!("expected Num literal, got {:?}", body.result),
                }
            }
            _ => panic!("expected ServerFunction"),
        }
    }

    #[test]
    fn parse_server_function_with_lets() {
        let source = "server function get-user(id: number) {\n  let user = fetch \"https://api.example.com/users/{id}\"\n  let name = user.name\n  name\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::ServerFunction {
                name, params, body, ..
            } => {
                assert_eq!(name, "get-user");
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "id");
                assert_eq!(body.lets.len(), 2);
                assert_eq!(body.lets[0].0, "user");
                assert!(matches!(body.lets[0].1, ServerExpr::Fetch(_)));
                assert_eq!(body.lets[1].0, "name");
                assert!(matches!(body.lets[1].1, ServerExpr::Expr(_)));
                assert!(matches!(body.result, Expression::StateRef(_)));
            }
            _ => panic!("expected ServerFunction"),
        }
    }

    #[test]
    fn parse_meta_stmt() {
        let source = "meta title: \"My Page\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Meta { key, value, .. } => {
                assert_eq!(key, "title");
                match value {
                    Value::Str(s) => assert_eq!(s, "My Page"),
                    _ => panic!("expected Str value, got {:?}", value),
                }
            }
            _ => panic!("expected Meta, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_meta_in_page_block() {
        let source = "page \"/about\" {\n  meta title: \"About Us\"\n  meta description: \"Our company info\"\n  text \"hello\"\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Page { children, .. } => {
                // meta + meta + text = 3 children
                assert_eq!(children.len(), 3);
                assert!(matches!(&children[0], Node::Meta { key, .. } if key == "title"));
                assert!(matches!(&children[1], Node::Meta { key, .. } if key == "description"));
                assert!(matches!(&children[2], Node::Element { name, .. } if name == "text"));
            }
            _ => panic!("expected Page"),
        }
    }

    #[test]
    fn parse_server_data_stmt() {
        let source = "data result: add(1, 2)\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::ServerData {
                name,
                func_name,
                args,
                ..
            } => {
                assert_eq!(name, "result");
                assert_eq!(func_name, "add");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("expected ServerData, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_server_data_no_args() {
        let source = "data config: get-config()\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::ServerData {
                name,
                func_name,
                args,
                ..
            } => {
                assert_eq!(name, "config");
                assert_eq!(func_name, "get-config");
                assert_eq!(args.len(), 0);
            }
            _ => panic!("expected ServerData"),
        }
    }

    #[test]
    fn parse_prompt_all_props() {
        let source = "prompt summary: from openai {\n  system: \"Summarize concisely.\"\n  user: \"Tell me about {topic}\"\n  model: \"gpt-4o\"\n  max-tokens: 500\n  temperature: 0.7\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Prompt {
                name,
                provider,
                props,
                ..
            } => {
                assert_eq!(name, "summary");
                assert_eq!(provider, "openai");
                assert_eq!(props.len(), 5);
                assert_eq!(props[0].0, "system");
                assert_eq!(props[2].0, "model");
                assert_eq!(props[3].0, "max-tokens");
                assert_eq!(props[4].0, "temperature");
            }
            _ => panic!("expected Prompt, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_prompt_minimal() {
        let source =
            "prompt answer: from ollama {\n  system: \"You are helpful.\"\n  user: \"Hello\"\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Prompt {
                name,
                provider,
                props,
                ..
            } => {
                assert_eq!(name, "answer");
                assert_eq!(provider, "ollama");
                assert_eq!(props.len(), 2);
            }
            _ => panic!("expected Prompt"),
        }
    }

    #[test]
    fn parse_prompt_in_app() {
        let source = "app \"AI Test\" {\n  prompt reply: from anthropic {\n    system: \"Be concise.\"\n    user: \"Hi\"\n  }\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::App { children, .. } => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Prompt { name, provider, .. } => {
                        assert_eq!(name, "reply");
                        assert_eq!(provider, "anthropic");
                    }
                    _ => panic!("expected Prompt in app"),
                }
            }
            _ => panic!("expected App"),
        }
    }

    #[test]
    fn parse_guard_def() {
        let source = "guard is-admin\n  check auth-token == false redirect \"/login\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Guard { name, checks, .. } => {
                assert_eq!(name, "is-admin");
                assert_eq!(checks.len(), 1);
                assert_eq!(checks[0].redirect, "/login");
            }
            _ => panic!("expected Guard, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_guard_multiple_checks() {
        let source = "guard require-auth\n  check logged-in == false redirect \"/login\"\n  check is-verified == false redirect \"/verify\"\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Guard { name, checks, .. } => {
                assert_eq!(name, "require-auth");
                assert_eq!(checks.len(), 2);
                assert_eq!(checks[0].redirect, "/login");
                assert_eq!(checks[1].redirect, "/verify");
            }
            _ => panic!("expected Guard"),
        }
    }

    #[test]
    fn parse_page_with_guard() {
        let source = "page \"/admin\" guard: is-admin {\n  text \"Admin Panel\"\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Page {
                path,
                guard,
                children,
                ..
            } => {
                assert_eq!(path, "/admin");
                assert_eq!(guard.as_deref(), Some("is-admin"));
                assert_eq!(children.len(), 1);
            }
            _ => panic!("expected Page, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_page_without_guard() {
        let source = "page \"/about\" {\n  text \"About\"\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Page { guard, .. } => {
                assert!(guard.is_none());
            }
            _ => panic!("expected Page"),
        }
    }

    #[test]
    fn parse_server_function_with_sql() {
        let source = "server function get-users(limit: number) {\n  let users = sql \"SELECT id, name FROM users LIMIT $1\" [limit]\n  users\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::ServerFunction {
                name, params, body, ..
            } => {
                assert_eq!(name, "get-users");
                assert_eq!(params.len(), 1);
                assert_eq!(body.lets.len(), 1);
                assert_eq!(body.lets[0].0, "users");
                match &body.lets[0].1 {
                    ServerExpr::Sql { query, params } => {
                        assert_eq!(query, "SELECT id, name FROM users LIMIT $1");
                        assert_eq!(params.len(), 1);
                    }
                    _ => panic!("expected ServerExpr::Sql"),
                }
            }
            _ => panic!("expected ServerFunction, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_server_function_sql_no_params() {
        let source =
            "server function get-all() {\n  let rows = sql \"SELECT * FROM items\"\n  rows\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::ServerFunction { body, .. } => {
                assert_eq!(body.lets.len(), 1);
                match &body.lets[0].1 {
                    ServerExpr::Sql { query, params } => {
                        assert_eq!(query, "SELECT * FROM items");
                        assert!(params.is_empty());
                    }
                    _ => panic!("expected ServerExpr::Sql"),
                }
            }
            _ => panic!("expected ServerFunction"),
        }
    }

    // ─── M39: Declarative Database Queries ──────────────────────────────────────

    #[test]
    fn parse_model_def() {
        let source = "model users {\n  id number primary\n  name text\n  email text unique\n  active bool\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Model { name, fields, .. } => {
                assert_eq!(name, "users");
                assert_eq!(fields.len(), 4);
                assert_eq!(fields[0].name, "id");
                assert_eq!(fields[0].field_type, "number");
                assert!(fields[0].constraints.iter().any(|c| c == "primary"));
                assert_eq!(fields[1].name, "name");
                assert_eq!(fields[1].field_type, "text");
                assert!(fields[1].constraints.is_empty());
                assert_eq!(fields[2].name, "email");
                assert_eq!(fields[2].field_type, "text");
                assert!(fields[2].constraints.iter().any(|c| c == "unique"));
                assert_eq!(fields[3].name, "active");
                assert_eq!(fields[3].field_type, "bool");
            }
            _ => panic!("expected Model, got {:?}", nodes[0]),
        }
    }

    #[test]
    fn parse_model_with_defaults() {
        let source = "model posts {\n  id number primary\n  title text\n  published bool default false\n  created-at timestamp default now\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Model { name, fields, .. } => {
                assert_eq!(name, "posts");
                assert_eq!(fields.len(), 4);
                assert!(fields[2].constraints.iter().any(|c| c == "default:false"));
                assert!(fields[3].constraints.iter().any(|c| c == "default:now"));
            }
            _ => panic!("expected Model"),
        }
    }

    #[test]
    fn parse_find_query() {
        let source = "server function get-users() {\n  let users = find users where active == true order name limit 10\n  users\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::ServerFunction { body, .. } => {
                assert_eq!(body.lets.len(), 1);
                match &body.lets[0].1 {
                    ServerExpr::Find {
                        model,
                        conditions,
                        order,
                        limit,
                    } => {
                        assert_eq!(model, "users");
                        assert_eq!(conditions.len(), 1);
                        assert_eq!(conditions[0].field, "active");
                        assert_eq!(conditions[0].op, "==");
                        assert!(order.is_some());
                        let (field, asc) = order.as_ref().unwrap();
                        assert_eq!(field, "name");
                        assert!(*asc);
                        assert!(limit.is_some());
                    }
                    _ => panic!("expected ServerExpr::Find"),
                }
            }
            _ => panic!("expected ServerFunction"),
        }
    }

    #[test]
    fn parse_find_no_clauses() {
        let source = "server function get-all() {\n  let items = find items\n  items\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::ServerFunction { body, .. } => match &body.lets[0].1 {
                ServerExpr::Find {
                    model,
                    conditions,
                    order,
                    limit,
                } => {
                    assert_eq!(model, "items");
                    assert!(conditions.is_empty());
                    assert!(order.is_none());
                    assert!(limit.is_none());
                }
                _ => panic!("expected ServerExpr::Find"),
            },
            _ => panic!("expected ServerFunction"),
        }
    }

    #[test]
    fn parse_find_multiple_conditions() {
        let source = "server function search() {\n  let users = find users where active == true and age > 18\n  users\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::ServerFunction { body, .. } => match &body.lets[0].1 {
                ServerExpr::Find { conditions, .. } => {
                    assert_eq!(conditions.len(), 2);
                    assert_eq!(conditions[0].field, "active");
                    assert_eq!(conditions[0].op, "==");
                    assert_eq!(conditions[1].field, "age");
                    assert_eq!(conditions[1].op, ">");
                }
                _ => panic!("expected ServerExpr::Find"),
            },
            _ => panic!("expected ServerFunction"),
        }
    }

    #[test]
    fn parse_insert_query() {
        let source = "server function create-user(name: text, email: text) {\n  let user = insert users { name: name, email: email }\n  user\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::ServerFunction { body, .. } => match &body.lets[0].1 {
                ServerExpr::Insert { model, fields } => {
                    assert_eq!(model, "users");
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].0, "name");
                    assert_eq!(fields[1].0, "email");
                }
                _ => panic!("expected ServerExpr::Insert"),
            },
            _ => panic!("expected ServerFunction"),
        }
    }

    #[test]
    fn parse_update_query() {
        let source = "server function update-user(id: number, name: text) {\n  let result = update users set { name: name } where id == id\n  result\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::ServerFunction { body, .. } => match &body.lets[0].1 {
                ServerExpr::Update {
                    model,
                    set_fields,
                    conditions,
                } => {
                    assert_eq!(model, "users");
                    assert_eq!(set_fields.len(), 1);
                    assert_eq!(set_fields[0].0, "name");
                    assert_eq!(conditions.len(), 1);
                    assert_eq!(conditions[0].field, "id");
                    assert_eq!(conditions[0].op, "==");
                }
                _ => panic!("expected ServerExpr::Update"),
            },
            _ => panic!("expected ServerFunction"),
        }
    }

    #[test]
    fn parse_delete_query() {
        let source = "server function remove-user(id: number) {\n  let result = delete users where id == id\n  result\n}\n";
        let nodes = parse(source, "test.naze").unwrap();
        match &nodes[0] {
            Node::ServerFunction { body, .. } => match &body.lets[0].1 {
                ServerExpr::Delete { model, conditions } => {
                    assert_eq!(model, "users");
                    assert_eq!(conditions.len(), 1);
                    assert_eq!(conditions[0].field, "id");
                }
                _ => panic!("expected ServerExpr::Delete"),
            },
            _ => panic!("expected ServerFunction"),
        }
    }
}
