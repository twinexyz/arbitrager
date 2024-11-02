use std::collections::HashMap;

use alloy_primitives::U256;
use anyhow::Result;

use crate::{config::L1Details, types::PostParams};

use super::{
    dummy::DummyProvider,
    evm::provider::{EVMProvider, EVMProviderConfig},
    solana::provider::SolanaProvider,
};

pub trait BalanceProvider {
    fn query_balance(&self) -> impl std::future::Future<Output = Result<U256>> + Send;
}

pub trait ProofSubmitter {
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

impl ProofSubmitter for ChainProviders {
    async fn submit_proof(&self, params: PostParams) -> Result<()> {
        match self {
            ChainProviders::EVM(evmprovider) => evmprovider.submit_proof(params).await,
            ChainProviders::SVM(sp) => sp.submit_proof(params).await,
            ChainProviders::DummyVM(dp) => dp.submit_proof(params).await,
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
