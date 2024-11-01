use crate::chains::chains::BalanceProvider;

use super::provider::EVMProvider;
use alloy_primitives::U256;
use alloy_provider::{Provider, WalletProvider};
use anyhow::Result;

impl BalanceProvider for EVMProvider {
    async fn query_balance(&self) -> Result<U256> {
        let poster = self.provider.default_signer_address();
        let balance = self.provider.get_balance(poster).await?;
        Ok(balance)
    }

    async fn balance_under_threshold(&self, threshold: U256) -> Result<(bool, U256)> {
        let balance = self.query_balance().await?;
        Ok((balance.gt(&threshold), balance))
    }
}
