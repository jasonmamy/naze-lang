//! Shared server function helpers: JSON conversion and evaluation.
//! Used by both `dev.rs` (dev server) and `serve.rs` (production SSR server).

use std::collections::HashMap;

use naze_ir::{IrExpression, IrServerStep, RenderValue, ServerFuncDecl, TextPart};

/// Evaluate a server function with JSON arguments, returning a JSON result.
/// Optional `request_headers` are forwarded on outgoing fetch calls (e.g. Authorization).
pub fn evaluate_server_fn(
    func: &ServerFuncDecl,
    args_json: &[serde_json::Value],
) -> serde_json::Value {
    evaluate_server_fn_with_headers(func, args_json, &[])
}

/// Evaluate a server function, forwarding the given request headers to outgoing fetches.
pub fn evaluate_server_fn_with_headers(
    func: &ServerFuncDecl,
    args_json: &[serde_json::Value],
    forwarded_headers: &[(String, String)],
) -> serde_json::Value {
    let mut eval_state = HashMap::new();
    for (i, param_name) in func.params.iter().enumerate() {
        let val = args_json
            .get(i)
            .map(json_to_render_value)
            .unwrap_or(RenderValue::Num(0.0, None));
        eval_state.insert(param_name.clone(), val);
    }
    // Evaluate let bindings sequentially
    for (name, step) in &func.body.lets {
        let val = match step {
            IrServerStep::Fetch(url) => {
                // Resolve interpolations in URL against current state
                let resolved_url = resolve_url_interpolations(url, &eval_state);
                // Perform blocking HTTP GET with forwarded headers
                let client = reqwest::blocking::Client::new();
                let mut req = client.get(&resolved_url);
                for (hk, hv) in forwarded_headers {
                    req = req.header(hk, hv);
                }
                match req.send() {
                    Ok(resp) => match resp.json::<serde_json::Value>() {
                        Ok(json) => json_to_render_value(&json),
                        Err(_) => RenderValue::Str(String::new()),
                    },
                    Err(_) => RenderValue::Str(String::new()),
                }
            }
            IrServerStep::Sql { query, params } => execute_sql(query, params, &eval_state),
            IrServerStep::Expr(expr) => crate::exec::evaluate_expr(expr, &eval_state),
        };
        eval_state.insert(name.clone(), val);
    }
    let result = crate::exec::evaluate_expr(&func.body.result, &eval_state);
    render_value_to_json(&result)
}

/// Resolve {name} interpolations in a URL string against state.
fn resolve_url_interpolations(url: &str, state: &HashMap<String, RenderValue>) -> String {
    let mut result = String::with_capacity(url.len());
    let mut chars = url.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut name = String::new();
            for c2 in chars.by_ref() {
                if c2 == '}' {
                    break;
                }
                name.push(c2);
            }
            if let Some(val) = state.get(&name) {
                match val {
                    RenderValue::Str(s) => result.push_str(s),
                    RenderValue::Num(n, _) => {
                        if n.fract() == 0.0 {
                            result.push_str(&format!("{}", *n as i64));
                        } else {
                            result.push_str(&format!("{}", n));
                        }
                    }
                    _ => result.push_str(&format!("{{{}}}", name)),
                }
            } else {
                result.push_str(&format!("{{{}}}", name));
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert a serde_json::Value to a RenderValue for server function args.
pub fn json_to_render_value(v: &serde_json::Value) -> RenderValue {
    match v {
        serde_json::Value::Number(n) => RenderValue::Num(n.as_f64().unwrap_or(0.0), None),
        serde_json::Value::String(s) => RenderValue::Str(s.clone()),
        serde_json::Value::Bool(b) => RenderValue::Bool(*b),
        serde_json::Value::Array(arr) => {
            RenderValue::List(arr.iter().map(json_to_render_value).collect())
        }
        serde_json::Value::Object(map) => RenderValue::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), json_to_render_value(v)))
                .collect(),
        ),
        serde_json::Value::Null => RenderValue::Str(String::new()),
    }
}

/// Convert a RenderValue to serde_json::Value for API responses.
pub fn render_value_to_json(v: &RenderValue) -> serde_json::Value {
    match v {
        RenderValue::Str(s) => serde_json::Value::String(s.clone()),
        RenderValue::Num(n, _) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        RenderValue::Bool(b) => serde_json::Value::Bool(*b),
        RenderValue::Color(c) => serde_json::Value::String(format!("#{:06x}", c)),
        RenderValue::List(items) => {
            serde_json::Value::Array(items.iter().map(render_value_to_json).collect())
        }
        RenderValue::Object(entries) => {
            let map: serde_json::Map<String, serde_json::Value> = entries
                .iter()
                .map(|(k, v)| (k.clone(), render_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        RenderValue::Bind(name) => serde_json::Value::String(name.clone()),
        RenderValue::InterpolatedStr(parts) => {
            let s: String = parts
                .iter()
                .map(|p| match p {
                    TextPart::Literal(l) => l.clone(),
                    TextPart::StateRef(name) => format!("{{{}}}", name),
                })
                .collect();
            serde_json::Value::String(s)
        }
    }
}

/// Execute a SQL query using the DATABASE_URL environment variable.
/// Returns rows as a RenderValue::List of RenderValue::Object entries.
fn execute_sql(
    query: &str,
    param_exprs: &[IrExpression],
    eval_state: &HashMap<String, RenderValue>,
) -> RenderValue {
    #[cfg(feature = "database")]
    {
        use postgres::types::ToSql;
        use postgres::{Client, NoTls};

        let db_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                eprintln!("[naze] DATABASE_URL not set — cannot execute SQL");
                return RenderValue::List(vec![]);
            }
        };

        let mut client = match Client::connect(&db_url, NoTls) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[naze] database connection failed: {}", e);
                return RenderValue::List(vec![]);
            }
        };

        // Evaluate parameter expressions against current state
        let param_values: Vec<RenderValue> = param_exprs
            .iter()
            .map(|e| crate::exec::evaluate_expr(e, eval_state))
            .collect();

        // Convert RenderValues to postgres-compatible params
        let boxed_params: Vec<Box<dyn ToSql + Sync>> = param_values
            .iter()
            .map(|v| -> Box<dyn ToSql + Sync> {
                match v {
                    RenderValue::Str(s) => Box::new(s.clone()),
                    RenderValue::Num(n, _) => {
                        if n.fract() == 0.0 {
                            Box::new(*n as i64)
                        } else {
                            Box::new(*n)
                        }
                    }
                    RenderValue::Bool(b) => Box::new(*b),
                    _ => Box::new(String::new()),
                }
            })
            .collect();
        let params_ref: Vec<&(dyn ToSql + Sync)> = boxed_params.iter().map(|b| &**b).collect();

        match client.query(query, &params_ref) {
            Ok(rows) => {
                let results: Vec<RenderValue> = rows
                    .iter()
                    .map(|row| {
                        let cols = row.columns();
                        let entries: Vec<(String, RenderValue)> = cols
                            .iter()
                            .enumerate()
                            .map(|(i, col)| {
                                let val = if let Ok(s) = row.try_get::<_, String>(i) {
                                    RenderValue::Str(s)
                                } else if let Ok(n) = row.try_get::<_, i64>(i) {
                                    RenderValue::Num(n as f64, None)
                                } else if let Ok(n) = row.try_get::<_, i32>(i) {
                                    RenderValue::Num(n as f64, None)
                                } else if let Ok(n) = row.try_get::<_, f64>(i) {
                                    RenderValue::Num(n, None)
                                } else if let Ok(b) = row.try_get::<_, bool>(i) {
                                    RenderValue::Bool(b)
                                } else {
                                    RenderValue::Str(String::new())
                                };
                                (col.name().to_string(), val)
                            })
                            .collect();
                        RenderValue::Object(entries)
                    })
                    .collect();
                RenderValue::List(results)
            }
            Err(e) => {
                eprintln!("[naze] SQL error: {}", e);
                RenderValue::List(vec![])
            }
        }
    }
    #[cfg(not(feature = "database"))]
    {
        let _ = (query, param_exprs, eval_state);
        eprintln!("[naze] SQL support requires the 'database' feature: cargo build -p nazec --features database");
        RenderValue::List(vec![])
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use naze_ir::{IrExpression, IrServerBody};

    #[test]
    fn test_json_to_render_value_primitives() {
        assert_eq!(
            json_to_render_value(&serde_json::json!(42.0)),
            RenderValue::Num(42.0, None)
        );
        assert_eq!(
            json_to_render_value(&serde_json::json!("hello")),
            RenderValue::Str("hello".to_string())
        );
        assert_eq!(
            json_to_render_value(&serde_json::json!(true)),
            RenderValue::Bool(true)
        );
        assert_eq!(
            json_to_render_value(&serde_json::json!(null)),
            RenderValue::Str(String::new())
        );
    }

    #[test]
    fn test_json_to_render_value_nested() {
        let json = serde_json::json!([1.0, "two", false]);
        let val = json_to_render_value(&json);
        match val {
            RenderValue::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], RenderValue::Num(1.0, None));
                assert_eq!(items[1], RenderValue::Str("two".to_string()));
                assert_eq!(items[2], RenderValue::Bool(false));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_render_value_to_json_roundtrip() {
        let val = RenderValue::Num(3.14, None);
        let json = render_value_to_json(&val);
        assert_eq!(json, serde_json::json!(3.14));

        let val = RenderValue::Str("test".to_string());
        let json = render_value_to_json(&val);
        assert_eq!(json, serde_json::json!("test"));

        let val = RenderValue::Bool(true);
        let json = render_value_to_json(&val);
        assert_eq!(json, serde_json::json!(true));
    }

    #[test]
    fn test_evaluate_server_fn_simple() {
        // Server function that returns a string literal
        let func = ServerFuncDecl {
            name: "greet".to_string(),
            params: vec!["name".to_string()],
            body: IrServerBody {
                lets: vec![],
                result: IrExpression::Str("hello".to_string()),
            },
        };
        let result = evaluate_server_fn(&func, &[serde_json::json!("world")]);
        assert_eq!(result, serde_json::json!("hello"));
    }

    #[test]
    fn test_evaluate_server_fn_numeric() {
        // Server function that returns a number
        let func = ServerFuncDecl {
            name: "answer".to_string(),
            params: vec![],
            body: IrServerBody {
                lets: vec![],
                result: IrExpression::Num(42.0),
            },
        };
        let result = evaluate_server_fn(&func, &[]);
        assert_eq!(result, serde_json::json!(42.0));
    }

    #[test]
    fn test_evaluate_server_fn_with_let() {
        // Server function with a let binding
        let func = ServerFuncDecl {
            name: "calc".to_string(),
            params: vec![],
            body: IrServerBody {
                lets: vec![("x".to_string(), IrServerStep::Expr(IrExpression::Num(10.0)))],
                result: IrExpression::StateRef("x".to_string()),
            },
        };
        let result = evaluate_server_fn(&func, &[]);
        assert_eq!(result, serde_json::json!(10.0));
    }
}
