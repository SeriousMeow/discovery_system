use thiserror::Error;

use crate::utils::rpc;
use pow_puzzle::{PuzzleError, generate_static_solution};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Puzzle(#[from] PuzzleError),
}

pub type Command = rpc::Command<Request, Response>;

pub struct Request {
    pub min_difficulty_bits: u32,
}

pub type Response = Result<(), Error>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        let request = command.request;
        let response = match generate_static_solution(request.min_difficulty_bits) {
            Ok((solution, _achieved)) => {
                self.pending_static_solution = Some(solution);
                Ok(())
            }
            Err(e) => Err(Error::Puzzle(e)),
        };
        let _ = command.response_tx.send(response);
    }
}
