use crate::api::*;
use crate::broadcast::Payload;
use crate::state::AppState;

use anyhow::Result;
use pow_puzzle::PublicKey;

pub async fn handle(
    state: &AppState,
    request: QueuePostRequestParams,
    _public_key: PublicKey,
) -> Result<QueuePostResponse> {
    let recipient = request.body.recipient;
    let value = request.body.data;

    state
        .broadcast_tx
        .send(Payload {
            to: recipient,
            data: value,
        })
        .await?;

    Ok(QueuePostResponse::NoContent)
}
