use std::time::Duration;

use crate::api::{QueuePostRequest, QueuePostRequestParams};
use crate::utils::rpc;
use iroh::EndpointId;
use iroh_tickets::endpoint::EndpointTicket;
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
    #[error("failed to send discovery ticket to any peer server")]
    DiscoveryPostFailed,
    #[error("failed to connect")]
    ConnectionError(#[from] iroh::endpoint::ConnectionError),
    #[error("connection timed out")]
    Timeout,
}

pub type Command = rpc::Command<Request, Response>;

pub struct Request {
    pub peer_id: EndpointId,
    pub peer_discovery_servers: Vec<String>,
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

        let peer_id = command.request.peer_id;
        let pending_timeout = command.request.timeout;

        if self.connections.read().unwrap().contains_key(&peer_id) {
            let _ = response_tx.send(Err(Error::AlreadyConnected));
            return;
        }

        if module.has_pending_connection(peer_id) {
            let _ = response_tx.send(Err(Error::AlreadyConnecting));
            return;
        }

        let my_ticket = EndpointTicket::from(module.endpoint.addr()).to_string();

        let mut posted_to_any = false;
        for server_url in &command.request.peer_discovery_servers {
            let Ok(client) = crate::api::DiscoverySystemClient::with_base_url(server_url) else {
                continue;
            };

            let post_request = QueuePostRequestParams {
                body: QueuePostRequest {
                    credentials: module.get_credentials(),
                    recipient: peer_id.to_string(),
                    data: my_ticket.clone(),
                },
            };

            if client.queue_post(post_request).await.is_ok() {
                posted_to_any = true;
            }
        }

        if !posted_to_any {
            let _ = response_tx.send(Err(Error::DiscoveryPostFailed));
            return;
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
