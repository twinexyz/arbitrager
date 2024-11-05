use super::verifier::ProofTraits;

pub struct Dummy;

impl ProofTraits for Dummy {
    fn process_proof(proof: String, blocku64: u64) -> Option<crate::types::PostParams> {
        todo!()
    }
}
