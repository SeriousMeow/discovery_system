use crate::state::AppState;
use api::queue::get::{Request, Response};
use http::StatusCode;

pub async fn handle(request: Request, state: AppState) -> Result<Response, StatusCode> {
    let id = request.credentials.id;

    let mut guard = state
        .storage
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let storage = &mut *guard;

    let value = match storage.get(&id) {
        Some(value) => value,
        None => return Err(StatusCode::NOT_FOUND),
    };

    Ok(Response {
        data: vec![value.clone()],
    })
}
