use std::collections::HashMap;
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct ProofDetails{
    pub blocks: HashMap<String, BlockFields>,
}

#[derive(Serialize, Deserialize)]
pub struct L1Details {
    pub l1s: HashMap<String, HashMap<String, bool>>

}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockFields{
    pub prover_details: HashMap<String, ProverDetails>,
    pub threshold_verified: bool,
    pub timestamp: DateTime,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ProverDetails {
    pub proof: String,
    pub proof_type: String,
    pub verified: bool,
    pub timestamp: DateTime
}
