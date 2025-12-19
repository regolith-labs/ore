use fpow_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol};
use steel::*;

/// Claims ALGO rewards.
pub fn process_claim_algo(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&fpow_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)?;
    system_program.is_program(&system_program::ID)?;

    // Normalize amount.
    let amount = miner.claim_algo(&clock);

    sol_log(&format!("Claiming {} ALGO", lamports_to_sol(amount)).as_str());

    // Transfer reward to recipient.
    miner_info.send(amount, signer_info);

    Ok(())
}
