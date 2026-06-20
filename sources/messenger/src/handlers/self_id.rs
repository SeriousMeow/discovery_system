use crate::api::*;
use crate::state::MessengerState;

pub async fn handle(state: &MessengerState) -> anyhow::Result<MeIdResponse> {
    match state.handle.self_id().await {
        Some(id) => Ok(MeIdResponse::Ok(IdSelfResponse {
            endpoint_id: id.to_string(),
        })),
        None => Ok(MeIdResponse::ServiceUnavailable(ErrorBody {
            message: "No Iroh endpoint id yet (offline or endpoint not bound).".into(),
        })),
    }
}
