use crate::api::*;
use crate::state::AppState;
use anyhow::Result;
use chrono::Utc;

pub async fn handle(
    state: &AppState,
    request: RegisterRequestParams,
) -> Result<RegisterResponseEnum> {
    let id = request.body.credentials.id;

    let mut guard = state.storage.lock().map_err(|_| anyhow::anyhow!(""))?;
    let storage = &mut *guard;

    storage.touch(&id);

    Ok(RegisterResponseEnum::Ok(RegisterResponse {
        reserved_until: Utc::now(),
    }))
}
