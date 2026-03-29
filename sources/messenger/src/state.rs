use client::Handle;

use crate::stream_session::StreamSessionStore;

#[derive(Clone)]
pub struct MessengerState {
    pub handle: Handle,
    pub streams: StreamSessionStore,
}

impl MessengerState {
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
            streams: StreamSessionStore::new(),
        }
    }
}
