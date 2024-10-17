pub mod arbitrager;
pub mod types;
pub mod cmd;
pub mod config;
pub mod database;
pub mod chains;
pub mod poster;
pub mod balance_checker;
pub mod json_rpc_server;
pub mod utils;
pub mod verifier;

use crate::cmd::cfg::init;

#[tokio::main]
async fn main() {
    init().await;
}
