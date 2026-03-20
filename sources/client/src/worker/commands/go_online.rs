use thiserror::Error;

use crate::utils::rpc;
use crate::worker::discovery_module::DiscoveryModule;

#[derive(Error, Debug)]
pub enum Error {
    #[error("already online")]
    AlreadyOnline,
}

pub type Command = rpc::Command<Request, Response>;

pub type Request = ();
pub type Response = Result<(), Error>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        if self.discovery_module.is_some() {
            let _ = command.response_tx.send(Err(Error::AlreadyOnline));
            return;
        }

        self.discovery_module = Some(
            DiscoveryModule::new(self.pending_connections.clone(), self.connections.clone())
                .await
                .unwrap(),
        );
        let _ = command.response_tx.send(Ok(()));
    }
}
