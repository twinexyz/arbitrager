use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

use crate::{
    chains::chains::{ChainProviders, ProofSubmitter},
    types::PostParams,
};

pub struct PostStatus {
    pub chain: String,
    pub block: u64,
    pub posted: bool,
}

pub struct Poster {
    pub providers: HashMap<String, ChainProviders>,
    pub post_status_tx: Sender<PostStatus>,
}

impl Poster {
    pub fn new(l1s: HashMap<String, ChainProviders>, post_status_tx: Sender<PostStatus>) -> Self {
        Self {
            providers: l1s,
            post_status_tx,
        }
    }

    /// The data field incoming in the channel expects all the required parameters to post to the contract
    /// For verifying proof, it'll just be the public inputs and proof
    /// TODO: Communicate with the contract team for the structure
    pub async fn run(&self, mut post_rx: Receiver<PostParams>) -> Result<()> {
        tracing::info!("Prover service running");
        while let Some(data) = post_rx.recv().await {
            // mock post the data

            for (chain, provider) in &self.providers {
                let data_clone = data.clone();
                let chain = chain.to_string();
                match provider.submit_proof(data_clone.clone()).await {
                    Ok(_) => {
                        tracing::info!("Proof submitted. chain:{}", chain);
                        let post_status = match data_clone {
                            PostParams::RiscZero(_risc0_params, _) => todo!(),
                            PostParams::Sp1(_sp1params, block) => PostStatus {
                                chain: chain.clone(),
                                block,
                                posted: true,
                            },
                            PostParams::Dummy(_, block) => PostStatus {
                                chain,
                                block,
                                posted: true,
                            },
                        };
                        info!("Post status received. Sending status to post_status channel");
                        self.post_status_tx.send(post_status).await.unwrap();
                    }
                    Err(_) => {
                        // handle proofs
                        tracing::error!("Fail to submit proof. chain:{}", chain);
                    }
                }
                // match provider {
                //     ChainProviders::EVM(evmprovider) => {
                //         match evmprovider.submit_proof(data_clone.clone()).await {
                //             Ok(_) => {
                //                 tracing::info!("Proof submitted. chain:{}", chain);
                //                 let post_status = match data_clone {
                //                     PostParams::RiscZero(_risc0_params, _) => todo!(),
                //                     PostParams::Sp1(_sp1params, block) => PostStatus {
                //                         chain: chain.clone().to_string(),
                //                         block,
                //                         posted: true,
                //                     },
                //                     PostParams::Dummy(_dummy_params, _) => todo!(),
                //                 };
                //                 info!("Post status received. Sending status to post_status channel");
                //                 self.post_status_tx.send(post_status).await.unwrap();
                //             }
                //             Err(_) => {
                //                 tracing::error!("Fail to submit proof. chain:{}", chain);
                //             }
                //         }
                //     }
                //     ChainProviders::SVM() => {
                //         tracing::info!("SVM provider is not yet implemented. chain:{}", chain);
                //         todo!();
                //     }
                //     ChainProviders::DummyVM() => {
                //         tracing::info!("Proof submitted to dummy verifier. chain:{}", chain);
                //         let data_clone = data.clone();
                //         let post_status = match data_clone {
                //             PostParams::RiscZero(risc0_params, _) => todo!(),
                //             PostParams::Sp1(sp1params, _) => todo!(),
                //             PostParams::Dummy(dummy_params, block) => PostStatus {
                //                 chain: chain.to_string(),
                //                 block,
                //                 posted: true,
                //             },
                //         };

                //         self.post_status_tx.send(post_status).await.unwrap();
                //     }
                // }
            }
        }
        Ok(())
    }
}
