use std::collections::VecDeque;
use std::net::SocketAddr;

use anyhow::Result;
use rand::FromEntropy;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio::time::Instant;
use tokio::time::Interval;
use uuid::Uuid;

use crate::state::storage::{KeyType, ValueType};

pub mod config;

mod connections_manager;
use connections_manager::ConnectionsManager;

mod message;
use message::Message;

pub type MessageSender = mpsc::Sender<Payload>;
pub type MessageReceiver = mpsc::Receiver<Payload>;

pub type NodeId = SocketAddr;
pub type MessageId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Payload {
    pub to: KeyType,
    pub data: ValueType,
}

pub struct BroadcastSystem;

impl plumtree::System for BroadcastSystem {
    type NodeId = NodeId;
    type MessageId = MessageId;
    type MessagePayload = Payload;
}

pub struct BroadcastWorker {
    to_broadcast: MessageReceiver,
    new_messages: MessageSender,
    connections_manager: ConnectionsManager,
    membership_node: hyparview::Node<NodeId, StdRng>,
    membership_intervals_config: config::MembershipIntervals,
    broadcast_node: plumtree::Node<BroadcastSystem>,
    broadcast_intervals_config: config::BroadcastIntervals,
    delivered_message_ids: VecDeque<MessageId>,
    max_stored_messages: usize,
}

impl BroadcastWorker {
    pub async fn new(
        config: config::Config,
        contact_node: NodeId,
    ) -> Result<(MessageSender, MessageReceiver, Self)> {
        let (in_tx, in_rx) = mpsc::channel(config.broadcast_buffer_size);

        let (out_tx, out_rx) = mpsc::channel(config.new_messages_buffer_size);

        let id = config.listening_address;
        let listener = TcpListener::bind(id).await?;

        let mut membership_node = match config.membership_options {
            Some(options) => hyparview::Node::with_options(id, StdRng::from_entropy(), options),
            None => hyparview::Node::new(id, StdRng::from_entropy()),
        };

        membership_node.join(contact_node);

        let broadcast_node = match config.broadcast_option {
            Some(options) => plumtree::Node::with_options(id, options),
            None => plumtree::Node::new(id),
        };

        let worker = Self {
            to_broadcast: in_rx,
            new_messages: out_tx,
            connections_manager: ConnectionsManager::new(
                config.internal_messages_buffer_size,
                listener,
            ),
            membership_node: membership_node,
            membership_intervals_config: config.membership_intervals,
            broadcast_node: broadcast_node,
            broadcast_intervals_config: config.broadcast_intervals,
            delivered_message_ids: VecDeque::new(),
            max_stored_messages: config.max_stored_messages,
        };

        Ok((in_tx, out_rx, worker))
    }

    pub async fn run(mut self) -> Result<()> {
        fn get_interval(duration: Duration) -> Interval {
            use tokio::time::{MissedTickBehavior, interval};

            let mut int = interval(duration);
            int.set_missed_tick_behavior(MissedTickBehavior::Delay);

            int
        }

        let mut shuffle_passive_interval =
            get_interval(self.membership_intervals_config.shuffle_passive);
        let mut fill_active_interval = get_interval(self.membership_intervals_config.fill_active);
        let mut sync_active_interval = get_interval(self.membership_intervals_config.sync_active);
        let mut membership_poll_interval = get_interval(self.membership_intervals_config.poll);
        let mut cleanup_interval = get_interval(self.membership_intervals_config.cleanup);

        let mut broadcast_poll_interval = get_interval(self.broadcast_intervals_config.poll);
        let mut broadcast_tick_interval = get_interval(self.broadcast_intervals_config.tick);
        let mut last_ticked = Instant::now();

        loop {
            select! {
                _ = shuffle_passive_interval.tick() => self.membership_node.shuffle_passive_view(),
                _ = fill_active_interval.tick() => self.membership_node.fill_active_view(),
                _ = sync_active_interval.tick() => self.membership_node.sync_active_view(),
                _ = cleanup_interval.tick() => self.connections_manager.cleanup().await,
                now = broadcast_tick_interval.tick() => {
                    let elapsed = now.duration_since(last_ticked);
                    last_ticked = now;
                    self.broadcast_node.clock_mut().tick(elapsed);
                },
                _ = membership_poll_interval.tick() => {
                    while let Some(action) = self.membership_node.poll_action() {
                        self.handle_membership_action(action).await;
                    };
                },
                _ = broadcast_poll_interval.tick() => {
                    while let Some(action) = self.broadcast_node.poll_action() {
                        self.handle_broadcast_action(action).await;
                    };
                },
                Some(message) = self.connections_manager.new_messages.recv() => {
                    match message {
                        Message::Membership(message) => self.membership_node.handle_protocol_message(message),
                        Message::Broadcast(message) => {let _ =  self.broadcast_node.handle_protocol_message(message);}
                    }
                },
                Some(broadcast_message) = self.to_broadcast.recv() => {
                    let id = Uuid::new_v4();
                    let packed_message = plumtree::message::Message::new(id, broadcast_message);
                    self.broadcast_node.broadcast_message(packed_message);
                },
                else => break
            }
        }

        Ok(())
    }

    async fn handle_membership_action(&mut self, action: hyparview::Action<NodeId>) {
        match action {
            hyparview::Action::Send {
                destination,
                message,
            } => {
                self.connections_manager
                    .send(destination, Message::Membership(message))
                    .await
            }

            hyparview::Action::Disconnect { node } => {
                self.connections_manager.disconnet(node).await
            }

            hyparview::Action::Notify { event } => match event {
                hyparview::Event::NeighborUp { node } => {
                    self.broadcast_node.handle_neighbor_up(&node)
                }
                hyparview::Event::NeighborDown { node } => {
                    self.broadcast_node.handle_neighbor_down(&node)
                }
            },
        }
    }

    async fn handle_broadcast_action(&mut self, action: plumtree::Action<BroadcastSystem>) {
        match action {
            plumtree::Action::Send {
                destination,
                message,
            } => {
                self.connections_manager
                    .send(destination, message.into())
                    .await;
            }
            plumtree::Action::Deliver { message } => {
                let message_id = message.id;
                let _ = self.new_messages.send(message.payload).await;
                self.delivered_message_ids.push_back(message_id);

                while self.broadcast_node.messages().len() > self.max_stored_messages {
                    let Some(remove_id) = self.delivered_message_ids.pop_front() else {
                        break;
                    };
                    self.broadcast_node.forget_message(&remove_id);
                }
            }
        }
    }
}
