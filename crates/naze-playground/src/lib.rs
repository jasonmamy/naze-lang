//! In-browser playground: wraps the Naze parser, compiler, and IR serializer
//! so that `.naze` source can be compiled entirely inside a WASM module.

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use naze_compiler::codegen;
use naze_compiler::error::{CompileError, Severity};
use naze_compiler::resolve;
use naze_compiler::typecheck;

// ---- Base64 encoder (no external dependency) --------------------------------

const B64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

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
                serde_json::to_string(&e.message)
                    .unwrap_or_else(|_| format!("\"{}\"", escape_json_str(&e.message))),
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
    format!(r#"{{"success":false,"errors":{}}}"#, errors_to_json(errors))
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
        themes: vec![resolve::default_theme()],
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
    // ── Forms ────────────────────────────────────────────────────────────
    Example {
        name: "Text Input",
        category: "Forms",
        source: r#"app "Text Input Test" {
  state name = ""

  column padding: 20px, gap: 16px {
    heading "Text Input Demo"

    text "Enter your name:"

    input bind: name, placeholder: "Type here..."

    text "Hello, {name}!" color: #2563eb
  }
}"#,
    },
    Example {
        name: "Checkbox",
        category: "Forms",
        source: r#"app "Checkbox Test" {
  state agreed = false

  column padding: 20px, gap: 16px {
    heading "Checkbox Demo"

    checkbox "I agree to the terms and conditions" bind: agreed

    if agreed {
      text "Thank you for agreeing!" color: #22c55e
    }

    if agreed == false {
      text "Please check the box above to continue." color: #94a3b8
    }
  }
}"#,
    },
    Example {
        name: "Select",
        category: "Forms",
        source: r#"app "Select Test" {
  state country = "us"

  column padding: 20px, gap: 16px {
    heading "Select Demo"

    text "Choose a country:"

    select bind: country {
      option "USA" value: "us"
      option "Canada" value: "ca"
      option "Mexico" value: "mx"
    }

    text "You selected: {country}" color: #2563eb
  }
}"#,
    },
    Example {
        name: "Form Validation",
        category: "Forms",
        source: r#"-- Input validation example
app "Input Validation Demo" {
  state username = ""
  state email = ""
  state age = ""

  column padding: 20px, gap: 16px {
    heading "Input Validation Demo"

    -- Username with min/max length
    text "Username (3-20 characters)"
    input bind: username, placeholder: "Enter username", validate: { required: true, min-length: 3, max-length: 20 }
    if username_error {
      text "{username_error}" color: #dc2626
    }

    -- Email with pattern validation
    text "Email"
    input bind: email, type: "email", placeholder: "Enter email", validate: { required: true }
    if email_error {
      text "{email_error}" color: #dc2626
    }

    -- Age with min/max number validation
    text "Age (18-120)"
    input bind: age, type: "number", placeholder: "Enter age", validate: { required: true, min: 18, max: 120 }
    if age_error {
      text "{age_error}" color: #dc2626
    }

    -- Show validation status
    row gap: 8px {
      if username_valid {
        text "Username valid" color: #16a34a
      }
      if email_valid {
        text "Email valid" color: #16a34a
      }
      if age_valid {
        text "Age valid" color: #16a34a
      }
    }
  }
}"#,
    },
    // ── Layout ───────────────────────────────────────────────────────────
    Example {
        name: "Grid",
        category: "Layout",
        source: r#"-- Grid of items
app "Grid" {
  column padding: 20px, gap: 16px {
    heading "Grid Layout"
    grid columns: 3, gap: 12px {
      rect width: 100px, height: 100px, color: #f43f5e, radius: 8px
      rect width: 100px, height: 100px, color: #8b5cf6, radius: 8px
      rect width: 100px, height: 100px, color: #06b6d4, radius: 8px
      rect width: 100px, height: 100px, color: #f59e0b, radius: 8px
      rect width: 100px, height: 100px, color: #10b981, radius: 8px
      rect width: 100px, height: 100px, color: #6366f1, radius: 8px
    }
  }
}"#,
    },
    Example {
        name: "Responsive",
        category: "Layout",
        source: r#"-- Responsive layout example
app "Responsive Demo" {
  column padding: 20px, gap: 16px {
    heading "Responsive Layout"

    -- This row becomes a column on narrow viewports
    row responsive: 768px, gap: 16px {
      column grow: 1 {
        heading "Main Content"
        text "This is the primary content area."
        text "On narrow screens, the sidebar stacks below."
      }
      column width: 280px {
        heading "Sidebar"
        text "Navigation or supplementary info."
      }
    }

    -- This panel hides on viewports narrower than 1200px
    column collapsible: 1200px {
      text "Extra detail panel (visible on wide screens only)"
    }

    -- Responsive grid: 3 columns on wide, 1 column on narrow
    grid columns: 3, responsive: 768px, gap: 12px {
      rect width: 100px, height: 80px, color: #2563eb
      rect width: 100px, height: 80px, color: #22c55e
      rect width: 100px, height: 80px, color: #f59e0b
    }
  }
}"#,
    },
    Example {
        name: "Scroll Containers",
        category: "Layout",
        source: r#"-- Scroll Container Demo
app "Scroll Demo" {
  state items = ["Apple", "Banana", "Cherry", "Date", "Elderberry", "Fig", "Grape", "Honeydew", "Kiwi", "Lemon", "Mango", "Nectarine", "Orange", "Papaya", "Quince"]

  column padding: 20px, gap: 16px {
    heading "Scroll Container Demo"

    text "Vertical Scroll (height: 200px)"
    scroll height: 200px, color: #f8fafc, radius: 8px {
      column padding: 12px, gap: 8px {
        each item in items {
          rect color: #ffffff, padding: 12px, radius: 4px, border: 1px, border-color: #e2e8f0 {
            text "{item}"
          }
        }
      }
    }

    text "Horizontal Scroll (width: 300px)"
    scroll width: 300px, height: 100px, overflow: "x", color: #f8fafc, radius: 8px {
      row padding: 12px, gap: 8px {
        rect width: 100px, height: 80px, color: #ef4444, radius: 8px {
          column align: center, justify: center {
            text "Red", color: #ffffff
          }
        }
        rect width: 100px, height: 80px, color: #22c55e, radius: 8px {
          column align: center, justify: center {
            text "Green", color: #ffffff
          }
        }
        rect width: 100px, height: 80px, color: #3b82f6, radius: 8px {
          column align: center, justify: center {
            text "Blue", color: #ffffff
          }
        }
        rect width: 100px, height: 80px, color: #8b5cf6, radius: 8px {
          column align: center, justify: center {
            text "Purple", color: #ffffff
          }
        }
        rect width: 100px, height: 80px, color: #f59e0b, radius: 8px {
          column align: center, justify: center {
            text "Orange", color: #ffffff
          }
        }
      }
    }
  }
}"#,
    },
    // ── Styling ──────────────────────────────────────────────────────────
    Example {
        name: "Typography",
        category: "Styling",
        source: r#"-- Text at different sizes
app "Typography" {
  column padding: 20px, gap: 12px {
    heading "Typography" font-size: 32px
    heading "Heading Large" font-size: 24px
    heading "Heading Medium" font-size: 20px
    text "Body text at default size"
    text "Small text" font-size: 12px
  }
}"#,
    },
    Example {
        name: "Shadows",
        category: "Styling",
        source: r#"-- Shadow: named presets and custom shadow strings
app "Shadow Demo" {
  column padding: 40px, gap: 24px, color: #f1f5f9 {
    heading "Shadow Presets"

    row gap: 24px {
      rect width: 100px, height: 100px, color: #ffffff, radius: 8px, shadow: "sm"
      rect width: 100px, height: 100px, color: #ffffff, radius: 8px, shadow: "md"
      rect width: 100px, height: 100px, color: #ffffff, radius: 8px, shadow: "lg"
      rect width: 100px, height: 100px, color: #ffffff, radius: 8px, shadow: "xl"
    }

    heading "Custom Shadow"
    rect width: 200px, height: 80px, color: #ffffff, radius: 12px, shadow: "0 8px 30px rgba(0,0,0,0.12)"
  }
}"#,
    },
    Example {
        name: "Gradients",
        category: "Styling",
        source: r#"-- Gradient: linear and radial gradients
app "Gradient Demo" {
  column padding: 20px, gap: 16px {
    heading "Linear Gradients"

    rect width: 300px, height: 60px, radius: 8px, gradient: "linear(to-right, #3b82f6, #8b5cf6)"
    rect width: 300px, height: 60px, radius: 8px, gradient: "linear(to-bottom, #f59e0b, #ef4444)"
    rect width: 300px, height: 60px, radius: 8px, gradient: "linear(to-bottom-right, #10b981, #3b82f6, #8b5cf6)"

    heading "Radial Gradient"
    rect width: 200px, height: 200px, radius: 100px, gradient: "radial(#ffffff, #3b82f6)"
  }
}"#,
    },
    Example {
        name: "Transforms",
        category: "Styling",
        source: r#"-- Transform: rotate, scale, translate
app "Transform Demo" {
  column padding: 40px, gap: 32px {
    heading "Transforms"

    row gap: 40px {
      rect width: 80px, height: 80px, color: #3b82f6, radius: 8px, transform: "rotate(45deg)"
      rect width: 80px, height: 80px, color: #ef4444, radius: 8px, transform: "scale(1.3)"
      rect width: 80px, height: 80px, color: #10b981, radius: 8px, transform: "translate(10px, -5px)"
    }

    text "Rotated text", transform: "rotate(-5deg)"
  }
}"#,
    },
    // ── Interactivity ────────────────────────────────────────────────────
    Example {
        name: "Animation",
        category: "Interactivity",
        source: r#"-- Animation: demonstrates property transitions
app "Animation Demo" {
  state expanded = false
  state color_index = 0

  column gap: 24px, padding: 32px {
    heading "Animation Demo"
    text "Click the boxes to see smooth transitions" color: #666666

    -- Size transition
    column gap: 8px {
      heading "Size Transition" font-size: 18px

      if expanded {
        row width: 200px, height: 150px, color: #3b82f6, radius: 8px, padding: 16px, transition: "height 300ms ease-out" {
          text "Click to shrink" color: #ffffff
          on click: set expanded = false
        }
      }
      if expanded == false {
        row width: 200px, height: 60px, color: #3b82f6, radius: 8px, padding: 16px, transition: "height 300ms ease-out" {
          text "Click to expand" color: #ffffff
          on click: set expanded = true
        }
      }
    }

    -- Color transition
    column gap: 8px {
      heading "Color Transition" font-size: 18px

      if color_index == 0 {
        row width: 200px, height: 80px, color: #3b82f6, radius: 8px, padding: 16px, transition: "color 200ms ease" {
          text "Click to change color" color: #ffffff
          on click: set color_index = 1
        }
      }
      if color_index == 1 {
        row width: 200px, height: 80px, color: #10b981, radius: 8px, padding: 16px, transition: "color 200ms ease" {
          text "Click to change color" color: #ffffff
          on click: set color_index = 2
        }
      }
      if color_index == 2 {
        row width: 200px, height: 80px, color: #f59e0b, radius: 8px, padding: 16px, transition: "color 200ms ease" {
          text "Click to change color" color: #ffffff
          on click: set color_index = 3
        }
      }
      if color_index == 3 {
        row width: 200px, height: 80px, color: #ef4444, radius: 8px, padding: 16px, transition: "color 200ms ease" {
          text "Click to change color" color: #ffffff
          on click: set color_index = 4
        }
      }
      if color_index == 4 {
        row width: 200px, height: 80px, color: #8b5cf6, radius: 8px, padding: 16px, transition: "color 200ms ease" {
          text "Click to change color" color: #ffffff
          on click: set color_index = 0
        }
      }
    }
  }
}"#,
    },
    Example {
        name: "Drag & Drop",
        category: "Interactivity",
        source: r#"-- Drag & Drop Example
app "Drag and Drop Demo" {
  state dropped-item = ""
  state drag-active = false

  column padding: 20px, gap: 16px {
    heading "Drag & Drop Demo"

    text "Drag the colored boxes to the drop zone below"

    -- Draggable items
    row gap: 12px {
      rect draggable: true, drag-data: "Red", width: 80px, height: 80px, color: #ef4444, radius: 8px {
        on drag-start: set drag-active = true
        column align: center, justify: center {
          text "Red", color: #ffffff
        }
      }
      rect draggable: true, drag-data: "Green", width: 80px, height: 80px, color: #22c55e, radius: 8px {
        on drag-start: set drag-active = true
        column align: center, justify: center {
          text "Green", color: #ffffff
        }
      }
      rect draggable: true, drag-data: "Blue", width: 80px, height: 80px, color: #3b82f6, radius: 8px {
        on drag-start: set drag-active = true
        column align: center, justify: center {
          text "Blue", color: #ffffff
        }
      }
    }

    -- Drop zone
    rect drop-target: true, width: 300px, height: 120px, color: #f1f5f9, radius: 8px, border: 2px, border-color: #cbd5e1 {
      on drop: set dropped-item = "Item dropped!"
      on drop: set drag-active = false
      on drag-over: set dropped-item = "Release to drop..."

      column padding: 16px, align: center, justify: center {
        if dropped-item {
          text "{dropped-item}", color: #16a34a
        }
        if drag-active {
          if dropped-item == "" {
            text "Drag an item here!", color: #64748b
          }
        }
        if drag-active == false {
          if dropped-item == "" {
            text "Drop zone", color: #94a3b8
          }
        }
      }
    }

    -- Reset button
    rect width: 100px, height: 36px, color: #6366f1, radius: 6px {
      on click: set dropped-item = ""
      on click: set drag-active = false
      column align: center, justify: center {
        text "Reset", color: #ffffff
      }
    }
  }
}"#,
    },
    Example {
        name: "Overlay Dialog",
        category: "Interactivity",
        source: r#"-- Modal overlay with backdrop and click-outside dismiss
app "Overlay Dialog" {
  state show_dialog = false

  column padding: 20px, gap: 16px {
    heading "Overlay Dialog Example"
    text "Click the button to open a modal dialog."

    rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
      text "Open Dialog", color: #ffffff
      on click: set show_dialog = true
    }

    if show_dialog {
      overlay focus-trap: true, scroll-lock: true, role: "dialog" {
        -- Backdrop
        rect width: 100%, height: 100%, color: #00000080

        -- Dialog box
        container width: 400px, height: 250px, color: #ffffff, radius: 12px, padding: 24px {
          column gap: 16px {
            heading "Confirm Action"
            text "Are you sure you want to proceed? This action cannot be undone."

            row gap: 12px {
              rect width: 120px, height: 40px, color: #e5e7eb, radius: 6px {
                text "Cancel"
                on click: set show_dialog = false
              }
              rect width: 120px, height: 40px, color: #2563eb, radius: 6px {
                text "Confirm", color: #ffffff
                on click: set show_dialog = false
              }
            }
          }
        }
        on click-outside: set show_dialog = false
      }
    }
  }
}"#,
    },
    Example {
        name: "Timer",
        category: "Interactivity",
        source: r#"-- Timer: setTimeout and setInterval for scheduling
app "Timer Demo" {
  state seconds = 0
  state toast-visible = true

  timer tick: every 1s {
    set seconds = seconds + 1
  }

  timer hide-toast: after 5s {
    set toast-visible = false
  }

  column padding: 20px, gap: 16px {
    heading "Timer Demo"
    text "Elapsed: {seconds}s"
    if toast-visible {
      rect width: 300px, height: 50px, color: #059669, radius: 8px {
        text "Welcome! This toast disappears after 5s"
      }
    }
  }
}"#,
    },
    // ── Computation ──────────────────────────────────────────────────────
    Example {
        name: "Pipelines",
        category: "Computation",
        source: r#"-- Pipeline operators: transform data declaratively
app "Student Scores" {
  state students = [{name: "Alice", score: 92}, {name: "Bob", score: 67}, {name: "Carol", score: 85}, {name: "Dave", score: 45}, {name: "Eve", score: 78}]

  computed passing = students | filter score > 60
  computed top-scores = students | filter score > 80 | sort-by name
  computed total-score = students | map score | sum
  computed student-count = students | count
  computed top-3 = students | sort-by score | take 3

  column padding: 20px, gap: 16px {
    heading "Student Scores"

    text "Total students: {student-count}"
    text "Total score: {total-score}"

    heading "Passing (score > 60):"
    each student in students | filter score > 60 | sort-by name {
      text "{student.name}: {student.score}"
    }

    heading "Top 3 by score:"
    each student in students | sort-by score | take 3 {
      text "{student.name}: {student.score}"
    }
  }
}"#,
    },
    Example {
        name: "Advanced Pipelines",
        category: "Computation",
        source: r#"-- Advanced pipeline operators: reduce, group-by, flatten, distinct
app "Pipeline Advanced" {
  state items = [{name: "Alice", dept: "Engineering", score: 92}, {name: "Bob", dept: "Sales", score: 67}, {name: "Carol", dept: "Engineering", score: 85}, {name: "Dave", dept: "Sales", score: 45}, {name: "Eve", dept: "Engineering", score: 78}]
  state nested = [[1, 2, 3], [4, 5], [6]]
  state tags = ["rust", "wasm", "rust", "js", "wasm", "rust"]

  computed total = items | map score | reduce acc + it 0
  computed by-dept = items | group-by dept
  computed flat = nested | flatten
  computed unique-tags = tags | distinct

  column padding: 20px, gap: 16px {
    heading "Advanced Pipelines"

    text "Total score (reduce): {total}"

    heading "Flattened list:"
    each n in nested | flatten {
      text "{n}"
    }

    heading "Unique tags:"
    each tag in tags | distinct {
      text "{tag}"
    }

    heading "Top scorers by department:"
    each person in items | filter score > 70 | sort-by dept {
      text "{person.name} ({person.dept}): {person.score}"
    }
  }
}"#,
    },
    Example {
        name: "Pattern Matching",
        category: "Computation",
        source: r#"-- Pattern matching: declarative conditional rendering
app "Match Demo" {
  state status = "active"
  state theme = "dark"

  column padding: 20px, gap: 16px {
    heading "Pattern Matching"

    match status {
      "active": text "Status: Active", color: #00cc00
      "inactive": text "Status: Inactive", color: #cc0000
      "pending": text "Status: Pending...", color: #cccc00
      _: text "Status: Unknown"
    }

    match theme {
      "dark": {
        rect width: 200px, height: 100px, color: #333333 {
          text "Dark Mode", color: #ffffff
        }
      }
      "light": {
        rect width: 200px, height: 100px, color: #eeeeee {
          text "Light Mode", color: #000000
        }
      }
      _: text "Unknown theme"
    }
  }
}"#,
    },
    Example {
        name: "Functions",
        category: "Computation",
        source: r#"-- Pure functions: compile-time inlined helpers
app "Functions" {
  state width = 200
  state height = 100

  function area(w: number, h: number) -> number {
    w * h
  }

  function double(x: number) -> number {
    x + x
  }

  computed surface = area(width, height)
  computed big-width = double(width)

  column padding: 20px, gap: 16px {
    heading "Pure Functions"
    text "Width: {width}"
    text "Height: {height}"
    text "Area: {surface}"
    text "Double width: {big-width}"
  }
}"#,
    },
    // ── State ────────────────────────────────────────────────────────────
    Example {
        name: "Computed Values",
        category: "State",
        source: r#"-- Computed values: read-only derived state that auto-updates
app "Shopping Cart" {
  state quantity = 1
  state price = 25
  computed total = quantity * price
  computed discounted = total * 0.9

  column padding: 20px, gap: 16px {
    heading "Shopping Cart"
    text "Quantity: {quantity}"
    text "Price: ${price}"
    text "Total: ${total}"
    text "With 10% discount: ${discounted}"
    row gap: 8px {
      rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
        text "Add One"
        on click: set quantity = quantity + 1
      }
      rect width: 120px, height: 40px, color: #dc2626, radius: 8px {
        text "Reset"
        on click: set quantity = 1
      }
    }
  }
}"#,
    },
    Example {
        name: "Storage",
        category: "State",
        source: r#"-- Storage: persistent state backed by localStorage/sessionStorage
app "Theme Switcher" {
  storage theme: local "theme-preference" default: "light"
  storage font-size: session "font-size" default: 16

  column padding: 20px, gap: 16px {
    heading "Settings"
    text "Theme: {theme}"
    text "Font size: {font-size}px"
    row gap: 8px {
      rect width: 120px, height: 40px, color: #2563eb, radius: 8px {
        text "Light"
        on click: set theme = "light"
      }
      rect width: 120px, height: 40px, color: #1e293b, radius: 8px {
        text "Dark"
        on click: set theme = "dark"
      }
    }
    row gap: 8px {
      rect width: 80px, height: 40px, color: #059669, radius: 8px {
        text "A+"
        on click: set font-size = font-size + 2
      }
      rect width: 80px, height: 40px, color: #dc2626, radius: 8px {
        text "A-"
        on click: set font-size = font-size - 2
      }
    }
  }
}"#,
    },
    Example {
        name: "Navigation",
        category: "State",
        source: r#"-- Multi-page apps with routing and link elements
app "Navigation Demo" {
  state current-page = "home"

  column color: #f8fafc {
    row padding: 16px, gap: 24px, color: #1e293b {
      heading "My App" color: #ffffff

      link "Home", to: "/"
      link "About", to: "/about"
      link "Contact", to: "/contact"
    }

    page "/" {
      column padding: 24px, gap: 16px {
        heading "Welcome Home"
        text "This is the home page of our multi-page app."
        text "Click the links above to navigate between pages."

        row gap: 8px {
          rect width: 100px, height: 100px, color: #3b82f6, radius: 8px
          rect width: 100px, height: 100px, color: #8b5cf6, radius: 8px
          rect width: 100px, height: 100px, color: #ec4899, radius: 8px
        }
      }
    }

    page "/about" {
      column padding: 24px, gap: 16px {
        heading "About Us"
        text "Naze is a declarative UI language that compiles to WebAssembly."

        column gap: 8px, padding: 16px, color: #f1f5f9, radius: 8px {
          text "Features:" color: #1e293b
          text "- Declarative syntax" color: #64748b
          text "- Component system" color: #64748b
          text "- State management" color: #64748b
          text "- Cross-platform rendering" color: #64748b
        }
      }
    }

    page "/contact" {
      column padding: 24px, gap: 16px {
        heading "Contact"
        text "Get in touch with us!"

        column gap: 12px, padding: 16px, color: #f1f5f9, radius: 8px {
          row gap: 8px {
            text "Email:" color: #64748b
            text "hello@naze.dev" color: #3b82f6
          }
          row gap: 8px {
            text "GitHub:" color: #64748b
            text "github.com/naze-lang" color: #3b82f6
          }
        }
      }
    }
  }
}"#,
    },
    Example {
        name: "Images & Effects",
        category: "Styling",
        source: r#"-- Images, opacity, borders, and visual effects
app "Images Demo" {
  column padding: 20px, gap: 20px {
    heading "Images & Effects"

    -- Images from URL
    row gap: 16px {
      column gap: 8px {
        text "Default (contain)"
        image src: "https://picsum.photos/seed/naze1/200/150", width: 200px, height: 150px
      }
      column gap: 8px {
        text "Cover fit"
        image src: "https://picsum.photos/seed/naze2/300/200", width: 150px, height: 150px, fit: "cover"
      }
    }

    -- Opacity examples
    heading "Opacity"
    row gap: 16px {
      rect width: 80px, height: 80px, color: #3b82f6, radius: 8px
      rect width: 80px, height: 80px, color: #3b82f6, radius: 8px, opacity: 0.75
      rect width: 80px, height: 80px, color: #3b82f6, radius: 8px, opacity: 0.5
      rect width: 80px, height: 80px, color: #3b82f6, radius: 8px, opacity: 0.25
    }

    -- Border examples
    heading "Borders"
    row gap: 16px {
      rect width: 80px, height: 80px, color: #ffffff, border: 2px, border-color: #000000
      rect width: 80px, height: 80px, color: #fef3c7, border: 3px, border-color: #f59e0b, radius: 8px
      rect width: 80px, height: 80px, color: #dcfce7, border: 4px, border-color: #22c55e, radius: 16px
      rect width: 80px, height: 80px, color: #fee2e2, border: 2px, border-color: #ef4444, radius: 40px
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
