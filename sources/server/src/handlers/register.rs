use crate::state::AppState;
use api::register::{Request, Response};
use chrono::Utc;
use http::StatusCode;

pub async fn handle(request: Request, state: AppState) -> Result<Response, StatusCode> {
    let id = request.credentials.id;

    let mut guard = state
        .storage
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let storage = &mut *guard;

    storage.touch(&id);

    Ok(Response {
        reserved_until: Utc::now(),
    })
}
