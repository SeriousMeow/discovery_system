use crate::utils::rpc;

pub type Command = rpc::Command<Request, Response>;

pub type Request = ();
pub type Response = bool;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        command.send_result(self.discovery_module.is_some());
    }
}
