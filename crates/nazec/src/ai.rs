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
            return vec![CompileError::new(
                e.to_string(),
                "generated.naze".into(),
                0,
                0,
                Severity::Error,
            )];
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

const LANGUAGE_REFERENCE: &str = include_str!("../../../docs/AGENTS_SLIM.md");

fn is_finetuned_model(model: Option<&str>) -> bool {
    model
        .map(|m| m.to_lowercase().contains("naze"))
        .unwrap_or(false)
}

fn build_system_prompt(model: Option<&str>) -> String {
    if is_finetuned_model(model) {
        "You are a Naze language expert. Generate valid .naze code based on the user's description.\n\
         Output ONLY the raw .naze code. No markdown fences, no explanations, no commentary."
            .to_string()
    } else {
        format!(
            "You are a Naze language expert. Generate valid .naze code based on the user's description.\n\
             Output ONLY the raw .naze code. No markdown fences, no explanations, no commentary.\n\n\
             {}",
            LANGUAGE_REFERENCE,
        )
    }
}

fn build_fix_system_prompt(model: Option<&str>) -> String {
    if is_finetuned_model(model) {
        "You are a Naze language expert. Fix the compiler errors in the given code.\n\
         Output ONLY the corrected .naze code. No markdown fences, no explanations."
            .to_string()
    } else {
        format!(
            "You are a Naze language expert. Fix the compiler errors in the given code.\n\
             Output ONLY the corrected .naze code. No markdown fences, no explanations.\n\n\
             {}",
            LANGUAGE_REFERENCE,
        )
    }
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
    let system = build_system_prompt(model);
    // For retries, always use the full reference — error correction is
    // out-of-distribution for fine-tuned models that only learned "instruction → code"
    let retry_system = build_fix_system_prompt(None);
    let mut user_prompt = prompt.to_string();
    let mut last_code = String::new();
    let mut is_retry = false;

    for attempt in 1..=max_retries + 1 {
        eprintln!(
            "\n--- attempt {}/{} --- generating...",
            attempt,
            max_retries + 1
        );
        let t0 = std::time::Instant::now();

        let current_system = if is_retry { &retry_system } else { &system };

        let req = prompt_handlers::PromptRequest {
            provider: provider.to_string(),
            system: current_system.clone(),
            user: user_prompt.clone(),
            model: model.unwrap_or("").to_string(),
            max_tokens: 4000,
            temperature: 0.3,
        };

        let resp = prompt_handlers::execute_prompt(&req)
            .await
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        let elapsed = t0.elapsed();
        eprintln!("  completed in {:.1}s", elapsed.as_secs_f64());

        let code = extract_code(&resp.text);
        last_code = code.clone();

        let errors = validate_source(&code);
        if !has_errors(&errors) {
            eprintln!("  result: valid!");
            output_code(&code, output)?;
            return Ok(());
        }

        if attempt <= max_retries {
            let error_text = format_errors(&errors);
            eprintln!("  result: {} error(s), retrying...", errors.len());
            is_retry = true;
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

    let system = build_fix_system_prompt(model);
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
            "\n--- attempt {}/{} --- fixing {} error(s)...",
            attempt,
            max_retries,
            current_errors.len()
        );
        let t0 = std::time::Instant::now();

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

        let elapsed = t0.elapsed();
        eprintln!("  completed in {:.1}s", elapsed.as_secs_f64());

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
            CompileError::new(
                "unknown element 'colum'".into(),
                "test.naze".into(),
                2,
                3,
                Severity::Error,
            ),
            CompileError::new(
                "something".into(),
                "test.naze".into(),
                0,
                0,
                Severity::Warning,
            ),
        ];
        let text = format_errors(&errors);
        assert_eq!(text, "Line 2: unknown element 'colum'");
    }

    #[test]
    fn test_language_reference_examples_validate() {
        // Extract ```naze code blocks from AGENTS_SLIM.md and verify they compile
        let mut in_block = false;
        let mut block = String::new();
        let mut blocks = Vec::new();

        for line in LANGUAGE_REFERENCE.lines() {
            if line.starts_with("```naze") {
                in_block = true;
                block.clear();
            } else if line.starts_with("```") && in_block {
                in_block = false;
                if !block.trim().is_empty() {
                    blocks.push(block.clone());
                }
            } else if in_block {
                block.push_str(line);
                block.push('\n');
            }
        }

        // Only validate complete app/component examples (skip fragments)
        let full_examples: Vec<_> = blocks
            .iter()
            .filter(|b| b.contains("app ") || b.contains("component "))
            .collect();

        assert!(
            full_examples.len() >= 4,
            "expected at least 4 complete examples in AGENTS_SLIM.md, found {}",
            full_examples.len()
        );

        for (i, code) in full_examples.iter().enumerate() {
            let errors = validate_source(code);
            assert!(
                !has_errors(&errors),
                "AGENTS_SLIM.md example {} has errors: {:?}\n\nCode:\n{}",
                i + 1,
                errors,
                code
            );
        }
    }
}
