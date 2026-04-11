mod api;
mod handlers;
mod service;
mod state;
mod stream_session;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use axum::serve;
use client::{Config, init};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use api::router;
use service::MessengerApi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config {
        command_buffer_size: 1024,
        poll_interval: Duration::from_secs(5),
        puzzle_min_difficulty_bits: 8,
    };

    let (handle, worker) = init(config);

    tokio::spawn(async move {
        if let Err(e) = worker.run().await {
            error!("client worker exited with error: {}", e);
        }
    });

    let http_addr: SocketAddr = std::env::var("MESSENGER_HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3030".into())
        .parse()
        .context("parse MESSENGER_HTTP_ADDR")?;

    let webui_path = std::env::var_os("MESSENGER_WEBUI_DIR")
        .map(PathBuf::from)
        .context("MESSENGER_WEBUI_DIR must be set to the static webui directory")?;
    let webui_meta = std::fs::metadata(&webui_path)
        .with_context(|| format!("webui directory missing at {}", webui_path.display()))?;
    if !webui_meta.is_dir() {
        anyhow::bail!("webui path is not a directory: {}", webui_path.display());
    }
    info!("Using webui path: {}", webui_path.display());

    let app = router(MessengerApi::new(handle))
        .fallback_service(ServeDir::new(&webui_path))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(http_addr)
        .await
        .with_context(|| format!("bind {http_addr}"))?;

    info!("Messenger HTTP listening on  address {}", http_addr);

    serve(listener, app).await.context("HTTP server failed")?;

    Ok(())
}
