//! AI code generation, validation feedback loop, and fine-tuning dataset tools.
//! Implements `nazec ai generate`, `nazec ai fix`, and `nazec ai dataset` subcommands.

use std::collections::HashMap;
use std::path::Path;

use naze_compiler::error::{CompileError, Severity};
use naze_compiler::resolve;
use naze_compiler::typecheck;

use crate::cli::{AiCommand, DatasetCommand};
use crate::prompt_handlers;

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn run(cmd: AiCommand) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { run_async(cmd).await })
}

async fn run_async(cmd: AiCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        AiCommand::Generate {
            prompt,
            provider,
            model,
            retries,
            output,
        } => {
            do_generate(
                &prompt,
                &provider,
                model.as_deref(),
                retries,
                output.as_deref(),
            )
            .await
        }
        AiCommand::Fix {
            file,
            provider,
            model,
            retries,
        } => do_fix(&file, &provider, model.as_deref(), retries).await,
        AiCommand::Dataset { subcommand } => match subcommand {
            DatasetCommand::Export {
                dir,
                provider,
                model,
                output,
            } => do_dataset_export(&dir, &provider, model.as_deref(), &output).await,
            DatasetCommand::Validate { file } => do_dataset_validate(&file),
        },
    }
}

// ─── Compiler validation ─────────────────────────────────────────────────────

/// Validate a .naze source string without needing a project manifest.
/// Returns an empty vec if the code is valid.
fn validate_source(source: &str) -> Vec<CompileError> {
    // 1. Parse
    let nodes = match naze_parser::parse(source, "generated.naze") {
        Ok(n) => n,
        Err(e) => {
            return vec![CompileError {
                message: e.to_string(),
                file: "generated.naze".into(),
                line: 0,
                column: 0,
                severity: Severity::Error,
            }];
        }
    };

    // 2. Build a minimal ResolvedProject (no imports, no external deps)
    let project = resolve::ResolvedProject {
        entry: resolve::SourceFile {
            path: "generated.naze".into(),
            nodes,
        },
        components: HashMap::new(),
        themes: vec![],
        imports: vec![],
        errors: vec![],
    };

    // 3. Typecheck
    typecheck::typecheck(&project)
}

/// Format compiler errors as a human-readable string for LLM feedback.
fn format_errors(errors: &[CompileError]) -> String {
    errors
        .iter()
        .filter(|e| matches!(e.severity, Severity::Error))
        .map(|e| {
            if e.line > 0 {
                format!("Line {}: {}", e.line, e.message)
            } else {
                e.message.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Check if errors contain actual errors (not just warnings).
fn has_errors(errors: &[CompileError]) -> bool {
    errors.iter().any(|e| matches!(e.severity, Severity::Error))
}

// ─── Code extraction ─────────────────────────────────────────────────────────

/// Extract .naze code from an LLM response, stripping markdown fences if present.
fn extract_code(response: &str) -> String {
    let trimmed = response.trim();

    // Try to find a fenced code block
    if let Some(start) = trimmed.find("```") {
        let after_fence = &trimmed[start + 3..];
        // Skip language tag (e.g., "naze", "naze-lang")
        let code_start = after_fence.find('\n').map(|i| i + 1).unwrap_or(0);
        let code_body = &after_fence[code_start..];
        if let Some(end) = code_body.find("```") {
            return code_body[..end].trim().to_string();
        }
        // No closing fence, take everything after opening
        return code_body.trim().to_string();
    }

    // No fences — use the response as-is
    trimmed.to_string()
}

// ─── Prompt templates ────────────────────────────────────────────────────────

const LANGUAGE_REFERENCE: &str = include_str!("../../../docs/AGENTS.md");

const EXAMPLE_COUNTER: &str = r#"-- Counter with increment and reset
app "Counter" {
  state count = 0
  column padding: 20px, gap: 16px {
    heading "My Counter"
    text "Count: {count}"
    rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
      text "Increment"
      on click: set count = count + 1
    }
    rect width: 200px, height: 50px, color: #dc2626, radius: 8px {
      text "Reset"
      on click: set count = 0
    }
  }
}"#;

const EXAMPLE_DASHBOARD: &str = r#"-- Dashboard layout with header, sidebar, and metric cards
app "Dashboard" {
  column gap: 0px {
    container padding: 16px, color: #1e293b {
      heading "Dashboard" font-size: 20px, color: #ffffff
    }
    row padding: 20px, gap: 20px {
      column width: 200px, gap: 8px, padding: 16px, color: #f8fafc {
        text "Overview"
        text "Analytics"
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
        }
      }
    }
  }
}"#;

const EXAMPLE_DATA_FETCH: &str = r#"-- Data fetching with loading/error states
app "Posts" {
  data posts: fetch "https://jsonplaceholder.typicode.com/posts?_limit=5"

  column gap: 16px, padding: 20px {
    heading "API Data"

    if posts.loading {
      text "Loading...", color: #666666
    }

    if posts.error {
      text "Error: {posts.error}", color: #dc2626
    }

    if posts.data {
      each post in posts.data {
        column padding: 12px, color: #f3f4f6, radius: 8px {
          heading "{post.title}", font-size: 16px
          text "{post.body}", color: #666666, font-size: 14px
        }
      }
    }
  }
}"#;

const EXAMPLE_PIPELINE: &str = r#"-- Pipeline operators: filter, sort, aggregate
app "Student Scores" {
  state students = [{name: "Alice", score: 92}, {name: "Bob", score: 67}, {name: "Carol", score: 85}, {name: "Dave", score: 45}]

  computed passing = students | filter score > 60
  computed total-score = students | map score | sum
  computed student-count = students | count

  column padding: 20px, gap: 16px {
    heading "Student Scores"
    text "Total students: {student-count}"
    text "Total score: {total-score}"

    heading "Passing (score > 60):" font-size: 18px
    each student in students | filter score > 60 | sort-by name {
      text "{student.name}: {student.score}"
    }
  }
}"#;

const EXAMPLE_FORM: &str = r#"-- Form with validation and error display
app "Sign Up" {
  state username = ""
  state email = ""

  column padding: 20px, gap: 12px {
    heading "Create Account"

    text "Username"
    input bind: username, placeholder: "Enter username", validate: { required: true, min-length: 3, max-length: 20 }
    if username_error {
      text "{username_error}" color: #dc2626
    }

    text "Email"
    input bind: email, type: "email", placeholder: "Enter email", validate: { required: true }
    if email_error {
      text "{email_error}" color: #dc2626
    }

    row gap: 8px {
      if username_valid {
        text "Username OK" color: #16a34a
      }
      if email_valid {
        text "Email OK" color: #16a34a
      }
    }
  }
}"#;

const EXAMPLE_ROUTING: &str = r#"-- Multi-page app with navigation
app "My Site" {
  row padding: 16px, gap: 24px, color: #1e293b {
    heading "My App" color: #ffffff, font-size: 18px
    link "Home", to: "/"
    link "About", to: "/about"
  }

  page "/" {
    column padding: 24px, gap: 16px {
      heading "Welcome Home"
      text "Click the links above to navigate."
    }
  }

  page "/about" {
    column padding: 24px, gap: 16px {
      heading "About Us"
      text "Built with Naze."
    }
  }
}"#;

const EXAMPLE_COMPONENT: &str = r#"-- Component definition with typed parameters and defaults
component card(bg: color = #ffffff, width: number = 200px) {
  container padding: 16px, color: bg, radius: 8px, width: width {
    column gap: 8px {
      heading "Card Title" font-size: 16px
      text "Card content goes here."
    }
  }
}"#;

const EXAMPLE_MATCH: &str = r#"-- Pattern matching for conditional rendering
app "Match Demo" {
  state status = "active"

  column padding: 20px, gap: 16px {
    heading "Pattern Matching"

    match status {
      "active": text "Status: Active" color: #16a34a
      "inactive": text "Status: Inactive" color: #dc2626
      "pending": text "Status: Pending..." color: #eab308
      _: text "Status: Unknown"
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "Active" color: #ffffff
        on click: set status = "active"
      }
      rect width: 80px, height: 36px, color: #dc2626, radius: 4px {
        text "Inactive" color: #ffffff
        on click: set status = "inactive"
      }
    }
  }
}"#;

const EXAMPLE_THEME: &str = r#"-- Named themes with extends and runtime switching
theme light {
  colors {
    bg: #ffffff
    fg: #0f172a
    primary: #2563eb
  }
  spacing {
    sm: 8px
    md: 16px
  }
}

theme dark extends light {
  colors {
    bg: #1e293b
    fg: #f8fafc
    primary: #60a5fa
  }
}

app "Theme Demo" {
  column padding: 20px, gap: 16px, color: theme.colors.bg {
    heading "Theme Switching" color: theme.colors.fg
    row gap: 12px {
      rect width: 100px, height: 40px, color: theme.colors.primary, radius: 8px {
        text "Light" color: #ffffff
        on click: set-theme "light"
      }
      rect width: 100px, height: 40px, color: theme.colors.primary, radius: 8px {
        text "Dark" color: #ffffff
        on click: set-theme "dark"
      }
    }
  }
}"#;

const EXAMPLE_ANIMATION: &str = r#"-- Transitions and interactive toggle
app "Animation" {
  state expanded = false

  column gap: 16px, padding: 20px {
    heading "Animation Demo"

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
}"#;

fn build_system_prompt() -> String {
    format!(
        "You are a Naze language expert. Generate valid .naze code based on the user's description.\n\
         Output ONLY the raw .naze code. No markdown fences, no explanations, no commentary.\n\n\
         {}\n\n\
         ## Additional Examples\n\n\
         ### Counter app\n```\n{}\n```\n\n\
         ### Dashboard layout\n```\n{}\n```\n\n\
         ### Data fetching\n```\n{}\n```\n\n\
         ### Pipeline operators\n```\n{}\n```\n\n\
         ### Form with validation\n```\n{}\n```\n\n\
         ### Multi-page routing\n```\n{}\n```\n\n\
         ### Component definition\n```\n{}\n```\n\n\
         ### Pattern matching\n```\n{}\n```\n\n\
         ### Theming\n```\n{}\n```\n\n\
         ### Animation\n```\n{}\n```",
        LANGUAGE_REFERENCE,
        EXAMPLE_COUNTER,
        EXAMPLE_DASHBOARD,
        EXAMPLE_DATA_FETCH,
        EXAMPLE_PIPELINE,
        EXAMPLE_FORM,
        EXAMPLE_ROUTING,
        EXAMPLE_COMPONENT,
        EXAMPLE_MATCH,
        EXAMPLE_THEME,
        EXAMPLE_ANIMATION,
    )
}

fn build_fix_system_prompt() -> String {
    format!(
        "You are a Naze language expert. Fix the compiler errors in the given code.\n\
         Output ONLY the corrected .naze code. No markdown fences, no explanations.\n\n\
         {}",
        LANGUAGE_REFERENCE,
    )
}

fn build_describe_system_prompt() -> String {
    "You are a Naze language expert. Given a .naze source file, write a single sentence \
     describing what UI it creates, phrased as a generation instruction. \
     Example: \"Create a counter app with increment and reset buttons.\"\n\
     Output ONLY the instruction sentence, nothing else."
        .to_string()
}

// ─── Generate command ────────────────────────────────────────────────────────

async fn do_generate(
    prompt: &str,
    provider: &str,
    model: Option<&str>,
    max_retries: u32,
    output: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let system = build_system_prompt();
    let mut user_prompt = prompt.to_string();
    let mut last_code = String::new();

    for attempt in 1..=max_retries + 1 {
        eprintln!("  generating... (attempt {}/{})", attempt, max_retries + 1);

        let req = prompt_handlers::PromptRequest {
            provider: provider.to_string(),
            system: system.clone(),
            user: user_prompt.clone(),
            model: model.unwrap_or("").to_string(),
            max_tokens: 4000,
            temperature: 0.3,
        };

        let resp = prompt_handlers::execute_prompt(&req)
            .await
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        let code = extract_code(&resp.text);
        last_code = code.clone();

        let errors = validate_source(&code);
        if !has_errors(&errors) {
            eprintln!("  valid!");
            output_code(&code, output)?;
            return Ok(());
        }

        if attempt <= max_retries {
            let error_text = format_errors(&errors);
            eprintln!("  {} error(s), retrying...", errors.len());
            user_prompt = format!(
                "The following .naze code has compiler errors. Fix them and output ONLY the corrected code.\n\n\
                 ## Code\n{}\n\n## Errors\n{}",
                code, error_text
            );
        } else {
            eprintln!(
                "  warning: could not produce valid code after {} attempts",
                max_retries + 1
            );
            let error_text = format_errors(&errors);
            eprintln!("  remaining errors:\n{}", error_text);
        }
    }

    // Output last attempt even if invalid
    output_code(&last_code, output)?;
    Ok(())
}

fn output_code(code: &str, output: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    match output {
        Some(path) => {
            std::fs::write(path, code)?;
            eprintln!("  wrote {}", path);
        }
        None => println!("{}", code),
    }
    Ok(())
}

// ─── Fix command ─────────────────────────────────────────────────────────────

async fn do_fix(
    file: &str,
    provider: &str,
    model: Option<&str>,
    max_retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(file)?;

    // Check if already valid
    let errors = validate_source(&source);
    if !has_errors(&errors) {
        eprintln!("  no errors found in {}", file);
        return Ok(());
    }

    let system = build_fix_system_prompt();
    let mut current_code = source;

    for attempt in 1..=max_retries {
        let current_errors = validate_source(&current_code);
        if !has_errors(&current_errors) {
            eprintln!("  fixed! writing {}", file);
            std::fs::write(file, &current_code)?;
            return Ok(());
        }

        let error_text = format_errors(&current_errors);
        eprintln!(
            "  fixing... (attempt {}/{}) — {} error(s)",
            attempt,
            max_retries,
            current_errors.len()
        );

        let user_prompt = format!(
            "Fix the compiler errors in this code.\n\n## Code\n{}\n\n## Errors\n{}",
            current_code, error_text
        );

        let req = prompt_handlers::PromptRequest {
            provider: provider.to_string(),
            system: system.clone(),
            user: user_prompt,
            model: model.unwrap_or("").to_string(),
            max_tokens: 4000,
            temperature: 0.2,
        };

        let resp = prompt_handlers::execute_prompt(&req)
            .await
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        current_code = extract_code(&resp.text);
    }

    // Final check
    let final_errors = validate_source(&current_code);
    if !has_errors(&final_errors) {
        eprintln!("  fixed! writing {}", file);
        std::fs::write(file, &current_code)?;
    } else {
        eprintln!(
            "  could not fully fix after {} attempts, writing best attempt",
            max_retries
        );
        let error_text = format_errors(&final_errors);
        eprintln!("  remaining errors:\n{}", error_text);
        std::fs::write(file, &current_code)?;
    }

    Ok(())
}

// ─── Dataset export ──────────────────────────────────────────────────────────

async fn do_dataset_export(
    dir: &str,
    provider: &str,
    model: Option<&str>,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = Path::new(dir);
    if !dir_path.is_dir() {
        return Err(format!("'{}' is not a directory", dir).into());
    }

    // Collect all .naze files (not .test.naze)
    let mut naze_files: Vec<std::path::PathBuf> = Vec::new();
    collect_naze_files(dir_path, &mut naze_files)?;
    naze_files.sort();

    eprintln!("  found {} .naze files in {}", naze_files.len(), dir);

    let system = build_describe_system_prompt();
    let mut out = std::fs::File::create(output)?;
    use std::io::Write;

    let mut exported = 0;
    let mut skipped = 0;

    for (i, path) in naze_files.iter().enumerate() {
        let source = std::fs::read_to_string(path)?;

        // Skip files that don't compile
        let errors = validate_source(&source);
        if has_errors(&errors) {
            eprintln!(
                "  [{}/{}] skipping {} (compile errors)",
                i + 1,
                naze_files.len(),
                path.display()
            );
            skipped += 1;
            continue;
        }

        eprintln!(
            "  [{}/{}] describing {}",
            i + 1,
            naze_files.len(),
            path.display()
        );

        let req = prompt_handlers::PromptRequest {
            provider: provider.to_string(),
            system: system.clone(),
            user: source.clone(),
            model: model.unwrap_or("").to_string(),
            max_tokens: 200,
            temperature: 0.3,
        };

        match prompt_handlers::execute_prompt(&req).await {
            Ok(resp) => {
                let instruction = resp.text.trim().to_string();
                let entry = serde_json::json!({
                    "instruction": instruction,
                    "response": source.trim(),
                });
                writeln!(out, "{}", serde_json::to_string(&entry)?)?;
                exported += 1;
            }
            Err(e) => {
                eprintln!("  warning: failed to describe {}: {}", path.display(), e);
                skipped += 1;
            }
        }
    }

    eprintln!(
        "  exported {} pairs, skipped {} — wrote {}",
        exported, skipped, output
    );
    Ok(())
}

fn collect_naze_files(
    dir: &Path,
    files: &mut Vec<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_naze_files(&path, files)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".naze") && !name.ends_with(".test.naze") {
                files.push(path);
            }
        }
    }
    Ok(())
}

// ─── Dataset validate ────────────────────────────────────────────────────────

fn do_dataset_validate(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file)?;
    let mut total = 0;
    let mut valid = 0;
    let mut invalid = 0;

    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        total += 1;

        let entry: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("  line {}: invalid JSON: {}", i + 1, e);
                invalid += 1;
                continue;
            }
        };

        let response = match entry.get("response").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                eprintln!("  line {}: missing 'response' field", i + 1);
                invalid += 1;
                continue;
            }
        };

        let errors = validate_source(response);
        if has_errors(&errors) {
            let error_text = format_errors(&errors);
            eprintln!("  line {}: compile errors: {}", i + 1, error_text);
            invalid += 1;
        } else {
            valid += 1;
        }
    }

    eprintln!("  {} total, {} valid, {} invalid", total, valid, invalid);
    if invalid > 0 {
        return Err(format!("{} entries failed validation", invalid).into());
    }
    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_source_valid() {
        let source = r#"app "Test" {
  column padding: 20px {
    text "Hello"
  }
}"#;
        let errors = validate_source(source);
        assert!(!has_errors(&errors), "expected no errors: {:?}", errors);
    }

    #[test]
    fn test_validate_source_parse_error() {
        let source = r#"app "Test" { colum { text "hi" }"#;
        let errors = validate_source(source);
        assert!(has_errors(&errors), "expected parse error");
    }

    #[test]
    fn test_validate_source_empty_app() {
        let source = r#"app "Empty" {}"#;
        let errors = validate_source(source);
        assert!(!has_errors(&errors));
    }

    #[test]
    fn test_extract_code_no_fences() {
        let resp = "app \"Test\" {\n  text \"hello\"\n}";
        assert_eq!(extract_code(resp), "app \"Test\" {\n  text \"hello\"\n}");
    }

    #[test]
    fn test_extract_code_with_fences() {
        let resp = "Here is the code:\n```naze\napp \"Test\" {\n  text \"hello\"\n}\n```\nDone.";
        assert_eq!(extract_code(resp), "app \"Test\" {\n  text \"hello\"\n}");
    }

    #[test]
    fn test_extract_code_with_bare_fences() {
        let resp = "```\napp \"Test\" {}\n```";
        assert_eq!(extract_code(resp), "app \"Test\" {}");
    }

    #[test]
    fn test_format_errors() {
        let errors = vec![
            CompileError {
                message: "unknown element 'colum'".into(),
                file: "test.naze".into(),
                line: 2,
                column: 3,
                severity: Severity::Error,
            },
            CompileError {
                message: "something".into(),
                file: "test.naze".into(),
                line: 0,
                column: 0,
                severity: Severity::Warning,
            },
        ];
        let text = format_errors(&errors);
        assert_eq!(text, "Line 2: unknown element 'colum'");
    }

    #[test]
    fn test_all_examples_validate() {
        // Verify that all embedded few-shot examples compile
        for (name, code) in [
            ("counter", EXAMPLE_COUNTER),
            ("dashboard", EXAMPLE_DASHBOARD),
            ("data-fetch", EXAMPLE_DATA_FETCH),
            ("pipeline", EXAMPLE_PIPELINE),
            ("form", EXAMPLE_FORM),
            ("routing", EXAMPLE_ROUTING),
            ("component", EXAMPLE_COMPONENT),
            ("match", EXAMPLE_MATCH),
            ("theme", EXAMPLE_THEME),
            ("animation", EXAMPLE_ANIMATION),
        ] {
            let errors = validate_source(code);
            assert!(
                !has_errors(&errors),
                "example '{}' has errors: {:?}",
                name,
                errors
            );
        }
    }
}
