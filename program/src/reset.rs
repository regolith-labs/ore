use ore_api::prelude::*;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Resets a block.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_prev_info, block_next_info, config_info, fee_collector_info, market_info, mint_info, vault_info, system_program, token_program, ore_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block_next = block_next_info.as_account_mut::<Block>(&ore_api::ID)?;
    let config = config_info.as_account::<Config>(&ore_api::ID)?;
    fee_collector_info
        .is_writable()?
        .as_associated_token_account(&config.fee_collector, &mint_info.key)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.block_id == block_next.id - 1)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let vault = vault_info.as_associated_token_account(&mint_info.key, &mint_info.key)?;
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

        // Set the slot hash of the previous block.
        let slot_hashes =
            bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
        let Some(slot_hash) = slot_hashes.get(&block_prev.end_slot) else {
            // If mine is not called within ~2.5 minutes of the block starting,
            // then the slot hash will be unavailable and secure hashes cannot be generated.
            return Ok(());
        };
        block_prev.slot_hash = slot_hash.to_bytes();
    }

    // Payout fee.
    if market.fee.uncollected > 0 {
        transfer_signed(
            market_info,
            vault_info,
            fee_collector_info,
            token_program,
            market.fee.uncollected,
            &[MARKET],
        )?;
        market.fee.uncollected = 0;
    }

    // Burn vault liquidity.
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
    market.fee.rate = config.fee_rate;
    market.fee.uncollected = 0;
    market.fee.cumulative = 0;

    // Mark the block start and end slots.
    block_next.start_slot = clock.slot;
    block_next.end_slot = clock.slot + config.block_duration;

    Ok(())
}
