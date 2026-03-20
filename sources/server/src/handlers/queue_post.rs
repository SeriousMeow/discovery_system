use crate::api::*;
use crate::state::AppState;
use anyhow::Result;

pub async fn handle(
    state: &AppState,
    request: QueuePostRequestParams,
) -> Result<QueuePostResponse> {
    let recipient = request.body.recipient;
    let value = request.body.data;

    let mut guard = state.storage.write().map_err(|_| anyhow::anyhow!(""))?;
    let storage = &mut *guard;

    storage.insert(&recipient, value);
    Ok(QueuePostResponse::NotFound)
}
