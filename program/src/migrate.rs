use ore_api::prelude::*;
use solana_program::{pubkey, rent::Rent};
use steel::*;

const ADMIN_ADDRESS: Pubkey = pubkey!("EqbHxJd7UJDjDnZtbbgjo4egidbNgvSjttn1NHmF4aos");

/// Migrates the config account.
pub fn process_migrate(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, treasury_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.has_address(&ADMIN_ADDRESS)?.is_signer()?;
    // let treasury = treasury_info.as_account_mut::<TreasuryV1>(&ore_api::ID)?;
    // system_program.is_program(&system_program::ID)?;

    // // Record old data.
    // let motherlode = treasury.motherlode;
    // let miner_rewards_factor = treasury.miner_rewards_factor;
    // let total_refined = treasury.total_refined;
    // let total_unclaimed = treasury.total_unclaimed;

    // // Migrate the treasury account.
    // let old_size = TreasuryV1::SIZE;
    // let new_size = TreasuryV4::SIZE;
    // let old_rent = Rent::get()?.minimum_balance(old_size);
    // let new_rent = Rent::get()?.minimum_balance(new_size);
    // let lamports = new_rent.saturating_sub(old_rent);

    // // Transfer the required rent to the account.
    // treasury_info.realloc(new_size, true)?;
    // treasury_info.collect(lamports, signer_info)?;

    // // Update account state.
    // let treasury = treasury_info.as_account_mut::<TreasuryV4>(&ore_api::ID)?;
    // treasury.motherlode = motherlode;
    // treasury.miner_rewards_factor = miner_rewards_factor;
    // treasury.total_refined = total_refined;
    // treasury.total_unclaimed = total_unclaimed;

    Ok(())
}
