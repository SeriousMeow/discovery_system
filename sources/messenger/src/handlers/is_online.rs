use crate::api::*;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    _request: IsOnlineRequest,
) -> anyhow::Result<IsOnlineResponseEnum> {
    let online = state.handle.is_online().await;
    Ok(IsOnlineResponseEnum::Ok(IsOnlineResponse { online }))
}
