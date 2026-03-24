use std::net::SocketAddr;

use anyhow::Result;
use hyparview::{self, Action};
use rand::FromEntropy;
use rand::rngs::StdRng;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio::time::Interval;

pub mod config;

mod connections_manager;
use connections_manager::ConnectionsManager;

use crate::broadcast::message::Message;

mod message;

pub type MessageSender = mpsc::Sender<Message>;
pub type MessageReceiver = mpsc::Receiver<Message>;

pub type NodeId = SocketAddr;

pub struct BroadcastWorker {
    to_broadcast: MessageReceiver,
    new_messages: MessageSender,
    connections_manager: ConnectionsManager,
    membership_node: hyparview::Node<NodeId, StdRng>,
    membership_intervals_config: config::MembershipIntervals,
}

impl BroadcastWorker {
    pub async fn new(config: config::Config) -> Result<(MessageSender, MessageReceiver, Self)> {
        let (in_tx, in_rx) = mpsc::channel(config.broadcast_buffer_size);

        let (out_tx, out_rx) = mpsc::channel(config.new_messages_buffer_size);

        let id = config.listening_address;
        let listener = TcpListener::bind(id).await?;

        let membership_node = match config.membership_options {
            Some(options) => hyparview::Node::with_options(id, StdRng::from_entropy(), options),
            None => hyparview::Node::new(id, StdRng::from_entropy()),
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
        let mut poll_interval = get_interval(self.membership_intervals_config.poll);
        let mut cleanup_interval = get_interval(self.membership_intervals_config.cleanup);

        loop {
            select! {
                _ = shuffle_passive_interval.tick() => self.membership_node.shuffle_passive_view(),
                _ = fill_active_interval.tick() => self.membership_node.fill_active_view(),
                _ = sync_active_interval.tick() => self.membership_node.sync_active_view(),
                _ = cleanup_interval.tick() => self.connections_manager.cleanup().await,
                _ = poll_interval.tick() => {
                    let action = match self.membership_node.poll_action() {
                        Some(action) => action,
                        None => continue
                    };
                    self.handle_membership_action(action).await;
                },
                Some(_message) = self.connections_manager.new_messages.recv() => {
                    // Handle message in Plumtree
                },
                else => break
            }
        }

        Ok(())
    }

    async fn handle_membership_action(&mut self, action: hyparview::Action<NodeId>) {
        match action {
            Action::Send {
                destination,
                message,
            } => {
                self.connections_manager
                    .send(destination, Message::Membership(message))
                    .await
            }

            Action::Disconnect { node } => self.connections_manager.disconnet(node).await,

            Action::Notify { event } => {
                // Implement for Plumtree
            }
        }
    }
}
