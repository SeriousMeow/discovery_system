use anyhow::Result;
use axum::serve;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod api;
use api::*;

pub mod handlers;

pub mod state;
use state::AppState;

pub mod broadcast;
use broadcast::MessageSender;

#[derive(Clone)]
struct ApiImpl {
    state: AppState,
}

impl ApiImpl {
    fn new(broadcast_tx: MessageSender) -> Self {
        Self {
            state: AppState::new(broadcast_tx),
        }
    }
}

impl ApiServer for ApiImpl {
    async fn queue_get(
        &self,
        request: QueueGetRequestParams,
    ) -> anyhow::Result<QueueGetResponseEnum> {
        handlers::queue_get::handle(&self.state, request).await
    }

    async fn queue_post(
        &self,
        request: QueuePostRequestParams,
    ) -> anyhow::Result<QueuePostResponse> {
        handlers::queue_post::handle(&self.state, request).await
    }

    async fn register(
        &self,
        request: RegisterRequestParams,
    ) -> anyhow::Result<RegisterResponseEnum> {
        handlers::register::handle(&self.state, request).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let (config, addrs) = broadcast::config::Config::from_sources()?;
    let (broadcast_tx, incoming_rx, worker) =
        broadcast::BroadcastWorker::new(config, addrs.local_node_id, addrs.contact_node).await?;

    tokio::spawn(worker.run());

    let api_impl = ApiImpl::new(broadcast_tx);

    tokio::spawn(state::handle_incoming(
        incoming_rx,
        api_impl.state.storage.clone(),
    ));

    let app = router(api_impl).layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(addrs.http_addr).await?;
    info!("HTTP server listening on {}", addrs.http_addr);

    serve(listener, app).await.expect("Server error");

    Ok(())
}
