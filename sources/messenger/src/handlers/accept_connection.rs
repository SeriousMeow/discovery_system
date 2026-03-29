use std::time::Duration;

use client::error::AcceptConnectionError;

use crate::api::*;
use crate::handlers::common::parse_peer_id;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    request: AcceptConnectionRequestParams,
) -> anyhow::Result<AcceptConnectionResponse> {
    let peer_id = match parse_peer_id(&request.body.peer_id) {
        Ok(id) => id,
        Err(e) => {
            return Ok(AcceptConnectionResponse::BadRequest(ErrorBody {
                message: e.to_string(),
            }));
        }
    };
    let timeout = Duration::from_millis(request.body.timeout_ms as u64);

    let result = state.handle.accept_connection(peer_id, timeout).await;

    let response = match result {
        Ok(()) => AcceptConnectionResponse::NoContent,
        Err(AcceptConnectionError::Offline) => {
            AcceptConnectionResponse::ServiceUnavailable(ErrorBody {
                message: "offline".into(),
            })
        }
        Err(AcceptConnectionError::NoIncomingConnection) => {
            AcceptConnectionResponse::NotFound(ErrorBody {
                message: "no matching pending incoming connection".into(),
            })
        }
        Err(AcceptConnectionError::AlreadyConnected)
        | Err(AcceptConnectionError::AlreadyConnecting) => {
            AcceptConnectionResponse::Conflict(ErrorBody {
                message: "already connected or connection in progress".into(),
            })
        }
        Err(AcceptConnectionError::ConnectionError(e)) => {
            AcceptConnectionResponse::BadGateway(ErrorBody {
                message: e.to_string(),
            })
        }
        Err(AcceptConnectionError::Timeout) => {
            AcceptConnectionResponse::GatewayTimeout(ErrorBody {
                message: "accept timed out".into(),
            })
        }
    };
    Ok(response)
}
