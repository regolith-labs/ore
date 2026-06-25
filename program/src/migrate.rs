use ore_api::prelude::*;
use solana_program::{pubkey, rent::Rent};
use steel::*;

const ADMIN_ADDRESS: Pubkey = pubkey!("EqbHxJd7UJDjDnZtbbgjo4egidbNgvSjttn1NHmF4aos");

/// Migrates the config account.
pub fn process_migrate(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.has_address(&ADMIN_ADDRESS)?.is_signer()?;
    // let miner = miner_info.as_account_mut::<MinerV1>(&ore_api::ID)?;
    // system_program.is_program(&system_program::ID)?;

    // // Record old data.
    // let authority = miner.authority;
    // let deployed = miner.deployed;
    // let cumulative = miner.cumulative;
    // let checkpoint_fee = miner.checkpoint_fee;
    // let checkpoint_id = miner.checkpoint_id;
    // let last_claim_ore_at = miner.last_claim_ore_at;
    // let last_claim_sol_at = miner.last_claim_sol_at;
    // let rewards_factor = miner.rewards_factor;
    // let rewards_sol = miner.rewards_sol;
    // let rewards_ore = miner.rewards_ore;
    // let refined_ore = miner.refined_ore;
    // let round_id = miner.round_id;
    // let lifetime_rewards_sol = miner.lifetime_rewards_sol;
    // let lifetime_rewards_ore = miner.lifetime_rewards_ore;
    // let lifetime_deployed = miner.lifetime_deployed;

    // // Migrate the miner account.
    // let old_size = MinerV1::SIZE;
    // let new_size = MinerV4::SIZE;
    // let old_rent = Rent::get()?.minimum_balance(old_size);
    // let new_rent = Rent::get()?.minimum_balance(new_size);
    // let lamports = new_rent.saturating_sub(old_rent);

    // // Transfer the required rent to the account.
    // miner_info.realloc(new_size, true)?;
    // miner_info.collect(lamports, signer_info)?;

    // // Update account state.
    // let miner = miner_info.as_account_mut::<MinerV4>(&ore_api::ID)?;
    // miner.authority = authority;
    // miner.deployed = deployed;
    // miner.cumulative = cumulative;
    // miner.checkpoint_fee = checkpoint_fee;
    // miner.checkpoint_id = checkpoint_id;
    // miner.last_claim_ore_at = last_claim_ore_at;
    // miner.last_claim_sol_at = last_claim_sol_at;
    // miner.rewards_factor = rewards_factor;
    // miner.rewards_sol = rewards_sol;
    // miner.rewards_ore = rewards_ore;
    // miner.refined_ore = refined_ore;
    // miner.round_id = round_id;
    // miner.lifetime_rewards_sol = lifetime_rewards_sol;
    // miner.lifetime_rewards_ore = lifetime_rewards_ore;
    // miner.lifetime_deployed = lifetime_deployed;
    // miner.auto_return = 1;
    // miner.mass = [0; 25];

    Ok(())
}
