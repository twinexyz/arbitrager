use crate::{arbitrager::run, config::Config};
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

    #[clap(long, short)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run,
    Show,
    DeleteDB,
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
        Commands::DeleteDB => todo!(),
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
