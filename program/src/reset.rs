use ore_api::prelude::*;
use solana_program::{log::sol_log, slot_hashes::SlotHashes};
use steel::*;

use crate::swap::update_block_reward;

/// Resets a block.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_prev_info, block_next_info, config_info, market_info, mint_info, reserve_tokens_info, treasury_info, treasury_tokens_info, vault_info, system_program, token_program, ore_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block_next = block_next_info.as_account_mut::<Block>(&ore_api::ID)?;
    let config = config_info.as_account::<Config>(&ore_api::ID)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.block_id == block_next.id - 1)?;
    let ore_mint = mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    reserve_tokens_info
        .has_address(&BOOST_RESERVE_TOKEN)?
        .as_token_account()?
        .assert(|t| t.mint() == MINT_ADDRESS)?;
    treasury_info.as_account::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info
        .is_writable()?
        .as_associated_token_account(treasury_info.key, mint_info.key)?;
    let vault = vault_info.as_associated_token_account(&market_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Load previous block if market block ID is not 0.
    if market.block_id > 0 {
        let block_prev = block_prev_info
            .as_account_mut::<Block>(&ore_api::ID)?
            .assert_mut(|b| b.id == market.block_id)?
            .assert_mut(|b| b.end_slot <= clock.slot)?;

        // Get the slot hash, given the end slot of the previous block.
        let slot_hashes =
            bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
        if let Some(slot_hash) = slot_hashes.get(&block_prev.end_slot) {
            let slot_hash = slot_hash.to_bytes();

            // Set the block slot hash.
            block_prev.slot_hash = slot_hash;

            // Update the block reward.
            let clock = Clock::get()?;
            let reward_bytes = block_prev.reward.to_le_bytes();
            let limit = reward_bytes[0];
            let steps = reward_bytes[1];
            let (limit, _) = update_block_reward(
                limit as u64,
                steps as u64,
                &slot_hashes,
                block_prev.start_slot,
                clock.slot,
                block_prev.end_slot,
            );

            // Calculate the final block reward.
            let block_reward = finalize_block_reward(&block_prev.slot_hash, limit as u64);

            // Limit the block reward to supply cap.
            let max_reward = MAX_SUPPLY.saturating_sub(ore_mint.supply());
            let block_reward = block_reward.min(max_reward);

            // Set the block reward.
            block_prev.reward = block_reward;

            // Mint the block reward to the treasury.
            // This will get transferred to the miner account for claiming when the block is closed.
            mint_to_signed(
                mint_info,
                treasury_tokens_info,
                treasury_info,
                token_program,
                block_reward,
                &[TREASURY],
            )?;

            // Burn any remaining ORE in market liquidity vault.
            let burn_amount = vault.amount();
            burn_signed(
                vault_info,
                mint_info,
                market_info,
                token_program,
                burn_amount,
                &[MARKET],
            )?;

            sol_log(&format!("Burn amount: {:?}", burn_amount));
            sol_log(&format!("Mint amount: {:?}", block_reward));
        }
    }

    // Burn any remaining ORE in market liquidity vault.
    let vault = vault_info.as_associated_token_account(&market_info.key, &mint_info.key)?;
    let burn_amount = vault.amount();
    burn_signed(
        vault_info,
        mint_info,
        market_info,
        token_program,
        burn_amount,
        &[MARKET],
    )?;

    // Reset market account.
    market.block_id = block_next.id;
    market.base.balance = HASHPOWER_LIQUIDITY;
    market.base.balance_virtual = 0;
    market.quote.balance = 0;
    market.quote.balance_virtual = ORE_LIQUIDITY;
    market.snapshot.enabled = 1;
    market.snapshot.base_balance = 0;
    market.snapshot.quote_balance = 0;
    market.snapshot.slot = 0;
    market.fee.rate = 0;
    market.fee.uncollected = 0;
    market.fee.cumulative = 0;

    // Setup the next block start and end slots.
    block_next.start_at = clock.unix_timestamp;
    block_next.start_slot = clock.slot;
    block_next.end_slot = clock.slot + config.block_duration;
    block_next.reward = u64::from_le_bytes([5, 0, 0, 0, 0, 0, 0, 0]);

    // Mint tokens to the boost reserve.
    mint_to_signed(
        mint_info,
        reserve_tokens_info,
        treasury_info,
        token_program,
        ONE_ORE * 3,
        &[TREASURY],
    )?;

    // Emit event.
    program_log(
        &[market_info.clone(), ore_program.clone()],
        &ResetEvent {
            disc: 0,
            authority: *signer_info.key,
            block_id: block_next.id,
            start_slot: block_next.start_slot,
            end_slot: block_next.end_slot,
            ts: clock.unix_timestamp,
        }
        .to_bytes(),
    )?;

    Ok(())
}

// pub fn get_slot_hash(
//     slot: u64,
//     slot_hashes_sysvar: &AccountInfo<'_>,
// ) -> Result<[u8; 32], ProgramError> {
//     let slot_hashes =
//         bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
//     let Some(slot_hash) = slot_hashes.get(&slot) else {
//         // If reset is not called within ~2.5 minutes of the block ending,
//         // then the slot hash will be unavailable and secure hashes cannot be generated.
//         return Err(ProgramError::InvalidAccountData);
//     };
//     let slot_hash = slot_hash.to_bytes();
//     Ok(slot_hash)
// }

fn finalize_block_reward(slot_hash: &[u8], limit: u64) -> u64 {
    // Use slot hash to generate a random u64
    let r1 = u64::from_le_bytes(slot_hash[0..8].try_into().unwrap());
    let r2 = u64::from_le_bytes(slot_hash[8..16].try_into().unwrap());
    let r3 = u64::from_le_bytes(slot_hash[16..24].try_into().unwrap());
    let r4 = u64::from_le_bytes(slot_hash[24..32].try_into().unwrap());
    let r = r1 ^ r2 ^ r3 ^ r4;

    // Use modulo to get a number between 0 and (limit-1), then add 1
    let k = (r % limit) + 1;
    k * ONE_ORE
}
