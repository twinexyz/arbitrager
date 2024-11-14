use crate::chains::chains::{BalanceProvider, L1Transactions};
use alloy_primitives::U256;
use anyhow::Result;

use super::{chains::FetchL2TransactionData, evm::sender::TwineChain::CommitBatchInfo};

#[derive(Clone)]
pub struct DummyProvider {}

impl BalanceProvider for DummyProvider {
    async fn query_balance(&self) -> Result<U256> {
        todo!()
    }

    async fn balance_under_threshold(&self, threshold: U256) -> Result<(bool, String)> {
        let _ = threshold;
        todo!()
    }
}

impl L1Transactions for DummyProvider {
    async fn submit_proof(&self, params: crate::types::PostParams) -> Result<()> {
        let _ = params;
        todo!()
    }

    async fn commit_batch(&self, params: CommitBatchInfo, height: u64) -> Result<()> {
        let _ = params;
        let _ = height;
        todo!()
    }
}

impl FetchL2TransactionData for DummyProvider {
    async fn fetch_commit_batch(&self, height: u64) -> Result<CommitBatchInfo> {
        let _ = height;
        todo!()
    }
}
