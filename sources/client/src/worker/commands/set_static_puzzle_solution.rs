use crate::utils::rpc;
use pow_puzzle::StaticPuzzleSolution;

pub type Command = rpc::Command<Request, Response>;

pub struct Request {
    pub solution: StaticPuzzleSolution,
}

pub type Response = ();

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        self.pending_static_solution = Some(command.request.solution);
        let _ = command.response_tx.send(());
    }
}
