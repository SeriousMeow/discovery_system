use client::error::{AcceptStreamError, OpenStreamError};
use iroh::EndpointId;

use crate::api::*;
use crate::stream_session::StreamSessionError;

pub fn parse_peer_id(s: &str) -> Result<EndpointId, anyhow::Error> {
    s.parse()
        .map_err(|e| anyhow::anyhow!("invalid peer_id: {e}"))
}

pub fn parse_peer_discovery_public_key(s: &str) -> Result<String, anyhow::Error> {
    let key = s.trim();
    if key.is_empty() {
        return Err(anyhow::anyhow!("invalid peer_discovery_public_key: empty"));
    }
    Ok(key.to_owned())
}

pub fn map_open_stream_error(e: OpenStreamError) -> OpenStreamResponseEnum {
    match e {
        OpenStreamError::NotConnected => OpenStreamResponseEnum::Conflict(ErrorBody {
            message: "not connected to peer".into(),
        }),
        other => OpenStreamResponseEnum::BadGateway(ErrorBody {
            message: other.to_string(),
        }),
    }
}

pub fn map_accept_stream_error(e: AcceptStreamError) -> AcceptStreamResponseEnum {
    match e {
        AcceptStreamError::NotConnected => AcceptStreamResponseEnum::Conflict(ErrorBody {
            message: "not connected to peer".into(),
        }),
        other => AcceptStreamResponseEnum::BadGateway(ErrorBody {
            message: other.to_string(),
        }),
    }
}

pub fn map_send_session_error(e: StreamSessionError) -> StreamSendResponse {
    match e {
        StreamSessionError::UnknownStreamId => StreamSendResponse::NotFound(ErrorBody {
            message: "unknown stream_id".into(),
        }),
        StreamSessionError::NotActive { detail } => StreamSendResponse::Conflict(ErrorBody {
            message: detail.into(),
        }),
        StreamSessionError::Utf8Framing => StreamSendResponse::UnprocessableEntity(ErrorBody {
            message: "invalid UTF-8 framing payload".into(),
        }),
        StreamSessionError::QueueOverflow => StreamSendResponse::BadGateway(ErrorBody {
            message: "incoming message queue overflow".into(),
        }),
        StreamSessionError::Io(msg) => StreamSendResponse::BadGateway(ErrorBody { message: msg }),
    }
}

pub fn map_drain_session_error(e: StreamSessionError) -> StreamDrainResponseEnum {
    match e {
        StreamSessionError::UnknownStreamId => StreamDrainResponseEnum::NotFound(ErrorBody {
            message: "unknown stream_id".into(),
        }),
        StreamSessionError::NotActive { detail } => StreamDrainResponseEnum::Conflict(ErrorBody {
            message: detail.into(),
        }),
        StreamSessionError::Utf8Framing => {
            StreamDrainResponseEnum::UnprocessableEntity(ErrorBody {
                message: "invalid UTF-8 framing payload".into(),
            })
        }
        StreamSessionError::QueueOverflow => StreamDrainResponseEnum::BadGateway(ErrorBody {
            message: "incoming message queue overflow".into(),
        }),
        StreamSessionError::Io(msg) => {
            StreamDrainResponseEnum::BadGateway(ErrorBody { message: msg })
        }
    }
}
