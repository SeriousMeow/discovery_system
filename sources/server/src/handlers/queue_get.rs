use crate::api::*;
use crate::state::AppState;
use anyhow::Result;

pub async fn handle(
    state: &AppState,
    request: QueueGetRequestParams,
) -> Result<QueueGetResponseEnum> {
    let id = request.body.credentials.id;

    let mut guard = state.storage.lock().map_err(|_| anyhow::anyhow!(""))?;
    let storage = &mut *guard;

    let value = match storage.get(&id) {
        Some(value) => value,
        None => return Ok(QueueGetResponseEnum::NotFound),
    };

    Ok(QueueGetResponseEnum::Ok(QueueGetResponse {
        data: vec![value.clone()],
    }))
}
