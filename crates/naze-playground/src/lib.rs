//! In-browser playground: wraps the Naze parser, compiler, and IR serializer
//! so that `.naze` source can be compiled entirely inside a WASM module.

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use naze_compiler::codegen;
use naze_compiler::error::{CompileError, Severity};
use naze_compiler::resolve;
use naze_compiler::typecheck;

// ---- Base64 encoder (no external dependency) --------------------------------

const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(B64_ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            out.push(B64_ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(B64_ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

// ---- JSON helpers -----------------------------------------------------------

fn errors_to_json(errors: &[CompileError]) -> String {
    let items: Vec<String> = errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .map(|e| {
            format!(
                r#"{{"file":"{}","line":{},"column":{},"message":{}}}"#,
                escape_json_str(&e.file),
                e.line,
                e.column,
                serde_json::to_string(&e.message).unwrap_or_else(|_| format!(
                    "\"{}\"",
                    escape_json_str(&e.message)
                )),
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

fn escape_json_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn success_json(binary_b64: &str) -> String {
    format!(
        r#"{{"success":true,"binary":"{}","errors":[]}}"#,
        binary_b64
    )
}

fn error_json(errors: &[CompileError]) -> String {
    format!(
        r#"{{"success":false,"errors":{}}}"#,
        errors_to_json(errors)
    )
}

// ---- Internal pipeline helpers ----------------------------------------------

/// Parse source into AST nodes, returning either `Ok(nodes)` or `Err(json_string)`.
fn parse_source(source: &str) -> Result<Vec<naze_parser::ast::Node>, String> {
    match naze_parser::parse(source, "playground.naze") {
        Ok(nodes) => Ok(nodes),
        Err(e) => {
            let err = CompileError {
                message: e.to_string(),
                file: "playground.naze".into(),
                line: 0,
                column: 0,
                severity: Severity::Error,
            };
            Err(error_json(&[err]))
        }
    }
}

/// Build a minimal `ResolvedProject` from parsed AST nodes (no imports, no
/// external components, no themes).
fn build_project(nodes: Vec<naze_parser::ast::Node>) -> resolve::ResolvedProject {
    resolve::ResolvedProject {
        entry: resolve::SourceFile {
            path: "playground.naze".into(),
            nodes,
        },
        components: HashMap::new(),
        themes: vec![],
        imports: vec![],
        errors: vec![],
    }
}

/// Run the type checker on a project. Returns `Ok(())` when there are no
/// errors, or `Err(json_string)` on type errors.
fn run_typecheck(project: &resolve::ResolvedProject) -> Result<(), String> {
    let errors = typecheck::typecheck(project);
    let real_errors: Vec<&CompileError> = errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .collect();
    if real_errors.is_empty() {
        Ok(())
    } else {
        Err(error_json(&errors))
    }
}

// ---- wasm_bindgen exports ---------------------------------------------------

/// Full compile pipeline: parse, typecheck, codegen, serialize to binary.
///
/// Returns a JSON string with shape:
/// - On success: `{ "success": true, "binary": "<base64>", "errors": [] }`
/// - On failure: `{ "success": false, "errors": [{ "file", "line", "column", "message" }] }`
#[wasm_bindgen]
pub fn compile(source: &str) -> String {
    // 1. Parse
    let nodes = match parse_source(source) {
        Ok(n) => n,
        Err(json) => return json,
    };

    // 2. Build project
    let project = build_project(nodes);

    // 3. Typecheck
    if let Err(json) = run_typecheck(&project) {
        return json;
    }

    // 4. Codegen
    let tree = codegen::lower(&project);

    // 5. Serialize
    let binary = naze_ir::serialize(&tree);

    // 6. Base64 encode and return
    let b64 = base64_encode(&binary);
    success_json(&b64)
}

/// Parse and typecheck only (no codegen). Returns JSON error array.
///
/// Returns a JSON string with shape:
/// - On success: `{ "success": true, "errors": [] }`
/// - On failure: `{ "success": false, "errors": [{ "file", "line", "column", "message" }] }`
#[wasm_bindgen]
pub fn check(source: &str) -> String {
    // 1. Parse
    let nodes = match parse_source(source) {
        Ok(n) => n,
        Err(json) => return json,
    };

    // 2. Build project
    let project = build_project(nodes);

    // 3. Typecheck
    let errors = typecheck::typecheck(&project);
    let real_errors: Vec<&CompileError> = errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .collect();

    if real_errors.is_empty() {
        r#"{"success":true,"errors":[]}"#.to_string()
    } else {
        error_json(&errors)
    }
}

/// Return a JSON array of curated example programs for the playground UI.
///
/// Each entry has `{ "name": "...", "category": "...", "source": "..." }`.
#[wasm_bindgen]
pub fn get_examples() -> String {
    examples_to_json()
}

// ---- Embedded examples ------------------------------------------------------

struct Example {
    name: &'static str,
    category: &'static str,
    source: &'static str,
}

// serde_json::to_string needs Serialize; implement manual JSON for the array.
impl Example {
    fn to_json(&self) -> String {
        format!(
            r#"{{"name":{},"category":{},"source":{}}}"#,
            serde_json::to_string(self.name).unwrap_or_default(),
            serde_json::to_string(self.category).unwrap_or_default(),
            serde_json::to_string(self.source).unwrap_or_default(),
        )
    }
}

const EXAMPLES: &[Example] = &[
    Example {
        name: "Hello World",
        category: "Basics",
        source: r#"-- A simple hello world
app "Hello Naze" {
  column padding: 20px, gap: 16px {
    heading "Hello, Naze!"
    text "Welcome to the future of UI."
  }
}"#,
    },
    Example {
        name: "Counter",
        category: "Basics",
        source: r#"-- Counter with state and events
app "Counter" {
  state count = 0

  column padding: 20px, gap: 16px {
    heading "My Counter"
    text "Current count: {count}"
    rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
      text "Increment"
      on click: set count = count + 1
    }
    rect width: 200px, height: 50px, color: #dc2626, radius: 8px {
      text "Reset"
      on click: set count = 0
    }
  }
}"#,
    },
    Example {
        name: "Conditionals",
        category: "Basics",
        source: r#"-- Conditional rendering and iteration
app "Conditional" {
  state count = 0
  state items = ["Apple", "Banana", "Cherry"]

  column padding: 20px, gap: 16px {
    heading "Conditional Demo"

    row gap: 12px {
      rect width: 120px, height: 50px, color: #2563eb, radius: 8px {
        text "Increment"
        on click: set count = count + 1
      }
      rect width: 120px, height: 50px, color: #dc2626, radius: 8px {
        text "Reset"
        on click: set count = 0
      }
    }

    if count > 0 {
      text "Count is {count} (positive)"
    } else {
      text "Count is zero"
    }

    heading "Items:"
    each item in items {
      text "{item}"
    }
  }
}"#,
    },
    Example {
        name: "Dashboard",
        category: "Layout",
        source: r#"-- Dashboard with header, sidebar, and cards
app "Dashboard" {
  column gap: 0px {
    container padding: 16px, color: #1e293b {
      heading "Dashboard" font-size: 20px, color: #ffffff
    }

    row padding: 20px, gap: 20px {
      column width: 200px, gap: 8px, padding: 16px, color: #f8fafc {
        text "Overview"
        text "Analytics"
        text "Reports"
        text "Settings"
      }

      column gap: 16px {
        heading "Overview"
        row gap: 16px {
          container padding: 16px, color: #eff6ff, radius: 8px, width: 180px {
            column gap: 4px {
              text "Revenue"
              heading "$12,345" font-size: 24px
            }
          }
          container padding: 16px, color: #f0fdf4, radius: 8px, width: 180px {
            column gap: 4px {
              text "Users"
              heading "1,234" font-size: 24px
            }
          }
          container padding: 16px, color: #fefce8, radius: 8px, width: 180px {
            column gap: 4px {
              text "Orders"
              heading "567" font-size: 24px
            }
          }
        }
      }
    }
  }
}"#,
    },
    Example {
        name: "Theme Tokens",
        category: "Styling",
        source: r#"-- Using design tokens for consistent styling
app "Theming Demo" {
  column padding: theme.spacing.lg, gap: theme.spacing.md, color: theme.colors.background {
    heading "Design Tokens" color: theme.colors.foreground

    text "Colors" color: theme.colors.muted
    row gap: theme.spacing.sm {
      rect width: 60px, height: 60px, color: theme.colors.primary, radius: 8px
      rect width: 60px, height: 60px, color: theme.colors.secondary, radius: 8px
      rect width: 60px, height: 60px, color: theme.colors.success, radius: 8px
      rect width: 60px, height: 60px, color: theme.colors.warning, radius: 8px
      rect width: 60px, height: 60px, color: theme.colors.danger, radius: 8px
    }

    text "Themed Card" color: theme.colors.muted
    container padding: theme.spacing.md, color: #ffffff, radius: 8px {
      column gap: theme.spacing.sm {
        heading "Card Title" color: theme.colors.foreground
        text "Uses theme tokens for consistent styling." color: theme.colors.muted
      }
    }
  }
}"#,
    },
    Example {
        name: "Data Fetching",
        category: "Advanced",
        source: r#"-- Fetching data from an API
app "Posts" {
  data posts: fetch "https://jsonplaceholder.typicode.com/posts?_limit=5"

  column gap: 16px, padding: 20px {
    heading "API Data"

    if posts.loading {
      text "Loading..." color: #666666
    }

    if posts.error {
      text "Error: {posts.error}" color: #dc2626
    }

    if posts.data {
      each post in posts.data {
        container padding: 12px, color: #f3f4f6, radius: 8px {
          column gap: 4px {
            heading "{post.title}" font-size: 16px
            text "{post.body}" color: #666666, font-size: 14px
          }
        }
      }
    }
  }
}"#,
    },
];

// Manual JSON serialization for the example array (avoids deriving Serialize
// on a static slice which would require a wrapper type).
fn examples_to_json() -> String {
    let items: Vec<String> = EXAMPLES.iter().map(Example::to_json).collect();
    format!("[{}]", items.join(","))
}
