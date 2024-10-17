use std::time::Duration;

use crate::{chains::chains::ProofSubmitter, types::PostParams};

use super::provider::EVMProvider;
use alloy::{
    network::TransactionBuilder,
    rpc::types::TransactionRequest,
    sol,
};
use alloy_provider::Provider;
use anyhow::Result;
use tokio::time::sleep;

sol! {
    #[sol(rpc)]
    contract SP1PlonkVerifier {
        constructor(address) {}

        #[derive(Debug)]
        function verifyProof(
            bytes32 programVKey,
            bytes calldata publicValues,
            bytes calldata proofBytes
        ) external;
    }
}

impl ProofSubmitter for EVMProvider {
    async fn submit_proof(&self, params: PostParams) -> Result<()> {
        match params {
            PostParams::RiscZero(evm_risc0_params, block) => todo!(),
            PostParams::Sp1(sp1_params, block) => {
                let contract =
                    SP1PlonkVerifier::new(self.config.contract_address, self.provider.clone());

                let vk = sp1_params.vk;
                let public_values = sp1_params.public_values;

                let plonk_proof = sp1_params.plonk_proof;

                let tx_data = contract.verifyProof(vk, public_values.clone(), plonk_proof.clone());

                let tx_req = tx_data
                    .max_fee_per_gas(200000000000000)
                    .max_priority_fee_per_gas(2000000)
                    .into_transaction_request();

                match self.send_transaction(tx_req).await {
                    Ok(_) => {
                        tracing::info!("Posted sp1 proof for block:{}", block);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to post proof for block:{} error: {:?}",
                            block,
                            e.to_string()
                        );
                    }
                }
            }
            PostParams::Dummy(dummy_params, _) => todo!(),
        }
        Ok(())
    }
}

impl EVMProvider {
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
