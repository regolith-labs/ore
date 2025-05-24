use meteora_pools_sdk::instructions::{SwapCpi, SwapCpiAccounts, SwapInstructionArgs};
use ore_api::prelude::*;
use steel::*;

/// Swaps bets into ORE and buries the ORE.
pub fn process_bury(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let (required_accounts, meteora_accounts) = accounts.split_at(5);
    let [signer_info, block_info, block_bets_info, block_ore_info, bet_mint_info, ore_mint_info] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&INITIALIZER_ADDRESS)?;
    block_info.as_account::<Block>(&ore_api::ID)?;
    let block_bets =
        block_bets_info.as_associated_token_account(block_info.key, bet_mint_info.key)?;
    block_ore_info.as_associated_token_account(block_info.key, &MINT_ADDRESS)?;
    bet_mint_info.as_mint()?;
    ore_mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;

    // Load meteora accounts.
    let [pool_info, a_vault_info, b_vault_info, a_token_vault_info, b_token_vault_info, a_vault_lp_mint_info, b_vault_lp_mint_info, a_vault_lp_info, b_vault_lp_info, protocol_token_fee_info, vault_program_info, token_program_info, meteora_pools_program] =
        meteora_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    meteora_pools_program.is_program(&meteora_pools_sdk::programs::AMM_ID)?;

    // Execute swap.
    let swap = SwapCpi::new(
        &meteora_pools_program,
        SwapCpiAccounts {
            pool: pool_info,
            user_source_token: block_bets_info,
            user_destination_token: block_ore_info,
            a_vault: a_vault_info,
            b_vault: b_vault_info,
            a_token_vault: a_token_vault_info,
            b_token_vault: b_token_vault_info,
            a_vault_lp_mint: a_vault_lp_mint_info,
            b_vault_lp_mint: b_vault_lp_mint_info,
            a_vault_lp: a_vault_lp_info,
            b_vault_lp: b_vault_lp_info,
            protocol_token_fee: protocol_token_fee_info,
            user: block_info,
            vault_program: vault_program_info,
            token_program: token_program_info,
        },
        SwapInstructionArgs {
            in_amount: block_bets.amount(),
            minimum_out_amount: 0, // TODO: Calculate minimum out amount with slippage
        },
    );
    let block_bump = block_pda().1;
    swap.invoke_signed(&[&[BLOCK, &[block_bump]]])?;

    // Burn (bury) the purchased ORE.
    let block_ore = block_ore_info.as_associated_token_account(block_info.key, &MINT_ADDRESS)?;
    burn_signed_with_bump(
        block_ore_info,
        ore_mint_info,
        block_info,
        token_program_info,
        block_ore.amount(),
        &[BLOCK],
        block_bump,
    )?;

    // Emit an event.
    BuryEvent {
        amount: block_ore.amount(),
        ts: clock.unix_timestamp as u64,
    }
    .log();

    Ok(())
}
