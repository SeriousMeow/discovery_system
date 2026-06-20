use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use thiserror::Error;

use crate::utils::rpc;
use crate::worker::discovery_module::DiscoveryModule;
use pow_puzzle::{PublicKey, StaticPuzzleSolution, generate_static_solution, verify_static};

#[derive(Error, Debug)]
pub enum Error {
    #[error("already online")]
    AlreadyOnline,
    #[error("invalid discovery server url")]
    InvalidUrl(#[from] anyhow::Error),
    #[error("discovery rejected credentials (static puzzle or identity)")]
    ForbiddenCredentials,
    #[error("invalid discovery public key: {0}")]
    InvalidDiscoveryPublicKey(String),
    #[error(transparent)]
    Puzzle(#[from] pow_puzzle::PuzzleError),
}

pub type Command = rpc::Command<Request, Response>;

pub struct Request {
    pub server_url: String,
    pub discovery_public_key: Option<String>,
}

pub type Response = Result<(), Error>;

impl rpc::Handler<Request, Response> for crate::worker::Worker {
    async fn handle(&mut self, command: Command) {
        if self.discovery_module.is_some() {
            let _ = command.response_tx.send(Err(Error::AlreadyOnline));
            return;
        }

        let server_url = command.request.server_url;
        let min_bits = self.config.puzzle_min_difficulty_bits;

        let explicit = command
            .request
            .discovery_public_key
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        let solution: StaticPuzzleSolution = if let Some(s) = explicit {
            let pk = match parse_discovery_public_key(s) {
                Ok(pk) => pk,
                Err(msg) => {
                    let _ = command
                        .response_tx
                        .send(Err(Error::InvalidDiscoveryPublicKey(msg)));
                    return;
                }
            };
            if let Err(e) = verify_static(&pk, min_bits) {
                let _ = command.response_tx.send(Err(Error::Puzzle(e)));
                return;
            }
            let sol = StaticPuzzleSolution(pk.0);
            self.pending_static_solution = Some(sol);
            sol
        } else {
            match self.pending_static_solution {
                Some(s) => s,
                None => match generate_static_solution(min_bits) {
                    Ok((s, _)) => {
                        self.pending_static_solution = Some(s);
                        s
                    }
                    Err(e) => {
                        let _ = command.response_tx.send(Err(Error::Puzzle(e)));
                        return;
                    }
                },
            }
        };

        let module = match DiscoveryModule::new(
            self.pending_connections.clone(),
            self.connections.clone(),
            server_url,
            solution,
        )
        .await
        {
            Ok(m) => m,
            Err(e) => {
                let _ = command.response_tx.send(Err(Error::from(e)));
                return;
            }
        };

        let register_request = crate::api::RegisterRequestParams {
            body: crate::api::RegisterRequest {
                credentials: module.get_credentials(),
            },
        };

        match module.discovery_client.register(register_request).await {
            Ok(crate::api::RegisterResponseEnum::Ok(_)) => {}
            Ok(crate::api::RegisterResponseEnum::Forbidden) => {
                let _ = command.response_tx.send(Err(Error::ForbiddenCredentials));
                return;
            }
            Ok(crate::api::RegisterResponseEnum::Unauthorized) => {
                let _ = command.response_tx.send(Err(Error::ForbiddenCredentials));
                return;
            }
            Ok(crate::api::RegisterResponseEnum::Unknown) => {
                let _ = command.response_tx.send(Err(Error::ForbiddenCredentials));
                return;
            }
            Err(e) => {
                let _ = command.response_tx.send(Err(Error::InvalidUrl(e)));
                return;
            }
        }

        self.discovery_module = Some(module);
        let _ = command.response_tx.send(Ok(()));
    }
}

fn parse_discovery_public_key(s: &str) -> Result<PublicKey, String> {
    let raw = B64
        .decode(s.as_bytes())
        .map_err(|e| format!("base64 decode: {e}"))?;
    PublicKey::try_from(raw.as_slice()).map_err(|_| "base64 must decode to exactly 32 bytes".into())
}
