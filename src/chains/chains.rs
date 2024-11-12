use std::collections::HashMap;

use alloy_primitives::U256;
use anyhow::Result;

use crate::{
    config::{L1Details, L2Details},
    types::PostParams,
};

use super::{
    dummy::DummyProvider,
    evm::{
        provider::{EVMProvider, EVMProviderConfig},
        sender::TwineChain::CommitBatchInfo,
    },
    solana::provider::SolanaProvider,
};

pub trait BalanceProvider {
    fn query_balance(&self) -> impl std::future::Future<Output = Result<U256>> + Send;

    /// return units as well
    /// "example: Ok(true,123.00eth)"
    fn balance_under_threshold(
        &self,
        threshold: U256,
    ) -> impl std::future::Future<Output = Result<(bool, String)>> + Send;
}

pub trait FetchL2TransactionData {
    fn fetch_commit_batch(
        &self,
        height: u64,
    ) -> impl std::future::Future<Output = Result<CommitBatchInfo>> + Send;
}

pub trait L1Transactions {
    fn commit_batch(
        &self,
        params: CommitBatchInfo,
        height: u64,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn submit_proof(
        &self,
        params: PostParams,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

#[derive(Clone)]
pub enum ChainProviders {
    EVM(EVMProvider),
    SVM(SolanaProvider),
    DummyVM(DummyProvider),
}

impl L1Transactions for ChainProviders {
    async fn submit_proof(&self, params: PostParams) -> Result<()> {
        match self {
            ChainProviders::EVM(evmprovider) => evmprovider.submit_proof(params).await,
            ChainProviders::SVM(sp) => sp.submit_proof(params).await,
            ChainProviders::DummyVM(dp) => dp.submit_proof(params).await,
        }
    }

    async fn commit_batch(&self, params: CommitBatchInfo, height: u64) -> Result<()> {
        match self {
            ChainProviders::EVM(evmprovider) => evmprovider.commit_batch(params, height).await,
            ChainProviders::SVM(sp) => sp.commit_batch(params, height).await,
            ChainProviders::DummyVM(dp) => dp.commit_batch(params, height).await,
        }
    }
}

impl FetchL2TransactionData for ChainProviders {
    async fn fetch_commit_batch(&self, height: u64) -> Result<CommitBatchInfo> {
        match self {
            ChainProviders::EVM(evmprovider) => evmprovider.fetch_commit_batch(height).await,
            ChainProviders::SVM(solana_provider) => {
                solana_provider.fetch_commit_batch(height).await
            }
            ChainProviders::DummyVM(dummy_provider) => {
                dummy_provider.fetch_commit_batch(height).await
            }
        }
    }
}

impl BalanceProvider for ChainProviders {
    async fn query_balance(&self) -> Result<U256> {
        match self {
            ChainProviders::EVM(evm_provider) => evm_provider.query_balance().await,
            ChainProviders::SVM(solana_provider) => solana_provider.query_balance().await,
            ChainProviders::DummyVM(dp) => dp.query_balance().await,
        }
    }

    async fn balance_under_threshold(&self, threshold: U256) -> Result<(bool, String)> {
        match self {
            ChainProviders::EVM(evmprovider) => {
                evmprovider.balance_under_threshold(threshold).await
            }
            ChainProviders::SVM(solana_provider) => {
                solana_provider.balance_under_threshold(threshold).await
            }
            ChainProviders::DummyVM(dummy_provider) => {
                dummy_provider.balance_under_threshold(threshold).await
            }
        }
    }
}

/// Makes chain providers from data in the config
pub fn make_providers(l1s: HashMap<String, L1Details>) -> HashMap<String, ChainProviders> {
    l1s.iter()
        .map(|(key, detail)| {
            let provider = match detail {
                L1Details::Solana(solana_config) => todo!(),
                L1Details::EVM(evmconfig) => {
                    let evm_config = EVMProviderConfig::new(
                        evmconfig.rpc.clone(),
                        evmconfig.private_key.clone(),
                        evmconfig.contract.clone(),
                    );

                    evm_config
                        .build()
                        .map(|provider| {
                            ChainProviders::EVM(EVMProvider {
                                provider,
                                config: evm_config,
                            })
                        })
                        .unwrap_or_else(|_| panic!("Failed building provider!"))
                }
            };
            (key.to_string(), provider)
        })
        .collect()
}

pub fn make_l2_provider(l2: L2Details) -> EVMProvider {
    // We won't use this key for anythng
    let dummy_private_key = "2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";
    let evm_cfg =
        EVMProviderConfig::new(l2.rpc, dummy_private_key.to_string(), l2.messenger_contract);

    EVMProvider::new(evm_cfg)
}
