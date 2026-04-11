use crate::api::*;
use crate::state::AppState;
use anyhow::Result;
use pow_puzzle::{PublicKey, public_key_hex_id};

pub async fn handle(
    state: &AppState,
    _request: QueueGetRequestParams,
    public_key: PublicKey,
) -> Result<QueueGetResponseEnum> {
    let key = public_key_hex_id(&public_key);

    let guard = state.storage.read().map_err(|_| anyhow::anyhow!(""))?;
    let storage = &*guard;

    let Some(data) = storage.get(&key).cloned() else {
        return Ok(QueueGetResponseEnum::NotFound);
    };

    Ok(QueueGetResponseEnum::Ok(QueueGetResponse { data }))
}
