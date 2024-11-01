use anyhow::Result;
use jsonrpsee::{server::Server, RpcModule};
use serde::{Deserialize, Serialize};
use sp1_sdk::SP1ProofWithPublicValues;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Sender;

use crate::types::ProofType;

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
                    let proof: ProofTypes = params.one().unwrap();
                    match proof {
                        ProofTypes::RISC0Proof { ..} => {
                            panic!("Unimplemented!")
                        }
                        ProofTypes::SP1Proof { proof , identifier} => {
                            server_handle.handle_sp1_proof(proof, identifier).await;
                        }
                        ProofTypes::Dummy { proof , identifier} => {
                            server_handle.handle_dummy_proof(proof, identifier).await;
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

    async fn handle_sp1_proof(&self, proof: SP1ProofWithPublicValues, identifier: String)  {
        if !self.valid_senders.contains_key(&identifier) {
            tracing::error!("Invalid sender. Identifier:{}", identifier);
            return;
        }

        self.verifier_tx
            .send(ProofType::SP1Proof(proof, identifier))
            .await
            .expect("Failed sending sp1 proof through mpsc");
    }

    async fn handle_dummy_proof(&self, proof: Vec<u8>, identifier: String) {
        self.verifier_tx
            .send(ProofType::Dummy(proof, identifier))
            .await
            .expect("Failed sending sp1 proof through mpsc");
    }
}

/*
{
  "jsonrpc": "2.0",
  "method": "twarb_sendProof",
  "params": [
    {
      "type": "RISC0Proof",
      "identifier": "prover-1",
      "proof":{
        "entire-proof-json"
      }
    }
  ],
  "id": 1
}
*/
