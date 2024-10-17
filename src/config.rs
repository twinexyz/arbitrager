use anyhow::Error;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

use crate::types::SupportedProvers;
use crate::utils::check_directory_exists;
use crate::utils::is_valid_url;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub elf: HashMap<String, String>,
    pub provers: HashMap<String, ProverDetails>,
    pub l1s: HashMap<String, L1Details>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub logging: String,
    pub server_port: u16,
    pub threshold: usize,
    pub db_path: String,
    pub balance_check_interval: u64, // in minutes
}

#[derive(Debug, Deserialize)]
pub struct ProverDetails {
    pub prover_ip: String,
    pub prover_type: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum L1Details {
    Solana(SolanaConfig),
    EVM(EVMConfig),
}

impl L1Details {
    pub fn get_balance_threshold(&self) -> String {
        match self {
            L1Details::Solana(solana_config) => solana_config.balance_threshold.clone(),
            L1Details::EVM(evmconfig) => evmconfig.balance_threshold.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SolanaConfig {
    pub contract: String,
    pub rpc: String,
    pub balance_threshold: String,
    pub solana_key_path: String,
    pub solana_password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EVMConfig {
    pub contract: String,
    pub balance_threshold: String,
    pub rpc: String,
    pub private_key: String,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        // Validate threshold
        let number_of_provers = self.provers.len();
        if self.global.threshold < 1 {
            return Err(Error::msg("threshold must be greater than 0".to_string()));
        }
        if self.global.threshold > number_of_provers {
            return Err(Error::msg(format!(
                "threshold must be less than or equal to the number of provers ({}), found: {}",
                number_of_provers, self.global.threshold
            )));
        }

        // Ensure ELF File exists
        for (_, v) in &self.elf {
            if !v.is_empty() && !check_directory_exists(v) {
                return Err(Error::msg(
                    format!("{} elf file does not exist", v).to_string(),
                ));
            }
        }

        for (_, value) in &self.provers {
            if !is_valid_url(&value.prover_ip) {
                return Err(Error::msg("prover grpc_server must be valid url"));
            }

            match SupportedProvers::from_str(&value.prover_type) {
                Ok(_) => {}
                Err(_) => return Err(Error::msg("prover type not supported")),
            }
        }

        for (key, details) in &self.l1s {
            match &details {
                L1Details::Solana(solana_config) => {
                    // required solana validation

                    if !is_valid_url(&solana_config.rpc) {
                        return Err(Error::msg(format!(
                            "Invalid l1_rpc URL for {}: {}",
                            key, solana_config.rpc
                        )));
                    }
                }
                L1Details::EVM(evmconfig) => {
                    // required evm validation

                    if !is_valid_url(&evmconfig.rpc) {
                        return Err(Error::msg(format!(
                            "Invalid l1_rpc URL for {}: {}",
                            key, evmconfig.rpc
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}
