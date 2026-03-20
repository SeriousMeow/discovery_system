use derive_more::From;

pub mod accept_connection;
pub mod accept_stream;
pub mod add_discovery_server;
pub mod connect;
pub mod go_online;
pub mod is_online;
pub mod list_discovery_servers;
pub mod list_incoming_connections;
pub mod open_stream;
pub mod remove_discovery_server;

#[derive(From)]
pub enum WorkerCommand {
    Connect(connect::Command),
    GoOnline(go_online::Command),
    IsOnline(is_online::Command),
    AddDiscoveryServer(add_discovery_server::Command),
    RemoveDiscoveryServer(remove_discovery_server::Command),
    ListDiscoveryServers(list_discovery_servers::Command),
    ListIncomingConnections(list_incoming_connections::Command),
    AcceptConnection(accept_connection::Command),
    AcceptStream(accept_stream::Command),
    OpenStream(open_stream::Command),
}
