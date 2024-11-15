use std::time::Duration;

use crate::{chains::chains::L1Transactions, types::PostParams};

use super::provider::EVMProvider;
use alloy::{rpc::types::TransactionRequest, sol};

use alloy_primitives::U256;
use alloy_provider::Provider;
use anyhow::Result;
use tokio::time::sleep;
use TwineChain::CommitBatchInfo;

static MAX_RETRIES: i32 = 10;

sol! {
    #[sol(rpc)]
    contract TwineChain {

        #[derive(Debug)]
        struct TransactionObject {
            uint256 chainId;
            uint256 nonce;
            uint256 maxPriorityFeePerGas;
            uint256 maxFeePerGas;
            uint256 gas;
            address to;
            uint256 value;
            bytes input;
            AccessList[] accesslist;
            uint64 v;
            bytes32 r;
            bytes32 s;
        }

        #[derive(Debug)]
        struct AccessList {
            address _address;
            bytes32[] storageKeys;
        }


        #[derive(Debug)]
        struct CommitBatchInfo{
            uint64 batchNumber;
            bytes32 batchHash;
            bytes32 previousStateRoot;
            bytes32 stateRoot;
            bytes32 transactionRoot;
            bytes32 receiptRoot;
            TransactionObject[] depositTransactionObject;
            TransactionObject[] forcedTransactionObjects;
            TransactionObject[] otherTransactions;
        }

        #[derive(Debug)]
        function commitBatch(CommitBatchInfo calldata _newBatchData) external;

        #[derive(Debug)]
        function finalizeBatch(uint256 batchNumber, bytes calldata _proofBytes) external;
    }
}

impl L1Transactions for EVMProvider {
    async fn commit_batch(&self, params: CommitBatchInfo, height: u64) -> Result<()> {
        tracing::info!("Commit batch for batch: {}", params.batchNumber);
        let batch = params.batchNumber;
        let contract = TwineChain::new(self.config.contract_address, self.provider.clone());
        let tx_data = contract.commitBatch(params);

        let tx_req = tx_data
            .max_fee_per_gas(20000000000)
            .max_priority_fee_per_gas(2000000)
            .into_transaction_request();

        match self.send_transaction(tx_req).await {
            Ok(_) => {
                tracing::info!("Commited batch for height: {} batch: {}", height, batch);
            }
            Err(e) => {
                tracing::error!(
                    "Failed to commit for block: {} batch: {} error: {:?}",
                    height,
                    batch,
                    e.to_string()
                );
                return Err(e);
            }
        }
        Ok(())
    }

    async fn submit_proof(&self, params: PostParams) -> Result<()> {
        tracing::info!("Submitting proof for batch: {}", params.height());
        match params {
            PostParams::RiscZero(_evm_risc0_params, _block) => todo!(),
            PostParams::Sp1(sp1_params, block) => {
                let contract = TwineChain::new(self.config.contract_address, self.provider.clone());

                let plonk_proof = sp1_params.plonk_proof;
                let block = U256::from(block);

                let tx_data = contract.finalizeBatch(block, plonk_proof.clone());

                let tx_req = tx_data
                    .max_fee_per_gas(20000000000)
                    .max_priority_fee_per_gas(2000000)
                    .into_transaction_request();

                // handle nonce error, gas error correctly
                match self.send_transaction(tx_req).await {
                    Ok(_) => {
                        tracing::info!("Posted sp1 proof for batch:{}", block);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to post proof for batch:{} error: {:?}",
                            block,
                            e.to_string()
                        );
                        return Err(e);
                    }
                }
            }
            PostParams::Dummy(_dummy_params, _) => {
                tracing::warn!("Dummy chain: Mock txn submission successful");
            }
        }
        Ok(())
    }
}

impl EVMProvider {
    /// This function keeps on running unless the transaction is successful.
    /// If failed, it tries to do the transaction again, waiting for 15 seconds. This runs in infinite loop.
    pub async fn send_transaction(&self, transaction: TransactionRequest) -> Result<String> {
        let mut attempt = 0;
        loop {
            let pending_tx = self.provider.send_transaction(transaction.clone()).await?;

            tracing::debug!("Pending transaction hash: {}", pending_tx.tx_hash());
            match pending_tx.get_receipt().await {
                Ok(receipt) => {
                    let txn_hash = receipt.transaction_hash.to_string();
                    tracing::info!("Transaction Submitted! txn_hash: {}", txn_hash);
                    return Ok(txn_hash);
                }
                Err(e) => {
                    attempt += 1;
                    if attempt >= MAX_RETRIES {
                        tracing::error!(
                            "Transaction failed after {} attempts. Error: {}",
                            attempt,
                            e
                        );
                        return Err(e.into());
                    }

                    tracing::error!(
                        "Transaction Failed! Error: {}. Retrying ({}/{}) in 15 seconds.",
                        e,
                        attempt,
                        MAX_RETRIES
                    );
                    sleep(Duration::from_secs(15)).await;
                }
            }
        }
    }
}
