use crate::utils::rpc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("offline")]
    Offline,
    #[error("invalid discovery server url")]
    InvalidUrl(#[from] anyhow::Error),
}

pub type Command = rpc::Command<Request, Response>;
pub struct Request {
    pub url: String,
}
pub type Response = Result<(), Error>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let Some(module) = self.discovery_module.as_mut() else {
            command.send_result(Err(Error::Offline));
            return;
        };

        let request = command.request;
        let response_tx = command.response_tx;
        let result = module.add_server(request.url).map_err(Error::from);
        let _ = response_tx.send(result);
    }
}
