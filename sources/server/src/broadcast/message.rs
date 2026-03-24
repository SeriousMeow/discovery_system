use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::broadcast::NodeId;

#[derive(From, Clone)]
pub enum Message {
    Membership(hyparview::message::ProtocolMessage<NodeId>),
}

#[derive(Serialize, Deserialize)]
enum MessageMirror {
    Membership(membership::mirror::ProtocolMessage),
}

impl From<Message> for MessageMirror {
    fn from(value: Message) -> Self {
        match value {
            Message::Membership(value) => Self::Membership(value.into()),
        }
    }
}

impl From<MessageMirror> for Message {
    fn from(value: MessageMirror) -> Self {
        match value {
            MessageMirror::Membership(value) => Self::Membership(value.into()),
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
