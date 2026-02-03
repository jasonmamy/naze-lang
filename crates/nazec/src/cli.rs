use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "nazec", about = "The Naze compiler and build tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output format for errors and diagnostics
    #[arg(long, global = true, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable terminal output with source snippets
    Text,
    /// Machine-readable JSON output
    Json,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new Naze project
    New {
        /// Project name
        name: String,
    },
    /// Compile the project to WASM
    Build,
    /// Type-check without building
    Check,
    /// Preview the built app in a native desktop window
    Run,
    /// Parse a .naze file and dump the AST as JSON
    Parse {
        /// Path to the .naze file
        file: String,
    },
    /// Build and serve example gallery
    Gallery {
        /// Only build, don't serve
        #[arg(long)]
        build: bool,
    },
}
