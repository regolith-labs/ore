use ore_api::prelude::*;
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
        .as_account::<Proof>(&ore_api::ID)?
        .assert_err(
            |p| p.authority == *signer_info.key,
            ProgramError::MissingRequiredSignature,
        )?
        .assert(|p| p.balance == 0)?;
    system_program.is_program(&system_program::ID)?;

    // Return rent to signer.
    proof_info.close(signer_info)?;

    Ok(())
}
