use axum::extract::Json;
use axum::routing::post;
use axum::{Router, http::StatusCode, serve};

type Response<T> = Result<Json<T>, StatusCode>;

async fn register(
    Json(request): Json<api::register::Request>,
) -> Response<api::register::Response> {
    Ok(Json(server::handlers::register::handle(request).await?))
}

async fn queue_post(Json(request): Json<api::queue::post::Request>) -> Response<()> {
    Ok(Json(server::handlers::queue_post::handle(request).await?))
}

async fn queue_get(
    Json(request): Json<api::queue::get::Request>,
) -> Response<api::queue::get::Response> {
    Ok(Json(server::handlers::queue_get::handle(request).await?))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(api::register::ENDPOINT, post(register))
        .route(api::queue::get::ENDPOINT, post(queue_get))
        .route(api::queue::post::ENDPOINT, post(queue_post));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    serve(listener, app).await.unwrap();
}
