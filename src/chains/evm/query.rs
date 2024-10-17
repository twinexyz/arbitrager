use crate::chains::chains::BalanceProvider;

use super::provider::EVMProvider;
use alloy_primitives::U256;
use alloy_provider::{Provider, WalletProvider};
use anyhow::{Error, Result};


impl BalanceProvider for EVMProvider {
    async fn query_balance(&self) -> Result<U256> {
        let poster = self.provider.default_signer_address();
        let balance = self.provider.get_balance(poster).await;
        match balance {
            Ok(b) => Ok(b),
            Err(e) => {
                Err(Error::new(e))
            }
        }
    }
}
