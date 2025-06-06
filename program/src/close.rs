use ore_api::prelude::*;
use steel::*;

/// Closes a block.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, market_info, market_ore_info, mint_base_info, mint_quote_info, recipient_info, treasury_info, vault_base_info, vault_quote_info, system_program, token_program] =
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
    let vault_base =
        vault_base_info.as_associated_token_account(market_info.key, mint_base_info.key)?;
    let vault_quote =
        vault_quote_info.as_associated_token_account(market_info.key, mint_quote_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Payout block reward.
    if block.best_miner != Pubkey::default() {
        recipient_info.as_associated_token_account(&block.best_miner, &MINT_ADDRESS)?;
        mint_to_signed(
            mint_quote_info,
            recipient_info,
            treasury_info,
            token_program,
            block.reward,
            &[TREASURY],
        )?;
    }

    // Burn base liquidity.
    burn_signed(
        vault_base_info,
        mint_base_info,
        market_info,
        token_program,
        vault_base.amount(),
        &[MARKET, &market.id.to_le_bytes()],
    )?;

    // Burn quote liquidity.
    burn_signed(
        vault_quote_info,
        mint_quote_info,
        market_info,
        token_program,
        vault_quote.amount(),
        &[MARKET, &market.id.to_le_bytes()],
    )?;

    // Close block.
    block_info.close(signer_info)?;

    // Close market.
    market_info.close(signer_info)?;

    Ok(())
}
