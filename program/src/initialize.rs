use ore_api::prelude::*;
use steel::*;

/// Initialize the program.
pub fn process_initialize(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, block_info, block_bets_info, block_ore_info, ore_mint_info, sol_mint_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN_ADDRESS)?;
    block_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BLOCK], &ore_api::ID)?;
    block_bets_info.is_empty()?.is_writable()?;
    block_ore_info.is_empty()?.is_writable()?;
    ore_mint_info.has_address(&MINT_ADDRESS)?;
    sol_mint_info.has_address(&spl_token::native_mint::ID)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Initialize config.
    create_program_account::<Block>(
        block_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[BLOCK],
    )?;
    let block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
    block.cumulative_sum = 0;
    block.current_round = 0;
    block.ends_at = 0;
    block.mint = spl_token::native_mint::ID;
    block.noise = [0; 32];
    block.payed_out = 0;
    block.reward = 0;
    block.started_at = 0;
    block.total_wagers = 0;

    // Initialize block token accounts.
    create_associated_token_account(
        signer_info,
        block_info,
        block_bets_info,
        sol_mint_info,
        system_program,
        token_program,
        associated_token_program,
    )?;
    create_associated_token_account(
        signer_info,
        block_info,
        block_ore_info,
        ore_mint_info,
        system_program,
        token_program,
        associated_token_program,
    )?;

    Ok(())
}
