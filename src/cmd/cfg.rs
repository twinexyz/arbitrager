use crate::{
    arbitrager::run,
    chains::{
        chains::{make_l2_provider, FetchL2TransactionData, L1Transactions},
        evm::provider::{EVMProvider, EVMProviderConfig},
    },
    config::Config,
    database::db::DB,
    types::SupportedProvers,
    verifier::{dummy::Dummy, risc0::RISC0, sp1::SP1, verifier::ProofTraits},
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use home::home_dir;
use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    process::{self},
};

use super::logger::logging;

const DEFAULT_CONFIG_DIR: &str = ".twine/arbitrager/config.yaml";

#[derive(Parser, Debug)]
#[command(name = "twarb")]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[clap(long, short)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run,
    Show,
    DeleteDB,
    ManualRelay {
        #[arg(short, long)]
        height: u64,

        #[arg(short, long)]
        chain: String,

        #[arg(short, long)]
        proof_type: String,

        #[arg(short, long)]
        proof_json: PathBuf,
    },
    PublicValues {
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
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub async fn init() -> Result<()> {
    let cli = Args::parse();

    let path = if let Some(config_path) = cli.config {
        config_path
    } else {
        default_config_path()
    };

    let cfg;

    match load_and_validate_config(path) {
        Ok(config) => {
            cfg = config;
            // Set up logging
            logging(&cfg.global.logging);
            tracing::info!("Config ok!");
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            process::exit(1);
        }
    };

    match &cli.command {
        Commands::Run => handle_run_command(cfg).await,
        Commands::Show => handle_show_command(cfg),
        Commands::DeleteDB => delete_db(cfg).await,
        Commands::ManualRelay {
            height,
            chain,
            proof_type,
            proof_json,
        } => {
            manual_proof_relay(cfg, height, chain, proof_type, proof_json).await;
            Ok(())
        }
        Commands::PublicValues {
            proof_type,
            proof_json,
        } => print_public_values(proof_type, proof_json),
    }
}

fn print_public_values(proof_type: &String, proof_json: &PathBuf) -> Result<()> {
    let public_values = match SupportedProvers::from_str(proof_type).expect("Invalid proof type") {
        SupportedProvers::SP1 => SP1::public_values(proof_json),
        SupportedProvers::RISC0 => RISC0::public_values(proof_json),
        SupportedProvers::Dummy => RISC0::public_values(proof_json),
    }?;

    println!("Public values:  {public_values}");

    return Ok(());
}

// for manual relaying, so unwrap/expect is okay
pub async fn manual_proof_relay(
    cfg: Config,
    height: &u64,
    chain: &String,
    proof_type: &String,
    proof_json: &PathBuf,
) {
    let l1_chains = cfg.l1s;
    let proof_string =
        std::fs::read_to_string(proof_json).expect("Failed to read proof file as string");
    let destination = l1_chains.get(chain).expect("Invalid chain name");

    let l2_chain = cfg.l2;
    let l2_provider = make_l2_provider(l2_chain);
    match destination {
        crate::config::L1Details::Solana(_solana_config) => todo!(),
        crate::config::L1Details::EVM(evmconfig) => {
            let provider_config = EVMProviderConfig::new(
                evmconfig.rpc.clone(),
                evmconfig.private_key.clone(),
                evmconfig.contract.clone(),
            );
            let l1_provider = EVMProvider::new(provider_config);

            let commit_batch_info = l2_provider
                .fetch_commit_batch(*height)
                .await
                .expect("Failed to construct commit batch info");

            match l1_provider.commit_batch(commit_batch_info, *height).await {
                Ok(_) => {
                    println!("Commit batch successful");
                }
                Err(e) => {
                    eprintln!("{}", e);
                    panic!("Commit failed");
                }
            }

            let post_params =
                match SupportedProvers::from_str(proof_type).expect("Invalid proof type") {
                    SupportedProvers::SP1 => SP1::process_proof(proof_string, *height),
                    SupportedProvers::RISC0 => RISC0::process_proof(proof_string, *height),
                    SupportedProvers::Dummy => Dummy::process_proof(proof_string, *height),
                }
                .expect("Failed to construct proof params");

            match l1_provider.submit_proof(post_params).await {
                Ok(_) => {
                    println!("Proof submission successful ");
                }
                Err(e) => {
                    println!("Transaction failed! {e:?}");
                }
            }
        }
    }
}

async fn handle_run_command(cfg: Config) -> Result<()> {
    // Run the main process
    let _ = run(cfg).await.map_err(|e| {
        tracing::error!("Error running arbitrager: {}", e);
        process::exit(1);
    });

    Ok(())
}

async fn delete_db(cfg: Config) -> Result<()> {
    let db = DB::new(cfg.global.threshold, cfg.global.db_path).await;
    let _ = db
        .delete_db()
        .await
        .map_err(|e| tracing::error!("Failed to delete DB: {:?}", e));
    Ok(())
}

fn handle_show_command(cfg: Config) -> Result<()> {
    let pretty_printed = serde_json::to_string_pretty(&cfg).expect("Failed to serialize config");
    println!("{}", pretty_printed);
    Ok(())
}

fn load_and_validate_config(config_path: PathBuf) -> Result<Config> {
    let config_content = load_config(config_path)?;

    let cfg: Config = serde_yaml::from_str(&config_content)?;

    cfg.validate()?;

    Ok(cfg)
}
