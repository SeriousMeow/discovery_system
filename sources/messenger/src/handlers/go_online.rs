use client::error::GoOnlineError;

use crate::api::*;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    request: GoOnlineRequestParams,
) -> anyhow::Result<GoOnlineResponse> {
    let server_url = request.body.server_url;
    let discovery_public_key = request.body.discovery_public_key;

    let result = state
        .handle
        .go_online(server_url, discovery_public_key)
        .await;

    let response = match result {
        Ok(()) => GoOnlineResponse::NoContent,
        Err(GoOnlineError::AlreadyOnline) => GoOnlineResponse::Conflict(ErrorBody {
            message: "already online".into(),
        }),
        Err(GoOnlineError::InvalidUrl(e)) => GoOnlineResponse::BadRequest(ErrorBody {
            message: e.to_string(),
        }),
        Err(GoOnlineError::ForbiddenCredentials) => GoOnlineResponse::BadRequest(ErrorBody {
            message: "discovery rejected credentials (static puzzle or identity)".into(),
        }),
        Err(GoOnlineError::InvalidDiscoveryPublicKey(msg)) => {
            GoOnlineResponse::BadRequest(ErrorBody {
                message: format!("invalid discovery public key: {msg}"),
            })
        }
        Err(GoOnlineError::Puzzle(e)) => GoOnlineResponse::BadRequest(ErrorBody {
            message: e.to_string(),
        }),
    };
    Ok(response)
}
