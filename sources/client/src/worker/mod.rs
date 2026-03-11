use tokio::sync::mpsc::Receiver;

pub(crate) mod commands;
use commands::Command;

pub struct Worker {
    commands_receiver: Receiver<Command>,
}

impl Worker {
    pub async fn run() {}

    pub(crate) fn new(commands_receiver: Receiver<Command>) -> Self {
        Self { commands_receiver }
    }
}
