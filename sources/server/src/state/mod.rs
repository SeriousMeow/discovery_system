pub mod storage;

use std::sync::{Arc, Mutex};
use storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Mutex<Storage>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(Storage::new())),
        }
    }
}
