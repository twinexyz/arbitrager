use std::collections::HashMap;

use anyhow::{Error, Result};
use futures::stream::StreamExt;
use mongodb::{
    bson::{self, doc, Bson, DateTime},
    Client, Collection, Database,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info_span;

use crate::verifier::verifier::ProofTraits;
use crate::{
    poster::poster::PostStatus,
    types::{DummyParams, PostParams, SupportedProvers},
};
use crate::{verifier::sp1::SP1, MAX_RETRIES};

use super::schema::{BlockFields, L1Details, ProofDetails, ProverDetails};

static DB_NAME: &str = "twine_arbitrager";
static PROOF_COLLECTION_NAME: &str = "proof_collection";
static POSTER_COLLECTION_NAME: &str = "l1s_collection";

async fn connect_to_mongodb(uri: &str) -> mongodb::error::Result<Database> {
    let client = Client::with_uri_str(uri).await?;
    let database = client.database(DB_NAME);
    Ok(database)
}

pub struct DB {
    pub threshold: usize,
    pub proof_collection: Collection<ProofDetails>,
    pub l1_collection: Collection<L1Details>,
}

impl DB {
    pub async fn new(threshold: usize, db_conn_str: String) -> Self {
        let database = match connect_to_mongodb(&db_conn_str).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("Failed to connect to database");
            }
        };

        let proof_collection: Collection<ProofDetails> = database.collection(PROOF_COLLECTION_NAME);
        let l1_collection: Collection<L1Details> = database.collection(POSTER_COLLECTION_NAME);

        Self {
            threshold,
            proof_collection,
            l1_collection,
        }
    }

    /// Delete all mongodb collections
    pub async fn delete_db(&self) -> Result<()> {
        self.proof_collection.drop().await?;
        self.l1_collection.drop().await?;
        Ok(())
    }

    /// The post status rx always receives response about the proof that was posted to multiple L1s.
    /// Once the proof posting is successful, this function saves that information to the posted collection.
    /// For now, the previous proofs are not deleted, but that can be done later with this function
    pub async fn run(&self, mut post_status_rx: Receiver<PostStatus>) -> Result<()> {
        tracing::info!("Database service running");
        while let Some(post_status) = post_status_rx.recv().await {
            let block = post_status.block.to_string();
            let chain = post_status.chain;
            let posted = post_status.posted;
            info_span!("", chain = chain, block = block);
            tracing::info!(
                "Command to update collection received after the data was posted to l1:{}",
                chain
            );

            let filter = doc! { format!("l1s.{}", block): { "$exists": true } };

            // TODO: Verify later
            match self.l1_collection.find_one(filter.clone()).await {
                Ok(doc) => match doc {
                    Some(mut doc) => {
                        let l1_entry = doc.l1s.entry(block.clone()).or_insert_with(HashMap::new);
                        l1_entry.insert(chain.clone(), posted);

                        let update = doc! {
                            "$set": {
                                format!("l1s.{}", block): bson::to_bson(&doc.l1s)?,
                            }
                        };
                        self.l1_collection.update_one(filter, update).await?;

                        tracing::info!(
                            "Proof post result added in db for chain:{} block:{}",
                            chain,
                            block
                        );
                    }
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
                },
                Err(e) => {
                    tracing::error!("Error finding status error: {:?}", e.to_string());
                    
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
    pub async fn find_oldest_proof(&self, block: String) -> Result<ProverDetails> {
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

        let mut cursor = self.proof_collection.aggregate(pipeline).await?;

        while let Some(doc) = cursor.next().await {
            if let Ok(document) = doc {
                if let Some(prover_detail_bson) = document
                    .get("prover_details_array")
                    .and_then(|pd_array| pd_array.as_document())
                    .and_then(|pd| pd.get("v").and_then(|v| v.as_document()))
                {
                    let prover_details: ProverDetails =
                        bson::from_bson(Bson::Document(prover_detail_bson.clone()))?;

                    return Ok(prover_details);
                }
            }
        }
        Err(Error::msg("Failed to fetch proofs"))
    }

    /// Should only arrive at this function ONLY IF the proof has been verified.
    /// The first proof, a new document is created. For second and onwards, the document is updated,
    /// In every step, threshold is checked. If threshold has reached, notify poster with block number
    pub async fn save_proof_to_db(
        &self,
        identifier: String,
        prover_type: SupportedProvers,
        block: u64,
        proof: String,
        poster_tx: Sender<PostParams>,
    ) -> Result<()> {
        let block_str = block.to_string();
        let filter = doc! { format!("blocks.{}", block_str): { "$exists": true } };

        match self.proof_collection.find_one(filter.clone()).await? {
            Some(existing_block_details) => {
                let mut blocks = existing_block_details.blocks;

                // New prover detail
                let prover_detail = ProverDetails {
                    proof,
                    proof_type: prover_type.to_string(),
                    verified: true,
                    timestamp: DateTime::now(),
                };

                let block_fields = blocks
                    .get_mut(&block_str)
                    .ok_or_else(|| Error::msg("Block fields missing"))?;

                let threshold_was_verified = block_fields.threshold_verified;
                if block_fields.prover_details.contains_key(&identifier) {
                    tracing::info!(
                        "Proof already submitted height:{} identifier:{}",
                        block_str,
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

                let update = doc! {
                    "$set": {
                        format!("blocks.{}.prover_details", block_str): bson::to_bson(&block_fields.prover_details)?,
                        format!("blocks.{}.threshold_verified", block_str): threshold_verified,
                        format!("blocks.{}.timestamp", block_str): block_fields.timestamp,
                    }
                };

                self.proof_collection.update_one(filter, update).await?;
                tracing::info!(
                    "Proof updated in db for block: {} identifier:{}",
                    block_str.clone(),
                    identifier
                );
                if !threshold_was_verified {
                    let proof: ProverDetails;
                    let mut count = 1;
                    loop {
                        match self.find_oldest_proof(block_str.clone()).await {
                            Ok(p) => {
                                proof = p;
                                break;
                            }
                            Err(e) => {
                                if count > MAX_RETRIES {
                                    return Err(e);
                                }
                                count += 1;
                            }
                        }
                    }

                    // Notify Poster It's ready to send proof for the block
                    if threshold_verified {
                        tracing::info!("Threshold verified for block: {}", block_str);
                        let params = match SupportedProvers::from_str(&proof.proof_type)? {
                            SupportedProvers::SP1 => SP1::process_proof(proof.proof, block),
                            SupportedProvers::RISC0 => todo!(),
                            SupportedProvers::Dummy => {
                                let proof_bytes = hex::decode(proof.proof)?;
                                let params_inner = DummyParams { proof: proof_bytes };
                                Ok(PostParams::Dummy(params_inner, block))
                            }
                        };
                        if let Ok(ref param) = params {
                            poster_tx.send(param.clone()).await?;
                        }
                    }
                }
            }
            None => {
                tracing::info!("Instance of that proof not found. Creating a new document!");
                let prover_detail = ProverDetails {
                    proof: proof.clone(),
                    proof_type: prover_type.to_string(),
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
                blocks.insert(block_str.clone(), block_fields);

                let final_struct = ProofDetails { blocks };

                // required structure is now ready
                tracing::info!("Ready to post to db");
                let res = self.proof_collection.insert_one(final_struct).await?;
                tracing::info!(
                    "Proof inserted to db at id: {} height:{} identifier:{}",
                    res.inserted_id,
                    block_str,
                    identifier
                );

                if threshold_verified {
                    let params = match prover_type {
                        SupportedProvers::SP1 => SP1::process_proof(proof, block),
                        SupportedProvers::RISC0 => todo!(),
                        SupportedProvers::Dummy => {
                            let proof_bytes = hex::decode(proof)?;
                            let params_inner = DummyParams { proof: proof_bytes };
                            Ok(PostParams::Dummy(params_inner, block_str.parse()?))
                        }
                    };
                    if let Ok(ref param) = params {
                        poster_tx.send(param.clone()).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
