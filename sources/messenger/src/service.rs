use client::Handle;
use validator::Validate;

use crate::api::*;
use crate::handlers;
use crate::state::MessengerState;

#[derive(Clone)]
pub struct MessengerApi {
    pub state: MessengerState,
}

impl MessengerApi {
    pub fn new(handle: Handle) -> Self {
        Self {
            state: MessengerState::new(handle),
        }
    }
}

impl crate::api::ApiServer for MessengerApi {
    async fn accept_connection(
        &self,
        request: AcceptConnectionRequestParams,
    ) -> anyhow::Result<AcceptConnectionResponse> {
        if let Err(e) = request.validate() {
            return Ok(AcceptConnectionResponse::BadRequest(ErrorBody {
                message: format!("validation: {e}"),
            }));
        }
        handlers::accept_connection::handle(&self.state, request).await
    }

    async fn connect(&self, request: ConnectRequestParams) -> anyhow::Result<ConnectResponse> {
        if let Err(e) = request.validate() {
            return Ok(ConnectResponse::BadRequest(ErrorBody {
                message: format!("validation: {e}"),
            }));
        }
        handlers::connect::handle(&self.state, request).await
    }

    async fn go_online(&self, request: GoOnlineRequestParams) -> anyhow::Result<GoOnlineResponse> {
        if let Err(e) = request.validate() {
            return Ok(GoOnlineResponse::BadRequest(ErrorBody {
                message: format!("validation: {e}"),
            }));
        }
        handlers::go_online::handle(&self.state, request).await
    }

    async fn is_online(&self, request: IsOnlineRequest) -> anyhow::Result<IsOnlineResponseEnum> {
        handlers::is_online::handle(&self.state, request).await
    }

    async fn list_incoming_connections(
        &self,
        request: ListIncomingConnectionsRequest,
    ) -> anyhow::Result<ListIncomingConnectionsResponseEnum> {
        handlers::list_incoming_connections::handle(&self.state, request).await
    }

    async fn list_connections(
        &self,
        request: ListConnectionsRequest,
    ) -> anyhow::Result<ListConnectionsResponseEnum> {
        handlers::list_connections::handle(&self.state, request).await
    }

    async fn me_id(&self, request: MeIdRequest) -> anyhow::Result<MeIdResponseEnum> {
        handlers::me_id::handle(&self.state, request).await
    }

    async fn open_stream(
        &self,
        request: OpenStreamRequestParams,
    ) -> anyhow::Result<OpenStreamResponseEnum> {
        if let Err(e) = request.validate() {
            return Ok(OpenStreamResponseEnum::BadRequest(ErrorBody {
                message: format!("validation: {e}"),
            }));
        }
        handlers::open_stream::handle(&self.state, request).await
    }

    async fn accept_stream(
        &self,
        request: AcceptStreamRequestParams,
    ) -> anyhow::Result<AcceptStreamResponseEnum> {
        if let Err(e) = request.validate() {
            return Ok(AcceptStreamResponseEnum::BadRequest(ErrorBody {
                message: format!("validation: {e}"),
            }));
        }
        handlers::accept_stream::handle(&self.state, request).await
    }

    async fn stream_send(
        &self,
        request: StreamSendRequestParams,
    ) -> anyhow::Result<StreamSendResponse> {
        request.validate().map_err(|e| anyhow::anyhow!("validation: {e}"))?;
        handlers::stream_send::handle(&self.state, request).await
    }

    async fn stream_drain(
        &self,
        request: StreamDrainRequestParams,
    ) -> anyhow::Result<StreamDrainResponseEnum> {
        request.validate().map_err(|e| anyhow::anyhow!("validation: {e}"))?;
        handlers::stream_drain::handle(&self.state, request).await
    }

    async fn delete_stream(
        &self,
        request: DeleteStreamRequest,
    ) -> anyhow::Result<DeleteStreamResponse> {
        request.validate().map_err(|e| anyhow::anyhow!("validation: {e}"))?;
        handlers::delete_stream::handle(&self.state, request).await
    }
}
