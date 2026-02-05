use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use naze_parser::ast::{Node, Param, Span};

use crate::error::{CompileError, Severity};

/// Theme tokens for consistent styling.
/// Tokens are resolved at compile time and inlined as values.
#[derive(Debug, Clone)]
pub struct Theme {
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

/// The result of resolving a project.
#[derive(Debug)]
pub struct ResolvedProject {
    /// The entry file's AST (the app).
    pub entry: SourceFile,
    /// All component definitions, keyed by import path.
    pub components: HashMap<String, ComponentDef>,
    /// Theme tokens (from theme.naze or default).
    pub theme: Theme,
    /// Errors encountered during resolution.
    pub errors: Vec<CompileError>,
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
pub fn resolve(project_dir: &Path, entry: &str) -> ResolvedProject {
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
                theme: default_theme(),
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
                theme: default_theme(),
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

    // 3. Extract component definitions from all discovered files
    let mut components: HashMap<String, ComponentDef> = HashMap::new();
    for (import_path, source_file) in &all_files {
        for node in &source_file.nodes {
            if let Node::Component {
                name,
                params,
                children,
                span,
            } = node
            {
                let def = ComponentDef {
                    import_path: import_path.clone(),
                    name: name.clone(),
                    params: params.clone(),
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
    }

    // Also extract components defined inline in the entry file
    for node in &entry_file.nodes {
        if let Node::Component {
            name,
            params,
            children,
            span,
        } = node
        {
            let import_path = name.clone();
            let def = ComponentDef {
                import_path: import_path.clone(),
                name: name.clone(),
                params: params.clone(),
                children: children.clone(),
                span: span.clone(),
                file: entry_file.path.clone(),
            };
            components.entry(import_path).or_insert(def);
        }
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

    // 7. Load theme (from theme.naze or use default)
    let theme = load_theme(project_dir, &mut errors);

    ResolvedProject {
        entry: entry_file,
        components,
        theme,
        errors,
    }
}

/// Load theme from theme.naze file, or return default theme if not present.
fn load_theme(project_dir: &Path, errors: &mut Vec<CompileError>) -> Theme {
    let theme_path = project_dir.join("theme.naze");
    if !theme_path.exists() {
        return default_theme();
    }

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
            return default_theme();
        }
    };

    let nodes = match naze_parser::parse(&source, "theme.naze") {
        Ok(n) => n,
        Err(e) => {
            errors.push(CompileError {
                message: e.message,
                file: e.file,
                line: e.line,
                column: e.column,
                severity: Severity::Error,
            });
            return default_theme();
        }
    };

    // Extract theme from parsed nodes
    for node in nodes {
        if let Node::Theme { colors, spacing, .. } = node {
            let mut theme = default_theme();
            // Merge custom colors (override defaults)
            for (name, color) in colors {
                theme.colors.insert(name, color);
            }
            // Merge custom spacing (override defaults)
            for (name, value, _unit) in spacing {
                theme.spacing.insert(name, value);
            }
            return theme;
        }
    }

    // No theme block found in file
    errors.push(CompileError {
        message: "theme.naze must contain a theme block".to_string(),
        file: "theme.naze".to_string(),
        line: 0,
        column: 0,
        severity: Severity::Warning,
    });
    default_theme()
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
            // Build import path: relative to root, without extension
            // e.g., "components/card" for "project/components/card.naze"
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let import_path = rel
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");

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
            Node::Component { children, .. } => {
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
                                message: format!(
                                    "circular dependency: '{}' -> '{}'",
                                    node, dep
                                ),
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

        let project = resolve(dir.path(), "app.naze");
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

        let project = resolve(dir.path(), "app.naze");
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

        let project = resolve(dir.path(), "app.naze");
        assert!(!project.errors.is_empty());
        assert!(project.errors.iter().any(|e| e.message.contains("unresolved import")));
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

        let project = resolve(dir.path(), "app.naze");
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

        let project = resolve(dir.path(), "app.naze");
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
        let project = resolve(dir.path(), "nonexistent.naze");
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

        let project = resolve(dir.path(), "app.naze");
        assert!(project.errors.is_empty(), "errors: {:?}", project.errors);
        assert_eq!(project.components.len(), 2);
        assert!(project.components.contains_key("components/box"));
        assert!(project.components.contains_key("components/card"));
    }
}
