use tracing::warn;

use crate::api::*;
use crate::handlers::common::map_send_session_error;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    request: StreamSendRequestParams,
) -> anyhow::Result<StreamSendResponse> {
    let stream_id = request.body.stream_id.as_str();
    match state
        .streams
        .send_string(stream_id, &request.body.message)
        .await
    {
        Ok(()) => Ok(StreamSendResponse::NoContent),
        Err(e) => {
            warn!(stream_id, error = ?e, "stream_send failed");
            Ok(map_send_session_error(e))
        }
    }
}
