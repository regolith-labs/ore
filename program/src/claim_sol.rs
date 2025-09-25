use ore_api::prelude::*;
use solana_program::{log::sol_log, native_token::lamports_to_sol};
use steel::*;

/// Claims a block reward.
pub fn process_claim_sol(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
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
    let total_claimable = miner.rewards_sol + miner.refund_sol;
    let amount = total_claimable.min(amount);

    sol_log(&format!("Claiming {} SOL", lamports_to_sol(amount)).as_str());

    // Update miner. Deduct from refund first, then from rewards.
    let from_refund = amount.min(miner.refund_sol);
    miner.refund_sol -= from_refund;
    let from_rewards = amount - from_refund;
    miner.rewards_sol -= from_rewards;

    // Transfer reward to recipient.
    miner_info.send(amount, signer_info);

    Ok(())
}
