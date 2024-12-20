use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    database::db::DB,
    types::{PostParams, ProofType, SupportedProvers},
};

use super::sp1::SP1;

pub trait ProofTraits {
    fn process_proof(proof: String, blocku64: u64) -> Result<PostParams>;
    fn public_values(proof_json: &PathBuf) -> Result<String>;
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

    pub async fn run(&mut self, sp1: SP1, poster_tx: Sender<PostParams>) -> Result<()> {
        tracing::info!("Verifier service running");
        while let Some(proof) = self.verifier_rx.recv().await {
            let poster_tx = poster_tx.clone();
            match proof {
                ProofType::SP1Proof(sp1_proof_with_public_values, identifier) => {
                    match sp1.verify_sp1_proof(sp1_proof_with_public_values.clone()) {
                        Ok(height) => {
                            tracing::info!("Proof verified. proof_type=sp1 client={}", identifier);
                            let raw_string = serde_json::to_string(&sp1_proof_with_public_values)?;

                            let _ = self
                                .db
                                .save_proof_to_db(
                                    identifier,
                                    SupportedProvers::SP1,
                                    height,
                                    raw_string,
                                    poster_tx,
                                )
                                .await
                                .map_err(|e| {
                                    tracing::error!("Error saving proof to db {:?}", e.to_string());
                                });
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
                ProofType::RISC0(_vec, identifier) => {
                    tracing::error!("RISC0 not supported! client={}", identifier);
                }
                ProofType::Dummy(vec, identifier) => {
                    tracing::warn!("Running dummy prover");
                    let proof_string = hex::encode(&vec);
                    let _ = self
                        .db
                        .save_proof_to_db(
                            identifier,
                            SupportedProvers::Dummy,
                            vec[1] as u64,
                            proof_string,
                            poster_tx,
                        )
                        .await;
                }
            }
        }
        Ok(())
    }
}
