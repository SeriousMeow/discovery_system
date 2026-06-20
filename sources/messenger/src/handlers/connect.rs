use std::time::Duration;

use client::error::ConnectError;

use crate::api::*;
use crate::handlers::common::{parse_peer_discovery_public_key, parse_peer_id};
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    request: ConnectRequestParams,
) -> anyhow::Result<ConnectResponse> {
    let peer_id = match parse_peer_id(&request.body.peer_iroh_id) {
        Ok(id) => id,
        Err(e) => {
            return Ok(ConnectResponse::BadRequest(ErrorBody {
                message: e.to_string(),
            }));
        }
    };
    let peer_discovery_public_key =
        match parse_peer_discovery_public_key(&request.body.peer_discovery_public_key) {
            Ok(key) => key,
            Err(e) => {
                return Ok(ConnectResponse::BadRequest(ErrorBody {
                    message: e.to_string(),
                }));
            }
        };
    let timeout = Duration::from_millis(request.body.timeout_ms as u64);

    let result = state
        .handle
        .connect(peer_id, peer_discovery_public_key, timeout)
        .await;

    let response = match result {
        Ok(()) => ConnectResponse::NoContent,
        Err(ConnectError::Offline) => ConnectResponse::ServiceUnavailable(ErrorBody {
            message: "offline".into(),
        }),
        Err(ConnectError::AlreadyConnected) | Err(ConnectError::AlreadyConnecting) => {
            ConnectResponse::Conflict(ErrorBody {
                message: "already connected or connection in progress".into(),
            })
        }
        Err(ConnectError::InvalidPeerDiscoveryKey(e)) => ConnectResponse::BadRequest(ErrorBody {
            message: format!("invalid peer_discovery_public_key: {e}"),
        }),
        Err(ConnectError::NotRegistered) => ConnectResponse::BadGateway(ErrorBody {
            message: "discovery queue post rejected (sender not registered)".into(),
        }),
        Err(ConnectError::DiscoveryPostFailed(e)) => ConnectResponse::BadGateway(ErrorBody {
            message: format!("failed to send discovery ticket to peer server: {}", e),
        }),
        Err(ConnectError::ConnectionError(e)) => ConnectResponse::BadGateway(ErrorBody {
            message: e.to_string(),
        }),
        Err(ConnectError::Timeout) => ConnectResponse::GatewayTimeout(ErrorBody {
            message: "connect timed out".into(),
        }),
    };
    Ok(response)
}
