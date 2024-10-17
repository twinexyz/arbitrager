use std::str::FromStr;

use alloy::{
    network::EthereumWallet,
    signers::local::PrivateKeySigner,
    transports::http::{Client, Http},
};
use alloy_primitives::Address;
use alloy_provider::WalletProvider;
use alloy_provider::{fillers::FillProvider, ProviderBuilder, RootProvider};
use anyhow::{Context, Error};

#[derive(Debug, Clone)]
pub struct EVMProviderConfig {
    pub rpc_url: String,
    pub private_key: String,
    pub contract_address: Address,
}

#[derive(Clone)]
pub struct EVMProvider {
    pub config: EVMProviderConfig,
    pub provider: AlloyProvider,
}

pub type AlloyProvider = FillProvider<
    alloy_provider::fillers::JoinFill<
        alloy_provider::fillers::JoinFill<
            alloy_provider::Identity,
            alloy_provider::fillers::JoinFill<
                alloy_provider::fillers::GasFiller,
                alloy_provider::fillers::JoinFill<
                    alloy_provider::fillers::BlobGasFiller,
                    alloy_provider::fillers::JoinFill<
                        alloy_provider::fillers::NonceFiller,
                        alloy_provider::fillers::ChainIdFiller,
                    >,
                >,
            >,
        >,
        alloy_provider::fillers::WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    alloy::network::Ethereum,
>;


impl EVMProviderConfig {
    pub fn new(rpc_url: String, private_key: String, contract_address: String) -> Self {
        let contract = Address::from_str(contract_address.trim_start_matches("0x"))
            .expect("Invalid contract address");

        Self {
            rpc_url,
            private_key,
            contract_address: contract,
        }
    }
}

impl EVMProvider {
    pub fn new(config: EVMProviderConfig) -> Self {
        let provider = config.build().expect("Failed to build EVM Provider");
        Self { config, provider }
    }

    pub fn address(&self) -> String {
        let addr = self.provider.default_signer_address();
        addr.to_string()
    }
}

impl EVMProviderConfig {
    pub fn build(&self) -> Result<AlloyProvider, Error> {
        let signer: PrivateKeySigner = self
            .private_key
            .trim_start_matches("0x")
            .parse()
            .with_context(|| "Error parsing private key")?;
        let wallet = EthereumWallet::from(signer);

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(
                self.rpc_url
                    .parse()
                    .with_context(|| "Error parsing RPC URL")?,
            );

        Ok(provider)
    }
}
