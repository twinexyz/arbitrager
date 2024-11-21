use super::verifier::ProofTraits;
use anyhow::Result;

pub struct Dummy;

impl ProofTraits for Dummy {
    fn process_proof(proof: String, blocku64: u64) -> Result<crate::types::PostParams> {
        let _ = proof;
        let _ = blocku64;
        todo!()
    }
    
    fn public_values(proof_json: &std::path::PathBuf) -> Result<String> {
        let _ = proof_json;
        todo!()
    }
}
