use crate::utils::rpc;
use iroh::endpoint::{RecvStream, SendStream};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not connected")]
    NotConnected,
    #[error("failed to open bi stream: {0}")]
    OpenBiError(String),
    #[error("failed to write hello: {0}")]
    WriteHelloError(String),
    #[error("failed to read hello: {0}")]
    ReadHelloError(String),
    #[error("invalid hello payload")]
    InvalidHelloPayload,
}

pub type Command = rpc::Command<Request, Response>;

pub struct Request {
    pub peer_id: iroh::EndpointId,
}

pub type Response = Result<(SendStream, RecvStream), Error>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let rpc::Command {
            request,
            response_tx,
        } = command;

        let connection = self
            .connections
            .read()
            .unwrap()
            .get(&request.peer_id)
            .cloned();

        let Some(connection) = connection else {
            let _ = response_tx.send(Err(Error::NotConnected));
            return;
        };

        tokio::spawn(async move {
            let response: Response = async {
                let (mut tx, mut rx) = connection
                    .open_bi()
                    .await
                    .map_err(|err| Error::OpenBiError(err.to_string()))?;

                tx.write_all(b"Hello")
                    .await
                    .map_err(|err| Error::WriteHelloError(err.to_string()))?;

                let mut hello = [0_u8; 5];
                rx.read_exact(&mut hello)
                    .await
                    .map_err(|err| Error::ReadHelloError(err.to_string()))?;

                if hello != *b"Hello" {
                    return Err(Error::InvalidHelloPayload);
                }

                Ok((tx, rx))
            }
            .await;

            let _ = response_tx.send(response);
        });
    }
}
