use ore_api::{loaders::*, state::Proof};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::AccountDeserialize;

/// Update changes the miner authority on a proof account.
pub fn process_update<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts
    let [signer, miner_info, proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_system_account(miner_info, false)?;
    load_proof(proof_info, signer.key, true)?;

    // Update the proof
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.miner = *miner_info.key;

    Ok(())
}
