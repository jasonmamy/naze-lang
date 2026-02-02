#![allow(dead_code)]

use naze_compiler::error::{CompileError, Severity};
use std::collections::HashMap;
use std::path::Path;

/// Output format for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Text,
    Json,
}

/// Collects source files for rendering diagnostics with source snippets.
pub struct DiagnosticPrinter {
    format: Format,
    /// Cached source file contents: file path -> source text.
    sources: HashMap<String, String>,
}

impl DiagnosticPrinter {
    pub fn new(format: Format) -> Self {
        Self {
            format,
            sources: HashMap::new(),
        }
    }

    /// Pre-load a source file for diagnostics.
    pub fn add_source(&mut self, file: &str, source: String) {
        self.sources.insert(file.to_string(), source);
    }

    /// Load a source file from disk if not already cached.
    fn ensure_source(&mut self, file: &str) {
        if !self.sources.contains_key(file) {
            if let Ok(content) = std::fs::read_to_string(file) {
                self.sources.insert(file.to_string(), content);
            } else {
                // Try relative to current dir
                let path = Path::new(file);
                if let Ok(content) = std::fs::read_to_string(path) {
                    self.sources.insert(file.to_string(), content);
                }
            }
        }
    }

    /// Print a single diagnostic.
    pub fn print(&mut self, error: &CompileError) {
        match self.format {
            Format::Text => self.print_text(error),
            Format::Json => self.print_json(error),
        }
    }

    /// Print all diagnostics.
    pub fn print_all(&mut self, errors: &[CompileError]) {
        match self.format {
            Format::Text => {
                for error in errors {
                    self.print_text(error);
                }
            }
            Format::Json => {
                let json = serde_json::to_string_pretty(errors).unwrap_or_default();
                println!("{json}");
            }
        }
    }

    /// Print a summary line (e.g., "2 error(s), 1 warning(s)").
    pub fn print_summary(&self, errors: &[CompileError]) {
        if self.format == Format::Json {
            return; // JSON output is self-contained
        }

        let num_errors = errors
            .iter()
            .filter(|e| matches!(e.severity, Severity::Error))
            .count();
        let num_warnings = errors
            .iter()
            .filter(|e| matches!(e.severity, Severity::Warning))
            .count();

        if num_errors > 0 || num_warnings > 0 {
            let mut parts = Vec::new();
            if num_errors > 0 {
                parts.push(format!(
                    "{num_errors} error{}",
                    if num_errors == 1 { "" } else { "s" }
                ));
            }
            if num_warnings > 0 {
                parts.push(format!(
                    "{num_warnings} warning{}",
                    if num_warnings == 1 { "" } else { "s" }
                ));
            }
            eprintln!("{}", parts.join(", "));
        }
    }

    fn print_text(&mut self, error: &CompileError) {
        let severity_label = match error.severity {
            Severity::Error => "\x1b[1;31merror\x1b[0m",
            Severity::Warning => "\x1b[1;33mwarning\x1b[0m",
        };

        // Header: error: message
        eprintln!("{severity_label}: {}", error.message);

        // Location: --> file:line:column
        if error.line > 0 {
            eprintln!(
                "  \x1b[1;34m-->\x1b[0m {}:{}:{}",
                error.file, error.line, error.column
            );
        } else {
            eprintln!("  \x1b[1;34m-->\x1b[0m {}", error.file);
        }

        // Source snippet with underline
        if error.line > 0 {
            self.ensure_source(&error.file);
            if let Some(source) = self.sources.get(&error.file) {
                let lines: Vec<&str> = source.lines().collect();
                let line_idx = error.line - 1;

                if line_idx < lines.len() {
                    let line_num_width = format!("{}", error.line).len();
                    let pad = " ".repeat(line_num_width);

                    // Blank line before
                    eprintln!("  {pad} \x1b[1;34m|\x1b[0m");

                    // The source line
                    eprintln!(
                        "  \x1b[1;34m{}\x1b[0m \x1b[1;34m|\x1b[0m {}",
                        error.line,
                        lines[line_idx]
                    );

                    // The underline
                    if error.column > 0 {
                        let col_offset = error.column - 1;
                        let underline_pad = " ".repeat(col_offset);
                        let indicator = match error.severity {
                            Severity::Error => "\x1b[1;31m^\x1b[0m",
                            Severity::Warning => "\x1b[1;33m^\x1b[0m",
                        };
                        eprintln!(
                            "  {pad} \x1b[1;34m|\x1b[0m {underline_pad}{indicator}"
                        );
                    }
                }
            }
        }

        eprintln!();
    }

    fn print_json(&self, error: &CompileError) {
        if let Ok(json) = serde_json::to_string(error) {
            println!("{json}");
        }
    }
}
