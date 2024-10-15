use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::types::{ChainProviders, PostParams};

pub struct PostStatus {
    pub chain: String,
    pub block: u64,
    pub posted: bool,
}

pub struct Poster {
    pub providers: HashMap<String, ChainProviders>,
    pub post_rx: Receiver<PostParams>,
    pub post_status_tx: Sender<PostStatus>,
}

impl Poster {
    pub fn new(
        l1s: HashMap<String, ChainProviders>,
        post_rx: Receiver<PostParams>,
        post_status_tx: Sender<PostStatus>,
    ) -> Self {
        Self {
            providers: l1s,
            post_rx,
            post_status_tx,
        }
    }

    /// The data field incoming in the channel expects all the required parameters to post to the contract
    /// For verifying proof, it'll just be the public inputs and proof
    /// TODO: Communicate with the contract team for the structure
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Prover service running");
        while let Some(data) = self.post_rx.recv().await {
            // mock post the data
            let post_status = PostStatus {
                chain: "ethereum".to_string(),
                block: 10u64,
                posted: false,
            };
            self.post_status_tx.send(post_status).await.unwrap();

            // let _ = self.providers.iter().map(|k| {
            //     let _chain = k.0;
            //     match k.1 {
            //         ChainProviders::EVM(evmprovider) => {
            //             // evmprovider.send_transaction(transaction)
            //             tracing::info!("Proof submitted. chain:{}", _chain)
            //         }
            //         ChainProviders::SVM() => todo!(),
            //     }
            // });
        }
        Ok(())
    }
}
