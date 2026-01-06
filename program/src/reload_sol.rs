use ore_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol};
use steel::*;

/// Claims a block reward.
pub fn process_reload_sol(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, automation_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let automation = automation_info
        .as_account_mut::<Automation>(&ore_api::ID)?
        .assert_mut(|a| a.executor == *signer_info.key || a.executor == EXECUTOR_ADDRESS)?
        .assert_mut(|a| a.reload > 0)?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == automation.authority)?;
    system_program.is_program(&system_program::ID)?;

    // Claim sol from the miner.
    let amount = miner.claim_sol(&clock);

    // Increment automation balance.
    automation.balance += amount;

    // Transfer SOL to automation.
    miner_info.send(amount, automation_info);

    // Log
    sol_log(&format!("Reloading {} SOL", lamports_to_sol(amount)).as_str());

    Ok(())
}
