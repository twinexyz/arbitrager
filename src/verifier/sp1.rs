use anyhow::Result;
use sp1_sdk::SP1ProofWithPublicValues;

pub fn sp1_proof_verify(identifier: String, proof: SP1ProofWithPublicValues) -> Result<()> {
    verify_proof(proof)
}

/// TODO:
/// How to fetch the vk?
///     - From the contract on L1 ?
///     - From the config?
///     - From the elf file?
///
/// Parse the public block parameters to check which block is the one being verfied
/// After this verification is done, save to DB service
fn verify_proof(proof: SP1ProofWithPublicValues) -> Result<()> {
    // let client = ProverClient::new();

    // let block = 1u64;

    // match client.verify(&proof, vk) {
    //     Ok(_) => {
    //         tracing::info!("Block verified for block: client:");
    //         Ok(())
    //     },
    //     Err(e) => Err(Error::msg("SP1 Verification Error!")),
    // }
    Ok(())
}
