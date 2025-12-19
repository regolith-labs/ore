use ore_api::prelude::*;
use solana_program::log::sol_log;
use solana_program::native_token::lamports_to_sol;
use solana_program::pubkey;
use steel::*;

const LIQ_MANAGER: Pubkey = pubkey!("DJqfQWB8tZE6fzqWa8okncDh7ciTuD8QQKp1ssNETWee");

/// Send SOL to the liq manager.
pub fn process_liq(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, board_info, _config_info, manager_info, manager_sol_info, treasury_info, treasury_sol_info, token_program, ore_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&BURY_AUTHORITY)?;
    board_info.as_account_mut::<Board>(&ore_api::ID)?;
    manager_info.has_address(&LIQ_MANAGER)?;
    manager_sol_info
        .is_writable()?
        .as_associated_token_account(&manager_info.key, &SOL_MINT)?;
    treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_sol_info.as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Sync native token balance.
    sync_native(treasury_sol_info)?;

    // Record pre-swap balances.
    let treasury_sol =
        treasury_sol_info.as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    let liq_amount = treasury_sol.amount();
    assert!(liq_amount > 0);

    // Build swap accounts.
    transfer_signed(
        treasury_info,
        treasury_sol_info,
        manager_sol_info,
        token_program,
        liq_amount,
        &[TREASURY],
    )?;

    // Record post-swap balances.
    let treasury_sol =
        treasury_sol_info.as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    assert_eq!(treasury_sol.amount(), 0);
    sol_log(&format!("💦 Sent {} SOL to liq manager", lamports_to_sol(liq_amount)).as_str());

    // Emit event.
    program_log(
        &[board_info.clone(), ore_program.clone()],
        LiqEvent {
            disc: 3,
            sol_amount: liq_amount,
            recipient: *manager_info.key,
            ts: Clock::get()?.unix_timestamp,
        }
        .to_bytes(),
    )?;

    Ok(())
}
