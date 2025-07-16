use ore_api::{prelude::*, sdk::program_log};
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
    let [signer_info, block_info, market_info, mint_quote_info, tokens_info, vault_info, system_program, token_program, associated_token_program, ore_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block: &mut Block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
    // .assert_mut(|b| clock.slot < b.start_slot)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.block_id == block.id)?
        .assert_mut(|m| m.base.liquidity() > 0)?
        .assert_mut(|m| m.quote.liquidity() > 0)?;
    mint_quote_info.has_address(&market.quote.mint)?.as_mint()?;
    vault_info
        .is_writable()?
        .has_address(&vault_pda().0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_quote_info.key)?
        .assert(|t| t.owner() == *market_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Load token acccounts.
    if tokens_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            signer_info,
            tokens_info,
            mint_quote_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        tokens_info
            .is_writable()?
            .as_associated_token_account(signer_info.key, mint_quote_info.key)?;
    }

    // Update market state.
    let mut swap_event = market.swap(amount, direction, precision, clock)?;
    swap_event.authority = *signer_info.key;
    swap_event.block_id = block.id;

    // Transfer tokens
    match direction {
        SwapDirection::Buy => {
            transfer(
                signer_info,
                tokens_info,
                vault_info,
                token_program,
                swap_event.quote_to_transfer,
            )?;
        }
        SwapDirection::Sell => {
            transfer_signed(
                market_info,
                vault_info,
                tokens_info,
                token_program,
                swap_event.quote_to_transfer,
                &[MARKET],
            )?;
        }
    };

    // Validate vault reserves.
    // let vault_base = vault_base_info.as_token_account()?;
    // let vault_quote = vault_quote_info.as_token_account()?;
    // market.check_vaults(&vault_base, &vault_quote)?;

    // Emit event.
    program_log(
        block.id,
        &[block_info.clone(), ore_program.clone()],
        &swap_event.to_bytes(),
    )?;

    Ok(())
}
