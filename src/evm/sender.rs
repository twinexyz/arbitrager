use std::time::Duration;

use crate::types::PostParams;

use super::provider::EVMProvider;
use alloy::{rpc::types::TransactionRequest, sol};
use alloy_provider::Provider;
use anyhow::Result;
use tokio::time::sleep;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SP1PlonkVerifier,
    "src/evm/SP1VerifierPlonk.json"
);

impl EVMProvider {
    pub async fn post_to_evm(&self, params: PostParams) -> Result<()> {
        match params {
            PostParams::RiscZero(evm_risc0_params, block) => todo!(),
            PostParams::Sp1(evm_sp1_params, block) => {
                let contract =
                    SP1PlonkVerifier::new(self.config.contract_address, self.provider.clone());

                let vk = evm_sp1_params.vk;
                let public_values = evm_sp1_params.public_values;
                let plonk_proof = evm_sp1_params.plonk_proof;

                let res = contract
                    .verifyProof(vk, public_values, plonk_proof)
                    .call()
                    .await;
                match res {
                    Ok(_) => {
                        tracing::info!("Posted sp1 proof for block:{}", block);
                    }
                    Err(e) => tracing::error!(
                        "Failed to post proof for block:{} error: {:?}",
                        block,
                        e.to_string()
                    ),
                }
            }
            PostParams::Dummy(evm_dummy_params, block) => {
                tracing::info!("Posting dummy proof for block:{}", block);
            }
        }
        Ok(())
    }

    pub fn make_transaction_request(&self, params: PostParams) -> TransactionRequest {
        TransactionRequest::default()
    }

    /// This function keeps on running unless the transaction is successful.
    /// If failed, it tries to do the transaction again, waiting for 15 seconds. This runs in infinite loop.
    pub async fn send_transaction(&self, transaction: TransactionRequest) -> Result<String> {
        loop {
            let pending_tx = self
                .provider
                .send_transaction(transaction.clone())
                .await
                .unwrap();
            tracing::debug!("Pending transaction hash: {}", pending_tx.tx_hash());
            match pending_tx.get_receipt().await {
                Ok(receipt) => {
                    let txn_hash = receipt.transaction_hash.to_string();
                    tracing::info!("Transaction Submitted! txn_hash: {}", txn_hash);
                    return Ok(txn_hash);
                }
                Err(receipt) => {
                    tracing::info!("Transaction Failed! error: {} Retrying!", receipt);
                    sleep(Duration::from_secs(15)).await;
                }
            }
        }
    }
}
