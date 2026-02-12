//! Binary size analyzer for Naze app bundles.
//!
//! Breaks down `app_data.bin` by section and optionally analyzes WASM binary sections.

use naze_ir::{RenderNode, RenderTree};
use std::collections::HashMap;

/// A measured section of the binary.
struct Section {
    name: String,
    bytes: usize,
    count: usize,
}

/// Recursive node statistics.
struct NodeStats {
    total_nodes: usize,
    total_handlers: usize,
    kinds: HashMap<String, usize>,
}

pub fn run(
    bin_path: &str,
    wasm_path: Option<&str>,
    compare_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read(bin_path)?;
    let total_size = bytes.len();
    let tree = naze_ir::deserialize(&bytes)?;

    let sections = measure_sections(&tree);
    let node_stats = count_nodes(&tree);

    eprintln!("=== Naze Binary Analysis ===\n");
    eprintln!("File: {bin_path}");
    eprintln!("Total size: {}\n", format_bytes(total_size));

    // Section breakdown
    eprintln!(
        "{:<24} {:>10} {:>8} {:>8}",
        "Section", "Bytes", "Count", "%"
    );
    eprintln!("{}", "-".repeat(52));
    let mut accounted = 0;
    for s in &sections {
        let pct = if total_size > 0 {
            (s.bytes as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        accounted += s.bytes;
        eprintln!(
            "{:<24} {:>10} {:>8} {:>7.1}%",
            s.name,
            format_bytes(s.bytes),
            s.count,
            pct
        );
    }
    let overhead = total_size.saturating_sub(accounted);
    if overhead > 0 {
        let pct = (overhead as f64 / total_size as f64) * 100.0;
        eprintln!(
            "{:<24} {:>10} {:>8} {:>7.1}%",
            "overhead/framing",
            format_bytes(overhead),
            "",
            pct
        );
    }

    // Node breakdown
    eprintln!("\n=== Node Statistics ===\n");
    eprintln!("Total nodes: {}", node_stats.total_nodes);
    eprintln!("Total handlers: {}", node_stats.total_handlers);
    if !node_stats.kinds.is_empty() {
        eprintln!("\n{:<20} {:>8}", "Node Kind", "Count");
        eprintln!("{}", "-".repeat(30));
        let mut kinds: Vec<_> = node_stats.kinds.iter().collect();
        kinds.sort_by(|a, b| b.1.cmp(a.1));
        for (kind, count) in kinds {
            eprintln!("{:<20} {:>8}", kind, count);
        }
    }

    // WASM analysis
    if let Some(wasm) = wasm_path {
        eprintln!();
        analyze_wasm(wasm)?;
    }

    // Comparison mode
    if let Some(cmp_path) = compare_path {
        eprintln!();
        compare_binaries(bin_path, cmp_path, &sections)?;
    }

    Ok(())
}

/// Measure each section by re-serializing parts individually.
fn measure_sections(tree: &RenderTree) -> Vec<Section> {
    let mut sections = Vec::new();

    // State declarations
    let state_bytes = measure_by_serializing(&RenderTree {
        state: tree.state.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "state".into(),
        bytes: state_bytes,
        count: tree.state.len(),
    });

    // Data declarations
    let data_bytes = measure_by_serializing(&RenderTree {
        data: tree.data.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "data".into(),
        bytes: data_bytes,
        count: tree.data.len(),
    });

    // Computed declarations
    let computed_bytes = measure_by_serializing(&RenderTree {
        computed: tree.computed.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "computed".into(),
        bytes: computed_bytes,
        count: tree.computed.len(),
    });

    // Storage declarations
    let storage_bytes = measure_by_serializing(&RenderTree {
        storage: tree.storage.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "storage".into(),
        bytes: storage_bytes,
        count: tree.storage.len(),
    });

    // Timer declarations
    let timer_bytes = measure_by_serializing(&RenderTree {
        timers: tree.timers.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "timers".into(),
        bytes: timer_bytes,
        count: tree.timers.len(),
    });

    // Param declarations
    let param_bytes = measure_by_serializing(&RenderTree {
        params: tree.params.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "params".into(),
        bytes: param_bytes,
        count: tree.params.len(),
    });

    // Root nodes
    let root_bytes = measure_by_serializing(&RenderTree {
        root: tree.root.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "root nodes".into(),
        bytes: root_bytes,
        count: tree.root.len(),
    });

    // Pages
    let pages_bytes = measure_by_serializing(&RenderTree {
        pages: tree.pages.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "pages".into(),
        bytes: pages_bytes,
        count: tree.pages.len(),
    });

    // Themes
    let themes_bytes = measure_by_serializing(&RenderTree {
        themes: tree.themes.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "themes".into(),
        bytes: themes_bytes,
        count: tree.themes.len(),
    });

    // Imports
    let imports_bytes = measure_by_serializing(&RenderTree {
        imports: tree.imports.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "imports".into(),
        bytes: imports_bytes,
        count: tree.imports.len(),
    });

    // Server functions
    let sf_bytes = measure_by_serializing(&RenderTree {
        server_functions: tree.server_functions.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "server functions".into(),
        bytes: sf_bytes,
        count: tree.server_functions.len(),
    });

    // Server calls
    let sc_bytes = measure_by_serializing(&RenderTree {
        server_calls: tree.server_calls.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "server calls".into(),
        bytes: sc_bytes,
        count: tree.server_calls.len(),
    });

    // Prompts
    let prompt_bytes = measure_by_serializing(&RenderTree {
        prompts: tree.prompts.clone(),
        ..empty_tree()
    }) - empty_tree_size();
    sections.push(Section {
        name: "prompts".into(),
        bytes: prompt_bytes,
        count: tree.prompts.len(),
    });

    sections
}

fn empty_tree() -> RenderTree {
    RenderTree {
        title: String::new(),
        root: vec![],
        state: vec![],
        data: vec![],
        computed: vec![],
        storage: vec![],
        timers: vec![],
        params: vec![],
        pages: vec![],
        themes: vec![],
        imports: vec![],
        server_functions: vec![],
        server_calls: vec![],
        prompts: vec![],
        guards: vec![],
    }
}

fn empty_tree_size() -> usize {
    let (bytes, _) = naze_ir::serialize_with_source_map(&empty_tree());
    bytes.len()
}

fn measure_by_serializing(tree: &RenderTree) -> usize {
    let (bytes, _) = naze_ir::serialize_with_source_map(tree);
    bytes.len()
}

/// Recursively count nodes and handlers.
fn count_nodes(tree: &RenderTree) -> NodeStats {
    let mut stats = NodeStats {
        total_nodes: 0,
        total_handlers: 0,
        kinds: HashMap::new(),
    };
    for node in &tree.root {
        count_node(node, &mut stats);
    }
    for page in &tree.pages {
        for node in &page.root {
            count_node(node, &mut stats);
        }
    }
    stats
}

fn count_node(node: &RenderNode, stats: &mut NodeStats) {
    stats.total_nodes += 1;
    stats.total_handlers += node.handlers.len();
    *stats.kinds.entry(node.kind.clone()).or_insert(0) += 1;
    for child in &node.children {
        count_node(child, stats);
    }
    if let Some(ref else_children) = node.else_children {
        for child in else_children {
            count_node(child, stats);
        }
    }
}

/// Analyze WASM binary sections using wasmparser.
fn analyze_wasm(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    let total = bytes.len();

    eprintln!("=== WASM Binary Analysis ===\n");
    eprintln!("File: {path}");
    eprintln!("Total size: {}\n", format_bytes(total));

    let mut sections: Vec<(String, usize)> = Vec::new();

    let parser = wasmparser::Parser::new(0);
    for payload in parser.parse_all(&bytes) {
        let payload = payload?;
        use wasmparser::Payload::*;
        match payload {
            TypeSection(s) => sections.push(("type".into(), s.range().len())),
            ImportSection(s) => sections.push(("import".into(), s.range().len())),
            FunctionSection(s) => sections.push(("function".into(), s.range().len())),
            TableSection(s) => sections.push(("table".into(), s.range().len())),
            MemorySection(s) => sections.push(("memory".into(), s.range().len())),
            GlobalSection(s) => sections.push(("global".into(), s.range().len())),
            ExportSection(s) => sections.push(("export".into(), s.range().len())),
            ElementSection(s) => sections.push(("element".into(), s.range().len())),
            DataSection(s) => sections.push(("data".into(), s.range().len())),
            CodeSectionStart { size, .. } => sections.push(("code".into(), size as usize)),
            CustomSection(s) => {
                sections.push((format!("custom({})", s.name()), s.data().len()));
            }
            _ => {}
        }
    }

    eprintln!("{:<24} {:>10} {:>8}", "Section", "Bytes", "%");
    eprintln!("{}", "-".repeat(44));
    for (name, size) in &sections {
        let pct = if total > 0 {
            (*size as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        eprintln!("{:<24} {:>10} {:>7.1}%", name, format_bytes(*size), pct);
    }

    Ok(())
}

/// Compare two binaries section by section.
fn compare_binaries(
    path_a: &str,
    path_b: &str,
    sections_a: &[Section],
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes_b = std::fs::read(path_b)?;
    let total_b = bytes_b.len();
    let tree_b = naze_ir::deserialize(&bytes_b)?;
    let sections_b = measure_sections(&tree_b);

    let bytes_a = std::fs::read(path_a)?;
    let total_a = bytes_a.len();

    eprintln!("=== Comparison ===\n");
    eprintln!("A: {path_a} ({})", format_bytes(total_a));
    eprintln!("B: {path_b} ({})", format_bytes(total_b));
    let delta_total = total_b as i64 - total_a as i64;
    eprintln!("Delta: {}\n", format_delta(delta_total));

    eprintln!("{:<24} {:>10} {:>10} {:>10}", "Section", "A", "B", "Delta");
    eprintln!("{}", "-".repeat(56));

    for (sa, sb) in sections_a.iter().zip(sections_b.iter()) {
        let delta = sb.bytes as i64 - sa.bytes as i64;
        eprintln!(
            "{:<24} {:>10} {:>10} {:>10}",
            sa.name,
            format_bytes(sa.bytes),
            format_bytes(sb.bytes),
            format_delta(delta)
        );
    }

    Ok(())
}

fn format_bytes(n: usize) -> String {
    if n >= 1024 * 1024 {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    } else if n >= 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{n} B")
    }
}

fn format_delta(n: i64) -> String {
    if n > 0 {
        format!("+{}", format_bytes(n as usize))
    } else if n < 0 {
        format!("-{}", format_bytes((-n) as usize))
    } else {
        "0 B".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use naze_ir::RenderValue;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_format_delta() {
        assert_eq!(format_delta(0), "0 B");
        assert_eq!(format_delta(100), "+100 B");
        assert_eq!(format_delta(-100), "-100 B");
    }

    #[test]
    fn test_empty_tree_sections() {
        let tree = empty_tree();
        let sections = measure_sections(&tree);
        // All sections should be 0 bytes for an empty tree
        for s in &sections {
            assert_eq!(s.bytes, 0, "section {} should be 0 bytes", s.name);
            assert_eq!(s.count, 0, "section {} should have 0 items", s.name);
        }
    }

    #[test]
    fn test_count_nodes_empty() {
        let tree = empty_tree();
        let stats = count_nodes(&tree);
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_handlers, 0);
        assert!(stats.kinds.is_empty());
    }

    #[test]
    fn test_count_nodes_with_children() {
        let tree = RenderTree {
            root: vec![RenderNode {
                kind: "column".into(),
                props: HashMap::new(),
                children: vec![
                    RenderNode {
                        kind: "text".into(),
                        props: HashMap::new(),
                        children: vec![],
                        handlers: vec![],
                        span: None,
                        condition: None,
                        else_children: None,
                        each_binding: None,
                    },
                    RenderNode {
                        kind: "text".into(),
                        props: HashMap::new(),
                        children: vec![],
                        handlers: vec![],
                        span: None,
                        condition: None,
                        else_children: None,
                        each_binding: None,
                    },
                ],
                handlers: vec![],
                span: None,
                condition: None,
                else_children: None,
                each_binding: None,
            }],
            ..empty_tree()
        };
        let stats = count_nodes(&tree);
        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.total_handlers, 0);
        assert_eq!(*stats.kinds.get("column").unwrap(), 1);
        assert_eq!(*stats.kinds.get("text").unwrap(), 2);
    }

    #[test]
    fn test_sections_with_state() {
        let tree = RenderTree {
            state: vec![naze_ir::StateDecl {
                name: "count".into(),
                initial: RenderValue::Num(0.0, None),
                shared: false,
            }],
            ..empty_tree()
        };
        let sections = measure_sections(&tree);
        let state_section = sections.iter().find(|s| s.name == "state").unwrap();
        assert!(state_section.bytes > 0, "state section should have bytes");
        assert_eq!(state_section.count, 1);
    }
}
