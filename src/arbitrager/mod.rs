use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use once_cell::sync::Lazy;
use tokio::{sync::mpsc, task};

use crate::{
    balance_checker::BalanceChecker,
    chains::chains::{make_l2_provider, make_providers, ChainProviders},
    config::Config,
    database::db::DB,
    error::ArbitragerError,
    json_rpc_server::server::JsonRpcServer,
    poster::poster::Poster,
    types::make_threshold_map,
    verifier::{sp1::SP1, verifier::Verifier},
};

pub static ELF_CONFIG: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn run(cfg: Config) -> Result<()> {
    tracing::info!("Starting twine arbitrager");

    let provers: HashMap<String, String> = cfg
        .provers
        .iter()
        .map(|k| (k.0.clone(), k.1.prover_type.clone()))
        .collect();

    // To pass data from server to the verifier
    let (verifier_tx, verifier_rx) = mpsc::channel(100);

    // From database to poster to post onchain
    let (poster_tx, poster_rx) = mpsc::channel(100);

    // Post status saved to db
    let (post_status_tx, post_status_rx) = mpsc::channel(100);

    let port = cfg.global.server_port;
    let db_path = cfg.global.db_path;
    let threshold = cfg.global.threshold;
    let balance_check_interval = cfg.global.balance_check_interval;
    let l1s = cfg.l1s;
    let l2 = cfg.l2;
    let start_batch_number = l2.start_batch_number;

    let elfs: HashMap<String, String> = cfg.elf;
    {
        let mut elf_config: std::sync::RwLockWriteGuard<'_, HashMap<String, String>> =
            ELF_CONFIG.write().unwrap();
        *elf_config = elfs;
    }

    let providers: HashMap<String, ChainProviders> = make_providers(l1s.clone());
    let balance_threshold: HashMap<String, String> = make_threshold_map(l1s);

    let balance_checker =
        BalanceChecker::new(providers.clone(), balance_threshold, balance_check_interval);

    let proof_receiver = JsonRpcServer::new(provers, verifier_tx);
    let db_arc = Arc::new(DB::new(threshold, db_path).await);

    let mut verifier = Verifier::new(verifier_rx, Arc::clone(&db_arc));

    let mut poster = Poster::new(
        providers,
        post_status_tx,
        make_l2_provider(l2),
        start_batch_number,
    );

    let server_task = task::spawn(async move {
        proof_receiver
            .run_server(port)
            .await
            .map_err(|e| ArbitragerError::JsonRPCServerError(e.to_string()))
    });

    let sp1 = SP1::new().await;

    let validator_task = task::spawn(async move {
        verifier
            .run(sp1, poster_tx)
            .await
            .map_err(|e| ArbitragerError::Custom(e.to_string()))
    });

    let db_clone = Arc::clone(&db_arc);
    let db_task = tokio::spawn(async move {
        db_clone
            .run(post_status_rx)
            .await
            .map_err(|e| ArbitragerError::DBError(e.to_string()))
    });

    let poster_task = task::spawn(async move {
        poster
            .run(poster_rx)
            .await
            .map_err(|e| ArbitragerError::PosterError(e.to_string()))
    });

    let balance_check_task = task::spawn(async move {
        balance_checker
            .run()
            .await
            .expect("Failed to run balance checker");
    });

    let _ = tokio::try_join!(
        server_task,
        validator_task,
        poster_task,
        balance_check_task,
        db_task
    );

    Ok(())
}
