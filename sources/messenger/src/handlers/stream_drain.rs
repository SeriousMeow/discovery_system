use tracing::warn;

use crate::api::*;
use crate::handlers::common::map_drain_session_error;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    request: StreamDrainRequestParams,
) -> anyhow::Result<StreamDrainResponseEnum> {
    let stream_id = request.body.stream_id.as_str();
    match state.streams.drain_incoming(stream_id).await {
        Ok(messages) => Ok(StreamDrainResponseEnum::Ok(StreamDrainResponse {
            messages,
        })),
        Err(e) => {
            warn!(stream_id, error = ?e, "stream_drain failed");
            Ok(map_drain_session_error(e))
        }
    }
}
