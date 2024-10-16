use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc::Receiver;

use crate::{
    database::db::DB,
    types::{PostParams, ProofType, SupportedProvers},
    verifier::sp1::verify_sp1_proof,
};

pub trait ProofTraits {
    fn process_proof(proof: String, blocku64: u64) -> Option<PostParams>;
}

pub struct Verifier {
    pub verifier_rx: Receiver<ProofType>,
    pub db: Arc<DB>,
}

impl Verifier {
    pub fn new(validator_rx: Receiver<ProofType>, db: Arc<DB>) -> Self {
        Self {
            verifier_rx: validator_rx,
            db,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Verifier service running");
        while let Some(proof) = self.verifier_rx.recv().await {
            match proof {
                ProofType::SP1Proof(sp1_proof_with_public_values, identifier) => {
                    match verify_sp1_proof(sp1_proof_with_public_values.clone()) {
                        Ok(_) => {
                            tracing::info!("Proof verified. proof_type=sp1 client={}", identifier);
                            let raw_string =
                                serde_json::to_string(&sp1_proof_with_public_values).unwrap();

                            let _ = self
                                .db
                                .save_proof_to_db(
                                    identifier,
                                    SupportedProvers::SP1,
                                    365u64,
                                    raw_string,
                                )
                                .await;
                        }
                        Err(e) => {
                            tracing::error!(
                                "Proof not verified. proof_type=sp1 client={} error={}",
                                identifier,
                                e
                            );
                        }
                    }
                }
                ProofType::RISC0(vec, identifier) => {
                    tracing::error!("RISC0 not supported! client={}", identifier);
                }
                ProofType::Dummy(vec, identifier) => {
                    tracing::warn!("Running dummy prover");
                    let proof_string = hex::encode(&vec);
                    let _ = self
                        .db
                        .save_proof_to_db(identifier, SupportedProvers::Dummy, 33u64, proof_string)
                        .await;
                }
            }
        }
        Ok(())
    }
}
