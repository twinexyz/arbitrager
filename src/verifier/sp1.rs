use std::fs::{self};

use alloy_primitives::{Bytes, FixedBytes};
use anyhow::{Error, Result};
use hex::FromHex;
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues};

use crate::{
    arbitrager::ELF_CONFIG,
    types::{PostParams, Sp1params, SupportedProvers},
};

use super::verifier::ProofTraits;

pub struct SP1;

impl ProofTraits for SP1 {
    fn process_proof(proof: String, blocku64: u64) -> Option<PostParams> {
        match serde_json::from_str::<SP1ProofWithPublicValues>(&proof) {
            Ok(proof) => {
                let public_values = Bytes::copy_from_slice(proof.public_values.as_slice());

                let prf = proof.clone().proof.try_as_groth_16().unwrap().encoded_proof;

                // TODO: Handle versions dynamically
                let verifier_selector = "0x09069090".to_string();

                let final_proof = format!("{}{}", verifier_selector, prf);
                let plonk_proof = Bytes::from_hex(final_proof.clone()).unwrap();

                let vk = FixedBytes::from_hex(
                    "00cc40b54ea20360aef4ad2a5665727179352b9cb7fb0df285468be78d71eff3",
                )
                .unwrap();

                let sp1_params = Sp1params {
                    vk,
                    public_values,
                    plonk_proof,
                };

                Some(PostParams::Sp1(sp1_params, blocku64))
            }
            Err(e) => {
                tracing::error!("Failed to parse SP1 proof. error:{}", e);
                None
            }
        }
    }
}

pub fn verify_sp1_proof(proof: SP1ProofWithPublicValues) -> Result<u64> {
    tracing::info!("Verifying sp1 proof");
    let client = ProverClient::new();
    let binding = ELF_CONFIG.read().unwrap();

    let elf_path = binding
        .get(&SupportedProvers::SP1.to_string())
        .expect("Failed to get elf path");
    let elf = fs::read(elf_path).expect("Failed to read ELF file");

    let (_, vk) = client.setup(&elf);

    match client.verify(&proof, &vk) {
        Ok(_) => {
            tracing::info!("SP1 Proof locally verified!");
            let pub_values = proof.public_values.as_slice();
            let height: u64 = u64::from_be_bytes(pub_values[0..8].try_into().unwrap());
            Ok(height)
        }
        Err(_) => Err(Error::msg("failed to verify proof")),
    }
}
