use clap::Parser;
use reddit_agent::cli::{AppContext, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging if RUST_LOG is set
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    let cli = Cli::parse();
    let app = AppContext::init(cli.client_id.clone(), cli.client_secret.clone());
    app.run(cli).await
}
