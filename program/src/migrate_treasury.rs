use ore_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

/// Sets the admin.
pub fn process_migrate_treasury(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, treasury_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    let treasury = treasury_info.as_account_mut::<TreasuryOLD>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Record old values.
    let balance = treasury.balance;
    let motherlode = treasury.motherlode;
    let stake_rewards_factor = treasury.stake_rewards_factor;
    let total_staked = treasury.total_staked;
    let total_unclaimed = treasury.total_unclaimed;
    let miner_rewards_factor = treasury.miner_rewards_factor;

    // Realloc treasury.
    let new_size = 8 + std::mem::size_of::<Treasury>();
    let old_size = 8 + std::mem::size_of::<TreasuryOLD>();
    let new_rent = Rent::get()?.minimum_balance(new_size);
    let old_rent = Rent::get()?.minimum_balance(old_size);
    let additional_rent = new_rent - old_rent;
    treasury_info.realloc(new_size, false)?;
    treasury_info.collect(additional_rent, &signer_info)?;

    // Update treasury.
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury.balance = balance;
    treasury.motherlode = motherlode;
    treasury.stake_rewards_factor = stake_rewards_factor;
    treasury.total_staked = total_staked;
    treasury.total_unclaimed = total_unclaimed;
    treasury.miner_rewards_factor = miner_rewards_factor;

    Ok(())
}
