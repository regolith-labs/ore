use fpow_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol};
use steel::*;

/// Reloads ALGO rewards.
pub fn process_reload_algo(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, automation_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let automation = automation_info
        .as_account_mut::<Automation>(&fpow_api::ID)?
        .assert_mut(|a| a.executor == *signer_info.key)?
        .assert_mut(|a| a.reload > 0)?;
    let miner = miner_info
        .as_account_mut::<Miner>(&fpow_api::ID)?
        .assert_mut(|m| m.authority == automation.authority)?;
    system_program.is_program(&system_program::ID)?;

    // Claim algo from the miner.
    let amount = miner.claim_algo(&clock);

    // Increment automation balance.
    automation.balance += amount;

    // Transfer ALGO to automation.
    miner_info.send(amount, automation_info);

    // Log
    sol_log(&format!("Reloading {} ALGO", lamports_to_sol(amount)).as_str());

    Ok(())
}
