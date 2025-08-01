use std::collections::HashMap;

use ore_api::prelude::*;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Resets a block.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_prev_info, block_next_info, config_info, market_info, mint_info, treasury_info, treasury_tokens_info, vault_info, system_program, token_program, ore_program, slot_hashes_sysvar] =
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
        if let Ok(slot_hash) = get_slot_hash(block_prev.end_slot, slot_hashes_sysvar) {
            // Set the block slot hash.
            block_prev.slot_hash = slot_hash;

            // Calculate the block reward.
            let block_reward = calculate_block_reward(&block_prev.slot_hash);

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

            // Transfer any remaining ORE in market liquidity vault to the treasury as additional block reward.
            let liq_amount = vault.amount();
            block_prev.reward += liq_amount;
            transfer_signed(
                market_info,
                vault_info,
                treasury_tokens_info,
                token_program,
                liq_amount,
                &[MARKET],
            )?;
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
    block_next.start_slot = clock.slot;
    block_next.end_slot = clock.slot + config.block_duration;

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

fn get_slot_hash(
    slot: u64,
    slot_hashes_sysvar: &AccountInfo<'_>,
) -> Result<[u8; 32], ProgramError> {
    let slot_hashes =
        bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
    let Some(slot_hash) = slot_hashes.get(&slot) else {
        // If reset is not called within ~2.5 minutes of the block ending,
        // then the slot hash will be unavailable and secure hashes cannot be generated.
        return Err(ProgramError::InvalidAccountData);
    };
    let slot_hash = slot_hash.to_bytes();
    Ok(slot_hash)
}

fn calculate_block_reward(slot_hash: &[u8]) -> u64 {
    let block_distribution = HashMap::from([
        (1u64, 737869762948382064),     // 4%
        (2u64, 1641760222560150093),    // 4.9%
        (3u64, 2564097426245627674),    // 5%
        (4u64, 3486434629931105255),    // 5%
        (5u64, 4408771833616582835),    // 5%
        (6u64, 5331109037302060416),    // 5%
        (7u64, 6253446240987537997),    // 5%
        (8u64, 7175783444673015578),    // 5%
        (9u64, 8098120648358493158),    // 5%
        (10u64, 9020457852043970739),   // 5%
        (11u64, 9942795055729448320),   // 5%
        (12u64, 10865132259414925901),  // 5%
        (13u64, 11787469463100403481),  // 5%
        (14u64, 12709806666785881062),  // 5%
        (15u64, 13632143870471358643),  // 5%
        (16u64, 14554481074156836224),  // 5%
        (17u64, 15476818277842313804),  // 5%
        (18u64, 16399155481527791385),  // 5%
        (19u64, 17321492685213268966),  // 5%
        (20u64, 18243829888898746547),  // 5%
        (100u64, 18428297329635842063), // 1%
        (1000u64, u64::MAX),            // 0.1%
    ]);
    let r1 = u64::from_le_bytes(slot_hash[0..8].try_into().unwrap());
    let r2 = u64::from_le_bytes(slot_hash[8..16].try_into().unwrap());
    let r3 = u64::from_le_bytes(slot_hash[16..24].try_into().unwrap());
    let r4 = u64::from_le_bytes(slot_hash[24..32].try_into().unwrap());
    let r = r1 ^ r2 ^ r3 ^ r4;
    for (k, v) in block_distribution.iter() {
        if r <= *v {
            return *k * ONE_ORE;
        }
    }
    0
}
