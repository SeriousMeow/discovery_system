use anyhow::Result;
use futures_util::SinkExt;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::broadcast::NodeId;
use crate::broadcast::message::Message;

pub type Sender = mpsc::Sender<Message>;
pub type Receiver = mpsc::Receiver<Message>;

pub struct ConnectionsManager {
    connections: Arc<RwLock<HashMap<NodeId, Sender>>>,
    pub new_messages: Receiver,
    base_sender: Sender,
    buffer_size: usize,
}

impl ConnectionsManager {
    pub fn new(buffer_size: usize, listener: TcpListener) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        let connections = Arc::new(RwLock::new(HashMap::new()));

        let listener_sender = tx.clone();
        let listener_connections = connections.clone();

        tokio::spawn(async move {
            listen(listener, listener_sender, listener_connections, buffer_size).await?;

            Ok::<_, anyhow::Error>(())
        });

        Self {
            connections: connections,
            new_messages: rx,
            base_sender: tx,
            buffer_size: buffer_size,
        }
    }

    pub async fn send(&mut self, id: NodeId, message: Message) {
        if let Some(tx) = self.connections.read().await.get(&id) {
            let _ = tx.send(message).await;
            return;
        }

        let Ok(tx) = self.connect(id) else {
            return;
        };

        if tx.send(message).await.is_err() {
            return;
        }

        self.connections.write().await.insert(id, tx);
    }

    pub fn connect(&mut self, id: NodeId) -> Result<Sender> {
        let (in_tx, in_rx) = mpsc::channel::<Message>(self.buffer_size);
        let out_tx = self.base_sender.clone();

        tokio::spawn(async move {
            let connection = TcpStream::connect(id).await?;

            let _ = handle_connection(id, connection, in_rx, &out_tx).await;

            Ok::<_, anyhow::Error>(())
        });

        Ok(in_tx)
    }

    pub async fn disconnet(&mut self, id: NodeId) {
        self.connections.write().await.remove(&id);
    }

    pub async fn cleanup(&mut self) {
        self.connections
            .write()
            .await
            .retain(|_, tx| tx.is_closed());
    }
}

async fn listen(
    listener: TcpListener,
    base_sender: Sender,
    connections: Arc<RwLock<HashMap<NodeId, Sender>>>,
    buffer_size: usize,
) -> Result<()> {
    loop {
        let (connection, id) = listener.accept().await?;
        let (in_tx, in_rx) = mpsc::channel(buffer_size);

        connections.write().await.insert(id, in_tx);

        let out_tx = base_sender.clone();
        tokio::spawn(async move {
            let _ = handle_connection(id, connection, in_rx, &out_tx).await;
        });
    }
}

async fn handle_connection(
    id: NodeId,
    connection: TcpStream,
    mut in_rx: Receiver,
    out_tx: &Sender,
) -> Result<()> {
    let codec = LengthDelimitedCodec::new();

    let mut connection = Framed::new(connection, codec);

    loop {
        tokio::select! {
            result = connection.next() => {
                let Some(Ok(raw_msg)) = result else { break; };

                let Ok(msg) = serde_json::from_slice(&raw_msg) else {break;};

                if out_tx.send(msg).await.is_err() {
                    break;
                }
            },
            result = in_rx.recv() => {
                let Some(msg) = result else { break; };

                let raw_msg = serde_json::to_vec(&msg)?;

                connection.send(raw_msg.into()).await?;
            },
        };
    }

    let _ = connection.close().await;

    let msg = hyparview::message::DisconnectMessage {
        sender: id,
        alive: false,
    };

    let _ = out_tx.send(Message::Membership(msg.into())).await;
    Ok(())
}
