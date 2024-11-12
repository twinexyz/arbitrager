use crate::chains::{
    chains::{BalanceProvider, FetchL2TransactionData, L1Transactions},
    evm::sender::TwineChain::CommitBatchInfo,
};
use alloy_primitives::U256;
use anyhow::Result;

#[derive(Clone)]
pub struct SolanaProvider {}

impl BalanceProvider for SolanaProvider {
    async fn query_balance(&self) -> Result<U256> {
        todo!()
    }

    async fn balance_under_threshold(&self, threshold: U256) -> Result<(bool, String)> {
        todo!()
    }
}

impl L1Transactions for SolanaProvider {
    async fn submit_proof(&self, params: crate::types::PostParams) -> Result<()> {
        todo!()
    }

    async fn commit_batch(&self, params: CommitBatchInfo, height: u64) -> Result<()> {
        todo!()
    }
}

impl FetchL2TransactionData for SolanaProvider {
    async fn fetch_commit_batch(&self, height: u64) -> Result<CommitBatchInfo> {
        todo!()
    }
}
