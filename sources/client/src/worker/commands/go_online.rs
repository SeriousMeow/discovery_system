use thiserror::Error;

use crate::utils::rpc;
use crate::worker::discovery_module::DiscoveryModule;

#[derive(Error, Debug)]
pub enum Error {
    #[error("already online")]
    AlreadyOnline,
    #[error("invalid discovery server url")]
    InvalidUrl(#[from] anyhow::Error),
}

pub type Command = rpc::Command<Request, Response>;

pub struct Request {
    pub server_url: String,
}

pub type Response = Result<(), Error>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        if self.discovery_module.is_some() {
            let _ = command.response_tx.send(Err(Error::AlreadyOnline));
            return;
        }

        let server_url = command.request.server_url;
        let module = match DiscoveryModule::new(
            self.pending_connections.clone(),
            self.connections.clone(),
            server_url,
        )
        .await
        {
            Ok(m) => m,
            Err(e) => {
                let _ = command.response_tx.send(Err(Error::from(e)));
                return;
            }
        };

        self.discovery_module = Some(module);
        let _ = command.response_tx.send(Ok(()));
    }
}
