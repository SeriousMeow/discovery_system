use tokio::sync::mpsc::channel;

mod api;
mod handle;
mod worker;

pub use handle::EmptyHandle;
pub use worker::Worker;

pub struct Config {
    pub command_buffer_size: usize,
}

pub fn init(config: Config) -> (EmptyHandle, Worker) {
    let (tx, rx) = channel(config.command_buffer_size);

    (EmptyHandle::new(tx), Worker::new(rx))
}
