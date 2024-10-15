use std::collections::HashMap;

use anyhow::Result;
use futures::stream::StreamExt;
use mongodb::{
    bson::{self, doc, Bson, DateTime},
    Client, Collection, Database,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::trace_span;

use crate::{types::DummyParams, poster::poster::PostStatus, types::PostParams};

use super::schema::{BlockFields, L1Details, ProofDetails, ProverDetails};

static DB_NAME: &str = "twine_arbitrager";
static PROOF_COLLECTION_NAME: &str = "proof_collection";
static POSTER_COLLECTION_NAME: &str = "l1s_collection";

async fn connect_to_mongodb(uri: &str) -> mongodb::error::Result<Database> {
    let client = Client::with_uri_str(uri).await.unwrap();
    let database = client.database(DB_NAME);
    Ok(database)
}

pub struct DB {
    pub poster_tx: Sender<PostParams>,
    pub post_status_rx: Receiver<PostStatus>,
    pub threshold: usize,
    pub proof_collection: Collection<ProofDetails>,
    pub l1_collection: Collection<L1Details>,
}

impl DB {
    pub async fn new(
        poster_tx: Sender<PostParams>,
        post_status_rx: Receiver<PostStatus>,
        threshold: usize,
        db_conn_str: String,
    ) -> Self {
        let database = match connect_to_mongodb(&db_conn_str).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("Failed to connect to database");
            }
        };

        if threshold < 1 {
            eprintln!("Threshold cannot be less than 1");
        }

        let proof_collection: Collection<ProofDetails> = database.collection(PROOF_COLLECTION_NAME);
        let l1_collection: Collection<L1Details> = database.collection(POSTER_COLLECTION_NAME);

        Self {
            poster_tx,
            post_status_rx,
            threshold,
            proof_collection,
            l1_collection,
        }
    }

    /// The post status rx always receives response about the proof that was posted to multiple L1s.
    /// Once the proof posting is successful, this function saves that information to the posted collection.
    /// For now, the previous proofs are not deleted, but that can be done later with this function
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Database service running");
        while let Some(post_status) = self.post_status_rx.recv().await {
            println!("Received msg in post status");
            let block = post_status.block.to_string();
            let chain = post_status.chain;
            let posted = post_status.posted;
            trace_span!("", chain = chain, block = block);

            let filter = doc! { format!("l1s.{}", block): { "$exists": true } };

            match self.l1_collection.find_one(filter.clone()).await? {
                Some(_) => todo!(),
                None => {
                    let mut poster_1 = HashMap::new();
                    poster_1.insert(chain.clone(), posted);
                    let mut l1s = HashMap::new();
                    l1s.insert(block.clone(), poster_1);

                    let final_struct = L1Details { l1s };
                    let res = self.l1_collection.insert_one(final_struct).await?;
                    tracing::info!(
                        "Proof post result inserted to db at id: {} chain:{} height:{}",
                        res.inserted_id,
                        chain,
                        block
                    );
                }
            }
        }
        Ok(())
    }

    /// Get the first proof that was submitted to db for the block.
    pub async fn find_oldest_proof(&self, block: String) -> Option<ProverDetails> {
        let pipeline = vec![
            doc! {
                "$match": doc! {
                    format!("blocks.{}.prover_details",block): doc! {
                        "$exists": true
                    }
                }
            },
            doc! {
                "$project": doc! {
                    "prover_details_array": doc! {
                        "$objectToArray": format!("$blocks.{}.prover_details",block)
                    }
                }
            },
            doc! {
                "$unwind": doc! {
                    "path": "$prover_details_array"
                }
            },
            doc! {
                "$sort": doc! {
                    "prover_details_array.v.timestamp": 1
                }
            },
            doc! {
                "$limit": 1
            },
        ];

        let mut cursor = self.proof_collection.aggregate(pipeline).await.unwrap();

        while let Some(doc) = cursor.next().await {
            if let Ok(document) = doc {
                if let Some(prover_detail_bson) = document
                    .get("prover_details_array")
                    .and_then(|pd_array| pd_array.as_document())
                    .and_then(|pd| pd.get("v").and_then(|v| v.as_document()))
                {
                    let prover_details: ProverDetails =
                        bson::from_bson(Bson::Document(prover_detail_bson.clone())).unwrap();

                    return Some(prover_details);
                }
            }
        }
        None
    }

    /// Should only arrive at this function ONLY IF the proof has been verified.
    /// The first proof, a new document is created. For second and onwards, the document is updated,
    /// In every step, threshold is checked. If threshold has reached, notify poster with block number
    pub async fn save_proof_to_db(
        &self,
        identifier: String,
        prover_type: String,
        block: u64,
        proof: String,
    ) -> Result<()> {
        println!("Saving proof to db");
        let block = block.to_string();
        let filter = doc! { format!("blocks.{}", block): { "$exists": true } };

        match self.proof_collection.find_one(filter.clone()).await? {
            Some(existing_block_details) => {
                tracing::info!("Instance of that proof found!");
                let mut blocks = existing_block_details.blocks;

                // New prover detail
                let prover_detail = ProverDetails {
                    proof,
                    proof_type: prover_type,
                    verified: true,
                    timestamp: DateTime::now(),
                };

                let block_fields = blocks.get_mut(&block.to_string()).unwrap();

                let threshold_was_verified = block_fields.threshold_verified;
                tracing::info!("Block fields received");
                if block_fields.prover_details.contains_key(&identifier) {
                    tracing::info!(
                        "Proof already submitted height:{} identifier:{}",
                        block,
                        identifier
                    );
                    return Ok(());
                }
                block_fields
                    .prover_details
                    .insert(identifier.clone(), prover_detail);

                let threshold_verified = block_fields.prover_details.len() >= self.threshold;
                block_fields.threshold_verified = threshold_verified;
                block_fields.timestamp = DateTime::now();
                tracing::info!("Time to update now!");

                let update = doc! {
                    "$set": {
                        format!("blocks.{}.prover_details", block): bson::to_bson(&block_fields.prover_details)?,
                        format!("blocks.{}.threshold_verified", block): threshold_verified,
                        format!("blocks.{}.timestamp", block): block_fields.timestamp,
                    }
                };

                tracing::info!("Query readyy!");
                self.proof_collection
                    .update_one(filter, update)
                    .await
                    .unwrap();
                tracing::info!(
                    "Proof updated in db for block: {} identifier:{}",
                    block.clone(),
                    identifier
                );
                if !threshold_was_verified {
                    let proof: ProverDetails = self.find_oldest_proof(block.clone()).await.unwrap();

                    // Parse the proof into required structure i.e. PostParams

                    // Notify Poster It's ready to send proof for the block
                    if threshold_verified {
                        let proof_bytes = hex::decode(proof.proof).unwrap();
                        let params_inner = DummyParams{
                            block: block.parse().unwrap(),
                            proof: proof_bytes,
                        };
                        let params = PostParams::Dummy(params_inner);
                        self.poster_tx.send(params).await.unwrap();
                    }
                }
            }
            None => {
                tracing::info!("Instance of that proof not found. Creating a new document!");
                let prover_detail = ProverDetails {
                    proof,
                    proof_type: prover_type,
                    verified: true,
                    timestamp: DateTime::now(),
                };
                let mut prover_details = HashMap::new();
                prover_details.insert(identifier.clone(), prover_detail);

                let threshold_verified = self.threshold == 1;

                let block_fields = BlockFields {
                    prover_details,
                    threshold_verified,
                    timestamp: DateTime::now(),
                };

                let mut blocks = HashMap::new();
                blocks.insert(block.clone(), block_fields);

                let final_struct = ProofDetails { blocks };

                // required structure is now ready
                tracing::info!("Ready to post to db");
                let res = self
                    .proof_collection
                    .insert_one(final_struct)
                    .await
                    .unwrap();
                tracing::info!(
                    "Proof inserted to db at id: {} height:{} identifier:{}",
                    res.inserted_id,
                    block,
                    identifier
                );

                if threshold_verified {
                    // self.poster_tx.send(block).await.unwrap();
                }
            }
        }
        Ok(())
    }
}
