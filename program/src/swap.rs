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
    let [signer_info, block_info, collateral_info, market_info, mint_base_info, mint_quote_info, stake_info, tokens_base_info, tokens_quote_info, vault_base_info, vault_quote_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block: &mut Block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot < b.start_slot)?;
    collateral_info
        .is_writable()?
        .as_associated_token_account(block_info.key, mint_quote_info.key)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.id == block.id)?
        .assert_mut(|m| m.base.liquidity() > 0)?
        .assert_mut(|m| m.quote.liquidity() > 0)?;
    mint_base_info.has_address(&market.base.mint)?.as_mint()?;
    mint_quote_info.has_address(&market.quote.mint)?.as_mint()?;
    stake_info
        .as_account_mut::<Stake>(&ore_api::ID)?
        .assert_mut(|p| p.authority == *signer_info.key)?
        .assert_mut(|p| p.block_id == block.id)?;
    vault_base_info
        .is_writable()?
        .as_associated_token_account(market_info.key, mint_base_info.key)?;
    vault_quote_info
        .is_writable()?
        .as_associated_token_account(market_info.key, mint_quote_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Load stake account.
    let stake = if stake_info.data_is_empty() {
        create_program_account::<Stake>(
            stake_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[STAKE, &signer_info.key.to_bytes(), &block.id.to_le_bytes()],
        )?;
        let stake = stake_info.as_account_mut::<Stake>(&ore_api::ID)?;
        stake.authority = *signer_info.key;
        stake.block_id = block.id;
        stake.capacity = 0;
        stake.utilization = 0;
        stake
    } else {
        stake_info
            .as_account_mut::<Stake>(&ore_api::ID)?
            .assert_mut(|p| p.authority == *signer_info.key)?
            .assert_mut(|p| p.block_id == block.id)?
    };

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
    let mut swap_result = market.swap(amount, direction, precision, clock)?;
    swap_result.authority = *signer_info.key;
    swap_result.block_id = block.id;
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

    // Update stake state.
    match direction {
        SwapDirection::Buy => {
            stake.utilization += in_amount;
        }
        SwapDirection::Sell => {
            stake.utilization = stake.utilization.saturating_sub(out_amount);
        }
    }

    // Assert utilization is not greater than capacity.
    if stake.utilization > stake.capacity {
        panic!("utilization is greater than capacity");
    }

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
