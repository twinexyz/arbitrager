use std::fs::{self};

use alloy_primitives::{Bytes, FixedBytes};
use anyhow::Result;
use hex::FromHex;
use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues, SP1VerifyingKey};

use crate::{
    arbitrager::ELF_CONFIG,
    error::ArbitragerError,
    types::{PostParams, Sp1params, SupportedProvers},
};

use super::verifier::ProofTraits;

pub struct SP1 {
    pub prover_client: ProverClient,
    pub vk: SP1VerifyingKey,
}

impl SP1 {
    pub async fn new() -> SP1 {
        let client = ProverClient::new();

        // loaded with lazy static, should not fail
        let binding = ELF_CONFIG.read().unwrap();

        let elf_path = binding
            .get(&SupportedProvers::SP1.to_string())
            .ok_or_else(|| ArbitragerError::ELFFileNotFound(SupportedProvers::SP1.to_string()))
            .unwrap();

        let elf = fs::read(elf_path)
            .map_err(|_| ArbitragerError::FailToReadELF)
            .unwrap();

        let (_, vk) = client.setup(&elf);

        let vk_hash = vk.bytes32();
        tracing::info!("The verifying key is: {}", vk_hash);


        SP1 {
            prover_client: client,
            vk,
        }
    }

    pub fn verify_sp1_proof(&self, proof: SP1ProofWithPublicValues) -> Result<u64> {
        tracing::info!("Verifying sp1 proof");
        match self.prover_client.verify(&proof, &self.vk) {
            Ok(_) => {
                tracing::info!("SP1 Proof locally verified!");
                let pub_values = proof.public_values.as_slice();
                let height: u64 = u64::from_be_bytes(pub_values[0..8].try_into()?);
                Ok(height)
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl ProofTraits for SP1 {
    fn process_proof(proof: String, blocku64: u64) -> Result<PostParams> {
        match serde_json::from_str::<SP1ProofWithPublicValues>(&proof) {
            Ok(proof) => {
                let public_values = Bytes::copy_from_slice(proof.public_values.as_slice());

                let prf = proof
                    .clone()
                    .proof
                    .try_as_groth_16()
                    .ok_or(ArbitragerError::ProofParsingFailed)?
                    .encoded_proof;


                // TODO: Handle versions dynamically, fetch the key using Groth16Bn254Prover::get_vkey_hash method
                // bytes4(hash) is the selector
                // https://github.com/succinctlabs/sp1/blob/dev/crates/recursion/gnark-ffi/src/groth16_bn254.rs#L32
                let verifier_selector = "0x09069090".to_string();

                let final_proof = format!("{}{}", verifier_selector, prf);
                let plonk_proof = Bytes::from_hex(final_proof.clone())?;

                // Will not panic here
                let vk = FixedBytes::from_hex(
                    "00cc40b54ea20360aef4ad2a5665727179352b9cb7fb0df285468be78d71eff3",
                )
                .unwrap();

                let sp1_params = Sp1params {
                    vk,
                    public_values,
                    plonk_proof,
                };

                Ok(PostParams::Sp1(sp1_params, blocku64))
            }
            Err(e) => {
                tracing::error!("Failed to parse SP1 proof. error:{}", e);
                Err(e.into())
            }
        }
    }
}

pub fn verify_sp1_proof(proof: SP1ProofWithPublicValues) -> Result<u64> {
    tracing::info!("Verifying sp1 proof");
    let client = ProverClient::new();

    // loaded with lazy static, should not fail
    let binding = ELF_CONFIG.read().unwrap();

    let elf_path = binding
        .get(&SupportedProvers::SP1.to_string())
        .ok_or_else(|| ArbitragerError::ELFFileNotFound(SupportedProvers::SP1.to_string()))?;

    let elf = fs::read(elf_path).map_err(|_| ArbitragerError::FailToReadELF)?;

    let (_, vk) = client.setup(&elf);

    match client.verify(&proof, &vk) {
        Ok(_) => {
            tracing::info!("SP1 Proof locally verified!");
            let pub_values = proof.public_values.as_slice();
            let height: u64 = u64::from_be_bytes(pub_values[0..8].try_into()?);
            Ok(height)
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::BufReader};

    use sp1_sdk::{ProverClient, SP1ProofWithPublicValues};
    #[test]
    fn test_decode_proof() {
        let file = File::open("./assets/proof.json").expect("Proof File not found!");
        let reader = BufReader::new(file);
        let proof: SP1ProofWithPublicValues = serde_json::from_reader(reader).unwrap();
        let pub_values = proof.public_values.as_slice();
        let height: u64 = u64::from_be_bytes(pub_values[0..8].try_into().unwrap());
        println!("height: {height}");

        let prf = proof.clone().proof.try_as_groth_16().unwrap().encoded_proof;
        println!("Proof: {prf}");
    }

    #[test]
    fn test_verify_sp1_proof() {
        let elf = include_bytes!("../../assets/elf/riscv32im-succinct-zkvm-elf");
        // Initialize the prover client.
        let client = ProverClient::new();

        // Setup the program.
        let (pk, vk) = client.setup(elf);
        let file = File::open("./assets/proof.json").expect("Proof File not found!");

        println!("{file:?}");
        let reader = BufReader::new(file);
        let proof: SP1ProofWithPublicValues = serde_json::from_reader(reader).unwrap();

        let result = client.verify(&proof, &vk);
        assert!(result.is_ok());
    }
}
