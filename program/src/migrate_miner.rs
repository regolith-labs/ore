use ore_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

/// Sets the admin.
pub fn process_migrate_miner(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner_old = miner_info
        .as_account_mut::<MinerOLD>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)?;
    config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    system_program.is_program(&system_program::ID)?;

    // Record old values.
    let authority = miner_old.authority;
    let deployed = miner_old.deployed;
    let executor = miner_old.executor;
    let rewards_sol = miner_old.rewards_sol;
    let rewards_ore = miner_old.rewards_ore;
    let round_id = miner_old.round_id;
    let lifetime_rewards_sol = miner_old.lifetime_rewards_sol;
    let lifetime_rewards_ore = miner_old.lifetime_rewards_ore;

    // Realloc miner
    let new_size = 8 + std::mem::size_of::<Miner>();
    let old_size = 8 + std::mem::size_of::<MinerOLD>();
    let new_rent = Rent::get()?.minimum_balance(new_size);
    let old_rent = Rent::get()?.minimum_balance(old_size);
    let additional_rent = new_rent - old_rent;
    miner_info.realloc(new_size, false)?;
    miner_info.collect(additional_rent, &signer_info)?;

    // Update miner.
    let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
    miner.authority = authority;
    miner.deployed = deployed;
    miner.executor = executor;
    miner.refund_sol = 0;
    miner.rewards_sol = rewards_sol;
    miner.rewards_ore = rewards_ore;
    miner.round_id = round_id;
    miner.lifetime_rewards_sol = lifetime_rewards_sol;
    miner.lifetime_rewards_ore = lifetime_rewards_ore;

    Ok(())
}
