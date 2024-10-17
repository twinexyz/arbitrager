use std::collections::HashMap;

use alloy_primitives::Bytes;
use alloy_primitives::FixedBytes;
use anyhow::Error;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sp1_sdk::SP1ProofWithPublicValues;

use crate::config::L1Details;

/// To be synced with json_rpc_server::ProofTypes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    SP1Proof(SP1ProofWithPublicValues, String),
    RISC0(Vec<u8>, String),
    Dummy(Vec<u8>, String),
}



#[derive(Serialize, Deserialize, Clone)]
pub enum PostParams {
    RiscZero(Risc0Params, u64),
    Sp1(Sp1params, u64),
    Dummy(DummyParams, u64),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sp1params {
    pub vk: FixedBytes<32>,
    pub public_values: Bytes,
    pub plonk_proof: Bytes,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Risc0Params {
    pub proof: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DummyParams {
    pub proof: Vec<u8>,
}

pub enum SupportedProvers {
    SP1,
    RISC0,
    Dummy,
}

impl SupportedProvers {
    pub fn to_string(&self) -> String {
        match &self {
            SupportedProvers::SP1 => "sp1".to_owned(),
            SupportedProvers::RISC0 => "risc0".to_owned(),
            SupportedProvers::Dummy => "dummy".to_owned(),
        }
    }

    pub fn from_str(prover: &str) -> Result<SupportedProvers> {
        match prover {
            "sp1" => Ok(SupportedProvers::SP1),
            "risc0" => Ok(SupportedProvers::RISC0),
            "dummy" => Ok(SupportedProvers::Dummy),
            _ => Err(Error::msg("Invalid prover. sp1 and risc0 supported")),
        }
    }
}

pub fn make_threshold_map(l1s: HashMap<String, L1Details>) -> HashMap<String, String> {
    l1s.iter()
        .map(|(key, detail)| (key.to_string(), detail.get_balance_threshold()))
        .collect()
}

