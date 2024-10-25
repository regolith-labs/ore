use ore_api::prelude::*;
use steel::*;

/// Update changes the miner authority on a proof account.
pub fn process_update(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, miner_info, proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let proof = proof_info
        .as_account_mut::<Proof>(&ore_api::ID)?
        .assert_mut_err(
            |p| p.authority == *signer_info.key,
            ProgramError::MissingRequiredSignature,
        )?;

    // Update the proof's miner authority.
    proof.miner = *miner_info.key;

    Ok(())
}
