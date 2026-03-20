use anyhow::Result;
use iroh::EndpointId;
use iroh::endpoint::Connection;
use iroh_tickets::endpoint::EndpointTicket;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio::time::interval;

use crate::Config;
use crate::api::{
    QueueGetRequest, QueueGetRequestParams, QueueGetResponseEnum, RegisterRequest,
    RegisterRequestParams,
};
use crate::utils::rpc::Handler;

mod discovery_module;

pub(crate) mod commands;
use commands::WorkerCommand;

pub struct Worker {
    commands_receiver: Receiver<WorkerCommand>,
    discovery_module: Option<discovery_module::DiscoveryModule>,
    connections: Arc<RwLock<HashMap<EndpointId, Connection>>>,
    pending_connections: Arc<RwLock<HashMap<EndpointId, discovery_module::PendingConnection>>>,
    pending_incoming_connections: HashMap<EndpointId, EndpointTicket>,
    config: Config,
}

impl Worker {
    pub async fn run(mut self) -> Result<()> {
        let mut periodic_timer = interval(self.periodic_interval());

        loop {
            let is_online = self.discovery_module.is_some();
            select! {
                Some(command) = self.commands_receiver.recv() => {
                    self.handle_command(command).await;
                },
                _ = periodic_timer.tick(), if is_online => {
                    self.poll_incoming_tickets().await;
                },
                else => break
            }
        }

        Ok(())
    }

    pub(crate) fn new(commands_receiver: Receiver<WorkerCommand>, config: Config) -> Self {
        Self {
            commands_receiver,
            discovery_module: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
            pending_connections: Arc::new(RwLock::new(HashMap::new())),
            pending_incoming_connections: HashMap::new(),
            config,
        }
    }

    async fn handle_command(&mut self, command: WorkerCommand) {
        match command {
            WorkerCommand::Connect(command) => {
                self.handle(command).await;
            }
            WorkerCommand::GoOnline(command) => {
                self.handle(command).await;
            }
            WorkerCommand::IsOnline(command) => {
                self.handle(command).await;
            }
            WorkerCommand::AddDiscoveryServer(command) => {
                self.handle(command).await;
            }
            WorkerCommand::RemoveDiscoveryServer(command) => {
                self.handle(command).await;
            }
            WorkerCommand::ListDiscoveryServers(command) => {
                self.handle(command).await;
            }
            WorkerCommand::ListIncomingConnections(command) => {
                self.handle(command).await;
            }
            WorkerCommand::AcceptConnection(command) => {
                self.handle(command).await;
            }
            WorkerCommand::AcceptStream(command) => {
                self.handle(command).await;
            }
            WorkerCommand::OpenStream(command) => {
                self.handle(command).await;
            }
        }
    }

    fn periodic_interval(&self) -> Duration {
        if self.config.poll_interval.is_zero() {
            Duration::from_millis(100)
        } else {
            self.config.poll_interval
        }
    }

    async fn poll_incoming_tickets(&mut self) {
        let Some(module) = self.discovery_module.as_mut() else {
            return;
        };

        let credentials = module.get_credentials();
        let mut raw_tickets: Vec<String> = Vec::new();

        for server in &mut module.discovery_servers {
            let request = QueueGetRequestParams {
                body: QueueGetRequest {
                    credentials: credentials.clone(),
                },
            };

            match server.client.queue_get(request).await {
                Ok(QueueGetResponseEnum::Ok(response)) => {
                    raw_tickets.extend(response.data);
                }
                Ok(QueueGetResponseEnum::NotFound) => {
                    let register_request = RegisterRequestParams {
                        body: RegisterRequest {
                            credentials: credentials.clone(),
                        },
                    };
                    let _ = server.client.register(register_request).await;
                }
                _ => {}
            }
        }

        for raw in raw_tickets {
            let Ok(ticket) = EndpointTicket::from_str(&raw) else {
                continue;
            };

            let peer_id = ticket.endpoint_addr().id;

            let connections = self.connections.read().unwrap();
            if connections.contains_key(&peer_id) {
                continue;
            }
            if module.has_pending_connection(peer_id) {
                continue;
            }

            self.pending_incoming_connections.insert(peer_id, ticket);
        }
    }
}
