use anyhow::Result;
use futures_util::SinkExt;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info, trace, warn};

use crate::broadcast::NodeId;
use crate::broadcast::message::Message;

pub type Sender = mpsc::Sender<Message>;
pub type Receiver = mpsc::Receiver<Message>;

pub struct ConnectionsManager {
    connections: Arc<RwLock<HashMap<NodeId, Sender>>>,
    pub new_messages: Receiver,
    base_sender: Sender,
    buffer_size: usize,
    peer_port: u16,
}

impl ConnectionsManager {
    pub fn new(buffer_size: usize, listener: TcpListener, peer_port: u16) -> Self {
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
            peer_port,
        }
    }

    pub async fn send(&mut self, id: NodeId, message: Message) {
        if let Some(tx) = self.connections.read().await.get(&id) {
            if message.is_plumtree_user_gossip() {
                debug!("Sending message to {} via existing channel", id);
            } else {
                trace!("Sending message to {} via existing channel", id);
            }
            let _ = tx.send(message).await;
            return;
        }

        if message.is_plumtree_user_gossip() {
            debug!("Initializing connection to {}", id);
        } else {
            trace!("Initializing connection to {}", id);
        }

        let tx = match self.connect(id) {
            Ok(tx) => tx,
            Err(e) => {
                warn!("Failed to connect to {}: {}", id, e);
                return;
            }
        };

        let user_gossip = message.is_plumtree_user_gossip();
        if user_gossip {
            debug!("Sending message to {} via new channel", id);
        } else {
            trace!("Sending message to {} via new channel", id);
        }
        if let Err(e) = tx.send(message).await {
            warn!("Failed to send message to {}: {}", id, e);
            return;
        }

        self.connections.write().await.insert(id, tx);
        if user_gossip {
            debug!("Connection to {} saved in connections manager", id);
        } else {
            trace!("Connection to {} saved in connections manager", id);
        }
    }

    pub fn connect(&mut self, id: NodeId) -> Result<Sender> {
        let (in_tx, in_rx) = mpsc::channel::<Message>(self.buffer_size);
        let out_tx = self.base_sender.clone();
        let peer_port = self.peer_port;

        tokio::spawn(async move {
            let peer_addr = SocketAddr::new(id, peer_port);
            info!("Starting connection to {}", peer_addr);
            let connection = match TcpStream::connect(peer_addr).await {
                Ok(connection) => connection,
                Err(e) => {
                    warn!("Failed to connect to {}: {}", peer_addr, e);
                    return Err(anyhow::Error::new(e));
                }
            };
            info!("Connection to {} established", peer_addr);

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
            .retain(|_, tx| !tx.is_closed());
    }
}

async fn listen(
    listener: TcpListener,
    base_sender: Sender,
    connections: Arc<RwLock<HashMap<NodeId, Sender>>>,
    buffer_size: usize,
) -> Result<()> {
    loop {
        let (connection, peer_addr) = listener.accept().await?;
        let node_id = peer_addr.ip();
        info!("Accepted new incoming connection from {}", peer_addr);
        let (in_tx, in_rx) = mpsc::channel(buffer_size);

        connections.write().await.insert(node_id, in_tx);

        let out_tx = base_sender.clone();
        tokio::spawn(async move {
            let _ = handle_connection(node_id, connection, in_rx, &out_tx).await;
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
                let raw_msg = match result {
                    None => {
                        info!("Closing connection to {}: stream ended", id);
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("Closing connection to {}: read error: {}", id, e);
                        break;
                    }
                    Some(Ok(raw_msg)) => raw_msg,
                };

                let Ok(msg) = serde_json::from_slice::<Message>(&raw_msg) else {
                    warn!(
                        "closing connection to {id}: invalid message"
                    );
                    break;
                };

                if msg.is_plumtree_user_gossip() {
                    debug!("Received message from {}", id);
                } else {
                    trace!("Received message from {}", id);
                }

                if let Err(e) = out_tx.send(msg).await {
                    warn!("Failed to send message to {}: {}", id, e);
                    break;
                }
            },
            result = in_rx.recv() => {
                let Some(msg) = result else {
                    info!("Closing connection to {}: message receiver closed", id);
                    break;
                };

                let raw_msg = serde_json::to_vec(&msg)?;
                if msg.is_plumtree_user_gossip() {
                    debug!("Sending message to {}", id);
                } else {
                    trace!("Sending message to {}", id);
                }

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
