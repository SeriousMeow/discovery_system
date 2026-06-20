use crate::api::*;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    _request: ListConnectionsRequest,
) -> anyhow::Result<ListConnectionsResponseEnum> {
    let ids = state.handle.list_connections().await;
    let peer_ids = ids.iter().map(ToString::to_string).collect();
    Ok(ListConnectionsResponseEnum::Ok(ListConnectionsResponse {
        peer_ids,
    }))
}
