use std::time::Duration;

use crate::api::{QueuePostRequest, QueuePostRequestParams, QueuePostResponse};
use crate::utils::rpc;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use iroh::EndpointId;
use iroh_tickets::endpoint::EndpointTicket;
use pow_puzzle::{PublicKey, public_key_hex_id};
use thiserror::Error;
use tokio::time::timeout;

#[derive(Error, Debug)]
pub enum Error {
    #[error("offline")]
    Offline,
    #[error("already connected")]
    AlreadyConnected,
    #[error("connection with peer is already in progress")]
    AlreadyConnecting,
    #[error("invalid peer discovery public key: {0}")]
    InvalidPeerDiscoveryKey(String),
    #[error("failed to send discovery ticket to peer server")]
    DiscoveryPostFailed(#[from] anyhow::Error),
    #[error("discovery queue post rejected (sender not registered)")]
    NotRegistered,
    #[error("failed to connect")]
    ConnectionError(#[from] iroh::endpoint::ConnectionError),
    #[error("connection timed out")]
    Timeout,
}

pub type Command = rpc::Command<Request, Response>;

pub struct Request {
    pub peer_iroh_endpoint_id: EndpointId,
    pub peer_discovery_public_key: String,
    pub timeout: Duration,
}

pub type Response = Result<(), Error>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let response_tx = command.response_tx;

        let Some(module) = self.discovery_module.as_mut() else {
            let _ = response_tx.send(Err(Error::Offline));
            return;
        };

        let peer_id = command.request.peer_iroh_endpoint_id;
        let peer_discovery_public_key = command.request.peer_discovery_public_key;
        let pending_timeout = command.request.timeout;

        if self.connections.read().unwrap().contains_key(&peer_id) {
            let _ = response_tx.send(Err(Error::AlreadyConnected));
            return;
        }

        if module.has_pending_connection(peer_id) {
            let _ = response_tx.send(Err(Error::AlreadyConnecting));
            return;
        }

        let peer_discovery_public_key = match parse_discovery_public_key(&peer_discovery_public_key) {
            Ok(key) => key,
            Err(msg) => {
                let _ = response_tx.send(Err(Error::InvalidPeerDiscoveryKey(msg)));
                return;
            }
        };

        let my_ticket = EndpointTicket::from(module.endpoint.addr()).to_string();
        let post_request = QueuePostRequestParams {
            body: QueuePostRequest {
                credentials: module.get_credentials(),
                recipient: public_key_hex_id(&peer_discovery_public_key),
                data: my_ticket.clone(),
            },
        };

        match module.discovery_client.queue_post(post_request).await {
            Ok(QueuePostResponse::NoContent) => {}
            Ok(QueuePostResponse::Forbidden) => {
                let _ = response_tx.send(Err(Error::NotRegistered));
                return;
            }
            Ok(_) => {
                let _ = response_tx.send(Err(Error::DiscoveryPostFailed(anyhow::anyhow!(
                    "unexpected queue_post status"
                ))));
                return;
            }
            Err(e) => {
                let _ = response_tx.send(Err(Error::DiscoveryPostFailed(e)));
                return;
            }
        }

        let (callback_tx, callback_rx) = tokio::sync::oneshot::channel();
        module.add_pending_connection(peer_id, callback_tx);

        let pending_connections = module.pending_connections.clone();
        tokio::spawn(async move {
            let response = match timeout(pending_timeout, callback_rx).await {
                Ok(Ok(Ok(_connection))) => Ok(()),
                Ok(Ok(Err(e))) => Err(Error::ConnectionError(e)),
                Ok(Err(_)) => Err(Error::Timeout),
                Err(_) => {
                    let mut guard = pending_connections.write().unwrap();
                    if guard.remove(&peer_id).is_some() {
                        Err(Error::Timeout)
                    } else {
                        return;
                    }
                }
            };

            let _ = response_tx.send(response);
        });
    }
}

fn parse_discovery_public_key(s: &str) -> Result<PublicKey, String> {
    let raw = B64
        .decode(s.as_bytes())
        .map_err(|e| format!("base64 decode: {e}"))?;
    PublicKey::try_from(raw.as_slice()).map_err(|_| "base64 must decode to exactly 32 bytes".into())
}
