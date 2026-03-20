use crate::utils::rpc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("offline")]
    Offline,
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

        module.remove_server(&command.request.url);
        command.send_result(Ok(()));
    }
}
