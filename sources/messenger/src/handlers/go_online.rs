use client::error::GoOnlineError;

use crate::api::*;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    request: GoOnlineRequestParams,
) -> anyhow::Result<GoOnlineResponse> {
    let server_url = request.body.server_url;

    let result = state.handle.go_online(server_url).await;

    let response = match result {
        Ok(()) => GoOnlineResponse::NoContent,
        Err(GoOnlineError::AlreadyOnline) => GoOnlineResponse::Conflict(ErrorBody {
            message: "already online".into(),
        }),
        Err(GoOnlineError::InvalidUrl(e)) => GoOnlineResponse::BadRequest(ErrorBody {
            message: e.to_string(),
        }),
    };
    Ok(response)
}
