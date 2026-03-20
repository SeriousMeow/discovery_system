use crate::api::Credentials;
use crate::api::DiscoverySystemClient;
use anyhow::Result;
use iroh::endpoint::{Connection, ConnectionError, VarInt};
use iroh::protocol::{ProtocolHandler, Router};
use iroh::{Endpoint, EndpointId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use tokio::sync::oneshot;

pub const ALPN: &[u8] = b"discovery_system/1";

pub struct DiscoveryServer {
    pub url: String,
    pub client: DiscoverySystemClient,
}

pub struct PendingConnection {
    pub callback: oneshot::Sender<Result<Connection, ConnectionError>>,
}

pub struct DiscoveryModule {
    pub endpoint: Endpoint,
    pub discovery_servers: Vec<DiscoveryServer>,
    pub pending_connections: Arc<RwLock<HashMap<EndpointId, PendingConnection>>>,
    _router: Router,
}

impl DiscoveryModule {
    pub async fn new(
        pending_connections: Arc<RwLock<HashMap<EndpointId, PendingConnection>>>,
        connections: Arc<RwLock<HashMap<EndpointId, Connection>>>,
    ) -> Result<Self> {
        let endpoint = Endpoint::bind().await?;
        let handler = Handler::new(pending_connections.clone(), connections);

        let router = Router::builder(endpoint.clone())
            .accept(ALPN, handler)
            .spawn();

        Ok(Self {
            endpoint: endpoint,
            discovery_servers: Vec::new(),
            pending_connections,
            _router: router,
        })
    }

    pub fn add_pending_connection(
        &self,
        id: EndpointId,
        callback: oneshot::Sender<Result<Connection, ConnectionError>>,
    ) {
        let mut guard = self.pending_connections.write().unwrap();
        let pending_connections = &mut *guard;
        pending_connections.insert(id, PendingConnection { callback });
    }

    pub fn has_pending_connection(&self, id: EndpointId) -> bool {
        let guard = self.pending_connections.read().unwrap();
        guard.contains_key(&id)
    }

    pub fn add_server(&mut self, url: String) -> Result<()> {
        let client = DiscoverySystemClient::with_base_url(&url)?;
        self.discovery_servers.push(DiscoveryServer { url, client });
        Ok(())
    }

    pub fn remove_server(&mut self, url: &str) {
        self.discovery_servers.retain(|server| server.url != url);
    }

    pub fn list_servers(&self) -> Vec<String> {
        self.discovery_servers
            .iter()
            .map(|server| server.url.clone())
            .collect()
    }

    pub fn get_credentials(&self) -> Credentials {
        Credentials {
            id: self.endpoint.id().to_string(),
        }
    }
}

struct Handler {
    pending_connections: Arc<RwLock<HashMap<EndpointId, PendingConnection>>>,
    connections: Arc<RwLock<HashMap<EndpointId, Connection>>>,
}

impl fmt::Debug for Handler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handler").finish()
    }
}

impl Handler {
    fn new(
        pending_connections: Arc<RwLock<HashMap<EndpointId, PendingConnection>>>,
        connections: Arc<RwLock<HashMap<EndpointId, Connection>>>,
    ) -> Self {
        Self {
            pending_connections,
            connections,
        }
    }
}

impl ProtocolHandler for Handler {
    async fn accept(&self, connection: Connection) -> Result<(), iroh::protocol::AcceptError> {
        let peer_id = connection.remote_id();

        let already_connected = {
            let guard = self.connections.read().unwrap();
            guard.contains_key(&peer_id)
        };

        if already_connected {
            connection.close(VarInt::from_u32(400), b"Already connected");
            return Ok(());
        }

        let pending = match self.pending_connections.write().unwrap().remove(&peer_id) {
            Some(pending) => pending,
            None => {
                connection.close(VarInt::from_u32(400), b"Not expected");
                return Ok(());
            }
        };

        let _ = pending.callback.send(Ok(connection.clone()));

        self.connections
            .write()
            .unwrap()
            .insert(peer_id, connection);
        Ok(())
    }
}
