use ore_api::prelude::*;
use steel::*;

/// Closes a block.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, config_info, collateral_info, commitment_info, fee_collector_info, market_info, miner_info, mint_base_info, mint_quote_info, opener_info, recipient_info, treasury_info, vault_base_info, vault_quote_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.start_slot + 1500)?;
    let config = config_info.as_account::<Config>(&ore_api::ID)?;
    collateral_info
        .is_writable()?
        .has_address(&collateral_pda(block.id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_quote_info.key)?
        .assert(|t| t.owner() == *market_info.key)?;
    commitment_info
        .is_writable()?
        .has_address(&commitment_pda(block.id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_base_info.key)?
        .assert(|t| t.owner() == *block_info.key)?;
    fee_collector_info
        .is_writable()?
        .as_associated_token_account(&config.fee_collector, &mint_quote_info.key)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.id == block.id)?;
    mint_base_info.has_address(&market.base.mint)?.as_mint()?;
    mint_quote_info.has_address(&market.quote.mint)?.as_mint()?;
    opener_info.is_writable()?.has_address(&block.opener)?;
    treasury_info
        .is_writable()?
        .has_address(&TREASURY_ADDRESS)?;
    vault_base_info
        .is_writable()?
        .has_address(&vault_base_pda(block.id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_base_info.key)?
        .assert(|t| t.owner() == *market_info.key)?;
    vault_quote_info
        .is_writable()?
        .has_address(&vault_quote_pda(block.id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_quote_info.key)?
        .assert(|t| t.owner() == *market_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Payout block reward.
    if block.reward.lode_reward > 0 && block.reward.lode_authority != Pubkey::default() {
        // Load recipient.
        recipient_info.as_associated_token_account(&block.reward.lode_authority, &MINT_ADDRESS)?;

        let miner = miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.authority == block.reward.lode_authority)?;

        // Limit payout to supply cap.
        let ore_mint = mint_quote_info.as_mint()?;
        let max_reward = MAX_SUPPLY.saturating_sub(ore_mint.supply());
        let reward_amount = block.reward.lode_reward.min(max_reward);

        // Update stats.
        block.total_rewards += reward_amount;
        miner.total_rewards += reward_amount;

        // Mint reward to recipient.
        mint_to_signed(
            mint_quote_info,
            recipient_info,
            treasury_info,
            token_program,
            reward_amount,
            &[TREASURY],
        )?;

        // Emit event.
        RewardEvent {
            disc: OreEvent::Reward as u64,
            amount: reward_amount,
            authority: block.reward.lode_authority,
            block_id: block.id,
            rewards_type: RewardsType::Lode as u64,
            ts: clock.unix_timestamp,
        }
        .log();
    }

    // Payout fee.
    if market.fee.uncollected > 0 {
        transfer_signed(
            market_info,
            vault_quote_info,
            fee_collector_info,
            token_program,
            market.fee.uncollected,
            &[MARKET, &market.id.to_le_bytes()],
        )?;
        market.fee.uncollected = 0;
    }

    // Burn base liquidity.
    let vault_base = vault_base_info.as_token_account()?;
    let base_burned = vault_base.amount();
    burn_signed(
        vault_base_info,
        mint_base_info,
        market_info,
        token_program,
        base_burned,
        &[MARKET, &market.id.to_le_bytes()],
    )?;

    // Burn quote liquidity.
    let vault_quote = vault_quote_info.as_token_account()?;
    let quote_burned = vault_quote.amount();
    burn_signed(
        vault_quote_info,
        mint_quote_info,
        market_info,
        token_program,
        quote_burned,
        &[MARKET, &market.id.to_le_bytes()],
    )?;

    // Burn any remaining commitment.
    let commitment = commitment_info.as_token_account()?;
    let commitment_burned = commitment.amount();
    burn_signed(
        commitment_info,
        mint_base_info,
        block_info,
        token_program,
        commitment_burned,
        &[BLOCK, &block.id.to_le_bytes()],
    )?;

    // Close token accounts.
    close_token_account_signed(
        vault_base_info,
        opener_info,
        market_info,
        token_program,
        &[MARKET, &market.id.to_le_bytes()],
    )?;
    close_token_account_signed(
        vault_quote_info,
        opener_info,
        market_info,
        token_program,
        &[MARKET, &market.id.to_le_bytes()],
    )?;
    close_token_account_signed(
        commitment_info,
        opener_info,
        block_info,
        token_program,
        &[BLOCK, &block.id.to_le_bytes()],
    )?;

    // Close block.
    block_info.close(opener_info)?;

    // Close market.
    market_info.close(opener_info)?;

    // Emit event.
    CloseEvent {
        authority: *signer_info.key,
        id: block.id,
        burned_base: base_burned,
        burned_quote: quote_burned,
        ts: clock.unix_timestamp,
    }
    .log_return();

    Ok(())
}
