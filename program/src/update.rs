use ore_api::{loaders::*, state::Proof};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};
use steel::*;

/// Update changes the miner authority on a proof account.
pub fn process_update(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer, miner_info, proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_any(miner_info, false)?;
    load_proof(proof_info, signer.key, true)?;

    // Update the proof's miner authority.
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.miner = *miner_info.key;

    Ok(())
}
