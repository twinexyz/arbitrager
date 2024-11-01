use crate::chains::chains::{BalanceProvider, ProofSubmitter};
use alloy_primitives::U256;
use anyhow::Result;

#[derive(Clone)]
pub struct DummyProvider {}

impl BalanceProvider for DummyProvider {
    async fn query_balance(&self) -> Result<U256> {
        todo!()
    }

    async fn balance_under_threshold(&self, threshold: U256) -> Result<(bool, U256)> {
        todo!()
    }
}

impl ProofSubmitter for DummyProvider {
    async fn submit_proof(&self, params: crate::types::PostParams) -> Result<()> {
        todo!()
    }
}
