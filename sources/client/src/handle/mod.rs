use iroh::EndpointId;
use iroh::endpoint::{RecvStream, SendStream};
use std::time::Duration;
use tokio::sync::mpsc::Sender;

use crate::handle::error::{
    AcceptConnectionError, AddDiscoveryServerError, GoOnlineError, RemoveDiscoveryServerError,
};
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
        peer_discovery_servers: Vec<String>,
        timeout: Duration,
    ) -> Result<(), error::ConnectError> {
        use crate::worker::commands::connect::*;

        let request = Request {
            peer_id,
            peer_discovery_servers,
            timeout,
        };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn go_online(&self) -> Result<(), GoOnlineError> {
        use crate::worker::commands::go_online::*;

        let (command, rx) = Command::new(());

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

    pub async fn add_discovery_server(&self, url: String) -> Result<(), AddDiscoveryServerError> {
        use crate::worker::commands::add_discovery_server::*;

        let request = Request { url };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn remove_discovery_server(
        &self,
        url: String,
    ) -> Result<(), RemoveDiscoveryServerError> {
        use crate::worker::commands::remove_discovery_server::*;

        let request = Request { url };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn list_discovery_servers(&self) -> Vec<String> {
        use crate::worker::commands::list_discovery_servers::*;

        let (command, rx) = Command::new(());

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
    pub type AddDiscoveryServerError = commands::add_discovery_server::Error;
    pub type RemoveDiscoveryServerError = commands::remove_discovery_server::Error;
}
