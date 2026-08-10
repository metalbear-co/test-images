//! A Kafka consumer whose consumption the test drives over HTTP.
//!
//! It lets drain tests hold fallback lag open and release it on demand.

mod app;
mod http;

use clap::Parser;

#[derive(Parser)]
struct Config {
    #[arg(long, env = "ADDRESS")]
    address: String,
    #[arg(long, env = "GROUP")]
    group: String,
    #[arg(long, env = "TOPIC")]
    topic: String,
    #[arg(long, env = "PORT", default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::parse();
    let app_handle = app::App::spawn(&config.address, &config.group, &config.topic)?;
    tracing::info!(topic = %config.topic, group = %config.group, "controllable consumer started");
    http::serve(app_handle.clone(), config.port).await?;
    app_handle.shutdown().await?;
    Ok(())
}
