use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use anyhow::Result;
use axum::serve;

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
    let config = broadcast::config::Config::default();
    let contact_node = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8000));
    let (broadcast_tx, incoming_rx, worker) =
        broadcast::BroadcastWorker::new(config, contact_node).await?;

    tokio::spawn(worker.run());

    let api_impl = ApiImpl::new(broadcast_tx);

    tokio::spawn(state::handle_incoming(
        incoming_rx,
        api_impl.state.storage.clone(),
    ));

    let app = router(api_impl);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    serve(listener, app).await.expect("Server error");

    Ok(())
}
