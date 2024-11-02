pub mod arbitrager;
pub mod balance_checker;
pub mod chains;
pub mod cmd;
pub mod config;
pub mod database;
pub mod json_rpc_server;
pub mod poster;
pub mod types;
pub mod utils;
pub mod verifier;

use crate::cmd::cfg::init;

#[tokio::main]
async fn main() {
    init().await;
}
