use crate::{arbitrager::run, config::Config};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{env::home_dir, fs::File, io::Read, path::PathBuf, process::{self}};

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
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!("Error running arbitrager: {}", e);
                        },
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load config: {}", e)
                }
            }
        }
        Commands::Show { config } => todo!(),
    }
}
