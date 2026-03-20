use crate::utils::rpc;

pub type Command = rpc::Command<Request, Response>;
pub type Request = ();
pub type Response = Vec<String>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let servers = self
            .discovery_module
            .as_ref()
            .map(|module| module.list_servers())
            .unwrap_or_default();
        command.send_result(servers);
    }
}
