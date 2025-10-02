use ore_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

/// Closes a round accound, and returns the rent to the rent payer.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, board_info, rent_payer_info, round_info, treasury_info, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let board = board_info.as_account_mut::<Board>(&ore_api::ID)?;
    rent_payer_info.is_writable()?;
    round_info
        .as_account_mut::<Round>(&ore_api::ID)?
        .assert_mut(|r| r.id < board.round_id)?
        .assert_mut(|r| r.expires_at < clock.slot)? // Ensure round has expired.
        .assert_mut(|r| r.rent_payer == *rent_payer_info.key)?; // Ensure the rent payer is the correct one.
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Vault all unclaimed rewards.
    let size = 8 + std::mem::size_of::<Round>();
    let min_rent = Rent::get()?.minimum_balance(size);
    let unclaimed_sol = round_info.lamports() - min_rent;
    if unclaimed_sol > 0 {
        round_info.send(unclaimed_sol, treasury_info);
        treasury.balance += unclaimed_sol;
    }

    // Close the account.
    round_info.close(rent_payer_info)?;

    Ok(())
}
