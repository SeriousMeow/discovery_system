use derive_more::From;

pub mod accept_connection;
pub mod accept_stream;
pub mod connect;
pub mod go_online;
pub mod is_online;
pub mod list_connections;
pub mod list_incoming_connections;
pub mod open_stream;
pub mod self_id;

#[derive(From)]
pub enum WorkerCommand {
    Connect(connect::Command),
    GoOnline(go_online::Command),
    IsOnline(is_online::Command),
    SelfId(self_id::Command),
    ListConnections(list_connections::Command),
    ListIncomingConnections(list_incoming_connections::Command),
    AcceptConnection(accept_connection::Command),
    AcceptStream(accept_stream::Command),
    OpenStream(open_stream::Command),
}
