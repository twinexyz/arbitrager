use std::{
    collections::HashMap,
    net::SocketAddr,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{Result, Error};

use clap::builder::Str;
use rusqlite::{params, Connection};
use tokio::sync::mpsc::Sender;

use crate::config::{L1Details, ProverDetails};

pub struct DB {
    poster_tx: Sender<bool>,
    conn: Arc<Mutex<Connection>>,
}

static MAIN_TABLE: &str = "twine_arbitrager_main";
static PROOFS_TABLE_SUFFIX: &str = "_proofs";
static POSTER_TABLE_SUFFIX: &str = "_poster";

impl DB {
    pub fn new(
        poster_tx: Sender<bool>,
        path: String,
        provers: Vec<String>,
        l1s: Vec<String>,
    ) -> Self {
        let p = Path::new(&path);

        let conn = Connection::open(p).expect("Failed connecting to db");

        // Main table
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY,
                block_number INTEGER NOT NULL UNIQUE
            )",
            MAIN_TABLE
        );
        conn.execute(&create_table_query, [])
            .expect("Failed creating main table");

        // Provers Table
        for k in provers.iter() {
            let table_name = format!("{}{}", k, PROOFS_TABLE_SUFFIX);
            let query = format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    block INTEGER PRIMARY KEY,
                    proof TEXT,
                    verified BOOLEAN DEFAULT FALSE,
                    FOREIGN KEY (block) REFERENCES {}(block_number) ON DELETE CASCADE
                )",
                table_name, MAIN_TABLE
            );

            conn.execute(&query, [])
                .expect("Failed creating proofs table");
        }

        // Posters Table
        for k in l1s.iter() {
            let table_name = format!("{}{}", k, POSTER_TABLE_SUFFIX);
            let query = format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    block INTEGER PRIMARY KEY,
                    posted BOOLEAN DEFAULT FALSE,
                    FOREIGN KEY (block) REFERENCES {}(block_number) ON DELETE CASCADE
                )",
                table_name, MAIN_TABLE
            );

            conn.execute(&query, [])
                .expect("Failed creating poster table");
        }

        tracing::info!("Database loaded!");

        Self {
            poster_tx,
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn save_proof_to_db(&self, identifier: String, block: u64, proof: String) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Insert Block Number into main table
        let query = format!(
            "INSERT INTO {}{} (block_number) VALUES (?1)",
            identifier, MAIN_TABLE 
        );

        conn.execute(&query, [block])?;

        // Insert Proof into proofs table
        let query = format!(
            "INSERT INTO {}{} (batch, proof, verified) VALUES (?1, ?2, ?3)",
            identifier, MAIN_TABLE 
        );
        conn.execute(&query, params![block, proof, true]).expect("Failed to execute query");

        

        Ok(())
    }
}

// Mutex needed
pub fn save_to_db(block: u64, prover_address: SocketAddr, proof: Vec<u8>) {
    panic!("todo: implement this");
}
