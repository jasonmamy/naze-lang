mod android_build;
mod build;
mod cli;
mod dev;
mod diagnostic;
mod gallery;
mod manifest;
mod native_build;
mod native_renderer;
mod new;
mod run;

use clap::Parser;
use cli::{BuildTarget, Cli, Command, OutputFormat};
use diagnostic::Format;

fn main() {
    let cli = Cli::parse();

    let format = match cli.format {
        OutputFormat::Text => Format::Text,
        OutputFormat::Json => Format::Json,
    };

    let result = match cli.command {
        Command::New { name } => new::run(&name),
        Command::Build { target } => do_build(target, format),
        Command::Check => do_check(format),
        Command::Run => do_run(),
        Command::Dev { port, open } => do_dev(port, open),
        Command::Parse { file } => parse_file(&file),
        Command::Gallery { build, native } => gallery::run(build, native),
    };

    if let Err(e) = result {
        if format == Format::Text {
            eprintln!("error: {e}");
        }
        std::process::exit(1);
    }
}

fn do_build(target: BuildTarget, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    if format == Format::Text {
        let target_str = match target {
            BuildTarget::Web => "web",
            BuildTarget::Native => "native",
            BuildTarget::Android => "android",
        };
        eprintln!(
            "building {} v{} ({})",
            manifest.app.name, manifest.app.version, target_str
        );
    }
    match target {
        BuildTarget::Web => build::run(&manifest, format),
        BuildTarget::Native => native_build::run(&manifest, format),
        BuildTarget::Android => android_build::run(&manifest, format),
    }
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

fn do_dev(port: u16, open: bool) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    dev::run(&manifest, port, open)
}

fn parse_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let nodes = naze_parser::parse(&source, path)
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
    let json = serde_json::to_string_pretty(&nodes)?;
    println!("{json}");
    Ok(())
}
