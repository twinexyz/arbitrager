use anyhow::Result;
use jsonrpsee::{server::Server, RpcModule};
use serde::{Deserialize, Serialize};
use sp1_sdk::SP1ProofWithPublicValues;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Sender;

use crate::{error::ArbitragerError, json_rpc_server::ServerReturnType, types::ProofType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ProofTypes {
    // TODO: ProofType for RISC0
    RISC0Proof {
        proof: Vec<u8>,
        identifier: String,
    },
    SP1Proof {
        proof: SP1ProofWithPublicValues,
        identifier: String,
    },
    Dummy {
        proof: Vec<u8>,
        identifier: String,
    },
}

// pub trait TwineArbApi<T> {
//     async fn submit_proof(&self, proof: ProofTypes) -> RpcResult<T>;
//     async fn get_proof(&self, block: u64) -> RpcResult<ProofTypes>;
//     fn health_check(&self) -> RpcResult<T>;
//     fn l1_proof_submitted(&self, block: u64) -> RpcResult<T>;
// }

#[derive(Clone)]
pub struct JsonRpcServer {
    // valid_senders: Arc<HashMap<SocketAddr, String>>,
    valid_senders: Arc<HashMap<String, String>>,
    verifier_tx: Sender<ProofType>,
}

impl JsonRpcServer {
    pub fn new(addresses: HashMap<String, String>, verifier_tx: Sender<ProofType>) -> Self {
        Self {
            valid_senders: Arc::new(addresses),
            verifier_tx,
        }
    }

    pub async fn run_server(self, port: u16) -> Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        tracing::info!("JSON RPC server running at {}", addr);
        let server = Server::builder().build(addr).await?;
        let mut module = RpcModule::new(());

        let server_handle = self.clone();

        module
            .register_async_method("twarb_sendProof", move |params, _ctx, _| {
                let server_handle = server_handle.clone();
                async move {
                    let proof: ProofTypes = match params.one() {
                        Ok(p) => p,
                        Err(e) => {
                            return ServerReturnType::Failure(format!(
                                "Failed deserializing proof: {e:?}"
                            ));
                        }
                    };
                    match proof {
                        ProofTypes::RISC0Proof { .. } => {
                            tracing::info!("RISC0 Proof not supported at the moment");
                            ServerReturnType::Failure("Not supported".to_string())
                        }
                        ProofTypes::SP1Proof { proof, identifier } => {
                            match server_handle.handle_sp1_proof(proof, identifier).await {
                                Ok(_) => ServerReturnType::Success,
                                Err(e) => {
                                    let error_msg = e.to_string();
                                    ServerReturnType::Failure(error_msg)
                                }
                            }
                        }
                        ProofTypes::Dummy { proof, identifier } => {
                            match server_handle.handle_dummy_proof(proof, identifier).await {
                                Ok(_) => ServerReturnType::Success,
                                Err(e) => {
                                    let error_msg = e.to_string();
                                    ServerReturnType::Failure(error_msg)
                                }
                            }
                        }
                    }
                }
            })
            .unwrap();

        module.register_method("twarb_healthCheck", |params, _, _| {
            let msg: String = params.one().unwrap();
            format!("Status: 1 Msg: {}", msg)
        })?;

        let handle = server.start(module);
        handle.stopped().await;

        Ok(())
    }

    async fn handle_sp1_proof(
        &self,
        proof: SP1ProofWithPublicValues,
        identifier: String,
    ) -> Result<()> {

        if !self.valid_senders.contains_key(&identifier) {
            tracing::error!("Invalid sender. Identifier:{}", identifier);
            return Err(ArbitragerError::InvalidSender(identifier).into());
        }

        self.verifier_tx
            .send(ProofType::SP1Proof(proof, identifier))
            .await?;
        Ok(())
    }

    async fn handle_dummy_proof(&self, proof: Vec<u8>, identifier: String) -> Result<()> {
        self.verifier_tx
            .send(ProofType::Dummy(proof, identifier))
            .await?;
        Ok(())
    }
}
