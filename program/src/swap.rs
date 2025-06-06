use ore_api::prelude::*;
use steel::*;

/// Swap in a hashpower market.
pub fn process_swap(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Swap::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);
    let direction = SwapDirection::try_from(args.direction).unwrap();
    let precision = SwapPrecision::try_from(args.precision).unwrap();

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, market_info, mint_base_info, mint_quote_info, tokens_base_info, tokens_quote_info, vault_base_info, vault_quote_info, system_program, token_program] =
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
        .assert_mut(|m| m.id == block.id)?
        .assert_mut_err(
            |m| m.base.reserves() > 0,
            OreError::InsufficientLiquidity.into(),
        )?
        .assert_mut_err(
            |m| m.quote.reserves() > 0,
            OreError::InsufficientLiquidity.into(),
        )?;
    mint_base_info.has_address(&market.base.mint)?.as_mint()?;
    mint_quote_info.has_address(&market.quote.mint)?.as_mint()?;
    tokens_base_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, mint_base_info.key)?;
    tokens_quote_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, mint_quote_info.key)?;
    vault_base_info
        .is_writable()?
        .as_associated_token_account(market_info.key, mint_base_info.key)?;
    vault_quote_info
        .is_writable()?
        .as_associated_token_account(market_info.key, mint_quote_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Update market state.
    let swap_result = market.swap(amount, direction, precision, clock)?;
    swap_result.log_return();

    // Get transfer amounts and accounts.
    let (in_amount, in_from, in_to, out_amount, out_from, out_to) = match direction {
        SwapDirection::Buy => (
            swap_result.quote_to_transfer,
            tokens_quote_info,
            vault_quote_info,
            swap_result.base_to_transfer,
            vault_base_info,
            tokens_base_info,
        ),
        SwapDirection::Sell => (
            swap_result.base_to_transfer,
            tokens_base_info,
            vault_base_info,
            swap_result.quote_to_transfer,
            vault_quote_info,
            tokens_quote_info,
        ),
    };

    // Transfer tokens.
    transfer(signer_info, in_from, in_to, token_program, in_amount)?;
    transfer_signed(
        market_info,
        out_from,
        out_to,
        token_program,
        out_amount,
        &[
            MARKET,
            market.base.mint.as_ref(),
            market.quote.mint.as_ref(),
            market.id.to_le_bytes().as_ref(),
        ],
    )?;

    // Validate vault reserves.
    let vault_base =
        vault_base_info.as_associated_token_account(market_info.key, mint_base_info.key)?;
    let vault_quote =
        vault_quote_info.as_associated_token_account(market_info.key, mint_quote_info.key)?;
    market.check_vaults(&vault_base, &vault_quote)?;

    Ok(())
}
