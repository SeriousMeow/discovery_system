use anyhow::Result;
use axum::serve;

mod api;
use api::*;

pub mod handlers;

pub mod state;
use state::AppState;

pub mod broadcast;

#[derive(Clone)]
struct ApiImpl {
    state: AppState,
}

impl ApiImpl {
    fn new() -> Self {
        Self {
            state: AppState::new(),
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
    let (_, _, worker) = broadcast::BroadcastWorker::new(config).await?;

    tokio::spawn(async move {
        worker.run().await?;

        Ok::<_, anyhow::Error>(())
    });

    let api_impl = ApiImpl::new();

    let app = router(api_impl);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    serve(listener, app).await.expect("Server error");

    Ok(())
}
