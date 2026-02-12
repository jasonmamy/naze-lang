//! Test runner for .test.naze files.
//!
//! Compiles referenced components/apps through the standard pipeline
//! (resolve → typecheck → codegen → RenderTree), then executes test steps
//! against the resolved render tree and layout tree headlessly.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use naze_ir::{IrAction, RenderTree, RenderValue};
use naze_layout::{LayoutTree, PositionedNode};
use naze_parser::ast::{AssertKind, TestFile, TestStep};
use serde::Serialize;

use crate::exec;

// ─── Result types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TestSuiteResult {
    pub file: String,
    pub results: Vec<TestResult>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub name: String,
    pub kind: String, // "test" or "flow"
    pub passed: bool,
    pub assertions: Vec<AssertionResult>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AssertionResult {
    pub description: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
}

// ─── Test execution environment ──────────────────────────────────────────────

struct TestEnv {
    render_tree: RenderTree,
    state: HashMap<String, RenderValue>,
    layout: Option<LayoutTree>,
    resolved_nodes: Vec<naze_ir::RenderNode>,
    current_page: String,
}

impl TestEnv {
    fn new(render_tree: RenderTree) -> Self {
        let state = exec::init_state(&render_tree);
        let mut env = TestEnv {
            render_tree,
            state,
            layout: None,
            resolved_nodes: vec![],
            current_page: "/".to_string(),
        };
        env.refresh_layout();
        env
    }

    fn refresh_layout(&mut self) {
        // Re-evaluate computed values after state changes
        for comp in &self.render_tree.computed {
            let val = exec::evaluate_expr(&comp.expr, &self.state);
            self.state.insert(comp.name.clone(), val);
        }

        // Get the nodes for the current page
        let nodes = self.get_current_page_nodes();
        let resolved = exec::resolve_nodes(&nodes, &self.state);

        // Store resolved nodes for text searching (layout drops children of leaf nodes like rect)
        self.resolved_nodes = resolved.clone();

        // Build a temporary RenderTree for layout computation
        let resolved_tree = RenderTree {
            title: self.render_tree.title.clone(),
            state: self.render_tree.state.clone(),
            data: self.render_tree.data.clone(),
            computed: self.render_tree.computed.clone(),
            storage: self.render_tree.storage.clone(),
            timers: self.render_tree.timers.clone(),
            params: self.render_tree.params.clone(),
            root: resolved,
            pages: vec![],
            themes: vec![],
            imports: vec![],
            server_functions: vec![],
            server_calls: vec![],
            prompts: vec![],
            guards: vec![],
        };

        self.layout = Some(naze_layout::compute_layout(&resolved_tree, 1024.0, 768.0));
    }

    fn get_current_page_nodes(&self) -> Vec<naze_ir::RenderNode> {
        // If the app has pages, find the one matching current_page
        if !self.render_tree.pages.is_empty() {
            for page in &self.render_tree.pages {
                if page.path == self.current_page {
                    return page.root.clone();
                }
            }
            // Fall back to "/" page or root
            for page in &self.render_tree.pages {
                if page.path == "/" {
                    return page.root.clone();
                }
            }
        }
        self.render_tree.root.clone()
    }

    fn layout_nodes(&self) -> &[PositionedNode] {
        match &self.layout {
            Some(layout) => &layout.root,
            None => &[],
        }
    }

    fn click(&mut self, text: &str) -> Result<(), String> {
        let layout_nodes = self.layout_nodes().to_vec();
        let resolved = &self.resolved_nodes;

        // Find the clickable element: search both layout and render trees
        // First try direct layout tree search (for elements where text IS in layout)
        let node = exec::find_element_by_text(&layout_nodes, text)
            // Then try render tree search (for text inside leaf nodes like rect)
            .or_else(|| exec::find_clickable_for_text(&layout_nodes, resolved, text))
            .ok_or_else(|| format!("no element with text \"{}\" found", text))?;

        // Get center coordinates
        let cx = node.x + node.width / 2.0;
        let cy = node.y + node.height / 2.0;

        // Find and execute click handlers
        let handlers = exec::find_click_handlers(&layout_nodes, cx, cy, &self.state);
        for handler in &handlers {
            exec::execute_action(&handler.action, &mut self.state);
            // Handle navigate actions
            if let IrAction::Navigate { path } = &handler.action {
                self.current_page = path.clone();
            }
        }

        // Refresh layout after state changes
        self.refresh_layout();
        Ok(())
    }

    fn fill(&mut self, label: &str, value: &str) -> Result<(), String> {
        let layout_nodes = self.layout_nodes().to_vec();
        let resolved = &self.resolved_nodes;

        // Search layout tree first, then render tree for inputs inside containers
        let node = exec::find_input_by_label(&layout_nodes, label)
            .or_else(|| exec::find_input_in_render_nodes(&layout_nodes, resolved, label))
            .ok_or_else(|| format!("no input with label/placeholder \"{}\" found", label))?;

        // Get the bind variable from either layout or render node props
        if let Some(RenderValue::Bind(var)) = node.props.get("bind") {
            self.state
                .insert(var.clone(), RenderValue::Str(value.to_string()));
            self.refresh_layout();
            Ok(())
        } else {
            Err(format!("input \"{}\" has no bind property", label))
        }
    }

    fn navigate(&mut self, path: &str) {
        self.current_page = path.to_string();
        self.refresh_layout();
    }

    fn check_assert(&self, kind: &AssertKind) -> AssertionResult {
        match kind {
            AssertKind::TextVisible(text) => {
                // Search both layout tree and resolved render tree for text
                let visible = exec::is_text_visible(self.layout_nodes(), text)
                    || exec::is_text_in_render_nodes(&self.resolved_nodes, text);
                AssertionResult {
                    description: format!("text \"{}\" is visible", text),
                    passed: visible,
                    expected: format!("\"{}\" visible", text),
                    actual: if visible {
                        "found".to_string()
                    } else {
                        "not found".to_string()
                    },
                }
            }
            AssertKind::TextNotVisible(text) => {
                let visible = exec::is_text_visible(self.layout_nodes(), text)
                    || exec::is_text_in_render_nodes(&self.resolved_nodes, text);
                AssertionResult {
                    description: format!("text \"{}\" is not visible", text),
                    passed: !visible,
                    expected: format!("\"{}\" not visible", text),
                    actual: if visible {
                        "found".to_string()
                    } else {
                        "not found".to_string()
                    },
                }
            }
            AssertKind::PageIs(path) => {
                let on_page = self.current_page == *path;
                AssertionResult {
                    description: format!("page is \"{}\"", path),
                    passed: on_page,
                    expected: path.clone(),
                    actual: self.current_page.clone(),
                }
            }
            AssertKind::StateIs { name, value } => {
                let expected_rv = ast_value_to_render_value(value);
                let actual = self.state.get(name);
                let passed = actual.is_some_and(|v| render_values_equal(v, &expected_rv));
                AssertionResult {
                    description: format!("state {} is {:?}", name, expected_rv),
                    passed,
                    expected: exec::render_value_to_string(&expected_rv),
                    actual: actual
                        .map(exec::render_value_to_string)
                        .unwrap_or_else(|| "<undefined>".to_string()),
                }
            }
            AssertKind::Emitted(name) => {
                // Emitted events are not tracked in the headless runner yet
                AssertionResult {
                    description: format!("emitted {}", name),
                    passed: false,
                    expected: format!("{} emitted", name),
                    actual: "event tracking not implemented".to_string(),
                }
            }
            AssertKind::NoA11yViolations => {
                let violations = check_a11y(self.layout_nodes());
                let passed = violations.is_empty();
                AssertionResult {
                    description: "no accessibility violations".to_string(),
                    passed,
                    expected: "0 violations".to_string(),
                    actual: if passed {
                        "0 violations".to_string()
                    } else {
                        format!(
                            "{} violation(s): {}",
                            violations.len(),
                            violations.join(", ")
                        )
                    },
                }
            }
        }
    }
}

// ─── Compilation helpers ─────────────────────────────────────────────────────

/// Compile a .naze file through the full pipeline and return a RenderTree.
fn compile_naze_file(project_dir: &Path, entry: &str) -> Result<RenderTree, String> {
    let project = naze_compiler::resolve::resolve(project_dir, entry, &[]);

    // Check for resolution errors
    let has_errors = project
        .errors
        .iter()
        .any(|e| matches!(e.severity, naze_compiler::error::Severity::Error));
    if has_errors {
        let msgs: Vec<String> = project.errors.iter().map(|e| e.message.clone()).collect();
        return Err(format!("resolution errors: {}", msgs.join("; ")));
    }

    // Type-check
    let tc_errors = naze_compiler::typecheck::typecheck(&project);
    let has_tc_errors = tc_errors
        .iter()
        .any(|e| matches!(e.severity, naze_compiler::error::Severity::Error));
    if has_tc_errors {
        let msgs: Vec<String> = tc_errors.iter().map(|e| e.message.clone()).collect();
        return Err(format!("type errors: {}", msgs.join("; ")));
    }

    // Codegen
    let render_tree = naze_compiler::codegen::lower(&project);
    Ok(render_tree)
}

/// Resolve a component name from test `use` paths to a .naze file path.
/// Searches: project_dir, test_file_dir, test_file_dir/../ (common tests/ convention).
fn resolve_component_path(
    project_dir: &Path,
    test_file_dir: &Path,
    uses: &[String],
    component_name: &str,
) -> Option<PathBuf> {
    // Directories to search, in priority order
    let search_dirs: Vec<&Path> = {
        let mut dirs = vec![project_dir, test_file_dir];
        // Parent of test file dir (common convention: tests/ is one level below source)
        if let Some(parent) = test_file_dir.parent() {
            dirs.push(parent);
        }
        dirs
    };

    // First check if the component name directly matches a use path's last segment
    for use_path in uses {
        let segments: Vec<&str> = use_path.split('/').collect();
        if let Some(last) = segments.last() {
            if *last == component_name {
                for dir in &search_dirs {
                    let file_path = dir.join(format!("{}.naze", use_path));
                    if file_path.exists() {
                        return Some(file_path);
                    }
                }
            }
        }
    }

    // Try as a direct file name in each search directory
    for dir in &search_dirs {
        let direct = dir.join(format!("{}.naze", component_name));
        if direct.exists() {
            return Some(direct);
        }
    }

    None
}

/// Build a RenderTree for a component by wrapping it in a synthetic app entry.
fn compile_component(
    project_dir: &Path,
    test_file_dir: &Path,
    uses: &[String],
    component_name: &str,
    props: &[naze_parser::ast::Prop],
) -> Result<RenderTree, String> {
    // Try to find the component's source file
    let component_path = resolve_component_path(project_dir, test_file_dir, uses, component_name);

    if let Some(path) = &component_path {
        // Check if this is a full app file (contains app block at top level)
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        // Strip leading comments and whitespace to find the first real statement
        let first_stmt = source
            .lines()
            .find(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with("--")
            })
            .unwrap_or("");
        if first_stmt.trim().starts_with("app ") {
            // It's a full app file — compile directly
            let rel_path = path
                .strip_prefix(project_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            return compile_naze_file(project_dir, &rel_path);
        }
    }

    // Build a synthetic app that wraps the component.
    // Generate use statements and component invocation with props.
    let mut synthetic = String::new();
    for use_path in uses {
        synthetic.push_str(&format!("use {}\n", use_path));
    }
    synthetic.push_str(&format!("app \"test\" {{\n  {} ", component_name));
    if !props.is_empty() {
        let prop_strs: Vec<String> = props
            .iter()
            .map(|p| format!("{}: {}", p.key, ast_value_to_naze_literal(&p.value)))
            .collect();
        synthetic.push_str(&prop_strs.join(", "));
    }
    synthetic.push_str("\n}\n");

    // Write the synthetic file to a temp location
    let temp_entry = project_dir.join(".naze-test-entry.naze");
    std::fs::write(&temp_entry, &synthetic)
        .map_err(|e| format!("cannot write temp entry: {}", e))?;

    let result = compile_naze_file(project_dir, ".naze-test-entry.naze");

    // Clean up
    let _ = std::fs::remove_file(&temp_entry);

    result
}

// ─── Main runner ─────────────────────────────────────────────────────────────

/// Discover and run all .test.naze files in the project directory.
pub fn run_all(
    project_dir: &Path,
    filter: Option<&str>,
) -> Result<Vec<TestSuiteResult>, Box<dyn std::error::Error>> {
    let test_files = discover_test_files(project_dir)?;

    if test_files.is_empty() {
        return Ok(vec![]);
    }

    let mut suites = Vec::new();

    for test_path in &test_files {
        let suite = run_test_file(project_dir, test_path, filter)?;
        suites.push(suite);
    }

    Ok(suites)
}

/// Discover all .test.naze files by walking the project directory.
fn discover_test_files(project_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    walk_for_test_files(project_dir, project_dir, &mut files)?;
    files.sort();
    Ok(files)
}

#[allow(clippy::only_used_in_recursion)]
fn walk_for_test_files(
    root: &Path,
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() {
        return Ok(());
    }
    let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // Skip hidden dirs, dist/, target/
    if dir_name.starts_with('.') || dir_name == "dist" || dir_name == "target" || dir_name == "pkg"
    {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_for_test_files(root, &path, out)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".test.naze") {
                out.push(path);
            }
        }
    }
    Ok(())
}

/// Run a single .test.naze file and return results.
fn run_test_file(
    project_dir: &Path,
    test_path: &Path,
    filter: Option<&str>,
) -> Result<TestSuiteResult, Box<dyn std::error::Error>> {
    let suite_start = Instant::now();
    let source = std::fs::read_to_string(test_path)?;
    let file_str = test_path
        .strip_prefix(project_dir)
        .unwrap_or(test_path)
        .to_string_lossy()
        .to_string();

    let test_file = naze_parser::parse_test_file(&source, &file_str)
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

    // Compute the test file's directory for relative component resolution
    let test_file_dir = test_path.parent().unwrap_or(project_dir);

    let mut results = Vec::new();

    // Run test blocks
    for test_block in &test_file.tests {
        if let Some(f) = filter {
            if !test_block.name.contains(f) {
                continue;
            }
        }
        let result = run_test_block(
            project_dir,
            test_file_dir,
            &test_file,
            &test_block.name,
            &test_block.steps,
        );
        results.push(result);
    }

    // Run flow blocks
    for flow_block in &test_file.flows {
        if let Some(f) = filter {
            if !flow_block.name.contains(f) {
                continue;
            }
        }
        let result = run_flow_block(
            project_dir,
            test_file_dir,
            &test_file,
            &flow_block.name,
            &flow_block.steps,
        );
        results.push(result);
    }

    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;
    let duration_ms = suite_start.elapsed().as_millis() as u64;

    Ok(TestSuiteResult {
        file: file_str,
        results,
        total,
        passed,
        failed,
        duration_ms,
    })
}

/// Run a single test block (component-scoped).
fn run_test_block(
    project_dir: &Path,
    test_file_dir: &Path,
    test_file: &TestFile,
    name: &str,
    steps: &[TestStep],
) -> TestResult {
    let start = Instant::now();
    let mut assertions = Vec::new();
    let mut env: Option<TestEnv> = None;
    let mut error: Option<String> = None;

    for step in steps {
        match step {
            TestStep::Render {
                component, props, ..
            } => {
                match compile_component(
                    project_dir,
                    test_file_dir,
                    &test_file.uses,
                    component,
                    props,
                ) {
                    Ok(tree) => {
                        env = Some(TestEnv::new(tree));
                    }
                    Err(e) => {
                        error = Some(format!("compile error for {}: {}", component, e));
                        break;
                    }
                }
            }
            TestStep::Click { text, .. } => {
                if let Some(ref mut e) = env {
                    if let Err(msg) = e.click(text) {
                        error = Some(msg);
                        break;
                    }
                } else {
                    error = Some("click before render".to_string());
                    break;
                }
            }
            TestStep::Fill { target, value, .. } => {
                if let Some(ref mut e) = env {
                    if let Err(msg) = e.fill(target, value) {
                        error = Some(msg);
                        break;
                    }
                } else {
                    error = Some("fill before render".to_string());
                    break;
                }
            }
            TestStep::Navigate { path, .. } => {
                if let Some(ref mut e) = env {
                    e.navigate(path);
                } else {
                    error = Some("navigate before render".to_string());
                    break;
                }
            }
            TestStep::Wait { .. } => {
                // In headless mode, waits are no-ops (no real timers)
            }
            TestStep::Assert { kind, .. } => {
                if let Some(ref e) = env {
                    let result = e.check_assert(kind);
                    assertions.push(result);
                } else {
                    error = Some("assert before render".to_string());
                    break;
                }
            }
        }
    }

    let passed = error.is_none() && assertions.iter().all(|a| a.passed);
    TestResult {
        name: name.to_string(),
        kind: "test".to_string(),
        passed,
        assertions,
        duration_ms: start.elapsed().as_millis() as u64,
        error,
    }
}

/// Run a single flow block (app-scoped with navigation).
fn run_flow_block(
    project_dir: &Path,
    test_file_dir: &Path,
    test_file: &TestFile,
    name: &str,
    steps: &[TestStep],
) -> TestResult {
    let start = Instant::now();
    let mut assertions = Vec::new();
    let mut env: Option<TestEnv> = None;
    let mut error: Option<String> = None;

    for step in steps {
        match step {
            TestStep::Render {
                component, props, ..
            } => {
                // In a flow block, render compiles the target (app or component)
                match compile_component(
                    project_dir,
                    test_file_dir,
                    &test_file.uses,
                    component,
                    props,
                ) {
                    Ok(tree) => {
                        env = Some(TestEnv::new(tree));
                    }
                    Err(e) => {
                        error = Some(format!("compile error for {}: {}", component, e));
                        break;
                    }
                }
            }
            TestStep::Click { text, .. } => {
                if let Some(ref mut e) = env {
                    if let Err(msg) = e.click(text) {
                        error = Some(msg);
                        break;
                    }
                } else {
                    error = Some("click before render".to_string());
                    break;
                }
            }
            TestStep::Fill { target, value, .. } => {
                if let Some(ref mut e) = env {
                    if let Err(msg) = e.fill(target, value) {
                        error = Some(msg);
                        break;
                    }
                } else {
                    error = Some("fill before render".to_string());
                    break;
                }
            }
            TestStep::Navigate { path, .. } => {
                if let Some(ref mut e) = env {
                    e.navigate(path);
                } else {
                    error = Some("navigate before render".to_string());
                    break;
                }
            }
            TestStep::Wait { .. } => {
                // No-op in headless mode
            }
            TestStep::Assert { kind, .. } => {
                if let Some(ref e) = env {
                    let result = e.check_assert(kind);
                    assertions.push(result);
                } else {
                    error = Some("assert before render".to_string());
                    break;
                }
            }
        }
    }

    let passed = error.is_none() && assertions.iter().all(|a| a.passed);
    TestResult {
        name: name.to_string(),
        kind: "flow".to_string(),
        passed,
        assertions,
        duration_ms: start.elapsed().as_millis() as u64,
        error,
    }
}

// ─── Value conversion helpers ────────────────────────────────────────────────

/// Convert an AST Value to a RenderValue for assertion comparison.
fn ast_value_to_render_value(value: &naze_parser::ast::Value) -> RenderValue {
    match value {
        naze_parser::ast::Value::Str(s) => RenderValue::Str(s.clone()),
        naze_parser::ast::Value::Num(n, _) => RenderValue::Num(*n, None),
        naze_parser::ast::Value::Bool(b) => RenderValue::Bool(*b),
        naze_parser::ast::Value::Color(c) => RenderValue::Color(*c),
        naze_parser::ast::Value::Ref(parts) => {
            // A bare ref used as a value — treat as string
            RenderValue::Str(parts.join("."))
        }
        naze_parser::ast::Value::InterpolatedStr(_) => {
            // Flatten interpolated string parts
            RenderValue::Str("<interpolated>".to_string())
        }
        naze_parser::ast::Value::List(items) => {
            RenderValue::List(items.iter().map(ast_value_to_render_value).collect())
        }
        naze_parser::ast::Value::Object(entries) => RenderValue::Object(
            entries
                .iter()
                .map(|(k, v)| (k.clone(), ast_value_to_render_value(v)))
                .collect(),
        ),
        naze_parser::ast::Value::Bind(s) => RenderValue::Bind(s.clone()),
    }
}

/// Convert an AST Value back to a Naze literal string (for synthetic files).
fn ast_value_to_naze_literal(value: &naze_parser::ast::Value) -> String {
    match value {
        naze_parser::ast::Value::Str(s) => format!("\"{}\"", s),
        naze_parser::ast::Value::Num(n, unit) => {
            let num_str = if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            };
            match unit {
                Some(naze_parser::ast::Unit::Px) => format!("{}px", num_str),
                Some(naze_parser::ast::Unit::Percent) => format!("{}%", num_str),
                Some(naze_parser::ast::Unit::Em) => format!("{}em", num_str),
                None => num_str,
            }
        }
        naze_parser::ast::Value::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
        naze_parser::ast::Value::Color(c) => format!("#{:06x}", c),
        naze_parser::ast::Value::Ref(parts) => parts.join("."),
        _ => "0".to_string(),
    }
}

/// Compare two RenderValues for equality (for assertions).
fn render_values_equal(a: &RenderValue, b: &RenderValue) -> bool {
    match (a, b) {
        (RenderValue::Str(a), RenderValue::Str(b)) => a == b,
        (RenderValue::Num(a, _), RenderValue::Num(b, _)) => (a - b).abs() < f64::EPSILON,
        (RenderValue::Bool(a), RenderValue::Bool(b)) => a == b,
        (RenderValue::Color(a), RenderValue::Color(b)) => a == b,
        // Numeric comparison for mixed types (e.g., state is Num, expected is Num)
        _ => {
            let a_str = exec::render_value_to_string(a);
            let b_str = exec::render_value_to_string(b);
            a_str == b_str
        }
    }
}

// ─── Accessibility checks ────────────────────────────────────────────────────

/// Basic accessibility checks on the layout tree.
fn check_a11y(nodes: &[PositionedNode]) -> Vec<String> {
    let mut violations = Vec::new();
    check_a11y_recursive(nodes, &mut violations);
    violations
}

fn check_a11y_recursive(nodes: &[PositionedNode], violations: &mut Vec<String>) {
    for node in nodes {
        // Images must have alt text
        if node.kind == "image" && !node.props.contains_key("alt") {
            violations.push("image missing alt text".to_string());
        }
        // Inputs should have a placeholder or label
        if (node.kind == "input" || node.kind == "textarea")
            && !node.props.contains_key("placeholder")
            && !node.props.contains_key("label")
        {
            violations.push(format!("{} missing label or placeholder", node.kind));
        }
        check_a11y_recursive(&node.children, violations);
    }
}

// ─── Output formatting ──────────────────────────────────────────────────────

pub fn print_results_text(suites: &[TestSuiteResult]) {
    for suite in suites {
        eprintln!("\nrunning {} test(s) from {}", suite.total, suite.file);
        for result in &suite.results {
            let status = if result.passed { "ok" } else { "FAILED" };
            eprintln!(
                "  {} \"{}\" ... {} ({}ms)",
                result.kind, result.name, status, result.duration_ms
            );
            if !result.passed {
                for assertion in &result.assertions {
                    if !assertion.passed {
                        eprintln!("    {}", assertion.description);
                        eprintln!("      expected: {}", assertion.expected);
                        eprintln!("      actual:   {}", assertion.actual);
                    }
                }
                if let Some(err) = &result.error {
                    eprintln!("    error: {}", err);
                }
            }
        }
    }

    let total: usize = suites.iter().map(|s| s.total).sum();
    let passed: usize = suites.iter().map(|s| s.passed).sum();
    let failed: usize = suites.iter().map(|s| s.failed).sum();
    let duration: u64 = suites.iter().map(|s| s.duration_ms).sum();
    eprintln!(
        "\ntest result: {} failed, {} passed ({}ms)\n",
        failed, passed, duration
    );
    if total == 0 {
        eprintln!("no test files found");
    }
}

pub fn print_results_json(suites: &[TestSuiteResult]) {
    let total: usize = suites.iter().map(|s| s.total).sum();
    let passed: usize = suites.iter().map(|s| s.passed).sum();
    let failed: usize = suites.iter().map(|s| s.failed).sum();
    let output = serde_json::json!({
        "total": total,
        "passed": passed,
        "failed": failed,
        "suites": suites,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
