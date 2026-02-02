mod build;
mod cli;
mod diagnostic;
mod manifest;
mod native_renderer;
mod new;
mod run;

use clap::Parser;
use cli::{Cli, Command, OutputFormat};
use diagnostic::Format;

fn main() {
    let cli = Cli::parse();

    let format = match cli.format {
        OutputFormat::Text => Format::Text,
        OutputFormat::Json => Format::Json,
    };

    let result = match cli.command {
        Command::New { name } => new::run(&name),
        Command::Build => do_build(format),
        Command::Check => do_check(format),
        Command::Run => do_run(),
        Command::Parse { file } => parse_file(&file),
    };

    if let Err(e) = result {
        if format == Format::Text {
            eprintln!("error: {e}");
        }
        std::process::exit(1);
    }
}

fn do_build(format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    if format == Format::Text {
        eprintln!("building {} v{}", manifest.app.name, manifest.app.version);
    }
    build::run(&manifest, format)
}

fn do_check(format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    if format == Format::Text {
        eprintln!("checking {} v{}", manifest.app.name, manifest.app.version);
    }
    build::check(&manifest, format)
}

fn do_run() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    run::run(&manifest)
}

fn parse_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let nodes = naze_parser::parse(&source, path)
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
    let json = serde_json::to_string_pretty(&nodes)?;
    println!("{json}");
    Ok(())
}
