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
    let [signer_info, block_info, market_info, mint_base_info, mint_quote_info, stake_info, tokens_base_info, tokens_quote_info, vault_base_info, vault_quote_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block: &mut Block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot < b.start_slot)?;
    // collateral_info
    //     .is_writable()?
    //     .has_address(&collateral_pda(block.id).0)?
    //     .as_token_account()?
    //     .assert(|t| t.mint() == *mint_quote_info.key)?
    //     .assert(|t| t.owner() == *market_info.key)?;
    // collateral_info.as_associated_token_account(block_info.key, mint_quote_info.key)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.id == block.id)?
        .assert_mut(|m| m.base.liquidity() > 0)?
        .assert_mut(|m| m.quote.liquidity() > 0)?;
    mint_base_info.has_address(&market.base.mint)?.as_mint()?;
    mint_quote_info.has_address(&market.quote.mint)?.as_mint()?;
    let stake = stake_info
        .as_account_mut::<Stake>(&ore_api::ID)?
        .assert_mut(|p| p.authority == *signer_info.key)?
        .assert_mut(|p| p.block_id == block.id)?;
    vault_base_info
        .is_writable()?
        .has_address(&vault_base_pda(block.id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_base_info.key)?
        .assert(|t| t.owner() == *market_info.key)?;
    // vault_base_info.as_associated_token_account(market_info.key, mint_base_info.key)?;
    vault_quote_info
        .is_writable()?
        .has_address(&vault_quote_pda(block.id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_quote_info.key)?
        .assert(|t| t.owner() == *market_info.key)?;
    // vault_quote_info.as_associated_token_account(market_info.key, mint_quote_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Load token acccounts.
    if tokens_base_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            signer_info,
            tokens_base_info,
            mint_base_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        tokens_base_info
            .is_writable()?
            .as_associated_token_account(signer_info.key, mint_base_info.key)?;
    }
    if tokens_quote_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            signer_info,
            tokens_quote_info,
            mint_quote_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        tokens_quote_info
            .is_writable()?
            .as_associated_token_account(signer_info.key, mint_quote_info.key)?;
    }

    // Update market state.
    let mut swap_event = market.swap(amount, direction, precision, clock)?;
    swap_event.authority = *signer_info.key;
    swap_event.block_id = block.id;

    // Get transfer amounts and accounts.
    let (in_amount, in_from, in_to, out_amount, out_from, out_to) = match direction {
        SwapDirection::Buy => (
            swap_event.quote_to_transfer,
            tokens_quote_info,
            vault_quote_info,
            swap_event.base_to_transfer,
            vault_base_info,
            tokens_base_info,
        ),
        SwapDirection::Sell => (
            swap_event.base_to_transfer,
            tokens_base_info,
            vault_base_info,
            swap_event.quote_to_transfer,
            vault_quote_info,
            tokens_quote_info,
        ),
    };

    // Update stake state.
    match direction {
        SwapDirection::Buy => {
            // TODO Fail if out_amount is zero
            stake.spend += in_amount;
        }
        SwapDirection::Sell => {
            stake.spend = stake.spend.saturating_sub(out_amount);
        }
    }

    // Assert utilization is not greater than capacity.
    if stake.spend > stake.collateral {
        panic!("spend is greater than collateral");
    }

    // Transfer tokens.
    transfer(signer_info, in_from, in_to, token_program, in_amount)?;
    transfer_signed(
        market_info,
        out_from,
        out_to,
        token_program,
        out_amount,
        &[MARKET, market.id.to_le_bytes().as_ref()],
    )?;

    // Validate vault reserves.
    let vault_base = vault_base_info.as_token_account()?; //.as_associated_token_account(market_info.key, mint_base_info.key)?;
    let vault_quote = vault_quote_info.as_token_account()?; //.as_associated_token_account(market_info.key, mint_quote_info.key)?;
    market.check_vaults(&vault_base, &vault_quote)?;

    // Emit event.
    swap_event.log_return();

    Ok(())
}
