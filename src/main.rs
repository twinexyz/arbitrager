pub mod aggregator;
pub mod balance_checker;
pub mod chains;
pub mod cmd;
pub mod config;
pub mod database;
pub mod error;
pub mod json_rpc_server;
pub mod poster;
pub mod types;
pub mod utils;
pub mod verifier;

use crate::cmd::cfg::init;
use anyhow::Result;

static MAX_RETRIES: i32 = 5;

#[tokio::main]
async fn main() -> Result<()> {
    init().await
}
