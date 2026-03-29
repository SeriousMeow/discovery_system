use iroh::EndpointId;
use iroh::endpoint::{RecvStream, SendStream};
use std::time::Duration;
use tokio::sync::mpsc::Sender;

use crate::handle::error::{AcceptConnectionError, GoOnlineError};
use crate::worker::commands;

#[derive(Clone)]
pub struct Handle {
    commands_sender: Sender<commands::WorkerCommand>,
}

impl Handle {
    pub(crate) fn new(commands_sender: Sender<commands::WorkerCommand>) -> Self {
        Self {
            commands_sender: commands_sender,
        }
    }

    pub async fn connect(
        &self,
        peer_id: EndpointId,
        timeout: Duration,
    ) -> Result<(), error::ConnectError> {
        use crate::worker::commands::connect::*;

        let request = Request { peer_id, timeout };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn go_online(&self, server_url: String) -> Result<(), GoOnlineError> {
        use crate::worker::commands::go_online::*;

        let request = Request { server_url };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn accept_connection(
        &self,
        peer_id: EndpointId,
        timeout: Duration,
    ) -> Result<(), AcceptConnectionError> {
        use crate::worker::commands::accept_connection::*;

        let request = Request { peer_id, timeout };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn open_stream(
        &self,
        peer_id: EndpointId,
    ) -> Result<(SendStream, RecvStream), error::OpenStreamError> {
        use crate::worker::commands::open_stream::*;

        let request = Request { peer_id };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn accept_stream(
        &self,
        peer_id: EndpointId,
    ) -> Result<(SendStream, RecvStream), error::AcceptStreamError> {
        use crate::worker::commands::accept_stream::*;

        let request = Request { peer_id };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn is_online(&self) -> bool {
        use crate::worker::commands::is_online::*;

        let (command, rx) = Command::new(());

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn self_id(&self) -> Option<EndpointId> {
        use crate::worker::commands::self_id::*;

        let (command, rx) = Command::new(());

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn list_connections(&self) -> Vec<EndpointId> {
        use crate::worker::commands::list_connections::*;

        let (command, rx) = Command::new(Request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn list_incoming_connections(&self) -> Vec<EndpointId> {
        use crate::worker::commands::list_incoming_connections::*;

        let (command, rx) = Command::new(Request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }
}

pub mod error {
    use crate::worker::commands;

    pub type ConnectError = commands::connect::Error;
    pub type AcceptConnectionError = commands::accept_connection::Error;
    pub type AcceptStreamError = commands::accept_stream::Error;
    pub type OpenStreamError = commands::open_stream::Error;
    pub type GoOnlineError = commands::go_online::Error;
}
