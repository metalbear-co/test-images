//! Small HTTP control surface for the drain tests.

use std::{net::SocketAddr, time::Duration};

use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;

use crate::app::{AppHandle, ConsumedMessage};

pub async fn serve(app: AppHandle, port: u16) -> anyhow::Result<()> {
    let router = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/consume", post(consume))
        .route("/peek", post(peek))
        .with_state(app);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("http server failed")?;

    Ok(())
}

/// Stop serving on SIGTERM so the consumer leaves its group before the pod exits.
async fn shutdown_signal() {
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    tokio::select! {
        () = terminate => {}
        result = tokio::signal::ctrl_c() => {
            let _ = result;
        }
    }
}

#[derive(Deserialize)]
struct ConsumeParams {
    #[serde(default = "ConsumeParams::default_count")]
    count: usize,
    #[serde(default = "ConsumeParams::default_wait_ms")]
    wait_ms: u64,
}

impl ConsumeParams {
    fn default_count() -> usize {
        1
    }

    fn default_wait_ms() -> u64 {
        5000
    }
}

async fn consume(
    State(app): State<AppHandle>,
    Query(params): Query<ConsumeParams>,
) -> Result<Json<Vec<ConsumedMessage>>, (StatusCode, String)> {
    app.consume(params.count, Duration::from_millis(params.wait_ms))
        .await
        .map(Json)
        .map_err(|error| {
            tracing::error!(%error, "consume request failed");
            (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
        })
}

#[derive(Deserialize)]
struct PeekParams {
    #[serde(default = "ConsumeParams::default_count")]
    count: usize,
    #[serde(default = "ConsumeParams::default_wait_ms")]
    wait_ms: u64,
    topic: String,
}

async fn peek(
    State(app): State<AppHandle>,
    Query(params): Query<PeekParams>,
) -> Result<Json<Vec<ConsumedMessage>>, (StatusCode, String)> {
    if params.topic.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "topic query parameter must not be empty".to_owned(),
        ));
    }

    app.peek(
        params.topic,
        params.count,
        Duration::from_millis(params.wait_ms),
    )
    .await
    .map(Json)
    .map_err(|error| {
        tracing::error!(%error, "peek request failed");
        (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
    })
}
