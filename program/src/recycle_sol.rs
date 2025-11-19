use ore_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol, pubkey};
use steel::*;

const TESTER: Pubkey = pubkey!("pqspJ298ryBjazPAr95J9sULCVpZe3HbZTWkbC1zrkS");

/// Claims a block reward.
pub fn process_recycle_sol(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, automation_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let automation = automation_info
        .as_account_mut::<Automation>(&ore_api::ID)?
        .assert_mut(|a| a.executor == *signer_info.key)?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == automation.authority)?;
    system_program.is_program(&system_program::ID)?;

    // Whitelist feature
    assert_eq!(*signer_info.key, TESTER);

    // Increment automation balance.
    let amount = miner.claim_sol(&clock);
    automation.balance += amount;

    // Transfer SOL to automation.
    miner_info.send(amount, automation_info);

    // Log
    sol_log(&format!("Recycling {} SOL", lamports_to_sol(amount)).as_str());

    Ok(())
}
