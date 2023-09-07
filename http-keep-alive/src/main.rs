//! Program contains an actix HTTP server that replies to `GET` requests on `/` path.
//!
//! Intended to check if we can mirror existing sessions.
#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .write_style(env_logger::WriteStyle::Never)
        .init();

    HttpServer::new(|| App::new().service(index).wrap(Logger::default()))
        .bind(("0.0.0.0", 80))?
        .keep_alive(Duration::from_secs(240))
        .run()
        .await
}

#[get("/")]
#[tracing::instrument(level = "info", ret)]
async fn index(incoming: String) -> String {
    format!("Echo [remote]: {incoming}")
}

use std::time::Duration;

use actix_web::{get, middleware::Logger, App, HttpServer};
