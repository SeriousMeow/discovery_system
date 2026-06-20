use tracing::{info, warn};

use crate::api::*;
use crate::state::MessengerState;
use crate::stream_session::StreamSessionError;

pub async fn handle(
    state: &MessengerState,
    request: DeleteStreamRequest,
) -> anyhow::Result<DeleteStreamResponse> {
    let stream_id = request.path.stream_id.as_str();
    match state.streams.remove(stream_id).await {
        Ok(()) => {
            info!(stream_id, "stream session removed");
            Ok(DeleteStreamResponse::NoContent)
        }
        Err(StreamSessionError::UnknownStreamId) => Ok(DeleteStreamResponse::NotFound(ErrorBody {
            message: "unknown stream_id".into(),
        })),
        Err(e) => {
            warn!(stream_id, ?e, "delete_stream: unexpected remove error");
            Ok(DeleteStreamResponse::Conflict(ErrorBody {
                message: format!("unexpected: {e:?}"),
            }))
        }
    }
}
