use client::{Config, init};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = Config {
        command_buffer_size: 1024,
        poll_interval: Duration::from_secs(5),
    };

    let (_, worker) = init(config);

    tokio::spawn(async move { worker.run().await.expect("Worker error") });
}
