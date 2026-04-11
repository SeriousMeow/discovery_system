use iroh::EndpointId;
use iroh::endpoint::{RecvStream, SendStream};
use std::time::Duration;
use tokio::sync::mpsc::Sender;

use crate::handle::error::{AcceptConnectionError, GoOnlineError};

/// Identity on the Discovery server: static puzzle public key (same as `credentials.public_key` in Discovery HTTP APIs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryIdentity {
    pub public_key_base64: String,
    pub public_key_hex: String,
}
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

    pub async fn go_online(
        &self,
        server_url: String,
        discovery_public_key: Option<String>,
    ) -> Result<(), GoOnlineError> {
        use crate::worker::commands::go_online::*;

        let request = Request {
            server_url,
            discovery_public_key,
        };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn set_static_puzzle_solution(&self, solution: crate::StaticPuzzleSolution) {
        use crate::worker::commands::set_static_puzzle_solution::*;

        let request = Request { solution };
        let (command, rx) = Command::new(request);

        self.commands_sender.send(command.into()).await.unwrap();

        let _ = rx.await;
    }

    pub async fn generate_static_puzzle_solution(
        &self,
        min_difficulty_bits: u32,
    ) -> Result<(), error::GenerateStaticPuzzleSolutionError> {
        use crate::worker::commands::generate_static_puzzle_solution::*;

        let request = Request {
            min_difficulty_bits,
        };
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

    /// Discovery registration identity (puzzle public key), if [`go_online`](Self::go_online) succeeded.
    pub async fn discovery_identity(&self) -> Option<DiscoveryIdentity> {
        use crate::worker::commands::discovery_identity::*;

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
    pub type GenerateStaticPuzzleSolutionError = commands::generate_static_puzzle_solution::Error;
    pub type AcceptConnectionError = commands::accept_connection::Error;
    pub type AcceptStreamError = commands::accept_stream::Error;
    pub type OpenStreamError = commands::open_stream::Error;
    pub type GoOnlineError = commands::go_online::Error;
}
