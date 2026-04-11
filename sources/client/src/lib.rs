use std::time::Duration;
use tokio::sync::mpsc::channel;

mod api;
mod handle;
pub(crate) mod utils;
mod worker;

pub use handle::DiscoveryIdentity;
pub use handle::Handle;
pub use handle::error;
pub use pow_puzzle::{self, StaticPuzzleSolution, generate_static_solution};
pub use worker::Worker;

pub struct Config {
    pub command_buffer_size: usize,
    pub poll_interval: Duration,
    pub puzzle_min_difficulty_bits: u32,
}

pub fn init(config: Config) -> (Handle, Worker) {
    let (tx, rx) = channel(config.command_buffer_size);

    (Handle::new(tx), Worker::new(rx, config))
}
