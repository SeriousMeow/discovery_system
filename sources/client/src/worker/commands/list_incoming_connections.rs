use crate::utils::rpc;
use iroh::EndpointId;

pub type Command = rpc::Command<Request, Response>;
pub struct Request;
pub type Response = Vec<EndpointId>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let peer_ids = self.pending_incoming_connections.keys().copied().collect();
        command.send_result(peer_ids);
    }
}
