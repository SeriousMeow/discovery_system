use std::time::Duration;

use tokio::time::timeout;
use tracing::{info, warn};

use crate::api::*;
use crate::handlers::common::{map_open_stream_error, parse_peer_id};
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    request: OpenStreamRequestParams,
) -> anyhow::Result<OpenStreamResponseEnum> {
    let peer_id = match parse_peer_id(&request.body.peer_id) {
        Ok(id) => id,
        Err(e) => {
            return Ok(OpenStreamResponseEnum::BadRequest(ErrorBody {
                message: e.to_string(),
            }));
        }
    };
    let dur = Duration::from_millis(request.body.timeout_ms as u64);

    let result = timeout(dur, state.handle.open_stream(peer_id)).await;

    match result {
        Err(_) => {
            warn!(%peer_id, "open_stream timed out");
            Ok(OpenStreamResponseEnum::GatewayTimeout(ErrorBody {
                message: "open timed out".into(),
            }))
        }
        Ok(Err(e)) => {
            warn!(%peer_id, error = %e, "open_stream failed");
            Ok(map_open_stream_error(e))
        }
        Ok(Ok((send, recv))) => {
            let stream_id = state.streams.register_session(send, recv).await;
            info!(%stream_id, %peer_id, "stream session registered (open_stream)");
            Ok(OpenStreamResponseEnum::Ok(OpenStreamResponse { stream_id }))
        }
    }
}
