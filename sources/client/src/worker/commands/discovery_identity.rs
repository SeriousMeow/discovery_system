use crate::DiscoveryIdentity;
use crate::utils::rpc;

pub type Command = rpc::Command<Request, Response>;

pub type Request = ();
pub type Response = Option<DiscoveryIdentity>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let out = self.discovery_module.as_ref().map(|m| DiscoveryIdentity {
            public_key_base64: m.public_key_base64(),
            public_key_hex: m.public_key_hex(),
        });
        command.send_result(out);
    }
}
