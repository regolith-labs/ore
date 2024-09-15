use coal_api::{consts::WOOD_MINT_ADDRESS, loaders::*, state::ProofV2};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    system_program,
};

use crate::utils::AccountDeserialize;

/// Close closes a proof account and returns the rent to the owner.
pub fn process_close_wood<'a, 'info>(accounts: &'a [AccountInfo<'info>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_proof_v2(proof_info, signer.key, &WOOD_MINT_ADDRESS, true)?;
    load_program(system_program, system_program::id())?;

    // Validate balance is zero.
    let proof_data = proof_info.data.borrow();
    let proof = ProofV2::try_from_bytes(&proof_data)?;
    if proof.balance.gt(&0) {
        return Err(ProgramError::InvalidAccountData);
    }
    drop(proof_data);

    // Realloc data to zero.
    proof_info.realloc(0, true)?;

    // Send remaining lamports to signer.
    **signer.lamports.borrow_mut() += proof_info.lamports();
    **proof_info.lamports.borrow_mut() = 0;

    Ok(())
}