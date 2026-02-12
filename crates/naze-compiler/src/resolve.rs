use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use naze_parser::ast::{Node, Param, Prop, Span, Unit, Value};

use crate::error::{CompileError, Severity};

/// Theme tokens for consistent styling.
/// Tokens are resolved at compile time and inlined as values.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name: "default" for unnamed themes, or a user-specified name like "dark".
    pub name: String,
    /// Color tokens: "primary" -> 0x2563eb
    pub colors: HashMap<String, u32>,
    /// Spacing tokens: "md" -> 16.0 (in pixels)
    pub spacing: HashMap<String, f64>,
}

impl Default for Theme {
    fn default() -> Self {
        default_theme()
    }
}

/// Built-in default theme with common design tokens.
pub fn default_theme() -> Theme {
    Theme {
        name: "default".to_string(),
        colors: [
            ("primary", 0x2563eb),
            ("secondary", 0x64748b),
            ("success", 0x22c55e),
            ("warning", 0xf59e0b),
            ("danger", 0xdc2626),
            ("background", 0xffffff),
            ("foreground", 0x0f172a),
            ("muted", 0x94a3b8),
            ("border", 0xe2e8f0),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect(),
        spacing: [
            ("xs", 4.0),
            ("sm", 8.0),
            ("md", 16.0),
            ("lg", 24.0),
            ("xl", 32.0),
            ("xxl", 48.0),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect(),
    }
}

/// A parsed source file with its AST.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub nodes: Vec<Node>,
}

/// A resolved component definition.
#[derive(Debug, Clone)]
pub struct ComponentDef {
    /// The import path used to reference this component (e.g., "components/card").
    pub import_path: String,
    /// Short name (e.g., "card").
    pub name: String,
    /// Declared parameters.
    pub params: Vec<Param>,
    /// Component body nodes.
    pub children: Vec<Node>,
    /// Where it was defined.
    pub span: Span,
    /// Source file path.
    pub file: PathBuf,
}

/// A resolved external dependency (passed from nazec dep resolver to compiler).
#[derive(Debug, Clone)]
pub struct ResolvedDep {
    /// Package name, e.g., "@naze/ui-kit" or "local-lib"
    pub name: String,
    /// Absolute path to the dependency's source directory
    pub local_path: PathBuf,
}

/// A resolved WASM module import.
#[derive(Debug, Clone)]
pub struct ResolvedImport {
    /// Local binding name (e.g., "crypto")
    pub name: String,
    /// Absolute path to the .wasm file
    pub wasm_path: PathBuf,
    /// Original source string from the import statement
    pub source: String,
}

/// The result of resolving a project.
#[derive(Debug)]
pub struct ResolvedProject {
    /// The entry file's AST (the app).
    pub entry: SourceFile,
    /// All component definitions, keyed by import path.
    pub components: HashMap<String, ComponentDef>,
    /// Theme definitions (first = default/active, rest = named alternatives).
    pub themes: Vec<Theme>,
    /// WASM module imports.
    pub imports: Vec<ResolvedImport>,
    /// Errors encountered during resolution.
    pub errors: Vec<CompileError>,
}

/// Cache for incremental compilation: stores parsed ASTs keyed by file content hash.
/// On hot-reload, unchanged files reuse their cached AST instead of re-parsing.
#[derive(Debug, Default)]
pub struct BuildCache {
    /// Maps file path → (content hash, cached AST nodes).
    pub file_cache: HashMap<PathBuf, (u64, Vec<Node>)>,
}

impl BuildCache {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Compute a simple hash of file contents for cache invalidation.
fn hash_content(content: &[u8]) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Built-in element names that are not component invocations.
const BUILTIN_ELEMENTS: &[&str] = &[
    "row",
    "column",
    "stack",
    "grid",
    "spacer",
    "rect",
    "text",
    "heading",
    "container",
    "image",
    "checkbox",
    "radio",
    "input",
    "select",
    "option",
    "scroll",
];

/// Resolve all imports for a project rooted at `project_dir` with entry file `entry`.
/// `deps` contains resolved external dependencies whose .naze files will be discovered.
pub fn resolve(project_dir: &Path, entry: &str, deps: &[ResolvedDep]) -> ResolvedProject {
    let mut errors = Vec::new();

    // 1. Parse the entry file
    let entry_path = project_dir.join(entry);
    let entry_source = match std::fs::read_to_string(&entry_path) {
        Ok(s) => s,
        Err(e) => {
            errors.push(CompileError {
                message: format!("cannot read entry file '{}': {}", entry, e),
                file: entry.to_string(),
                line: 0,
                column: 0,
                severity: Severity::Error,
            });
            return ResolvedProject {
                entry: SourceFile {
                    path: entry_path,
                    nodes: vec![],
                },
                components: HashMap::new(),
                themes: vec![default_theme()],
                imports: vec![],
                errors,
            };
        }
    };

    let entry_nodes = match naze_parser::parse(&entry_source, entry) {
        Ok(nodes) => nodes,
        Err(e) => {
            errors.push(CompileError {
                message: e.message.clone(),
                file: e.file.clone(),
                line: e.line,
                column: e.column,
                severity: Severity::Error,
            });
            return ResolvedProject {
                entry: SourceFile {
                    path: entry_path,
                    nodes: vec![],
                },
                components: HashMap::new(),
                themes: vec![default_theme()],
                imports: vec![],
                errors,
            };
        }
    };

    let entry_file = SourceFile {
        path: entry_path,
        nodes: entry_nodes,
    };

    // 2. Discover all .naze files in the project directory (recursively)
    let mut all_files: HashMap<String, SourceFile> = HashMap::new();
    discover_naze_files(project_dir, project_dir, &mut all_files, &mut errors);

    // 2b. Discover .naze files from external dependencies
    for dep in deps {
        discover_dep_files(&dep.name, &dep.local_path, &mut all_files, &mut errors);
    }

    // 3. Extract component definitions from all discovered files
    let mut components: HashMap<String, ComponentDef> = HashMap::new();
    for (import_path, source_file) in &all_files {
        for node in &source_file.nodes {
            let (name, params, children, span) = match node {
                Node::Component {
                    name,
                    params,
                    children,
                    span,
                } => (name, params.clone(), children, span),
                Node::Template {
                    name,
                    children,
                    span,
                    ..
                } => (name, vec![], children, span),
                _ => continue,
            };

            let def = ComponentDef {
                import_path: import_path.clone(),
                name: name.clone(),
                params,
                children: children.clone(),
                span: span.clone(),
                file: source_file.path.clone(),
            };

            if let Some(existing) = components.get(import_path) {
                errors.push(CompileError {
                    message: format!(
                        "duplicate component '{}': already defined in {}",
                        import_path,
                        existing.file.display()
                    ),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            } else {
                components.insert(import_path.clone(), def);
            }
        }
    }

    // Also extract components and templates defined inline in the entry file
    for node in &entry_file.nodes {
        let (name, params, children, span) = match node {
            Node::Component {
                name,
                params,
                children,
                span,
            } => (name, params.clone(), children, span),
            Node::Template {
                name,
                children,
                span,
                ..
            } => (name, vec![], children, span),
            _ => continue,
        };

        let import_path = name.clone();
        let def = ComponentDef {
            import_path: import_path.clone(),
            name: name.clone(),
            params,
            children: children.clone(),
            span: span.clone(),
            file: entry_file.path.clone(),
        };
        components.entry(import_path).or_insert(def);
    }

    // 3b. Register built-in templates (user definitions take precedence)
    for def in builtin_templates() {
        components.entry(def.name.clone()).or_insert(def);
    }

    // 4. Resolve use statements in the entry file
    let imports = collect_use_paths(&entry_file.nodes);
    for (use_path, span) in &imports {
        if !components.contains_key(use_path) {
            errors.push(CompileError {
                message: format!("unresolved import '{}'", use_path),
                file: span.file.clone(),
                line: span.line,
                column: span.col,
                severity: Severity::Error,
            });
        }
    }

    // 5. Check for element names that look like component invocations but aren't imported
    let imported_names: HashSet<&str> = imports
        .iter()
        .filter_map(|(path, _)| path.split('/').last())
        .collect();

    check_unresolved_elements(&entry_file.nodes, &imported_names, &components, &mut errors);

    // 6. Check for circular dependencies among component files
    check_circular_deps(&all_files, &mut errors);

    // 7. Load themes (from theme.naze, entry file inline themes, or use default)
    let themes = load_themes(project_dir, &entry_file.nodes, &mut errors);

    // 8. Collect WASM module imports from entry file
    let wasm_imports = collect_wasm_imports(&entry_file.nodes, project_dir, deps, &mut errors);

    ResolvedProject {
        entry: entry_file,
        components,
        themes,
        imports: wasm_imports,
        errors,
    }
}

/// Incremental resolve: reuses cached ASTs for unchanged files.
/// On first call, behaves identically to `resolve()`. On subsequent calls,
/// skips re-parsing files whose content hash hasn't changed.
pub fn resolve_incremental(
    project_dir: &Path,
    entry: &str,
    cache: &mut BuildCache,
    deps: &[ResolvedDep],
) -> ResolvedProject {
    let mut errors = Vec::new();

    // 1. Parse the entry file (always re-parse — it's the most likely to change)
    let entry_path = project_dir.join(entry);
    let entry_source = match std::fs::read_to_string(&entry_path) {
        Ok(s) => s,
        Err(e) => {
            errors.push(CompileError {
                message: format!("cannot read entry file '{}': {}", entry, e),
                file: entry.to_string(),
                line: 0,
                column: 0,
                severity: Severity::Error,
            });
            return ResolvedProject {
                entry: SourceFile {
                    path: entry_path,
                    nodes: vec![],
                },
                components: HashMap::new(),
                themes: vec![default_theme()],
                imports: vec![],
                errors,
            };
        }
    };

    let entry_hash = hash_content(entry_source.as_bytes());
    let entry_nodes = if let Some((cached_hash, cached_nodes)) = cache.file_cache.get(&entry_path) {
        if *cached_hash == entry_hash {
            cached_nodes.clone()
        } else {
            match naze_parser::parse(&entry_source, entry) {
                Ok(nodes) => {
                    cache
                        .file_cache
                        .insert(entry_path.clone(), (entry_hash, nodes.clone()));
                    nodes
                }
                Err(e) => {
                    errors.push(CompileError {
                        message: e.message.clone(),
                        file: e.file.clone(),
                        line: e.line,
                        column: e.column,
                        severity: Severity::Error,
                    });
                    return ResolvedProject {
                        entry: SourceFile {
                            path: entry_path,
                            nodes: vec![],
                        },
                        components: HashMap::new(),
                        themes: vec![default_theme()],
                        imports: vec![],
                        errors,
                    };
                }
            }
        }
    } else {
        match naze_parser::parse(&entry_source, entry) {
            Ok(nodes) => {
                cache
                    .file_cache
                    .insert(entry_path.clone(), (entry_hash, nodes.clone()));
                nodes
            }
            Err(e) => {
                errors.push(CompileError {
                    message: e.message.clone(),
                    file: e.file.clone(),
                    line: e.line,
                    column: e.column,
                    severity: Severity::Error,
                });
                return ResolvedProject {
                    entry: SourceFile {
                        path: entry_path,
                        nodes: vec![],
                    },
                    components: HashMap::new(),
                    themes: vec![default_theme()],
                    imports: vec![],
                    errors,
                };
            }
        }
    };

    let entry_file = SourceFile {
        path: entry_path,
        nodes: entry_nodes,
    };

    // 2. Discover .naze files with caching
    let mut all_files: HashMap<String, SourceFile> = HashMap::new();
    discover_naze_files_cached(project_dir, project_dir, &mut all_files, &mut errors, cache);

    // 2b. Discover .naze files from external dependencies
    for dep in deps {
        discover_dep_files(&dep.name, &dep.local_path, &mut all_files, &mut errors);
    }

    // 3-7: Same as resolve() — extract components, check imports, etc.
    let mut components: HashMap<String, ComponentDef> = HashMap::new();
    for (import_path, source_file) in &all_files {
        for node in &source_file.nodes {
            let (name, params, children, span) = match node {
                Node::Component {
                    name,
                    params,
                    children,
                    span,
                } => (name, params.clone(), children, span),
                Node::Template {
                    name,
                    children,
                    span,
                    ..
                } => (name, vec![], children, span),
                _ => continue,
            };

            let def = ComponentDef {
                import_path: import_path.clone(),
                name: name.clone(),
                params,
                children: children.clone(),
                span: span.clone(),
                file: source_file.path.clone(),
            };

            if let Some(existing) = components.get(import_path) {
                errors.push(CompileError {
                    message: format!(
                        "duplicate component '{}': already defined in {}",
                        import_path,
                        existing.file.display()
                    ),
                    file: span.file.clone(),
                    line: span.line,
                    column: span.col,
                    severity: Severity::Error,
                });
            } else {
                components.insert(import_path.clone(), def);
            }
        }
    }

    for node in &entry_file.nodes {
        let (name, params, children, span) = match node {
            Node::Component {
                name,
                params,
                children,
                span,
            } => (name, params.clone(), children, span),
            Node::Template {
                name,
                children,
                span,
                ..
            } => (name, vec![], children, span),
            _ => continue,
        };

        let import_path = name.clone();
        let def = ComponentDef {
            import_path: import_path.clone(),
            name: name.clone(),
            params,
            children: children.clone(),
            span: span.clone(),
            file: entry_file.path.clone(),
        };
        components.entry(import_path).or_insert(def);
    }

    for def in builtin_templates() {
        components.entry(def.name.clone()).or_insert(def);
    }

    let imports = collect_use_paths(&entry_file.nodes);
    for (use_path, span) in &imports {
        if !components.contains_key(use_path) {
            errors.push(CompileError {
                message: format!("unresolved import '{}'", use_path),
                file: span.file.clone(),
                line: span.line,
                column: span.col,
                severity: Severity::Error,
            });
        }
    }

    let imported_names: HashSet<&str> = imports
        .iter()
        .filter_map(|(path, _)| path.split('/').last())
        .collect();

    check_unresolved_elements(&entry_file.nodes, &imported_names, &components, &mut errors);
    check_circular_deps(&all_files, &mut errors);

    let themes = load_themes(project_dir, &entry_file.nodes, &mut errors);

    let wasm_imports = collect_wasm_imports(&entry_file.nodes, project_dir, deps, &mut errors);

    ResolvedProject {
        entry: entry_file,
        components,
        themes,
        imports: wasm_imports,
        errors,
    }
}

/// Collect WASM module imports from entry file nodes.
/// Resolves import sources to absolute .wasm file paths.
fn collect_wasm_imports(
    nodes: &[Node],
    project_dir: &Path,
    deps: &[ResolvedDep],
    errors: &mut Vec<CompileError>,
) -> Vec<ResolvedImport> {
    let mut imports = Vec::new();
    for node in nodes {
        if let Node::Import { name, source, span } = node {
            match resolve_wasm_path(project_dir, source, deps) {
                Ok(wasm_path) => {
                    imports.push(ResolvedImport {
                        name: name.clone(),
                        wasm_path,
                        source: source.clone(),
                    });
                }
                Err(msg) => {
                    errors.push(CompileError {
                        message: msg,
                        file: span.file.clone(),
                        line: span.line,
                        column: span.col,
                        severity: Severity::Error,
                    });
                }
            }
        }
    }
    imports
}

/// Resolve an import source string to an absolute .wasm file path.
///
/// Supported source formats:
/// - `"./path/to/file.wasm"` — relative to project dir
/// - `"@org/package"` — look for .wasm in dependency's resolved dir
/// - `"@org/package/module.wasm"` — specific file in dependency dir
fn resolve_wasm_path(
    project_dir: &Path,
    source: &str,
    deps: &[ResolvedDep],
) -> Result<PathBuf, String> {
    if source.starts_with("./") || source.starts_with("../") {
        // Local file path
        let path = project_dir.join(source);
        if path.exists() {
            Ok(path.canonicalize().unwrap_or(path))
        } else {
            Err(format!(
                "WASM module not found: '{}' (resolved to '{}')",
                source,
                path.display()
            ))
        }
    } else if source.starts_with('@') {
        // Dependency-based import: @org/package or @org/package/module.wasm
        let parts: Vec<&str> = source.splitn(3, '/').collect();
        if parts.len() < 2 {
            return Err(format!(
                "invalid import source '{}': expected @org/package",
                source
            ));
        }
        let dep_name = format!("{}/{}", parts[0], parts[1]);
        let dep = deps.iter().find(|d| d.name == dep_name);
        let Some(dep) = dep else {
            return Err(format!(
                "import '{}' references unknown dependency '{}' (not in [dependencies])",
                source, dep_name
            ));
        };
        if parts.len() == 3 {
            // @org/package/module.wasm — specific file
            let path = dep.local_path.join(parts[2]);
            if path.exists() {
                Ok(path.canonicalize().unwrap_or(path))
            } else {
                Err(format!(
                    "WASM module '{}' not found in dependency '{}'",
                    parts[2], dep_name
                ))
            }
        } else {
            // @org/package — look for lib.wasm or <package>.wasm
            let lib_path = dep.local_path.join("lib.wasm");
            if lib_path.exists() {
                return Ok(lib_path.canonicalize().unwrap_or(lib_path));
            }
            let pkg_wasm = format!("{}.wasm", parts[1]);
            let pkg_path = dep.local_path.join(&pkg_wasm);
            if pkg_path.exists() {
                return Ok(pkg_path.canonicalize().unwrap_or(pkg_path));
            }
            Err(format!(
                "no .wasm file found in dependency '{}' (looked for lib.wasm, {})",
                dep_name, pkg_wasm
            ))
        }
    } else {
        Err(format!(
            "invalid import source '{}': must start with './' or '@'",
            source
        ))
    }
}

/// Load themes from theme.naze file + inline theme blocks in entry file.
/// Returns a Vec where the first element is the default/active theme.
/// Named themes can use `extends` to inherit from another theme.
fn load_themes(
    project_dir: &Path,
    entry_nodes: &[Node],
    errors: &mut Vec<CompileError>,
) -> Vec<Theme> {
    let mut raw_themes: Vec<RawTheme> = Vec::new();

    // 1. Load from theme.naze if present
    let theme_path = project_dir.join("theme.naze");
    if theme_path.exists() {
        let source = match std::fs::read_to_string(&theme_path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(CompileError {
                    message: format!("cannot read theme.naze: {}", e),
                    file: "theme.naze".to_string(),
                    line: 0,
                    column: 0,
                    severity: Severity::Warning,
                });
                String::new()
            }
        };

        if !source.is_empty() {
            match naze_parser::parse(&source, "theme.naze") {
                Ok(nodes) => {
                    collect_raw_themes(&nodes, &mut raw_themes);
                }
                Err(e) => {
                    errors.push(CompileError {
                        message: e.message,
                        file: e.file,
                        line: e.line,
                        column: e.column,
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    // 2. Collect inline theme blocks from entry file
    collect_raw_themes(entry_nodes, &mut raw_themes);

    // 3. If no themes found, return just the default
    if raw_themes.is_empty() {
        return vec![default_theme()];
    }

    // 4. Build a map of name -> index for topological sorting
    let name_to_idx: HashMap<String, usize> = raw_themes
        .iter()
        .enumerate()
        .map(|(i, t)| (t.name.clone(), i))
        .collect();

    // 5. Topological sort by extends chain (detect cycles)
    let mut resolved: Vec<Theme> = Vec::new();
    let mut resolved_names: HashSet<String> = HashSet::new();
    let mut visited: HashSet<String> = HashSet::new();

    // Iteratively resolve themes (max iterations = count to prevent infinite loops)
    let max_iterations = raw_themes.len() * 2;
    let mut iteration = 0;
    while resolved.len() < raw_themes.len() && iteration < max_iterations {
        iteration += 1;
        let mut made_progress = false;

        for raw in &raw_themes {
            if resolved_names.contains(&raw.name) {
                continue;
            }

            // Check if parent is resolved (or no parent needed)
            let parent_ready = match &raw.extends {
                None => true,
                Some(parent_name) => {
                    if !name_to_idx.contains_key(parent_name) && parent_name != "default" {
                        // Parent doesn't exist — report error if not already visited
                        if visited.insert(raw.name.clone()) {
                            errors.push(CompileError {
                                message: format!(
                                    "theme '{}' extends unknown theme '{}'",
                                    raw.name, parent_name
                                ),
                                file: "theme".to_string(),
                                line: 0,
                                column: 0,
                                severity: Severity::Error,
                            });
                        }
                        true // resolve anyway using default as base
                    } else {
                        resolved_names.contains(parent_name) || parent_name == "default"
                    }
                }
            };

            if !parent_ready {
                continue;
            }

            // Build theme starting from parent's resolved tokens
            let base = match &raw.extends {
                Some(parent_name) => {
                    resolved
                        .iter()
                        .find(|t| t.name == *parent_name)
                        .cloned()
                        .unwrap_or_else(default_theme)
                }
                None => default_theme(), // unnamed/default themes inherit built-in defaults
            };

            let mut theme = Theme {
                name: raw.name.clone(),
                colors: base.colors,
                spacing: base.spacing,
            };

            // Overlay this theme's own tokens
            for (name, color) in &raw.colors {
                theme.colors.insert(name.clone(), *color);
            }
            for (name, value) in &raw.spacing {
                theme.spacing.insert(name.clone(), *value);
            }

            resolved_names.insert(raw.name.clone());
            resolved.push(theme);
            made_progress = true;
        }

        if !made_progress {
            // Cycle detected — report and break
            for raw in &raw_themes {
                if !resolved_names.contains(&raw.name) {
                    errors.push(CompileError {
                        message: format!(
                            "circular theme inheritance involving '{}'",
                            raw.name
                        ),
                        file: "theme".to_string(),
                        line: 0,
                        column: 0,
                        severity: Severity::Error,
                    });
                    // Resolve with default as base anyway
                    let mut theme = default_theme();
                    theme.name = raw.name.clone();
                    for (name, color) in &raw.colors {
                        theme.colors.insert(name.clone(), *color);
                    }
                    for (name, value) in &raw.spacing {
                        theme.spacing.insert(name.clone(), *value);
                    }
                    resolved_names.insert(raw.name.clone());
                    resolved.push(theme);
                }
            }
            break;
        }
    }

    // Ensure "default" is first
    if let Some(default_idx) = resolved.iter().position(|t| t.name == "default") {
        if default_idx != 0 {
            let default = resolved.remove(default_idx);
            resolved.insert(0, default);
        }
    } else {
        // No "default" theme — prepend the built-in default
        resolved.insert(0, default_theme());
    }

    resolved
}

/// Raw theme data extracted from AST before inheritance resolution.
struct RawTheme {
    name: String,
    extends: Option<String>,
    colors: Vec<(String, u32)>,
    spacing: Vec<(String, f64)>,
}

/// Extract raw theme data from AST nodes.
fn collect_raw_themes(nodes: &[Node], raw_themes: &mut Vec<RawTheme>) {
    for node in nodes {
        if let Node::Theme {
            name,
            extends,
            colors,
            spacing,
            ..
        } = node
        {
            let theme_name = name.clone().unwrap_or_else(|| "default".to_string());
            raw_themes.push(RawTheme {
                name: theme_name,
                extends: extends.clone(),
                colors: colors.clone(),
                spacing: spacing.iter().map(|(n, v, _unit)| (n.clone(), *v)).collect(),
            });
        }
        // Also check inside app blocks for inline themes
        if let Node::App { children, .. } = node {
            collect_raw_themes(children, raw_themes);
        }
    }
}

/// Recursively discover .naze files under `dir`, building import paths relative to `root`.
fn discover_naze_files(
    root: &Path,
    dir: &Path,
    files: &mut HashMap<String, SourceFile>,
    errors: &mut Vec<CompileError>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            discover_naze_files(root, &path, files, errors);
        } else if path.extension().is_some_and(|ext| ext == "naze") {
            // Skip .test.naze files (test grammar) and temp test entry files
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".test.naze") || name == ".naze-test-entry.naze" {
                    continue;
                }
            }
            // Build import path: relative to root, without extension
            // e.g., "components/card" for "project/components/card.naze"
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let import_path = rel.with_extension("").to_string_lossy().replace('\\', "/");

            // Skip the entry file — it's handled separately
            if import_path == "app" {
                continue;
            }

            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(CompileError {
                        message: format!("cannot read '{}': {}", path.display(), e),
                        file: path.display().to_string(),
                        line: 0,
                        column: 0,
                        severity: Severity::Error,
                    });
                    continue;
                }
            };

            let nodes = match naze_parser::parse(&source, &import_path) {
                Ok(n) => n,
                Err(e) => {
                    errors.push(CompileError {
                        message: e.message,
                        file: e.file,
                        line: e.line,
                        column: e.column,
                        severity: Severity::Error,
                    });
                    continue;
                }
            };

            files.insert(
                import_path,
                SourceFile {
                    path: path.clone(),
                    nodes,
                },
            );
        }
    }
}

/// Discover .naze files from an external dependency directory.
/// Import paths are prefixed with the dependency name, e.g., "@naze/ui-kit/button"
/// for a file `button.naze` in the dependency root.
fn discover_dep_files(
    dep_name: &str,
    dep_dir: &Path,
    files: &mut HashMap<String, SourceFile>,
    errors: &mut Vec<CompileError>,
) {
    if !dep_dir.exists() {
        errors.push(CompileError {
            message: format!("dependency '{}' directory not found: {}", dep_name, dep_dir.display()),
            file: dep_name.to_string(),
            line: 0,
            column: 0,
            severity: Severity::Error,
        });
        return;
    }
    discover_dep_files_recursive(dep_name, dep_dir, dep_dir, files, errors);
}

fn discover_dep_files_recursive(
    dep_name: &str,
    dep_root: &Path,
    dir: &Path,
    files: &mut HashMap<String, SourceFile>,
    errors: &mut Vec<CompileError>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            discover_dep_files_recursive(dep_name, dep_root, &path, files, errors);
        } else if path.extension().is_some_and(|ext| ext == "naze") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".test.naze") {
                    continue;
                }
            }

            // Build import path: dep_name + relative path within dep
            // e.g., "@naze/ui-kit" + "button.naze" → "@naze/ui-kit/button"
            let rel = path.strip_prefix(dep_root).unwrap_or(&path);
            let rel_import = rel.with_extension("").to_string_lossy().replace('\\', "/");
            let import_path = format!("{}/{}", dep_name, rel_import);

            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(CompileError {
                        message: format!("cannot read '{}': {}", path.display(), e),
                        file: path.display().to_string(),
                        line: 0,
                        column: 0,
                        severity: Severity::Error,
                    });
                    continue;
                }
            };

            let nodes = match naze_parser::parse(&source, &import_path) {
                Ok(n) => n,
                Err(e) => {
                    errors.push(CompileError {
                        message: e.message,
                        file: e.file,
                        line: e.line,
                        column: e.column,
                        severity: Severity::Error,
                    });
                    continue;
                }
            };

            files.insert(
                import_path,
                SourceFile {
                    path: path.clone(),
                    nodes,
                },
            );
        }
    }
}

/// Cached version of `discover_naze_files`: skips parsing files whose content hash hasn't changed.
fn discover_naze_files_cached(
    root: &Path,
    dir: &Path,
    files: &mut HashMap<String, SourceFile>,
    errors: &mut Vec<CompileError>,
    cache: &mut BuildCache,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            discover_naze_files_cached(root, &path, files, errors, cache);
        } else if path.extension().is_some_and(|ext| ext == "naze") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".test.naze") || name == ".naze-test-entry.naze" {
                    continue;
                }
            }
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let import_path = rel.with_extension("").to_string_lossy().replace('\\', "/");

            if import_path == "app" {
                continue;
            }

            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    errors.push(CompileError {
                        message: format!("cannot read '{}': {}", path.display(), e),
                        file: path.display().to_string(),
                        line: 0,
                        column: 0,
                        severity: Severity::Error,
                    });
                    continue;
                }
            };

            let content_hash = hash_content(source.as_bytes());

            // Check cache: if hash matches, reuse AST
            let nodes = if let Some((cached_hash, cached_nodes)) = cache.file_cache.get(&path) {
                if *cached_hash == content_hash {
                    cached_nodes.clone()
                } else {
                    match naze_parser::parse(&source, &import_path) {
                        Ok(n) => {
                            cache.file_cache.insert(path.clone(), (content_hash, n.clone()));
                            n
                        }
                        Err(e) => {
                            errors.push(CompileError {
                                message: e.message,
                                file: e.file,
                                line: e.line,
                                column: e.column,
                                severity: Severity::Error,
                            });
                            continue;
                        }
                    }
                }
            } else {
                match naze_parser::parse(&source, &import_path) {
                    Ok(n) => {
                        cache.file_cache.insert(path.clone(), (content_hash, n.clone()));
                        n
                    }
                    Err(e) => {
                        errors.push(CompileError {
                            message: e.message,
                            file: e.file,
                            line: e.line,
                            column: e.column,
                            severity: Severity::Error,
                        });
                        continue;
                    }
                }
            };

            files.insert(
                import_path,
                SourceFile {
                    path: path.clone(),
                    nodes,
                },
            );
        }
    }
}

/// Collect all use paths from a node tree.
fn collect_use_paths(nodes: &[Node]) -> Vec<(String, Span)> {
    let mut paths = Vec::new();
    for node in nodes {
        if let Node::UseStmt { path, span } = node {
            paths.push((path.join("/"), span.clone()));
        }
    }
    paths
}

/// Walk the element tree and warn about element names that aren't builtins or imported components.
fn check_unresolved_elements(
    nodes: &[Node],
    imported_names: &HashSet<&str>,
    components: &HashMap<String, ComponentDef>,
    errors: &mut Vec<CompileError>,
) {
    for node in nodes {
        match node {
            Node::Element {
                name,
                children,
                span,
                ..  // handlers, props
            } => {
                if !BUILTIN_ELEMENTS.contains(&name.as_str())
                    && !imported_names.contains(name.as_str())
                    && !components.values().any(|c| c.name == *name)
                {
                    errors.push(CompileError {
                        message: format!(
                            "unknown element '{}': not a builtin or imported component",
                            name
                        ),
                        file: span.file.clone(),
                        line: span.line,
                        column: span.col,
                        severity: Severity::Warning,
                    });
                }
                check_unresolved_elements(children, imported_names, components, errors);
            }
            Node::App { children, .. } => {
                check_unresolved_elements(children, imported_names, components, errors);
            }
            Node::Component { children, .. } | Node::Template { children, .. } => {
                check_unresolved_elements(children, imported_names, components, errors);
            }
            Node::If {
                then_children,
                else_children,
                ..
            } => {
                check_unresolved_elements(then_children, imported_names, components, errors);
                check_unresolved_elements(else_children, imported_names, components, errors);
            }
            Node::Each { children, .. } => {
                check_unresolved_elements(children, imported_names, components, errors);
            }
            Node::Slot {
                default_children, ..
            } => {
                check_unresolved_elements(default_children, imported_names, components, errors);
            }
            Node::Fill { children, .. } => {
                check_unresolved_elements(children, imported_names, components, errors);
            }
            _ => {}
        }
    }
}

/// Detect circular dependencies among component files.
/// A circular dependency exists when file A uses B and B uses A (directly or transitively).
fn check_circular_deps(files: &HashMap<String, SourceFile>, errors: &mut Vec<CompileError>) {
    // Build adjacency list: file import_path -> list of use paths
    let mut deps: HashMap<&str, Vec<String>> = HashMap::new();
    for (import_path, source) in files {
        let use_paths = collect_use_paths(&source.nodes);
        deps.insert(
            import_path.as_str(),
            use_paths.into_iter().map(|(p, _)| p).collect(),
        );
    }

    // DFS cycle detection
    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    for start in deps.keys() {
        if !visited.contains(start) {
            let mut stack = vec![(*start, false)];
            while let Some((node, returning)) = stack.pop() {
                if returning {
                    in_stack.remove(node);
                    continue;
                }

                if in_stack.contains(node) {
                    errors.push(CompileError {
                        message: format!("circular dependency detected involving '{}'", node),
                        file: node.to_string(),
                        line: 0,
                        column: 0,
                        severity: Severity::Error,
                    });
                    continue;
                }

                if visited.contains(node) {
                    continue;
                }

                visited.insert(node);
                in_stack.insert(node);
                stack.push((node, true)); // marker to remove from in_stack on return

                if let Some(neighbors) = deps.get(node) {
                    for dep in neighbors {
                        if !visited.contains(dep.as_str()) {
                            stack.push((dep.as_str(), false));
                        } else if in_stack.contains(dep.as_str()) {
                            errors.push(CompileError {
                                message: format!("circular dependency: '{}' -> '{}'", node, dep),
                                file: node.to_string(),
                                line: 0,
                                column: 0,
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Helper to build a synthetic AST element node.
fn make_element(name: &str, props: Vec<Prop>, children: Vec<Node>) -> Node {
    Node::Element {
        name: name.to_string(),
        props,
        children,
        handlers: vec![],
        span: builtin_span(),
    }
}

/// Helper to build a slot statement node.
fn make_slot(name: &str) -> Node {
    Node::Slot {
        name: Some(name.to_string()),
        default_children: vec![],
        span: builtin_span(),
    }
}

fn builtin_span() -> Span {
    Span {
        file: "<builtin>".to_string(),
        line: 0,
        col: 0,
        offset: 0,
        len: 0,
    }
}

fn px_prop(key: &str, val: f64) -> Prop {
    Prop {
        key: key.to_string(),
        value: Value::Num(val, Some(Unit::Px)),
    }
}

fn num_prop(key: &str, val: f64) -> Prop {
    Prop {
        key: key.to_string(),
        value: Value::Num(val, None),
    }
}

/// Built-in layout templates available without `use` imports.
fn builtin_templates() -> Vec<ComponentDef> {
    vec![
        // app-shell(toolbar, sidebar, main, footer)
        // column: toolbar row 64px → row(sidebar 240px + main flex-grow) → footer row 48px
        ComponentDef {
            import_path: "app-shell".to_string(),
            name: "app-shell".to_string(),
            params: vec![],
            children: vec![make_element(
                "column",
                vec![],
                vec![
                    make_element(
                        "row",
                        vec![px_prop("height", 64.0)],
                        vec![make_slot("toolbar")],
                    ),
                    make_element(
                        "row",
                        vec![num_prop("grow", 1.0)],
                        vec![
                            make_element(
                                "column",
                                vec![px_prop("width", 240.0)],
                                vec![make_slot("sidebar")],
                            ),
                            make_element(
                                "column",
                                vec![num_prop("grow", 1.0)],
                                vec![make_slot("main")],
                            ),
                        ],
                    ),
                    make_element(
                        "row",
                        vec![px_prop("height", 48.0)],
                        vec![make_slot("footer")],
                    ),
                ],
            )],
            span: builtin_span(),
            file: PathBuf::from("<builtin>"),
        },
        // dashboard(header, cards, detail-panel)
        // column: header row 64px → row(cards flex-grow + detail-panel 400px collapsible:1200px)
        ComponentDef {
            import_path: "dashboard".to_string(),
            name: "dashboard".to_string(),
            params: vec![],
            children: vec![make_element(
                "column",
                vec![],
                vec![
                    make_element(
                        "row",
                        vec![px_prop("height", 64.0)],
                        vec![make_slot("header")],
                    ),
                    make_element(
                        "row",
                        vec![num_prop("grow", 1.0)],
                        vec![
                            make_element(
                                "column",
                                vec![num_prop("grow", 1.0)],
                                vec![make_slot("cards")],
                            ),
                            make_element(
                                "column",
                                vec![
                                    px_prop("width", 400.0),
                                    px_prop("collapsible", 1200.0),
                                ],
                                vec![make_slot("detail-panel")],
                            ),
                        ],
                    ),
                ],
            )],
            span: builtin_span(),
            file: PathBuf::from("<builtin>"),
        },
        // sidebar-layout(nav, content)
        // row: nav 240px + content flex-grow
        ComponentDef {
            import_path: "sidebar-layout".to_string(),
            name: "sidebar-layout".to_string(),
            params: vec![],
            children: vec![make_element(
                "row",
                vec![],
                vec![
                    make_element(
                        "column",
                        vec![px_prop("width", 240.0)],
                        vec![make_slot("nav")],
                    ),
                    make_element(
                        "column",
                        vec![num_prop("grow", 1.0)],
                        vec![make_slot("content")],
                    ),
                ],
            )],
            span: builtin_span(),
            file: PathBuf::from("<builtin>"),
        },
        // split-view(left, right)
        // row: left flex-grow + right flex-grow
        ComponentDef {
            import_path: "split-view".to_string(),
            name: "split-view".to_string(),
            params: vec![],
            children: vec![make_element(
                "row",
                vec![],
                vec![
                    make_element(
                        "column",
                        vec![num_prop("grow", 1.0)],
                        vec![make_slot("left")],
                    ),
                    make_element(
                        "column",
                        vec![num_prop("grow", 1.0)],
                        vec![make_slot("right")],
                    ),
                ],
            )],
            span: builtin_span(),
            file: PathBuf::from("<builtin>"),
        },
        // centered(content)
        // column with max-width 800px, centered content slot
        ComponentDef {
            import_path: "centered".to_string(),
            name: "centered".to_string(),
            params: vec![],
            children: vec![make_element(
                "column",
                vec![px_prop("max-width", 800.0)],
                vec![make_slot("content")],
            )],
            span: builtin_span(),
            file: PathBuf::from("<builtin>"),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_project(dir: &Path, files: &[(&str, &str)]) {
        for (name, content) in files {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }
    }

    #[test]
    fn resolve_simple_app() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(
            dir.path(),
            &[(
                "app.naze",
                r#"app "Hello" {
  text "world"
}
"#,
            )],
        );

        let project = resolve(dir.path(), "app.naze", &[]);
        assert!(project.errors.is_empty(), "errors: {:?}", project.errors);
        assert!(!project.entry.nodes.is_empty());
    }

    #[test]
    fn resolve_with_component_import() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(
            dir.path(),
            &[
                (
                    "components/box.naze",
                    r#"component box(color: color) {
  rect width: 80px, height: 80px, color: color
}
"#,
                ),
                (
                    "app.naze",
                    r#"use components/box

app "Test" {
  box color: #ff0000
}
"#,
                ),
            ],
        );

        let project = resolve(dir.path(), "app.naze", &[]);
        assert!(project.errors.is_empty(), "errors: {:?}", project.errors);
        assert!(project.components.contains_key("components/box"));
        let comp = &project.components["components/box"];
        assert_eq!(comp.name, "box");
        assert_eq!(comp.params.len(), 1);
    }

    #[test]
    fn resolve_missing_import() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(
            dir.path(),
            &[(
                "app.naze",
                r#"use components/missing

app "Test" {
  missing color: #ff0000
}
"#,
            )],
        );

        let project = resolve(dir.path(), "app.naze", &[]);
        assert!(!project.errors.is_empty());
        assert!(project
            .errors
            .iter()
            .any(|e| e.message.contains("unresolved import")));
    }

    #[test]
    fn resolve_unknown_element_warning() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(
            dir.path(),
            &[(
                "app.naze",
                r#"app "Test" {
  foobar width: 100px
}
"#,
            )],
        );

        let project = resolve(dir.path(), "app.naze", &[]);
        assert!(project
            .errors
            .iter()
            .any(|e| e.message.contains("unknown element 'foobar'")));
    }

    #[test]
    fn resolve_builtin_elements_ok() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(
            dir.path(),
            &[(
                "app.naze",
                r#"app "Test" {
  column padding: 20px {
    row gap: 8px {
      rect width: 50px, height: 50px, color: #000000
      text "hello"
      heading "title"
      spacer
    }
    container padding: 10px {
      stack {
        grid columns: 2 {
          text "a"
        }
      }
    }
  }
}
"#,
            )],
        );

        let project = resolve(dir.path(), "app.naze", &[]);
        let real_errors: Vec<_> = project
            .errors
            .iter()
            .filter(|e| matches!(e.severity, Severity::Error))
            .collect();
        assert!(real_errors.is_empty(), "errors: {:?}", real_errors);
    }

    #[test]
    fn resolve_missing_entry_file() {
        let dir = tempfile::tempdir().unwrap();
        let project = resolve(dir.path(), "nonexistent.naze", &[]);
        assert!(!project.errors.is_empty());
        assert!(project.errors[0].message.contains("cannot read"));
    }

    #[test]
    fn resolve_multiple_components() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(
            dir.path(),
            &[
                (
                    "components/box.naze",
                    "component box(color: color) {\n  rect width: 80px, height: 80px, color: color\n}\n",
                ),
                (
                    "components/card.naze",
                    "component card(bg: color = #ffffff) {\n  container color: bg, padding: 16px {\n    text \"card\"\n  }\n}\n",
                ),
                (
                    "app.naze",
                    "use components/box\nuse components/card\n\napp \"Test\" {\n  box color: #ff0000\n  card bg: #eee\n}\n",
                ),
            ],
        );

        let project = resolve(dir.path(), "app.naze", &[]);
        assert!(project.errors.is_empty(), "errors: {:?}", project.errors);
        assert!(project.components.contains_key("components/box"));
        assert!(project.components.contains_key("components/card"));
    }

    #[test]
    fn resolve_with_dependency() {
        let dir = tempfile::tempdir().unwrap();
        let dep_dir = tempfile::tempdir().unwrap();

        // Create a component in the dependency directory
        let button_path = dep_dir.path().join("button.naze");
        fs::write(
            &button_path,
            "component button(label: text) {\n  text \"{label}\"\n}\n",
        )
        .unwrap();

        // Create app that imports from the dependency
        setup_project(
            dir.path(),
            &[(
                "app.naze",
                "use @mylib/button\n\napp \"Test\" {\n  button label: \"Click\"\n}\n",
            )],
        );

        let deps = vec![ResolvedDep {
            name: "@mylib".to_string(),
            local_path: dep_dir.path().to_path_buf(),
        }];

        let project = resolve(dir.path(), "app.naze", &deps);
        assert!(project.errors.is_empty(), "errors: {:?}", project.errors);
        assert!(
            project.components.contains_key("@mylib/button"),
            "components: {:?}",
            project.components.keys().collect::<Vec<_>>()
        );
        assert_eq!(project.components["@mylib/button"].name, "button");
    }

    #[test]
    fn resolve_dependency_nested_component() {
        let dir = tempfile::tempdir().unwrap();
        let dep_dir = tempfile::tempdir().unwrap();

        // Create nested component in dependency
        let comp_dir = dep_dir.path().join("widgets");
        fs::create_dir_all(&comp_dir).unwrap();
        fs::write(
            comp_dir.join("card.naze"),
            "component card(title: text) {\n  text \"{title}\"\n}\n",
        )
        .unwrap();

        setup_project(
            dir.path(),
            &[(
                "app.naze",
                "use @ui/widgets/card\n\napp \"Test\" {\n  card title: \"Hello\"\n}\n",
            )],
        );

        let deps = vec![ResolvedDep {
            name: "@ui".to_string(),
            local_path: dep_dir.path().to_path_buf(),
        }];

        let project = resolve(dir.path(), "app.naze", &deps);
        assert!(project.errors.is_empty(), "errors: {:?}", project.errors);
        assert!(project.components.contains_key("@ui/widgets/card"));
    }

    #[test]
    fn resolve_missing_dependency_dir() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(
            dir.path(),
            &[(
                "app.naze",
                "app \"Test\" {\n  text \"hello\"\n}\n",
            )],
        );

        let deps = vec![ResolvedDep {
            name: "@missing".to_string(),
            local_path: PathBuf::from("/nonexistent/path"),
        }];

        let project = resolve(dir.path(), "app.naze", &deps);
        assert!(project.errors.iter().any(|e| e.message.contains("directory not found")));
    }
}
