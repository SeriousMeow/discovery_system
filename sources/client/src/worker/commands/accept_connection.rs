use std::time::Duration;

use crate::utils::rpc;
use crate::worker::discovery_module::ALPN;
use thiserror::Error;
use tokio::time::timeout;

#[derive(Error, Debug)]
pub enum Error {
    #[error("offline")]
    Offline,
    #[error("no incoming connection")]
    NoIncomingConnection,
    #[error("already connected")]
    AlreadyConnected,
    #[error("connection with peer is already in progress")]
    AlreadyConnecting,
    #[error("failed to connect")]
    ConnectionError(#[from] iroh::endpoint::ConnectError),
    #[error("connection timed out")]
    Timeout,
}

pub type Command = rpc::Command<Request, Response>;

pub struct Request {
    pub peer_id: iroh::EndpointId,
    pub timeout: Duration,
}

pub type Response = Result<(), Error>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let rpc::Command {
            request,
            response_tx,
        } = command;

        let Some(module) = self.discovery_module.as_mut() else {
            let _ = response_tx.send(Err(Error::Offline));
            return;
        };

        if self
            .connections
            .lock()
            .unwrap()
            .contains_key(&request.peer_id)
        {
            let _ = response_tx.send(Err(Error::AlreadyConnected));
            return;
        }

        let peer_id = request.peer_id;

        let Some(ticket) = self.pending_incoming_connections.remove(&peer_id) else {
            let _ = response_tx.send(Err(Error::NoIncomingConnection));
            return;
        };

        if module.has_pending_connection(peer_id) {
            let _ = response_tx.send(Err(Error::AlreadyConnecting));
            return;
        }

        let endpoint = module.endpoint.clone();
        let connections = self.connections.clone();
        let connect_timeout = request.timeout;

        tokio::spawn(async move {
            let connect_result = timeout(connect_timeout, endpoint.connect(ticket, ALPN)).await;

            let response = match connect_result {
                Ok(Ok(connection)) => {
                    let peer_id = connection.remote_id();
                    let mut guard = connections.lock().unwrap();
                    guard.insert(peer_id, connection);
                    Ok(())
                }
                Ok(Err(err)) => Err(Error::ConnectionError(err)),
                Err(_) => Err(Error::Timeout),
            };

            let _ = response_tx.send(response);
        });
    }
}
