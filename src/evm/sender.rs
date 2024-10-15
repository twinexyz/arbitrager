use std::time::Duration;

use crate::types::PostParams;

use super::provider::EVMProvider;
use alloy::rpc::types::TransactionRequest;
use alloy_provider::Provider;
use anyhow::Result;
use tokio::time::sleep;


impl EVMProvider {
    pub fn make_transaction_request(&self, params: PostParams) -> TransactionRequest {
        match  params {
            PostParams::RiscZero(evm_risc0_params) => todo!(),
            PostParams::Sp1(evm_sp1_params) => todo!(),
            PostParams::Dummy(evm_dummy_params) => todo!(),
        }

        
        TransactionRequest::default()
    }

    /// This function keeps on running unless the transaction is successful.
    /// If failed, it tries to do the transaction again, waiting for 10 seconds. This runs in infinite loop.
    pub async fn send_transaction(
        &self,
        transaction: TransactionRequest,
    ) -> Result<String> {
        loop {
            let pending_tx = self.provider
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
                    sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }
}
