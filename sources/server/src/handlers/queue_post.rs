use crate::state::AppState;
use api::queue::post::Request;
use http::StatusCode;

pub async fn handle(request: Request, state: AppState) -> Result<(), StatusCode> {
    let recipient = request.recipient;
    let value = request.data;

    let mut guard = state
        .storage
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let storage = &mut *guard;

    storage.insert(&recipient, value);
    Ok(())
}
