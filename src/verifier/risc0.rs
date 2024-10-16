
use super::verifier::ProofTraits;

pub struct RISC0;

impl ProofTraits for RISC0 {
    fn process_proof(proof: String, blocku64: u64) -> Option<crate::types::PostParams> {
        todo!()
    }
}
