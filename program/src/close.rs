use ore_api::{loaders::*, state::Proof};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    system_program,
};
use steel::*;

/// Close closes a proof account and returns the rent to the owner.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    load_proof(proof_info, signer_info.key, true)?;
    system_program.is_program(&system_program::ID)?;

    // Validate balance is zero.
    let proof_data = proof_info.data.borrow();
    let proof = Proof::try_from_bytes(&proof_data)?;
    if proof.balance.gt(&0) {
        return Err(ProgramError::InvalidAccountData);
    }
    drop(proof_data);

    // Realloc data to zero.
    proof_info.realloc(0, true)?;

    // Send remaining lamports to signer.
    **signer_info.lamports.borrow_mut() += proof_info.lamports();
    **proof_info.lamports.borrow_mut() = 0;

    Ok(())
}
