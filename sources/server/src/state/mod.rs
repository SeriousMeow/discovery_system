pub mod storage;

use std::sync::{Arc, RwLock};
use storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<RwLock<Storage>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(Storage::new())),
        }
    }
}
