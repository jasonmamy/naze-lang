use naze_compiler::resolve::{self, ResolvedDep, ResolvedProject};
use naze_parser::ast::{DataSource, FuncParam, GuardCheckAst, ModelField, Node, Param, Type};
use serde::Serialize;
use std::path::Path;

use crate::manifest::Manifest;

// ─── Output types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ProjectContext {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub components: Vec<ComponentInterface>,
    pub server_functions: Vec<ServerFnInterface>,
    pub state: Vec<StateVar>,
    pub data_sources: Vec<DataSourceDef>,
    pub pages: Vec<Route>,
    pub guards: Vec<GuardDef>,
    pub theme_tokens: ThemeTokens,
    pub env_vars: Vec<String>,
    pub models: Vec<ModelInterface>,
    pub prompts: Vec<PromptInterface>,
}

#[derive(Serialize)]
pub struct ComponentInterface {
    pub name: String,
    pub import_path: String,
    pub params: Vec<ParamDef>,
}

#[derive(Serialize)]
pub struct ParamDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub has_default: bool,
}

#[derive(Serialize)]
pub struct ServerFnInterface {
    pub name: String,
    pub params: Vec<FuncParamDef>,
}

#[derive(Serialize)]
pub struct FuncParamDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Serialize)]
pub struct StateVar {
    pub name: String,
    pub shared: bool,
}

#[derive(Serialize)]
pub struct DataSourceDef {
    pub name: String,
    pub url: String,
    pub source_type: String,
}

#[derive(Serialize)]
pub struct Route {
    pub path: String,
    pub params: Vec<String>,
    pub guard: Option<String>,
}

#[derive(Serialize)]
pub struct GuardDef {
    pub name: String,
    pub checks: Vec<GuardCheckDef>,
}

#[derive(Serialize)]
pub struct GuardCheckDef {
    pub redirect: String,
}

#[derive(Serialize)]
pub struct ThemeTokens {
    pub colors: Vec<String>,
    pub spacing: Vec<String>,
}

#[derive(Serialize)]
pub struct ModelInterface {
    pub name: String,
    pub fields: Vec<ModelFieldDef>,
}

#[derive(Serialize)]
pub struct ModelFieldDef {
    pub name: String,
    pub field_type: String,
    pub constraints: Vec<String>,
}

#[derive(Serialize)]
pub struct PromptInterface {
    pub name: String,
    pub provider: String,
}

// ─── Extraction ──────────────────────────────────────────────────────────────

fn type_name(ty: &Type) -> &'static str {
    match ty {
        Type::Text => "text",
        Type::Number => "number",
        Type::Bool => "bool",
        Type::Color => "color",
    }
}

fn extract_param(p: &Param) -> ParamDef {
    ParamDef {
        name: p.name.clone(),
        ty: type_name(&p.ty).to_string(),
        has_default: p.default.is_some(),
    }
}

fn extract_func_param(p: &FuncParam) -> FuncParamDef {
    FuncParamDef {
        name: p.name.clone(),
        ty: type_name(&p.ty).to_string(),
    }
}

fn data_source_name(ds: &DataSource) -> &'static str {
    match ds {
        DataSource::Fetch => "fetch",
        DataSource::Stream => "stream",
        DataSource::JsCall => "js",
        DataSource::Device => "device",
    }
}

fn extract_guard_check(c: &GuardCheckAst) -> GuardCheckDef {
    GuardCheckDef {
        redirect: c.redirect.clone(),
    }
}

fn extract_model_field(f: &ModelField) -> ModelFieldDef {
    ModelFieldDef {
        name: f.name.clone(),
        field_type: f.field_type.clone(),
        constraints: f.constraints.clone(),
    }
}

/// Walk a list of AST nodes and collect context information.
fn collect_from_nodes(
    nodes: &[Node],
    ctx: &mut ProjectContext,
) {
    for node in nodes {
        match node {
            Node::ServerFunction { name, params, .. } => {
                ctx.server_functions.push(ServerFnInterface {
                    name: name.clone(),
                    params: params.iter().map(extract_func_param).collect(),
                });
            }
            Node::State { name, shared, .. } => {
                ctx.state.push(StateVar {
                    name: name.clone(),
                    shared: *shared,
                });
            }
            Node::Data { name, url, source, .. } => {
                ctx.data_sources.push(DataSourceDef {
                    name: name.clone(),
                    url: url.clone(),
                    source_type: data_source_name(source).to_string(),
                });
            }
            Node::Page { path, params, guard, children, .. } => {
                ctx.pages.push(Route {
                    path: path.clone(),
                    params: params.clone(),
                    guard: guard.clone(),
                });
                // Recurse into page children for nested state/data
                collect_from_nodes(children, ctx);
            }
            Node::Guard { name, checks, .. } => {
                ctx.guards.push(GuardDef {
                    name: name.clone(),
                    checks: checks.iter().map(extract_guard_check).collect(),
                });
            }
            Node::Model { name, fields, .. } => {
                ctx.models.push(ModelInterface {
                    name: name.clone(),
                    fields: fields.iter().map(extract_model_field).collect(),
                });
            }
            Node::Prompt { name, provider, .. } => {
                ctx.prompts.push(PromptInterface {
                    name: name.clone(),
                    provider: provider.clone(),
                });
            }
            Node::App { children, .. } => {
                collect_from_nodes(children, ctx);
            }
            Node::Component { children, .. } => {
                collect_from_nodes(children, ctx);
            }
            _ => {}
        }
    }
}

pub fn extract_context(project: &ResolvedProject, manifest: &Manifest) -> ProjectContext {
    let mut ctx = ProjectContext {
        name: manifest.app.name.clone(),
        version: manifest.app.version.clone(),
        entry: manifest.build.entry.clone(),
        components: Vec::new(),
        server_functions: Vec::new(),
        state: Vec::new(),
        data_sources: Vec::new(),
        pages: Vec::new(),
        guards: Vec::new(),
        theme_tokens: ThemeTokens {
            colors: Vec::new(),
            spacing: Vec::new(),
        },
        env_vars: Vec::new(),
        models: Vec::new(),
        prompts: Vec::new(),
    };

    // Components from resolved project
    for comp in project.components.values() {
        ctx.components.push(ComponentInterface {
            name: comp.name.clone(),
            import_path: comp.import_path.clone(),
            params: comp.params.iter().map(extract_param).collect(),
        });
    }
    // Sort for deterministic output
    ctx.components.sort_by(|a, b| a.name.cmp(&b.name));

    // Walk entry file nodes
    collect_from_nodes(&project.entry.nodes, &mut ctx);

    // Theme tokens from first (active) theme
    if let Some(theme) = project.themes.first() {
        let mut colors: Vec<String> = theme.colors.keys().cloned().collect();
        colors.sort();
        let mut spacing: Vec<String> = theme.spacing.keys().cloned().collect();
        spacing.sort();
        ctx.theme_tokens = ThemeTokens { colors, spacing };
    }

    // Env vars from manifest
    let mut env_keys: Vec<String> = manifest.env.keys().cloned().collect();
    env_keys.sort();
    ctx.env_vars = env_keys;

    ctx
}

// ─── CLI entry point ─────────────────────────────────────────────────────────

pub fn run(manifest: &Manifest, deps: &[ResolvedDep]) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = Path::new(".");
    let entry = &manifest.build.entry;

    let project = resolve::resolve(project_dir, entry, deps);

    // Report resolve errors to stderr but continue (context is still useful)
    if !project.errors.is_empty() {
        for err in &project.errors {
            eprintln!("warning: {err}");
        }
    }

    let ctx = extract_context(&project, manifest);
    println!("{}", serde_json::to_string_pretty(&ctx)?);
    Ok(())
}
