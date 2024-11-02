use crate::chains::chains::{BalanceProvider, ProofSubmitter};
use alloy_primitives::U256;
use anyhow::Result;

#[derive(Clone)]
pub struct SolanaProvider {}

impl BalanceProvider for SolanaProvider {
    async fn query_balance(&self) -> Result<U256> {
        todo!()
    }
}

impl ProofSubmitter for SolanaProvider {
    async fn submit_proof(&self, params: crate::types::PostParams) -> Result<()> {
        todo!()
    }
}
