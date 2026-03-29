use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use iroh::endpoint::{RecvStream, SendStream};
use tokio::sync::{Mutex, RwLock};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub const MAX_INCOMING_QUEUE: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamSessionError {
    UnknownStreamId,
    NotActive { detail: &'static str },
    Utf8Framing,
    QueueOverflow,
    Io(String),
}

#[derive(Debug, Clone)]
pub enum LifecycleState {
    Active,
    Closed,
    Failed(StreamSessionError),
}

pub struct StreamSession {
    lifecycle: Arc<RwLock<LifecycleState>>,
    terminal: Arc<RwLock<Option<StreamSessionError>>>,
    send: Mutex<FramedWrite<SendStream, LengthDelimitedCodec>>,
    incoming: Arc<Mutex<VecDeque<String>>>,
}

impl StreamSession {
    async fn check_usable(&self) -> Result<(), StreamSessionError> {
        if let Some(e) = self.terminal.read().await.clone() {
            return Err(e);
        }
        match self.lifecycle.read().await.clone() {
            LifecycleState::Active => Ok(()),
            LifecycleState::Closed => Err(StreamSessionError::NotActive {
                detail: "stream closed",
            }),
            LifecycleState::Failed(e) => Err(e),
        }
    }

    pub async fn send_string(&self, message: &str) -> Result<(), StreamSessionError> {
        self.check_usable().await?;
        let mut sink = self.send.lock().await;
        sink.send(Bytes::copy_from_slice(message.as_bytes()))
            .await
            .map_err(|e| StreamSessionError::Io(e.to_string()))?;
        Ok(())
    }

    pub async fn drain_incoming(&self) -> Result<Vec<String>, StreamSessionError> {
        if let Some(e) = self.terminal.read().await.clone() {
            return Err(e);
        }
        let mut q = self.incoming.lock().await;
        Ok(q.drain(..).collect())
    }
}

fn spawn_reader(
    mut framed: FramedRead<RecvStream, LengthDelimitedCodec>,
    incoming: Arc<Mutex<VecDeque<String>>>,
    lifecycle: Arc<RwLock<LifecycleState>>,
    terminal: Arc<RwLock<Option<StreamSessionError>>>,
) {
    tokio::spawn(async move {
        loop {
            let next = framed.next().await;
            match next {
                Some(Ok(frame)) => {
                    let s = match String::from_utf8(frame.to_vec()) {
                        Ok(s) => s,
                        Err(_) => {
                            let err = StreamSessionError::Utf8Framing;
                            *terminal.write().await = Some(err.clone());
                            *lifecycle.write().await = LifecycleState::Failed(err);
                            return;
                        }
                    };
                    let mut q = incoming.lock().await;
                    if q.len() >= MAX_INCOMING_QUEUE {
                        let err = StreamSessionError::QueueOverflow;
                        *terminal.write().await = Some(err.clone());
                        *lifecycle.write().await = LifecycleState::Failed(err);
                        return;
                    }
                    q.push_back(s);
                }
                Some(Err(e)) => {
                    let err = StreamSessionError::Io(e.to_string());
                    *terminal.write().await = Some(err.clone());
                    *lifecycle.write().await = LifecycleState::Failed(err);
                    return;
                }
                None => {
                    *lifecycle.write().await = LifecycleState::Closed;
                    return;
                }
            }
        }
    });
}

#[derive(Clone)]
pub struct StreamSessionStore {
    sessions: Arc<RwLock<HashMap<String, Arc<StreamSession>>>>,
}

impl StreamSessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_session(
        &self,
        send: SendStream,
        recv: RecvStream,
    ) -> String {
        let codec = LengthDelimitedCodec::new();
        let framed_write = FramedWrite::new(send, codec.clone());
        let framed_read = FramedRead::new(recv, codec);

        let lifecycle = Arc::new(RwLock::new(LifecycleState::Active));
        let terminal = Arc::new(RwLock::new(None));
        let incoming = Arc::new(Mutex::new(VecDeque::new()));

        spawn_reader(
            framed_read,
            incoming.clone(),
            lifecycle.clone(),
            terminal.clone(),
        );

        let session = Arc::new(StreamSession {
            lifecycle,
            terminal,
            send: Mutex::new(framed_write),
            incoming,
        });

        let stream_id = uuid::Uuid::new_v4().to_string();
        self.sessions
            .write()
            .await
            .insert(stream_id.clone(), session);
        stream_id
    }

    pub async fn get(&self, stream_id: &str) -> Option<Arc<StreamSession>> {
        self.sessions.read().await.get(stream_id).cloned()
    }

    pub async fn send_string(&self, stream_id: &str, message: &str) -> Result<(), StreamSessionError> {
        let session = self
            .get(stream_id)
            .await
            .ok_or(StreamSessionError::UnknownStreamId)?;
        session.send_string(message).await
    }

    pub async fn drain_incoming(&self, stream_id: &str) -> Result<Vec<String>, StreamSessionError> {
        let session = self
            .get(stream_id)
            .await
            .ok_or(StreamSessionError::UnknownStreamId)?;
        session.drain_incoming().await
    }

    pub async fn remove(&self, stream_id: &str) -> Result<(), StreamSessionError> {
        self.sessions
            .write()
            .await
            .remove(stream_id)
            .ok_or(StreamSessionError::UnknownStreamId)?;
        Ok(())
    }
}

impl Default for StreamSessionStore {
    fn default() -> Self {
        Self::new()
    }
}
