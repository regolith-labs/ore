use ore_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

/// Sets the admin.
pub fn process_migrate_miner(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    let miner = miner_info.as_account_mut::<MinerOLD>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Record old values.
    let authority = miner.authority;
    let deployed = miner.deployed;
    let cumulative = miner.cumulative;
    let checkpoint_fee = miner.checkpoint_fee;
    let checkpoint_id = miner.checkpoint_id;
    let rewards_sol = miner.rewards_sol;
    let rewards_ore = miner.rewards_ore;
    let round_id = miner.round_id;
    let lifetime_rewards_sol = miner.lifetime_rewards_sol;
    let lifetime_rewards_ore = miner.lifetime_rewards_ore;

    // Realloc miner.
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
    miner.cumulative = cumulative;
    miner.checkpoint_fee = checkpoint_fee;
    miner.checkpoint_id = checkpoint_id;
    miner.rewards_sol = rewards_sol;
    miner.rewards_ore = rewards_ore;
    miner.round_id = round_id;
    miner.lifetime_rewards_sol = lifetime_rewards_sol;
    miner.lifetime_rewards_ore = lifetime_rewards_ore;
    miner.last_claim_ore_at = 0;
    miner.last_claim_sol_at = 0;
    miner.rewards_factor = Numeric::ZERO;
    miner.refined_ore = 0;

    Ok(())
}
