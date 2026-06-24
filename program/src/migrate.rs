use ore_api::prelude::*;
use solana_program::{pubkey, rent::Rent};
use steel::*;

const ADMIN_ADDRESS: Pubkey = pubkey!("EqbHxJd7UJDjDnZtbbgjo4egidbNgvSjttn1NHmF4aos");

const NEW_ADMIN: Pubkey = pubkey!("J5K5tWj3nKfxuSkAJ25WTMf4u5EsxJRfUoRKKxgrfFGV");

/// Migrates the config account.
pub fn process_migrate(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, round_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.has_address(&ADMIN_ADDRESS)?.is_signer()?;
    let round = round_info.as_account_mut::<RoundV1>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Record old data.
    let id = round.id;
    let deployed = round.deployed;
    let slot_hash = round.slot_hash;
    let count = round.count;
    let expires_at = round.expires_at;
    let motherlode = round.motherlode;
    let rent_payer = round.rent_payer;
    let top_miner = round.top_miner;
    let top_miner_reward = round.top_miner_reward;
    let total_deployed = round.total_deployed;
    let total_miners = round.total_miners;
    let total_vaulted = round.total_vaulted;
    let total_winnings = round.total_winnings;

    // Migrate the config account.
    // let old_size = ConfigV1::SIZE;
    // let new_size = ConfigV4::SIZE;
    // let old_rent = Rent::get()?.minimum_balance(old_size);
    // let new_rent = Rent::get()?.minimum_balance(new_size);
    // let lamports = new_rent - old_rent;

    // // Transfer the required rent to the new config account.
    // config_info.realloc(new_size, true)?;
    // config_info.collect(lamports, signer_info)?;

    // // Update config state.
    // let config = config_info.as_account_mut::<ConfigV4>(&ore_api::ID)?;
    // config.admin.authority = NEW_ADMIN;
    // config.admin.fee_collector = ADMIN_FEE_COLLECTOR;
    // config.admin.fee_rate = ADMIN_FEE;
    // config.protocol.authority = NEW_ADMIN;
    // config.protocol.fee_collector = TREASURY_ADDRESS;
    // config.protocol.fee_rate = ADMIN_FEE * 10;
    // config.protocol.intermission_slots = INTERMISSION_SLOTS;
    // config.protocol.round_slots = ROUND_SLOTS;

    Ok(())
}
