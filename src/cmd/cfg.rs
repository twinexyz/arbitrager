use crate::{
    arbitrager::run,
    chains::{
        chains::ProofSubmitter,
        evm::provider::{EVMProvider, EVMProviderConfig},
    },
    config::Config,
    types::SupportedProvers,
    verifier::{dummy::Dummy, risc0::RISC0, sp1::SP1, verifier::ProofTraits},
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{
    env::home_dir,
    fs::File,
    io::Read,
    path::PathBuf,
    process::{self},
};

use super::logger::logging;

const DEFAULT_CONFIG_DIR: &str = "./twine/arbitrager/config.yaml";

#[derive(Parser, Debug)]
#[command(name = "twarb")]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run {
        #[arg(short, long, default_value = default_config_path().into_os_string())]
        config: PathBuf,
    },
    Show {
        #[arg(short, long, default_value = default_config_path().into_os_string())]
        config: PathBuf,
    },
    ManualRelay {
        #[arg(short, long, default_value = default_config_path().into_os_string())]
        config: PathBuf,

        #[arg(short, long)]
        height: u64,

        #[arg(short, long)]
        chain: String,

        #[arg(short, long)]
        proof_type: String,

        #[arg(short, long)]
        proof_json: PathBuf,
    },
}

fn default_config_path() -> PathBuf {
    home_dir()
        .map(|dir| dir.join(DEFAULT_CONFIG_DIR))
        .expect("Failed to get home directory")
}

fn load_config(config_path: PathBuf) -> Result<String> {
    let mut file = File::open(config_path).expect("Failed to open config");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Ok(contents)
}

pub async fn init() {
    let cli = Args::parse();
    match &cli.command {
        Commands::Run { config } => {
            match load_config(config.to_path_buf()) {
                Ok(config) => {
                    // parse config
                    let cfg: Config = serde_yaml::from_str(&config).expect("Failed to parse yaml");
                    // validate config
                    match cfg.validate() {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Config Validation Failed. Error: {}", e);
                            process::exit(1);
                        }
                    }
                    // logging setup
                    logging(&cfg.global.logging);

                    match run(cfg).await {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Error running arbitrager: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load config: {}", e)
                }
            }
        }
        Commands::Show { config } => todo!(),
        Commands::ManualRelay {
            config,
            height,
            chain,
            proof_type,
            proof_json,
        } => manual_proof_relay(config, height, chain, proof_type, proof_json).await,
    }
}

pub async fn manual_proof_relay(
    config: &PathBuf,
    height: &u64,
    chain: &String,
    proof_type: &String,
    proof_json: &PathBuf,
) {
    match load_config(config.to_path_buf()) {
        Ok(config) => {
            // parse config
            let cfg: Config = serde_yaml::from_str(&config).expect("Failed to parse yaml");
            // validate config
            match cfg.validate() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Config Validation Failed. Error: {}", e);
                    process::exit(1);
                }
            }
            // logging setup
            logging(&cfg.global.logging);

            let l2_chains = cfg.l1s;
            let proof_string =
                std::fs::read_to_string(proof_json).expect("Failed to read proof file as string");
            let destination = l2_chains.get(chain).expect("Invalid chain name");
            match destination {
                crate::config::L1Details::Solana(solana_config) => todo!(),
                crate::config::L1Details::EVM(evmconfig) => {
                    let provider_config = EVMProviderConfig::new(
                        evmconfig.rpc.clone(),
                        evmconfig.private_key.clone(),
                        evmconfig.contract.clone(),
                    );
                    let provider = EVMProvider::new(provider_config);
                    let post_params = match SupportedProvers::from_str(&proof_type)
                        .expect("Invalid proof type")
                    {
                        SupportedProvers::SP1 => SP1::process_proof(proof_string, height.clone()),
                        SupportedProvers::RISC0 => {
                            RISC0::process_proof(proof_string, height.clone())
                        }
                        SupportedProvers::Dummy => {
                            Dummy::process_proof(proof_string, height.clone())
                        }
                    }
                    .expect("Failed to construct proof params");

                    match provider.submit_proof(post_params).await {
                        Ok(_) => {
                            println!("Transaction successful ");
                        }
                        Err(e) => {
                            println!("Transaction failed! {e:?}");
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load config: {}", e)
        }
    }
}
