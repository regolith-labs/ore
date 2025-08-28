use std::mem::size_of;

use ore_api::prelude::*;
use solana_program::{log::sol_log, rent::Rent};
use steel::*;

/// Migrates a miner account from the old format to the new format.
pub fn process_migrate_miner_account(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut(|c| c.admin == *signer_info.key)?;
    let miner = miner_info.as_account_mut::<MinerOLD>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Copy old data.
    let authority = miner.authority;
    let block_id = miner.block_id;
    let hashpower = miner.hashpower;
    let seed = miner.seed;
    let total_hashpower = miner.total_hashpower;
    let total_rewards = miner.total_rewards;

    // Reallocate new account.
    miner_info.realloc(8 + size_of::<Miner>(), false)?;

    // Transfer min required rent.
    let rent = Rent::get()?;
    let min_rent = rent.minimum_balance(8 + size_of::<Miner>());
    let lamport_delta = min_rent.saturating_sub(miner_info.lamports());
    miner_info.collect(lamport_delta, signer_info)?;

    // Copy new data.
    let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
    miner.authority = authority;
    miner.block_id = block_id;
    miner.executor = Pubkey::default();
    miner.hashpower = hashpower;
    miner.seed = seed;
    miner.total_hashpower = total_hashpower;
    miner.total_rewards = total_rewards;

    Ok(())
}
