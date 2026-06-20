use crate::api::*;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    _request: ListIncomingConnectionsRequest,
) -> anyhow::Result<ListIncomingConnectionsResponseEnum> {
    let ids = state.handle.list_incoming_connections().await;
    let peer_ids = ids.iter().map(ToString::to_string).collect();
    Ok(ListIncomingConnectionsResponseEnum::Ok(
        ListIncomingConnectionsResponse { peer_ids },
    ))
}
