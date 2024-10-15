use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc::Receiver, Mutex};

use crate::{database::db::DB, types::ProofType, types::SupportedProvers};

use super::sp1::sp1_proof_verify;

pub struct Verifier {
    pub verifier_rx: Receiver<ProofType>,
    pub db: Arc<Mutex<DB>>,
}

impl Verifier {
    pub fn new(validator_rx: Receiver<ProofType>, db: Arc<Mutex<DB>>) -> Self {
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
                    match sp1_proof_verify(identifier.clone(), sp1_proof_with_public_values.clone())
                    {
                        Ok(_) => {
                            tracing::info!("Proof verified. proof_type=sp1 client={}", identifier);
                            let raw_string = sp1_proof_with_public_values.raw();

                            let locked_db = self.db.lock().await;
                            let _ = locked_db
                                .save_proof_to_db(
                                    identifier,
                                    SupportedProvers::SP1.to_string(),
                                    0u64,
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
                    tracing::info!("Running dummy prover");
                    let proof_string = hex::encode(&vec);

                    println!("Trying to get access to db: {}", proof_string);

                    let locked_db = self.db.lock().await;
                    println!("Got access to db");
                    let _ = locked_db
                        .save_proof_to_db(
                            identifier,
                            SupportedProvers::Dummy.to_string(),
                            101u64,
                            proof_string,
                        )
                        .await;
                    drop(locked_db)
                }
            }
        }
        Ok(())
    }
}
