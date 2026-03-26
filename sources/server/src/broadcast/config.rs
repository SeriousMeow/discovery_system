use smart_default::SmartDefault;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::time::Duration;

pub const DEFAULT_BUFFER_SIZE: usize = 1024;

#[derive(SmartDefault)]
pub struct Config {
    #[default(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080)))]
    pub listening_address: SocketAddr,
    #[default(DEFAULT_BUFFER_SIZE)]
    pub broadcast_buffer_size: usize,
    #[default(DEFAULT_BUFFER_SIZE)]
    pub new_messages_buffer_size: usize,
    #[default(DEFAULT_BUFFER_SIZE)]
    pub internal_messages_buffer_size: usize,
    #[default(DEFAULT_BUFFER_SIZE)]
    pub max_stored_messages: usize,
    #[default(None)]
    pub membership_options: Option<hyparview::NodeOptions>,
    #[default(None)]
    pub broadcast_option: Option<plumtree::NodeOptions>,
    pub membership_intervals: MembershipIntervals,
    pub broadcast_intervals: BroadcastIntervals,
}

#[derive(SmartDefault)]
pub struct MembershipIntervals {
    pub shuffle_passive: Duration,
    pub fill_active: Duration,
    pub sync_active: Duration,
    pub poll: Duration,
    pub cleanup: Duration,
}

#[derive(SmartDefault)]
pub struct BroadcastIntervals {
    pub poll: Duration,
    pub tick: Duration,
}
