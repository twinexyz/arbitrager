use std::{collections::HashMap, time::Duration};

use alloy_primitives::utils::{format_units, parse_units};
use tokio::time::sleep;

use crate::chains::chains::{BalanceProvider, ChainProviders};
use anyhow::Result;

pub struct BalanceChecker {
    pub providers: HashMap<String, ChainProviders>,
    pub balance_threshold: HashMap<String, String>,
    pub time_threshold: u64,
}

impl BalanceChecker {
    pub fn new(
        providers: HashMap<String, ChainProviders>,
        balance_threshold: HashMap<String, String>,
        threshold: u64,
    ) -> Self {
        Self {
            providers,
            balance_threshold,
            time_threshold: threshold,
        }
    }

    /// This function is used to check the balances of poster wallet addresses
    /// This is periodically called every x minutes, and check if the balance is below threshold
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Balance checker running");
        loop {
            for (chain, provider) in self.providers.clone() {
                match provider.query_balance().await {
                    Ok(balance) => {
                        let threshold_balance =
                            parse_units(self.balance_threshold.get(&chain).unwrap(), "ether")?;
                        if balance.lt(&threshold_balance.into()) {
                            let num: String = format_units(balance, "ether")?;
                            tracing::warn!(
                                "Balance under threshold! chain:{} balance:{} eth",
                                chain,
                                num
                            );
                            // webhook
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to query balance. chain:{} error:{}", chain, e);
                    }
                }
            }
            // sleep for 10 minutes
            sleep(Duration::from_secs(self.time_threshold * 60)).await;
        }
    }
}
