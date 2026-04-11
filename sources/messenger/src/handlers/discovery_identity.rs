use crate::api::*;
use crate::state::MessengerState;

pub async fn handle(state: &MessengerState) -> anyhow::Result<IdsDiscoveryResponse> {
    match state.handle.discovery_identity().await {
        Some(id) => Ok(IdsDiscoveryResponse::Ok(DiscoveryIdentityResponse {
            public_key_base64: id.public_key_base64,
            public_key_hex: id.public_key_hex,
        })),
        None => Ok(IdsDiscoveryResponse::ServiceUnavailable(ErrorBody {
            message: "Not registered with Discovery (offline or go_online not completed).".into(),
        })),
    }
}
