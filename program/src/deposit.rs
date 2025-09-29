use ore_api::prelude::*;
use solana_program::log::sol_log;
use spl_token::amount_to_ui_amount;
use steel::*;

use crate::AUTHORIZED_ACCOUNTS;

/// Deposits ORE into the staking contract.
pub fn process_deposit(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Deposit::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, sender_info, stake_info, treasury_info, treasury_tokens_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let sender = sender_info
        .is_writable()?
        .as_associated_token_account(&signer_info.key, &MINT_ADDRESS)?;
    stake_info.is_writable()?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info
        .is_writable()?
        .as_associated_token_account(&treasury_info.key, &MINT_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Check whitelist
    if !AUTHORIZED_ACCOUNTS.contains(&signer_info.key) {
        return Err(trace("Not authorized", OreError::NotAuthorized.into()));
    }

    // Open stake account.
    let stake = if stake_info.data_is_empty() {
        create_program_account::<Stake>(
            stake_info,
            system_program,
            &signer_info,
            &ore_api::ID,
            &[STAKE, &signer_info.key.to_bytes()],
        )?;
        let stake = stake_info.as_account_mut::<Stake>(&ore_api::ID)?;
        stake.authority = *signer_info.key;
        stake.balance = 0;
        stake.last_claim_at = 0;
        stake.last_deposit_at = 0;
        stake.last_withdraw_at = 0;
        stake.is_seeker = 0;
        stake.rewards_factor = treasury.rewards_factor;
        stake.rewards = 0;
        stake.lifetime_rewards = 0;
        stake
    } else {
        stake_info
            .as_account_mut::<Stake>(&ore_api::ID)?
            .assert_mut(|s| s.authority == *signer_info.key)?
    };

    // Only allow deposits from seekers.
    assert!(stake.is_seeker == 1, "Only seekers can deposit stake");

    // Deposit into stake account.
    let amount = stake.deposit(amount, &clock, treasury, &sender);

    // Transfer ORE to treasury.
    transfer(
        signer_info,
        sender_info,
        treasury_tokens_info,
        token_program,
        amount,
    )?;

    // Log deposit.
    sol_log(
        &format!(
            "Depositing {} ORE",
            amount_to_ui_amount(amount, TOKEN_DECIMALS)
        )
        .as_str(),
    );

    Ok(())
}
