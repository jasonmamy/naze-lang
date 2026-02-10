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

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum BuildTarget {
    /// Build for web (WASM + HTML)
    Web,
    /// Build standalone native binary
    Native,
    /// Build Android project with WebView
    Android,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new Naze project
    New {
        /// Project name
        name: String,
    },
    /// Compile the project
    Build {
        /// Target platform
        #[arg(short, long, default_value = "web")]
        target: BuildTarget,
    },
    /// Type-check without building
    Check,
    /// Preview the built app in a native desktop window
    Run,
    /// Start dev server with hot reload for browser development
    Dev {
        /// Port to run the server on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Automatically open browser
        #[arg(short, long)]
        open: bool,
    },
    /// Parse a .naze file and dump the AST as JSON
    Parse {
        /// Path to the .naze file
        file: String,
    },
    /// Run tests from .test.naze files
    Test {
        /// Run only tests matching this pattern
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Build and serve example gallery
    Gallery {
        /// Only build, don't serve
        #[arg(long)]
        build: bool,

        /// Show gallery in a native window instead of browser
        #[arg(long)]
        native: bool,
    },
}
