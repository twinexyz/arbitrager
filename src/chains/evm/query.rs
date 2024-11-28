use std::collections::HashMap;

use crate::{
    chains::chains::{BalanceProvider, FetchL2TransactionData},
    error::AggregatorError,
    MAX_RETRIES,
};

use super::{
    provider::EVMProvider,
    sender::TwineChain::{self, CommitBatchInfo},
};
use alloy::{
    eips::BlockNumberOrTag,
    rpc::types::{Block, Filter},
    sol,
    sol_types::SolEvent,
};
use alloy_primitives::{utils::format_units, FixedBytes, U256};
use alloy_provider::{Provider, WalletProvider};
use anyhow::Result;

impl BalanceProvider for EVMProvider {
    async fn query_balance(&self) -> Result<U256> {
        let poster = self.provider.default_signer_address();
        let balance = self.provider.get_balance(poster).await?;
        Ok(balance)
    }

    async fn balance_under_threshold(&self, threshold: U256) -> Result<(bool, String)> {
        let balance = self.query_balance().await?;
        let eth_balance = format_units(balance, "eth")?;
        Ok((balance.lt(&threshold), format!("{eth_balance} eth")))
    }
}

sol! {
    #[sol(rpc)]
    contract L2Messenger {

        #[derive(Debug)]
        event L1TokenDeposit();

        #[derive(Debug)]
        event ForcedWithdrawal(
            address indexed from,
            address to,
            address counterpartGateway,
            address counterpartMessenger,
            uint256 value,
            uint256 indexed chainId,
            uint256 blockNumber,
            uint256 gasLimit,
            bytes message
        );

        #[derive(Debug)]
        event LayerzeroPayload(
            uint256 indexed sourceChainId,
            bytes32 indexed guId
        );
    }
}

impl FetchL2TransactionData for EVMProvider {
    async fn fetch_commit_batch(&self, height: u64) -> Result<CommitBatchInfo> {
        let mut attempt = 0;

        loop {
            match self
                .provider
                .get_block_by_number(BlockNumberOrTag::Number(height), true)
                .await
            {
                Ok(block) => {
                    if let Some(blk) = block {
                        let prev_state_root = match self
                            .provider
                            .get_block_by_number(
                                BlockNumberOrTag::Number(blk.header.number - 1),
                                true,
                            )
                            .await
                        {
                            Ok(xx) => xx.unwrap().header.state_root,
                            Err(_) => {
                                attempt += 1;
                                continue;
                            }
                        };
                        match self.filter_l2_transactions(blk.header.hash).await {
                            Ok(filter) => {
                                return Ok(EVMProvider::generate_commit_params(
                                    blk,
                                    filter,
                                    prev_state_root,
                                ))
                            }
                            Err(e) => {
                                attempt += 1;
                                tracing::warn!("Failed to get logs: {}", e.to_string());
                            }
                        }
                    } else {
                        attempt += 1;
                        tracing::warn!("No result in block");
                    }
                }
                Err(e) => {
                    attempt += 1;
                    if attempt > MAX_RETRIES {
                        tracing::error!("Failed to query after {MAX_RETRIES} attempts");
                        return Err(AggregatorError::Custom(
                            "Failed to fetch commit batch params".to_string(),
                        )
                        .into());
                    }
                    tracing::warn!("Failed to query block: {}", e.to_string());
                }
            }
        }
    }
}

impl EVMProvider {
    pub async fn filter_l2_transactions(
        &self,
        block_hash: FixedBytes<32>,
    ) -> Result<HashMap<u64, L2TxType>> {
        let mut tx_types = HashMap::<u64, L2TxType>::new();

        let filter = Filter::new()
            .events(["L1TokenDeposit()", "ForcedWithdrawal(address,address,address,address,uint256,uint256,uint256,uint256,bytes)","LayerzeroPayload(uint256,bytes32)"])
            .at_block_hash(block_hash)
            .address(self.config.contract_address);

        let logs = self.provider.get_logs(&filter).await?;

        for l in logs {
            match l.topic0() {
                Some(&L2Messenger::L1TokenDeposit::SIGNATURE_HASH) => {
                    if let Some(idx) = l.transaction_index {
                        tx_types.insert(idx, L2TxType::Deposit);
                    }
                }
                Some(&L2Messenger::ForcedWithdrawal::SIGNATURE_HASH) => {
                    if let Some(idx) = l.transaction_index {
                        tx_types.insert(idx, L2TxType::Forced);
                    }
                }
                Some(&L2Messenger::LayerzeroPayload::SIGNATURE_HASH) => {
                    if let Some(idx) = l.transaction_index {
                        tx_types.insert(idx, L2TxType::LZ);
                    }
                }
                _ => (),
            }
        }

        Ok(tx_types)
    }

    pub fn generate_commit_params(
        block: Block,
        filter: HashMap<u64, L2TxType>,
        prev_state_root: FixedBytes<32>,
    ) -> TwineChain::CommitBatchInfo {
        let mut deposit_txns_object = Vec::new();
        let mut withdraw_txns_object = Vec::new();
        let mut lz_txns_object= Vec::new();
        let mut normal_txns_object = Vec::new();

        for txn in block.transactions.into_transactions() {
            let signature = if let Some(sig) = txn.signature {
                sig
            } else {
                continue;
            };

            let l1_txn = TwineChain::TransactionObject {
                // for contract creation, to address can be none
                to: txn.to.unwrap_or_default(),
                nonce: U256::from(txn.nonce),
                value: txn.value,
                maxFeePerGas: U256::from(txn.max_fee_per_gas.unwrap_or(0)),
                maxPriorityFeePerGas: U256::from(txn.max_priority_fee_per_gas.unwrap_or(0)),
                v: signature.v.to(),
                r: uint256_to_bytes32(signature.r),
                s: uint256_to_bytes32(signature.s),
                gas: U256::from(txn.gas),
                input: txn.input,
                chainId: U256::from(txn.chain_id.unwrap()),
                accesslist: Vec::new(),
            };

            // when is index optional in txn ?
            let index = txn.transaction_index.unwrap();
            match filter.get(&index) {
                Some(tx_type) => match tx_type {
                    L2TxType::Deposit => deposit_txns_object.push(l1_txn),
                    L2TxType::Forced => withdraw_txns_object.push(l1_txn),
                    L2TxType::LZ => lz_txns_object.push(l1_txn),
                    L2TxType::Normal => panic!("should not arrive here"),
                },
                None => {
                    normal_txns_object.push(l1_txn);
                }
            }
        }

        TwineChain::CommitBatchInfo {
            batchNumber: block.header.number,
            batchHash: block.header.hash,
            stateRoot: block.header.state_root,
            previousStateRoot: prev_state_root,
            transactionRoot: block.header.transactions_root,
            receiptRoot: block.header.receipts_root,
            otherTransactions: normal_txns_object,
            depositTransactionObject: deposit_txns_object,
            forcedTransactionObjects: withdraw_txns_object,
            lzDvnTransactions: lz_txns_object,
        }
    }
}

#[derive(Clone)]
pub enum L2TxType {
    Deposit,
    Forced,
    Normal,
    LZ,
}

fn uint256_to_bytes32(val: U256) -> FixedBytes<32> {
    let k: [u8; 32] = val.to_be_bytes();
    let r: FixedBytes<32> = FixedBytes::from_slice(&k);
    r
}
