mod ai;
mod analyze;
mod android_build;
mod build;
mod cli;
mod dep_commands;
mod deps;
mod dev;
mod diagnostic;
mod exec;
mod gallery;
mod grammar;
mod html_renderer;
mod manifest;
mod native_build;
mod native_renderer;
mod new;
mod playground;
mod prompt_handlers;
mod registry;
mod run;
mod seo;
mod serve;
mod server_fns;
mod test_runner;

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
        Command::Build {
            target,
            static_render,
        } => do_build(target, format, static_render),
        Command::Check => do_check(format),
        Command::Run => do_run(),
        Command::Dev { port, open } => do_dev(port, open),
        Command::Serve { port, host } => do_serve(port, &host),
        Command::Parse { file } => parse_file(&file),
        Command::Test { filter } => do_test(filter.as_deref(), format),
        Command::Gallery { build, native } => gallery::run(build, native),
        Command::Grammar {
            grammar_format,
            no_test,
        } => grammar::run(grammar_format, no_test),
        Command::Add {
            package,
            path,
            git,
            tag,
            branch,
            rev,
            version,
        } => dep_commands::add(
            &package,
            path.as_deref(),
            git.as_deref(),
            tag.as_deref(),
            branch.as_deref(),
            rev.as_deref(),
            version.as_deref(),
        ),
        Command::Remove { package } => dep_commands::remove(&package),
        Command::Update { package } => dep_commands::update(package.as_deref()),
        Command::Publish { registry } => registry::publish_package(registry.as_deref()),
        Command::Search {
            query,
            limit,
            registry,
        } => registry::search_packages(&query, limit, registry.as_deref()),
        Command::Analyze { bin, wasm, compare } => {
            analyze::run(&bin, wasm.as_deref(), compare.as_deref())
        }
        Command::Playground { port } => playground::run(port),
        Command::Ai { subcommand } => ai::run(subcommand),
    };

    if let Err(e) = result {
        if format == Format::Text {
            eprintln!("error: {e}");
        }
        std::process::exit(1);
    }
}

fn do_build(
    target: BuildTarget,
    format: Format,
    static_render: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    if format == Format::Text {
        let target_str = match target {
            BuildTarget::Web => {
                if static_render {
                    "web (static)"
                } else {
                    "web"
                }
            }
            BuildTarget::Native => "native",
            BuildTarget::Android => "android",
        };
        eprintln!(
            "building {} v{} ({})",
            manifest.app.name, manifest.app.version, target_str
        );
    }
    let resolved_deps = deps::resolve_deps(&manifest, std::path::Path::new("."))?;
    match target {
        BuildTarget::Web => build::run(&manifest, format, &resolved_deps, static_render),
        BuildTarget::Native => native_build::run(&manifest, format),
        BuildTarget::Android => android_build::run(&manifest, format),
    }
}

fn do_check(format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    if format == Format::Text {
        eprintln!("checking {} v{}", manifest.app.name, manifest.app.version);
    }
    let resolved_deps = deps::resolve_deps(&manifest, std::path::Path::new("."))?;
    build::check(&manifest, format, &resolved_deps)
}

fn do_run() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    run::run(&manifest)
}

fn do_dev(port: u16, open: bool) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    dev::run(&manifest, port, open)
}

fn do_serve(port: u16, host: &str) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = manifest::load("naze.toml")?;
    serve::run(&manifest, port, host)
}

fn do_test(filter: Option<&str>, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = std::path::Path::new(".");
    let suites = test_runner::run_all(project_dir, filter)?;

    match format {
        Format::Text => test_runner::print_results_text(&suites),
        Format::Json => test_runner::print_results_json(&suites),
    }

    let failed: usize = suites.iter().map(|s| s.failed).sum();
    if failed > 0 {
        return Err(format!("{} test(s) failed", failed).into());
    }
    Ok(())
}

fn parse_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    if path.ends_with(".test.naze") {
        let test_file = naze_parser::parse_test_file(&source, path)
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
        let json = serde_json::to_string_pretty(&test_file)?;
        println!("{json}");
    } else {
        let nodes = naze_parser::parse(&source, path)
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
        let json = serde_json::to_string_pretty(&nodes)?;
        println!("{json}");
    }
    Ok(())
}
