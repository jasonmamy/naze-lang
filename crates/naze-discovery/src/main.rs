mod api;
mod dashboard;
mod extractor_json;
mod identity_apikey;
mod matcher_sql;
mod storage_sqlite;
mod sync_stub;
mod traits;
mod trust_simple;
mod types;

use clap::Parser;

#[derive(Parser)]
#[command(name = "naze-discovery", about = "Naze discovery network reference server")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8889")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Directory to store data and database
    #[arg(short, long, default_value = "./discovery-data")]
    data_dir: String,

    /// Network identity name
    #[arg(long, default_value = "default")]
    network_id: String,

    /// Network scope (public or private)
    #[arg(long, default_value = "public")]
    scope: String,

    /// API key required for write operations (registration, flagging)
    #[arg(long)]
    write_key: Option<String>,

    /// API key required for read operations (search, download)
    #[arg(long)]
    read_key: Option<String>,

    /// Convenience flag: sets both write_key and read_key
    #[arg(long)]
    api_key: Option<String>,

    /// Disable the built-in web dashboard
    #[arg(long)]
    no_dashboard: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let write_key = args.write_key.or_else(|| args.api_key.clone());
    let read_key = args.read_key.or(args.api_key);

    let storage = storage_sqlite::SqliteStorage::open(&args.data_dir)?;
    let scorer = trust_simple::SimpleScorer::new();
    let matcher = matcher_sql::SqlMatcher::new();
    let extractor = extractor_json::JsonExtractor::new();
    let identity = identity_apikey::ApiKeyVerifier::new(write_key, read_key);
    let sync = sync_stub::StubSync::new();

    let state = api::AppState {
        storage: Box::new(storage),
        scorer: Box::new(scorer),
        matcher: Box::new(matcher),
        extractor: Box::new(extractor),
        identity: Box::new(identity),
        sync: Box::new(sync),
        network_id: args.network_id,
        scope: args.scope,
    };

    let app = api::router(state);

    let addr = format!("{}:{}", args.host, args.port);
    eprintln!("naze-discovery listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
