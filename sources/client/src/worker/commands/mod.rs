use derive_more::From;

pub mod accept_connection;
pub mod accept_stream;
pub mod connect;
pub mod discovery_identity;
pub mod generate_static_puzzle_solution;
pub mod go_online;
pub mod is_online;
pub mod list_connections;
pub mod list_incoming_connections;
pub mod open_stream;
pub mod self_id;
pub mod set_static_puzzle_solution;

#[derive(From)]
pub enum WorkerCommand {
    Connect(connect::Command),
    GoOnline(go_online::Command),
    GenerateStaticPuzzleSolution(generate_static_puzzle_solution::Command),
    SetStaticPuzzleSolution(set_static_puzzle_solution::Command),
    IsOnline(is_online::Command),
    DiscoveryIdentity(discovery_identity::Command),
    SelfId(self_id::Command),
    ListConnections(list_connections::Command),
    ListIncomingConnections(list_incoming_connections::Command),
    AcceptConnection(accept_connection::Command),
    AcceptStream(accept_stream::Command),
    OpenStream(open_stream::Command),
}
