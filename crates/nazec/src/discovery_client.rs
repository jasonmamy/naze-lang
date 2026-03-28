use std::path::Path;

use crate::manifest::Manifest;

/// Resolve the discovery server URL from flag → env → default.
pub fn resolve_url(flag: Option<&str>) -> String {
    if let Some(url) = flag {
        return url.to_string();
    }
    if let Ok(url) = std::env::var("NAZE_DISCOVERY_URL") {
        return url;
    }
    "http://localhost:8889".to_string()
}

/// Resolve the API key from flag → env.
pub fn resolve_api_key(flag: Option<&str>) -> Option<String> {
    if let Some(key) = flag {
        return Some(key.to_string());
    }
    std::env::var("NAZE_DISCOVERY_KEY").ok()
}

/// Generate a manifest JSON from ProjectContext for announcement.
pub fn context_to_manifest(manifest: &Manifest, ctx: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "name": ctx.get("name").and_then(|n| n.as_str()).unwrap_or(&manifest.app.name),
        "version": ctx.get("version").and_then(|v| v.as_str()).unwrap_or(&manifest.app.version),
        "state": build_state_schema(ctx),
        "server_functions": build_server_functions(ctx),
        "actions": build_actions(ctx),
        "data_sources": build_data_sources(ctx),
        "pages": build_pages(ctx),
    })
}

fn build_state_schema(ctx: &serde_json::Value) -> serde_json::Value {
    let mut state = serde_json::Map::new();
    if let Some(vars) = ctx.get("state").and_then(|s| s.as_array()) {
        for v in vars {
            let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
            state.insert(
                name.to_string(),
                serde_json::json!({"type": "text", "shared": v.get("shared").and_then(|s| s.as_bool()).unwrap_or(false)}),
            );
        }
    }
    serde_json::Value::Object(state)
}

fn build_server_functions(ctx: &serde_json::Value) -> Vec<String> {
    ctx.get("server_functions")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| f.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn build_actions(_ctx: &serde_json::Value) -> Vec<String> {
    // Actions are not directly available in ProjectContext,
    // will be populated when manifest generation is implemented
    Vec::new()
}

fn build_data_sources(ctx: &serde_json::Value) -> Vec<serde_json::Value> {
    ctx.get("data_sources")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .map(|ds| {
                    serde_json::json!({
                        "name": ds.get("name").and_then(|n| n.as_str()).unwrap_or(""),
                        "url": ds.get("url").and_then(|u| u.as_str()).unwrap_or(""),
                        "type": ds.get("source_type").and_then(|t| t.as_str()).unwrap_or("fetch"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn build_pages(ctx: &serde_json::Value) -> Vec<serde_json::Value> {
    ctx.get("pages")
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .map(|pg| {
                    serde_json::json!({
                        "path": pg.get("path").and_then(|p| p.as_str()).unwrap_or("/"),
                        "params": pg.get("params"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Announce the current project to a discovery server.
pub fn announce(
    server_url: &str,
    domain: &str,
    visibility: &str,
    api_key: Option<&str>,
    manifest: &Manifest,
    deps: &[naze_compiler::resolve::ResolvedDep],
) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = Path::new(".");
    let entry = &manifest.build.entry;

    let project = naze_compiler::resolve::resolve(project_dir, entry, deps);
    if !project.errors.is_empty() {
        for err in &project.errors {
            eprintln!("warning: {err}");
        }
    }

    let ctx = crate::context::extract_context(&project, manifest);
    let ctx_json = serde_json::to_value(&ctx)?;
    let discovery_manifest = context_to_manifest(manifest, &ctx_json);

    let body = serde_json::json!({
        "domain": domain,
        "manifest": discovery_manifest,
        "visibility": visibility,
    });

    let client = reqwest::blocking::Client::new();
    let mut req = client
        .post(format!("{}/api/v1/discovery/services", server_url))
        .header("content-type", "application/json")
        .body(serde_json::to_string(&body)?);

    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
    }

    let resp = req.send()?;
    let status = resp.status();
    let body: serde_json::Value = resp.json()?;

    if status.is_success() {
        eprintln!("Announced to {server_url}");
        eprintln!("  Domain: {domain}");
        eprintln!("  Name: {}", body["name"].as_str().unwrap_or("?"));
        eprintln!("  Version: {}", body["version"].as_str().unwrap_or("?"));
        eprintln!(
            "  Capabilities indexed: {}",
            body["capabilities_indexed"].as_u64().unwrap_or(0)
        );
        if let Some(scores) = body["trust_scores"].as_object() {
            eprintln!("  Trust scores:");
            for (profile, score) in scores {
                eprintln!("    {profile}: {:.2}", score.as_f64().unwrap_or(0.0));
            }
        }
    } else {
        eprintln!(
            "Failed to announce: {} — {}",
            status,
            body["error"].as_str().unwrap_or("unknown error")
        );
    }

    Ok(())
}

/// Parse shorthand query syntax into capability matchers.
/// Format: "fn:order,state:price:number,action:click"
fn parse_query(query: &str) -> Vec<serde_json::Value> {
    query
        .split(',')
        .filter_map(|part| {
            let segments: Vec<&str> = part.trim().split(':').collect();
            if segments.is_empty() {
                return None;
            }
            let (kind, name, value_type) = match segments[0] {
                "fn" => ("server_function", segments.get(1).copied(), None),
                "state" => (
                    "state_field",
                    segments.get(1).copied(),
                    segments.get(2).copied(),
                ),
                "action" => ("action", segments.get(1).copied(), None),
                "data" => ("data_source", segments.get(1).copied(), None),
                "model" => ("model_field", segments.get(1).copied(), None),
                _ => return None,
            };

            let mut matcher = serde_json::json!({"kind": kind});
            if let Some(n) = name {
                if n.contains('%') {
                    matcher["name_like"] = serde_json::Value::String(n.to_string());
                } else {
                    matcher["name"] = serde_json::Value::String(n.to_string());
                }
            }
            if let Some(vt) = value_type {
                matcher["value_type"] = serde_json::Value::String(vt.to_string());
            }
            Some(matcher)
        })
        .collect()
}

/// Discover services matching a capability query.
pub fn discover(
    server_url: &str,
    query: &str,
    profile: &str,
    min_trust: Option<f64>,
    limit: u32,
    api_key: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let matchers = parse_query(query);
    if matchers.is_empty() {
        eprintln!("No valid matchers in query: {query}");
        eprintln!("Format: fn:order,state:price:number,action:click");
        return Ok(());
    }

    let body = serde_json::json!({
        "require": matchers,
        "trust_profile": profile,
        "min_trust": min_trust,
        "limit": limit,
    });

    let client = reqwest::blocking::Client::new();
    let mut req = client
        .post(format!("{}/api/v1/discovery/search", server_url))
        .header("content-type", "application/json")
        .body(serde_json::to_string(&body)?);

    if let Some(key) = api_key {
        req = req.header("x-api-key", key);
    }

    let resp = req.send()?;
    let status = resp.status();
    let body: serde_json::Value = resp.json()?;

    if status.is_success() {
        let total = body["total"].as_u64().unwrap_or(0);
        eprintln!("Found {total} matching services (profile: {profile}):\n");

        if let Some(results) = body["results"].as_array() {
            for (i, r) in results.iter().enumerate() {
                let score = r["trust_score"].as_f64().unwrap_or(0.0);
                let score_color = if score >= 0.8 {
                    "high"
                } else if score >= 0.5 {
                    "medium"
                } else {
                    "low"
                };
                println!(
                    "{}. {} ({}) v{} — trust: {:.2} [{}]",
                    i + 1,
                    r["name"].as_str().unwrap_or("?"),
                    r["domain"].as_str().unwrap_or("?"),
                    r["version"].as_str().unwrap_or("?"),
                    score,
                    score_color,
                );
                if let Some(caps) = r["matched_capabilities"].as_array() {
                    for cap in caps {
                        print!(
                            "   {}:{}",
                            cap["kind"].as_str().unwrap_or(""),
                            cap["name"].as_str().unwrap_or(""),
                        );
                        if let Some(vt) = cap["value_type"].as_str() {
                            print!(":{vt}");
                        }
                        println!();
                    }
                }
                println!();
            }
        }
    } else {
        eprintln!(
            "Search failed: {} — {}",
            status,
            body["error"].as_str().unwrap_or("unknown error")
        );
    }

    Ok(())
}
