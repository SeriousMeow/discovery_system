use crate::utils::rpc;
use iroh::EndpointId;

pub type Command = rpc::Command<Request, Response>;

pub type Request = ();
pub type Response = Option<EndpointId>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let id = self
            .discovery_module
            .as_ref()
            .map(|m| m.endpoint.id());
        command.send_result(id);
    }
}
