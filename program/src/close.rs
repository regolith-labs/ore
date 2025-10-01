use ore_api::prelude::*;
use steel::*;

/// Closes a round accound, and returns the rent to the rent payer.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, rent_payer_info, round_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    rent_payer_info.is_writable()?;
    round_info
        .as_account_mut::<Round>(&ore_api::ID)?
        .assert_mut(|r| r.expires_at >= clock.slot)? // Ensure round has ended.
        .assert_mut(|r| r.rent_payer == *rent_payer_info.key)?; // Ensure the rent payer is the correct one.
    system_program.is_program(&system_program::ID)?;

    // Close the account.
    round_info.close(rent_payer_info)?;

    Ok(())
}
