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
pub enum GrammarFormat {
    /// GBNF format for llama.cpp / XGrammar / SGLang constrained decoding
    Gbnf,
    /// Human-readable EBNF for documentation
    Ebnf,
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
        /// Pre-render all routes to static HTML at build time (SSG)
        #[arg(long = "static")]
        static_render: bool,
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
    /// Export the Naze grammar for AI constrained decoding
    Grammar {
        /// Grammar output format (gbnf for llama.cpp/XGrammar, ebnf for docs)
        #[arg(short = 'g', long = "grammar-format", default_value = "gbnf")]
        grammar_format: GrammarFormat,

        /// Exclude test grammar rules
        #[arg(long)]
        no_test: bool,
    },
    /// Start a production SSR server (requires prior `nazec build`)
    Serve {
        /// Port to run the server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Bind address
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
    },
    /// Add a dependency to naze.toml
    Add {
        /// Package name (e.g., "@naze/ui-kit")
        package: String,
        /// Local path to the package
        #[arg(long)]
        path: Option<String>,
        /// Git repository URL
        #[arg(long)]
        git: Option<String>,
        /// Git tag
        #[arg(long)]
        tag: Option<String>,
        /// Git branch
        #[arg(long)]
        branch: Option<String>,
        /// Git commit hash
        #[arg(long)]
        rev: Option<String>,
        /// Version constraint (e.g., "^1.0") — fetches from registry
        #[arg(long)]
        version: Option<String>,
    },
    /// Remove a dependency from naze.toml
    Remove {
        /// Package name to remove
        package: String,
    },
    /// Update dependencies to latest matching versions
    Update {
        /// Specific package to update (all if omitted)
        package: Option<String>,
    },
    /// Publish the current package to a registry
    Publish {
        /// Registry URL (overrides NAZE_REGISTRY_URL env var)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Search the package registry
    Search {
        /// Search query
        query: String,
        /// Max results to return
        #[arg(long, default_value = "20")]
        limit: u32,
        /// Registry URL (overrides NAZE_REGISTRY_URL env var)
        #[arg(long)]
        registry: Option<String>,
    },
    /// Analyze binary sizes (app_data.bin and WASM)
    Analyze {
        /// Path to app_data.bin
        #[arg(long, default_value = "dist/app_data.bin")]
        bin: String,
        /// Path to WASM binary for section breakdown
        #[arg(long)]
        wasm: Option<String>,
        /// Compare against another binary (shows delta)
        #[arg(long)]
        compare: Option<String>,
    },
    /// Start the interactive playground
    Playground {
        /// Port for playground server
        #[arg(long, default_value = "4000")]
        port: u16,
    },
    /// Export project context as JSON for AI agents
    Context,
    /// AI code generation and dataset tools
    Ai {
        #[command(subcommand)]
        subcommand: AiCommand,
    },
}

#[derive(Subcommand)]
pub enum AiCommand {
    /// Generate a .naze file from a natural language description
    Generate {
        /// Description of the UI to generate
        prompt: String,
        /// AI provider (openai, anthropic, ollama)
        #[arg(long, default_value = "openai")]
        provider: String,
        /// Model override (defaults to provider's best)
        #[arg(long)]
        model: Option<String>,
        /// Max validation retries
        #[arg(long, default_value = "3")]
        retries: u32,
        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Auto-fix compiler errors in a .naze file using AI
    Fix {
        /// Path to the .naze file to fix
        file: String,
        /// AI provider
        #[arg(long, default_value = "openai")]
        provider: String,
        /// Model override
        #[arg(long)]
        model: Option<String>,
        /// Max fix attempts
        #[arg(long, default_value = "3")]
        retries: u32,
    },
    /// Generate a fine-tuning dataset from examples
    Dataset {
        #[command(subcommand)]
        subcommand: DatasetCommand,
    },
}

#[derive(Subcommand)]
pub enum DatasetCommand {
    /// Export examples as instruction/response JSONL training pairs
    Export {
        /// Directory containing .naze examples
        #[arg(long, default_value = "examples")]
        dir: String,
        /// AI provider for generating descriptions
        #[arg(long, default_value = "openai")]
        provider: String,
        /// Model override
        #[arg(long)]
        model: Option<String>,
        /// Output file
        #[arg(short, long, default_value = "dataset.jsonl")]
        output: String,
    },
    /// Validate all .naze files in a JSONL dataset compile correctly
    Validate {
        /// Path to the JSONL dataset file
        file: String,
    },
}
