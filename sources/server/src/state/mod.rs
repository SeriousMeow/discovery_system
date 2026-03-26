pub mod storage;

use std::sync::{Arc, RwLock};
use storage::Storage;

use crate::broadcast::{MessageReceiver, MessageSender};

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<RwLock<Storage>>,
    pub broadcast_tx: MessageSender,
}

impl AppState {
    pub fn new(broadcast_tx: MessageSender) -> Self {
        Self {
            storage: Arc::new(RwLock::new(Storage::new())),
            broadcast_tx: broadcast_tx,
        }
    }
}

pub async fn handle_incoming(mut incoming_rx: MessageReceiver, storage: Arc<RwLock<Storage>>) {
    while let Some(message) = incoming_rx.recv().await {
        storage.write().unwrap().insert(&message.to, message.data);
    }
}
