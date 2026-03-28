use std::collections::VecDeque;
use std::net::IpAddr;

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
use tracing::{debug, info, trace};
use uuid::Uuid;

use crate::state::storage::{KeyType, ValueType};

pub mod config;

mod connections_manager;
use connections_manager::ConnectionsManager;

mod message;
use message::Message;

pub type MessageSender = mpsc::Sender<Payload>;
pub type MessageReceiver = mpsc::Receiver<Payload>;

pub type NodeId = IpAddr;
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
        local_node_id: IpAddr,
        contact_node: Option<NodeId>,
    ) -> Result<(MessageSender, MessageReceiver, Self)> {
        let (in_tx, in_rx) = mpsc::channel(config.broadcast_buffer_size);

        let (out_tx, out_rx) = mpsc::channel(config.new_messages_buffer_size);

        let bind_addr = config.listening_address;
        let listener = TcpListener::bind(bind_addr).await?;
        info!("broadcast listening on {}", bind_addr);

        let mut membership_node = match config.membership_options {
            Some(options) => {
                hyparview::Node::with_options(local_node_id, StdRng::from_entropy(), options)
            }
            None => hyparview::Node::new(local_node_id, StdRng::from_entropy()),
        };

        if let Some(contact_node) = contact_node {
            membership_node.join(contact_node);
        }

        let broadcast_node = match config.broadcast_option {
            Some(options) => plumtree::Node::with_options(local_node_id, options),
            None => plumtree::Node::new(local_node_id),
        };

        let worker = Self {
            to_broadcast: in_rx,
            new_messages: out_tx,
            connections_manager: ConnectionsManager::new(
                config.internal_messages_buffer_size,
                listener,
                bind_addr.port(),
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
                _ = shuffle_passive_interval.tick() => {
                    trace!("Shuffling passive view");
                    self.membership_node.shuffle_passive_view()}
                ,
                _ = fill_active_interval.tick() => {
                    trace!("Filling active view");
                    self.membership_node.fill_active_view()},
                _ = sync_active_interval.tick() => {
                    trace!("Syncing active view");
                    self.membership_node.sync_active_view()
                },
                _ = cleanup_interval.tick() => {
                    trace!("Cleaning up closed connections");
                    self.connections_manager.cleanup().await
                },
                now = broadcast_tick_interval.tick() => {
                    trace!("Ticking broadcast node");
                    let elapsed = now.duration_since(last_ticked);
                    last_ticked = now;
                    self.broadcast_node.clock_mut().tick(elapsed);
                },
                _ = membership_poll_interval.tick() => {
                    trace!("Polling membership node for actions");
                    while let Some(action) = self.membership_node.poll_action() {
                        self.handle_membership_action(action).await;
                    };
                    trace!("Finished polling membership node for actions");
                },
                _ = broadcast_poll_interval.tick() => {
                    trace!("Polling broadcast node for actions");
                    while let Some(action) = self.broadcast_node.poll_action() {
                        self.handle_broadcast_action(action).await;
                    };
                    trace!("Finished polling broadcast node for actions");
                },
                Some(message) = self.connections_manager.new_messages.recv() => {
                    let json = serde_json::to_string(&message)
                                .unwrap_or_else(|e| format!("<json error: {e}>"));
                    let user_gossip = message.is_plumtree_user_gossip();
                    match message {
                        Message::Membership(message) => {
                            trace!(
                                message = %json,
                                "Received membership message"
                            );
                            self.membership_node.handle_protocol_message(message)
                        },
                        Message::Broadcast(message) => {
                            if user_gossip {
                                debug!(
                                    message = %json,
                                    "Received broadcast message"
                                );
                            } else {
                                trace!(
                                    message = %json,
                                    "Received plumtree broadcast control message"
                                );
                            }
                            let _ = self.broadcast_node.handle_protocol_message(message);
                        }
                    }
                },
                Some(broadcast_message) = self.to_broadcast.recv() => {
                    let id = Uuid::new_v4();
                    debug!("Enqueuing message {} to {} for broadcast", id, broadcast_message.to);
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
                trace!("Sending membership message to {}", destination);
                self.connections_manager
                    .send(destination, Message::Membership(message))
                    .await
            }

            hyparview::Action::Disconnect { node } => {
                trace!("Disconnecting from node: {}", node);
                self.connections_manager.disconnet(node).await
            }

            hyparview::Action::Notify { event } => match event {
                hyparview::Event::NeighborUp { node } => {
                    trace!("Handling NeighborUp for node: {}", node);
                    self.broadcast_node.handle_neighbor_up(&node)
                }
                hyparview::Event::NeighborDown { node } => {
                    trace!("Handling NeighborDown for node: {}", node);
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
                let wire: Message = message.into();
                if wire.is_plumtree_user_gossip() {
                    debug!("Sending broadcast message to {}", destination);
                } else {
                    trace!("Sending broadcast message to {}", destination);
                }
                self.connections_manager.send(destination, wire).await;
            }
            plumtree::Action::Deliver { message } => {
                let message_id = message.id;
                debug!("Delivering broadcast message {}", message_id);
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
