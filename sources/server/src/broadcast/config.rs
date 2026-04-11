use smart_default::SmartDefault;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::Path;
use tokio::time::Duration;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use validator::{Validate, ValidationError};

pub const DEFAULT_BUFFER_SIZE: usize = 1024;

pub const ENV_PREFIX: &str = "DISCOVERY_SERVER_";
pub const ENV_LISTEN_ADDR: &str = "DISCOVERY_SERVER_LISTEN_ADDR";
pub const ENV_HTTP_ADDR: &str = "DISCOVERY_SERVER_HTTP_ADDR";
pub const ENV_CONTACT_NODE: &str = "DISCOVERY_SERVER_CONTACT_NODE";
pub const ENV_CONFIG_JSON: &str = "DISCOVERY_SERVER_CONFIG_JSON";
pub const ENV_STATIC_PUZZLE_DIFFICULTY_BITS: &str =
    "DISCOVERY_SERVER_STATIC_PUZZLE_DIFFICULTY_BITS";

#[derive(SmartDefault, Validate)]
pub struct Config {
    /// Minimum leading zero bits required for `SHA256(SHA256(public_key))` at registration.
    #[default(8)]
    #[validate(range(min = 0, max = 256))]
    pub static_puzzle_difficulty_bits: u32,
    #[default(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080)))]
    pub listening_address: SocketAddr,
    #[default(DEFAULT_BUFFER_SIZE)]
    #[validate(range(min = 1))]
    pub broadcast_buffer_size: usize,
    #[default(DEFAULT_BUFFER_SIZE)]
    #[validate(range(min = 1))]
    pub new_messages_buffer_size: usize,
    #[default(DEFAULT_BUFFER_SIZE)]
    #[validate(range(min = 1))]
    pub internal_messages_buffer_size: usize,
    #[default(DEFAULT_BUFFER_SIZE)]
    #[validate(range(min = 1))]
    pub max_stored_messages: usize,
    #[default(None)]
    pub membership_options: Option<hyparview::NodeOptions>,
    #[default(None)]
    pub broadcast_option: Option<plumtree::NodeOptions>,
    #[validate(nested)]
    pub membership_intervals: MembershipIntervals,
    #[validate(nested)]
    pub broadcast_intervals: BroadcastIntervals,
}

#[derive(SmartDefault, Clone, Debug, PartialEq, Eq)]
pub struct Addresses {
    #[default(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080)))]
    pub http_addr: SocketAddr,
    #[default(None)]
    pub contact_node: Option<IpAddr>,
    #[default(IpAddr::V4(Ipv4Addr::LOCALHOST))]
    pub local_node_id: IpAddr,
}

#[derive(SmartDefault, Validate)]
pub struct MembershipIntervals {
    #[validate(custom(function = "validate_duration_gt_zero"))]
    pub shuffle_passive: Duration,
    #[validate(custom(function = "validate_duration_gt_zero"))]
    pub fill_active: Duration,
    #[validate(custom(function = "validate_duration_gt_zero"))]
    pub sync_active: Duration,
    #[validate(custom(function = "validate_duration_gt_zero"))]
    pub poll: Duration,
    #[validate(custom(function = "validate_duration_gt_zero"))]
    pub cleanup: Duration,
}

#[derive(SmartDefault, Validate)]
pub struct BroadcastIntervals {
    #[validate(custom(function = "validate_duration_gt_zero"))]
    pub poll: Duration,
    #[validate(custom(function = "validate_duration_gt_zero"))]
    pub tick: Duration,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigJson {
    static_puzzle_difficulty_bits: Option<u32>,
    broadcast_buffer_size: Option<usize>,
    new_messages_buffer_size: Option<usize>,
    internal_messages_buffer_size: Option<usize>,
    max_stored_messages: Option<usize>,
    membership_intervals: Option<MembershipIntervalsJson>,
    broadcast_intervals: Option<BroadcastIntervalsJson>,
    membership_options: Option<HyparviewNodeOptionsMirror>,
    broadcast_option: Option<PlumtreeNodeOptionsMirror>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MembershipIntervalsJson {
    shuffle_passive_ms: u64,
    fill_active_ms: u64,
    sync_active_ms: u64,
    poll_ms: u64,
    cleanup_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BroadcastIntervalsJson {
    poll_ms: u64,
    tick_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct HyparviewNodeOptionsMirror {
    max_active_view_size: u8,
    max_passive_view_size: u8,
    shuffle_active_view_size: u8,
    shuffle_passive_view_size: u8,
    active_random_walk_len: u8,
    passive_random_walk_len: u8,
}

impl From<HyparviewNodeOptionsMirror> for hyparview::NodeOptions {
    fn from(value: HyparviewNodeOptionsMirror) -> Self {
        Self {
            max_active_view_size: value.max_active_view_size,
            max_passive_view_size: value.max_passive_view_size,
            shuffle_active_view_size: value.shuffle_active_view_size,
            shuffle_passive_view_size: value.shuffle_passive_view_size,
            active_random_walk_len: value.active_random_walk_len,
            passive_random_walk_len: value.passive_random_walk_len,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlumtreeNodeOptionsMirror {
    ihave_timeout_ms: u64,
    optimization_threshold: u16,
}

impl From<PlumtreeNodeOptionsMirror> for plumtree::NodeOptions {
    fn from(value: PlumtreeNodeOptionsMirror) -> Self {
        Self {
            ihave_timeout: Duration::from_millis(value.ihave_timeout_ms),
            optimization_threshold: value.optimization_threshold,
        }
    }
}

impl Config {
    pub fn from_sources() -> Result<(Self, Addresses)> {
        let mut cfg = Self::default();
        let mut addrs = Addresses::default();

        if let Ok(raw) = std::env::var(ENV_STATIC_PUZZLE_DIFFICULTY_BITS) {
            let bits: u32 = raw.parse().with_context(|| {
                format!(
                    "invalid {ENV_STATIC_PUZZLE_DIFFICULTY_BITS} (expected integer), got {raw:?}"
                )
            })?;
            cfg.static_puzzle_difficulty_bits = bits;
        }

        if let Some(path) = std::env::var_os(ENV_CONFIG_JSON) {
            let path = Path::new(&path);
            let json = std::fs::read_to_string(path).with_context(|| {
                format!("failed to read JSON config file at {}", path.display())
            })?;
            let parsed: ConfigJson = serde_json::from_str(&json).with_context(|| {
                format!(
                    "failed to parse JSON config at {} (must contain non-address fields only)",
                    path.display()
                )
            })?;
            cfg.apply_json(parsed)?;
        }

        apply_addr_env(ENV_LISTEN_ADDR, |a| cfg.listening_address = a)?;
        apply_addr_env(ENV_HTTP_ADDR, |a| addrs.http_addr = a)?;
        apply_contact_env(ENV_CONTACT_NODE, |ip| addrs.contact_node = Some(ip))?;

        addrs.local_node_id = resolve_local_node_id(cfg.listening_address)?;

        cfg.validate_config()?;

        Ok((cfg, addrs))
    }

    fn apply_json(&mut self, json: ConfigJson) -> Result<()> {
        if let Some(v) = json.static_puzzle_difficulty_bits {
            self.static_puzzle_difficulty_bits = v;
        }
        if let Some(v) = json.broadcast_buffer_size {
            self.broadcast_buffer_size = v;
        }
        if let Some(v) = json.new_messages_buffer_size {
            self.new_messages_buffer_size = v;
        }
        if let Some(v) = json.internal_messages_buffer_size {
            self.internal_messages_buffer_size = v;
        }
        if let Some(v) = json.max_stored_messages {
            self.max_stored_messages = v;
        }

        if let Some(v) = json.membership_options {
            self.membership_options = Some(v.into());
        }
        if let Some(v) = json.broadcast_option {
            self.broadcast_option = Some(v.into());
        }

        if let Some(ints) = json.membership_intervals {
            self.membership_intervals = MembershipIntervals {
                shuffle_passive: Duration::from_millis(ints.shuffle_passive_ms),
                fill_active: Duration::from_millis(ints.fill_active_ms),
                sync_active: Duration::from_millis(ints.sync_active_ms),
                poll: Duration::from_millis(ints.poll_ms),
                cleanup: Duration::from_millis(ints.cleanup_ms),
            };
        }

        if let Some(ints) = json.broadcast_intervals {
            self.broadcast_intervals = BroadcastIntervals {
                poll: Duration::from_millis(ints.poll_ms),
                tick: Duration::from_millis(ints.tick_ms),
            };
        }

        Ok(())
    }

    fn validate_config(&self) -> Result<()> {
        validator::Validate::validate(self)
            .map_err(|error| anyhow!("invalid server config: {error}"))?;
        Ok(())
    }
}

fn validate_duration_gt_zero(value: &Duration) -> std::result::Result<(), ValidationError> {
    if value.as_nanos() == 0 {
        return Err(ValidationError::new("duration_must_be_positive"));
    }
    Ok(())
}

fn apply_addr_env(var: &str, mut set: impl FnMut(SocketAddr)) -> Result<()> {
    let Ok(raw) = std::env::var(var) else {
        return Ok(());
    };
    let parsed: SocketAddr = raw
        .parse()
        .with_context(|| format!("invalid {var} (expected host:port), got {raw:?}"))?;
    set(parsed);
    Ok(())
}

fn parse_contact_node(raw: &str) -> Result<IpAddr> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(anyhow!("contact node address is empty"));
    }
    if let Ok(ip) = raw.parse::<IpAddr>() {
        return Ok(ip);
    }
    let sa: SocketAddr = raw
        .parse()
        .with_context(|| format!("invalid contact node (expected IP or host:port), got {raw:?}"))?;
    Ok(sa.ip())
}

fn apply_contact_env(var: &str, mut set: impl FnMut(IpAddr)) -> Result<()> {
    let Ok(raw) = std::env::var(var) else {
        return Ok(());
    };
    set(parse_contact_node(&raw).with_context(|| format!("invalid {var}"))?);
    Ok(())
}

fn resolve_local_node_id(listening_address: SocketAddr) -> Result<IpAddr> {
    let ip = listening_address.ip();
    if ip.is_unspecified() {
        return Err(anyhow!(
            "broadcast listen address {} has unspecified IP; use a concrete address in {} (e.g. 172.29.0.11:8081) so gossip has a stable node identity",
            listening_address,
            ENV_LISTEN_ADDR
        ));
    }
    Ok(ip)
}
