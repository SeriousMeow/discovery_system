use api::queue::post::Request;
use http::StatusCode;

pub async fn handle(request: Request) -> Result<(), StatusCode> {
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
