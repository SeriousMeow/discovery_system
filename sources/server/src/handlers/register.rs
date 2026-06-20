use crate::api::*;
use crate::state::AppState;
use anyhow::Result;
use chrono::Utc;
use pow_puzzle::{PublicKey, public_key_hex_id};

pub async fn handle(
    state: &AppState,
    _request: RegisterRequestParams,
    public_key: PublicKey,
) -> Result<RegisterResponseEnum> {
    let mut guard = state.storage.write().map_err(|_| anyhow::anyhow!(""))?;
    let storage = &mut *guard;

    storage.ensure_queue(public_key_hex_id(&public_key));

    Ok(RegisterResponseEnum::Ok(RegisterResponse {
        reserved_until: Utc::now(),
    }))
}
