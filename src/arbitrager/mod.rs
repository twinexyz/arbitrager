use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use tokio::{
    sync::{mpsc, Mutex},
    task,
};

use crate::{
    balance_checker::BalanceChecker,
    config::Config,
    database::db::DB,
    json_rpc_server::server::ProofReceiver,
    poster::poster::Poster,
    types::{make_providers, ChainProviders},
    verifier::verifier::Verifier,
};


pub async fn run(cfg: Config) -> Result<()> {
    tracing::info!("Starting twine arbitrager");

    let provers: HashMap<String, String> = cfg
        .provers
        .iter()
        .map(|k| (k.1.prover_ip.clone(), k.1.prover_type.clone()))
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

    let providers: HashMap<String, ChainProviders> = make_providers(l1s);
    let balance_checker = BalanceChecker::new(providers.clone(), balance_check_interval);

    let proof_receiver = ProofReceiver::new(provers, verifier_tx);
    let db = Arc::new(Mutex::new(
        DB::new(poster_tx, post_status_rx, threshold, db_path).await,
    ));

    let mut verifier = Verifier::new(verifier_rx, Arc::clone(&db));

    let mut poster = Poster::new(providers, poster_rx, post_status_tx);

    let server_task = task::spawn(async move {
        proof_receiver
            .run_server(port)
            .await
            .expect("Failed to start server");
    });

    let validator_task = task::spawn(async move {
        verifier.run().await.expect("Failed to run validator");
    });

    let db_task = {
        let db_clone = Arc::clone(&db);
        task::spawn(async move {
            let mut db_lock = db_clone.lock().await;
            db_lock.run().await;
        })
    };

    let poster_task = task::spawn(async move {
        poster.run().await.expect("Failed to run validator");
    });

    let balance_check_task = task::spawn(async move {
        balance_checker
            .run()
            .await
            .expect("Failed to run validator");
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
