use api::register::{Request, Response};
use chrono::Utc;
use http::StatusCode;

pub async fn handle(request: Request) -> Result<Response, StatusCode> {
    Ok(Response {
        reserved_until: Utc::now(),
    })
}
