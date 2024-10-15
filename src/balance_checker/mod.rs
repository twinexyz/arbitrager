use std::{collections::HashMap, time::Duration};

use tokio::time::sleep;

use crate::types::ChainProviders;
use anyhow::Result;

pub struct BalanceChecker {
    pub providers: HashMap<String, ChainProviders>,
    pub threshold: u64,
}

impl BalanceChecker {
    pub fn new(providers: HashMap<String, ChainProviders>, threshold: u64) -> Self {
        Self {
            providers,
            threshold,
        }
    }

    /// This function is used to check the balances of poster wallet addresses
    /// This is periodically called every x minutes, and check if the balance is below threshold
    pub async fn run(&self) -> Result<()> {
        tracing::info!("Balance checker running");
        loop {
            // sleep for 10 minutes
            sleep(Duration::from_secs(self.threshold * 60)).await;
        }
    }
}
