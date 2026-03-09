use axum::serve;

mod api;
use api::*;

pub mod handlers;

pub mod state;
use state::AppState;

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
async fn main() {
    let api_impl = ApiImpl::new();

    let app = router(api_impl);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    serve(listener, app).await.expect("Server error");
}
