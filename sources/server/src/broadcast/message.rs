use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::broadcast::BroadcastSystem;
use crate::broadcast::NodeId;

#[derive(From, Clone)]
pub enum Message {
    Membership(hyparview::message::ProtocolMessage<NodeId>),
    Broadcast(plumtree::message::ProtocolMessage<BroadcastSystem>),
}

#[derive(Serialize, Deserialize)]
enum MessageMirror {
    Membership(membership::mirror::ProtocolMessage),
    Broadcast(broadcast::mirror::ProtocolMessage),
}

impl From<Message> for MessageMirror {
    fn from(value: Message) -> Self {
        match value {
            Message::Membership(value) => Self::Membership(value.into()),
            Message::Broadcast(value) => Self::Broadcast(value.into()),
        }
    }
}

impl From<MessageMirror> for Message {
    fn from(value: MessageMirror) -> Self {
        match value {
            MessageMirror::Membership(value) => Self::Membership(value.into()),
            MessageMirror::Broadcast(value) => Self::Broadcast(value.into()),
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mirror: MessageMirror = (*self).clone().into();
        mirror.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(MessageMirror::deserialize(deserializer)?.into())
    }
}

mod membership {
    use crate::broadcast::NodeId;

    mod time_to_live {
        use hyparview::TimeToLive;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        pub fn serialize<S>(ttl: &TimeToLive, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            ttl.as_u8().serialize(serializer)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<TimeToLive, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(TimeToLive::new(u8::deserialize(deserializer)?))
        }
    }

    pub mod mirror {
        use hyparview::TimeToLive;
        use serde::{Deserialize, Serialize};

        use crate::broadcast::NodeId;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct JoinMessage {
            pub sender: NodeId,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct ForwardJoinMessage {
            pub sender: NodeId,
            pub new_node: NodeId,
            #[serde(with = "super::time_to_live")]
            pub ttl: TimeToLive,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct NeighborMessage {
            pub sender: NodeId,
            pub high_priority: bool,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct ShuffleMessage {
            pub sender: NodeId,
            pub origin: NodeId,
            pub nodes: Vec<NodeId>,
            #[serde(with = "super::time_to_live")]
            pub ttl: TimeToLive,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct ShuffleReplyMessage {
            pub sender: NodeId,
            pub nodes: Vec<NodeId>,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct DisconnectMessage {
            pub sender: NodeId,
            pub alive: bool,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ProtocolMessage {
            Join(JoinMessage),
            ForwardJoin(ForwardJoinMessage),
            Neighbor(NeighborMessage),
            Shuffle(ShuffleMessage),
            ShuffleReply(ShuffleReplyMessage),
            Disconnect(DisconnectMessage),
        }
    }

    impl From<hyparview::message::JoinMessage<NodeId>> for mirror::JoinMessage {
        fn from(m: hyparview::message::JoinMessage<NodeId>) -> Self {
            Self { sender: m.sender }
        }
    }

    impl From<mirror::JoinMessage> for hyparview::message::JoinMessage<NodeId> {
        fn from(m: mirror::JoinMessage) -> Self {
            Self { sender: m.sender }
        }
    }

    impl From<hyparview::message::ForwardJoinMessage<NodeId>> for mirror::ForwardJoinMessage {
        fn from(m: hyparview::message::ForwardJoinMessage<NodeId>) -> Self {
            Self {
                sender: m.sender,
                new_node: m.new_node,
                ttl: m.ttl,
            }
        }
    }

    impl From<mirror::ForwardJoinMessage> for hyparview::message::ForwardJoinMessage<NodeId> {
        fn from(m: mirror::ForwardJoinMessage) -> Self {
            Self {
                sender: m.sender,
                new_node: m.new_node,
                ttl: m.ttl,
            }
        }
    }

    impl From<hyparview::message::NeighborMessage<NodeId>> for mirror::NeighborMessage {
        fn from(m: hyparview::message::NeighborMessage<NodeId>) -> Self {
            Self {
                sender: m.sender,
                high_priority: m.high_priority,
            }
        }
    }

    impl From<mirror::NeighborMessage> for hyparview::message::NeighborMessage<NodeId> {
        fn from(m: mirror::NeighborMessage) -> Self {
            Self {
                sender: m.sender,
                high_priority: m.high_priority,
            }
        }
    }

    impl From<hyparview::message::ShuffleMessage<NodeId>> for mirror::ShuffleMessage {
        fn from(m: hyparview::message::ShuffleMessage<NodeId>) -> Self {
            Self {
                sender: m.sender,
                origin: m.origin,
                nodes: m.nodes,
                ttl: m.ttl,
            }
        }
    }

    impl From<mirror::ShuffleMessage> for hyparview::message::ShuffleMessage<NodeId> {
        fn from(m: mirror::ShuffleMessage) -> Self {
            Self {
                sender: m.sender,
                origin: m.origin,
                nodes: m.nodes,
                ttl: m.ttl,
            }
        }
    }

    impl From<hyparview::message::ShuffleReplyMessage<NodeId>> for mirror::ShuffleReplyMessage {
        fn from(m: hyparview::message::ShuffleReplyMessage<NodeId>) -> Self {
            Self {
                sender: m.sender,
                nodes: m.nodes,
            }
        }
    }

    impl From<mirror::ShuffleReplyMessage> for hyparview::message::ShuffleReplyMessage<NodeId> {
        fn from(m: mirror::ShuffleReplyMessage) -> Self {
            Self {
                sender: m.sender,
                nodes: m.nodes,
            }
        }
    }

    impl From<hyparview::message::DisconnectMessage<NodeId>> for mirror::DisconnectMessage {
        fn from(m: hyparview::message::DisconnectMessage<NodeId>) -> Self {
            Self {
                sender: m.sender,
                alive: m.alive,
            }
        }
    }

    impl From<mirror::DisconnectMessage> for hyparview::message::DisconnectMessage<NodeId> {
        fn from(m: mirror::DisconnectMessage) -> Self {
            Self {
                sender: m.sender,
                alive: m.alive,
            }
        }
    }

    impl From<hyparview::message::ProtocolMessage<NodeId>> for mirror::ProtocolMessage {
        fn from(m: hyparview::message::ProtocolMessage<NodeId>) -> Self {
            match m {
                hyparview::message::ProtocolMessage::Join(x) => Self::Join(x.into()),
                hyparview::message::ProtocolMessage::ForwardJoin(x) => Self::ForwardJoin(x.into()),
                hyparview::message::ProtocolMessage::Neighbor(x) => Self::Neighbor(x.into()),
                hyparview::message::ProtocolMessage::Shuffle(x) => Self::Shuffle(x.into()),
                hyparview::message::ProtocolMessage::ShuffleReply(x) => {
                    Self::ShuffleReply(x.into())
                }
                hyparview::message::ProtocolMessage::Disconnect(x) => Self::Disconnect(x.into()),
            }
        }
    }

    impl From<mirror::ProtocolMessage> for hyparview::message::ProtocolMessage<NodeId> {
        fn from(m: mirror::ProtocolMessage) -> Self {
            match m {
                mirror::ProtocolMessage::Join(x) => Self::Join(x.into()),
                mirror::ProtocolMessage::ForwardJoin(x) => Self::ForwardJoin(x.into()),
                mirror::ProtocolMessage::Neighbor(x) => Self::Neighbor(x.into()),
                mirror::ProtocolMessage::Shuffle(x) => Self::Shuffle(x.into()),
                mirror::ProtocolMessage::ShuffleReply(x) => Self::ShuffleReply(x.into()),
                mirror::ProtocolMessage::Disconnect(x) => Self::Disconnect(x.into()),
            }
        }
    }
}

pub mod broadcast {
    use crate::broadcast::BroadcastSystem;

    pub mod mirror {
        use serde::{Deserialize, Serialize};

        use crate::broadcast::MessageId;
        use crate::broadcast::NodeId;
        use crate::broadcast::Payload;

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Message {
            pub id: MessageId,
            pub payload: Payload,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct GossipMessage {
            pub sender: NodeId,
            pub message: Message,
            pub round: u16,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct IhaveMessage {
            pub sender: NodeId,
            pub message_id: MessageId,
            pub round: u16,
            pub realtime: bool,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct GraftMessage {
            pub sender: NodeId,
            pub message_id: Option<MessageId>,
            pub round: u16,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct PruneMessage {
            pub sender: NodeId,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ProtocolMessage {
            Gossip(GossipMessage),
            Ihave(IhaveMessage),
            Graft(GraftMessage),
            Prune(PruneMessage),
        }
    }

    impl From<plumtree::message::Message<BroadcastSystem>> for mirror::Message {
        fn from(m: plumtree::message::Message<BroadcastSystem>) -> Self {
            Self {
                id: m.id,
                payload: m.payload,
            }
        }
    }

    impl From<mirror::Message> for plumtree::message::Message<BroadcastSystem> {
        fn from(m: mirror::Message) -> Self {
            Self {
                id: m.id,
                payload: m.payload,
            }
        }
    }

    impl From<plumtree::message::GossipMessage<BroadcastSystem>> for mirror::GossipMessage {
        fn from(m: plumtree::message::GossipMessage<BroadcastSystem>) -> Self {
            Self {
                sender: m.sender,
                message: m.message.into(),
                round: m.round,
            }
        }
    }

    impl From<mirror::GossipMessage> for plumtree::message::GossipMessage<BroadcastSystem> {
        fn from(m: mirror::GossipMessage) -> Self {
            Self {
                sender: m.sender,
                message: m.message.into(),
                round: m.round,
            }
        }
    }

    impl From<plumtree::message::IhaveMessage<BroadcastSystem>> for mirror::IhaveMessage {
        fn from(m: plumtree::message::IhaveMessage<BroadcastSystem>) -> Self {
            Self {
                sender: m.sender,
                message_id: m.message_id,
                round: m.round,
                realtime: m.realtime,
            }
        }
    }

    impl From<mirror::IhaveMessage> for plumtree::message::IhaveMessage<BroadcastSystem> {
        fn from(m: mirror::IhaveMessage) -> Self {
            Self {
                sender: m.sender,
                message_id: m.message_id,
                round: m.round,
                realtime: m.realtime,
            }
        }
    }

    impl From<plumtree::message::GraftMessage<BroadcastSystem>> for mirror::GraftMessage {
        fn from(m: plumtree::message::GraftMessage<BroadcastSystem>) -> Self {
            Self {
                sender: m.sender,
                message_id: m.message_id,
                round: m.round,
            }
        }
    }

    impl From<mirror::GraftMessage> for plumtree::message::GraftMessage<BroadcastSystem> {
        fn from(m: mirror::GraftMessage) -> Self {
            Self {
                sender: m.sender,
                message_id: m.message_id,
                round: m.round,
            }
        }
    }

    impl From<plumtree::message::PruneMessage<BroadcastSystem>> for mirror::PruneMessage {
        fn from(m: plumtree::message::PruneMessage<BroadcastSystem>) -> Self {
            Self { sender: m.sender }
        }
    }

    impl From<mirror::PruneMessage> for plumtree::message::PruneMessage<BroadcastSystem> {
        fn from(m: mirror::PruneMessage) -> Self {
            Self { sender: m.sender }
        }
    }

    impl From<plumtree::message::ProtocolMessage<BroadcastSystem>> for mirror::ProtocolMessage {
        fn from(m: plumtree::message::ProtocolMessage<BroadcastSystem>) -> Self {
            match m {
                plumtree::message::ProtocolMessage::Gossip(x) => Self::Gossip(x.into()),
                plumtree::message::ProtocolMessage::Ihave(x) => Self::Ihave(x.into()),
                plumtree::message::ProtocolMessage::Graft(x) => Self::Graft(x.into()),
                plumtree::message::ProtocolMessage::Prune(x) => Self::Prune(x.into()),
            }
        }
    }

    impl From<mirror::ProtocolMessage> for plumtree::message::ProtocolMessage<BroadcastSystem> {
        fn from(m: mirror::ProtocolMessage) -> Self {
            match m {
                mirror::ProtocolMessage::Gossip(x) => Self::Gossip(x.into()),
                mirror::ProtocolMessage::Ihave(x) => Self::Ihave(x.into()),
                mirror::ProtocolMessage::Graft(x) => Self::Graft(x.into()),
                mirror::ProtocolMessage::Prune(x) => Self::Prune(x.into()),
            }
        }
    }
}
