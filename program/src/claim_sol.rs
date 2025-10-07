use ore_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol};
use steel::*;

/// Claims a block reward.
pub fn process_claim_sol(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    panic!("program is currently in migration mode");

    // Parse data.
    let args = ClaimSOL::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)?;
    system_program.is_program(&system_program::ID)?;

    // Normalize amount.
    let amount = miner.rewards_sol.min(amount);

    sol_log(&format!("Claiming {} SOL", lamports_to_sol(amount)).as_str());

    // Update miner.
    miner.rewards_sol -= amount;

    // Transfer reward to recipient.
    miner_info.send(amount, signer_info);

    Ok(())
}
