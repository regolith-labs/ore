use ore_api::state::Proof;
use solana_program::system_program;
use steel::*;

/// Close closes a proof account and returns the rent to the owner.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    proof_info
        .is_writable()?
        .to_account::<Proof>(&ore_api::ID)?
        .check(|p| p.authority == *signer_info.key)?
        .check(|p| p.balance == 0)?;
    system_program.is_program(&system_program::ID)?;

    // Realloc data to zero.
    proof_info.realloc(0, true)?;

    // Send remaining lamports to signer.
    **signer_info.lamports.borrow_mut() += proof_info.lamports();
    **proof_info.lamports.borrow_mut() = 0;

    Ok(())
}
