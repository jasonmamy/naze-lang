mod api;
mod db;
mod storage;

use clap::Parser;

#[derive(Parser)]
#[command(name = "naze-registry", about = "Naze package registry server")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8888")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Directory to store packages and database
    #[arg(short, long, default_value = "./registry-data")]
    data_dir: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db = db::Db::open(&args.data_dir)?;
    db.init_schema()?;

    let store = storage::Storage::new(&args.data_dir);

    let app = api::router(db, store);

    let addr = format!("{}:{}", args.host, args.port);
    eprintln!("naze-registry listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
