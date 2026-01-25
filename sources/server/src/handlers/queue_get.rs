use api::queue::get::{Request, Response};
use http::StatusCode;

pub async fn handle(request: Request) -> Result<Response, StatusCode> {
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
