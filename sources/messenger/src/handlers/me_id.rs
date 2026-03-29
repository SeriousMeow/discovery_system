use crate::api::*;
use crate::state::MessengerState;

pub async fn handle(
    state: &MessengerState,
    _request: MeIdRequest,
) -> anyhow::Result<MeIdResponseEnum> {
    match state.handle.self_id().await {
        Some(id) => Ok(MeIdResponseEnum::Ok(MeIdResponse {
            id: id.to_string(),
        })),
        None => Ok(MeIdResponseEnum::ServiceUnavailable(ErrorBody {
            message: "offline".into(),
        })),
    }
}
