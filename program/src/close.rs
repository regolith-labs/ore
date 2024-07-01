use ore_api::{loaders::*, state::Proof};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};

use crate::utils::AccountDeserialize;

/// Close closes a proof account and returns the rent to the owner. Its responsibilities include:
/// 1. Realloc proof account size to 0.
/// 2. Transfer lamports to the owner.
///
/// Safety requirements:
/// - Deregister is a permissionless instruction and can be invoked by any singer.
/// - Can only succeed if the provided proof acount PDA is valid (associated with the signer).
/// - The provided system program must be valid.
pub fn process_close<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts
    let [signer, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_proof(proof_info, signer.key, true)?;
    load_program(system_program, system_program::id())?;

    // Validate balance is zero
    let proof_data = proof_info.data.borrow();
    let proof = Proof::try_from_bytes(&proof_data)?;
    if proof.balance.gt(&0) {
        return Err(ProgramError::InvalidAccountData);
    }
    drop(proof_data);

    // Realloc data to zero
    proof_info.realloc(0, true)?;

    // Send lamports to signer
    **signer.lamports.borrow_mut() += proof_info.lamports();
    **proof_info.lamports.borrow_mut() = 0;

    Ok(())
}
