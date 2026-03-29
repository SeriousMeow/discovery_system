use std::time::Duration;
use tokio::sync::mpsc::channel;

mod api;
mod handle;
pub(crate) mod utils;
mod worker;

pub use handle::error;
pub use handle::Handle;
pub use worker::Worker;

pub struct Config {
    pub command_buffer_size: usize,
    pub poll_interval: Duration,
}

pub fn init(config: Config) -> (Handle, Worker) {
    let (tx, rx) = channel(config.command_buffer_size);

    (Handle::new(tx), Worker::new(rx, config))
}
