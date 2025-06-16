use ore_api::prelude::*;
use steel::*;

/// Closes a block.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, market_info, mint_base_info, mint_quote_info, recipient_info, treasury_info, treasury_tokens_info, vault_base_info, vault_quote_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.start_slot + 1500)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.id == block.id)?;
    mint_base_info.has_address(&market.base.mint)?.as_mint()?;
    mint_quote_info.has_address(&market.quote.mint)?.as_mint()?;
    treasury_info
        .is_writable()?
        .has_address(&TREASURY_ADDRESS)?;
    let vault_base =
        vault_base_info.as_associated_token_account(market_info.key, mint_base_info.key)?;
    let vault_quote =
        vault_quote_info.as_associated_token_account(market_info.key, mint_quote_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Load treasury token accounts.
    if treasury_tokens_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            treasury_info,
            treasury_tokens_info,
            mint_quote_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        treasury_tokens_info.as_associated_token_account(&TREASURY_ADDRESS, mint_quote_info.key)?;
    }

    // Fund motherlode reward.
    mint_to_signed(
        mint_quote_info,
        treasury_tokens_info,
        treasury_info,
        token_program,
        OPEN_FEE,
        &[TREASURY],
    )?;

    // Payout block reward.
    if block.reward.lode_reward > 0 && block.reward.lode_authority != Pubkey::default() {
        // Load recipient.
        recipient_info.as_associated_token_account(&block.reward.lode_authority, &MINT_ADDRESS)?;

        // Mint reward to recipient.
        mint_to_signed(
            mint_quote_info,
            recipient_info,
            treasury_info,
            token_program,
            block.reward.lode_reward,
            &[TREASURY],
        )?;

        // Emit event.
        RewardEvent {
            amount: block.reward.lode_reward,
            authority: block.reward.lode_authority,
            block_id: block.id,
            rewards_type: RewardsType::Lode as u64,
            ts: clock.unix_timestamp,
        }
        .log();
    }

    // Burn base liquidity.
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
    let quote_burned = vault_quote.amount();
    burn_signed(
        vault_quote_info,
        mint_quote_info,
        market_info,
        token_program,
        quote_burned,
        &[MARKET, &market.id.to_le_bytes()],
    )?;

    // Close block.
    block_info.close(signer_info)?;

    // Close market.
    market_info.close(signer_info)?;

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
