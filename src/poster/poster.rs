use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

use crate::{
    chains::{
        chains::{ChainProviders, FetchL2TransactionData, L1Transactions},
        evm::provider::EVMProvider,
    },
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
    pub l2_provider: EVMProvider,
}

impl Poster {
    pub fn new(
        l1s: HashMap<String, ChainProviders>,
        post_status_tx: Sender<PostStatus>,
        l2_provider: EVMProvider,
    ) -> Self {
        Self {
            providers: l1s,
            post_status_tx,
            l2_provider,
        }
    }

    /// The data field incoming in the channel expects all the required parameters to post to the contract
    /// For verifying proof, it'll just be the public inputs and proof
    /// TODO: Communicate with the contract team for the structure
    pub async fn run(&mut self, mut post_rx: Receiver<PostParams>) -> Result<()> {
        tracing::info!("Prover service running");
        while let Some(data) = post_rx.recv().await {
            tracing::info!("Ready for commit batch and finalize batch");
            // handle batch number here
            let l2_height = data.height();
            let commit_batch_info = self.l2_provider.fetch_commit_batch(l2_height).await?;
            let providers = self.providers.clone();
            let mut tasks = vec![];

            for (chain, provider) in providers {
                let chain = chain.to_string();
                let batch = commit_batch_info.clone();
                let batch_number = batch.batchNumber;
                let data_clone = data.clone();
                let post_status_tx = self.post_status_tx.clone();

                let task = tokio::spawn(async move {
                    match provider.commit_batch(batch, l2_height).await {
                        Ok(_) => {
                            tracing::info!(
                                "Batch committed! batch: {} chain: {}",
                                batch_number,
                                chain
                            );
                        }
                        Err(_) => {
                            // notify
                            tracing::error!(
                                "Failed posting batch: {} chain: {}",
                                batch_number,
                                chain
                            );
                        }
                    }

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
                                    chain: chain.clone(),
                                    block,
                                    posted: true,
                                },
                            };
                            info!("Post status received. Sending status to post_status channel");
                            if let Err(e) = post_status_tx.send(post_status).await {
                                tracing::error!(
                                    "Failed to send post status. chain: {} error: {}",
                                    chain,
                                    e.to_string()
                                );
                            }
                        }
                        Err(_) => {
                            // notify
                            tracing::error!("Fail to submit proof. chain:{}", chain);
                        }
                    }
                });
                tasks.push(task);
            }

            // Wait for all spawned tasks to complete
            for task in tasks {
                if let Err(err) = task.await {
                    tracing::error!("Task failed with error: {:?}", err);
                }
            }
        }
        Ok(())
    }
}
