use super::verifier::ProofTraits;
use anyhow::Result;

pub struct RISC0;

impl ProofTraits for RISC0 {
    fn process_proof(proof: String, blocku64: u64) -> Result<crate::types::PostParams> {
        let _ = blocku64;
        let _ = proof;
        todo!()
    }
}
