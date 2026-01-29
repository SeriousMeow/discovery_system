use axum::extract::{Json, State};
use axum::routing::post;
use axum::{Router, http::StatusCode, serve};

use server::state::AppState;

type Response<T> = Result<Json<T>, StatusCode>;

async fn register(
    State(state): State<AppState>,
    Json(request): Json<api::register::Request>,
) -> Response<api::register::Response> {
    Ok(Json(
        server::handlers::register::handle(request, state).await?,
    ))
}

async fn queue_post(
    State(state): State<AppState>,
    Json(request): Json<api::queue::post::Request>,
) -> Response<()> {
    Ok(Json(
        server::handlers::queue_post::handle(request, state).await?,
    ))
}

async fn queue_get(
    State(state): State<AppState>,
    Json(request): Json<api::queue::get::Request>,
) -> Response<api::queue::get::Response> {
    Ok(Json(
        server::handlers::queue_get::handle(request, state).await?,
    ))
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new()
        .route(api::register::ENDPOINT, post(register))
        .route(api::queue::get::ENDPOINT, post(queue_get))
        .route(api::queue::post::ENDPOINT, post(queue_post))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    serve(listener, app).await.unwrap();
}
