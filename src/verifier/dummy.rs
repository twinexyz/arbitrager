use super::verifier::ProofTraits;
use anyhow::Result;

pub struct Dummy;

impl ProofTraits for Dummy {
    fn process_proof(proof: String, blocku64: u64) -> Result<crate::types::PostParams> {
        todo!()
    }
}
